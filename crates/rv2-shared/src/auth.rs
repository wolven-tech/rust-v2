//! The authenticated principal.
//!
//! §5.4: better-auth stores `user.role` as a single string. getformlab parses
//! it as a comma-separated **set**, and rust-v2 adopts the set model from day
//! one even though only one role exists — widening a scalar to a set after
//! users are stored means rewriting every stored user record.

use std::{collections::BTreeSet, fmt, str::FromStr};

use uuid::Uuid;

/// A role a user may hold. Unknown strings are preserved rather than dropped,
/// so a role added by a newer deployment survives a round-trip through an older
/// one instead of being silently stripped from the record.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    User,
    Admin,
    Other(String),
}

impl Role {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Role::User => "user",
            Role::Admin => "admin",
            Role::Other(s) => s,
        }
    }

    /// Parse better-auth's comma-separated `user.role` string into a set.
    ///
    /// An empty or whitespace-only input yields `{User}` — every authenticated
    /// principal holds at least the base role, so callers never have to handle
    /// "authenticated but role-less".
    #[must_use]
    pub fn parse_set(raw: &str) -> BTreeSet<Role> {
        let set: BTreeSet<Role> = raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().expect("Role::from_str is infallible"))
            .collect();
        if set.is_empty() {
            BTreeSet::from([Role::User])
        } else {
            set
        }
    }

    /// Render a set back to better-auth's storage form. Sorted, so the string
    /// is stable and two equal sets never produce two different records.
    #[must_use]
    pub fn render_set(roles: &BTreeSet<Role>) -> String {
        roles.iter().map(Role::as_str).collect::<Vec<_>>().join(",")
    }
}

impl FromStr for Role {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_ascii_lowercase().as_str() {
            "user" => Role::User,
            "admin" => Role::Admin,
            other => Role::Other(other.to_string()),
        })
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The authenticated principal, produced by `apps/api`'s `ExtractAuthUser`.
///
/// R4: handlers take this by value, not `Option<AuthUser>`, so "I forgot to
/// check authentication" is a type error rather than an omission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub roles: BTreeSet<Role>,
}

impl AuthUser {
    #[must_use]
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.has_role(&Role::Admin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_set_handles_the_real_better_auth_shapes() {
        assert_eq!(Role::parse_set("user"), BTreeSet::from([Role::User]));
        assert_eq!(
            Role::parse_set("user,admin"),
            BTreeSet::from([Role::User, Role::Admin])
        );
        assert_eq!(
            Role::parse_set(" User , ADMIN "),
            BTreeSet::from([Role::User, Role::Admin]),
            "case and whitespace insensitive"
        );
    }

    /// Every authenticated principal has at least the base role, so no caller
    /// needs a "no roles at all" branch.
    #[test]
    fn an_empty_role_string_still_yields_the_base_role() {
        assert_eq!(Role::parse_set(""), BTreeSet::from([Role::User]));
        assert_eq!(Role::parse_set("  , ,"), BTreeSet::from([Role::User]));
    }

    /// A role this build has never heard of must survive a read/write cycle.
    #[test]
    fn unknown_roles_round_trip_rather_than_being_dropped() {
        let set = Role::parse_set("user,moderator");
        assert!(set.contains(&Role::Other("moderator".into())));
        let rendered = Role::render_set(&set);
        assert_eq!(Role::parse_set(&rendered), set);
    }

    #[test]
    fn render_is_stable_and_sorted() {
        assert_eq!(
            Role::render_set(&Role::parse_set("user,admin")),
            Role::render_set(&Role::parse_set("admin,user"))
        );
    }
}
