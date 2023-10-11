use std::convert::Infallible;

use utoipa::OpenApi;
use warp::{http::StatusCode, Filter};

use crate::handlers::requests::metadata::{UpdateMetadataParams, UpdateMetadataParamsExt};
use crate::{api_doc_addon, catch_error_500};

#[derive(OpenApi)]
#[openapi(
    paths(
        update_metadata
    ),
    components(schemas(
        UpdateMetadataParams
    )),
    tags(
        (name = "metadata", description = "Refresh metadata")
    ),
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

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
pub fn update_metadata(
    indexer_api_url: String,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("update-metadata")
        .and(warp::post())
        .and(warp::body::json::<UpdateMetadataParams>())
        .and(warp::any().map(move || indexer_api_url.clone()))
        .and_then(update_metadata_handler)
}

pub async fn update_metadata_handler(
    params: UpdateMetadataParams,
    indexer_api_url: String,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let client = catch_error_500!(reqwest::Client::builder().build());

    let url = format!("{indexer_api_url}/metadata/refresh/");

    log::info!("Requesting meta update (url = {url}");

    let params = UpdateMetadataParamsExt {
        nft: params.nft,
        collection: params.collection,
        only_collection_info: true,
    };

    let response = catch_error_500!(client.post(url).json(&params).send().await);

    if response.status().is_client_error() || response.status().is_server_error() {
        let status = response.status();
        let reply = response.text().await.unwrap_or("Unknown error".to_string());

        Ok(Box::from(warp::reply::with_status(reply, status)))
    } else {
        Ok(Box::from(warp::reply::with_status("", StatusCode::OK)))
    }
}
