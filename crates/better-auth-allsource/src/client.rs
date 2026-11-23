use crate::error::AllsourceAuthError;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Core's `/api/v1/events/query` scopes every read to a tenant and returns an
/// EMPTY result set — `{"events":[],"count":0}`, HTTP 200, no error — when the
/// `tenant_id` parameter is absent. Every query below must therefore carry it,
/// or session lookups silently find nothing and every request 401s.
///
/// The Query Service derives the tenant from the API key, which is why traffic
/// through the gateway does not need this. Talking to Core directly does.
const DEFAULT_TENANT: &str = "default";

/// How many events one family scan reads.
const SCAN_LIMIT: &str = "10000";

/// Low-level HTTP client for Allsource Core and Query Service.
#[derive(Clone)]
pub struct AllsourceClient {
    http: Client,
    core_url: String,
    query_url: String,
    api_key: String,
    tenant_id: String,
}

#[derive(Debug, Serialize)]
struct IngestEvent<'a> {
    entity_id: &'a str,
    event_type: &'a str,
    payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct QueryResponse {
    events: Vec<StoredEvent>,
}

#[derive(Debug, Deserialize)]
pub struct StoredEvent {
    #[serde(default)]
    pub entity_id: String,
    #[serde(default)]
    pub timestamp: String,
    pub payload: serde_json::Value,
}

