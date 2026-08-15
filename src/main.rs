//! # nbsp
//!
//! > non-breaking space: a thoughtful community forum platform

use std::net::SocketAddr;

use axum::{Router, response::IntoResponse, routing};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    request_id::MakeRequestUuid,
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub mod utilities;

use utilities::CustomMakeSpan;

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

    let services = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid {})
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(CustomMakeSpan {})
                .on_response(DefaultOnResponse::new().include_headers(true)),
        );

    let router = Router::new().route("/", routing::get(root)).layer(services);

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind to 127.0.0.1:3000");

    tracing::info!("listening on http://127.0.0.1:3000");

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("axum::serve error");
}

/// The route for `GET /` (the home page)
pub async fn root() -> impl IntoResponse {
    "Welcome to nbsp!"
}
