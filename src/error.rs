use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("multipart error: {0}")]
    Multipart(#[from] MultipartError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Db(sqlx::Error::RowNotFound) => (
                StatusCode::NOT_FOUND,
                "The requested item was not found.".to_string(),
            ),
            AppError::Db(e) => {
                tracing::error!(error = %e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A database error occurred. Please try again.".to_string(),
                )
            }
            AppError::Json(e) => {
                tracing::error!(error = %e, "serialization error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred.".to_string(),
                )
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Multipart(e) => {
                tracing::error!(error = %e, "multipart error");
                (StatusCode::BAD_REQUEST, e.to_string())
            }
        };
        crate::templates::error::error_page(status, &message).into_response()
    }
}
