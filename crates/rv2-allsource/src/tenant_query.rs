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
//! `tenant_id` field (v0.23.0). The Query Service derives the tenant from the
//! API key, so SDK reads work through the gateway — but a `QueryClient` aimed
//! at Core silently reads nothing, forever.
//!
//! That is not theoretical. It made `POST /posts` return 404: the handler
//! appends, then reads back through the fold to prove the write landed, and the
//! read returned empty. The append had succeeded.
//!
//! So single-node reads go through here, which sends `tenant_id` and folds with
//! the same [`allsource::EventFolder`] the gateway path uses. Point
//! `ALLSOURCE_QUERY_URL` at a real Query Service and the SDK path is fine; this
//! keeps "one Core, no Query Service" honest for local development.

use allsource::{Event, EventFolder};
use serde::Deserialize;

/// Core's default tenant. Overridable for multi-tenant deployments.
const DEFAULT_TENANT: &str = "default";

#[derive(Debug, Deserialize)]
struct QueryResponse {
    events: Vec<Event>,
}

/// Read one entity's events from Core and fold them.
///
/// Returns `None` when the entity genuinely has no events — the same contract
/// as [`allsource::EventFolder::finalize`].
///
/// # Errors
///
/// [`reqwest::Error`] if Core is unreachable or answers with a non-success
/// status.
pub async fn fold_entity<F: EventFolder>(
    query_url: &str,
    api_key: &str,
    entity_id: &str,
) -> Result<Option<F::State>, reqwest::Error> {
    let tenant =
        std::env::var("ALLSOURCE_TENANT_ID").unwrap_or_else(|_| DEFAULT_TENANT.to_string());

    let response: QueryResponse = reqwest::Client::new()
        .get(format!(
            "{}/api/v1/events/query",
            query_url.trim_end_matches('/')
        ))
        .header("X-API-Key", api_key)
        // `tenant_id` is the whole point of this module — see the module docs.
        .query(&[
            ("entity_id", entity_id),
            ("tenant_id", &tenant),
            ("limit", "10000"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let mut folder = F::default();
    for event in &response.events {
        folder.apply(event);
    }
    Ok(folder.finalize())
}
