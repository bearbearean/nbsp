//! # nbsp
//!
//! > non-breaking space: a thoughtful community forum platform

use axum::{Router, response::IntoResponse, routing};
use tokio::net::TcpListener;

/// The main function for nbsp.
#[tokio::main]
pub async fn main() {
    println!("non-breaking space: a thoughtful community forum platform");

    let router = Router::new().route("/", routing::get(root));

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind to 127.0.0.1:3000");

    println!("listening on http://127.0.0.1:3000");
    axum::serve(listener, router)
        .await
        .expect("axum::serve error");
}

/// The route for `GET /` (the home page)
pub async fn root() -> impl IntoResponse {
    "Welcome to nbsp!"
}
