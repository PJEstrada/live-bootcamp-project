use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TwoFaError {
    #[error("invalid 2FA code")]
    InvalidCode,
}

pub enum AuthAPIError {
    UserAlreadyExists,
    InvalidCredentials,
    UnexpectedError,
    IncorrectCredentials,
    MissingToken,
    InvalidToken
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IntoResponse for AuthAPIError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthAPIError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AuthAPIError::InvalidCredentials => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::IncorrectCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthAPIError::MissingToken => (StatusCode::BAD_REQUEST, "Invalid credentials"),
            AuthAPIError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthAPIError::UnexpectedError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
            }
        };
        let body = Json(ErrorResponse {
            error: error_message.to_string(),
        });
        (status, body).into_response()
    }
}