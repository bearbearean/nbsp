//! Encrypted cookies all in one place

use axum::extract::Request;
use axum_extra::extract::PrivateCookieJar;
use cookie::{CookieBuilder, Key, SameSite, time::Duration};

/// The cookie name for the JWT
pub static COOKIE_JWT: &str = "jwt";

/// The cookie name for the refresh token
pub static COOKIE_REFRESH: &str = "refresh";

/// The default max age of cookies: 30 minutes
pub static COOKIES_MAX_AGE: Duration = Duration::minutes(30);

/// The max age of the refresh token cookie: 30 days
pub static COOKIE_REFRESH_MAX_AGE: Duration = Duration::days(30);

/// Create a [`PrivateCookieJar`] from a [`Request`]
pub fn cookie_jar_from_request(request: &Request, key: Key) -> PrivateCookieJar {
    PrivateCookieJar::from_headers(request.headers(), key)
}

/// Clear all known nbsp cookies from a cookie jar
pub fn clear_cookie_jar(cookie_jar: PrivateCookieJar) -> PrivateCookieJar {
    cookie_jar
        .remove(removal_cookie(COOKIE_JWT))
        .remove(removal_cookie(COOKIE_REFRESH))
}

/// Build a cookie with an optional `max_age` defaulting to [`COOKIES_MAX_AGE`]
pub fn build_cookie<'a>(
    name: &'a str,
    value: String,
    max_age: Option<Duration>,
) -> CookieBuilder<'a> {
    CookieBuilder::new(name, value)
        .http_only(true)
        .max_age(max_age.unwrap_or(COOKIES_MAX_AGE))
        .path("/")
        .same_site(SameSite::Strict)
        .secure(true)
}

/// Build a cookie for it to be removed
pub fn removal_cookie<'a>(name: &'a str) -> CookieBuilder<'a> {
    build_cookie(name, String::new(), None).removal()
}
