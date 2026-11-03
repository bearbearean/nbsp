//! Axum routes for nbsp

mod account;
mod post;
mod user;

use axum::extract::State;

use crate::{
    GlobalState, database::PostListItem, jwt::auth::Auth, prelude::*, templates::Homepage,
    utilities::html,
};

pub use account::*;
pub use post::*;
pub use user::*;

/// The route for `GET /` (the home page)
pub async fn root(State(gs): State<GlobalState>, auth: Auth) -> WebResult {
    let posts = if auth.user.is_some() {
        Some(PostListItem::get_recent_25(&gs.pool).await?)
    } else {
        None
    };

    html(Homepage {
        config: gs.config,
        auth,
        posts,
    })
}
