//! # nbsp
//!
//! > non-breaking space: a thoughtful community forum platform

use std::net::SocketAddr;

use axum::{Router, extract::State, response::IntoResponse, routing};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    request_id::MakeRequestUuid,
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub mod database;
pub mod templates;
pub mod utilities;

use crate::{
    database::NbspConfig,
    templates::Homepage,
    utilities::{CustomMakeSpan, html},
};

/// The struct for [`axum::extract::State`] with all global state
#[derive(Clone)]
pub struct GlobalState {
    /// The database connection pool to PostgreSQL
    pub pool: PgPool,
    /// Global configuration data used in many places
    pub config: NbspConfig,
}

/// The main function for nbsp.
#[tokio::main]
pub async fn main() {
    // Setup tracing with JSON logging to stdout
    // By default using the DEBUG level, but can be adjusted using the RUST_LOG environment variable
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

    // Keep this log as an indicator when nbsp has initially started
    tracing::info!("non-breaking space: a thoughtful community forum platform");

    let pool = crate::database::initialize().await;

    let global_state = GlobalState {
        pool: pool.clone(),
        config: NbspConfig::load(&pool)
            .await
            .expect("failed to load NbspConfig"),
    };

    // Set up the tower and axum middlewares/services
    let services = ServiceBuilder::new()
        // Add an x-request-id HTTP header to every request to group all logs together for that request
        .set_x_request_id(MakeRequestUuid {})
        // Set up tracing using our CustomMakeSpan and include headers on the response trace
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(CustomMakeSpan {})
                .on_response(DefaultOnResponse::new().include_headers(true)),
        );

    let router = Router::new()
        .route("/", routing::get(root))
        .layer(services)
        .nest("/assets", memory_serve::load!().into_router())
        .with_state(global_state);

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
pub async fn root(
    State(GlobalState {
        pool: _pool,
        config,
    }): State<GlobalState>,
) -> impl IntoResponse {
    html(Homepage {
        // TODO: Make this default info a struct that can be easily obtained from NbspConfig itself
        nbsp_homepage_notice: config.nbsp_homepage_notice,
        nbsp_community_title: config.nbsp_community_title,
        nbsp_community_subtitle: config.nbsp_community_subtitle,
    })
}
