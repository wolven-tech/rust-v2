//! The variant ↔ AllSource wire-type mapping (D11).
//!
//! Two namespaces collide here and the whole point of this module is to keep
//! them explicitly separate rather than hoping they line up:
//!
//! - AllSource validates event types as **lowercase dot-notation**
//!   (`all-source:docs/current/EVENT_STORE_FEATURES.md:53`).
//! - `#[serde(tag = "type")]` writes the **PascalCase** variant name into the
//!   payload.
//!
//! [`DomainEvent::event_type`] is the single place the dotted string is
//! spelled; [`decode`] is the single place it is checked. There is deliberately
//! **no** fallback that reads the discriminator from the envelope when the
//! payload lacks one (getformlab's `decode_event` does that, and its `.ok()`
//! swallows every decode failure, so a renamed variant degrades to "the
//! projection quietly stops updating").

use crate::{DomainEvent, EventEnvelope};

impl DomainEvent {
    /// The AllSource wire event type. This is the ONLY place the mapping lives.
    #[must_use]
    pub const fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::UserRegistered { .. } => "identity.user.registered",
            DomainEvent::UserProfileUpdated { .. } => "identity.user.profile_updated",
            DomainEvent::PostCreated { .. } => "content.post.created",
            DomainEvent::PostEdited { .. } => "content.post.edited",
            DomainEvent::PostDeleted { .. } => "content.post.deleted",
        }
    }

    /// Every wire type this codebase has ever emitted. **Append-only** — a
    /// string may be added but never removed or changed, because events already
    /// written carry it forever (D12).
    pub const ALL_WIRE_TYPES: &'static [&'static str] = &[
        "identity.user.registered",
        "identity.user.profile_updated",
        "content.post.created",
        "content.post.edited",
        "content.post.deleted",
    ];

    /// Wire types on the `content.post.*` stream family. Used to scope the
    /// `posts_v1` projection worker's subscription.
    pub const POST_WIRE_TYPES: &'static [&'static str] = &[
        "content.post.created",
        "content.post.edited",
        "content.post.deleted",
    ];

    /// Wire types on the `identity.user.*` stream family.
    pub const USER_WIRE_TYPES: &'static [&'static str] =
        &["identity.user.registered", "identity.user.profile_updated"];
}

/// Why an envelope could not be turned into a [`DomainEvent`].
///
/// Every variant is a *real* error that must be visible. Callers log it; they
/// do not swallow it.
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// The payload did not deserialize into any known variant.
    #[error("event payload for `{event_type}` did not deserialize: {source}")]
    Payload {
        event_type: String,
        #[source]
        source: serde_json::Error,
    },
    /// The payload decoded, but to a variant whose wire type is not the one the
    /// envelope claims. That means either a bad writer or a mapping change, and
    /// both are bugs.
    #[error("wire type mismatch: envelope says `{envelope}`, payload decodes as `{payload}`")]
    TypeMismatch {
        envelope: String,
        payload: &'static str,
    },
}

/// Decode strictly.
///
/// The payload's `"type"` tag is the ONLY discriminator we trust; the envelope's
/// `event_type` is then *verified* against it. A mismatch is an error, not a
/// tie-break.
///
/// # Errors
///
/// [`DecodeError::Payload`] if the JSON does not match any variant, and
/// [`DecodeError::TypeMismatch`] if it matches a variant whose wire type
/// differs from the envelope's.
pub fn decode(envelope: &EventEnvelope) -> Result<DomainEvent, DecodeError> {
    let event: DomainEvent =
        serde_json::from_value(envelope.data.clone()).map_err(|source| DecodeError::Payload {
            event_type: envelope.event_type.clone(),
            source,
        })?;
    if event.event_type() != envelope.event_type {
        return Err(DecodeError::TypeMismatch {
            envelope: envelope.event_type.clone(),
            payload: event.event_type(),
        });
    }
    Ok(event)
}

/// Fixtures shared by this crate's unit tests and its integration tests.
///
/// Exposed under `cfg(test)` **and** unconditionally behind the `test-support`
/// path so `tests/` can use it: an integration test links the crate as an
/// external dependency and cannot see `#[cfg(test)]` items.
pub mod test_support {
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use crate::{DomainEvent, EventEnvelope};

