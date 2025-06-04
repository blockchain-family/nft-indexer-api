use std::sync::Arc;

use super::HttpState;
use crate::db::Address;
use crate::model::UserDto;
use crate::{catch_error_500, response};
use axum::extract::{Json, Path, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
pub async fn get_user_by_address(
    State(s): State<Arc<HttpState>>,
    Path(address): Path<Address>,
) -> impl IntoResponse {
    let user = catch_error_500!(s.db.get_user_by_address(&address).await);
    let mut user = user.unwrap_or_default();
    user.address = address;
    let user = UserDto::from(user);
    response!(&user)
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpsertUserPayload {
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
pub async fn upsert_user(
    State(s): State<Arc<HttpState>>,
    Json(payload): Json<UpsertUserPayload>,
) -> impl IntoResponse {
    catch_error_500!(
        s.db.upsert_user(
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

    response!(&())
}
