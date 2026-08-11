//! Continuous folding via [`allsource::ProjectionWorker`] (§4.2).
//!
//! ## When to use this instead of fold-on-read
//!
//! | | Fold-on-read | Continuous worker |
//! |---|---|---|
//! | Query scoped to **one** `entity_id` | ✅ `QueryClient::query_and_fold` | overkill |
//! | Read model spanning **many** entities | ❌ scans the whole store per request | ✅ |
//!
//! So `GET /posts/{id}` and `GET /me` fold on read, and `GET /posts` — the list
//! — is served from the worker below.
//!
//! ## Rebuilding (§4.4)
//!
//! Core keys the consumer cursor by the worker's **name**, so a new name has no
//! cursor and replays from zero. The rename *is* the rebuild trigger. Bump
//! [`POSTS_WORKER_NAME`] to `posts_v2`, run both workers in parallel until
//! `is_caught_up()`, flip the handler, then delete the old state. Do not try to
//! do cursor surgery.

use std::collections::HashMap;

use allsource::{CoreClient, Error, Event, EventFolder, ProjectionHandle, ProjectionWorker};
use rv2_api_types::PostView;
use rv2_events::DomainEvent;
use uuid::Uuid;

use crate::folders::PostFolder;

/// The durable consumer id for the posts read model.
///
/// **Changing this string triggers a full replay.** That is the intended
/// rebuild mechanism — see the module docs — not an accident to avoid.
pub const POSTS_WORKER_NAME: &str = "posts_v1";

/// The cross-entity posts read model: every live post, keyed by id.
///
/// `Serialize + Deserialize` are required by the SDK's `WorkerState` bound (it
/// can push state back to Core). `Uuid` keys serialize as strings, which is
/// what `serde_json` requires of a map key.
pub type PostsReadModel = HashMap<Uuid, PostView>;

/// Apply one event to the whole-collection read model.
///
/// Exposed (rather than inlined into the builder closure) so it can be
/// unit-tested without a running Core — the reducer is the part with the
/// interesting behaviour, and it should not need a network to exercise.
///
/// It delegates per-entity logic to [`PostFolder`] so there is exactly one
/// definition of "what a post is", shared by the fold-on-read and continuous
/// paths. A divergence between those two is the classic event-sourcing bug.
///
/// # Errors
///
/// Never. The signature returns `Result` because the SDK's `Reducer` type
/// demands it, and an `Err` **aborts the worker** — so a malformed event must
/// be skipped (with a warning, inside `PostFolder`) rather than propagated.
pub fn apply_post_event(state: &mut PostsReadModel, event: &Event) -> Result<(), Error> {
    let mut folder = PostFolder::default();

    // Seed the folder with what we already know about this entity, so an
    // `edited` event applies on top of the existing view rather than being
    // dropped as an orphan.
    let existing = crate::envelope_from_sdk(event)
        .entity_id
        .split_once(':')
        .and_then(|(_, id)| id.parse::<Uuid>().ok())
        .and_then(|id| state.get(&id).cloned());

    match existing {
        Some(view) => {
            let id = view.id;
            folder.seed(view);
            if folder.apply(event) {
                match folder.finalize() {
                    Some(updated) => {
                        state.insert(id, updated);
                    }
                    // `finalize` returning `None` after a seeded fold means the
                    // post was deleted. Removing is idempotent.
                    None => {
                        state.remove(&id);
                    }
                }
            }
        }
        None => {
            if folder.apply(event)
                && let Some(created) = folder.finalize()
            {
                state.insert(created.id, created);
            }
        }
    }
    Ok(())
}

