//! Axum routes for nbsp

mod account;
mod user;

use axum::{Extension, extract::State};

use crate::{GlobalState, jwt::auth::Auth, prelude::*, templates::Homepage, utilities::html};

pub use account::*;
pub use user::*;

/// The route for `GET /` (the home page)
pub async fn root(State(gs): State<GlobalState>, Extension(auth): Extension<Auth>) -> WebResult {
    html(Homepage {
        config: gs.config,
        auth,
    })
}
