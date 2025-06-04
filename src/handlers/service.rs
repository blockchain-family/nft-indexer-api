use crate::{
    catch_error_500,
    handlers::HttpState,
    model::{Root, Roots},
    response,
};
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

#[utoipa::path(
    get,
    tag = "service",
    path = "/roots",
    responses(
        (status = 200, body = Roots),
        (status = 500),
    ),
)]
pub async fn list_roots(State(s): State<Arc<HttpState>>) -> impl IntoResponse {
    let list = catch_error_500!(s.db.list_roots().await);
    let roots: Vec<Root> = list.into_iter().map(Root::from).collect();
    response!(&Roots { roots })
}
