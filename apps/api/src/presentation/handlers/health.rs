//! `GET /health` (liveness) and `GET /ready` (readiness).
//!
//! Both are mounted outside the rate limiter (see `build_router`): a probe that
//! gets 429'd pulls the instance out of the load balancer for no reason.
//!
//! ## Why these are two endpoints and not one
//!
//! They answer different questions, and the answers differ:
//!
//! - **Liveness** — "is this process wedged?" The only remedy an orchestrator
//!   has is to kill it, so it must depend on nothing external: reporting 503
//!   because *Core* is down gets every instance killed during someone else's
//!   outage, and the restart cannot possibly help.
//! - **Readiness** — "should this instance receive traffic?" Here a dependency
//!   absolutely does count, and the remedy — route around it, leave it
//!   running — is the right one.
//!
//! ## `/health` makes no outbound call, on purpose
//!
//! It used to check Core on every hit. That is one unauthenticated,
//! unrate-limited request in and one request out, so any caller could amplify
//! traffic onto Core through it — and each failure logged a line, so a Core
//! outage also produced a log flood from the probe path. Liveness needs neither
//! the call nor the log.

use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use rv2_allsource::ProjectionHandle;
use rv2_api_types::{HealthResponse, ReadinessResponse};

use crate::infrastructure::state::AppState;

/// Liveness. Always `200` while the process can serve a request — which is
/// exactly what answering this proves.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Readiness. `200` when this instance should be sent traffic, `503` otherwise.
///
/// Two checks, and note which one is deliberately *not* fatal:
///
/// - Core must be reachable. Without it every domain route is a 502.
/// - The projection worker, **if it is running**, must have caught up. During
///   replay the instance serves data that is real but stale, and holding it out
///   of the load balancer for that window is the entire job of this endpoint.
///   A worker that never started is a supported degraded mode — `GET /posts`
///   folds on read — so it reports `null` and does not fail readiness.
pub async fn ready(State(state): State<Arc<AppState>>) -> (StatusCode, Json<ReadinessResponse>) {
    let allsource_reachable = state.allsource.core.health().await.is_ok();
    let projection_caught_up = state.posts_handle().map(ProjectionHandle::is_caught_up);

    // `!= Some(false)` rather than `unwrap_or(true)` so the three states stay
    // visible: running-and-caught-up, running-and-behind, not-running.
    let ready = allsource_reachable && projection_caught_up != Some(false);
    if !ready {
        tracing::warn!(
            allsource_reachable,
            ?projection_caught_up,
            "not ready to serve traffic"
        );
    }

    let status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(ReadinessResponse {
            ready,
            allsource_reachable,
            projection_caught_up,
        }),
    )
}