fn is_tombstone(payload: &serde_json::Value) -> bool {
    payload
        .get("_deleted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Given an entity's events in AllSource order (OLDEST-first), return the
/// current-state payload: the LAST event's payload, or `None` if the stream is
/// empty or the latest event is a delete tombstone (`_deleted: true`).
///
/// Regression guard: a previous version took `.first()` (with `limit=1`), which
/// returned the `*.created` event and silently dropped every later
/// `*.updated`/delete — breaking user role updates and session sign-out.
fn latest_live_payload(events: &[StoredEvent]) -> Option<&serde_json::Value> {
    let event = events.last()?;
    if is_tombstone(&event.payload) {
        None
    } else {
        Some(&event.payload)
    }
}

/// The event-type prefix covering every event of an entity family.
///
/// Callers name a family by its `*.created` type. Scanning only that type
/// returns each entity's first snapshot and never its updates or tombstones:
/// consumed verification tokens stayed redeemable, deleted entities stayed
/// listed, and changed fields (email, expiry, password hash) read stale.
fn family_prefix(event_type: &str) -> &str {
    event_type
        .strip_suffix("created")
        .filter(|family| family.ends_with('.'))
        .unwrap_or(event_type)
}

/// The current state of every live entity in `events`: per `entity_id`, the
/// newest payload by timestamp, unless that payload is a tombstone.
///
/// Entities come back in the order they first appear. Timestamps compare as
/// strings, which orders Core's uniform RFC 3339 UTC stamps correctly; the sort
/// is stable, so equal stamps keep Core's order.
fn current_states(events: Vec<StoredEvent>) -> Vec<serde_json::Value> {
    let mut order: Vec<String> = Vec::new();
    let mut streams: std::collections::HashMap<String, Vec<StoredEvent>> =
        std::collections::HashMap::new();
    for event in events {
        let key = if event.entity_id.is_empty() {
            event
                .payload
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string()
        } else {
            event.entity_id.clone()
        };
        let stream = streams.entry(key.clone()).or_insert_with(|| {
            order.push(key);
            Vec::new()
        });
        stream.push(event);
    }

    order
        .into_iter()
        .filter_map(|key| {
            let mut stream = streams.remove(&key)?;
            stream.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            latest_live_payload(&stream).cloned()
        })
        .collect()
}

impl AllsourceClient {
    pub fn new(core_url: &str, query_url: &str, api_key: &str) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");

        Self {
            http,
            core_url: core_url.trim_end_matches('/').to_string(),
            query_url: query_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            tenant_id: std::env::var("ALLSOURCE_TENANT_ID")
                .unwrap_or_else(|_| DEFAULT_TENANT.to_string()),
        }
    }

    /// Append an event to Allsource Core.
    pub async fn append_event(
        &self,
        entity_id: &str,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<(), AllsourceAuthError> {
        let url = format!("{}/api/v1/events", self.core_url);
        let event = IngestEvent {
            entity_id,
            event_type,
            payload,
        };

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&event)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let message = extract_error_message(resp).await;
            return Err(AllsourceAuthError::Api {
                status: status.as_u16(),
                message,
            });
        }

        Ok(())
    }

    /// Query the latest event for an entity and deserialize the payload.
    pub async fn get_latest<T: DeserializeOwned>(
        &self,
        entity_id: &str,
    ) -> Result<Option<T>, AllsourceAuthError> {
        self.get_latest_raw(entity_id)
            .await
            .map(|opt| opt.map(|(entity, _raw)| entity))
    }

    /// Like `get_latest` but also returns the raw payload JSON.
    /// Needed because `User.metadata` has `#[serde(skip)]` and must be
    /// restored manually from the stored payload.
    pub async fn get_latest_raw<T: DeserializeOwned>(
        &self,
        entity_id: &str,
    ) -> Result<Option<(T, serde_json::Value)>, AllsourceAuthError> {
        let url = format!("{}/api/v1/events/query", self.query_url);

        // AllSource's query endpoint returns an entity's events OLDEST-first.
        // The current state is therefore the LAST event, not the first. The old
        // `limit=1` + `.first()` returned the `*.created` event and silently
        // ignored every later `*.updated`/delete — which dropped user role
        // changes (multi-role never accumulated) and made sign-out's delete
        // tombstone invisible (sessions stayed valid). Fetch the whole stream
        // (per-entity streams are tiny: created + a handful of updates) and take
        // the newest. The high cap guards against an unbounded fetch.
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[
                ("entity_id", entity_id),
                ("tenant_id", &self.tenant_id),
                ("limit", "1000"),
            ])
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let message = extract_error_message(resp).await;
            return Err(AllsourceAuthError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let query_resp: QueryResponse = resp.json().await?;

        match latest_live_payload(&query_resp.events) {
            Some(payload) => {
                let entity: T = serde_json::from_value(payload.clone())?;
                Ok(Some((entity, payload.clone())))
            }
            None => Ok(None),
        }
    }

    /// The current state of every live entity in the family `event_type`
    /// names.
    async fn scan_family(
        &self,
        event_type: &str,
    ) -> Result<Vec<serde_json::Value>, AllsourceAuthError> {
        let url = format!("{}/api/v1/events/query", self.query_url);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[
                ("event_type_prefix", family_prefix(event_type)),
                ("tenant_id", &self.tenant_id),
                ("limit", SCAN_LIMIT),
            ])
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let message = extract_error_message(resp).await;
            return Err(AllsourceAuthError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let query_resp: QueryResponse = resp.json().await?;
        Ok(current_states(query_resp.events))
    }

    /// Every live entity of the family `event_type` names, in its current
    /// state, at most `limit`.
    pub async fn query_all<T: DeserializeOwned>(
        &self,
        event_type: &str,
        limit: usize,
    ) -> Result<Vec<T>, AllsourceAuthError> {
        Ok(self
            .scan_family(event_type)
            .await?
            .into_iter()
            .filter_map(|payload| serde_json::from_value::<T>(payload).ok())
            .take(limit)
            .collect())
    }

    /// The first live entity whose CURRENT `field` equals `value`.
    ///
    /// Matches against current state, never a past snapshot: an email changed
    /// away from `value` no longer matches, and one changed to it does.
    pub async fn find_by_field<T: DeserializeOwned>(
        &self,
        event_type: &str,
        field: &str,
        value: &str,
    ) -> Result<Option<T>, AllsourceAuthError> {
        Ok(self
            .find_all_by_field(event_type, field, value)
            .await?
            .into_iter()
            .next())
    }

    /// Every live entity whose CURRENT `field` equals `value`.
    pub async fn find_all_by_field<T: DeserializeOwned>(
        &self,
        event_type: &str,
        field: &str,
        value: &str,
    ) -> Result<Vec<T>, AllsourceAuthError> {
        Ok(self
            .scan_family(event_type)
            .await?
            .into_iter()
            .filter(|payload| payload.get(field).and_then(|v| v.as_str()) == Some(value))
            .filter_map(|payload| serde_json::from_value::<T>(payload).ok())
            .collect())
    }

    /// Append a deletion marker event.
    pub async fn append_delete(
        &self,
        entity_id: &str,
        event_type: &str,
    ) -> Result<(), AllsourceAuthError> {
        self.append_event(
            entity_id,
            event_type,
            serde_json::json!({ "_deleted": true, "id": entity_id }),
        )
        .await
    }
}

