//! User endpoints. Replaces `packages/supabase`'s `getUser` / `updateUser`
//! (§7 items 6 and 8).

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::Utc;
use rv2_allsource::QueryEventsParams;
use rv2_api_types::{SessionView, UpdateProfileRequest, UserView};
use rv2_domain::user;
use rv2_events::{StreamKind, stream};
use rv2_shared::Role;
use uuid::Uuid;

use crate::infrastructure::{auth::ExtractAuthUser, error::ApiError, state::AppState};

/// Fold one user's `identity.user.*` stream on read.
///
/// Note this reads the **domain** stream (`user:<uuid>`), not better-auth's
/// private `auth-user:<id>` one — §5.2 keeps the two namespaces separate.
async fn load(state: &AppState, id: Uuid) -> Result<UserView, ApiError> {
    let params = QueryEventsParams::new().entity_id(&stream(StreamKind::User, id));
    state
        .allsource
        .query
        .query_and_fold::<rv2_allsource::UserFolder>(params)
        .await?
        .ok_or(ApiError::NotFound { kind: "user" })
}

/// `GET /me` — the authenticated principal, straight from the session.
///
/// Deliberately does **not** touch AllSource: the session extractor has already
/// paid two round-trips (R5), and making `/me` pay a third to return data it
/// already holds would be gratuitous.
pub async fn me(ExtractAuthUser(actor): ExtractAuthUser) -> Json<SessionView> {
    Json(SessionView {
        user_id: actor.id,
        email: actor.email,
        roles: actor.roles.iter().map(Role::to_string).collect(),
    })
}

/// `GET /users/{id}` — the domain profile, folded on read.
///
/// # Errors
///
/// 404 if the user has no `identity.user.*` stream yet.
pub async fn get_one(
    State(state): State<Arc<AppState>>,
    ExtractAuthUser(_actor): ExtractAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserView>, ApiError> {
    Ok(Json(load(&state, id).await?))
}

/// `PATCH /users/{id}`.
///
/// # Errors
///
/// 403 unless the actor is the account owner, 404 if absent, 422 on validation.
pub async fn update(
    State(state): State<Arc<AppState>>,
    ExtractAuthUser(actor): ExtractAuthUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserView>, ApiError> {
    let existing = load(&state, id).await?;
    // R4: the ownership rule is a named predicate in `rv2-domain`, not an
    // inline `if` in the handler.
    if !user::can_edit_profile(actor.id, &existing) {
        return Err(ApiError::Forbidden);
    }
    let event = user::update_profile(id, Utc::now(), &request)?;
    state.writer.append(&event).await?;
    Ok(Json(load(&state, id).await?))
}
