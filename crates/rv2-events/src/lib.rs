//! Domain events for rust-v2.
//!
//! This crate is **layer 0**: it depends on nothing else in the workspace and
//! is compiled for `wasm32-unknown-unknown` as well as the host. Keep it that
//! way — no `tokio`, no `reqwest`, no `allsource`, no clock reads.
//!
//! Two things live here and nowhere else:
//!
//! 1. [`DomainEvent`] — the closed set of facts this system records.
//! 2. The variant ↔ AllSource wire-type mapping in [`wire`], which is the only
//!    place a wire string is ever spelled out.
//!
//! ## The rules (architecture doc D10-D13)
//!
//! - **D10** — wire event types are `<domain>.<entity>.<action>`, lowercase
//!   dot-notation, exactly three segments.
//! - **D11** — the mapping is explicit and bijective. There is no fallback
//!   decode and no reliance on the SDK's `normalize_event_type`.
//! - **D12** — events are append-only and immutable. Fields may only be
//!   *added*, and every added field carries `#[serde(default)]`. A change that
//!   cannot be expressed additively becomes a new wire type plus a new variant.
//! - **D13** — domain time lives in the payload (`occurred_at`), never in the
//!   envelope. AllSource stamps the envelope server-side at ingest, so envelope
//!   time is *ingest* time and is wrong for anything backdated or migrated.

pub mod wire;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

pub use crate::wire::{DecodeError, decode};

/// Deserializer for a "doubly-sparse" `Option<Option<T>>` patch field.
///
/// **This is not a stylistic choice — without it the encoding is broken**, and
/// it is broken in the direction that silently loses data.
///
/// serde's derived impl for `Option<Option<T>>` maps a JSON `null` to the
/// *outer* `None`, so `Some(None)` ("clear this field") serializes to `null`
/// and then deserializes back as `None` ("leave it alone"). A clear that
/// round-trips through the event store therefore stops being a clear. Caught by
/// `folders::tests::profile_patches_distinguish_clear_from_unchanged` against a
/// real payload, not reasoned about in the abstract.
///
/// The fix pairs with `#[serde(default)]`: an absent key takes the default
/// (`None`); a present key — including an explicit `null` — is deserialized as
/// `Option<T>` and wrapped in `Some`.
///
/// # Errors
///
/// Whatever the inner `Option<T>` deserializer produces.
pub fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Envelope for every event read back out of AllSource.
///
/// Field names mirror AllSource's wire model deliberately. In particular
/// `entity_id` **is** the stream id — there is no separate `stream_id` field
/// (`all-source:docs/allsource-qs-api-reference.md:26`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnvelope {
    /// Server-assigned event id. A `String` rather than a `Uuid` because the
    /// SDK's `allsource::Event::id` is a `String` and we do not want to make
    /// its exact format part of our contract.
    pub id: String,
    /// The stream this event belongs to. See [`stream`].
    pub entity_id: String,
    /// Wire event type: `<domain>.<entity>.<action>`, lowercase dot-notation.
    pub event_type: String,
    /// The `DomainEvent` payload, still as JSON. `decode` turns it into a typed
    /// event.
    pub data: serde_json::Value,
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Server-assigned **ingest** time. NOT domain time — see
    /// `DomainEvent::occurred_at`. Kept as the raw RFC-3339 string the SDK
    /// returns so that a malformed server timestamp cannot make an otherwise
    /// good event undecodable.
    pub ingested_at: String,
    /// Per-entity version assigned by Core. Absent on some read paths.
    #[serde(default)]
    pub version: Option<i64>,
}

impl EventEnvelope {
    /// Parse [`Self::ingested_at`] if it is well-formed RFC 3339.
    ///
    /// Returns `None` rather than erroring: ingest time is observability data,
    /// not domain data, so a bad value must never break a fold (D13).
    #[must_use]
    pub fn ingested_at_utc(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.ingested_at)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }
}

/// The kinds of stream this workspace writes.
///
/// Stream ids are `<entity>:<uuid>` — `user:6f1c…`, `post:9ab2…`. The vendored
/// auth adapter uses `auth-user:` / `auth-session:` for its own private
/// namespace, so domain and auth streams never collide, and `entity_id_prefix`
/// queries work as a cheap keyspace shard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    User,
    Post,
}

