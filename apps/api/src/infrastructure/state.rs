//! `AppState` and its construction.

use std::sync::Arc;

use allframe::resilience::KeyedRateLimiter;
use rv2_allsource::{AllSource, EventWriter, PostsReadModel, ProjectionHandle};
use rv2_analytics::Analytics;
use rv2_email::Mailer;
use rv2_shared::ServerConfig;

use crate::infrastructure::auth::{ApiBetterAuth, build_auth};

/// Everything a handler can reach.
pub struct AppState {
    pub allsource: AllSource,
    pub writer: EventWriter,
    /// The `posts_v2` continuous projection (§4.2). Always `None` while the
    /// worker is suspended (`build_state`); `GET /posts` folds on read. See
    /// [`Self::posts_handle`].
    pub posts: Option<ProjectionHandle<PostsReadModel>>,
    pub auth: Arc<ApiBetterAuth>,
    pub cors_origins: Vec<String>,
    /// Replaces rust-v1's `packages/kv` Upstash limiter (D20). Keyed by client
    /// ip; in-memory, because a rate-limit counter is explicitly *not* an event
    /// (D19 / §9.2).
    pub rate_limiter: KeyedRateLimiter<String>,
    /// `TRUSTED_PROXY_HOPS`. `0` means the limiter ignores `x-forwarded-for`
    /// and keys on the socket address — see `infrastructure::rate_limit`, where
    /// trusting the header by default was both a bypass and an unbounded-memory
    /// vector.
    pub trusted_proxy_hops: usize,
    /// Replaces rust-v1's `packages/analytics` (PostHog). `Disabled` without
    /// `POSTHOG_API_KEY`, which is a supported state — a missing analytics key
    /// must never fail a request that would otherwise have succeeded.
    pub analytics: Analytics,
    /// Replaces rust-v1's `packages/email`, which was templates with no sender:
    /// `RESEND_API_KEY` was declared in its env schema and no code ever read it.
    /// Without the key this still renders, to the log.
    pub mailer: Mailer,
}

impl AppState {
    /// The posts read model, if the worker is running and caught up.
    #[must_use]
    pub fn posts_handle(&self) -> Option<&ProjectionHandle<PostsReadModel>> {
        self.posts.as_ref()
    }
}

/// Build the whole application state from configuration.
///
/// The posts projection worker is not started; see the note in the body.
///
/// # Errors
///
/// A `String` describing the first fatal failure — an unusable AllSource URL or
/// an auth stack that will not build. Both are configuration errors, so failing
/// at boot is correct.
pub async fn build_state(config: &ServerConfig) -> Result<AppState, String> {
    let allsource = AllSource::new(
        &config.allsource_core_url,
        &config.allsource_query_url,
        &config.allsource_api_key,
    )
    .map_err(|e| format!("could not construct AllSource clients: {e}"))?;

    let auth = build_auth(config)
        .await
        .map_err(|e| format!("could not build the auth stack: {e}"))?;

    // Neither of these can fail the boot: both degrade to a no-op when their key
    // is absent, which is what makes `cargo run -p api` work on a fresh clone
    // with no vendor accounts at all.
    let analytics = Analytics::from_env().await;
    let mailer = Mailer::from_env();

    // Not started: Core 0.23 reports `version: 1` on every queried and streamed
    // event, and the SDK worker drops any event at or below an entity's last
    // applied version, so it never applies an edit or a delete (foundation doc
    // C.2-f; contract test B18 fails once Core reports a real sequence).
    // `GET /posts` folds on read meanwhile, and `/ready` treats no worker as a
    // supported mode.
    let posts: Option<ProjectionHandle<PostsReadModel>> = None;

    Ok(AppState {
        writer: allsource.writer(),
        allsource,
        posts,
        auth,
        cors_origins: config.cors_origins.clone(),
        // 60 rps sustained, 120 burst, per client ip. Generous enough that a
        // normal SPA never notices and tight enough to blunt a scripted abuse
        // loop; tune once there is real traffic to measure.
        rate_limiter: KeyedRateLimiter::new(60, 120),
        trusted_proxy_hops: config.trusted_proxy_hops,
        analytics,
        mailer,
    })
}
