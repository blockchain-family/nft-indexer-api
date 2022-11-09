use serde::Serialize;
use std::{collections::HashMap, str::FromStr, time::Duration};
use crate::db::{Queries, TokenUsdPrice};
use sqlx::types::{BigDecimal, chrono::{NaiveDateTime, Local}};

#[derive(Debug, Clone)]
pub struct CurrencyClient {
    http_client: reqwest::Client,
    db: Queries,
}

#[derive(Debug, Clone, Serialize)]
pub struct TokenUsdPricesRequest {
    pub currency_addresses: Vec<String>,
}

pub type TokenUsdPricesResponse = HashMap<String, String>;

impl CurrencyClient {
    pub fn new(db: Queries) -> reqwest::Result<Self> {
        let http_client = reqwest::Client::builder().build()?;
        Ok(CurrencyClient{ http_client, db })
    }

    pub async fn get_prices(&self) -> reqwest::Result<TokenUsdPricesResponse> {
        self.http_client.post("https://api.flatqube.io/v1/currencies_usdt_prices")
            .json(&TokenUsdPricesRequest{
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
        let db_prices = prices.iter()
            .map(|(token, price)| TokenUsdPrice {
                ts,
                token: token.clone(),
                usd_price: BigDecimal::from_str(price).unwrap_or_default(),
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
                tokio::time::sleep(tokio::time::Duration::from_secs(5 * 60)).await;
            }
        });
        Ok(())
    }
}