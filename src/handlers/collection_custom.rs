use std::sync::Arc;

use axum::extract::{Json, State};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::HttpState;
use crate::db::{Address, Social};
use crate::{catch_empty, catch_error_401, catch_error_403, catch_error_500, response};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpsertCollectionCustomPayload {
    pub address: Address,
    pub name: Option<String>,
    pub description: Option<String>,
    pub wallpaper: Option<String>,
    pub logo: Option<String>,
    pub social: Option<Social>,
}

#[utoipa::path(
    post,
    tag = "collection",
    path = "/collections-custom",
    request_body(content = UpsertCollectionCustomPayload, description = "Upsert collection"),
    responses(
        (status = 200),
        (status = 500),
    ),
)]
pub async fn upsert_collection_custom(
    State(s): State<Arc<HttpState>>,
    headers: HeaderMap<HeaderValue>,
    Json(payload): Json<UpsertCollectionCustomPayload>,
) -> impl IntoResponse {
    let address = catch_error_401!(s.auth_service.authenticate(headers));
    let address_of_collection = payload.address;

    let validation_of_owner_result =
        s.db.validate_owner_of_collection(&address_of_collection, &address)
            .await;

    let validation_of_owner_option = catch_error_500!(validation_of_owner_result);

    let validation_of_owner = match catch_empty!(
        validation_of_owner_option,
        "Forbidden action for current user"
    ) {
        0 => None,
        v => Some(v),
    };

    catch_error_403!(validation_of_owner);

    catch_error_500!(
        s.db.upsert_collection_custom(
            &address_of_collection,
            &address,
            chrono::Utc::now().naive_utc(),
            payload.name,
            payload.description,
            payload.wallpaper,
            payload.logo,
            serde_json::to_value(payload.social).expect("Failed parsing social medias"),
        )
        .await
    );

    response!(&())
}
