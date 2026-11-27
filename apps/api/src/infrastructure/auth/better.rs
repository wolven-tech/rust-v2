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
    AuthBuilder, AuthConfig, BetterAuth, core_paths,
    plugins::{
        EmailPasswordPlugin, OAuthPlugin, SessionManagementPlugin,
        oauth::{OAuthProvider, OAuthUserInfo},
    },
};
use rv2_shared::{GoogleOAuthConfig, ServerConfig};

use crate::infrastructure::auth::oauth_glue::normalise_email;

#[cfg(feature = "allsource-auth")]
pub type ApiAuthDb = better_auth_allsource::AllsourceAuthAdapter;
#[cfg(not(feature = "allsource-auth"))]
pub type ApiAuthDb = better_auth::adapters::MemoryDatabaseAdapter;

pub type ApiBetterAuth = BetterAuth<ApiAuthDb>;

/// better-auth's core user routes, none of which this API mounts.
///
/// `/update-user` writes `role` and `metadata` (which holds the password hash)
/// straight from the request body, so any signed-in user could grant
/// themselves a role or replace their hash without the old password.
/// `/change-email` skips verification and uniqueness. The delete pair trusts
/// any unexpired verification value. Profile edits go through
/// `PATCH /users/{id}` instead.
const UNMOUNTED_USER_ROUTES: [&str; 4] = [
    core_paths::UPDATE_USER,
    core_paths::CHANGE_EMAIL,
    core_paths::DELETE_USER,
    core_paths::DELETE_USER_CALLBACK,
];

/// The OAuth plugin's token routes. They are mounted by the plugin and would
/// otherwise let a session read or refresh the stored provider tokens; refresh
/// is out of scope for this change (design, Scope).
const UNMOUNTED_OAUTH_ROUTES: [&str; 2] = ["/get-access-token", "/refresh-token"];

/// Session-management and username routes this app does not use. The client reads
/// the session from `/me` and signs out with `/auth/sign-out`, so each of these is
/// a door with no caller. The username route is a second password sign-in that the
/// sign-in gate does not cover. Disabled by path, so every method answers 404.
const UNMOUNTED_SESSION_ROUTES: [&str; 6] = [
    "/get-session",
    "/list-sessions",
    "/revoke-session",
    "/revoke-sessions",
    "/revoke-other-sessions",
    "/sign-in/username",
];

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

    for path in UNMOUNTED_USER_ROUTES
        .into_iter()
        .chain(UNMOUNTED_OAUTH_ROUTES)
        .chain(UNMOUNTED_SESSION_ROUTES)
    {
        auth_config = auth_config.disabled_path(path);
    }

    for origin in &config.cors_origins {
        auth_config = auth_config.trusted_origin(origin.clone());
    }

    // Linking by email is on for Google only. The library links without checking the
    // existing account's verification, so oauth_routes refuses an unverified account
    // after the callback and undoes the link (design, Amendment 1).
    auth_config.account.account_linking.enabled = true;
    auth_config.account.account_linking.trusted_providers = vec!["google".into()];
    // A refused link must leave the account's profile as it was.
    auth_config.account.account_linking.update_user_info_on_link = false;
    auth_config.account.encrypt_oauth_tokens = true;

    let mut builder = AuthBuilder::new(auth_config)
        .database(adapter(config))
        .plugin(EmailPasswordPlugin::new().enable_signup(true))
        .plugin(SessionManagementPlugin::new());

    if let Some(google) = &config.google_oauth {
        builder =
            builder.plugin(OAuthPlugin::new().add_provider("google", google_provider(google)));
    }

    builder
        .build()
        .await
        .map(Arc::new)
        .map_err(|e| format!("better-auth rejected the configuration: {e}"))
}

/// Google with the endpoints from configuration and the verified-email rule.
fn google_provider(google: &GoogleOAuthConfig) -> OAuthProvider {
    let mut provider = OAuthProvider::google(&google.client_id, &google.client_secret);
    provider.auth_url = google.endpoints.auth_url.clone();
    provider.token_url = google.endpoints.token_url.clone();
    provider.user_info_url = google.endpoints.user_info_url.clone();
    provider.map_user_info = verified_google_user;
    provider
}

/// Refuses an address Google has not verified, rather than creating an account under it.
fn verified_google_user(user: serde_json::Value) -> Result<OAuthUserInfo, String> {
    if user["email_verified"].as_bool() != Some(true) {
        return Err("Google reports the email as unverified".into());
    }
    Ok(OAuthUserInfo {
        id: user["sub"].as_str().ok_or("missing sub")?.to_string(),
        email: normalise_email(user["email"].as_str().ok_or("missing email")?),
        name: user["name"].as_str().map(String::from),
        image: user["picture"].as_str().map(String::from),
        email_verified: true,
    })
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
            oauth_flow: None,
        }
    }

    #[tokio::test]
    async fn google_registers_the_oauth_plugin_with_linking_for_google_only() {
        let mut config = config();
        config.google_oauth = Some(GoogleOAuthConfig {
            client_id: "client".into(),
            client_secret: "secret".into(),
            endpoints: rv2_shared::OAuthEndpoints::google(),
        });
        let auth = build_auth(&config).await.expect("auth builds");
        assert!(auth.plugin_names().contains(&"oauth"));
        assert!(auth.config().account.account_linking.enabled);
        assert_eq!(
            auth.config().account.account_linking.trusted_providers,
            vec!["google".to_string()]
        );
        assert!(auth.config().account.encrypt_oauth_tokens);
    }

    #[tokio::test]
    async fn without_google_there_is_no_oauth_plugin() {
        let auth = build_auth(&config()).await.expect("auth builds");
        assert!(!auth.plugin_names().contains(&"oauth"));
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
    async fn the_user_routes_that_write_role_or_email_are_not_mounted() {
        let auth = build_auth(&config()).await.unwrap();
        for path in UNMOUNTED_USER_ROUTES
            .into_iter()
            .chain(UNMOUNTED_OAUTH_ROUTES)
            .chain(UNMOUNTED_SESSION_ROUTES)
        {
            let mut request = better_auth::AuthRequest::new(better_auth::HttpMethod::Post, path);
            request
                .headers
                .insert("origin".into(), "http://localhost:4402".into());
            let response = auth.handle_request(request).await.unwrap();
            assert_eq!(response.status, 404, "{path} is still reachable");
        }
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
