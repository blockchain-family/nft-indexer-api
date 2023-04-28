mod nft;
pub use self::nft::*;

mod auction;
pub use self::auction::*;

mod events;
pub use self::events::*;

mod collection;
pub use self::collection::*;

mod owner;
pub use self::owner::*;

mod metrics;

pub use self::metrics::*;

#[macro_export]
macro_rules! catch_error {
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

use std::convert::Infallible;
use std::fmt::Display;
use warp::{
    http::{Response, StatusCode},
    Filter,
};

use crate::db::Queries;
use crate::model::{Root, Roots};
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    static ref SWAGGER: Vec<u8> = {
        std::fs::read("openapi.yml").expect("cannot read 'openapi.yml' from disk")
    };
}

/// GET /swagger
pub fn get_swagger() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
{
    warp::path!("swagger")
        .and(warp::get())
        .and_then(get_swagger_handler)
}

async fn get_swagger_handler() -> Result<impl warp::Reply, Infallible> {
    Ok(Box::from(warp::reply::with_status(
        Response::builder()
            .header("Content-Type", "application/yaml")
            .body::<&[u8]>(SWAGGER.as_ref()),
        StatusCode::OK,
    )))
}

/// POST /roots
pub fn list_roots(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("roots")
        .and(warp::get())
        .and(warp::any().map(move || db.clone()))
        .and_then(list_roots_handler)
}

pub async fn list_roots_handler(db: Queries) -> Result<Box<dyn warp::Reply>, Infallible> {
    let list = catch_error!(db.list_roots().await);

    let roots: Vec<Root> = list.into_iter().map(Root::from).collect();
    response!(&Roots { roots })
}

#[derive(Clone, Deserialize, Serialize, Hash)]
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
