//! The Google sign-in routes on `apps/api`, and the gate that keeps better-auth's
//! own OAuth endpoints from offering a second way into the same callback.
//!
//! ```text
//! GET /auth/oauth/{provider}/start?return_to=/path&app=<origin>
//!     302 to the provider; Set-Cookie: sealed pending-origin, bound to `state`
//! GET /auth/callback/{provider}?code=…&state=…
//!     opens the cookie, lets better-auth exchange the code and create the
//!     session, then 302 to the admitted target with the session cookie
//! ```
//!
//! better-auth's `/auth/callback/{provider}` would complete a sign-in without
//! the cookie, and its `/auth/sign-in/social` would issue a state no cookie
//! binds. The gate answers every method on those paths itself.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use allsource::{Error as SdkError, IngestEventInput};
use axum::body::Body;
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use better_auth::{
    AccountOps, AuthAccount, AuthResponse, AuthUser, HttpMethod, SessionOps, UserOps,
};
use chrono::{DateTime, Utc};
use rv2_shared::ServerConfig;
use serde::Deserialize;
use serde_json::json;

use crate::infrastructure::auth::oauth_glue::{
    self as glue, Pending, Refusal, ReturnTarget, normalise_email,
};
use crate::infrastructure::error::ApiError;
use crate::infrastructure::rate_limit;
use crate::infrastructure::state::{AppState, Coordination};

/// Upper bound on a credential body the gate will rewrite. Larger bodies are
/// refused before the library sees them.
const MAX_CREDENTIAL_BODY: usize = 16 * 1024;

/// What the OAuth routes need at request time. `None` in [`AppState`] when no
/// provider is configured, and then every OAuth route answers 404.
pub struct OAuthRuntime {
    pending_secret: String,
    return_origins: Vec<String>,
    providers: Vec<&'static str>,
    /// `state` → expiry of every callback that has passed the cookie checks in
    /// this process. A local fast path in front of the shared claim; the shared
    /// claim is the one that holds across instances.
    consumed: Mutex<HashMap<String, i64>>,
}

impl OAuthRuntime {
    #[must_use]
    pub fn from_config(config: &ServerConfig) -> Option<Self> {
        let flow = config.oauth_flow.as_ref()?;
        let mut providers = Vec::new();
        if config.google_oauth.is_some() {
            providers.push("google");
        }
        (!providers.is_empty()).then(|| Self {
            pending_secret: flow.pending_secret.clone(),
            return_origins: flow.return_origins.clone(),
            providers,
            consumed: Mutex::new(HashMap::new()),
        })
    }

    fn offers(&self, provider: &str) -> bool {
        self.providers.contains(&provider)
    }

    /// `true` for the first caller of `state` in this process, `false` after.
    /// Entries past their expiry are dropped on every call.
    fn claim_local(&self, state: &str, expires_at: i64, now: i64) -> bool {
        let Ok(mut consumed) = self.consumed.lock() else {
            return false;
        };
        consumed.retain(|_, expiry| *expiry > now);
        if consumed.contains_key(state) {
            return false;
        }
        consumed.insert(state.to_string(), expires_at);
        true
    }
}

#[derive(Deserialize)]
pub struct StartQuery {
    return_to: Option<String>,
    app: Option<String>,
}

#[derive(Deserialize)]
struct SignInReply {
    url: String,
}

/// Why a callback did not sign anyone in. Logged; the browser is redirected either
/// to the home page or, for an unverified account, to the sign-in page with a message.
enum CallbackError {
    Refused(Refusal),
    Provider(String),
    NoSessionCookie,
    /// The Google email matches an existing account whose email is not verified.
    /// The link is refused and undone.
    UnverifiedLink,
}

impl CallbackError {
    fn reason(&self) -> String {
        match self {
            Self::Refused(refusal) => format!("refused: {refusal:?}"),
            Self::Provider(message) => format!("better-auth: {message}"),
            Self::NoSessionCookie => "better-auth set no session cookie".into(),
            Self::UnverifiedLink => "existing account's email is not verified".into(),
        }
    }
}

