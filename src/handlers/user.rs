use crate::db::queries::Queries;
use crate::db::Address;
use crate::model::UserDto;
use crate::{api_doc_addon, catch_error_500, response};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use utoipa::OpenApi;
use utoipa::ToSchema;
use warp::http::StatusCode;
use warp::Filter;

#[derive(OpenApi)]
#[openapi(
        paths(get_user_by_address, upsert_user),
        components(schemas(UserDto, UpsertUserPayload)),
        tags(
            (name = "user", description = "User handlers")
        ),
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[utoipa::path(
    get,
    tag = "user",
    path = "/user/{address}",
    params(("address" = String, Path, description = "User address")),
    responses(
        (status = 200, body = UserDto),
        (status = 500),
    )
)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct UpsertUserPayload {
    address: Address,
    username: Option<String>,
    bio: Option<String>,
    logo_nft: Option<String>,
    twitter: Option<String>,
    instagram: Option<String>,
    facebook: Option<String>,
    link: Option<String>,
    email: Option<String>,
}

#[utoipa::path(
    post,
    tag = "user",
    path = "/user/",
    request_body(content = UpsertUserPayload, description = "Upsert user"),
    responses(
    (status = 200),
    (status = 500),
    )
)]
pub fn upsert_user(
    db: Queries,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user")
        .and(warp::post())
        .and(warp::body::json::<UpsertUserPayload>())
        .and(warp::any().map(move || db.clone()))
        .and_then(upsert_user_handler)
}

async fn upsert_user_handler(
    payload: UpsertUserPayload,
    db: Queries,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    catch_error_500!(
        db.upsert_user(
            payload.address,
            payload.bio,
            payload.username,
            payload.logo_nft,
            payload.twitter,
            payload.instagram,
            payload.facebook,
            payload.link,
            payload.email,
        )
        .await
    );

    Ok(Box::from(warp::reply::with_status("", StatusCode::OK)))
}
