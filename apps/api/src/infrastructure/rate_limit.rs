//! Per-ip rate limiting, using allframe's `KeyedRateLimiter` (D5, D20).
//!
//! §9.2: rate-limit counters are deliberately **not** events. They have a high
//! write rate, TTL semantics, and zero audit value — writing them to an
//! immutable log would be pure cost. They live in process memory and are lost
//! on restart, which is the correct trade for this workload.
//!
//! ## Why the client key is configured rather than guessed
//!
//! The first version took `x-forwarded-for`'s **leftmost** entry whenever the
//! header was present. That header is appended to by each hop, so the leftmost
//! entry is whatever the *client* sent — arbitrary, unauthenticated text on any
//! deployment that is not behind a proxy that overwrites it.
//!
//! Two consequences, both bad, and the second is the worse one:
//!
//! 1. A caller sending a different fake ip per request gets a fresh bucket every
//!    time, so the limiter is off for precisely the traffic it exists to blunt.
//! 2. Every fake ip becomes a **permanent key** in an in-memory map. A single
//!    scripted loop grows the process's memory without bound. A rate limiter
//!    that is also a memory-exhaustion vector is worse than no rate limiter.
//!
//! [`ServerConfig::trusted_proxy_hops`] fixes both by making the operator state
//! how many right-hand entries their own infrastructure appends. The default is
//! `0` — no trusted proxy — under which the header is ignored outright and the
//! key is the socket address, which a client cannot forge.
//!
//! [`ServerConfig::trusted_proxy_hops`]: rv2_shared::ServerConfig::trusted_proxy_hops

use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, State},
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::infrastructure::{error::ApiError, state::AppState};

/// The socket address, which no client can forge.
fn peer_ip<B>(request: &Request<B>) -> Option<String> {
    request
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip().to_string())
}

/// Client key for the limiter.
///
/// `trusted_hops` is how many entries on the **right** of `x-forwarded-for` are
/// appended by infrastructure the operator controls. The honest client ip is
/// therefore the entry `trusted_hops` from the end — with one proxy that is the
/// last entry, not the first.
///
/// `0` ignores the header completely. Anything the header cannot answer falls
/// back to the peer address.
fn client_key<B>(request: &Request<B>, trusted_hops: usize) -> String {
    if trusted_hops == 0 {
        return peer_ip(request).unwrap_or_else(|| "unknown".to_string());
    }

    let forwarded = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Fewer entries than configured hops means the request did not arrive
    // through the proxy chain we were told about — a direct hit on the service,
    // or a misconfiguration. Either way the header cannot be interpreted, so
    // fall back rather than pick an arbitrary entry from it.
    forwarded
        .len()
        .checked_sub(trusted_hops)
        .and_then(|index| forwarded.get(index))
        .map(|ip| (*ip).to_string())
        .or_else(|| peer_ip(request))
        .unwrap_or_else(|| "unknown".to_string())
}

/// Axum middleware. Rejects with 429 when the bucket is empty.
pub async fn layer(
    State(state): State<Arc<AppState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let key = client_key(&request, state.trusted_proxy_hops);
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

    /// The default. A spoofed header must not be able to change the bucket,
    /// because on a directly-exposed service every byte of it is attacker-chosen.
    #[test]
    fn with_no_trusted_proxy_the_header_is_ignored_entirely() {
        let request = Request::builder()
            .header("x-forwarded-for", "203.0.113.7")
            .body(())
            .unwrap();
        assert_eq!(client_key(&request, 0), "unknown");
    }

    /// Behind one proxy, the honest ip is the one the proxy appended — the
    /// **last** entry. Everything to its left is client-supplied.
    #[test]
    fn behind_one_proxy_the_last_hop_is_the_client() {
        let request = Request::builder()
            .header("x-forwarded-for", "203.0.113.7")
            .body(())
            .unwrap();
        assert_eq!(client_key(&request, 1), "203.0.113.7");
    }

    /// The attack the old code was open to: prepend a fake entry and get a new
    /// bucket. With the hop count configured, the prefix is ignored.
    #[test]
    fn a_spoofed_prefix_cannot_change_the_bucket() {
        let spoofed = Request::builder()
            .header("x-forwarded-for", "9.9.9.9, 203.0.113.7")
            .body(())
            .unwrap();
        let plain = Request::builder()
            .header("x-forwarded-for", "203.0.113.7")
            .body(())
            .unwrap();
        assert_eq!(client_key(&spoofed, 1), client_key(&plain, 1));
    }

    #[test]
    fn two_proxies_read_the_second_entry_from_the_end() {
        let request = Request::builder()
            .header("x-forwarded-for", "9.9.9.9, 203.0.113.7, 10.0.0.1")
            .body(())
            .unwrap();
        assert_eq!(client_key(&request, 2), "203.0.113.7");
    }

    /// A shorter chain than configured means the request did not come through
    /// the expected proxies. Falling back is correct; picking whatever entry
    /// happens to be there would trust a client-supplied value.
    #[test]
    fn a_chain_shorter_than_the_hop_count_falls_back() {
        let request = Request::builder()
            .header("x-forwarded-for", "203.0.113.7")
            .body(())
            .unwrap();
        assert_eq!(client_key(&request, 3), "unknown");
    }

    #[test]
    fn a_blank_header_falls_back_rather_than_bucketing_everyone_together() {
        let request = Request::builder()
            .header("x-forwarded-for", "")
            .body(())
            .unwrap();
        assert_eq!(client_key(&request, 1), "unknown");
    }
}
