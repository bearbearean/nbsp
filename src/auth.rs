//! Axum middleware to facilitate authentication with encrypted cookies and JWTs

use axum::{
    Extension,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::PrivateCookieJar;
use chrono::Duration;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode,
    errors::ErrorKind::ExpiredSignature,
};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::{
    GlobalState,
    database::{RefreshToken, User},
    prelude::*,
    utilities::build_cookie,
};

/// Authentication context for a request
#[derive(Clone)]
pub struct Auth {
    /// A possibly authenticated user, if this is `None` then the request was made unauthenticated
    pub user: Option<User>,
}

/// [`jsonwebtoken`] claims
#[derive(Deserialize, Serialize)]
pub struct JwtClaims {
    /// Subject claim, this will be the user_id
    pub sub: i64,
    /// When the JWT expires
    pub exp: usize,
    /// When the JWT was issued
    pub iat: usize,
}

/// The algorithm to use for [`generate_jwt`] and [`validate_jwt`]
pub static JWT_ALGORITHM: Algorithm = Algorithm::HS256;

/// Generate a JWT for a user
pub fn generate_jwt(key: &EncodingKey, user_id: i64) -> jsonwebtoken::errors::Result<String> {
    tracing::trace!(user_id, "generating jwt");
    let now = Utc::now();
    let header = Header::new(JWT_ALGORITHM);
    let claims = JwtClaims {
        sub: user_id,
        exp: (now + Duration::minutes(30)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(&header, &claims, key)
}

/// Validate a given JWT `token` and its claims
pub fn validate_jwt(key: &DecodingKey, token: &str) -> jsonwebtoken::errors::Result<JwtClaims> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.set_required_spec_claims(&["exp"]);
    Ok(decode::<JwtClaims>(token, key, &validation)?.claims)
}

/// Base authentication middleware that supports authenticated and unauthenticated requests
pub async fn auth_base(
    State(gs): State<GlobalState>,
    mut request: Request,
    next: Next,
) -> WebResult {
    let mut jar = PrivateCookieJar::from_headers(request.headers(), gs.cookies_key);
    let (jwt, refresh) = (jar.get("jwt"), jar.get("refresh"));

    let mut auth = Auth { user: None };

    // If we have a refresh cookie but no jwt cookie, do a refresh
    let mut do_refresh = refresh.is_some() && jwt.is_none();

    if let Some(jwt) = jwt {
        match validate_jwt(&gs.jwt_decoding_key, jwt.value()) {
            Ok(claims) => {
                auth.user = User::optional_find_by_id(claims.sub, &gs.pool).await?;
            }
            Err(err) => match *err.kind() {
                ExpiredSignature => {
                    // If the jwt has expired do a refresh
                    do_refresh = true;
                }
                _ => {
                    tracing::trace!(jwt_err = ?err, "invalid jwt encountered");
                    // Clear invalid tokens from cookies
                    jar = jar.remove("jwt").remove("refresh");
                    do_refresh = false;
                }
            },
        }
    }

    if do_refresh && let Some(refresh) = refresh {
        if let Ok(refresh) = Uuid::try_parse(refresh.value()) {
            if let Some(refresh_token) =
                RefreshToken::optional_find_by_token(&refresh, &gs.pool).await?
            {
                let refresh_token = refresh_token.update_last_used(&gs.pool).await?;
                let jwt = generate_jwt(&gs.jwt_encoding_key, refresh_token.user_id)?;
                let jwt_cookie = build_cookie("jwt", jwt, Some(cookie::time::Duration::hours(1)));
                jar = jar.add(jwt_cookie);
                auth.user = Some(User::find_by_id(refresh_token.user_id, &gs.pool).await?);
                tracing::trace!(user_id = refresh_token.user_id, "refreshed jwt");
            } else {
                tracing::warn!(
                    refresh = ?refresh,
                    "expired jwt with refresh token cookie could not find refresh token in db"
                );
                // Clear invalid tokens from cookies
                jar = jar.remove("jwt").remove("refresh");
            }
        } else {
            tracing::warn!(
                refresh = refresh.value(),
                "expired jwt with invalid uuid as refresh token"
            );
            // Clear invalid tokens from cookies
            jar = jar.remove("jwt").remove("refresh");
        }
    }

    request.extensions_mut().insert(auth);

    let response = next.run(request).await;

    Ok((jar, response).into_response())
}

/// Middleware to require authentication. Unauthenticated requests will be redirected to
/// `/account/login`
pub async fn auth_required(
    Extension(auth): Extension<Auth>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    if auth.user.is_some() {
        next.run(request).await
    } else {
        Redirect::to("/account/login").into_response()
    }
}