pub async fn start(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Query(query): Query<StartQuery>,
) -> Response {
    let Some(runtime) = state.oauth.as_ref().filter(|r| r.offers(&provider)) else {
        return not_found();
    };

    // A refused target is not an error for the user: they sign in and land on the
    // home page, which is where a target with no valid origin should go anyway.
    let target = match glue::admit(
        &runtime.return_origins,
        query.app.as_deref(),
        query.return_to.as_deref(),
    ) {
        Ok(target) => target,
        Err(reason) => {
            tracing::warn!(?reason, "sign-in return target refused; using the app home");
            ReturnTarget::home(&runtime.return_origins)
        }
    };

    let body = serde_json::json!({ "provider": provider })
        .to_string()
        .into_bytes();
    let reply = match library(
        &state,
        HttpMethod::Post,
        "/sign-in/social",
        HashMap::from([("content-type".to_string(), "application/json".to_string())]),
        HashMap::new(),
        Some(body),
    )
    .await
    {
        Ok(response) if response.status == 200 => {
            serde_json::from_slice::<SignInReply>(&response.body).ok()
        }
        Ok(response) => {
            tracing::warn!(
                status = response.status,
                "better-auth refused to start the flow"
            );
            None
        }
        Err(error) => {
            tracing::warn!(%error, "better-auth failed to start the flow");
            None
        }
    };
    let Some(reply) = reply else {
        return internal_error();
    };

    let Some(oauth_state) = url::Url::parse(&reply.url).ok().and_then(|url| {
        url.query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned())
    }) else {
        tracing::warn!("provider URL carries no state");
        return internal_error();
    };

    let pending = Pending::new(&provider, &oauth_state, &target, Utc::now().timestamp());
    let Ok(sealed) = glue::seal(runtime.pending_secret.as_bytes(), &pending) else {
        return internal_error();
    };
    redirect(&reply.url, &[glue::set_cookie(&sealed)])
}

/// Serve `/auth/callback/{provider}`. Called only from [`gate`].
async fn callback(
    state: &AppState,
    provider: &str,
    headers: &HeaderMap,
    query: Option<&str>,
) -> Response {
    let Some(runtime) = state.oauth.as_ref().filter(|r| r.offers(provider)) else {
        return not_found();
    };
    let home = ReturnTarget::home(&runtime.return_origins);

    match complete(state, runtime, provider, headers, query).await {
        Ok((target, session_cookie)) => {
            redirect(&target.url(), &[session_cookie, glue::clear_cookie()])
        }
        Err(error) => {
            tracing::warn!(reason = %error.reason(), "google sign-in did not complete");
            let location = match error {
                CallbackError::UnverifiedLink => {
                    format!("{}/login?error=unverified_link", home.origin)
                }
                _ => home.url(),
            };
            redirect(&location, &[glue::clear_cookie()])
        }
    }
}

async fn complete(
    state: &AppState,
    runtime: &OAuthRuntime,
    provider: &str,
    headers: &HeaderMap,
    query: Option<&str>,
) -> Result<(ReturnTarget, String), CallbackError> {
    let cookies = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let sealed = glue::pending_cookie(cookies).map_err(CallbackError::Refused)?;
    let now = Utc::now().timestamp();
    let pending = glue::open(runtime.pending_secret.as_bytes(), sealed, now)
        .map_err(CallbackError::Refused)?;

    if pending.provider != provider {
        return Err(CallbackError::Refused(Refusal::ProviderMismatch));
    }

    let params: HashMap<String, String> =
        url::form_urlencoded::parse(query.unwrap_or("").as_bytes())
            .into_owned()
            .collect();
    if params.get("state") != Some(&pending.state) {
        return Err(CallbackError::Refused(Refusal::StateMismatch));
    }
    // No `code` means the user declined consent, or the provider failed.
    let code = params
        .get("code")
        .ok_or(CallbackError::Refused(Refusal::Missing))?;

    if !runtime.claim_local(&pending.state, pending.expires_at, now)
        || !claim_shared(state, &pending.state, now).await?
    {
        return Err(CallbackError::Refused(Refusal::AlreadyUsed));
    }

    let target = match glue::admit(
        &runtime.return_origins,
        Some(&pending.origin),
        Some(&pending.path),
    ) {
        Ok(target) => target,
        Err(reason) => {
            tracing::warn!(
                ?reason,
                "sealed return target refused at callback; using the app home"
            );
            ReturnTarget::home(&runtime.return_origins)
        }
    };

    // Taken before the library runs. Google rows created at or after this instant are
    // this callback's; rows from earlier sign-ins are never undone (see `refuse_link`).
    let started = Utc::now();
    let response = library(
        state,
        HttpMethod::Get,
        &format!("/callback/{provider}"),
        HashMap::new(),
        HashMap::from([
            ("code".to_string(), code.clone()),
            ("state".to_string(), pending.state.clone()),
        ]),
        None,
    )
    .await
    .map_err(CallbackError::Provider)?;
    if response.status != 200 {
        refuse_residue(state, &response.body, started).await;
        return Err(CallbackError::Provider(format!(
            "better-auth answered {}",
            response.status
        )));
    }

    // The body carries the session token. It is dropped here; the browser gets
    // the token only as the HttpOnly cookie.
    let Some(session_cookie) = response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.clone())
    else {
        refuse_residue(state, &response.body, started).await;
        return Err(CallbackError::NoSessionCookie);
    };
    let session_cookie = settle_email_match(state, &response.body, session_cookie, started).await?;
    Ok((target, session_cookie))
}

