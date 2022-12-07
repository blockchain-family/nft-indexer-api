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

use std::convert::Infallible;
use warp::{Filter, http::{StatusCode, Response}};

lazy_static::lazy_static! {
    static ref SWAGGER: Vec<u8> = {
        std::fs::read("openapi.yml").expect("cannot read 'openapi.yml' from disk")
    };
}

/// GET /swagger
pub fn get_swagger() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("swagger")
        .and(warp::get())
        .and_then(get_swagger_handler)
}

async fn get_swagger_handler() -> Result<Box<dyn warp::Reply>, Infallible> {
    Ok(Box::from(
            warp::reply::with_status(
                Response::builder()
                    .header("Content-Type", "application/yaml")
                    .body::<&[u8]>(SWAGGER.as_ref()),
            StatusCode::OK)
    ))
}