//! # nbsp
//!
//! > non-breaking space: a thoughtful community forum platform

use axum::{Router, response::IntoResponse, routing};
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// The main function for nbsp.
#[tokio::main]
pub async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_current_span(true)
                .with_span_list(false)
                .with_writer(std::io::stdout),
        )
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env_lossy(),
        )
        .init();

    tracing::info!("non-breaking space: a thoughtful community forum platform");

    let router = Router::new().route("/", routing::get(root));

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind to 127.0.0.1:3000");

    tracing::info!("listening on http://127.0.0.1:3000");
    axum::serve(listener, router)
        .await
        .expect("axum::serve error");
}

/// The route for `GET /` (the home page)
pub async fn root() -> impl IntoResponse {
    "Welcome to nbsp!"
}
