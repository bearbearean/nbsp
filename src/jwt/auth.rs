//! Axum middleware to facilitate authentication with encrypted cookies and JWTs

use axum::{
    Extension,
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use cookie::Cookie;
use jsonwebtoken::{EncodingKey, errors::ErrorKind::ExpiredSignature};
use sqlx::types::Uuid;

use crate::{
    GlobalState,
    database::{RefreshToken, User},
    jwt::{
        cookies::{
            COOKIE_JWT, COOKIE_REFRESH, COOKIES_MAX_AGE, build_cookie, clear_cookie_jar,
            cookie_jar_from_request,
        },
        generate_jwt, validate_jwt,
    },
    prelude::*,
};

/// Authentication context for a request
#[derive(Clone)]
pub struct Auth {
    /// A possibly authenticated user, if this is `None` then the request was made unauthenticated
    pub user: Option<User>,
}

/// Update the refresh token's last updated date, generate a new JWT and return it as a cookie
pub async fn refresh_jwt_token<'a>(
    refresh_token: RefreshToken,
    jwt_encoding_key: &EncodingKey,
    pool: &PgPool,
) -> Result<(Cookie<'a>, RefreshToken), crate::WebError> {
    let refresh_token = refresh_token.update_last_used(pool).await?;
    let jwt = generate_jwt(jwt_encoding_key, refresh_token.user_id)?;
    let jwt_cookie = build_cookie(COOKIE_JWT, jwt, Some(COOKIES_MAX_AGE));
    Ok((jwt_cookie.build(), refresh_token))
}

/// Base authentication middleware that supports authenticated and unauthenticated requests
pub async fn auth_base(
    State(gs): State<GlobalState>,
    mut request: Request,
    next: Next,
) -> WebResult {
    let mut jar = cookie_jar_from_request(&request, gs.cookies_key);
    let (jwt, refresh) = (jar.get(COOKIE_JWT), jar.get(COOKIE_REFRESH));

    let mut auth = Auth { user: None };

    // If we have a refresh cookie but no jwt cookie, do a refresh
    let mut do_refresh = refresh.is_some() && jwt.is_none();

    if let Some(jwt) = jwt {
        match validate_jwt(&gs.jwt_decoding_key, jwt.value()) {
            Ok(claims) => {
                auth.user = User::optional_find_by_id(claims.sub, &gs.pool).await?;
            }
            Err(err) => match err.kind() {
                ExpiredSignature => {
                    // If the jwt has expired do a refresh
                    do_refresh = true;
                }
                _ => {
                    tracing::trace!(jwt_err = ?err, "invalid jwt encountered");
                    // Clear invalid tokens from cookies
                    jar = clear_cookie_jar(jar);
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
                let (jwt_cookie, refresh_token) =
                    refresh_jwt_token(refresh_token, &gs.jwt_encoding_key, &gs.pool).await?;
                jar = jar.add(jwt_cookie);
                auth.user = Some(User::find_by_id(refresh_token.user_id, &gs.pool).await?);
                tracing::trace!(user_id = refresh_token.user_id, "refreshed jwt");
            } else {
                tracing::warn!(
                    refresh = ?refresh,
                    "expired jwt with refresh token cookie could not find refresh token in db"
                );
                // Clear invalid tokens from cookies
                jar = clear_cookie_jar(jar);
            }
        } else {
            tracing::warn!(
                refresh = refresh.value(),
                "expired jwt with invalid uuid as refresh token"
            );
            // Clear invalid tokens from cookies
            jar = clear_cookie_jar(jar);
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
        let request_uri = request.uri().to_string();
        let request_uri =
            percent_encoding::utf8_percent_encode(&request_uri, percent_encoding::NON_ALPHANUMERIC);
        let redirect_url = format!("/account/login?redirect={}", request_uri);
        Redirect::to(&redirect_url).into_response()
    }
}

/// Middleware to require that the user is *not* authenticated. Authenticated requests will be
/// redirected to the home page.
///
/// This is useful for the registration and login pages, where if a user is already logged in they
/// don't need to be able to register or login again. They should log out first
pub async fn auth_not_allowed(
    Extension(auth): Extension<Auth>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    if auth.user.is_none() {
        next.run(request).await
    } else {
        Redirect::to("/").into_response()
    }
}
