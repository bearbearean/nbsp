//! Glue for combining [`axum::response::Html`] and a rendered [`askama::Template`].

use askama::Template;
use axum::response::Html;

use crate::prelude::*;

/// Render an [`askama::Template`] and then return it inside [`axum::response::Html`]
pub fn html(template: impl Template) -> crate::WebResult {
    Ok(Html(template.render()?).into_response())
}

/// Render an [`askama::Template`] and then return it inside [`axum::response::Html`] with a
/// specified HTTP status code
pub fn html_with_status(template: impl Template, status: StatusCode) -> crate::WebResult {
    Ok((status, Html(template.render()?)).into_response())
}
