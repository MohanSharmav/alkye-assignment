use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("User not found")]
    UserNotFound,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid 2FA code")]
    Invalid2FACode,

    #[error("2FA code expired")]
    Expired2FACode,

    #[error("2FA code already used")]
    Used2FACode,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Task not found")]
    TaskNotFound,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Internal server error")]
    InternalServerError,

    #[error("{0}")]
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
            ApiError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
            ApiError::Invalid2FACode => (StatusCode::BAD_REQUEST, "Invalid 2FA code".to_string()),
            ApiError::Expired2FACode => (StatusCode::BAD_REQUEST, "2FA code expired".to_string()),
            ApiError::Used2FACode => (StatusCode::BAD_REQUEST, "2FA code already used".to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            ApiError::TaskNotFound => (StatusCode::NOT_FOUND, "Task not found".to_string()),
            ApiError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
