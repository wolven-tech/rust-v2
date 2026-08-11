//! Post invariants and the command → event functions.
//!
//! The command functions are the *only* way a `content.post.*` event is
//! constructed. They take `now` and a fresh `id` as arguments rather than
//! reading a clock or minting a UUID, which is what keeps this crate pure and
//! WASM-safe (§2.2 trap 1: no `getrandom` in the WASM half).

use chrono::{DateTime, Utc};
use rv2_api_types::{CreatePostRequest, PostView, UpdatePostRequest};
use rv2_events::DomainEvent;
use uuid::Uuid;

use crate::{Validated, ValidationError, non_empty_bounded};

pub const TITLE_MAX: usize = 200;
pub const CONTENT_MAX: usize = 20_000;

/// Validate a create-post command and produce the event to append.
///
/// # Errors
///
/// [`ValidationError`] if the title or content is empty or over the limit.
pub fn create(
    id: Uuid,
    author_id: Uuid,
    now: DateTime<Utc>,
    req: &CreatePostRequest,
) -> Validated<DomainEvent> {
    Ok(DomainEvent::PostCreated {
        id,
        author_id,
        title: non_empty_bounded("title", &req.title, TITLE_MAX)?,
        content: non_empty_bounded("content", &req.content, CONTENT_MAX)?,
        occurred_at: now,
    })
}

/// Validate an edit-post command and produce the event to append.
///
/// A patch that changes nothing is rejected rather than silently appending a
/// no-op event — an empty edit in the stream is noise a future fold has to
/// reason about.
///
/// # Errors
///
/// [`ValidationError`] if a supplied field is invalid, or if the patch is
/// entirely empty.
pub fn edit(id: Uuid, now: DateTime<Utc>, req: &UpdatePostRequest) -> Validated<DomainEvent> {
    if req.title.is_none() && req.content.is_none() {
        return Err(ValidationError::new(
            "body",
            "must change at least one field",
        ));
    }
    let title = req
        .title
        .as_deref()
        .map(|t| non_empty_bounded("title", t, TITLE_MAX))
        .transpose()?;
    let content = req
        .content
        .as_deref()
        .map(|c| non_empty_bounded("content", c, CONTENT_MAX))
        .transpose()?;
    Ok(DomainEvent::PostEdited {
        id,
        occurred_at: now,
        title,
        content,
    })
}

/// Produce the delete event. Always valid — deleting is idempotent by
/// construction, because the folder treats "deleted" as a latch.
#[must_use]
pub fn delete(id: Uuid, now: DateTime<Utc>) -> DomainEvent {
    DomainEvent::PostDeleted {
        id,
        occurred_at: now,
    }
}

/// R4: authorization as a named, testable predicate rather than an inline `if`
/// somebody can forget to write.
#[must_use]
pub const fn can_edit_post(actor: Uuid, post: &PostView) -> bool {
    // `Uuid`'s `PartialEq` is not const, so compare the raw bytes.
    u128::from_be_bytes(*actor.as_bytes()) == u128::from_be_bytes(*post.author_id.as_bytes())
}

/// Same rule as editing today, but named separately so the two can diverge
/// (e.g. moderators gaining delete but not edit) without a shotgun change.
#[must_use]
pub const fn can_delete_post(actor: Uuid, post: &PostView) -> bool {
    can_edit_post(actor, post)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        "2026-01-15T10:30:00Z".parse().unwrap()
    }

    fn view(author: Uuid) -> PostView {
        PostView {
            id: Uuid::from_u128(9),
            author_id: author,
            title: "t".into(),
            content: "c".into(),
            created_at: now(),
            updated_at: now(),
        }
    }

    #[test]
    fn create_trims_and_emits_the_right_variant() {
        let event = create(
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            now(),
            &CreatePostRequest {
                title: "  Hello  ".into(),
                content: "  World  ".into(),
            },
        )
        .unwrap();
        assert_eq!(event.event_type(), "content.post.created");
        match event {
            DomainEvent::PostCreated { title, content, .. } => {
                assert_eq!(title, "Hello");
                assert_eq!(content, "World");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn create_rejects_empty_and_oversized_fields() {
        let bad_title = CreatePostRequest {
            title: "   ".into(),
            content: "ok".into(),
        };
        assert_eq!(
            create(Uuid::nil(), Uuid::nil(), now(), &bad_title)
                .unwrap_err()
                .field,
            "title"
        );

        let bad_content = CreatePostRequest {
            title: "ok".into(),
            content: "x".repeat(CONTENT_MAX + 1),
        };
        assert_eq!(
            create(Uuid::nil(), Uuid::nil(), now(), &bad_content)
                .unwrap_err()
                .field,
            "content"
        );
    }

    #[test]
    fn an_empty_patch_is_rejected_rather_than_appended() {
        let err = edit(Uuid::nil(), now(), &UpdatePostRequest::default()).unwrap_err();
        assert_eq!(err.field, "body");
    }

    #[test]
    fn edit_carries_only_the_changed_fields() {
        let event = edit(
            Uuid::nil(),
            now(),
            &UpdatePostRequest {
                title: Some("new title".into()),
                content: None,
            },
        )
        .unwrap();
        match event {
            DomainEvent::PostEdited { title, content, .. } => {
                assert_eq!(title.as_deref(), Some("new title"));
                assert_eq!(content, None);
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    /// R4 is "the risk most likely to produce a real incident" — so the
    /// negative case is tested, not just the positive one.
    #[test]
    fn only_the_author_may_edit_or_delete() {
        let author = Uuid::from_u128(1);
        let stranger = Uuid::from_u128(2);
        let post = view(author);
        assert!(can_edit_post(author, &post));
        assert!(can_delete_post(author, &post));
        assert!(!can_edit_post(stranger, &post));
        assert!(!can_delete_post(stranger, &post));
    }
}
