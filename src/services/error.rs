use thiserror::Error;

#[derive(Error, Debug)]
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
