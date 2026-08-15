//! A custom implementation for [`tower_http::trace::MakeSpan`].

use tower_http::trace::MakeSpan;

/// A custom implementation for [`tower_http::trace::MakeSpan`].
#[derive(Debug, Clone)]
pub struct CustomMakeSpan {}

impl<B> MakeSpan<B> for CustomMakeSpan {
    fn make_span(&mut self, request: &axum::http::Request<B>) -> tracing::Span {
        let span = tracing::span!(
            tracing::Level::DEBUG,
            "request",
            method = %request.method(),
            uri = %request.uri(),
            "remote-address" = tracing::field::Empty,
            "x-forwarded-for" = tracing::field::Empty,
            "x-request-id" = tracing::field::Empty,
        );

        let headers = ["x-forwarded-for", "x-request-id"];
        for header in headers {
            if let Some(value) = request
                .headers()
                .get(header)
                .and_then(|value| value.to_str().ok())
            {
                span.record(header, value);
            }
        }

        if let Some(remote_address) = request
            .extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|info| info.0.ip().to_string())
        {
            span.record("remote-address", remote_address);
        };

        span
    }
}
