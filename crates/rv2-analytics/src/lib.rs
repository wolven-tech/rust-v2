//! Product analytics.
//!
//! **Layer 2, SERVER-ONLY.** `posthog-rs` pulls `reqwest`, so nothing here may
//! be reached from the Dioxus apps. The `wasm32 boundary` check enforces that.
//!
//! ## What this replaces
//!
//! rust-v1's `packages/analytics` wrapped `posthog-node` in a `setupAnalytics`
//! factory returning `{ track, shutdown }`, called from the server-action
//! middleware. This is the same shape in Rust, on PostHog's **official** Rust
//! SDK rather than a hand-rolled HTTP client.
//!
//! ## Unconfigured is a supported state, not an error
//!
//! Without `POSTHOG_API_KEY` this is [`Analytics::Disabled`]: every `track`
//! call becomes a `tracing` line. That is deliberate and it is the behaviour
//! rust-v1 had — analytics is not load-bearing, and a missing key must never
//! fail a request that would otherwise have succeeded. The failure mode of "we
//! lost some product metrics" is enormously cheaper than "checkout 500s because
//! the analytics vendor is down".
//!
//! It is also what makes this crate testable without a network or a key: the
//! tests below drive the real code path.
//!
//! ## Tracking is off the request path, and that has two halves
//!
//! [`Analytics::track`] is **synchronous and fire-and-forget**: it hands the
//! event to the SDK's background worker and returns. It is not `async`, so a
//! caller physically cannot await a vendor round-trip.
//!
//! This was not always true, and the bug is worth recording. The first version
//! called `Client::capture_immediate` — whose own docs say it "sends inline"
//! and "retries transient failures per the client's retry configuration", and
//! "prefer fire-and-forget everywhere else". Awaited from a handler, that put
//! PostHog's latency *and its whole retry budget* on the request path, under a
//! comment claiming it never blocked on a vendor. The API was chosen by its
//! name.
//!
//! The second half is the one fire-and-forget always brings with it:
//! **[`Analytics::shutdown`] must be called**, or everything still queued dies
//! with the process. `apps/api` calls it after `axum::serve` returns. rust-v1's
//! `setupAnalytics` returned `{ track, shutdown }` for exactly this reason; the
//! shutdown half is easy to drop and silent when you do.
//!
//! Delivery failures surface through the SDK's `on_error` hook, registered in
//! [`Analytics::configure`], because a queued-and-lost event is otherwise
//! indistinguishable from a delivered one.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;

/// A tracking sink.
///
/// Cheap to clone. `posthog_rs::Client` is not itself `Clone`, so it is held
/// behind an `Arc` — constructing one per request is the anti-pattern its docs
/// warn about, and `AppState` is cloned on every request.
#[derive(Clone)]
pub enum Analytics {
    /// No `POSTHOG_API_KEY`. Events are logged, not sent.
    Disabled,
    PostHog(Arc<posthog_rs::Client>),
}

impl std::fmt::Debug for Analytics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Analytics::Disabled => f.write_str("Analytics::Disabled"),
            Analytics::PostHog(_) => f.write_str("Analytics::PostHog"),
        }
    }
}

/// One tracked event.
///
/// Built separately from sending so a caller can assemble it without holding a
/// client, and so the property map can be asserted in a test without a network.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackedEvent {
    pub name: String,
    /// Stable user identifier. `None` becomes PostHog's anonymous bucket rather
    /// than being dropped — an anonymous signup is still a signup.
    pub distinct_id: Option<String>,
    /// `BTreeMap` rather than `HashMap`: deterministic ordering makes the
    /// logged form stable and the tests meaningful.
    pub properties: BTreeMap<String, Value>,
}

impl TrackedEvent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            distinct_id: None,
            properties: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn actor(mut self, distinct_id: impl Into<String>) -> Self {
        self.distinct_id = Some(distinct_id.into());
        self
    }

    #[must_use]
    pub fn property(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// The id PostHog will bucket this under.
    fn resolved_distinct_id(&self) -> &str {
        self.distinct_id.as_deref().unwrap_or("anonymous")
    }
}

impl Analytics {
    /// Build from the environment.
    ///
    /// `POSTHOG_API_KEY` alone is enough; `POSTHOG_HOST` overrides the default
    /// for self-hosted or EU-resident instances.
    pub async fn from_env() -> Self {
        Self::configure(
            std::env::var("POSTHOG_API_KEY").ok(),
            std::env::var("POSTHOG_HOST").ok(),
        )
        .await
    }

