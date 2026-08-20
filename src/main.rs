//! # nbsp
//!
//! > non-breaking space: a thoughtful community forum platform

use std::net::SocketAddr;

use axum::{
    Form, Router,
    extract::{Query, Request, State},
    http::HeaderMap,
    response::Redirect,
    routing,
};
use serde::Deserialize;
use sqlx::types::Uuid;
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
pub mod prelude;
pub mod templates;
pub mod utilities;

use crate::{
    database::{Invite, NbspConfig, User},
    prelude::*,
    templates::{AccountRegister, Homepage, HttpStatusPage},
    utilities::{CustomMakeSpan, hash_password, html, html_with_status},
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
pub async fn main() -> Result<()> {
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

    let pool = crate::database::initialize()
        .await
        .context("failed to connect to PostgreSQL, check the NBSP_PG_... environment variables")?;

    let global_state = GlobalState {
        pool: pool.clone(),
        config: NbspConfig::load(&pool)
            .await
            .context("failed to load NbspConfig, check the nbsp_config table in PostgreSQL")?,
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
        )
        // This has to come after the trace layer
        .propagate_x_request_id();

    let router = Router::new()
        .route("/", routing::get(root))
        .route(
            "/account/register",
            routing::get(account_register).post(do_account_register),
        )
        .route("/robots.txt", routing::get(permanent_redirects))
        .fallback(fallback_http_404)
        .layer(services)
        .nest("/assets", memory_serve::load!().into_router())
        .with_state(global_state);

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .context("failed to bind 127.0.0.1:3000, is the port already in use?")?;

    tracing::info!("listening on http://127.0.0.1:3000");

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("failed to serve nbsp on http://127.0.0.1:3000")?;

    Ok(())
}

/// The route for `GET /` (the home page)
pub async fn root(
    State(GlobalState {
        pool: _pool,
        config,
    }): State<GlobalState>,
) -> WebResult {
    html(Homepage { config })
}

/// The fallback route when no other routes match (ie. HTTP 404)
pub async fn fallback_http_404(
    headers: HeaderMap,
    State(GlobalState {
        pool: _pool,
        config,
    }): State<GlobalState>,
) -> WebResult {
    let x_request_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    html_with_status(
        HttpStatusPage {
            config,
            title: "Page not found - HTTP 404",
            description: "Whatever you're looking for, we can't seem to find it!",
            x_request_id,
        },
        StatusCode::NOT_FOUND,
    )
}

/// A generic handler for any permanent redirects we may want
pub async fn permanent_redirects(request: Request) -> WebResult {
    let location = match request.uri().path() {
        "/robots.txt" => "/assets/robots.txt",
        _ => {
            // In theory this branch of the match could never be triggered because all the routes
            // that use this handler have to manually be added. So treat any other request we get
            // here as an unimplemented branch.
            return Err(WebError::InternalServerError(
                "unimplemented permanent_redirects branch".to_string(),
            ));
        }
    };

    Ok((Redirect::permanent(location)).into_response())
}

/// Query parameters for `GET /account/register`
#[derive(Deserialize)]
pub struct AccountRegisterQueryParams {
    /// An optional invite code to prefill in the account registration form
    pub invite: Option<String>,
}

/// The GET handler for `/account/register`
pub async fn account_register(
    params: Query<AccountRegisterQueryParams>,
    State(GlobalState {
        pool: _pool,
        config,
    }): State<GlobalState>,
) -> WebResult {
    html(AccountRegister {
        config,
        prefilled_invite_code: params.0.invite,
    })
}

/// Expected input form for `POST /account/register`
#[derive(Deserialize)]
pub struct AccountRegisterForm {
    /// Value of the username form input
    pub username: String,
    /// Value of the password form input
    pub password: String,
    /// Value of the confirm_password form input
    pub confirm_password: String,
    /// Value of the invite form input
    pub invite: String,
}

impl AccountRegisterForm {
    /// Validate an account registration form input matches all the expected formats
    pub fn validate(&self) -> bool {
        self.password == self.confirm_password
            && User::validate_username(&self.username)
            && User::validate_password(&self.password)
            && Invite::validate_invite(&self.invite)
    }
}

/// The POST handler for `/account/register`
pub async fn do_account_register(
    State(GlobalState { pool, config }): State<GlobalState>,
    Form(form): Form<AccountRegisterForm>,
) -> WebResult {
    let err_status = StatusCode::UNPROCESSABLE_ENTITY;
    let template = AccountRegister {
        config,
        prefilled_invite_code: Some(form.invite.clone()),
    };

    let invite = match Uuid::try_parse(&form.invite) {
        Ok(invite) => invite,
        Err(_) => return html_with_status(template, err_status),
    };

    let form_is_valid = form.validate();
    if !form_is_valid {
        tracing::info!("attempted user registration with invalid form details");
        return html_with_status(template, err_status);
    }

    let username_is_available = User::is_username_available(&form.username, &pool).await?;
    if !username_is_available {
        tracing::info!("attempted user registration with existing username");
        return html_with_status(template, err_status);
    }

    let invite_is_available = Invite::is_invite_available(&invite, &pool).await?;
    if !invite_is_available {
        tracing::info!("attempted user registration with unavailable invite code");
        return html_with_status(template, err_status);
    }

    if form_is_valid && username_is_available && invite_is_available {
        let password_hash = hash_password(form.password.as_bytes())?;
        let user = User::register_account(&form.username, &password_hash, &invite, &pool).await?;
        Ok(Redirect::to(&format!("/user/{}", user.username)).into_response())
    } else {
        html_with_status(template, err_status)
    }
}
