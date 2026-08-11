//! User/profile invariants and the command → event functions.

use chrono::{DateTime, Utc};
use rv2_api_types::{UpdateProfileRequest, UserView};
use rv2_events::DomainEvent;
use uuid::Uuid;

use crate::{Validated, ValidationError, non_empty_bounded};

pub const FULL_NAME_MAX: usize = 120;
pub const AVATAR_URL_MAX: usize = 2_000;

/// Produce the registration event.
///
/// Note this is `identity.user.registered` on the **domain** stream
/// (`user:{id}`), which is separate from better-auth's private `auth-user:{id}`
/// stream. §5.2: the two namespaces never merge; the adapter owns `auth.*` and
/// this crate owns `identity.*`.
///
/// # Errors
///
/// [`ValidationError`] if the email is empty or does not contain an `@`.
pub fn register(
    id: Uuid,
    email: &str,
    full_name: Option<&str>,
    now: DateTime<Utc>,
) -> Validated<DomainEvent> {
    let email = non_empty_bounded("email", email, 320)?;
    // Deliberately minimal: full RFC 5322 validation in the domain layer is a
    // liability (it rejects valid addresses). better-auth owns credential-grade
    // checking; this only catches obvious garbage before it reaches the log.
    if !email.contains('@') {
        return Err(ValidationError::new("email", "must contain '@'"));
    }
    let full_name = full_name
        .map(|n| non_empty_bounded("full_name", n, FULL_NAME_MAX))
        .transpose()?;
    Ok(DomainEvent::UserRegistered {
        id,
        email: email.to_lowercase(),
        occurred_at: now,
        full_name,
        avatar_url: None,
    })
}

/// Validate a sparse profile patch and produce the event.
///
/// The doubly-nested `Option` is load-bearing: `None` means "field absent from
/// the request", `Some(None)` means "explicitly cleared".
///
/// # Errors
///
/// [`ValidationError`] if a set field is invalid, or if the patch is empty.
pub fn update_profile(
    id: Uuid,
    now: DateTime<Utc>,
    req: &UpdateProfileRequest,
) -> Validated<DomainEvent> {
    if req.full_name.is_none() && req.avatar_url.is_none() {
        return Err(ValidationError::new(
            "body",
            "must change at least one field",
        ));
    }
    let full_name = match &req.full_name {
        None => None,
        Some(None) => Some(None),
        Some(Some(n)) => Some(Some(non_empty_bounded("full_name", n, FULL_NAME_MAX)?)),
    };
    let avatar_url = match &req.avatar_url {
        None => None,
        Some(None) => Some(None),
        Some(Some(u)) => {
            let url = non_empty_bounded("avatar_url", u, AVATAR_URL_MAX)?;
            if !(url.starts_with("https://") || url.starts_with("http://")) {
                return Err(ValidationError::new("avatar_url", "must be an http(s) URL"));
            }
            Some(Some(url))
        }
    };
    Ok(DomainEvent::UserProfileUpdated {
        id,
        occurred_at: now,
        full_name,
        avatar_url,
    })
}

/// R4: only the account owner may edit their own profile.
#[must_use]
pub const fn can_edit_profile(actor: Uuid, target: &UserView) -> bool {
    u128::from_be_bytes(*actor.as_bytes()) == u128::from_be_bytes(*target.id.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        "2026-01-15T10:30:00Z".parse().unwrap()
    }

    #[test]
    fn register_normalises_email_case() {
        let event = register(Uuid::nil(), "  Ada@Example.COM ", None, now()).unwrap();
        match event {
            DomainEvent::UserRegistered { email, .. } => assert_eq!(email, "ada@example.com"),
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn register_rejects_obvious_garbage() {
        assert_eq!(
            register(Uuid::nil(), "not-an-email", None, now())
                .unwrap_err()
                .field,
            "email"
        );
        assert_eq!(
            register(Uuid::nil(), "   ", None, now()).unwrap_err().field,
            "email"
        );
    }

    #[test]
    fn clearing_a_field_survives_validation_as_a_clear() {
        let event = update_profile(
            Uuid::nil(),
            now(),
            &UpdateProfileRequest {
                full_name: Some(None),
                avatar_url: None,
            },
        )
        .unwrap();
        match event {
            DomainEvent::UserProfileUpdated {
                full_name,
                avatar_url,
                ..
            } => {
                assert_eq!(full_name, Some(None), "explicit clear survives");
                assert_eq!(avatar_url, None, "absent stays absent");
            }
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn avatar_urls_must_be_http() {
        let err = update_profile(
            Uuid::nil(),
            now(),
            &UpdateProfileRequest {
                full_name: None,
                avatar_url: Some(Some("javascript:alert(1)".into())),
            },
        )
        .unwrap_err();
        assert_eq!(err.field, "avatar_url");
    }

    #[test]
    fn an_empty_patch_is_rejected() {
        assert_eq!(
            update_profile(Uuid::nil(), now(), &UpdateProfileRequest::default())
                .unwrap_err()
                .field,
            "body"
        );
    }

    #[test]
    fn only_the_owner_may_edit_a_profile() {
        let target = UserView {
            id: Uuid::from_u128(1),
            email: "a@b.c".into(),
            full_name: None,
            avatar_url: None,
            created_at: now(),
            updated_at: now(),
        };
        assert!(can_edit_profile(Uuid::from_u128(1), &target));
        assert!(!can_edit_profile(Uuid::from_u128(2), &target));
    }
}
