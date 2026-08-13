//! `ServerConfig::from_env`.
//!
//! ## R6 — never hard-code a port
//!
//! The architecture doc's risk R6 records that three sources give three
//! different port pairs for the same AllSource services (README `:3900`/`:3902`,
//! the QS reference `:3280`/`:3283`, getformlab `:3854`/`:3855`). The mitigation
//! is enforced here: [`ServerConfig::from_env`] **requires**
//! `ALLSOURCE_CORE_URL` and `ALLSOURCE_QUERY_URL` and has no fallback for them.
//! A misconfigured deployment fails at boot with a named variable rather than
//! silently talking to the wrong port.
//!
//! Ports that *are* defaulted are our own (`PORT`), because those are not in
//! dispute — and the default is still overridable.
//!
//! Note also getformlab's `AllSourceConfig::new`, which derives the query URL by
//! string-replacing `:3854` → `:3855`. We deliberately do not copy that.

use std::{env, fmt};

/// Everything `apps/api` needs from the environment.
#[derive(Clone)]
pub struct ServerConfig {
    /// Address `apps/api` binds. `HOST` + `PORT`, defaulted.
    pub bind_addr: String,
    /// AllSource Core, e.g. `http://localhost:3900`. **Required.**
    pub allsource_core_url: String,
    /// AllSource Query Service, e.g. `http://localhost:3902`. **Required.**
    ///
    /// May legitimately point at Core: the SDK's `QueryClient` calls Core's own
    /// `/api/v1/events/query`, so a single-node dev stack can set both to
    /// `:3900`. See `docker-compose.yml` and the README.
    pub allsource_query_url: String,
    /// API key (`ask_…`). Core in `ALLSOURCE_DEV_MODE` ignores its value, but
    /// the SDK rejects an empty one at construction, so it is still required.
    pub allsource_api_key: String,
    /// Signing secret for better-auth sessions.
    pub jwt_secret: String,
    /// Public base URL of the auth router, **including the `/auth` segment** —
    /// the OAuth plugin appends only `/callback/{provider}` (§5.4).
    pub auth_base_url: String,
    /// Origins allowed to send credentialed requests. Never `*`.
    pub cors_origins: Vec<String>,
    /// How many `x-forwarded-for` hops on the **right** are appended by
    /// infrastructure you control. `0` — the default — means "no trusted
    /// proxy", and the header is then ignored entirely.
    ///
    /// This has to be configured, not inferred. On a directly-exposed service
    /// `x-forwarded-for` is wholly client-controlled, so keying a rate limiter
    /// on its leftmost entry lets any caller mint a fresh bucket per request by
    /// sending a new fake ip — which disables the limiter *and* grows its key
    /// set without bound. Behind exactly one proxy the honest client ip is the
    /// **last** entry, not the first; behind two it is second-from-last. Only
    /// the operator knows which, so only the operator can say.
    pub trusted_proxy_hops: usize,
    /// Google OAuth, if configured. `None` disables the OAuth plugin entirely
    /// rather than registering a half-configured provider.
    pub google_oauth: Option<GoogleOAuthConfig>,
}

#[derive(Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: String,
    pub client_secret: String,
}

