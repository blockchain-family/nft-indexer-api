pub mod nft;
pub use self::nft::*;

pub mod auction;
pub use self::auction::*;

pub mod events;
pub use self::events::*;

pub mod collection;
pub use self::collection::*;

pub mod owner;
pub use self::owner::*;

pub use self::metrics::*;
pub mod metrics;

pub use self::auth::*;
pub mod auth;

pub use self::user::*;
pub mod user;
use warp::http::StatusCode;

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

use std::convert::Infallible;

use crate::db::queries::Queries;
use crate::model::{Root, Roots};

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
