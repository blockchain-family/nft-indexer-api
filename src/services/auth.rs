use std::borrow::Cow;
use super::error::Error;
use crate::model::{JwtClaims, LoginData};
use base64::engine::general_purpose;
use base64::Engine;
use ed25519_dalek::{PublicKey, Verifier};
use http::{HeaderMap, HeaderValue};
use nekoton::core::ton_wallet::compute_address;
use nekoton::core::ton_wallet::WalletType;
use sha2::Digest;
use std::str::FromStr;
use std::time::SystemTime;
use ton_block::MsgAddressInt;

pub struct AuthService {
    access_token_lifetime: u32,
    jwt_secret: String,
    base_url: String,
}

impl AuthService {
    pub fn new(access_token_lifetime: u32, jwt_secret: String, base_url: String) -> Self {
        Self {
            access_token_lifetime,
            jwt_secret,
            base_url,
        }
    }

    pub fn authenticate(&self, headers: HeaderMap<HeaderValue>) -> anyhow::Result<String> {
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

        match self.jwt_from_header(&headers) {
            Ok(jwt) => {
                let mut validation = Validation::new(Algorithm::default());
                validation.leeway = 2;

                let decoded = decode::<JwtClaims>(
                    &jwt,
                    &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
                    &validation,
                )
                .map_err(|_| Error::JwtToken)?;

                Ok(decoded.claims.sub)
            }
            Err(e) => anyhow::bail!(e),
        }
    }

    pub fn authorize(&self, login: LoginData) -> anyhow::Result<String> {
        let public_key = Self::parse_public_key(login.public_key.as_str())?;
        let wallet_type = WalletType::from_str(login.wallet_type.as_str())?;

        Self::ensure_right_address(login.address.clone(), public_key, wallet_type)?;
        Self::ensure_correct_signature(
            public_key,
            login.address.as_str(),
            login.signature.as_str(),
            login.timestamp,
            self.base_url.as_str(),
            login.with_signature_id
        )?;
        self.ensure_not_expired(login.timestamp)?;

        Ok(self.create_jwt(login.address.as_str()))
    }

    fn ensure_correct_signature(
        public_key: PublicKey,
        address: &str,
        signature: &str,
        timestamp: u64,
        base_url: &str,
        with_signature_id: Option<i32>
    ) -> anyhow::Result<()> {
        let msg = format!("I want to login at {base_url} with address {address} at {timestamp}");
        let mut hasher = sha2::Sha256::new();
        hasher.update(msg);
        let msg_hash = format!("{:X}", hasher.finalize());

        if !Self::verify_signature(public_key, msg_hash.as_str(), signature, with_signature_id)? {
            anyhow::bail!("Bad signature");
        }

        Ok(())
    }

    fn ensure_right_address(
        address: String,
        public_key: PublicKey,
        wallet_type: WalletType,
    ) -> anyhow::Result<()> {
        let workchain_id = Self::extract_address_workchain(address.as_str())?;
        let computed_address = compute_address(&public_key, wallet_type, workchain_id).to_string();
        if address != computed_address {
            anyhow::bail!("Bad address");
        }

        Ok(())
    }

    fn extract_address_workchain(address: &str) -> anyhow::Result<i8> {
        let address = match MsgAddressInt::from_str(address) {
            Ok(address) => address,
            Err(_) => match nekoton_utils::unpack_std_smc_addr(address, false) {
                Ok(address) => address,
                Err(_) => match nekoton_utils::unpack_std_smc_addr(address, true) {
                    Ok(address) => address,
                    Err(_) => anyhow::bail!("Failed to parse the address"),
                },
            },
        };
        Ok(address.workchain_id() as i8)
    }

    fn ensure_not_expired(&self, timestamp: u64) -> anyhow::Result<()> {
        let now = Self::get_sys_time_in_secs();
        if timestamp > now {
            anyhow::bail!("Timestamp from the future")
        }

        let expiration = timestamp + self.access_token_lifetime as u64;
        if expiration < now {
            anyhow::bail!("Login expired");
        }

        Ok(())
    }

    fn verify_signature(
        public_key: PublicKey,
        data_hash: &str,
        signature: &str,
        signature_id: Option<i32>,
    ) -> anyhow::Result<bool> {
        let data_hash = hex::decode(data_hash)?;
        if data_hash.len() != 32 {
            anyhow::bail!("Invalid data hash. Expected 32 bytes")
        }

        let data_hash = Self::extend_data_with_signature_id(&data_hash, signature_id);

        let signature = match general_purpose::STANDARD.decode(signature) {
            Ok(signature) => signature,
            Err(_) => hex::decode(signature)?,
        };

        let signature = match ed25519_dalek::Signature::try_from(signature.as_slice()) {
            Ok(signature) => signature,
            Err(_) => anyhow::bail!("Invalid signature. Expected 64 bytes"),
        };

        Ok(public_key.verify(&data_hash, &signature).is_ok())
    }

    pub fn extend_data_with_signature_id(data: &[u8], signature_id: Option<i32>) -> Cow<'_, [u8]> {
        match signature_id {
            Some(signature_id) => {
                let mut result = Vec::with_capacity(4 + data.len());
                result.extend_from_slice(&signature_id.to_be_bytes());
                result.extend_from_slice(data);
                Cow::Owned(result)
            }
            None => Cow::Borrowed(data),
        }
    }


    fn parse_public_key(public_key: &str) -> anyhow::Result<PublicKey> {
        Ok(PublicKey::from_bytes(&hex::decode(public_key)?)?)
    }

    pub fn create_jwt(&self, address: &str) -> String {
        let expiration = Self::get_sys_time_in_secs() + self.access_token_lifetime as u64;

        let claims = JwtClaims {
            sub: address.to_owned(),
            exp: expiration as usize,
        };

        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .expect("Failed to encode JWT")
    }

    fn get_sys_time_in_secs() -> u64 {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(e) => panic!("Get sys time error {e}!"),
        }
    }

    fn jwt_from_header(&self, headers: &HeaderMap<HeaderValue>) -> anyhow::Result<String> {
        const BEARER: &str = "Bearer ";

        let header = match headers.get(http::header::AUTHORIZATION) {
            Some(v) => v,
            None => anyhow::bail!(Error::NoAuthHeader),
        };
        let auth_header = match std::str::from_utf8(header.as_bytes()) {
            Ok(v) => v,
            Err(_) => anyhow::bail!(Error::NoAuthHeader),
        };
        if !auth_header.starts_with(BEARER) {
            anyhow::bail!(Error::InvalidAuthHeader);
        }
        Ok(auth_header.trim_start_matches(BEARER).to_owned())
    }
}
