use core::fmt::Display;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use serde_json::json;

pub type Result<T> = core::result::Result<T, AppError>;

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    LoginFail,
    UserNotFound,
    WrongPassword,
    FileNotFound,

    // -- Database errors.
    DatabaseConnectionFail,
    DatabaseQueryFail,
    DatabaseFailedToFindUser,
    AuthFailTokenNotFound,

    // -- Auth errors.
    AuthFailNoAuthTokenCookie,
    AuthFailTokenExpired,
    AuthFailTokenWrongFormat,
    AuthFailCtxNotInRequestExt,
    AuthFailInvalidToken,

    InternalServerError,
    FailedToCalculateScore,

    // -- Model errors.
    TicketDeleteFailIdNotFound { id: u64 },
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        println!("->> {:<12} - {self:?}", "INTO_RES");

        // Create a placeholder Axum response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response.
        response.extensions_mut().insert(self);

        response
    }
}

impl Error {
    #[must_use]
    pub const fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        #[allow(unreachable_patterns)]
        match self {
            Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            // -- Auth.
            Self::AuthFailNoAuthTokenCookie
            | Self::AuthFailTokenWrongFormat
            | Self::AuthFailCtxNotInRequestExt => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // -- Model.
            Self::TicketDeleteFailIdNotFound { .. } => {
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }

            // -- Fallback.
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

// https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs

// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppError: {}", self.0)
    }
}

// Tell axum how to convert `AppError` into a response.
// We can return client errors but everything else will be an internal server error.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, client_error) = self.0.downcast_ref::<Error>().map_or(
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
            |e| e.client_status_and_error(),
        );

        let json = Json(json!({ "error": client_error.as_ref() }));

        (status, json).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
#[allow(clippy::module_name_repetitions)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    INVALID_PARAMS,
    SERVICE_ERROR,
    INVALID_FILE,
}

// Clienterror implements apperror
impl From<ClientError> for AppError {
    fn from(err: ClientError) -> Self {
        anyhow::anyhow!("ClientError: {:?}", err).into()
    }
}