fn provider_error(error: impl std::fmt::Display) -> CallbackError {
    CallbackError::Provider(error.to_string())
}

/// The first callback for `oauth_state` across every instance sharing the store.
/// The claim is an append at expected version 0, so Core accepts exactly one.
async fn claim_shared(
    state: &AppState,
    oauth_state: &str,
    now: i64,
) -> Result<bool, CallbackError> {
    if state.faults.claims_down.load(Ordering::Relaxed) {
        return Err(CallbackError::Refused(Refusal::ClaimUnavailable));
    }
    let outcome = match state.coordination {
        Coordination::Local => return Ok(true),
        Coordination::Shared => {
            let input = IngestEventInput::new(
                "auth.oauth.claimed".to_string(),
                format!("oauth-claim:{oauth_state}"),
                json!({ "claimed_at": now }),
            )
            .with_expected_version(0);
            state.allsource.core.ingest_event(input).await.map(|_| ())
        }
    };
    claim_outcome(outcome)
        .inspect_err(|refusal| tracing::error!(?refusal, "single-use claim not granted"))
        .map_err(CallbackError::Refused)
}

/// Maps the store's answer to the claim. `Ok(true)`: first writer. `Ok(false)`: a
/// second use, which Core reports as a version conflict. Anything else is refused as
/// unavailable. This does not retry. The SDK retries transport errors and 5xx before
/// it returns, so a retry whose first attempt had committed reads as a conflict and
/// the callback is refused as a second use. The code is never exchanged twice.
fn claim_outcome(result: Result<(), SdkError>) -> Result<bool, Refusal> {
    match result {
        Ok(()) => Ok(true),
        Err(SdkError::VersionConflict { .. }) => Ok(false),
        Err(_) => Err(Refusal::ClaimUnavailable),
    }
}

/// Appends an audit record. Shared coordination writes it to the store; local
/// coordination logs it.
async fn record_audit(
    state: &AppState,
    event_type: &str,
    user_id: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    if state.faults.audit_down.load(Ordering::Relaxed) {
        return Err("audit write failed (injected)".to_string());
    }
    match state.coordination {
        Coordination::Shared => state
            .allsource
            .core
            .ingest_event(IngestEventInput::new(
                event_type.to_string(),
                format!("audit-auth:{user_id}"),
                payload,
            ))
            .await
            .map(|_| ())
            .map_err(|error| error.to_string()),
        Coordination::Local => {
            tracing::info!(event_type, user_id, %payload, "audit record (local coordination)");
            Ok(())
        }
    }
}

/// The user id and session token the library named in its callback body.
fn callback_identity(body: &[u8]) -> (Option<String>, Option<String>) {
    let Ok(reply) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (None, None);
    };
    let user_id = reply
        .pointer("/user/id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let token = reply
        .get("token")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    (user_id, token)
}

