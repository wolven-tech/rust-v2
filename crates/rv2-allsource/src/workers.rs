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

use std::collections::{HashMap, HashSet};

use allsource::{CoreClient, Error, Event, EventFolder, ProjectionHandle, ProjectionWorker};
use rv2_api_types::PostView;
use rv2_events::DomainEvent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::folders::PostFolder;

/// The durable consumer id for the posts read model.
///
/// **Changing this string triggers a full replay.** That is the intended
/// rebuild mechanism — see the module docs — not an accident to avoid.
///
/// `posts_v2`: `posts_v1`'s KV holds live snapshots of deleted and edited posts
/// with its cursor past those events, which hydration would serve forever.
pub const POSTS_WORKER_NAME: &str = "posts_v2";

/// The cross-entity posts read model: every live post, keyed by id, plus the
/// ids of deleted posts.
///
/// The deleted ids are flushed to Core's projection KV as tombstones. The SDK
/// can only upsert KV entries, never remove one, so without a tombstone a
/// deleted post's last live snapshot stays in the KV and is hydrated back into
/// the list on the next boot, with the cursor already past its delete.
///
/// `Serialize + Deserialize` are required by the SDK's `WorkerState` bound.
/// `Uuid` keys serialize as strings, which is what `serde_json` requires of a
/// map key.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PostsReadModel {
    live: HashMap<Uuid, PostView>,
    deleted: HashSet<Uuid>,
}

impl PostsReadModel {
    /// Every live post, in no particular order.
    pub fn posts(&self) -> impl Iterator<Item = &PostView> {
        self.live.values()
    }

    /// One live post.
    #[must_use]
    pub fn get(&self, id: &Uuid) -> Option<&PostView> {
        self.live.get(id)
    }

    /// How many live posts there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.live.len()
    }

    /// Whether there are no live posts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.live.is_empty()
    }

    fn delete(&mut self, id: Uuid) {
        self.live.remove(&id);
        self.deleted.insert(id);
    }

    /// Merge one flushed KV entry read back at boot, before the replay starts.
    ///
    /// A tombstone always wins over a live snapshot of the same post, in
    /// either order: a deleted post cannot come back.
    fn hydrate(&mut self, flushed: FlushedPost) {
        match flushed {
            FlushedPost::Live(view) => {
                if !self.deleted.contains(&view.id) {
                    self.live.entry(view.id).or_insert(view);
                }
            }
            FlushedPost::Deleted(tombstone) => {
                if tombstone.deleted {
                    self.delete(tombstone.id);
                }
            }
        }
    }
}

/// One entry of the posts worker's projection KV.
///
/// Untagged so entries flushed before tombstones existed, which are bare
/// [`PostView`]s, still deserialize as [`FlushedPost::Live`].
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum FlushedPost {
    Live(PostView),
    Deleted(Tombstone),
}

