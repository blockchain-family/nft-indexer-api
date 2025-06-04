use axum::routing::get;
use axum::{http, Router};
use utoipa::openapi::Server;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::api_doc::ApiDoc;

pub fn swagger_ui<S: Into<String>>(api_url: S) -> Router {
    let openapi = openapi(api_url);
    SwaggerUi::new("/swagger")
        .url("/api-docs/openapi.json", openapi)
        .into()
}

pub fn swagger_yaml<S: Into<String>>(api_url: S) -> Router {
    let openapi = openapi(api_url);
    let swagger_yaml = openapi.to_yaml().expect("failed to create swagger.yaml");

    Router::new().route(
        "/swagger.yaml",
        get(|| async { (http::StatusCode::OK, swagger_yaml) }),
    )
}

pub fn swagger_json<S: Into<String>>(api_url: S) -> Router {
    let openapi = openapi(api_url);
    let swagger_json = openapi
        .to_pretty_json()
        .expect("failed to create swagger.json");

    Router::new().route(
        "/swagger.json",
        get(|| async { (http::StatusCode::OK, swagger_json) }),
    )
}

fn openapi<S: Into<String>>(api_url: S) -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.servers = Some(vec![Server::new(api_url)]);
    openapi
}
