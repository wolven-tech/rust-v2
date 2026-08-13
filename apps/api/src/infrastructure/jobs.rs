//! The periodic jobs this service registers.
//!
//! One place, so "what runs in the background?" is answerable by reading a
//! single function rather than by grepping for `tokio::spawn`.
//!
//! Read `rv2_jobs`'s module docs before adding anything here. The constraint
//! that matters: **every instance runs every job**, so a job whose second
//! concurrent execution would be a defect does not belong in this scheduler.
//! Everything below is idempotent and safe to miss.

use std::{sync::Arc, time::Duration};

use rv2_jobs::Scheduler;

use crate::infrastructure::state::AppState;

/// How often dependency gauges are refreshed.
///
/// Slow on purpose. This is the only thing in the service that polls Core
/// without being asked to, so it should stay far below the noise floor of real
/// traffic — and a gauge that is up to half a minute stale is a perfectly good
/// answer to "is the projection keeping up?".
const DEPENDENCY_REFRESH: Duration = Duration::from_secs(30);

/// Register and start every background job.
///
/// Returns the scheduler so `main` can shut it down; dropping it instead would
/// leave the tasks running until the process died, which is exactly the
/// mid-write cut-off that graceful shutdown exists to avoid.
pub fn start(state: Arc<AppState>) -> Scheduler {
    let mut scheduler = Scheduler::new();

    // Why a job rather than measuring this inside `/ready`: readiness is polled
    // by the orchestrator, on a schedule this service does not control and
    // cannot see. Deriving a *metric* from it would make the graph's resolution
    // depend on the probe interval, and would show nothing at all in a
    // deployment that has no probe configured.
    scheduler.every("dependency_health", DEPENDENCY_REFRESH, move || {
        let state = Arc::clone(&state);
        async move {
            let reachable = state.allsource.core.health().await.is_ok();
            metrics::gauge!("allsource_reachable").set(f64::from(u8::from(reachable)));

            if let Some(handle) = state.posts_handle() {
                metrics::gauge!(
                    "projection_caught_up",
                    "worker" => rv2_allsource::workers::POSTS_WORKER_NAME,
                )
                .set(f64::from(u8::from(handle.is_caught_up())));
            }

            Ok(())
        }
    });

    scheduler
}
