//! Periodic background work.
//!
//! **Layer 2, SERVER-ONLY.** Spawns tokio tasks; nothing here may be reached
//! from the Dioxus apps. The `wasm32 boundary` check enforces that.
//!
//! ## What this is, and — more importantly — what it is not
//!
//! rust-v1 used trigger.dev. Nothing replaced it, and the honest options were
//! recorded rather than chosen: an in-process scheduler, a durable queue
//! (`apalis`, which needs a backing store this workspace does not have), or an
//! external scheduler hitting an authenticated endpoint.
//!
//! This is the first of those, and it is the smallest thing that is genuinely
//! useful: **periodic work, in the API process, for jobs that are safe to miss
//! and safe to repeat.** Refreshing a gauge, expiring a cache, emitting a
//! heartbeat.
//!
//! It is **not a job queue**, and the difference is not academic:
//!
//! | | This | A durable queue |
//! |---|---|---|
//! | Survives a restart | No — schedules are in memory | Yes |
//! | Runs once across N instances | **No — every instance runs every job** | Yes, via a lease |
//! | Retries a failure | No, it runs again next period | Yes, with backoff |
//! | Can be enqueued at runtime | No, schedules are registered at boot | Yes |
//!
//! The second row is the one that bites. Scale to three instances and every job
//! runs three times per period. That is fine for a gauge refresh and completely
//! wrong for "email the customer", so **do not put a job here whose second
//! execution would be a defect**. When you need one that is, the seam to build
//! against is a leased queue over AllSource — appending `job.claimed` /
//! `job.finished` events gives durability and an audit trail from the store that
//! already exists — and that is a real design, not a line of code.
//!
//! ## What it does get right
//!
//! - **A panicking run does not kill the job.** Each tick runs in its own task,
//!   so a panic is caught at the join and the schedule continues. The naive
//!   version — a `loop` with the work inline — silently stops that job forever,
//!   and the only symptom is a number that quietly stops changing.
//! - **A slow run does not pile up.** Missed ticks are skipped, not queued. A
//!   job that takes longer than its period would otherwise accumulate a backlog
//!   it can never clear, which turns one slow dependency into unbounded
//!   concurrency against it.
//! - **Jobs do not start together.** Each is offset by a deterministic fraction
//!   of its period derived from its name, so a process with ten jobs does not
//!   fire all ten in the same millisecond after every deploy.
//! - **Shutdown is graceful and bounded.** Jobs are told to stop and awaited, so
//!   a run in flight finishes rather than being cut off mid-write.

#![forbid(unsafe_code)]

use std::future::Future;
use std::time::{Duration, Instant};

use tokio::{sync::watch, task::JoinHandle, time::MissedTickBehavior};

/// Whatever a job failed with. Jobs report errors; the scheduler logs and
/// counts them and runs the job again next period.
pub type JobError = Box<dyn std::error::Error + Send + Sync>;

/// Registers periodic jobs and owns their tasks.
pub struct Scheduler {
    stop: watch::Sender<bool>,
    handles: Vec<(&'static str, JoinHandle<()>)>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    #[must_use]
    pub fn new() -> Self {
        let (stop, _) = watch::channel(false);
        Self {
            stop,
            handles: Vec::new(),
        }
    }

    /// Run `job` every `period`, forever, until [`Scheduler::shutdown`].
    ///
    /// `name` labels the metrics and the logs, so it wants to read like a metric
    /// label rather than a sentence: `projection_lag`, not "refresh the lag".
    ///
    /// The closure is called once per tick and must be re-callable — it is `Fn`,
    /// not `FnOnce`, which is the type system saying the same thing.
    ///
    /// # Panics
    ///
    /// Never. A panic inside `job` is caught at the task boundary, counted as a
    /// failed run, and the schedule continues.
    pub fn every<F, Fut>(&mut self, name: &'static str, period: Duration, job: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), JobError>> + Send + 'static,
    {
        let mut stop = self.stop.subscribe();
        let offset = start_offset(name, period);

        let handle = tokio::spawn(async move {
            tracing::info!(job = name, ?period, ?offset, "job scheduled");

            // The first tick of a tokio interval fires immediately, so the
            // offset is applied by starting the interval late rather than by
            // sleeping inside the loop.
            let mut ticker = tokio::time::interval_at(tokio::time::Instant::now() + offset, period);
            // Skip, do not burst. See the module docs: queued missed ticks turn
            // one slow run into unbounded concurrency against whatever was slow.
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    // Biased so a shutdown that arrives at the same moment as a
                    // tick wins. Otherwise the last act of a draining process is
                    // to start one more unit of work.
                    biased;
                    _ = stop.changed() => break,
                    _ = ticker.tick() => run_once(name, &job).await,
                }
            }

            tracing::info!(job = name, "job stopped");
        });

        self.handles.push((name, handle));
    }