    /// The real constructor, taking its inputs as arguments.
    ///
    /// `from_env` is a thin wrapper over this so the "unconfigured" decision is
    /// testable without mutating process environment — which is global state, is
    /// racy across parallel tests, and is `unsafe` in edition 2024.
    pub async fn configure(api_key: Option<String>, host: Option<String>) -> Self {
        let Some(key) = api_key else {
            tracing::info!("POSTHOG_API_KEY unset; analytics disabled");
            return Analytics::Disabled;
        };
        if key.trim().is_empty() {
            tracing::warn!("POSTHOG_API_KEY is empty; analytics disabled");
            return Analytics::Disabled;
        }

        let mut builder = posthog_rs::ClientOptionsBuilder::default();
        builder.api_key(key);
        // App hosts such as `https://eu.posthog.com` are normalized to
        // ingestion hosts by the SDK, so either form works here.
        if let Some(host) = host.filter(|h| !h.trim().is_empty()) {
            builder.host(host);
        }
        // Fire-and-forget capture reports nothing to the caller, so without this
        // a vendor outage is completely silent. The hook is the *only* delivery
        // signal there is.
        //
        // Log and nothing else. The SDK's own docs are emphatic that a hook must
        // never call back into it — emitting an event while handling a capture
        // failure is an amplification loop — and the hook runs on the transport
        // thread, so it must stay cheap.
        builder.on_error(|error| match error {
            posthog_rs::PostHogError::Capture(failure) => {
                tracing::warn!(
                    events = failure.event_count(),
                    status = ?failure.status(),
                    attempt = failure.attempt(),
                    "analytics batch was not delivered"
                );
            }
            other => tracing::warn!(?other, "analytics error"),
        });

        let options = match builder.build() {
            Ok(options) => options,
            Err(error) => {
                tracing::warn!(%error, "invalid PostHog options; analytics disabled");
                return Analytics::Disabled;
            }
        };

        tracing::info!("analytics enabled");
        Analytics::PostHog(Arc::new(posthog_rs::client(options).await))
    }

    /// Record an event.
    ///
    /// **Not `async`, and that is the contract** — a caller cannot accidentally
    /// await a vendor from a request handler, because there is nothing to await.
    /// The event is queued on the SDK's background worker; delivery failures
    /// arrive at the `on_error` hook registered in [`Analytics::configure`],
    /// never here.
    ///
    /// The queue is bounded. A full queue drops the event with a warning from
    /// the SDK, which is the correct trade: back-pressuring a request path on an
    /// analytics buffer would recreate the very problem this shape avoids.
    pub fn track(&self, event: TrackedEvent) {
        match self {
            Analytics::Disabled => {
                tracing::debug!(
                    event = %event.name,
                    distinct_id = %event.resolved_distinct_id(),
                    properties = ?event.properties,
                    "analytics disabled; event not sent"
                );
            }
            Analytics::PostHog(client) => {
                let mut payload = posthog_rs::Event::new(
                    event.name.clone(),
                    event.resolved_distinct_id().to_string(),
                );
                for (key, value) in &event.properties {
                    if let Err(error) = payload.insert_prop(key.clone(), value.clone()) {
                        tracing::warn!(%error, key, "dropping unserializable analytics property");
                    }
                }
                client.capture(payload);
            }
        }
    }

    /// Flush the queue and stop the background worker.
    ///
    /// **Required.** Fire-and-forget capture means that at any moment some
    /// events exist only in an in-process queue; a process that exits without
    /// this loses all of them, silently. `apps/api` calls it once
    /// `axum::serve` has returned from its graceful shutdown.
    ///
    /// Safe to call on [`Analytics::Disabled`], so a caller needs no branch.
    pub async fn shutdown(&self) {
        match self {
            Analytics::Disabled => {}
            Analytics::PostHog(client) => {
                // No count to log: `pending_events` is gated behind the SDK's
                // `test-harness` feature and is explicitly not public API.
                tracing::info!("flushing queued analytics events");
                client.shutdown().await;
            }
        }
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        matches!(self, Analytics::PostHog(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_event_without_an_actor_is_anonymous_rather_than_dropped() {
        let event = TrackedEvent::new("post_published");
        assert_eq!(event.resolved_distinct_id(), "anonymous");
    }

    #[test]
    fn properties_build_in_deterministic_order() {
        let event = TrackedEvent::new("post_published")
            .actor("user-1")
            .property("z_last", 1)
            .property("a_first", "yes");

        let keys: Vec<_> = event.properties.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec!["a_first", "z_last"],
            "property order is not stable"
        );
        assert_eq!(event.distinct_id.as_deref(), Some("user-1"));
    }

    /// The whole point of the `Disabled` variant: it must be reachable, and
    /// tracking through it must not panic.
    ///
    /// This is a plain `#[test]`, not `#[tokio::test]`, and that is the
    /// assertion — `track` is synchronous, so there is no runtime to need and
    /// no future a handler could be tempted to await. It stops compiling the
    /// moment someone puts a vendor round-trip back on the request path.
    #[test]
    fn tracking_while_disabled_is_a_silent_no_op() {
        let analytics = Analytics::Disabled;
        assert!(!analytics.is_enabled());
        analytics.track(TrackedEvent::new("post_published").actor("user-1"));
    }

    /// `shutdown` must be callable without a branch at the call site, including
    /// on the variant that has nothing to flush — otherwise the one code path
    /// that runs on every deploy is the one nobody exercises.
    #[tokio::test]
    async fn shutdown_is_safe_while_disabled() {
        Analytics::Disabled.shutdown().await;
    }

    /// A missing key must yield `Disabled` rather than panicking or building a
    /// client that fails on every call.
    #[tokio::test]
    async fn a_missing_key_disables_rather_than_failing() {
        assert!(!Analytics::configure(None, None).await.is_enabled());
    }

    /// A key set to the empty string is the same mistake as an unset one, and
    /// is what an unfilled `.env` entry actually produces.
    #[tokio::test]
    async fn a_blank_key_disables_too() {
        assert!(
            !Analytics::configure(Some("   ".to_string()), None)
                .await
                .is_enabled()
        );
    }
}