/// Start the `posts_v1` worker and return its handle.
///
/// `apps/api` holds the handle in `AppState`.
///
/// ## The restart bug this function exists to avoid
///
/// **Observed against a live Core, not inferred.** `ProjectionWorker::start`
/// registers the durable consumer and then streams **from the server-side
/// cursor** into a freshly `Default`-constructed in-process state. It does
/// **not** hydrate. So the second time a process boots, the worker replays only
/// events since its last ack — and everything folded before that ack is simply
/// gone from memory. `GET /posts` then returns a partial list, with no error
/// anywhere.
///
/// §4.3 describes the checkpoint as what makes "a worker restart
/// O(events-since-ack) instead of O(total)". That is true of the *replay* cost,
/// but it is only *correct* if the state itself is durable. It is not, in
/// process memory.
///
/// The fix is the mechanism Core already provides, wired up in both directions:
///
/// 1. **Flush** — `state_flush_entities` pushes each entity's state into Core's
///    projection KV as the worker folds.
/// 2. **Hydrate** — at boot, `get_projection_state_summary` reads that KV back
///    before the stream is attached.
///
/// This is safe because the flushed state always corresponds to a position at
/// or before the cursor, the worker replays forward from the cursor, and
/// [`apply_post_event`] is idempotent. Overlap is harmless; the gap is closed.
///
/// A hydration failure is logged and swallowed rather than propagated: a
/// worker with cold state is strictly better than no worker at all, and
/// `GET /posts` has a fold-on-read fallback underneath it either way.
///
/// # Errors
///
/// [`Error`] if the worker cannot be built or its consumer cannot be
/// registered.
pub async fn start_posts_worker(
    core: CoreClient,
) -> Result<ProjectionHandle<PostsReadModel>, Error> {
    let seed = match core
        .get_projection_state_summary::<PostView>(POSTS_WORKER_NAME)
        .await
    {
        Ok(states) => {
            tracing::info!(
                worker = POSTS_WORKER_NAME,
                entities = states.len(),
                "hydrated projection state from Core"
            );
            states
        }
        Err(error) => {
            tracing::warn!(
                worker = POSTS_WORKER_NAME,
                %error,
                "could not hydrate projection state; starting cold"
            );
            Vec::new()
        }
    };

    let handle = ProjectionWorker::<PostsReadModel>::builder(core)
        .name(POSTS_WORKER_NAME)
        .event_types(DomainEvent::POST_WIRE_TYPES)
        .reducer(apply_post_event)
        .checkpoint_interval(100)
        .state_flush_entities(flush_entities)
        .state_flush_every(25)
        .build()?
        .start()
        .await?;

    if !seed.is_empty() {
        let shared = handle.state();
        let mut state = shared.write().await;
        for (_entity_id, view) in seed {
            // `or_insert`: anything the worker has already folded since the
            // cursor is newer than the flushed snapshot and must win.
            state.entry(view.id).or_insert(view);
        }
    }

    Ok(handle)
}

/// Map the read model to `(entity_id, state)` pairs for push-back to Core.
///
/// Runs under a read lock on state, so it stays a plain map with no awaits.
/// The entity id is the real stream id (`post:<uuid>`), not the bare uuid, so
/// the flushed KV lines up with the events it was folded from.
fn flush_entities(state: &PostsReadModel) -> Vec<(String, serde_json::Value)> {
    state
        .values()
        .filter_map(|view| {
            serde_json::to_value(view).ok().map(|json| {
                (
                    rv2_events::stream(rv2_events::StreamKind::Post, view.id),
                    json,
                )
            })
        })
        .collect()
}

impl PostFolder {
    /// Prime the folder with already-known state.
    ///
    /// Only the continuous path needs this: fold-on-read always starts from an
    /// empty folder and replays the entity's whole (short) stream.
    pub(crate) fn seed(&mut self, view: PostView) {
        self.inner = Some(view);
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    fn sdk_event(event: &DomainEvent, version: i64) -> Event {
        Event {
            id: format!("evt-{version}"),
            event_type: event.event_type().to_string(),
            entity_id: event.stream_id(),
            payload: serde_json::to_value(event).unwrap(),
            metadata: serde_json::Value::Null,
            timestamp: "2099-12-31T23:59:59Z".into(),
            version: Some(version),
            tenant_id: None,
        }
    }

    fn created(id: Uuid, title: &str) -> DomainEvent {
        DomainEvent::PostCreated {
            id,
            author_id: Uuid::from_u128(1),
            title: title.into(),
            content: "body".into(),
            occurred_at: at("2024-03-01T12:00:00Z"),
        }
    }

    #[test]
    fn the_reducer_accumulates_across_entities() {
        let (a, b) = (Uuid::from_u128(10), Uuid::from_u128(11));
        let mut state = PostsReadModel::new();
        apply_post_event(&mut state, &sdk_event(&created(a, "first"), 1)).unwrap();
        apply_post_event(&mut state, &sdk_event(&created(b, "second"), 1)).unwrap();
        assert_eq!(state.len(), 2);
        assert_eq!(state[&a].title, "first");
        assert_eq!(state[&b].title, "second");
    }

    /// The bug this seeding exists to prevent: an `edited` arriving after the
    /// `created` has already been folded must patch the stored view, not be
    /// dropped as an orphan.
    #[test]
    fn an_edit_patches_the_already_stored_view() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::new();
        apply_post_event(&mut state, &sdk_event(&created(id, "first"), 1)).unwrap();
        apply_post_event(
            &mut state,
            &sdk_event(
                &DomainEvent::PostEdited {
                    id,
                    occurred_at: at("2024-04-01T12:00:00Z"),
                    title: Some("patched".into()),
                    content: None,
                },
                2,
            ),
        )
        .unwrap();
        assert_eq!(state[&id].title, "patched");
        assert_eq!(state[&id].content, "body", "untouched field survives");
    }

