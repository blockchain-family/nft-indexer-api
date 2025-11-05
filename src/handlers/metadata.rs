use std::sync::Arc;

use axum::extract::{Json, State};
use axum::response::IntoResponse;

use super::HttpState;
use crate::handlers::requests::metadata::{UpdateMetadataParams, UpdateMetadataParamsExt};
use crate::{catch_error_500, response};

#[utoipa::path(
    post,
    tag = "metadata",
    path = "/update-metadata",
    request_body(content = UpdateMetadataParams, description = "Update metadata"),
    responses(
        (status = 200),
        (status = 500),
    ),
)]
pub async fn update_metadata(
    State(s): State<Arc<HttpState>>,
    Json(params): Json<UpdateMetadataParams>,
) -> impl IntoResponse {
    let client = catch_error_500!(reqwest::Client::builder().build());

    let url = format!("{}/metadata/refresh/", s.indexer_api_url);

    log::info!("Requesting meta update (url = {url}");

    let params = UpdateMetadataParamsExt {
        nft: params.nft,
        collection: params.collection,
        only_collection_info: true,
    };

    let response = catch_error_500!(client.post(url).json(&params).send().await);

    if response.status().is_client_error() || response.status().is_server_error() {
        let status = catch_error_500!(axum::http::StatusCode::from_u16(response.status().as_u16()));
        let reply = response.text().await.unwrap_or("Unknown error".to_string());
        (status, reply).into_response()
    } else {
        response!(&())
    }
}
