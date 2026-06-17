use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("invalid credentials")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("validation error: {0}")]
    Validation(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    NotFound(String),
    #[error("internal error")]
    Internal(#[from] anyhow::Error),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        tracing::error!(error = %err, "io error");
        AppError::Internal(anyhow::anyhow!("io error"))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "database error");
        AppError::Internal(anyhow::anyhow!("database error"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Internal(err) => {
                tracing::error!(error = %err, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