#[derive(Debug, Serialize, Deserialize)]
struct Tombstone {
    id: Uuid,
    deleted: bool,
}

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
    let entity = crate::envelope_from_sdk(event)
        .entity_id
        .split_once(':')
        .and_then(|(_, id)| id.parse::<Uuid>().ok());
    let existing = entity.and_then(|id| state.live.get(&id).cloned());

    match existing {
        Some(view) => {
            let id = view.id;
            folder.seed(view);
            if folder.apply(event) {
                match folder.finalize() {
                    Some(updated) => {
                        state.live.insert(id, updated);
                    }
                    // `finalize` returning `None` after a seeded fold means the
                    // post was deleted. Deleting is idempotent.
                    None => state.delete(id),
                }
            }
        }
        None => {
            if folder.apply(event) {
                match folder.finalize() {
                    Some(created) if !state.deleted.contains(&created.id) => {
                        state.live.insert(created.id, created);
                    }
                    Some(_) => {}
                    // A delete for a post this state never held still has to be
                    // remembered, or a snapshot of it could be merged in later.
                    None => {
                        if let Some(id) = entity
                            && is_delete(event)
                        {
                            state.delete(id);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn is_delete(event: &Event) -> bool {
    event.event_type == "content.post.deleted"
}

/// Start the posts worker and return its handle.
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
///    into the worker's state **before `start()`**, so the replay from the
///    cursor applies on top of it.
///
/// The order matters. Hydrating after `start()` raced the replay: an edit or a
/// delete replayed into empty state found no post, and the older snapshot
/// merged in afterwards won. Observed against a live Core: after a restart,
/// deleted posts were listed again with their pre-edit titles.
///
/// The SDK checkpoints only during replay, so the cursor is never past events
/// the KV has not seen; replaying them over a snapshot that already has them
/// converges because they replay in order.
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
        .get_projection_state_summary::<FlushedPost>(POSTS_WORKER_NAME)
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

    let worker = ProjectionWorker::<PostsReadModel>::builder(core)
        .name(POSTS_WORKER_NAME)
        .event_types(DomainEvent::POST_WIRE_TYPES)
        .reducer(apply_post_event)
        .checkpoint_interval(100)
        .state_flush_entities(flush_entities)
        .state_flush_every(25)
        .build()?;

    {
        let shared = worker.state();
        let mut state = shared.write().await;
        for (_entity_id, flushed) in seed {
            state.hydrate(flushed);
        }
    }

    worker.start().await
}

/// Map the read model to `(entity_id, state)` pairs for push-back to Core.
///
/// Runs under a read lock on state, so it stays a plain map with no awaits.
/// The entity id is the real stream id (`post:<uuid>`), not the bare uuid, so
/// the flushed KV lines up with the events it was folded from.
fn flush_entities(state: &PostsReadModel) -> Vec<(String, serde_json::Value)> {
    let live = state
        .live
        .values()
        .map(|view| (view.id, serde_json::to_value(view)));
    let deleted = state
        .deleted
        .iter()
        .map(|&id| (id, serde_json::to_value(Tombstone { id, deleted: true })));
    live.chain(deleted)
        .filter_map(|(id, json)| {
            json.ok()
                .map(|json| (rv2_events::stream(rv2_events::StreamKind::Post, id), json))
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
        let mut state = PostsReadModel::default();
        apply_post_event(&mut state, &sdk_event(&created(a, "first"), 1)).unwrap();
        apply_post_event(&mut state, &sdk_event(&created(b, "second"), 1)).unwrap();
        assert_eq!(state.len(), 2);
        assert_eq!(state.get(&a).unwrap().title, "first");
        assert_eq!(state.get(&b).unwrap().title, "second");
    }

    /// The bug this seeding exists to prevent: an `edited` arriving after the
    /// `created` has already been folded must patch the stored view, not be
    /// dropped as an orphan.
    #[test]
    fn an_edit_patches_the_already_stored_view() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::default();
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
        assert_eq!(state.get(&id).unwrap().title, "patched");
        assert_eq!(
            state.get(&id).unwrap().content,
            "body",
            "untouched field survives"
        );
    }

    #[test]
    fn a_delete_removes_the_entry_idempotently() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::default();
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
        let mut state = PostsReadModel::default();
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

        let mut state = PostsReadModel::default();
        for event in &events {
            apply_post_event(&mut state, event).unwrap();
        }
        let via_worker = state.get(&id).unwrap().clone();
        let via_fold = allsource::fold_events::<PostFolder>(&events).unwrap();
        assert_eq!(via_worker, via_fold);
    }

    /// The flusher must key by the real stream id, so the pushed KV lines up
    /// with the events it was folded from — and so hydration is symmetric.
    #[test]
    fn flushed_entities_are_keyed_by_stream_id() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::default();
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
        let mut state = PostsReadModel::default();
        apply_post_event(&mut state, &sdk_event(&created(id, "first"), 1)).unwrap();

        let (_entity, json) = flush_entities(&state).remove(0);
        let view: PostView = serde_json::from_value(json).expect("round-trips");
        assert_eq!(&view, state.get(&id).unwrap());
    }

    fn hydrate_from(flushed: Vec<(String, serde_json::Value)>) -> PostsReadModel {
        let mut booted = PostsReadModel::default();
        for (_entity, json) in flushed {
            booted.hydrate(serde_json::from_value(json).expect("flushed entry deserializes"));
        }
        booted
    }

    /// The KV only upserts. A delete must overwrite the post's last live
    /// snapshot with a tombstone, or the next boot hydrates the post back.
    #[test]
    fn a_deleted_post_stays_deleted_across_a_restart() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::default();
        apply_post_event(&mut state, &sdk_event(&created(id, "first"), 1)).unwrap();
        let stale = flush_entities(&state);

        apply_post_event(
            &mut state,
            &sdk_event(
                &DomainEvent::PostDeleted {
                    id,
                    occurred_at: at("2024-05-01T12:00:00Z"),
                },
                2,
            ),
        )
        .unwrap();
        let flushed = flush_entities(&state);
        assert_eq!(flushed.len(), 1, "{flushed:?}");
        assert_eq!(
            flushed[0].0,
            format!("post:{id}"),
            "same KV key as the snapshot"
        );

        assert!(hydrate_from(flushed.clone()).is_empty());
        let stale_first: Vec<_> = stale.iter().chain(&flushed).cloned().collect();
        let tombstone_first: Vec<_> = flushed.iter().chain(&stale).cloned().collect();
        assert!(hydrate_from(stale_first).is_empty());
        assert!(
            hydrate_from(tombstone_first).is_empty(),
            "a tombstone wins over a live snapshot whatever the order"
        );
    }

    /// The live-Core restart case: the KV holds a pre-edit snapshot, and the
    /// edit and the delete come back in the replay.
    #[test]
    fn a_replay_over_a_stale_snapshot_converges_to_deleted() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::default();
        apply_post_event(&mut state, &sdk_event(&created(id, "first"), 1)).unwrap();
        let stale = flush_entities(&state);

        let mut booted = hydrate_from(stale.clone());
        let edit = DomainEvent::PostEdited {
            id,
            occurred_at: at("2024-04-01T12:00:00Z"),
            title: Some("patched".into()),
            content: None,
        };
        let delete = DomainEvent::PostDeleted {
            id,
            occurred_at: at("2024-05-01T12:00:00Z"),
        };
        apply_post_event(&mut booted, &sdk_event(&edit, 2)).unwrap();
        apply_post_event(&mut booted, &sdk_event(&delete, 3)).unwrap();
        assert!(booted.is_empty());

        // A delete for a post the state never held is still remembered.
        let mut cold = PostsReadModel::default();
        apply_post_event(&mut cold, &sdk_event(&delete, 3)).unwrap();
        for (_entity, json) in stale {
            cold.hydrate(serde_json::from_value(json).unwrap());
        }
        assert!(
            cold.is_empty(),
            "a later snapshot resurrected a deleted post"
        );
    }

    #[test]
    fn a_snapshot_flushed_before_tombstones_existed_still_hydrates() {
        let id = Uuid::from_u128(10);
        let mut state = PostsReadModel::default();
        apply_post_event(&mut state, &sdk_event(&created(id, "first"), 1)).unwrap();
        let bare = serde_json::to_value(state.get(&id).unwrap()).unwrap();

        let booted = hydrate_from(vec![(format!("post:{id}"), bare)]);
        assert_eq!(booted.get(&id).unwrap().title, "first");
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
