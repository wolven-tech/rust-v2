//! `GET /health`.
//!
//! Deliberately mounted outside the rate limiter (see `build_router`): a health
//! probe that gets 429'd pulls the instance out of the load balancer.
//!
//! The check reports AllSource reachability but still answers **200** when Core
//! is down. That is intentional — this is a liveness probe, and returning 503
//! because a *dependency* is unhealthy makes the orchestrator kill a process
//! that would have recovered on its own. Readiness is a separate concern; when
//! it is added, that is where `is_caught_up()` belongs.

use std::sync::Arc;

use axum::{Json, extract::State};
use rv2_api_types::HealthResponse;

use crate::infrastructure::state::AppState;

pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    let allsource_reachable = state.allsource.core.health().await.is_ok();
    if !allsource_reachable {
        tracing::warn!("AllSource Core did not answer its health check");
    }
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        allsource_reachable,
    })
}
