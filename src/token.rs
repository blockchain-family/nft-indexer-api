use crate::db::Address;
use serde::Deserialize;
use sqlx::types::BigDecimal;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Token {
    #[serde(rename = "chainId")]
    pub chain_id: usize,
    pub address: Address,
    pub name: String,
    pub symbol: String,
    pub vendor: Option<String>,
    #[serde(rename = "logoURI")]
    pub logo_uri: String,
    pub decimals: usize,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct TokenDict(Arc<HashMap<Address, Token>>);

impl TokenDict {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut map = HashMap::new();
        for token in tokens {
            map.insert(token.address.clone(), token);
        }
        TokenDict(Arc::new(map))
    }

    pub async fn load() -> anyhow::Result<Self> {
        let resp = reqwest::get(
            "https://raw.githubusercontent.com/broxus/ton-assets/master/manifest.json",
        )
        .await?
        .json::<Manifest>()
        .await?;
        Ok(Self::new(resp.tokens))
    }

    pub fn get(&self, token: &String) -> Option<&Token> {
        self.0.get(token)
    }

    pub fn format_value(&self, _token: &str, val: &BigDecimal) -> String {
        let s = val.round(0).to_string();
        /*if let Some(t) = self.0.get(token) {
            if s.len() > t.decimals {
                s.insert(s.len() - t.decimals, '.')
            } else if s.len() == t.decimals {
                let mut prefix = "0.".to_string();
                prefix.push_str(&s);
                s = prefix;
            }
            s = s.trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if s.len() == 0 {
                s = "0".to_string();
            }
        }
        s*/
        s
    }

    pub fn addresses(&self) -> Vec<String> {
        self.0.keys().map(Clone::clone).collect()
    }
}