/// Extract a human-readable error message from an API error response.
///
/// Core returns `{"error": "..."}`, so we parse the JSON and extract the
/// `error` field. Falls back to the raw response body if parsing fails.
async fn extract_error_message(resp: reqwest::Response) -> String {
    let body = resp.text().await.unwrap_or_default();
    // Try to extract structured error from Core's JSON response
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        if let Some(msg) = json.get("error").and_then(|e| e.as_str()) {
            return msg.to_string();
        }
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ev(payload: serde_json::Value) -> StoredEvent {
        StoredEvent {
            entity_id: String::new(),
            timestamp: String::new(),
            payload,
        }
    }

    fn at(entity_id: &str, timestamp: &str, payload: serde_json::Value) -> StoredEvent {
        StoredEvent {
            entity_id: entity_id.into(),
            timestamp: timestamp.into(),
            payload,
        }
    }

    /// AllSource returns events OLDEST-first, so the current state is the LAST
    /// event — not the `*.created` first one. This is the regression that broke
    /// multi-role and logout.
    #[test]
    fn latest_live_payload_takes_newest_update_not_created() {
        let events = vec![
            ev(json!({"id": "u1", "role": null})),            // created
            ev(json!({"id": "u1", "role": "coach"})),         // update
            ev(json!({"id": "u1", "role": "coach,trainee"})), // update (newest)
        ];
        let payload = latest_live_payload(&events).expect("has a live payload");
        assert_eq!(payload.get("role").unwrap(), "coach,trainee");
    }

    #[test]
    fn latest_live_payload_is_none_when_latest_is_delete_tombstone() {
        let events = vec![
            ev(json!({"id": "u1", "role": "coach"})),
            ev(json!({"id": "u1", "_deleted": true})), // tombstone is newest
        ];
        assert!(latest_live_payload(&events).is_none());
    }

    #[test]
    fn latest_live_payload_is_none_for_empty_stream() {
        assert!(latest_live_payload(&[]).is_none());
    }

    #[test]
    fn a_created_type_scans_its_whole_family() {
        assert_eq!(
            family_prefix("auth.verification.created"),
            "auth.verification."
        );
        assert_eq!(family_prefix("auth.two_factor.created"), "auth.two_factor.");
        assert_eq!(family_prefix("auth.user."), "auth.user.");
        assert_eq!(family_prefix("recreated"), "recreated");
    }

    /// The tombstone's payload `id` is the stream id, not the entity id, so
    /// grouping must go by the event's `entity_id`.
    #[test]
    fn a_consumed_verification_is_no_longer_current() {
        let events = vec![
            at(
                "auth-verification:v1",
                "2026-11-23T10:00:00Z",
                json!({"id": "v1", "value": "tok"}),
            ),
            at(
                "auth-verification:v1",
                "2026-11-23T10:05:00Z",
                json!({"id": "auth-verification:v1", "_deleted": true}),
            ),
        ];
        assert!(current_states(events).is_empty());
    }

    #[test]
    fn current_state_is_the_newest_snapshot_whatever_the_scan_order() {
        let events = vec![
            at(
                "auth-user:u1",
                "2026-11-23T10:05:00Z",
                json!({"id": "u1", "email": "new@x.com"}),
            ),
            at(
                "auth-user:u2",
                "2026-11-23T09:00:00Z",
                json!({"id": "u2", "email": "b@x.com"}),
            ),
            at(
                "auth-user:u1",
                "2026-11-23T10:00:00Z",
                json!({"id": "u1", "email": "old@x.com"}),
            ),
        ];
        let states = current_states(events);
        assert_eq!(states.len(), 2);
        assert_eq!(states[0]["email"], "new@x.com");
        assert_eq!(states[1]["email"], "b@x.com");
    }
}
