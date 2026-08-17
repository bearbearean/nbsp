//! Glue for combining [`axum::response::Html`] and a rendered [`askama::Template`].

use askama::Template;
use axum::response::Html;

use crate::prelude::*;

/// Render an [`askama::Template`] and then return it inside [`axum::response::Html`]
pub fn html(template: impl Template) -> crate::WebResult {
    let render = template.render()?;
    Ok(Html(render).into_response())
}