/// The library links a Google identity to an existing account with the same email
/// and does not check that account's verification. Settled here, after the library
/// has answered. Both sides must have proved the email: Google reported
/// `email_verified` (the library's gate, `verified_google_user`), and the existing
/// account's `email_verified` is true. Only then is the session kept, and an audit
/// record `auth.google.signed_in` is written for every accepted Google sign-in.
/// Every refusal path runs `refuse_link` once the user id is known.
async fn settle_email_match(
    state: &AppState,
    body: &[u8],
    library_cookie: String,
    started: DateTime<Utc>,
) -> Result<String, CallbackError> {
    let (user_id, token) = callback_identity(body);
    let user_id =
        user_id.ok_or_else(|| CallbackError::Provider("callback body has no user id".into()))?;

    let user = match state.auth.database().get_user_by_id(&user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            refuse_link(state, &user_id, token.as_deref(), started, false).await;
            return Err(CallbackError::Provider("linked user missing".into()));
        }
        Err(error) => {
            refuse_link(state, &user_id, token.as_deref(), started, false).await;
            return Err(provider_error(error));
        }
    };

    if state.faults.settle_down.load(Ordering::Relaxed) {
        let all = !user.email_verified();
        refuse_link(state, &user_id, token.as_deref(), started, all).await;
        return Err(CallbackError::Provider("settle fault (injected)".into()));
    }
    if !user.email_verified() {
        refuse_link(state, &user_id, token.as_deref(), started, true).await;
        return Err(CallbackError::UnverifiedLink);
    }
    if record_audit(
        state,
        "auth.google.signed_in",
        &user_id,
        json!({ "provider": "google" }),
    )
    .await
    .is_err()
    {
        refuse_link(state, &user_id, token.as_deref(), started, false).await;
        return Err(CallbackError::Refused(Refusal::AuditUnavailable));
    }
    Ok(library_cookie)
}

/// Undoes what the library wrote for a callback that did not complete, when the body
/// names the user. Used on every error path after the library has answered.
async fn refuse_residue(state: &AppState, body: &[u8], started: DateTime<Utc>) {
    let (user_id, token) = callback_identity(body);
    let Some(user_id) = user_id else {
        return;
    };
    let unverified = matches!(
        state.auth.database().get_user_by_id(&user_id).await,
        Ok(Some(user)) if !user.email_verified()
    );
    refuse_link(state, &user_id, token.as_deref(), started, unverified).await;
}

/// Reverses a refused callback's writes: the user's Google account rows and the
/// session the callback issued. Never the user's row or other sessions.
///
/// Which Google rows: when `all_google_rows` (the account is unverified), every Google
/// row goes, because no legitimate link can sit on an unverified account. Otherwise
/// only rows created at or after `started`, which this callback wrote. A link from an
/// earlier, verified sign-in is never touched, even when the audit of this callback fails.
async fn refuse_link(
    state: &AppState,
    user_id: &str,
    token: Option<&str>,
    started: DateTime<Utc>,
    all_google_rows: bool,
) {
    let db = state.auth.database();
    match db.get_user_accounts(user_id).await {
        Ok(accounts) => {
            for account in accounts.iter().filter(|account| {
                account.provider_id() == "google"
                    && (all_google_rows || account.created_at() >= started)
            }) {
                if let Err(error) = db.delete_account(account.id()).await {
                    tracing::error!(%error, "refused link: Google account not removed");
                }
            }
        }
        Err(error) => tracing::error!(%error, "refused link: accounts not read"),
    }
    if let Some(token) = token
        && let Err(error) = db.delete_session(token).await
    {
        tracing::error!(%error, "refused link: session not deleted");
    }
}

/// The one place a request reaches better-auth in-process. Its disabled-path and
/// plugin checks still run here, which is why the OAuth routes are not disabled
/// by path: that would refuse this call too.
async fn library(
    state: &AppState,
    method: HttpMethod,
    path: &str,
    headers: HashMap<String, String>,
    query: HashMap<String, String>,
    body: Option<Vec<u8>>,
) -> Result<AuthResponse, String> {
    let request =
        better_auth::AuthRequest::from_parts(method, path.to_string(), headers, body, query);
    state
        .auth
        .handle_request(request)
        .await
        .map_err(|error| error.to_string())
}

