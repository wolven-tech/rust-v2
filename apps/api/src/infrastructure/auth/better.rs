//! Building the better-auth stack.
//!
//! ## The `allsource-auth` feature, and why it exists
//!
//! With the feature **off** the adapter is `MemoryDatabaseAdapter`, so
//! `cargo test -p api` is hermetic and needs no running Core. With it **on**
//! the adapter is the vendored [`AllsourceAuthAdapter`], which stores users and
//! sessions as AllSource events. This split is copied from getformlab and is
//! the reason the API's own test suite is fast.
//!
//! ## Storage model (§5.2)
//!
//! The adapter maps every better-auth entity to a stream `auth-{type}:{id}`,
//! appends a **full-state snapshot** on each create/update, and marks deletes
//! with a `_deleted: true` tombstone. That is event-*storage*, not
//! event-*sourcing* — and it is fine: auth entities are small, mutable, and
//! read on every request.
//!
//! We deliberately do **not** unify these with `DomainEvent`. `auth-*` /
//! `auth.*` is the adapter's private namespace and our folders never read it;
//! `rv2-events` owns `identity.user.*` separately.

use std::sync::Arc;

use better_auth::{
    AuthBuilder, AuthConfig, BetterAuth,
    plugins::{EmailPasswordPlugin, SessionManagementPlugin},
};
use rv2_shared::ServerConfig;

#[cfg(feature = "allsource-auth")]
pub type ApiAuthDb = better_auth_allsource::AllsourceAuthAdapter;
#[cfg(not(feature = "allsource-auth"))]
pub type ApiAuthDb = better_auth::adapters::MemoryDatabaseAdapter;

pub type ApiBetterAuth = BetterAuth<ApiAuthDb>;

#[cfg(feature = "allsource-auth")]
fn adapter(config: &ServerConfig) -> ApiAuthDb {
    better_auth_allsource::AllsourceAuthAdapter::new(
        &config.allsource_core_url,
        &config.allsource_query_url,
        &config.allsource_api_key,
    )
}

#[cfg(not(feature = "allsource-auth"))]
fn adapter(_config: &ServerConfig) -> ApiAuthDb {
    better_auth::adapters::MemoryDatabaseAdapter::new()
}

/// Build the auth stack.
///
/// # Errors
///
/// A `String` if better-auth rejects the configuration — most likely a secret
/// under its minimum length.
pub async fn build_auth(config: &ServerConfig) -> Result<Arc<ApiBetterAuth>, String> {
    let mut auth_config = AuthConfig::new(&config.jwt_secret)
        .app_name("rust-v2")
        // NOTE: this MUST include the `/auth` segment. The OAuth plugin appends
        // only `/callback/{provider}`, so a base URL without it produces a
        // redirect_uri that will not match the one registered with Google
        // (§5.4).
        .base_url(&config.auth_base_url)
        .password_min_length(8);

    for origin in &config.cors_origins {
        auth_config = auth_config.trusted_origin(origin.clone());
    }

    let builder = AuthBuilder::new(auth_config)
        .database(adapter(config))
        .plugin(EmailPasswordPlugin::new().enable_signup(true))
        .plugin(SessionManagementPlugin::new());

    // ── SEAM: OAuth ──────────────────────────────────────────────────────────
    // §5.4 specifies Google via `OAuthPlugin`, with the callback landing on
    // `apps/api` and the return origin bound in a short-TTL HMAC-signed
    // `oauth_origin` cookie (getformlab's ADR-006 pattern). That signed-cookie
    // glue is what makes a multi-origin OAuth flow safe rather than an open
    // redirect, and it is **not implemented in this scaffold**.
    //
    // This is a marked gap, not a silent one: credential auth below is complete
    // end to end. Registering the plugin without the origin-binding glue would
    // ship a sign-in button that either fails at the callback or redirects
    // anywhere an attacker names — strictly worse than not offering it.
    //
    // To close it: add `hmac` + `subtle`, port
    // `getformlab:apps/api/src/infrastructure/auth/oauth_glue.rs`, then
    // `.plugin(OAuthPlugin::new().add_provider("google",
    //   OAuthProvider::google(&cfg.client_id, &cfg.client_secret)))`.
    if config.google_oauth.is_some() {
        tracing::warn!(
            "GOOGLE_CLIENT_ID is set but the OAuth plugin is not wired in this build; \
             see the SEAM comment in infrastructure/auth/better.rs"
        );
    }

    builder
        .build()
        .await
        .map(Arc::new)
        .map_err(|e| format!("better-auth rejected the configuration: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ServerConfig {
        ServerConfig {
            bind_addr: "127.0.0.1:0".into(),
            allsource_core_url: "http://localhost:3900".into(),
            allsource_query_url: "http://localhost:3900".into(),
            allsource_api_key: "ask_test".into(),
            jwt_secret: "test-secret-key-that-is-at-least-32-characters-long".into(),
            auth_base_url: "http://localhost:4400/auth".into(),
            cors_origins: vec!["http://localhost:4402".into()],
            trusted_proxy_hops: 0,
            google_oauth: None,
        }
    }

    #[tokio::test]
    async fn the_auth_stack_builds_with_the_expected_plugins() {
        let auth = build_auth(&config()).await.expect("auth builds");
        let plugins = auth.plugin_names();
        assert!(plugins.contains(&"email-password"), "got {plugins:?}");
    }

    /// §5.4: a base URL missing the `/auth` segment silently produces a
    /// `redirect_uri` Google will reject. Assert the shape we depend on.
    #[tokio::test]
    async fn the_auth_base_url_keeps_its_auth_segment() {
        let auth = build_auth(&config()).await.unwrap();
        assert!(
            auth.config().base_url.ends_with("/auth"),
            "base_url was {}",
            auth.config().base_url
        );
    }

    #[tokio::test]
    async fn cors_origins_become_trusted_origins() {
        let auth = build_auth(&config()).await.unwrap();
        assert!(
            auth.config()
                .trusted_origins
                .iter()
                .any(|o| o == "http://localhost:4402")
        );
    }
}