    #[test]
    fn a_delete_removes_the_entry_idempotently() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::new();
        apply_post_event(&mut state, &sdk_event(&created(id, "first"), 1)).unwrap();
        let deleted = DomainEvent::PostDeleted {
            id,
            occurred_at: at("2024-05-01T12:00:00Z"),
        };
        apply_post_event(&mut state, &sdk_event(&deleted, 2)).unwrap();
        assert!(state.is_empty());
        apply_post_event(&mut state, &sdk_event(&deleted, 3)).unwrap();
        assert!(state.is_empty(), "replayed delete is a no-op");
    }

    /// An `Err` from the reducer aborts the worker, so a garbage event must
    /// never produce one.
    #[test]
    fn a_malformed_event_never_aborts_the_worker() {
        let mut state = PostsReadModel::new();
        let mut bad = sdk_event(&created(Uuid::from_u128(10), "x"), 1);
        bad.payload = serde_json::json!({"nonsense": true});
        assert!(apply_post_event(&mut state, &bad).is_ok());
        assert!(state.is_empty());
    }

    /// The continuous and fold-on-read paths must agree. If they ever diverge,
    /// `GET /posts` and `GET /posts/{id}` start disagreeing about the same
    /// post, which is the classic event-sourcing bug.
    #[test]
    fn the_worker_and_fold_on_read_agree() {
        let id = Uuid::from_u128(10);
        let events = vec![
            sdk_event(&created(id, "first"), 1),
            sdk_event(
                &DomainEvent::PostEdited {
                    id,
                    occurred_at: at("2024-04-01T12:00:00Z"),
                    title: Some("patched".into()),
                    content: Some("new body".into()),
                },
                2,
            ),
        ];

        let mut state = PostsReadModel::new();
        for event in &events {
            apply_post_event(&mut state, event).unwrap();
        }
        let via_worker = state.remove(&id).unwrap();
        let via_fold = allsource::fold_events::<PostFolder>(&events).unwrap();
        assert_eq!(via_worker, via_fold);
    }

    /// The flusher must key by the real stream id, so the pushed KV lines up
    /// with the events it was folded from — and so hydration is symmetric.
    #[test]
    fn flushed_entities_are_keyed_by_stream_id() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::new();
        apply_post_event(&mut state, &sdk_event(&created(id, "first"), 1)).unwrap();

        let flushed = flush_entities(&state);
        assert_eq!(flushed.len(), 1);
        assert_eq!(flushed[0].0, format!("post:{id}"));
        assert_eq!(flushed[0].1["title"], "first");
    }

    /// Flush and hydrate must round-trip, or a restart silently loses posts.
    #[test]
    fn flushed_state_deserializes_back_into_the_read_model() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::new();
        apply_post_event(&mut state, &sdk_event(&created(id, "first"), 1)).unwrap();

        let (_entity, json) = flush_entities(&state).remove(0);
        let view: PostView = serde_json::from_value(json).expect("round-trips");
        assert_eq!(view, state[&id]);
    }

    #[test]
    fn the_worker_subscribes_to_exactly_the_post_family() {
        assert_eq!(
            DomainEvent::POST_WIRE_TYPES,
            &[
                "content.post.created",
                "content.post.edited",
                "content.post.deleted"
            ]
        );
    }
}
