use crate::db::queries::Queries;
use crate::db::Address;
use crate::{api_doc_addon, catch_error_500, catch_error_401};
use crate::services::auth::AuthService;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa::ToSchema;
use warp::http::StatusCode;
use warp::Filter;
use http::{HeaderMap, HeaderValue};

#[derive(OpenApi)]
#[openapi(
    paths(
        upsert_collection_custom
    ),
    components(schemas(
        UpsertCollectionCustomPayload
    )),
    tags(
        (name = "collection", description = "Collection_custom handlers"),
    ),
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[derive(Debug, Clone, Deserialize, Serialize, Hash, ToSchema)]
pub struct Social {
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub discord: Option<String>,
    pub youtube: Option<String>,
    pub facebook: Option<String>,
    pub medium: Option<String>,
}

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
pub fn upsert_collection_custom(
    db: Queries,
    auth_service: Arc<AuthService>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("collections-custom")
        .and(warp::post())
        .and(warp::body::json::<UpsertCollectionCustomPayload>())
        .and(warp::header::headers_cloned())
        .and(warp::any().map(move || db.clone()))
        .and(warp::any().map(move || auth_service.clone()))
        .and_then(upsert_collection_custom_handler)
}

pub async fn upsert_collection_custom_handler(
    payload: UpsertCollectionCustomPayload,
    headers: HeaderMap<HeaderValue>,
    db: Queries,
    auth_service: Arc<AuthService>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let address = catch_error_401!(auth_service.authenticate(headers));

    catch_error_500!(
        db.upsert_collection_custom(
            payload.address,
            &address,
            chrono::Utc::now().naive_utc(),
            payload.name,
            payload.description,
            payload.wallpaper,
            payload.logo,
            serde_json::to_value(payload.social).unwrap(),
        )
        .await
    );

    Ok(Box::from(warp::reply::with_status("", StatusCode::OK)))
}