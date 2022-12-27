use serde::Deserialize;
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use config::{self, Environment, ConfigError};
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    Error,
};

fn default_http_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080))
}

fn default_url() -> String {
    String::from("postgresql://localhost/nft_indexer")
}

fn default_max_connections() -> u32 {
    1
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
}

impl ApiConfig {
    pub fn new() -> Result<ApiConfig, ConfigError> {
        let prefix = std::env::var("PREFIX").unwrap_or_else(|_| String::from("indexer_api"));
        let r = config::Config::builder()
            .add_source(Environment::with_prefix(&prefix).separator("_"))
            .build()?
            .try_deserialize();
        r
        // if r.is_err() {
        //     config::Config::builder()
        //         .add_source(config::File::with_name("./Settings.toml"))
        //         .build()?
        //         .try_deserialize()
        // } else { r }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        ApiConfig { 
            http_address: default_http_address(),
            database: DatabaseConfig::default(),
        }
    }
}

