//! JSON Web Tokens for nbsp

use chrono::Duration;
use cookie::Cookie;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Result,
};
use serde::{Deserialize, Serialize};

use crate::{
    jwt::cookies::{COOKIE_JWT, COOKIES_MAX_AGE, build_cookie},
    prelude::*,
};

pub mod auth;
pub mod cookies;

/// The algorithm to use for [`generate_jwt`] and [`validate_jwt`]
pub static JWT_ALGORITHM: Algorithm = Algorithm::HS256;

/// The expiration duration for JWTs
pub static JWT_EXPIRY_DURATION: Duration = Duration::minutes(15);

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

/// Generate a JWT for a user
pub fn generate_jwt(key: &EncodingKey, user_id: i64) -> Result<String> {
    tracing::trace!(user_id, "generating jwt");
    let now = Utc::now();
    let header = Header::new(JWT_ALGORITHM);
    let claims = JwtClaims {
        sub: user_id,
        exp: (now + JWT_EXPIRY_DURATION).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(&header, &claims, key)
}

/// Validate a given JWT `token` and its claims
pub fn validate_jwt(key: &DecodingKey, token: &str) -> Result<JwtClaims> {
    let mut validation = Validation::new(JWT_ALGORITHM);
    validation.set_required_spec_claims(&["exp"]);
    Ok(decode::<JwtClaims>(token, key, &validation)?.claims)
}

/// Generate a JWT for a user and return it as a cookie
pub fn generate_jwt_cookie<'a>(key: &EncodingKey, user_id: i64) -> Result<Cookie<'a>> {
    let jwt = generate_jwt(key, user_id)?;
    let jwt_cookie = build_cookie(COOKIE_JWT, jwt, Some(COOKIES_MAX_AGE));
    Ok(jwt_cookie.build())
}
