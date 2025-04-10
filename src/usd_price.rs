use crate::db::queries::Queries;
use crate::db::TokenUsdPrice;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Local;
use std::{collections::HashMap, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct CurrencyClient {
    http_client: reqwest::Client,
    db: Queries,
    main_token: String,
    dex_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenUsdPricesRequest {
    pub currency_addresses: Vec<String>,
}

pub type TokenUsdPricesResponse = HashMap<String, String>;

impl CurrencyClient {
    pub fn new(db: Queries, main_token: String, dex_url: String) -> reqwest::Result<Self> {
        let http_client = reqwest::Client::builder().build()?;
        Ok(CurrencyClient {
            http_client,
            db,
            main_token,
            dex_url,
        })
    }

    pub async fn get_prices(&self) -> reqwest::Result<TokenUsdPricesResponse> {
        self.http_client
            .post(format!("{}/v1/currencies_usdt_prices", self.dex_url))
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
        let ts = DateTime::from_timestamp(Local::now().timestamp(), 0)
            .expect("Failed to get time")
            .naive_utc();
        let db_prices = prices
            .iter()
            .map(|(token, price)| {
                let decimals = self.db.tokens.get(token).expect("unknown token").decimals;
                let scale = BigDecimal::from(10_i64.pow(decimals));
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
                let price = self.get_prices_venom_dex(&self.main_token).await;
                match price {
                    Ok(price) => {
                        let price = TokenUsdPrice {
                            token: self.main_token.to_string(),
                            usd_price: price.price / BigDecimal::from(10_i64.pow(9)),
                            ts: Utc::now().naive_utc(),
                        };
                        if let Err(e) = self.db.update_token_usd_prices(vec![price]).await {
                            log::error!("usd prices update db error: {e}");
                        }
                    }
                    Err(e) => log::error!("get venom price error: {e}"),
                }
                tokio::time::sleep(Duration::from_secs(5 * 60)).await;
            }
        });
        Ok(())
    }

    async fn get_prices_venom_dex(&self, token: &str) -> reqwest::Result<VenomDexPriceResponse> {
        let url = format!("{}/v1/currencies/{token}", self.dex_url);
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
    pub price: BigDecimal,
}
