//! The nbsp prelude with commonly used types and traits

pub use anyhow::{Context, Result};
pub use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
pub use sqlx::PgPool;

/// All possible errors nbsp can encounter during HTTP requests
#[derive(Debug, thiserror::Error)]
pub enum WebError {
    /// [`askama`] HTML rendering error
    #[error("failed to render HTML")]
    Askama(#[from] askama::Error),
    /// Any custom internal server error
    #[error("internal server error")]
    InternalServerError(String),
}

/// Convenience wrapper for `Result<Response, WebError>`
pub type WebResult = Result<Response, WebError>;

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        tracing::error!(?self, "internal server error");

        let status = StatusCode::INTERNAL_SERVER_ERROR;

        // TODO: Render a proper HTML template
        let body = axum::response::Html("<h1>internal server error</h1>");

        (status, body).into_response()
    }
}
