use config::{self, ConfigError, Environment};
use serde::Deserialize;
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    Error,
};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

fn default_http_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080))
}

fn default_url() -> String {
    String::from("postgresql://localhost/nft_indexer")
}

fn default_max_connections() -> u32 {
    50
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "default_url")]
    pub url: String,

    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub async fn init(&self) -> Result<PgPool, Error> {
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .connect(&self.url)
            .await
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            url: default_url(),
            max_connections: default_max_connections(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    #[serde(default = "default_http_address")]
    pub http_address: SocketAddr,
    pub database: DatabaseConfig,
    pub auth_token_lifetime: u32,
    pub jwt_secret: String,
    pub base_url: String,
    pub prices_url: String,
    pub main_token: String,
}

impl ApiConfig {
    pub fn new() -> Result<ApiConfig, ConfigError> {
        let prefix = std::env::var("PREFIX").unwrap_or_else(|_| String::from("indexer_api"));
        config::Config::builder()
            .add_source(Environment::with_prefix(&prefix).separator("__"))
            .build()?
            .try_deserialize()
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig {
            http_address: default_http_address(),
            database: DatabaseConfig::default(),
            auth_token_lifetime: 999999999,
            jwt_secret: "jwtsecret".to_string(),
            base_url: String::default(),
            prices_url: "".to_string(),
            main_token: "".to_string(),
        }
    }
}
