//! Domain rules: validation and authorization, as pure functions.
//!
//! **Layer 1, WASM-safe.** This crate has no I/O and no clock. Every function
//! that needs "now" takes it as an argument, so the same rule can run in a
//! handler, in a test, and in the browser without behaving differently.
//!
//! Two reasons it exists as its own crate rather than living in the handlers:
//!
//! - **R4 mitigation.** The architecture doc flags "authorization moved from
//!   Postgres RLS into the application" as the risk most likely to cause a real
//!   incident. Ownership checks live here as *named, greppable, unit-tested*
//!   functions ([`can_edit_post`], [`can_delete_post`]) so a handler that
//!   forgets one is a visible omission rather than an invisible one.
//! - The Dioxus apps can run the same validation client-side for instant
//!   feedback, with the server re-running it as the real gate.

#![forbid(unsafe_code)]

pub mod post;
pub mod user;

use thiserror::Error;

/// A rejected command. Carries the offending field so `apps/api` can render a
/// per-field error without string-matching a message.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{field}: {reason}")]
pub struct ValidationError {
    pub field: &'static str,
    pub reason: String,
}

impl ValidationError {
    #[must_use]
    pub fn new(field: &'static str, reason: impl Into<String>) -> Self {
        Self {
            field,
            reason: reason.into(),
        }
    }
}

/// Result of a validating constructor.
pub type Validated<T> = Result<T, ValidationError>;

/// Trim, then reject if empty or longer than `max`.
///
/// # Errors
///
/// [`ValidationError`] when the trimmed value is empty or exceeds `max` chars.
pub fn non_empty_bounded(field: &'static str, value: &str, max: usize) -> Validated<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::new(field, "must not be empty"));
    }
    // Count chars, not bytes: a 300-byte emoji title is not a 300-character
    // title, and byte-length limits produce confusing errors for non-ASCII.
    let len = trimmed.chars().count();
    if len > max {
        return Err(ValidationError::new(
            field,
            format!("must be at most {max} characters (got {len})"),
        ));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_bounded_trims_and_bounds() {
        assert_eq!(non_empty_bounded("t", "  hi  ", 10).unwrap(), "hi");
        assert!(non_empty_bounded("t", "   ", 10).is_err());
        assert!(non_empty_bounded("t", "abcdefghijk", 10).is_err());
        assert_eq!(
            non_empty_bounded("t", "abcdefghij", 10).unwrap(),
            "abcdefghij"
        );
    }

    /// The limit is in characters, so multi-byte text is not penalised.
    #[test]
    fn bounds_count_characters_not_bytes() {
        let ten_emoji = "🦀".repeat(10);
        assert_eq!(ten_emoji.len(), 40, "40 bytes");
        assert!(non_empty_bounded("t", &ten_emoji, 10).is_ok());
        assert!(non_empty_bounded("t", &"🦀".repeat(11), 10).is_err());
    }
}