impl StreamKind {
    #[must_use]
    pub const fn prefix(self) -> &'static str {
        match self {
            StreamKind::User => "user",
            StreamKind::Post => "post",
        }
    }
}

/// Build a stream (entity) id: `user:<uuid>` / `post:<uuid>`.
#[must_use]
pub fn stream(kind: StreamKind, id: Uuid) -> String {
    format!("{}:{id}", kind.prefix())
}

/// Split a stream id back into its kind prefix and uuid.
///
/// Returns `None` if the id does not have the `<prefix>:<uuid>` shape or the
/// prefix is not one we own.
#[must_use]
pub fn parse_stream(entity_id: &str) -> Option<(StreamKind, Uuid)> {
    let (prefix, rest) = entity_id.split_once(':')?;
    let kind = match prefix {
        "user" => StreamKind::User,
        "post" => StreamKind::Post,
        _ => return None,
    };
    Some((kind, rest.parse().ok()?))
}

/// Top-level domain event.
///
/// `#[serde(tag = "type")]` means the payload itself carries the discriminator
/// under `"type"` — e.g. `{"type":"PostCreated","id":…}`. That tag is
/// PascalCase and is *not* the AllSource wire type; see [`wire`] for the
/// mapping and [`decode`] for the strict reconciliation between the two.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum DomainEvent {
    // ── identity ─────────────────────────────────────────────────────────────
    UserRegistered {
        id: Uuid,
        email: String,
        /// D13: domain time lives in the payload, never in the envelope.
        occurred_at: DateTime<Utc>,
        #[serde(default)]
        full_name: Option<String>,
        #[serde(default)]
        avatar_url: Option<String>,
    },
    UserProfileUpdated {
        id: Uuid,
        occurred_at: DateTime<Utc>,
        /// Sparse patch. Absent key = unchanged; explicit `null` = cleared.
        ///
        /// All three attributes are load-bearing and none may be dropped:
        /// `skip_serializing_if` is what makes "unchanged" absent rather than
        /// `null` (otherwise both states serialize identically),
        /// `deserialize_with` is what stops a present `null` collapsing back to
        /// "unchanged" (see [`double_option`]), and `default` is what turns an
        /// absent key back into "unchanged".
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "double_option"
        )]
        full_name: Option<Option<String>>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "double_option"
        )]
        avatar_url: Option<Option<String>>,
    },

    // ── content ──────────────────────────────────────────────────────────────
    PostCreated {
        id: Uuid,
        author_id: Uuid,
        title: String,
        content: String,
        occurred_at: DateTime<Utc>,
    },
    PostEdited {
        id: Uuid,
        occurred_at: DateTime<Utc>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        content: Option<String>,
    },
    PostDeleted {
        id: Uuid,
        occurred_at: DateTime<Utc>,
    },
}

impl DomainEvent {
    /// The aggregate id this event is about — i.e. the uuid half of its stream.
    #[must_use]
    pub const fn aggregate_id(&self) -> Uuid {
        match self {
            DomainEvent::UserRegistered { id, .. }
            | DomainEvent::UserProfileUpdated { id, .. }
            | DomainEvent::PostCreated { id, .. }
            | DomainEvent::PostEdited { id, .. }
            | DomainEvent::PostDeleted { id, .. } => *id,
        }
    }

    /// Which stream this event belongs on.
    #[must_use]
    pub const fn stream_kind(&self) -> StreamKind {
        match self {
            DomainEvent::UserRegistered { .. } | DomainEvent::UserProfileUpdated { .. } => {
                StreamKind::User
            }
            DomainEvent::PostCreated { .. }
            | DomainEvent::PostEdited { .. }
            | DomainEvent::PostDeleted { .. } => StreamKind::Post,
        }
    }

    /// The full stream id for this event.
    #[must_use]
    pub fn stream_id(&self) -> String {
        stream(self.stream_kind(), self.aggregate_id())
    }

    /// Domain time (D13). Always read time from here, never from the envelope.
    #[must_use]
    pub const fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            DomainEvent::UserRegistered { occurred_at, .. }
            | DomainEvent::UserProfileUpdated { occurred_at, .. }
            | DomainEvent::PostCreated { occurred_at, .. }
            | DomainEvent::PostEdited { occurred_at, .. }
            | DomainEvent::PostDeleted { occurred_at, .. } => *occurred_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::test_support::sample_of_every_variant;

