//! The rust-v2 API, as a library.
//!
//! `main.rs` is a thin wrapper over [`build_router`] so that integration tests
//! can mount the same router in-process without spawning a binary.
//!
//! ## Deviation from D5, recorded here because it is load-bearing
//!
//! D5 says "`allframe 0.1.28` for router / health / openapi / resilience /
//! rate-limit". `allframe` turns out to be a **complete framework** with its own
//! `App::new().route(...).run()` and a built-in Hyper server, not a set of
//! axum-compatible layers. Adopting its server would mean re-hosting
//! better-auth's axum `Router` inside it — a novel integration nobody has
//! proven, and exactly the risk D6 avoided elsewhere.
//!
//! So: `apps/api` is plain `axum::serve` with **allframe used as a library**
//! (`allframe::resilience::KeyedRateLimiter`), which is the shape getformlab
//! uses. That resolves OQ-8 in favour of getformlab's arrangement.

#![forbid(unsafe_code)]

pub mod infrastructure;
pub mod presentation;

pub use crate::infrastructure::{
    error::ApiError,
    state::{AppState, build_state},
};

use std::sync::Arc;

use axum::{Router, middleware, routing::get};
use better_auth::AxumIntegration;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnFailure, TraceLayer},
};
use tracing::Level;

use crate::presentation::handlers;

/// Assemble the full application router.
///
/// The ordering matters and is deliberate:
///
/// 1. `/health` and `/ready` are mounted **outside** the rate limiter, because
///    a probe that gets 429'd takes the service out of the load balancer. They
///    are two endpoints because they answer two questions — see that module.
/// 2. `/auth/*` is better-auth's own router, nested whole. It owns its cookie
///    handling and (when configured) its OAuth callbacks; we do not
///    reimplement any of it.
/// 3. Domain routes sit behind the rate limiter and take `ExtractAuthUser`
///    per-handler.
// No `#[must_use]`: axum's `Router` already carries it, and doubling it is a
// clippy error (`double_must_use`).
pub fn build_router(state: Arc<AppState>) -> Router {
    let auth_router = Arc::clone(&state.auth)
        .axum_router()
        .with_state(Arc::clone(&state.auth));

    let domain_routes = Router::new()
        .route(
            "/posts",
            get(handlers::posts::list).post(handlers::posts::create),
        )
        .route(
            "/posts/{id}",
            get(handlers::posts::get_one)
                .patch(handlers::posts::update)
                .delete(handlers::posts::delete),
        )
        .route(
            "/users/{id}",
            get(handlers::users::get_one).patch(handlers::users::update),
        )
        .route("/me", get(handlers::users::me))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            infrastructure::rate_limit::layer,
        ))
        .with_state(Arc::clone(&state));

    Router::new()
        // Deliberately outside the rate limiter.
        .route("/health", get(handlers::health::health))
        .route("/ready", get(handlers::health::ready))
        .route("/openapi.json", get(handlers::openapi::spec))
        .with_state(Arc::clone(&state))
        .merge(domain_routes)
        .nest("/auth", auth_router)
        // Outermost of the application layers, so it sees every request —
        // including the ones the rate limiter rejects. "How often are we
        // shedding load?" is one of the few questions this exists to answer,
        // and a limiter mounted above it would hide exactly that.
        //
        // With no recorder installed (the default) every macro inside is a
        // no-op, so this is not conditional on metrics being enabled.
        .layer(middleware::from_fn(
            infrastructure::observability::track_requests,
        ))
        .layer(cors_layer(&state.cors_origins))
        // `.level(Level::INFO)` is load-bearing, not cosmetic. `DefaultMakeSpan`
        // creates the per-request span at DEBUG, and the default filter is
        // `info` — so with the stock configuration there was no request span at
        // all, and the OTLP exporter therefore had nothing to export. Traces
        // were "configured" and the collector saw zero services. A span nobody
        // records is indistinguishable from an exporter that does not work.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                // WARN, not the default ERROR. `/ready` answers 503 by design
                // while a dependency is down, and an orchestrator polls it every
                // few seconds — at ERROR that is a page-worthy severity emitted
                // on a timer for a condition already logged, with its own
                // fields, by the handler itself.
                .on_failure(DefaultOnFailure::new().level(Level::WARN)),
        )
}

/// CORS for a credentialed session cookie (§5.3).
///
/// `Access-Control-Allow-Origin: *` is **illegal** alongside
/// `Allow-Credentials: true`, and every request from the Dioxus apps is
/// credentialed (D17). `ServerConfig::from_env` already rejects `*` in
/// `CORS_ORIGINS`; this function is what makes the explicit list take effect.
fn cors_layer(origins: &[String]) -> CorsLayer {
    use axum::http::{HeaderName, HeaderValue, Method, header};

    let parsed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(parsed)
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::COOKIE,
            HeaderName::from_static("x-requested-with"),
        ])
}
