//! # nbsp
//!
//! > non-breaking space: a thoughtful community forum platform

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Extension, Form, Router,
    extract::{FromRef, MatchedPath, Path, Query, Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Redirect,
    routing,
};
use axum_extra::extract::{PrivateCookieJar, cookie::Key};
use cookie::time::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use serde::Deserialize;
use sqlx::types::Uuid;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    request_id::MakeRequestUuid,
    sensitive_headers::{SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer},
    set_header::SetResponseHeaderLayer,
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub mod auth;
pub mod database;
pub mod prelude;
pub mod templates;
pub mod utilities;

use crate::{
    auth::{Auth, auth_base, auth_not_allowed, auth_required, generate_jwt},
    database::{Invite, NbspConfig, RefreshToken, User},
    prelude::*,
    templates::{AccountLogin, AccountRegister, Homepage, HttpStatusPage, UserProfile},
    utilities::{
        CustomMakeSpan, build_cookie, hash_password, html, html_with_status, removal_cookie,
        verify_password,
    },
};

/// The struct for [`axum::extract::State`] with all global state
#[derive(Clone)]
pub struct GlobalState {
    /// The database connection pool to PostgreSQL
    pub pool: PgPool,
    /// Global configuration data used in many places
    pub config: NbspConfig,
    /// The key to encrypt private cookies with
    pub cookies_key: Key,
    /// The key to encode JWTs with
    pub jwt_encoding_key: EncodingKey,
    /// The key to decode JWTs with
    pub jwt_decoding_key: DecodingKey,
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
    let config = NbspConfig::load(&pool)
        .await
        .context("failed to load NbspConfig, check the nbsp_config table in PostgreSQL")?;
    let cookies_key = config.nbsp_cookies_key.clone();
    let jwt_encoding_key = EncodingKey::from_secret(config.nbsp_jwt_signing_key.as_bytes());
    let jwt_decoding_key = DecodingKey::from_secret(config.nbsp_jwt_signing_key.as_bytes());
    let content_security_policy = config.nbsp_content_security_policy.clone();

    let global_state = GlobalState {
        pool: pool.clone(),
        config: config.clone(),
        cookies_key,
        jwt_encoding_key,
        jwt_decoding_key,
    };

    let sensitive_headers: Arc<[_]> = {
        use axum::http::header::*;
        Arc::new([AUTHORIZATION, PROXY_AUTHORIZATION, COOKIE, SET_COOKIE])
    };

    // Set up the tower and axum middlewares/services
    let services = ServiceBuilder::new()
        // Add an x-request-id HTTP header to every request to group all logs together for that request
        .set_x_request_id(MakeRequestUuid {})
        // The sensitive request headers layer must come *before* the trace layer
        .layer(SetSensitiveRequestHeadersLayer::from_shared(Arc::clone(
            &sensitive_headers,
        )))
        // Set up tracing using our CustomMakeSpan and include headers on the response trace
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(CustomMakeSpan {})
                .on_response(DefaultOnResponse::new().include_headers(true)),
        )
        // The sensitive response headers layer must come *after* the trace layer
        .layer(SetSensitiveResponseHeadersLayer::from_shared(
            sensitive_headers,
        ))
        // This has to come after the trace layer
        .propagate_x_request_id()
        // Set the content-security-policy header on every response
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            axum::http::HeaderValue::try_from(content_security_policy)
                .expect("nbsp_content_security_policy is not a valid HeaderValue"),
        ));

    let router_with_auth = Router::new()
        .route("/user/{username}", routing::get(user_profile))
        .layer(axum::middleware::from_fn_with_state(
            global_state.clone(),
            auth_required,
        ));

    let router_without_auth = Router::new()
        .route(
            "/account/register",
            routing::get(account_register).post(do_account_register),
        )
        .route(
            "/account/login",
            routing::get(account_login).post(do_account_login),
        )
        .layer(axum::middleware::from_fn_with_state(
            global_state.clone(),
            auth_not_allowed,
        ));

    let router_with_optional_auth = Router::new()
        .route("/", routing::get(root))
        .route("/account/logout", routing::get(account_logout));

    let router = Router::new()
        .merge(router_with_optional_auth)
        .merge(router_with_auth)
        .merge(router_without_auth)
        .route("/robots.txt", routing::get(permanent_redirects))
        .fallback(fallback_http_404)
        .layer(axum::middleware::from_fn_with_state(
            global_state.clone(),
            auth_base,
        ));

    let router = if config.nbsp_enable_prometheus_metrics {
        router.layer(axum::middleware::from_fn(http_metrics))
    } else {
        router
    };

    let router = router
        .layer(services)
        .nest("/assets", memory_serve::load!().into_router())
        .with_state(global_state);

    if config.nbsp_enable_prometheus_metrics {
        let recorder = start_metrics_recorder();
        let metrics_router =
            Router::new().route("/metrics", routing::get(async move || recorder.render()));
        let metrics_listener = TcpListener::bind("127.0.0.1:3001")
            .await
            .context("failed to bind 127.0.0.1:3001, is the port already in use?")?;

        tokio::spawn(async {
            tracing::info!("metrics listening on http://127.0.0.1:3001");
            axum::serve(metrics_listener, metrics_router)
                .await
                .context("failed to serve metrics on http://127.0.0.1:3001")
                .unwrap()
        });
    }

    start_refresh_tokens_cleaner(pool.clone());

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
pub async fn root(State(gs): State<GlobalState>, Extension(auth): Extension<Auth>) -> WebResult {
    html(Homepage {
        config: gs.config,
        auth,
    })
}

