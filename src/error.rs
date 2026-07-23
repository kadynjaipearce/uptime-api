use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Error type returned from axum handlers. Any `anyhow::Error` produced by
/// `?` inside a handler converts into this automatically; the underlying
/// error is logged with full context and the client gets an opaque 500.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("not found: {0}")]
    NotFoundMsg(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("database error")]
    Database(#[from] sqlx::Error),

    #[error("internal server error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            AppError::NotFoundMsg(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden".to_string()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Database(err) => {
                tracing::error!(error = ?err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
            AppError::Internal(err) => {
                tracing::error!(error = ?err, "unhandled error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        #[derive(Serialize)]
        struct Envelope {
            kind: &'static str,
            status: u16,
            message: String,
        }

        (
            status,
            Json(Envelope {
                kind: "error",
                status: status.as_u16(),
                message,
            }),
        )
            .into_response()
    }
}
