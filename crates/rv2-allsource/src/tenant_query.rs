//! Tenant-scoped reads against Core.
//!
//! ## Why this exists instead of `QueryClient::query_and_fold`
//!
//! Core's `GET /api/v1/events/query` scopes every read to a tenant. Omitting
//! the `tenant_id` parameter does not error — it returns
//! `{"events":[],"count":0}` with HTTP 200. A caller cannot distinguish that
//! from "this entity has no events".
//!
//! The SDK cannot send the parameter: `allsource::QueryEventsParams` has no
//! `tenant_id` field. **Re-checked at v0.24.0 and still true** — that bump
//! added `order` and `payload_filter` to the struct and no tenant. So this
//! module is not deletable yet; check again on the next SDK bump, because the
//! day it gains one is the day this file goes away.
//!
//! The Query Service derives the tenant from the API key, so SDK reads work
//! through the gateway — but a `QueryClient` aimed at Core silently reads
//! nothing, forever.
//!
//! That is not theoretical. It made `POST /posts` return 404: the handler
//! appends, then reads back through the fold to prove the write landed, and the
//! read returned empty. The append had succeeded. `GET /users/{id}` and the
//! `GET /posts` fallback later shipped with the same defect, which is why
//! `apps/api/tests/tenant_reads.rs` forbids the SDK read calls in handlers.
//!
//! So single-node reads go through here, which sends `tenant_id` and folds with
//! the same [`allsource::EventFolder`] the gateway path uses. Point
//! `ALLSOURCE_QUERY_URL` at a real Query Service and the SDK path is fine; this
//! keeps "one Core, no Query Service" honest for local development.

use allsource::{Event, EventFolder};
use serde::Deserialize;

/// Core's default tenant. Overridable for multi-tenant deployments.
pub const DEFAULT_TENANT: &str = "default";

/// How many events one tenant-scoped read asks Core for.
pub const READ_LIMIT: usize = 10_000;

/// A tenant-scoped read failed: Core unreachable, or a non-success status.
///
/// Distinct from "no events": callers map this to 502, never to 404.
#[derive(Debug, thiserror::Error)]
#[error("tenant-scoped read failed: {0}")]
pub struct ReadError(#[from] reqwest::Error);

#[derive(Debug, Deserialize)]
struct QueryResponse {
    events: Vec<Event>,
}

/// Where, and as which tenant, a read goes. One connection pool for all reads.
#[derive(Debug, Clone)]
pub struct TenantReader {
    http: reqwest::Client,
    query_url: String,
    api_key: String,
    tenant: String,
}

impl TenantReader {
    /// A reader scoped to `tenant`.
    #[must_use]
    pub fn new(query_url: &str, api_key: &str, tenant: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            query_url: query_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            tenant: tenant.to_string(),
        }
    }

    /// The tenant every read is scoped to.
    #[must_use]
    pub fn tenant(&self) -> &str {
        &self.tenant
    }

    async fn query(&self, filter: (&str, &str)) -> Result<Vec<Event>, ReadError> {
        let limit = READ_LIMIT.to_string();
        let response: QueryResponse = self
            .http
            .get(format!("{}/api/v1/events/query", self.query_url))
            .header("X-API-Key", &self.api_key)
            .query(&[
                filter,
                // `tenant_id` is the whole point of this module — see the module docs.
                ("tenant_id", self.tenant.as_str()),
                ("limit", limit.as_str()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(response.events)
    }

    /// Read one entity's events from Core and fold them.
    ///
    /// Returns `None` when the entity genuinely has no events — the same
    /// contract as [`allsource::EventFolder::finalize`].
    ///
    /// # Errors
    ///
    /// [`ReadError`] if Core is unreachable or answers with a non-success
    /// status.
    pub async fn fold_entity<F: EventFolder>(
        &self,
        entity_id: &str,
    ) -> Result<Option<F::State>, ReadError> {
        let events = self.query(("entity_id", entity_id)).await?;
        let mut folder = F::default();
        for event in &events {
            folder.apply(event);
        }
        Ok(folder.finalize())
    }

    /// Every event whose type starts with `prefix`, at most [`READ_LIMIT`].
    ///
    /// A result of exactly [`READ_LIMIT`] events may be truncated; the caller
    /// decides what that means.
    ///
    /// # Errors
    ///
    /// [`ReadError`] if Core is unreachable or answers with a non-success
    /// status.
    pub async fn events_by_type_prefix(&self, prefix: &str) -> Result<Vec<Event>, ReadError> {
        self.query(("event_type_prefix", prefix)).await
    }
}
