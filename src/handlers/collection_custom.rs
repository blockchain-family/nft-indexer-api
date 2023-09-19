use crate::db::queries::Queries;
use crate::db::{Address, Social};
use crate::services::auth::AuthService;
use crate::{api_doc_addon, catch_error_401, catch_error_403, catch_error_500};
use http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa::ToSchema;
use warp::http::StatusCode;
use warp::Filter;

#[derive(OpenApi)]
#[openapi(
    paths(
        upsert_collection_custom
    ),
    components(schemas(
        UpsertCollectionCustomPayload,
        Social
    )),
    tags(
        (name = "collection-custom", description = "Collection-custom handlers"),
    ),
)]
struct ApiDoc;
api_doc_addon!(ApiDoc);

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
    let address_of_collection = payload.address;

    let validation_of_owner = match db
        .validate_owner_of_collection(&address_of_collection, &address)
        .await
        .expect("Failed validation of collections owner")
        .expect("Failed validation of collections owner")
    {
        0 => None,
        v => Some(v),
    };

    catch_error_403!(validation_of_owner);

    catch_error_500!(
        db.upsert_collection_custom(
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

    Ok(Box::from(warp::reply::with_status("", StatusCode::OK)))
}
