//! Helper functions for HTTP cookies

use cookie::{Cookie, time::Duration};

/// Build a cookie with an optional max age
pub fn build_cookie<'a>(key: &'a str, value: String, max_age: Option<Duration>) -> Cookie<'a> {
    let mut cookie = Cookie::build((key, value))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(cookie::SameSite::Strict);

    if let Some(max_age) = max_age {
        cookie = cookie.max_age(max_age);
    }

    cookie.build()
}

/// Build a cookie to be removed
pub fn removal_cookie<'a>(key: &'a str) -> Cookie<'a> {
    // The path has to match the path in build_cookie otherwise the cookies won't be removed
    // correctly
    Cookie::build(key).path("/").removal().build()
}
