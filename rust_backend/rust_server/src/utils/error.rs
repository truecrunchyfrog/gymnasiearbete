use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
pub type Result<T> = core::result::Result<T, Error>;
use anyhow::Error as AnyhowError;

#[derive(Debug)]
pub enum Error {
    LoginFail,
    TokenError,
    UserNotFound,
    InternalServerError,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::LoginFail => (StatusCode::UNAUTHORIZED, "Login failed").into_response(),
            Error::TokenError => (StatusCode::FORBIDDEN, "Token error").into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
        }
    }
}

impl From<AnyhowError> for Error {
    fn from(err: AnyhowError) -> Error {
        // Here you can convert the anyhow::Error into your custom Error type.
        // This is just an example. You need to replace it with your actual conversion logic.
        error!("Error: {}", err);
        Error::InternalServerError
    }
}