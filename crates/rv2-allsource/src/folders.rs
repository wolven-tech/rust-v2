//! [`allsource::EventFolder`] implementations (D14).
//!
//! ## The constraint that shapes everything here
//!
//! A read-model rebuild replays the **entire** store (§4.4), so folders must be
//! **pure and total**: no clock reads, no network calls, no `unwrap()` on
//! payload shape, and `apply` must be idempotent for an event it has already
//! seen. The SDK worker does per-entity version dedup, but that only covers
//! replay overlap; logical idempotence is the folder's job.
//!
//! Domain time comes from the payload's `occurred_at`, never from
//! `Event::timestamp` (D13). getformlab reads `created_at: event.timestamp`
//! from the envelope; had we copied that, every migrated post would carry its
//! ingest instant instead of its real creation date.

use allsource::{Event, EventFolder};
use rv2_api_types::{PostView, UserView};
use rv2_events::{DomainEvent, decode};

use crate::envelope_from_sdk;

/// Decode an SDK event, logging (never swallowing) a failure.
///
/// Returning `None` here is how a folder ignores an event it cannot read. The
/// difference from getformlab's `.ok()` is the `warn!`: an undecodable event is
/// visible in the logs instead of manifesting weeks later as "the projection
/// quietly stopped updating".
fn decode_or_warn(event: &Event) -> Option<DomainEvent> {
    match decode(&envelope_from_sdk(event)) {
        Ok(decoded) => Some(decoded),
        Err(error) => {
            tracing::warn!(
                event_type = %event.event_type,
                event_id = %event.id,
                entity_id = %event.entity_id,
                %error,
                "undecodable event skipped by folder"
            );
            None
        }
    }
}

/// Folds `content.post.*` into a [`PostView`].
#[derive(Debug, Default)]
pub struct PostFolder {
    /// `pub(crate)` so `workers::start_posts_worker` can seed it with
    /// already-known state — see `PostFolder::seed`.
    pub(crate) inner: Option<PostView>,
    deleted: bool,
}

impl EventFolder for PostFolder {
    type State = PostView;

    fn apply(&mut self, event: &Event) -> bool {
        let Some(decoded) = decode_or_warn(event) else {
            return false;
        };
        match decoded {
            DomainEvent::PostCreated {
                id,
                author_id,
                title,
                content,
                occurred_at,
            } => {
                // Idempotent: a replayed `created` re-establishes exactly the
                // same state rather than accumulating.
                self.inner = Some(PostView {
                    id,
                    author_id,
                    title,
                    content,
                    created_at: occurred_at,
                    updated_at: occurred_at,
                });
                true
            }
            DomainEvent::PostEdited {
                occurred_at,
                title,
                content,
                ..
            } => {
                if let Some(post) = self.inner.as_mut() {
                    if let Some(t) = title {
                        post.title = t;
                    }
                    if let Some(c) = content {
                        post.content = c;
                    }
                    // `max` rather than assignment: an out-of-order replay must
                    // not move `updated_at` backwards.
                    post.updated_at = post.updated_at.max(occurred_at);
                }
                true
            }
            // A latch, so re-applying the delete is a no-op.
            DomainEvent::PostDeleted { .. } => {
                self.deleted = true;
                true
            }
            _ => false,
        }
    }

    fn finalize(self) -> Option<PostView> {
        if self.deleted { None } else { self.inner }
    }
}

/// Folds `identity.user.*` into a [`UserView`].
///
/// Note this is the **domain** user stream (`user:<uuid>`), not better-auth's
/// private `auth-user:<id>` stream. §5.2: the two namespaces never merge and
/// our folders never read the auth one.
#[derive(Debug, Default)]
pub struct UserFolder {
    inner: Option<UserView>,
}

impl EventFolder for UserFolder {
    type State = UserView;

