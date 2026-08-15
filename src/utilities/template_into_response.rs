//! Glue for combining [`axum::response::Html`] and a rendered [`askama::Template`].

use askama::Template;
use axum::response::{Html, IntoResponse};

/// Render an [`askama::Template`] and then return it inside [`axum::response::Html`]
pub fn html(template: impl Template) -> impl IntoResponse {
    let render = template.render().expect("template render");
    Html(render).into_response()
}
