//! A2UI over REST with JSONL bodies (A2UI7 in
//! `docs/architecture/004-a2ui-surface.md`).
//!
//! Every route requires a session: a surface is part of the signed-in app, and
//! an agent behind it may one day read the user's data.

use axum::{
    Json,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use rv2_a2ui::{ClientBody, ClientMessage, ServerMessage, catalog_document, to_jsonl};
use rv2_domain::ValidationError;

use crate::infrastructure::{auth::ExtractAuthUser, error::ApiError};
use crate::presentation::agents::runway;

const JSONL: &str = "application/jsonl";

fn jsonl(messages: &[ServerMessage]) -> Response {
    ([(header::CONTENT_TYPE, JSONL)], to_jsonl(messages)).into_response()
}

/// `GET /a2ui/catalog` — the catalog's JSON Schema, from the same table the
/// client validates against, so an agent built from it matches this deploy.
pub async fn catalog(ExtractAuthUser(_actor): ExtractAuthUser) -> Json<serde_json::Value> {
    Json(catalog_document())
}

/// `GET /a2ui/surfaces/{name}` — the messages that open a surface.
///
/// # Errors
///
/// 404 for a surface no agent serves.
pub async fn open(
    ExtractAuthUser(_actor): ExtractAuthUser,
    Path(name): Path<String>,
) -> Result<Response, ApiError> {
    match name.as_str() {
        runway::SURFACE_ID => Ok(jsonl(&runway::open())),
        _ => Err(ApiError::NotFound { kind: "surface" }),
    }
}

/// `POST /a2ui/actions` — one action in, the messages that answer it out.
///
/// # Errors
///
/// 422 for a version other than v0.9, an `error` body sent here, or an action
/// the surface does not offer; 404 for a surface no agent serves.
pub async fn act(
    ExtractAuthUser(_actor): ExtractAuthUser,
    Json(message): Json<ClientMessage>,
) -> Result<Response, ApiError> {
    message
        .check_version()
        .map_err(|e| ValidationError::new("version", e.to_string()))?;
    let ClientBody::Action(action) = message.body else {
        return Err(ValidationError::new("action", "send client errors to /a2ui/errors").into());
    };
    match action.surface_id.as_str() {
        runway::SURFACE_ID => Ok(jsonl(&runway::act(&action)?)),
        _ => Err(ApiError::NotFound { kind: "surface" }),
    }
}

/// `POST /a2ui/errors` — a client could not apply a message.
///
/// Logged at warn with the user, because each one means an agent emitted
/// something the catalog refuses: a defect on our side, not the user's.
///
/// # Errors
///
/// 422 for an `action` body sent here.
pub async fn report(
    ExtractAuthUser(actor): ExtractAuthUser,
    Json(message): Json<ClientMessage>,
) -> Result<StatusCode, ApiError> {
    let ClientBody::Error(error) = message.body else {
        return Err(ValidationError::new("error", "send actions to /a2ui/actions").into());
    };
    tracing::warn!(
        user = %actor.id,
        surface = %error.surface_id,
        code = %error.code,
        path = %error.path,
        message = %error.message,
        "a2ui client rejected a message"
    );
    Ok(StatusCode::NO_CONTENT)
}