    fn ts(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn serde_round_trips_every_variant() {
        for event in sample_of_every_variant() {
            let json = serde_json::to_value(&event).unwrap();
            let back: DomainEvent = serde_json::from_value(json.clone()).unwrap();
            assert_eq!(event, back, "round-trip failed for {json}");
        }
    }

    /// The regression guard for [`double_option`]. `Some(None)` means "clear
    /// this field"; if it comes back as `None` the fold treats it as "unchanged"
    /// and the user's clear is silently dropped.
    #[test]
    fn an_explicit_clear_survives_a_serde_round_trip() {
        let event = DomainEvent::UserProfileUpdated {
            id: Uuid::nil(),
            occurred_at: ts("2026-01-01T00:00:00Z"),
            full_name: None,        // absent  -> unchanged
            avatar_url: Some(None), // null    -> cleared
        };
        let json = serde_json::to_value(&event).unwrap();
        assert!(
            !json.as_object().unwrap().contains_key("full_name"),
            "an unchanged field must be absent, not null: {json}"
        );
        assert!(
            json.as_object().unwrap().contains_key("avatar_url") && json["avatar_url"].is_null(),
            "a cleared field must be present and null: {json}"
        );
        assert_eq!(
            serde_json::from_value::<DomainEvent>(json).unwrap(),
            event,
            "a cleared field must not decay into an unchanged one"
        );
    }

    #[test]
    fn the_serde_tag_is_the_pascal_case_variant_name() {
        let event = DomainEvent::PostDeleted {
            id: Uuid::nil(),
            occurred_at: ts("2026-01-01T00:00:00Z"),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "PostDeleted");
    }

    #[test]
    fn stream_ids_round_trip() {
        let id = Uuid::new_v4();
        for kind in [StreamKind::User, StreamKind::Post] {
            let s = stream(kind, id);
            assert_eq!(parse_stream(&s), Some((kind, id)));
        }
        assert_eq!(
            parse_stream("auth-user:abc"),
            None,
            "auth namespace is not ours"
        );
        assert_eq!(parse_stream("nonsense"), None);
        assert_eq!(parse_stream("post:not-a-uuid"), None);
    }

    #[test]
    fn stream_kind_matches_the_variant_family() {
        for event in sample_of_every_variant() {
            let expected = match event {
                DomainEvent::UserRegistered { .. } | DomainEvent::UserProfileUpdated { .. } => {
                    "user"
                }
                _ => "post",
            };
            assert!(event.stream_id().starts_with(&format!("{expected}:")));
        }
    }

    /// D13: `occurred_at` is read from the payload, and is deliberately
    /// unrelated to the envelope's ingest timestamp.
    #[test]
    fn domain_time_is_independent_of_ingest_time() {
        let event = DomainEvent::PostCreated {
            id: Uuid::nil(),
            author_id: Uuid::nil(),
            title: "backdated".into(),
            content: "backfilled".into(),
            occurred_at: ts("2024-03-01T12:00:00Z"),
        };
        let envelope = EventEnvelope {
            id: "evt-1".into(),
            entity_id: event.stream_id(),
            event_type: event.event_type().into(),
            data: serde_json::to_value(&event).unwrap(),
            metadata: serde_json::Value::Null,
            ingested_at: "2026-08-11T09:00:00Z".into(),
            version: Some(1),
        };
        let decoded = decode(&envelope).unwrap();
        assert_eq!(decoded.occurred_at(), ts("2024-03-01T12:00:00Z"));
        assert_ne!(
            Some(decoded.occurred_at()),
            envelope.ingested_at_utc(),
            "a migrated event must not read as 'now'"
        );
    }

    #[test]
    fn a_malformed_ingest_timestamp_does_not_break_decoding() {
        let event = DomainEvent::PostDeleted {
            id: Uuid::nil(),
            occurred_at: ts("2026-01-01T00:00:00Z"),
        };
        let envelope = EventEnvelope {
            id: "evt-2".into(),
            entity_id: event.stream_id(),
            event_type: event.event_type().into(),
            data: serde_json::to_value(&event).unwrap(),
            metadata: serde_json::Value::Null,
            ingested_at: "not a timestamp".into(),
            version: None,
        };
        assert_eq!(envelope.ingested_at_utc(), None);
        assert_eq!(decode(&envelope).unwrap(), event);
    }
}
