use crate::handlers::HttpState;
use crate::model::LoginData;
use crate::{catch_error_400, response};
use axum::extract::{Json, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInPayload {
    pub public_key: String,
    pub address: String,
    pub wallet_type: String,
    pub timestamp: u64,
    pub signature: String,
    pub with_signature_id: Option<i32>,
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
pub async fn sign_in(
    State(s): State<Arc<HttpState>>,
    Json(payload): Json<SignInPayload>,
) -> impl IntoResponse {
    let login_data = LoginData {
        public_key: payload.public_key,
        address: payload.address,
        wallet_type: payload.wallet_type,
        timestamp: payload.timestamp,
        signature: payload.signature,
        with_signature_id: payload.with_signature_id,
    };
    let authorize = catch_error_400!(s.auth_service.authorize(login_data));
    response!(&authorize)
}
