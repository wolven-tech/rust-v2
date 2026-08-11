//! The AllSource integration layer.
//!
//! **Layer 2, SERVER-ONLY.** `allsource` pulls `reqwest` and `tokio`, so
//! nothing here may be reached from the Dioxus apps.
//!
//! ## What this crate does and deliberately does not do
//!
//! - It **wraps** the official `allsource` SDK v0.23 (`CoreClient`,
//!   `QueryClient`, `ProjectionWorker`, `EventFolder`) — D3.
//! - It **does not** define a `Projection` trait of its own. D14: the SDK's
//!   [`allsource::EventFolder`] is strictly better than a hand-rolled one
//!   (`apply -> bool` distinguishes handled from ignored; `finalize ->
//!   Option<State>` gives "this entity does not exist" for free). getformlab
//!   hand-rolled theirs only because they pin `allframe 0.1.12` and the SDK did
//!   not exist yet.
//! - It **does not** hard-code a port. Both URLs come from `rv2-shared`'s
//!   `ServerConfig`, which requires them from the environment (R6).

#![forbid(unsafe_code)]

pub mod folders;
pub mod tenant_query;
pub mod workers;
pub mod writer;

use allsource::{CoreClient, QueryClient};
use rv2_events::EventEnvelope;

pub use crate::{
    folders::{PostFolder, UserFolder},
    workers::{PostsReadModel, start_posts_worker},
    writer::{AppendError, EventWriter},
};

/// Re-exported so `apps/api` does not need its own `allsource` dependency just
/// to name these in a signature.
pub use allsource::{Error as SdkError, Event as SdkEvent, ProjectionHandle, QueryEventsParams};

/// Both SDK clients, constructed once and cloned.
///
/// The SDK's own docs are explicit that cloning is cheap (the `reqwest` client
/// and its connection pool are shared via `Arc`) and that constructing a fresh
/// client per request is an anti-pattern. So this is a plain `Clone` struct,
/// not an `Arc<Mutex<…>>`.
#[derive(Debug, Clone)]
pub struct AllSource {
    /// Writes and projection work. Points at Core (`:3900` by convention).
    pub core: CoreClient,
    /// Reads. Points at the Query Service (`:3902` by convention) — but in a
    /// single-node dev stack it may point at Core, because
    /// `QueryClient::query_events` calls Core's own `/api/v1/events/query`.
    /// See the README's "one Core, no Query Service" note.
    pub query: QueryClient,
    /// Kept so [`AllSource::fold_entity`] can issue tenant-scoped reads, which
    /// the SDK cannot express. See [`tenant_query`].
    query_url: String,
    api_key: String,
}

impl AllSource {
    /// Build both clients.
    ///
    /// # Errors
    ///
    /// [`SdkError`] if either URL or the API key is empty — the SDK validates
    /// this at construction, which is why `ALLSOURCE_API_KEY` is required even
    /// when Core runs in dev mode and ignores its value.
    pub fn new(core_url: &str, query_url: &str, api_key: &str) -> Result<Self, SdkError> {
        Ok(Self {
            core: CoreClient::new(core_url, api_key)?,
            query: QueryClient::new(query_url, api_key)?,
            query_url: query_url.to_string(),
            api_key: api_key.to_string(),
        })
    }

    /// Read one entity's events and fold them, scoped to the tenant.
    ///
    /// Prefer this over [`QueryClient::query_and_fold`] whenever
    /// `ALLSOURCE_QUERY_URL` may point at Core: the SDK cannot send
    /// `tenant_id`, and Core answers a tenant-less query with an empty result
    /// set rather than an error. See [`tenant_query`].
    ///
    /// # Errors
    ///
    /// [`reqwest::Error`] if the query endpoint is unreachable or fails.
    pub async fn fold_entity<F: allsource::EventFolder>(
        &self,
        entity_id: &str,
    ) -> Result<Option<F::State>, reqwest::Error> {
        tenant_query::fold_entity::<F>(&self.query_url, &self.api_key, entity_id).await
    }

    /// A writer over the Core client.
    #[must_use]
    pub fn writer(&self) -> EventWriter {
        EventWriter::new(self.core.clone())
    }
}

/// Translate the SDK's `Event` into our `EventEnvelope`.
///
/// The two are near-identical by design — our envelope was defined to mirror
/// AllSource's wire model — but they are separate types on purpose:
/// `rv2-events` is WASM-safe and must not depend on the SDK, so the conversion
/// lives here, on the server side of the boundary.
#[must_use]
pub fn envelope_from_sdk(event: &SdkEvent) -> EventEnvelope {
    EventEnvelope {
        id: event.id.clone(),
        entity_id: event.entity_id.clone(),
        event_type: event.event_type.clone(),
        data: event.payload.clone(),
        metadata: event.metadata.clone(),
        ingested_at: event.timestamp.clone(),
        version: event.version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clients_reject_empty_configuration_at_construction() {
        assert!(AllSource::new("", "http://localhost:3902", "k").is_err());
        assert!(AllSource::new("http://localhost:3900", "", "k").is_err());
        assert!(
            AllSource::new("http://localhost:3900", "http://localhost:3902", "").is_err(),
            "an empty API key must fail at construction, not at first request"
        );
        assert!(AllSource::new("http://localhost:3900", "http://localhost:3902", "ask_x").is_ok());
    }

    #[test]
    fn sdk_events_convert_field_for_field() {
        let sdk = SdkEvent {
            id: "evt-1".into(),
            event_type: "content.post.created".into(),
            entity_id: "post:abc".into(),
            payload: serde_json::json!({"type": "PostCreated"}),
            metadata: serde_json::json!({"source": "test"}),
            timestamp: "2026-08-11T09:00:00Z".into(),
            version: Some(3),
            tenant_id: Some("default".into()),
        };
        let env = envelope_from_sdk(&sdk);
        assert_eq!(env.id, "evt-1");
        assert_eq!(env.entity_id, "post:abc");
        assert_eq!(env.event_type, "content.post.created");
        assert_eq!(env.ingested_at, "2026-08-11T09:00:00Z");
        assert_eq!(env.version, Some(3));
        assert_eq!(env.data, sdk.payload);
    }
}
