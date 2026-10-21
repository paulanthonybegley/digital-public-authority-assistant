//! Typed application errors mapped to HTTP responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use crate::services::deadline::DeadlineError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("rate limit exceeded, retry later")]
    RateLimited,
    #[error("service dependency unavailable: {0}")]
    DependencyUnavailable(String),
    #[error("internal error")]
    Internal(#[source] anyhow::Error),
}

impl AppError {
    pub fn internal<E: Into<anyhow::Error>>(err: E) -> Self {
        Self::Internal(err.into())
    }
}

impl From<DeadlineError> for AppError {
    fn from(err: DeadlineError) -> Self {
        match err {
            DeadlineError::InvalidDate(_) | DeadlineError::DateOutOfRange(_) => {
                Self::BadRequest(err.to_string())
            }
            DeadlineError::NegativeDays(_) => Self::Validation(err.to_string()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            Self::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, m.clone()),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            Self::DependencyUnavailable(m) => (StatusCode::SERVICE_UNAVAILABLE, m.clone()),
            Self::Internal(_) => {
                tracing::error!(error = ?self, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