/// Redacts secrets. `ServerConfig` gets logged at boot; the secrets must not.
impl fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerConfig")
            .field("bind_addr", &self.bind_addr)
            .field("allsource_core_url", &self.allsource_core_url)
            .field("allsource_query_url", &self.allsource_query_url)
            .field("allsource_api_key", &"<redacted>")
            .field("jwt_secret", &"<redacted>")
            .field("auth_base_url", &self.auth_base_url)
            .field("cors_origins", &self.cors_origins)
            .field("trusted_proxy_hops", &self.trusted_proxy_hops)
            .field("google_oauth", &self.google_oauth.is_some())
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("required environment variable `{0}` is not set")]
    Missing(&'static str),
    #[error("environment variable `{name}` is invalid: {reason}")]
    Invalid { name: &'static str, reason: String },
}

/// The rule, separated from the lookup so it can be tested without mutating the
/// process environment (which is `unsafe` in edition 2024, and this crate
/// `forbid`s that).
fn required_in(name: &'static str, raw: Option<String>) -> Result<String, ConfigError> {
    match raw {
        Some(v) if !v.trim().is_empty() => Ok(v.trim().to_string()),
        _ => Err(ConfigError::Missing(name)),
    }
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    required_in(name, env::var(name).ok())
}

fn optional(name: &str, default: &str) -> String {
    env::var(name)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn validate_url(name: &'static str, value: &str) -> Result<String, ConfigError> {
    if !(value.starts_with("http://") || value.starts_with("https://")) {
        return Err(ConfigError::Invalid {
            name,
            reason: format!("must start with http:// or https:// (got `{value}`)"),
        });
    }
    // Trailing slashes make `format!("{base}/path")` produce `//path`, which
    // some routers 404 on. Normalise once, here.
    Ok(value.trim_end_matches('/').to_string())
}

impl ServerConfig {
    /// Read and validate the environment.
    ///
    /// # Errors
    ///
    /// [`ConfigError::Missing`] for an absent required variable and
    /// [`ConfigError::Invalid`] for a malformed one. Both name the variable, so
    /// the failure is actionable from the log line alone.
    pub fn from_env() -> Result<Self, ConfigError> {
        let core = validate_url("ALLSOURCE_CORE_URL", &required("ALLSOURCE_CORE_URL")?)?;
        let query = validate_url("ALLSOURCE_QUERY_URL", &required("ALLSOURCE_QUERY_URL")?)?;

        let host = optional("HOST", "0.0.0.0");
        let port = optional("PORT", "4400");
        port.parse::<u16>().map_err(|e| ConfigError::Invalid {
            name: "PORT",
            reason: e.to_string(),
        })?;

        let cors_origins: Vec<String> = optional(
            "CORS_ORIGINS",
            "http://localhost:4401,http://localhost:4402",
        )
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
        if cors_origins.iter().any(|o| o == "*") {
            // `Access-Control-Allow-Origin: *` is illegal alongside
            // `Allow-Credentials: true`, and our whole session model is
            // credentialed (D17). Fail loudly rather than silently dropping
            // cookies in the browser (§5.3).
            return Err(ConfigError::Invalid {
                name: "CORS_ORIGINS",
                reason: "`*` is illegal with credentialed requests; list origins explicitly".into(),
            });
        }

        let trusted_proxy_hops = optional("TRUSTED_PROXY_HOPS", "0")
            .parse::<usize>()
            .map_err(|e| ConfigError::Invalid {
                name: "TRUSTED_PROXY_HOPS",
                reason: e.to_string(),
            })?;

        let google_oauth = match (
            env::var("GOOGLE_CLIENT_ID").ok().filter(|v| !v.is_empty()),
            env::var("GOOGLE_CLIENT_SECRET")
                .ok()
                .filter(|v| !v.is_empty()),
        ) {
            (Some(client_id), Some(client_secret)) => Some(GoogleOAuthConfig {
                client_id,
                client_secret,
            }),
            (None, None) => None,
            // Half-configured OAuth is worse than none: it produces a sign-in
            // button that fails at the callback.
            _ => {
                return Err(ConfigError::Invalid {
                    name: "GOOGLE_CLIENT_ID",
                    reason: "GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET must be set together".into(),
                });
            }
        };

        Ok(Self {
            bind_addr: format!("{host}:{port}"),
            allsource_core_url: core,
            allsource_query_url: query,
            allsource_api_key: required("ALLSOURCE_API_KEY")?,
            jwt_secret: required("JWT_SECRET")?,
            auth_base_url: validate_url(
                "AUTH_BASE_URL",
                &optional("AUTH_BASE_URL", "http://localhost:4400/auth"),
            )?,
            cors_origins,
            trusted_proxy_hops,
            google_oauth,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_must_have_a_scheme_and_lose_trailing_slashes() {
        assert_eq!(
            validate_url("X", "http://localhost:3900/").unwrap(),
            "http://localhost:3900"
        );
        assert!(validate_url("X", "localhost:3900").is_err());
        assert!(validate_url("X", "ws://localhost:3900").is_err());
    }

    /// R6: there is deliberately no default for the AllSource URLs, so a
    /// missing one must be a boot failure that **names the variable** — a log
    /// line saying "config error" would leave the operator guessing which of
    /// three documented port pairs was expected.
    #[test]
    fn a_missing_variable_reports_its_own_name() {
        match required_in("ALLSOURCE_CORE_URL", None) {
            Err(ConfigError::Missing(name)) => assert_eq!(name, "ALLSOURCE_CORE_URL"),
            other => panic!("expected Missing, got {other:?}"),
        }
    }

    /// A variable exported as `""` in a shell script is a very common way to
    /// think you configured something. Treat it as unset.
    #[test]
    fn blank_is_treated_as_unset() {
        assert!(required_in("X", Some("   ".into())).is_err());
        assert!(required_in("X", Some(String::new())).is_err());
        assert_eq!(required_in("X", Some("  v  ".into())).unwrap(), "v");
    }
}
