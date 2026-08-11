//! `ExtractAuthUser` — turning a request into an authenticated principal.
//!
//! Ported from `getformlab:apps/api/src/infrastructure/auth/middleware.rs`,
//! which accepts a session **cookie** or a `Bearer` header. The cookie is the
//! browser path (D17); `Bearer` exists for CLI and integration tests.
//!
//! ## R4, restated as a type
//!
//! Handlers take `ExtractAuthUser` **by value**, never `Option<…>`. There is no
//! database-level row security behind the handler to catch a missed check, so
//! "I forgot the auth check" has to be a compile error rather than a silent
//! data leak.
//!
//! ## R5, stated honestly
//!
//! This costs **two** AllSource round-trips per authenticated request
//! (`get_session`, then `get_user_by_id`) with no cache. §9.2 specifies a ≤30s
//! in-process session cache as the mitigation; it is **not implemented here**.
//! Measure p99 on the authenticated path before adding it — the doc is explicit
//! that measuring comes first.

use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
    response::{IntoResponse, Response},
};
use better_auth::{AuthSession, AuthUser as _, SessionManager, UserOps};
use rv2_shared::{AuthUser, Role};
use uuid::Uuid;

use crate::infrastructure::{error::ApiError, state::AppState};

/// Namespace for deriving a stable `Uuid` from a non-UUID better-auth user id.
///
/// better-auth types `User.id` as a `String` and does not promise it is a UUID
/// (the memory adapter's ids are not). Our domain streams are `user:<uuid>`, so
/// we need a total, **deterministic** mapping — v5 hashing gives one, and the
/// same better-auth id always yields the same domain uuid across restarts and
/// across processes. Parsing-or-failing would 500 on a perfectly valid user.
const USER_ID_NAMESPACE: Uuid = Uuid::from_u128(0x7276_3200_0000_4000_8000_0000_0000_0001);

/// Map a better-auth user id onto the uuid our domain streams are keyed by.
#[must_use]
pub fn domain_user_id(better_auth_id: &str) -> Uuid {
    better_auth_id
        .parse()
        .unwrap_or_else(|_| Uuid::new_v5(&USER_ID_NAMESPACE, better_auth_id.as_bytes()))
}

/// Pull the session token out of a request: `Authorization: Bearer …` first,
/// then the session cookie.
fn extract_token(parts: &Parts, cookie_name: &str) -> Option<String> {
    if let Some(bearer) = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        return Some(bearer.to_string());
    }

    parts
        .headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())?
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == cookie_name)
        .map(|(_, value)| value.to_string())
        .filter(|v| !v.is_empty())
}

/// The authenticated principal, as an axum extractor.
#[derive(Debug, Clone)]
pub struct ExtractAuthUser(pub AuthUser);

impl FromRequestParts<Arc<AppState>> for ExtractAuthUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let cookie_name = state.auth.config().session.cookie_name.clone();
        let token = extract_token(parts, &cookie_name)
            .ok_or_else(|| ApiError::Unauthenticated.into_response())?;

        let sessions = SessionManager::new(
            Arc::new(state.auth.config().clone()),
            state.auth.database().clone(),
        );

        // Round-trip 1.
        let session = sessions
            .get_session(&token)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()).into_response())?
            .ok_or_else(|| ApiError::Unauthenticated.into_response())?;

        // Round-trip 2. See the R5 note at the top of this module.
        let user = state
            .auth
            .database()
            .get_user_by_id(session.user_id())
            .await
            .map_err(|e| ApiError::Internal(e.to_string()).into_response())?
            .ok_or_else(|| ApiError::Unauthenticated.into_response())?;

        Ok(ExtractAuthUser(AuthUser {
            id: domain_user_id(user.id()),
            email: user.email().unwrap_or_default().to_string(),
            roles: Role::parse_set(user.role().unwrap_or_default()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use axum::http::Request;

    use super::*;

    fn parts(headers: &[(&str, &str)]) -> Parts {
        let mut builder = Request::builder();
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        builder.body(()).unwrap().into_parts().0
    }

    #[test]
    fn a_bearer_header_wins_over_a_cookie() {
        let p = parts(&[
            ("authorization", "Bearer header-token"),
            ("cookie", "session=cookie-token"),
        ]);
        assert_eq!(
            extract_token(&p, "session").as_deref(),
            Some("header-token")
        );
    }

    #[test]
    fn the_session_cookie_is_found_among_others() {
        let p = parts(&[("cookie", "theme=dark; session=abc123; locale=en")]);
        assert_eq!(extract_token(&p, "session").as_deref(), Some("abc123"));
    }

    /// A cookie whose name merely *contains* the session name must not match —
    /// `not_session=x` is a different cookie.
    #[test]
    fn a_similarly_named_cookie_is_not_mistaken_for_the_session() {
        let p = parts(&[("cookie", "not_session=x; session_backup=y")]);
        assert_eq!(extract_token(&p, "session"), None);
    }

    #[test]
    fn no_credentials_at_all_yields_none() {
        assert_eq!(extract_token(&parts(&[]), "session"), None);
        assert_eq!(
            extract_token(&parts(&[("authorization", "Bearer   ")]), "session"),
            None,
            "a blank bearer is not a token"
        );
        assert_eq!(
            extract_token(&parts(&[("cookie", "session=")]), "session"),
            None,
            "an empty cookie value is not a token"
        );
    }

    /// The mapping must be a function: same input, same output, every time and
    /// in every process. Otherwise a user's domain stream moves on restart.
    #[test]
    fn domain_user_ids_are_deterministic() {
        assert_eq!(domain_user_id("abc123"), domain_user_id("abc123"));
        assert_ne!(domain_user_id("abc123"), domain_user_id("abc124"));
    }

    /// When better-auth *does* hand back a UUID, use it verbatim rather than
    /// hashing it — otherwise a migrated user's id would change.
    #[test]
    fn an_id_that_is_already_a_uuid_passes_through_unchanged() {
        let id = "6f1c8e2a-0000-4000-8000-000000000001";
        assert_eq!(domain_user_id(id).to_string(), id);
    }
}