/// Mounted over the whole router, after the `/auth` nest, so it sees the
/// requests better-auth would otherwise answer.
pub async fn gate(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().clone();

    if let Some(provider) = path.strip_prefix("/auth/callback/") {
        // axum routes HEAD to better-auth's GET callback, so HEAD is refused here.
        // No provider redirect uses HEAD, and a HEAD that succeeded would spend the code.
        if method != Method::GET {
            return not_found();
        }
        if !rate_limit::within_limit(&state, &request) {
            return ApiError::RateLimited.into_response();
        }
        let provider = provider.to_string();
        let headers = request.headers().clone();
        let query = request.uri().query().map(str::to_string);
        return callback(&state, &provider, &headers, query.as_deref()).await;
    }

    if path.starts_with("/auth/oauth/") && path.ends_with("/start") {
        if !rate_limit::within_limit(&state, &request) {
            return ApiError::RateLimited.into_response();
        }
        return next.run(request).await;
    }

    // A trailing slash must not reach the library under a different spelling.
    let canonical = path.trim_end_matches('/');
    if matches!(canonical, "/auth/sign-in/social" | "/auth/link-social") {
        return not_found();
    }

    if method == Method::POST && matches!(canonical, "/auth/sign-up/email" | "/auth/sign-in/email")
    {
        return match normalise_credentials(request).await {
            Ok(request) => next.run(request).await,
            Err(response) => response,
        };
    }

    next.run(request).await
}

/// Lower-cases the `email` field of a credential body, so a password account is
/// stored and found under the same form the Google identity uses. A body that
/// is not a JSON object passes through unchanged and the library refuses it.
async fn normalise_credentials(request: Request) -> Result<Request, Response> {
    let (mut parts, body) = request.into_parts();
    let bytes = axum::body::to_bytes(body, MAX_CREDENTIAL_BODY)
        .await
        .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE.into_response())?;

    let rewritten = serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()
        .and_then(|mut value| {
            let email = normalise_email(value.get("email")?.as_str()?);
            value["email"] = serde_json::Value::String(email);
            serde_json::to_vec(&value).ok()
        })
        .unwrap_or_else(|| bytes.to_vec());

    // The length changed, so the declared one is no longer true.
    parts.headers.remove(header::CONTENT_LENGTH);
    Ok(Request::from_parts(parts, Body::from(rewritten)))
}

/// A 302 with the auth response headers that must not be cached.
fn redirect(location: &str, set_cookies: &[String]) -> Response {
    let mut builder = Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, location)
        .header(header::CACHE_CONTROL, "no-store");
    for cookie in set_cookies {
        builder = builder.header(header::SET_COOKIE, cookie);
    }
    builder
        .body(Body::empty())
        .unwrap_or_else(|_| internal_error())
}

fn not_found() -> Response {
    StatusCode::NOT_FOUND.into_response()
}

fn internal_error() -> Response {
    StatusCode::INTERNAL_SERVER_ERROR.into_response()
}

#[cfg(test)]
mod tests {
    use allsource::Error as SdkError;

    use super::*;

    #[test]
    fn the_first_writer_claims_and_a_second_use_is_a_conflict() {
        assert_eq!(claim_outcome(Ok(())), Ok(true));
        assert_eq!(
            claim_outcome(Err(SdkError::VersionConflict {
                expected: 0,
                current: 1,
            })),
            Ok(false)
        );
    }

    /// Any other failure, transport or server, is refused as unavailable. Never a
    /// claim, so the code is not exchanged on an outage.
    #[test]
    fn any_other_store_failure_is_refused_as_unavailable() {
        let outage = SdkError::Api {
            status: 503,
            message: "unavailable".into(),
            body: None,
        };
        assert_eq!(claim_outcome(Err(outage)), Err(Refusal::ClaimUnavailable));
    }

    #[test]
    fn the_callback_body_names_the_user_and_the_session() {
        let body = br#"{"token":"t1","user":{"id":"u1"}}"#;
        assert_eq!(
            callback_identity(body),
            (Some("u1".to_string()), Some("t1".to_string()))
        );
        assert_eq!(callback_identity(b"{\"message\":\"x\"}"), (None, None));
        assert_eq!(callback_identity(b"not json"), (None, None));
    }
}
