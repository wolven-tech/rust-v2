//! Per-ip rate limiting, using allframe's `KeyedRateLimiter` (D5, D20).
//!
//! §9.2: rate-limit counters are deliberately **not** events. They have a high
//! write rate, TTL semantics, and zero audit value — writing them to an
//! immutable log would be pure cost. They live in process memory and are lost
//! on restart, which is the correct trade for this workload.

use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, State},
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::infrastructure::{error::ApiError, state::AppState};

/// Client key for the limiter.
///
/// Prefers `x-forwarded-for`'s first hop when present (we sit behind a proxy in
/// production), falling back to the socket address. Note the honest limitation:
/// `x-forwarded-for` is client-controllable when *not* behind a trusted proxy,
/// so this is abuse-blunting, not a security control.
fn client_key<B>(request: &Request<B>) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| {
            request
                .extensions()
                .get::<ConnectInfo<std::net::SocketAddr>>()
                .map(|ConnectInfo(addr)| addr.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Axum middleware. Rejects with 429 when the bucket is empty.
pub async fn layer(
    State(state): State<Arc<AppState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let key = client_key(&request);
    if state.rate_limiter.check(&key).is_err() {
        tracing::debug!(%key, "rate limited");
        return ApiError::RateLimited.into_response();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use axum::http::Request;

    use super::client_key;

    #[test]
    fn the_first_forwarded_hop_wins() {
        let request = Request::builder()
            .header("x-forwarded-for", "203.0.113.7, 10.0.0.1")
            .body(())
            .unwrap();
        assert_eq!(client_key(&request), "203.0.113.7");
    }

    #[test]
    fn a_blank_forwarded_header_falls_through_rather_than_bucketing_everyone_together() {
        let request = Request::builder()
            .header("x-forwarded-for", "")
            .body(())
            .unwrap();
        assert_eq!(client_key(&request), "unknown");
    }

    #[test]
    fn a_missing_header_and_no_connect_info_is_a_single_named_bucket() {
        let request = Request::builder().body(()).unwrap();
        assert_eq!(client_key(&request), "unknown");
    }
}
