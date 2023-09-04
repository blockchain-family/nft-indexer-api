pub mod nft;
use std::collections::hash_map::DefaultHasher;
pub mod auction;
pub mod auth;
pub mod collection;
pub mod events;
pub mod metrics;
pub mod owner;
pub mod user;
use utoipa::ToSchema;
#[macro_export]
macro_rules! catch_error_500 {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                return Ok(Box::from(warp::reply::with_status(
                    e.to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )));
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
                return Ok(Box::from(warp::reply::with_status(
                    e.to_string(),
                    StatusCode::BAD_REQUEST,
                )));
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
                return Ok(Box::from(warp::reply::with_status(
                    $err,
                    StatusCode::INTERNAL_SERVER_ERROR,
                )));
            }
        }
    };
}

#[macro_export]
macro_rules! response {
    ($ret:expr) => {
        Ok(Box::from(warp::reply::with_status(
            warp::reply::json(&$ret),
            warp::http::StatusCode::OK,
        )))
    };
}

use std::fmt::Display;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref SWAGGER: Vec<u8> = {
        std::fs::read("openapi.yml").expect("cannot read 'openapi.yml' from disk")
    };
}

#[macro_export]
macro_rules! api_doc_addon {
    ($addon:ty) => {
        use utoipa::openapi::Components;
        use utoipa::Modify;
        pub struct ApiDocAddon;
        impl Modify for ApiDocAddon {
            fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
                let mut addon = <$addon>::openapi();
                openapi.paths.paths.append(&mut addon.paths.paths);
                if openapi.tags.is_none() {
                    openapi.tags = Some(vec![]);
                }
                if let Some(tags) = openapi.tags.as_mut() {
                    if let Some(addon_tags) = addon.tags.as_mut() {
                        tags.append(addon_tags);
                    }
                }
                if openapi.components.is_none() {
                    openapi.components = Some(Components::new());
                }
                if let Some(components) = openapi.components.as_mut() {
                    if let Some(addon_components) = addon.components.as_mut() {
                        components.schemas.append(&mut addon_components.schemas);
                    }
                }
            }
        }
    };
}

use crate::db::queries::Queries;
use crate::model::{Root, Roots};
use reqwest::StatusCode;
use std::convert::Infallible;

use utoipa::OpenApi;
use warp::Filter;

#[derive(OpenApi)]
#[openapi(
    paths(list_roots),
    components(schemas(Roots, Root)),
    tags(
        (name = "service", description = "Service handlers"),
    )
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[utoipa::path(
    get,
    tag = "service",
    path = "/roots",
    responses(
        (status = 200, body = Roots),
        (status = 500),
    ),
)]
pub fn list_roots(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("roots")
        .and(warp::get())
        .and(warp::any().map(move || db.clone()))
        .and_then(list_roots_handler)
}

pub async fn list_roots_handler(db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let list = catch_error_500!(db.list_roots().await);
    let roots: Vec<Root> = list.into_iter().map(Root::from).collect();
    response!(&Roots { roots })
}

#[derive(Clone, Deserialize, Serialize, Hash, ToSchema)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

impl Display for OrderDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderDirection::Asc => write!(f, "asc"),
            OrderDirection::Desc => write!(f, "desc"),
        }
    }
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
