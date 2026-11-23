//! User endpoints: read a user, and update a user's mutable profile fields.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::Utc;
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
    state
        .allsource
        .fold_entity::<rv2_allsource::UserFolder>(&stream(StreamKind::User, id))
        .await?
        .ok_or(ApiError::NotFound { kind: "user" })
}

/// Load `id`, and when it is the actor's own and has no domain stream yet,
/// register it from the session first.
///
/// Sign-up runs inside better-auth, which writes only its private
/// `auth-user:<id>` streams, so nothing else ever appends
/// `identity.user.registered` and every profile read answered 404.
/// better-auth 0.10's database hooks cannot be used to do it at sign-up:
/// `HookedDatabaseAdapter` does not implement `PasskeyOps`, so it is not a
/// `DatabaseAdapter`. Doing it here also covers accounts created before this.
async fn load_or_register(
    state: &AppState,
    id: Uuid,
    actor_id: Uuid,
    actor_email: &str,
) -> Result<UserView, ApiError> {
    match load(state, id).await {
        Err(ApiError::NotFound { .. }) if id == actor_id => {
            let event = user::register(id, actor_email, None, Utc::now())?;
            state.writer.append(&event).await?;
            load(state, id).await
        }
        other => other,
    }
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
/// 403 unless the actor is the account owner.
pub async fn get_one(
    State(state): State<Arc<AppState>>,
    ExtractAuthUser(actor): ExtractAuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserView>, ApiError> {
    let view = load_or_register(&state, id, actor.id, &actor.email).await?;
    if !user::can_view_profile(actor.id, &view) {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(view))
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
    let existing = load_or_register(&state, id, actor.id, &actor.email).await?;
    // R4: the ownership rule is a named predicate in `rv2-domain`, not an
    // inline `if` in the handler.
    if !user::can_edit_profile(actor.id, &existing) {
        return Err(ApiError::Forbidden);
    }
    let event = user::update_profile(id, Utc::now(), &request)?;
    state.writer.append(&event).await?;
    Ok(Json(load(&state, id).await?))
}
