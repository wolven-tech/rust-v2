//! The write path: `DomainEvent` → AllSource append.
//!
//! This is the mirror image of `rv2_events::decode`, and the only place a
//! `DomainEvent` becomes bytes on the wire.

use allsource::{CoreClient, IngestEventInput, normalize_event_type};
use rv2_events::DomainEvent;

/// Failure appending an event.
#[derive(Debug, thiserror::Error)]
pub enum AppendError {
    #[error("AllSource rejected the append: {0}")]
    Sdk(#[from] allsource::Error),
    #[error("could not serialize event payload: {0}")]
    Serialize(#[from] serde_json::Error),
    /// Defence in depth against the SDK's normalizer silently rewriting a wire
    /// type. See [`EventWriter::append`].
    #[error(
        "wire type `{declared}` is not stable under the SDK's normalizer (would become `{normalized}`)"
    )]
    UnstableWireType {
        declared: &'static str,
        normalized: String,
    },
}

/// Appends `DomainEvent`s to AllSource Core.
#[derive(Debug, Clone)]
pub struct EventWriter {
    core: CoreClient,
}

impl EventWriter {
    #[must_use]
    pub const fn new(core: CoreClient) -> Self {
        Self { core }
    }

    /// Append one event to its own stream.
    ///
    /// The stream id comes from the event itself (`user:<uuid>` /
    /// `post:<uuid>`), so a caller cannot append a post event onto a user
    /// stream by passing the wrong string.
    ///
    /// ## The normalizer check, and why it is here
    ///
    /// `CoreClient::ingest_event` **unconditionally** runs
    /// `normalize_event_type` on the event type before posting — verified in
    /// `allsource-0.23.0/src/client.rs`. The architecture doc (D11) rejects
    /// that normalizer as a schema because it is lossy and non-injective
    /// (`TwoFactorCreated` → `two.factor.created`; `user_created`,
    /// `userCreated`, `UserCreated` and `user-created` all collapse to
    /// `user.created`).
    ///
    /// We cannot switch it off. What we can do is guarantee our grammar is a
    /// **fixed point** of it: a type that already contains dots is only
    /// lowercased, and ours are already lowercase. That makes the normalizer a
    /// no-op for every type we emit — and this check fails the append loudly if
    /// that ever stops being true, rather than writing an event under a name
    /// nothing folds.
    ///
    /// # Errors
    ///
    /// [`AppendError::Serialize`] if the payload will not serialize,
    /// [`AppendError::UnstableWireType`] if the SDK would rewrite our wire type,
    /// and [`AppendError::Sdk`] for a transport or server error.
    pub async fn append(&self, event: &DomainEvent) -> Result<String, AppendError> {
        self.append_with_metadata(event, None).await
    }

    /// Append with provenance metadata — used by `tooling/pg2events`, which
    /// stamps every migrated event with `{"source": "supabase-migration", …}`.
    ///
    /// # Errors
    ///
    /// As [`Self::append`].
    pub async fn append_with_metadata(
        &self,
        event: &DomainEvent,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, AppendError> {
        let declared = event.event_type();
        let normalized = normalize_event_type(declared);
        if normalized != declared {
            return Err(AppendError::UnstableWireType {
                declared,
                normalized,
            });
        }

        let response = self
            .core
            .ingest_event(IngestEventInput {
                event_type: declared.to_string(),
                entity_id: event.stream_id(),
                payload: serde_json::to_value(event)?,
                metadata,
            })
            .await?;
        Ok(response.event_id)
    }
}

#[cfg(test)]
mod tests {
    use allsource::normalize_event_type;
    use rv2_events::DomainEvent;

    /// The single most important property in this file: the SDK's
    /// unconditional normalizer must be the identity on every wire type we
    /// emit. If this fails, `EventWriter::append` starts rejecting appends —
    /// which is the correct behaviour, but this test is what tells you *before*
    /// production does.
    #[test]
    fn our_wire_grammar_is_a_fixed_point_of_the_sdk_normalizer() {
        for wire in DomainEvent::ALL_WIRE_TYPES {
            assert_eq!(
                &normalize_event_type(wire),
                wire,
                "the SDK would rewrite `{wire}` on ingest"
            );
        }
    }

    /// Concretely why D11 rejects the normalizer as a schema: it is not
    /// injective, so it cannot be inverted, so it cannot be a mapping.
    #[test]
    fn the_normalizer_is_lossy_which_is_why_we_do_not_rely_on_it() {
        assert_eq!(
            normalize_event_type("TwoFactorCreated"),
            "two.factor.created"
        );
        for spelling in ["user_created", "userCreated", "UserCreated", "user-created"] {
            assert_eq!(normalize_event_type(spelling), "user.created");
        }
    }
}