    /// A fixed timestamp, so fixtures are byte-stable.
    #[must_use]
    pub fn fixed_time() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-01-15T10:30:00Z")
            .expect("static timestamp is valid RFC 3339")
            .with_timezone(&Utc)
    }

    /// One sample per `DomainEvent` variant.
    ///
    /// **This function must be updated whenever a variant is added.** The
    /// bijection test below is only exhaustive because this list is — the
    /// `match` in it has no wildcard arm, so a new variant fails to compile
    /// here rather than silently going untested.
    #[must_use]
    pub fn sample_of_every_variant() -> Vec<DomainEvent> {
        let user = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0001);
        let post = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0002);
        let at = fixed_time();
        vec![
            DomainEvent::UserRegistered {
                id: user,
                email: "ada@example.com".into(),
                occurred_at: at,
                full_name: Some("Ada Lovelace".into()),
                avatar_url: None,
            },
            DomainEvent::UserProfileUpdated {
                id: user,
                occurred_at: at,
                full_name: Some(Some("Ada King".into())),
                avatar_url: Some(None),
            },
            DomainEvent::PostCreated {
                id: post,
                author_id: user,
                title: "On the Analytical Engine".into(),
                content: "Note G.".into(),
                occurred_at: at,
            },
            DomainEvent::PostEdited {
                id: post,
                occurred_at: at,
                title: Some("On the Analytical Engine (rev. 2)".into()),
                content: None,
            },
            DomainEvent::PostDeleted {
                id: post,
                occurred_at: at,
            },
        ]
    }

    /// Wrap an event in the envelope AllSource would hand back for it.
    #[must_use]
    pub fn envelope_for(event: &DomainEvent) -> EventEnvelope {
        EventEnvelope {
            id: format!("evt-{}", event.event_type()),
            entity_id: event.stream_id(),
            event_type: event.event_type().to_string(),
            data: serde_json::to_value(event).expect("DomainEvent always serializes"),
            metadata: serde_json::Value::Null,
            ingested_at: "2026-08-11T09:00:00Z".to_string(),
            version: Some(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DecodeError, decode,
        test_support::{envelope_for, sample_of_every_variant},
    };
    use crate::DomainEvent;

    /// The load-bearing test for D11.
    #[test]
    fn wire_mapping_is_a_bijection_and_matches_allsource_grammar() {
        for event in sample_of_every_variant() {
            let wire = event.event_type();
            assert!(
                DomainEvent::ALL_WIRE_TYPES.contains(&wire),
                "{wire} missing from ALL_WIRE_TYPES"
            );
            assert!(
                wire.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'),
                "{wire} is not lowercase dot-notation"
            );
            assert_eq!(
                wire.split('.').count(),
                3,
                "{wire} must be <domain>.<entity>.<action>"
            );
            assert_eq!(decode(&envelope_for(&event)).unwrap(), event);
        }

        let mut seen = DomainEvent::ALL_WIRE_TYPES.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            DomainEvent::ALL_WIRE_TYPES.len(),
            "duplicate wire type"
        );

        // Every declared wire type must be produced by some variant: the map is
        // onto, not merely into.
        let produced: Vec<&str> = sample_of_every_variant()
            .iter()
            .map(DomainEvent::event_type)
            .collect();
        for wire in DomainEvent::ALL_WIRE_TYPES {
            assert!(
                produced.contains(wire),
                "{wire} is declared but no variant emits it"
            );
        }
    }

    /// The subscription lists must stay subsets of the master list, or a worker
    /// silently subscribes to a type that no longer exists.
    #[test]
    fn family_wire_type_lists_are_subsets_of_the_master_list() {
        for wire in DomainEvent::POST_WIRE_TYPES
            .iter()
            .chain(DomainEvent::USER_WIRE_TYPES)
        {
            assert!(DomainEvent::ALL_WIRE_TYPES.contains(wire), "{wire} unknown");
        }
        assert_eq!(
            DomainEvent::POST_WIRE_TYPES.len() + DomainEvent::USER_WIRE_TYPES.len(),
            DomainEvent::ALL_WIRE_TYPES.len(),
            "every wire type belongs to exactly one family"
        );
    }

    #[test]
    fn a_mismatched_envelope_type_is_an_error_not_a_tie_break() {
        let event = sample_of_every_variant().remove(2); // PostCreated
        let mut envelope = envelope_for(&event);
        envelope.event_type = "content.post.edited".into();
        let err = decode(&envelope).unwrap_err();
        assert!(
            matches!(err, DecodeError::TypeMismatch { .. }),
            "expected TypeMismatch, got {err:?}"
        );
    }

    /// getformlab's decoder patched a missing `"type"` in from the envelope. We
    /// do not: a payload without a discriminator is undecodable, loudly.
    #[test]
    fn a_payload_without_a_type_tag_is_an_error() {
        let event = sample_of_every_variant().remove(4); // PostDeleted
        let mut envelope = envelope_for(&event);
        envelope
            .data
            .as_object_mut()
            .unwrap()
            .remove("type")
            .unwrap();
        let err = decode(&envelope).unwrap_err();
        assert!(
            matches!(err, DecodeError::Payload { .. }),
            "expected Payload, got {err:?}"
        );
    }

    /// D12, the expensive-to-fix-later rule: an event written by an *older*
    /// build — one that predates a field being added — must still deserialize.
    ///
    /// The corpus in `tests/golden/` covers this against captured bytes; this
    /// test states the rule inline so it fails in the same file that would be
    /// edited to break it.
    #[test]
    fn older_payloads_still_deserialize_after_a_field_is_added() {
        // `full_name` and `avatar_url` were added to `UserRegistered` after v1.
        // This is what a v1 writer emitted — no such keys at all.
        let v1 = serde_json::json!({
            "type": "UserRegistered",
            "id": "00000000-0000-0000-0000-000000000001",
            "email": "ada@example.com",
            "occurred_at": "2026-01-15T10:30:00Z"
        });
        let envelope = crate::EventEnvelope {
            id: "evt-old".into(),
            entity_id: "user:00000000-0000-0000-0000-000000000001".into(),
            event_type: "identity.user.registered".into(),
            data: v1,
            metadata: serde_json::Value::Null,
            ingested_at: "2026-01-15T10:30:01Z".into(),
            version: Some(1),
        };
        let decoded = decode(&envelope).expect("a v1 payload must decode into the current build");
        match decoded {
            DomainEvent::UserRegistered {
                full_name,
                avatar_url,
                ..
            } => {
                assert_eq!(full_name, None);
                assert_eq!(avatar_url, None);
            }
            other => panic!("decoded to the wrong variant: {other:?}"),
        }
    }

    /// Unknown *keys* must be tolerated too: a newer writer rolled out ahead of
    /// a reader adds fields the reader has never heard of.
    #[test]
    fn newer_payloads_with_unknown_fields_still_deserialize() {
        let mut envelope = envelope_for(&sample_of_every_variant().remove(4));
        envelope
            .data
            .as_object_mut()
            .unwrap()
            .insert("redacted_by".into(), serde_json::json!("moderator-7"));
        decode(&envelope).expect("forward compatibility: unknown keys are ignored");
    }
}