    /// How many jobs are registered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Stop every job and wait for in-flight runs to finish.
    ///
    /// Awaited rather than aborted: a job cut off mid-run has done half of
    /// whatever it does, and half of an append is the kind of thing that is
    /// discovered months later.
    pub async fn shutdown(self) {
        if self.handles.is_empty() {
            return;
        }
        tracing::info!(jobs = self.handles.len(), "stopping background jobs");

        // A send error means every receiver is already gone, which is the state
        // this is trying to reach.
        let _ = self.stop.send(true);

        for (name, handle) in self.handles {
            if let Err(error) = handle.await {
                tracing::warn!(job = name, %error, "job task did not stop cleanly");
            }
        }
    }
}

/// One run, isolated.
///
/// The inner `tokio::spawn` is what makes a panic survivable: it moves the call
/// onto its own task, so unwinding stops at the join instead of taking out the
/// loop that schedules it.
async fn run_once<F, Fut>(name: &'static str, job: &F)
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), JobError>> + Send + 'static,
{
    let started = Instant::now();
    let outcome = match tokio::spawn(job()).await {
        Ok(Ok(())) => "ok",
        Ok(Err(error)) => {
            tracing::warn!(job = name, %error, "job run failed");
            "error"
        }
        Err(error) => {
            // A panic here has already been reported by the default hook; this
            // records that the *job* survived it, which is the part that is not
            // obvious from the panic message.
            tracing::error!(job = name, %error, "job run panicked; the schedule continues");
            "panic"
        }
    };

    metrics::counter!("job_runs_total", "job" => name, "outcome" => outcome).increment(1);
    metrics::histogram!("job_duration_seconds", "job" => name)
        .record(started.elapsed().as_secs_f64());
}

/// A stable per-job offset in `[0, period)`.
///
/// Derived from the name rather than randomly so that a restart does not
/// reshuffle every schedule, and so this crate needs no source of randomness.
/// The goal is only to stop N jobs firing in the same millisecond after every
/// deploy — spreading them is what matters, not being unpredictable.
fn start_offset(name: &str, period: Duration) -> Duration {
    // FNV-1a. Small, dependency-free, and more than good enough for spreading
    // a handful of names across a window.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let millis = period.as_millis().max(1);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the modulus is bounded by period.as_millis(), which fits u64 for any \
                  period a scheduler would accept"
    )]
    Duration::from_millis((u128::from(hash) % millis) as u64)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn the_start_offset_is_inside_the_period_and_stable() {
        let period = Duration::from_secs(30);
        let first = start_offset("projection_lag", period);
        assert!(first < period);
        assert_eq!(
            first,
            start_offset("projection_lag", period),
            "the offset must not change across restarts"
        );
    }

    /// Different jobs must not land on the same instant, or the whole point of
    /// offsetting is lost.
    #[test]
    fn different_jobs_get_different_offsets() {
        let period = Duration::from_secs(60);
        assert_ne!(
            start_offset("projection_lag", period),
            start_offset("session_sweep", period)
        );
    }

    #[tokio::test]
    async fn a_job_runs_repeatedly_and_stops_on_shutdown() {
        let runs = Arc::new(AtomicUsize::new(0));

        let mut scheduler = Scheduler::new();
        let counter = Arc::clone(&runs);
        scheduler.every("test_tick", Duration::from_millis(10), move || {
            let counter = Arc::clone(&counter);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
        assert_eq!(scheduler.len(), 1);

        tokio::time::sleep(Duration::from_millis(120)).await;
        scheduler.shutdown().await;

        let observed = runs.load(Ordering::SeqCst);
        assert!(observed >= 2, "job ran {observed} times; expected repeats");

        // Nothing may run after shutdown returns. That is the whole contract of
        // awaiting the handles rather than dropping them.
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert_eq!(
            runs.load(Ordering::SeqCst),
            observed,
            "a job ran after shutdown returned"
        );
    }

    /// The property that separates this from a naive `loop`: a panicking run
    /// must not take the schedule with it.
    #[tokio::test]
    async fn a_panicking_run_does_not_kill_the_job() {
        let runs = Arc::new(AtomicUsize::new(0));

        let mut scheduler = Scheduler::new();
        let counter = Arc::clone(&runs);
        scheduler.every("test_panic", Duration::from_millis(10), move || {
            let counter = Arc::clone(&counter);
            async move {
                let n = counter.fetch_add(1, Ordering::SeqCst);
                assert!(n != 0, "deliberate panic on the first run");
                Ok(())
            }
        });

        tokio::time::sleep(Duration::from_millis(120)).await;
        scheduler.shutdown().await;

        assert!(
            runs.load(Ordering::SeqCst) >= 3,
            "the schedule stopped after the panic"
        );
    }

    /// A failing job is a normal state, not a reason to stop scheduling it.
    #[tokio::test]
    async fn a_failing_run_does_not_kill_the_job() {
        let runs = Arc::new(AtomicUsize::new(0));

        let mut scheduler = Scheduler::new();
        let counter = Arc::clone(&runs);
        scheduler.every("test_error", Duration::from_millis(10), move || {
            let counter = Arc::clone(&counter);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err("nope".into())
            }
        });

        tokio::time::sleep(Duration::from_millis(120)).await;
        scheduler.shutdown().await;

        assert!(runs.load(Ordering::SeqCst) >= 3);
    }

    #[tokio::test]
    async fn shutting_down_an_empty_scheduler_is_fine() {
        Scheduler::new().shutdown().await;
    }
}
