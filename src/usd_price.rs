use crate::db::{Queries, TokenUsdPrice};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::types::{
    chrono::{Local, NaiveDateTime},
    BigDecimal,
};
use std::{collections::HashMap, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct CurrencyClient {
    http_client: reqwest::Client,
    db: Queries,
    pub chain: String,
    pub venom_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenUsdPricesRequest {
    pub currency_addresses: Vec<String>,
}

pub type TokenUsdPricesResponse = HashMap<String, String>;

impl CurrencyClient {
    pub fn new(db: Queries, chain: String, venom_token: String) -> reqwest::Result<Self> {
        let http_client = reqwest::Client::builder().build()?;
        Ok(CurrencyClient {
            http_client,
            db,
            chain,
            venom_token,
        })
    }

    pub async fn get_prices(&self) -> reqwest::Result<TokenUsdPricesResponse> {
        self.http_client
            .post("https://api.flatqube.io/v1/currencies_usdt_prices")
            .json(&TokenUsdPricesRequest {
                currency_addresses: self.db.tokens.addresses(),
            })
            .send()
            .await?
            .json::<TokenUsdPricesResponse>()
            .await
    }

    pub async fn update_prices(&self) -> anyhow::Result<()> {
        let prices = self.get_prices().await?;
        log::debug!("update_prices: {:?}", prices);
        let ts: NaiveDateTime = NaiveDateTime::from_timestamp(Local::now().timestamp(), 0);
        let ten: i64 = 10;
        let db_prices = prices
            .iter()
            .map(|(token, price)| {
                let decimals = self.db.tokens.get(token).expect("unknown token").decimals as u32;
                let scale = BigDecimal::from(ten.pow(decimals));
                let usd_price = BigDecimal::from_str(price).unwrap_or_default() / scale;
                TokenUsdPrice {
                    ts,
                    usd_price,
                    token: token.clone(),
                }
            })
            .collect();
        self.db.update_token_usd_prices(db_prices).await?;
        Ok(())
    }

    pub async fn start(self, _period: Duration) -> anyhow::Result<()> {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.update_prices().await {
                    log::error!("usd prices update task error: {}", e);
                }
                if self.chain == "venom" {
                    let price = self.get_prices_venom_dex(self.venom_token.as_str()).await;
                    match price {
                        Ok(price) => {
                            let price = TokenUsdPrice {
                                token: self.venom_token.to_string(),
                                usd_price: price.price / BigDecimal::from(10_i64.pow(9)),
                                ts: Utc::now().naive_utc(),
                            };
                            if let Err(e) = self.db.update_token_usd_prices(vec![price]).await {
                                log::error!("usd prices update db error: {e}");
                            }
                        }
                        Err(e) => log::error!("get venom price error: {e}"),
                    }
                }
                tokio::time::sleep(Duration::from_secs(5 * 60)).await;
            }
        });
        Ok(())
    }

    async fn get_prices_venom_dex(&self, token: &str) -> reqwest::Result<VenomDexPriceResponse> {
        let url = format!("https://testnetapi.web3.world/v1/currencies/{}", token);
        self.http_client
            .post(url)
            .send()
            .await?
            .json::<VenomDexPriceResponse>()
            .await
    }
}

#[derive(Deserialize)]
struct VenomDexPriceResponse {
    currency: String,
    address: String,
    price: BigDecimal,
}
