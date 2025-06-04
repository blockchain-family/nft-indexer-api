pub mod api_doc;
pub mod auction;
pub mod auth;
pub mod collection;
pub mod collection_custom;
pub mod events;
pub mod metadata;
pub mod metrics;
pub mod nft;
pub mod owner;
pub mod requests;
pub mod service;
pub mod swagger;
pub mod user;

use crate::db::queries::Queries;
use crate::services::auth::AuthService;
use moka::future::Cache;
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[macro_export]
macro_rules! catch_error_500 {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                use axum::response::IntoResponse;
                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                    .into_response();
            }
        }
    };
}

#[macro_export]
macro_rules! catch_error_400 {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                use axum::response::IntoResponse;
                return (axum::http::StatusCode::BAD_REQUEST, e.to_string()).into_response();
            }
        }
    };
}

#[macro_export]
macro_rules! catch_error_401 {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                use axum::response::IntoResponse;
                return (axum::http::StatusCode::UNAUTHORIZED, e.to_string()).into_response();
            }
        }
    };
}

#[macro_export]
macro_rules! catch_error_403 {
    ($expr:expr) => {
        match $expr {
            Some(val) => val,
            None => {
                use axum::response::IntoResponse;
                return (axum::http::StatusCode::FORBIDDEN, "Forbidden action").into_response();
            }
        }
    };
}

#[macro_export]
macro_rules! catch_empty {
    ($expr:expr, $err:expr) => {
        match $expr {
            Some(val) => val,
            None => {
                use axum::response::IntoResponse;
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    $err.to_string(),
                )
                    .into_response();
            }
        }
    };
}

#[macro_export]
macro_rules! response {
    ($ret:expr) => {{
        use axum::response::IntoResponse;
        (axum::http::StatusCode::OK, axum::Json($ret)).into_response()
    }};
}

pub struct HttpState {
    pub db: Queries,
    pub cache_minute: Cache<u64, Value>,
    pub cache_5_minutes: Cache<u64, Value>,
    pub cache_10_sec: Cache<u64, Value>,
    pub cache_1_sec: Cache<u64, Value>,
    pub auth_service: AuthService,
    pub indexer_api_url: String,
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
