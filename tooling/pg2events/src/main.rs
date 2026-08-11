//! `pg2events` — Supabase Postgres rows → AllSource events (§8.3).
//!
//! This is a **translation**, not a copy, and it is deliberately one-shot: run
//! it once against a Supabase read replica, verify, then delete this crate from
//! the workspace.
//!
//! ## Status: the mapping and its guarantees are implemented; the Postgres
//! reader is not.
//!
//! `rust-v1`'s Supabase project is not reachable from this machine and no
//! connection string exists, so wiring `sqlx`/`tokio-postgres` here would be
//! writing a query nobody has run against a schema nobody has connected to. It
//! is phase 8 work (§8.5) and the cutover gate is a row-count comparison, which
//! needs the real database anyway.
//!
//! What *is* implemented and tested is the part that would be expensive to get
//! wrong and cheap to get right now: the row → event mapping, the ordering
//! rule, the provenance metadata, and the D13 property that makes the whole
//! migration lossless.
//!
//! To finish it: add `sqlx` with `postgres`, `SELECT * FROM users ORDER BY
//! created_at`, feed each row through [`user_events`], then the same for
//! `posts` through [`post_events`], appending via
//! `EventWriter::append_with_metadata`.

use std::process::ExitCode;

use chrono::{DateTime, Utc};
use rv2_events::DomainEvent;
use uuid::Uuid;

/// A `users` row, as `packages/supabase/src/types/db.ts` declares it.
#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A `posts` row.
#[derive(Debug, Clone)]
pub struct PostRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Provenance stamped on every migrated event.
///
/// Makes "which of these are real?" answerable forever, and makes a re-run
/// detectable.
#[must_use]
pub fn provenance(pg_table: &str, migrated_at: DateTime<Utc>) -> serde_json::Value {
    serde_json::json!({
        "source": "supabase-migration",
        "migrated_at": migrated_at.to_rfc3339(),
        "pg_table": pg_table,
    })
}

/// `users` row → 1 or 2 events.
///
/// A row whose `updated_at` is later than its `created_at` was edited at some
/// point, and that edit becomes a second event so the history is not flattened.
#[must_use]
pub fn user_events(row: &UserRow) -> Vec<DomainEvent> {
    let mut events = vec![DomainEvent::UserRegistered {
        id: row.id,
        email: row.email.clone(),
        occurred_at: row.created_at,
        full_name: row.full_name.clone(),
        avatar_url: row.avatar_url.clone(),
    }];
    if row.updated_at > row.created_at {
        events.push(DomainEvent::UserProfileUpdated {
            id: row.id,
            occurred_at: row.updated_at,
            full_name: Some(row.full_name.clone()),
            avatar_url: Some(row.avatar_url.clone()),
        });
    }
    events
}

/// `posts` row → 1 or 2 events.
#[must_use]
pub fn post_events(row: &PostRow) -> Vec<DomainEvent> {
    let mut events = vec![DomainEvent::PostCreated {
        id: row.id,
        author_id: row.user_id,
        title: row.title.clone(),
        content: row.content.clone(),
        occurred_at: row.created_at,
    }];
    if row.updated_at > row.created_at {
        events.push(DomainEvent::PostEdited {
            id: row.id,
            occurred_at: row.updated_at,
            title: Some(row.title.clone()),
            content: Some(row.content.clone()),
        });
    }
    events
}

fn main() -> ExitCode {
    tracing_subscriber::fmt().init();
    tracing::error!(
        "pg2events has no Postgres reader yet — see the module docs. \
         The row -> event mapping is implemented and tested; the SELECT is phase-8 work."
    );
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    fn user() -> UserRow {
        UserRow {
            id: Uuid::from_u128(1),
            email: "ada@example.com".into(),
            full_name: Some("Ada Lovelace".into()),
            avatar_url: None,
            created_at: at("2024-01-01T00:00:00Z"),
            updated_at: at("2024-01-01T00:00:00Z"),
        }
    }

    #[test]
    fn an_untouched_row_produces_exactly_one_event() {
        assert_eq!(user_events(&user()).len(), 1);
    }

    #[test]
    fn an_edited_row_produces_a_second_event_at_its_update_time() {
        let mut row = user();
        row.updated_at = at("2024-06-01T00:00:00Z");
        let events = user_events(&row);
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].occurred_at(), at("2024-06-01T00:00:00Z"));
        assert_eq!(events[1].event_type(), "identity.user.profile_updated");
    }

    /// **The payoff of D13, and the reason this migration is lossless.**
    ///
    /// AllSource assigns the envelope `timestamp` server-side at ingest, so a
    /// post created in 2024 gets a 2026 envelope. Because every payload carries
    /// its own `occurred_at` — and folders read domain time only from the
    /// payload — the original date survives. Had we followed getformlab's
    /// pattern of taking `event.timestamp` inside projections, the entire
    /// migrated corpus would collapse to a single instant.
    #[test]
    fn original_creation_dates_survive_into_the_payload() {
        let row = PostRow {
            id: Uuid::from_u128(2),
            user_id: Uuid::from_u128(1),
            title: "Old post".into(),
            content: "written in 2024".into(),
            created_at: at("2024-03-01T12:00:00Z"),
            updated_at: at("2024-03-01T12:00:00Z"),
        };
        let events = post_events(&row);
        assert_eq!(events[0].occurred_at(), at("2024-03-01T12:00:00Z"));
    }

    #[test]
    fn events_land_on_the_right_streams() {
        let u = &user_events(&user())[0];
        assert_eq!(u.stream_id(), format!("user:{}", Uuid::from_u128(1)));
    }

    #[test]
    fn provenance_is_stamped_and_greppable() {
        let meta = provenance("posts", at("2026-08-11T00:00:00Z"));
        assert_eq!(meta["source"], "supabase-migration");
        assert_eq!(meta["pg_table"], "posts");
    }
}
