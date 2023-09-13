use http::StatusCode;
use serde::Serialize;
use std::convert::Infallible;
use thiserror::Error;
use warp::{Rejection, Reply};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum Error {
    #[error("Wrong credentials")]
    WrongCredentials,
    #[error("Jwt token not valid")]
    JwtToken,
    #[error("Jwt token creation error")]
    JwtTokenCreation,
    #[error("No auth header")]
    NoAuthHeader,
    #[error("Invalid auth header")]
    InvalidAuthHeader,
    #[error("No permission")]
    NoPermission,
}

#[derive(Serialize, Debug)]
struct ErrorResponse {
    message: String,
    status: String,
}

impl warp::reject::Reject for Error {}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::WrongCredentials => (StatusCode::FORBIDDEN, e.to_string()),
            Error::NoPermission => (StatusCode::UNAUTHORIZED, e.to_string()),
            Error::JwtToken => (StatusCode::UNAUTHORIZED, e.to_string()),
            Error::NoAuthHeader => (StatusCode::UNAUTHORIZED, e.to_string()),
            Error::JwtTokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            ),
            _ => (StatusCode::BAD_REQUEST, e.to_string()),
        }
    } else if err
        .find::<warp::filters::body::BodyDeserializeError>()
        .is_some()
    {
        (
            StatusCode::BAD_REQUEST,
            err.find::<warp::filters::body::BodyDeserializeError>()
                .expect("Failed finding error")
                .to_string(),
        )
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed".to_string(),
        )
    } else {
        println!("unhandled error: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    };

    let json = warp::reply::json(&ErrorResponse {
        status: code.to_string(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}
