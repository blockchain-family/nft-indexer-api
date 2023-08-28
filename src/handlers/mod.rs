use std::collections::hash_map::DefaultHasher;

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

pub mod metrics;
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

use crate::db::Queries;
use crate::docs;
use crate::model::{Root, Roots};
use opg::OpgModel;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use warp::filters::BoxedFilter;
use warp::{http::StatusCode, Filter};

fn json_reply_header() -> warp::filters::reply::WithHeader {
    warp::reply::with::header("Content-Type", "application/json")
}

fn yaml_reply_header() -> warp::filters::reply::WithHeader {
    warp::reply::with::header("Content-Type", "application/yaml")
}

pub fn swagger_yaml(api_url: &str) -> BoxedFilter<(impl warp::Reply,)> {
    let docs = docs::v1::swagger_yaml(api_url);
    warp::path!("swagger.yaml")
        .and(warp::get())
        .map(move || docs.clone())
        .with(yaml_reply_header())
        .boxed()
}

pub fn swagger_json(api_url: &str) -> BoxedFilter<(impl warp::Reply,)> {
    let docs = docs::v1::swagger_json(api_url);
    warp::path("swagger.json")
        .and(warp::get())
        .map(move || docs.clone())
        .with(json_reply_header())
        .boxed()
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

#[derive(Clone, Deserialize, Serialize, Hash, OpgModel)]
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
