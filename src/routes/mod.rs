//! Axum routes for nbsp

mod account;
mod post;
mod user;

use axum::extract::State;

use crate::{GlobalState, jwt::auth::Auth, prelude::*, templates::Homepage, utilities::html};

pub use account::*;
pub use post::*;
pub use user::*;

/// The route for `GET /` (the home page)
pub async fn root(State(gs): State<GlobalState>, auth: Auth) -> WebResult {
    html(Homepage {
        config: gs.config,
        auth,
    })
}