    fn apply(&mut self, event: &Event) -> bool {
        let Some(decoded) = decode_or_warn(event) else {
            return false;
        };
        match decoded {
            DomainEvent::UserRegistered {
                id,
                email,
                occurred_at,
                full_name,
                avatar_url,
            } => {
                self.inner = Some(UserView {
                    id,
                    email,
                    full_name,
                    avatar_url,
                    created_at: occurred_at,
                    updated_at: occurred_at,
                });
                true
            }
            DomainEvent::UserProfileUpdated {
                occurred_at,
                full_name,
                avatar_url,
                ..
            } => {
                if let Some(user) = self.inner.as_mut() {
                    // The doubly-nested Option is the patch semantics:
                    // `None` = key absent = unchanged, `Some(v)` = set to `v`
                    // (including `Some(None)` = cleared).
                    if let Some(name) = full_name {
                        user.full_name = name;
                    }
                    if let Some(url) = avatar_url {
                        user.avatar_url = url;
                    }
                    user.updated_at = user.updated_at.max(occurred_at);
                }
                true
            }
            _ => false,
        }
    }

    fn finalize(self) -> Option<UserView> {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use allsource::fold_events;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    /// Build the SDK event a real Core would hand back for this domain event.
    /// `timestamp` is deliberately a far-future ingest time, so any folder that
    /// wrongly reads domain time from the envelope fails loudly (D13).
    fn sdk_event(event: &DomainEvent, version: i64) -> Event {
        Event {
            id: format!("evt-{version}"),
            event_type: event.event_type().to_string(),
            entity_id: event.stream_id(),
            payload: serde_json::to_value(event).unwrap(),
            metadata: serde_json::Value::Null,
            timestamp: "2099-12-31T23:59:59Z".into(),
            version: Some(version),
            tenant_id: Some("default".into()),
        }
    }

    fn post_id() -> Uuid {
        Uuid::from_u128(2)
    }
    fn author_id() -> Uuid {
        Uuid::from_u128(1)
    }

    fn created() -> DomainEvent {
        DomainEvent::PostCreated {
            id: post_id(),
            author_id: author_id(),
            title: "original".into(),
            content: "body".into(),
            occurred_at: at("2024-03-01T12:00:00Z"),
        }
    }

    #[test]
    fn folds_create_then_edit() {
        let events = vec![
            sdk_event(&created(), 1),
            sdk_event(
                &DomainEvent::PostEdited {
                    id: post_id(),
                    occurred_at: at("2024-04-01T12:00:00Z"),
                    title: Some("edited".into()),
                    content: None,
                },
                2,
            ),
        ];
        let post = fold_events::<PostFolder>(&events).expect("post exists");
        assert_eq!(post.title, "edited");
        assert_eq!(post.content, "body", "an absent patch field is unchanged");
        assert_eq!(post.created_at, at("2024-03-01T12:00:00Z"));
        assert_eq!(post.updated_at, at("2024-04-01T12:00:00Z"));
    }

    /// D13, stated as a test: the envelope timestamp is 2099 and must not leak
    /// into the read model.
    #[test]
    fn domain_time_comes_from_the_payload_not_the_envelope() {
        let post = fold_events::<PostFolder>(&[sdk_event(&created(), 1)]).unwrap();
        assert_eq!(post.created_at.to_rfc3339(), "2024-03-01T12:00:00+00:00");
    }

    /// §4.4: a rebuild replays everything, so re-applying an event must be a
    /// no-op. getformlab tests exactly this shape; we copy the habit.
    #[test]
    fn folding_is_idempotent_under_replay() {
        let once = vec![sdk_event(&created(), 1)];
        let twice = vec![sdk_event(&created(), 1), sdk_event(&created(), 1)];
        assert_eq!(
            fold_events::<PostFolder>(&once).unwrap(),
            fold_events::<PostFolder>(&twice).unwrap()
        );
    }

    #[test]
    fn a_deleted_post_finalizes_to_none() {
        let events = vec![
            sdk_event(&created(), 1),
            sdk_event(
                &DomainEvent::PostDeleted {
                    id: post_id(),
                    occurred_at: at("2024-05-01T12:00:00Z"),
                },
                2,
            ),
            // Replayed delete: still just gone, not an error.
            sdk_event(
                &DomainEvent::PostDeleted {
                    id: post_id(),
                    occurred_at: at("2024-05-01T12:00:00Z"),
                },
                2,
            ),
        ];
        assert!(fold_events::<PostFolder>(&events).is_none());
    }

    #[test]
    fn an_empty_stream_means_the_entity_does_not_exist() {
        assert!(fold_events::<PostFolder>(&[]).is_none());
        assert!(fold_events::<UserFolder>(&[]).is_none());
    }

    /// An edit with no preceding create must not fabricate a post out of thin
    /// air — the stream genuinely has no `created`, so the entity is absent.
    #[test]
    fn an_orphan_edit_does_not_conjure_an_entity() {
        let events = vec![sdk_event(
            &DomainEvent::PostEdited {
                id: post_id(),
                occurred_at: at("2024-04-01T12:00:00Z"),
                title: Some("ghost".into()),
                content: None,
            },
            1,
        )];
        assert!(fold_events::<PostFolder>(&events).is_none());
    }

    /// A folder must be *total*: garbage in the stream is skipped, not a panic.
    /// A rebuild replays the whole store, so one bad event would otherwise take
    /// down every read model.
    #[test]
    fn an_undecodable_event_is_skipped_not_panicked_on() {
        let mut bad = sdk_event(&created(), 1);
        bad.payload = serde_json::json!({"type": "PostCreated", "id": "not-a-uuid"});
        let events = vec![bad, sdk_event(&created(), 2)];
        let post = fold_events::<PostFolder>(&events).expect("the good event still folds");
        assert_eq!(post.title, "original");
    }

    /// An event of an unrelated family must be reported as *not applied*, which
    /// is what lets the worker count progress honestly (the reason D14 prefers
    /// the SDK's `-> bool` over getformlab's `-> ()`).
    #[test]
    fn unrelated_events_report_as_not_applied() {
        let user_event = DomainEvent::UserRegistered {
            id: author_id(),
            email: "a@b.c".into(),
            occurred_at: at("2024-01-01T00:00:00Z"),
            full_name: None,
            avatar_url: None,
        };
        let mut folder = PostFolder::default();
        assert!(!folder.apply(&sdk_event(&user_event, 1)));
        assert!(folder.apply(&sdk_event(&created(), 2)));
    }

    #[test]
    fn profile_patches_distinguish_clear_from_unchanged() {
        let user = author_id();
        let events = vec![
            sdk_event(
                &DomainEvent::UserRegistered {
                    id: user,
                    email: "ada@example.com".into(),
                    occurred_at: at("2024-01-01T00:00:00Z"),
                    full_name: Some("Ada Lovelace".into()),
                    avatar_url: Some("https://example.com/a.png".into()),
                },
                1,
            ),
            sdk_event(
                &DomainEvent::UserProfileUpdated {
                    id: user,
                    occurred_at: at("2024-02-01T00:00:00Z"),
                    full_name: None,        // absent → unchanged
                    avatar_url: Some(None), // explicit null → cleared
                },
                2,
            ),
        ];
        let view = fold_events::<UserFolder>(&events).unwrap();
        assert_eq!(view.full_name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(view.avatar_url, None);
        assert_eq!(view.updated_at, at("2024-02-01T00:00:00Z"));
    }

    /// Out-of-order replay must not move `updated_at` backwards.
    #[test]
    fn out_of_order_edits_do_not_rewind_updated_at() {
        let events = vec![
            sdk_event(&created(), 1),
            sdk_event(
                &DomainEvent::PostEdited {
                    id: post_id(),
                    occurred_at: at("2024-06-01T00:00:00Z"),
                    title: Some("late".into()),
                    content: None,
                },
                2,
            ),
            sdk_event(
                &DomainEvent::PostEdited {
                    id: post_id(),
                    occurred_at: at("2024-05-01T00:00:00Z"),
                    title: Some("early".into()),
                    content: None,
                },
                3,
            ),
        ];
        let post = fold_events::<PostFolder>(&events).unwrap();
        assert_eq!(post.updated_at, at("2024-06-01T00:00:00Z"));
    }
}
