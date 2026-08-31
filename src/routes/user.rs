//! All axum routes under `/user/...`

use axum::{
    Extension,
    extract::{Path, State},
    http::HeaderMap,
};

use crate::{
    GlobalState,
    database::User,
    jwt::auth::Auth,
    prelude::*,
    templates::{HttpStatusPage, UserProfile},
    utilities::{html, html_with_status},
};

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
