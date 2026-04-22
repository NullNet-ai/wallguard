use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application-level error that maps to an HTTP response.
#[derive(Debug)]
pub enum AppError {
    Unauthorized,
    Forbidden,
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Unauthorized     => (StatusCode::UNAUTHORIZED,            "UNAUTHORIZED",  "unauthorized".to_owned()),
            Self::Forbidden        => (StatusCode::FORBIDDEN,               "FORBIDDEN",     "insufficient permissions".to_owned()),
            Self::NotFound(m)      => (StatusCode::NOT_FOUND,               "NOT_FOUND",     m),
            Self::Conflict(m)      => (StatusCode::CONFLICT,                "CONFLICT",      m),
            Self::BadRequest(m)    => (StatusCode::BAD_REQUEST,             "BAD_REQUEST",   m),
            Self::Internal(m)      => (StatusCode::INTERNAL_SERVER_ERROR,   "INTERNAL",      m),
        };
        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("database error: {e}");
        Self::Internal("database error".into())
    }
}
