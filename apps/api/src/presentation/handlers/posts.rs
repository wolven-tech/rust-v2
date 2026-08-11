//! Post endpoints — the write path and both read paths.
//!
//! ## Which read path, and why (§4.2)
//!
//! - `GET /posts/{id}` → **fold-on-read**. `entity_id` is known and the stream
//!   is a handful of events, so `QueryClient::query_and_fold` is one HTTP call
//!   and is perfectly fresh.
//! - `GET /posts` → **the `posts_v1` worker**. Folding on read would mean
//!   scanning every `content.post.*` event in the store on every page load.
//!
//! ## Authorization (R4)
//!
//! Every mutating handler re-reads the post and calls the named predicate in
//! `rv2_domain::post`. Reading current state before authorizing is the whole
//! point: the actor's claim about ownership is never trusted.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use rv2_allsource::QueryEventsParams;
use rv2_api_types::{CreatePostRequest, PostView, UpdatePostRequest};
use rv2_domain::post;
use rv2_events::{DomainEvent, StreamKind, stream};
use uuid::Uuid;

use crate::infrastructure::{auth::ExtractAuthUser, error::ApiError, state::AppState};

/// Fold one post's stream on read.
async fn load(state: &AppState, id: Uuid) -> Result<PostView, ApiError> {
    // `rv2_allsource::tenant_query` rather than `QueryClient::query_and_fold`:
    // Core scopes reads by tenant and returns an EMPTY result — HTTP 200, no
    // error — when `tenant_id` is absent, and the SDK's `QueryEventsParams` has
    // no field for it. Against Core the SDK path therefore reads nothing
    // forever, which made this handler 404 immediately after a successful
    // append. See that module's docs.
    state
        .allsource
        .fold_entity::<rv2_allsource::PostFolder>(&stream(StreamKind::Post, id))
        .await
        .map_err(|_| ApiError::NotFound { kind: "post" })?
        .ok_or(ApiError::NotFound { kind: "post" })
}

/// `POST /posts` — the write half of the vertical slice.
///
/// The id is minted **here**, server-side. That is what lets the WASM crates
/// stay free of `getrandom` (§2.2 trap 1): ids travel to the browser in DTOs
/// and are never generated there.
///
/// # Errors
///
/// 422 on validation failure, 502 if AllSource rejects the append.
pub async fn create(
    State(state): State<Arc<AppState>>,
    ExtractAuthUser(actor): ExtractAuthUser,
    Json(request): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostView>), ApiError> {
    let id = Uuid::new_v4();
    let event = post::create(id, actor.id, Utc::now(), &request)?;
    state.writer.append(&event).await?;

    // Read back through the same fold the query path uses, rather than
    // synthesising a response from the request. If the append and the fold ever
    // disagree, the caller sees it immediately instead of on the next page load.
    let view = load(&state, id).await?;
    Ok((StatusCode::CREATED, Json(view)))
}

/// `GET /posts` — from the `posts_v1` worker, falling back to a store scan.
///
/// The fallback exists because the worker is started best-effort (see
/// `build_state`). It is honestly slower — it scans every `content.post.*`
/// event — but a degraded list beats a 500.
///
/// # Errors
///
/// 502 if AllSource is unreachable on the fallback path.
pub async fn list(
    State(state): State<Arc<AppState>>,
    ExtractAuthUser(_actor): ExtractAuthUser,
) -> Result<Json<Vec<PostView>>, ApiError> {
    let mut posts: Vec<PostView> = if let Some(handle) = state.posts_handle() {
        // Bind the `Arc<RwLock<_>>` before locking: `handle.state()` returns by
        // value, so locking it inline would drop the temporary while borrowed.
        let shared = handle.state();
        let guard = shared.read().await;
        guard.values().cloned().collect()
    } else {
        tracing::debug!("posts worker unavailable; falling back to a full fold-on-read");
        fold_all_posts(&state).await?
    };

    // Newest first, then by id so the order is total and pagination will be
    // stable when it arrives.
    posts.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| a.id.cmp(&b.id))
    });
    Ok(Json(posts))
}

/// The `GET /posts` fallback: scan every post event and fold per entity.
async fn fold_all_posts(state: &AppState) -> Result<Vec<PostView>, ApiError> {
    use std::collections::BTreeMap;

    let response = state
        .allsource
        .query
        .query_events(
            QueryEventsParams::new()
                .event_type_prefix("content.post.")
                .limit(10_000),
        )
        .await?;

    // Group by stream, then fold each stream with the *same* folder the
    // per-entity path uses, so the two cannot drift.
    let mut by_entity: BTreeMap<String, Vec<rv2_allsource::SdkEvent>> = BTreeMap::new();
    for event in response.events {
        by_entity
            .entry(event.entity_id.clone())
            .or_default()
            .push(event);
    }
    Ok(by_entity
        .values()
        .filter_map(|events| allsource::fold_events::<rv2_allsource::PostFolder>(events))
        .collect())
}

/// `GET /posts/{id}` — fold-on-read.
///
/// # Errors
///
/// 404 if the stream is empty or the post was deleted.
pub async fn get_one(
    State(state): State<Arc<AppState>>,
    ExtractAuthUser(_actor): ExtractAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<PostView>, ApiError> {
    Ok(Json(load(&state, id).await?))
}

/// `PATCH /posts/{id}`.
///
/// # Errors
///
/// 404 if absent, 403 if the actor is not the author, 422 on validation.
pub async fn update(
    State(state): State<Arc<AppState>>,
    ExtractAuthUser(actor): ExtractAuthUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdatePostRequest>,
) -> Result<Json<PostView>, ApiError> {
    let existing = load(&state, id).await?;
    if !post::can_edit_post(actor.id, &existing) {
        return Err(ApiError::Forbidden);
    }
    let event = post::edit(id, Utc::now(), &request)?;
    state.writer.append(&event).await?;
    Ok(Json(load(&state, id).await?))
}

/// `DELETE /posts/{id}`.
///
/// # Errors
///
/// 404 if absent, 403 if the actor is not the author.
pub async fn delete(
    State(state): State<Arc<AppState>>,
    ExtractAuthUser(actor): ExtractAuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let existing = load(&state, id).await?;
    if !post::can_delete_post(actor.id, &existing) {
        return Err(ApiError::Forbidden);
    }
    let event: DomainEvent = post::delete(id, Utc::now());
    state.writer.append(&event).await?;
    Ok(StatusCode::NO_CONTENT)
}