/// The fallback route when no other routes match (ie. HTTP 404)
pub async fn fallback_http_404(headers: HeaderMap, State(gs): State<GlobalState>) -> WebResult {
    let x_request_id = headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    html_with_status(
        HttpStatusPage {
            config: gs.config,
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
    State(gs): State<GlobalState>,
) -> WebResult {
    html(AccountRegister {
        config: gs.config,
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
    State(gs): State<GlobalState>,
    jar: PrivateCookieJar,
    Form(form): Form<AccountRegisterForm>,
) -> WebResult {
    let err_status = StatusCode::UNPROCESSABLE_ENTITY;
    let template = AccountRegister {
        config: gs.config,
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

    let username_is_available = User::is_username_available(&form.username, &gs.pool).await?;
    if !username_is_available {
        tracing::info!("attempted user registration with existing username");
        return html_with_status(template, err_status);
    }

    let invite_is_available = Invite::is_invite_available(&invite, &gs.pool).await?;
    if !invite_is_available {
        tracing::info!("attempted user registration with unavailable invite code");
        return html_with_status(template, err_status);
    }

    if form_is_valid && username_is_available && invite_is_available {
        let password_hash = hash_password(form.password.as_bytes())?;
        let (user, refresh_token) =
            User::register_account(&form.username, &password_hash, &invite, &gs.pool).await?;
        let jwt = generate_jwt(&gs.jwt_encoding_key, user.user_id)?;
        let jwt_cookie = build_cookie("jwt", jwt, Some(Duration::hours(1)));
        let refresh_cookie = build_cookie(
            "refresh",
            refresh_token.refresh_token.to_string(),
            Some(Duration::days(30)),
        );

        Ok((
            jar.add(jwt_cookie).add(refresh_cookie),
            Redirect::to(&format!("/user/{}", user.username)),
        )
            .into_response())
    } else {
        html_with_status(template, err_status)
    }
}

// Tell `PrivateCookieJar` how to access the cookies encryption key from `GlobalState`
impl FromRef<GlobalState> for Key {
    fn from_ref(gs: &GlobalState) -> Self {
        gs.cookies_key.clone()
    }
}

/// Query parameters for `/account/login`
#[derive(Deserialize)]
pub struct AccountLoginQueryParams {
    /// An optional URL to redirect back to after the login is done
    pub redirect: Option<String>,
}

/// The GET handler for `/account/login`
pub async fn account_login(
    State(gs): State<GlobalState>,
    Query(params): Query<AccountLoginQueryParams>,
) -> WebResult {
    html(AccountLogin {
        config: gs.config,
        redirect: params.redirect,
    })
}

/// Expected input form for `POST /account/login`
#[derive(Deserialize)]
pub struct AccountLoginForm {
    /// Value of the username form input
    pub username: String,
    /// Value of the password form input
    pub password: String,
}

/// The POST handler for `/account/login`
pub async fn do_account_login(
    State(gs): State<GlobalState>,
    Query(params): Query<AccountLoginQueryParams>,
    jar: PrivateCookieJar,
    Form(form): Form<AccountLoginForm>,
) -> WebResult {
    let mut txn = gs.pool.begin().await?;
    let err_template = html_with_status(
        AccountLogin {
            config: gs.config,
            redirect: params.redirect.clone(),
        },
        StatusCode::UNAUTHORIZED,
    );

    let user = match User::optional_find_by_username(&form.username, &gs.pool).await? {
        Some(user) => user,
        None => {
            return err_template;
        }
    };

    if !verify_password(form.password.as_bytes(), user.password_hash.as_deref())? {
        return err_template;
    }

    let jwt = generate_jwt(&gs.jwt_encoding_key, user.user_id).map_err(WebError::Jwt)?;
    let jwt_cookie = build_cookie("jwt", jwt, Some(Duration::hours(1)));

    let refresh_token = RefreshToken::new_for_user(user.user_id, &mut txn).await?;
    let refresh_cookie = build_cookie(
        "refresh",
        refresh_token.refresh_token.to_string(),
        Some(Duration::days(30)),
    );

    txn.commit().await?;

    let redirect_url = match params.redirect {
        Some(redirect) => redirect,
        None => format!("/user/{}", user.username),
    };

    Ok((
        jar.add(jwt_cookie).add(refresh_cookie),
        Redirect::to(&redirect_url),
    )
        .into_response())
}

/// The route for `GET /user/{username}`
pub async fn user_profile(
    State(gs): State<GlobalState>,
    Extension(auth): Extension<Auth>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> WebResult {
    match User::optional_find_by_username(&username, &gs.pool).await? {
        Some(target_user) => html(UserProfile {
            auth,
            config: gs.config,
            target_user,
        }),
        None => html_with_status(
            HttpStatusPage {
                config: gs.config,
                title: "User not found - HTTP 404",
                description: "There doesn't seem to be anyone by that username.",
                x_request_id: headers
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or(""),
            },
            StatusCode::NOT_FOUND,
        ),
    }
}

/// The route for `GET /account/logout`
pub async fn account_logout(State(gs): State<GlobalState>, jar: PrivateCookieJar) -> WebResult {
    // If there is a refresh token in the cookies then delete that token from the database
    if let Some(Ok(refresh)) = jar
        .get("refresh")
        .map(|refresh| Uuid::try_parse(refresh.value()))
    {
        RefreshToken::optional_delete(&refresh, &gs.pool).await?;
    }

    let jar = jar
        .remove(removal_cookie("jwt"))
        .remove(removal_cookie("refresh"));
    Ok((jar, Redirect::to("/")).into_response())
}

/// Prometheus metrics for HTTP requests
pub async fn http_metrics(request: Request, next: Next) -> impl IntoResponse {
    let method = request.method().clone().to_string();
    // Use MatchedPath so we get "/user/{username}" as the path instead of actual usernames
    let path = if let Some(matched_path) = request.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_string()
    } else {
        request.uri().path().to_string()
    };

    let response = next.run(request).await;

    let status = response.status().as_u16().to_string();
    let labels = [("method", method), ("path", path), ("status", status)];

    metrics::counter!("http_requests_total", &labels).increment(1);

    response
}

/// Create and start the metrics recorder
pub fn start_metrics_recorder() -> PrometheusHandle {
    let recorder = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to start metrics recorder");

    let upkeep = recorder.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            upkeep.run_upkeep();
        }
    });

    recorder
}

/// Starts the background task to automatically clean up inactive refresh tokens
pub fn start_refresh_tokens_cleaner(pool: PgPool) {
    tokio::spawn(async move {
        tracing::info!("starting refresh tokens cleaner background task");
        loop {
            RefreshToken::clean_inactive(&pool).await;
            tokio::time::sleep(tokio::time::Duration::from_hours(24)).await;
        }
    });
}
