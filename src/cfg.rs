use std::net::SocketAddr;

use config::{self};
use serde::Deserialize;
use sqlx::Error;
use sqlx::postgres::{PgPool, PgPoolOptions};

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
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

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub http_address: SocketAddr,
    pub database: DatabaseConfig,
    pub auth_token_lifetime: u32,
    pub jwt_secret: String,
    pub base_url: String,
    pub dex_url: String,
    pub main_token: String,
    pub indexer_api_url: String,
    pub token_manifest_path: String,
    pub service_name: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiConfig {
    pub fn new() -> ApiConfig {
        let mut conf_builder = config::Config::builder().add_source(
            config::Environment::default()
                .separator("__")
                .try_parsing(true),
        );

        if std::path::Path::new("Settings.toml").exists() {
            conf_builder = conf_builder.add_source(config::File::with_name("./Settings.toml"));
        }

        conf_builder
            .build()
            .expect("Failed to build config")
            .try_deserialize::<ApiConfig>()
            .unwrap_or_else(|e| panic!("Error parsing config: {e}"))
    }
}
