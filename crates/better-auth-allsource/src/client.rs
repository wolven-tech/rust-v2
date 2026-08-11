use crate::error::AllsourceAuthError;
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Low-level HTTP client for Allsource Core and Query Service.
#[derive(Clone)]
pub struct AllsourceClient {
    http: Client,
    core_url: String,
    query_url: String,
    api_key: String,
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
    pub payload: serde_json::Value,
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
    let deleted = event
        .payload
        .get("_deleted")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if deleted {
        None
    } else {
        Some(&event.payload)
    }
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
            .query(&[("entity_id", entity_id), ("limit", "1000")])
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

    /// Query all non-deleted entities of a given type by event_type prefix.
    /// Scans events, groups by entity_id, takes the latest per entity.
    pub async fn query_all<T: DeserializeOwned>(
        &self,
        event_type_prefix: &str,
        limit: usize,
    ) -> Result<Vec<T>, AllsourceAuthError> {
        let url = format!("{}/api/v1/events/query", self.query_url);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[
                ("event_type_prefix", event_type_prefix),
                ("limit", &limit.to_string()),
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
        let mut results = Vec::new();

        // Group by entity_id, take latest per entity (events come sorted by time desc)
        let mut seen = std::collections::HashSet::new();
        for event in &query_resp.events {
            let entity_id = event
                .payload
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if seen.contains(entity_id) {
                continue;
            }
            seen.insert(entity_id.to_string());

            // Skip deleted entities
            if event
                .payload
                .get("_deleted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                continue;
            }

            if let Ok(entity) = serde_json::from_value::<T>(event.payload.clone()) {
                results.push(entity);
            }
        }

        Ok(results)
    }

    /// Search for entities matching a field value using payload filtering.
    pub async fn find_by_field<T: DeserializeOwned>(
        &self,
        event_type_prefix: &str,
        field: &str,
        value: &str,
    ) -> Result<Option<T>, AllsourceAuthError> {
        let url = format!("{}/api/v1/events/query", self.query_url);

        let filter = serde_json::json!({
            "field": format!("payload.{}", field),
            "op": "eq",
            "value": value
        });

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[
                ("event_type_prefix", event_type_prefix),
                ("payload_filter", &filter.to_string()),
                ("limit", "1"),
            ])
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            // If payload_filter is not supported, fall back to scanning
            if status.as_u16() == 400 || status.as_u16() == 422 {
                return self
                    .find_by_field_scan(event_type_prefix, field, value)
                    .await;
            }
            let message = extract_error_message(resp).await;
            return Err(AllsourceAuthError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let query_resp: QueryResponse = resp.json().await?;
        match query_resp.events.first() {
            Some(event) => {
                if event
                    .payload
                    .get("_deleted")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    return Ok(None);
                }
                match serde_json::from_value::<T>(event.payload.clone()) {
                    Ok(entity) => Ok(Some(entity)),
                    Err(_) => {
                        self.find_by_field_scan(event_type_prefix, field, value)
                            .await
                    }
                }
            }
            // payload_filter returned 200 but empty — AllSource may not support
            // the filter syntax. Fall back to in-memory scan.
            None => {
                self.find_by_field_scan(event_type_prefix, field, value)
                    .await
            }
        }
    }

    /// Fallback: scan all entities and filter in-memory.
    async fn find_by_field_scan<T: DeserializeOwned>(
        &self,
        event_type_prefix: &str,
        field: &str,
        value: &str,
    ) -> Result<Option<T>, AllsourceAuthError> {
        let url = format!("{}/api/v1/events/query", self.query_url);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[("event_type_prefix", event_type_prefix), ("limit", "10000")])
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
        let mut seen = std::collections::HashSet::new();

        for event in &query_resp.events {
            let entity_id = event
                .payload
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if seen.contains(entity_id) {
                continue;
            }
            seen.insert(entity_id.to_string());

            if event
                .payload
                .get("_deleted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                continue;
            }

            let field_val = event.payload.get(field).and_then(|v| v.as_str());
            if field_val == Some(value) {
                if let Ok(entity) = serde_json::from_value::<T>(event.payload.clone()) {
                    return Ok(Some(entity));
                }
            }
        }

        Ok(None)
    }

    /// Find all entities matching a field value.
    pub async fn find_all_by_field<T: DeserializeOwned>(
        &self,
        event_type_prefix: &str,
        field: &str,
        value: &str,
    ) -> Result<Vec<T>, AllsourceAuthError> {
        let url = format!("{}/api/v1/events/query", self.query_url);

        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[("event_type_prefix", event_type_prefix), ("limit", "10000")])
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
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for event in &query_resp.events {
            let entity_id = event
                .payload
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if seen.contains(entity_id) {
                continue;
            }
            seen.insert(entity_id.to_string());

            if event
                .payload
                .get("_deleted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                continue;
            }

            let field_val = event.payload.get(field).and_then(|v| v.as_str());
            if field_val == Some(value) {
                if let Ok(entity) = serde_json::from_value::<T>(event.payload.clone()) {
                    results.push(entity);
                }
            }
        }

        Ok(results)
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
        StoredEvent { payload }
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
}
