//! Wire DTOs for the `apps/api` REST surface.
//!
//! **Layer 1, WASM-safe.** These types are the contract between `apps/api` and
//! the Dioxus clients, and they are also the `State` types that the folders in
//! `rv2-allsource` produce. That is deliberate: a read model and the JSON the
//! API serves are the same shape, so there is no mapping layer to drift.
//!
//! Nothing here may depend on `allsource`, `axum`, `reqwest`, or `tokio` — the
//! Dioxus apps compile this crate to `wasm32-unknown-unknown`.
//!
//! These types replace `packages/supabase`'s generated `Tables<"posts">` /
//! `TablesUpdate<"users">` (§7 item 9). They are hand-written because AllSource
//! has no schema to generate from — which makes them the source of truth rather
//! than a derivative of one.

#![forbid(unsafe_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ═════════════════════════════════════════════════════════════════════════════
// Read models
// ═════════════════════════════════════════════════════════════════════════════

/// A post as folded from `content.post.*`.
///
/// `created_at`/`updated_at` come from the payload's `occurred_at`, never from
/// the AllSource envelope timestamp (D13).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostView {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A user profile as folded from `identity.user.*`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserView {
    pub id: Uuid,
    pub email: String,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ═════════════════════════════════════════════════════════════════════════════
// Requests
// ═════════════════════════════════════════════════════════════════════════════

/// `POST /posts`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

/// `PATCH /posts/{id}` — sparse. Absent field = unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePostRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// `PATCH /users/{id}` — doubly-sparse. Absent key = unchanged; explicit `null`
/// = cleared.
///
/// The `deserialize_with` is mandatory, not decorative: serde's derived impl
/// maps a JSON `null` onto the **outer** `None`, so without it a client's
/// explicit "clear this field" arrives at the handler as "leave it alone" and
/// the request is silently ignored. See [`double_option`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub full_name: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub avatar_url: Option<Option<String>>,
}

/// Deserializer for a doubly-sparse patch field.
///
/// Duplicated from `rv2_events::double_option` rather than imported: this crate
/// is layer 1 and `rv2-events` is layer 0, so importing would be legal — but
/// it would make the *wire DTO* depend on the *event model*, and the whole
/// point of having both is that the API contract can change without touching
/// the immutable event schema. Six lines is a cheaper price than that coupling.
///
/// # Errors
///
/// Whatever the inner `Option<T>` deserializer produces.
pub fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

// ═════════════════════════════════════════════════════════════════════════════
// Responses
// ═════════════════════════════════════════════════════════════════════════════

/// `GET /health`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    /// Whether Core answered its own `/health` on the last check.
    pub allsource_reachable: bool,
}

/// The session shape `GET /auth/get-session` is normalised into.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionView {
    pub user_id: Uuid,
    pub email: String,
    #[serde(default)]
    pub roles: Vec<String>,
}

/// Uniform error body. Every non-2xx response from `apps/api` is one of these,
/// so `rv2-client` has exactly one error shape to parse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Stable machine-readable code, e.g. `validation_failed`, `not_found`.
    pub code: String,
    /// Human-readable detail. Do not parse this.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_update_omits_absent_fields_entirely() {
        let req = UpdatePostRequest {
            title: Some("new".into()),
            content: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"title":"new"}"#);
    }

    /// The doubly-sparse encoding is the whole reason `UpdateProfileRequest`
    /// uses `Option<Option<_>>`: "clear this field" and "leave this field
    /// alone" must be distinguishable on the wire.
    #[test]
    fn clearing_a_profile_field_is_distinguishable_from_leaving_it_alone() {
        let clear = UpdateProfileRequest {
            full_name: Some(None),
            avatar_url: None,
        };
        assert_eq!(
            serde_json::to_string(&clear).unwrap(),
            r#"{"full_name":null}"#
        );

        let parsed: UpdateProfileRequest = serde_json::from_str(r#"{"full_name":null}"#).unwrap();
        assert_eq!(parsed.full_name, Some(None), "explicit null = clear");
        assert_eq!(parsed.avatar_url, None, "absent = unchanged");

        let empty: UpdateProfileRequest = serde_json::from_str("{}").unwrap();
        assert_eq!(empty, UpdateProfileRequest::default());
    }

    #[test]
    fn read_models_round_trip() {
        let post = PostView {
            id: Uuid::nil(),
            author_id: Uuid::nil(),
            title: "t".into(),
            content: "c".into(),
            created_at: "2026-01-15T10:30:00Z".parse().unwrap(),
            updated_at: "2026-01-15T10:30:00Z".parse().unwrap(),
        };
        let json = serde_json::to_string(&post).unwrap();
        assert_eq!(serde_json::from_str::<PostView>(&json).unwrap(), post);
    }
}
