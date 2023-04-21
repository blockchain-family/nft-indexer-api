use crate::model::LoginData;
use crate::services::auth::AuthService;
use crate::{api_doc_addon, catch_error_400, response};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa::ToSchema;
use warp::http::StatusCode;
use warp::Filter;
#[derive(OpenApi)]
#[openapi(paths(sign_in), components(schemas(SignInPayload)), tags(
(name = "auth", description = "Authorization handlers"),
))]
struct ApiDoc;
api_doc_addon!(ApiDoc);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInPayload {
    pub public_key: String,
    pub address: String,
    pub wallet_type: String,
    pub timestamp: u64,
    pub signature: String,
}

#[utoipa::path(
    post,
    tag = "auth",
    path = "/user/sign_in",
    request_body(content = SignInPayload, description = "Sign In"),
    responses(
        (status = 200, body = String),
        (status = 400),
    ),
)]
pub fn sign_in(
    auth_service: Arc<AuthService>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("user" / "sign_in")
        .and(warp::post())
        .and(warp::body::json::<SignInPayload>())
        .and(warp::any().map(move || auth_service.clone()))
        .and_then(sign_in_handler)
}

pub async fn sign_in_handler(
    payload: SignInPayload,
    auth_service: Arc<AuthService>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let login_data = LoginData {
        public_key: payload.public_key,
        address: payload.address,
        wallet_type: payload.wallet_type,
        timestamp: payload.timestamp,
        signature: payload.signature,
    };
    let authorize = auth_service.authorize(login_data);
    let authorize = catch_error_400!(authorize);
    response!(&authorize)
}
