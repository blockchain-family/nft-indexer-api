use crate::db::queries::Queries;
use crate::db::Address;
use crate::model::UserDto;
use crate::{catch_error_500, response};
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Filter;

/// GET /user/:address
pub fn get_user_by_address(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / String)
        .and(warp::get())
        .and(warp::any().map(move || db.clone()))
        .and_then(get_user_by_address_handler)
}

async fn get_user_by_address_handler(
    address: Address,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let user = catch_error_500!(db.get_user_by_address(&address).await);
    let mut user = user.unwrap_or_default();
    user.address = address;
    let user = UserDto::from(user);
    response!(&user)
}
