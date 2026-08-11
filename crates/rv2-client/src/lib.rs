//! Typed HTTP client for `apps/api`.
//!
//! **Layer 2, WASM-safe.** One function per endpoint, all sharing a base URL.
//!
//! ## Why `gloo-net` and not `reqwest` (D16)
//!
//! `gloo-net` is a thin `fetch` wrapper — materially smaller in the bundle —
//! and it is the `wasm32` path getformlab's frontends already run in
//! production. `reqwest`'s wasm backend drags a much larger tree in.
//!
//! ## Why there is no token here (D17)
//!
//! The session is an **HttpOnly cookie** issued by better-auth. The WASM client
//! never sees it, which removes the entire XSS-token-theft class. Every request
//! therefore sets [`RequestCredentials::Include`] so the browser attaches the
//! cookie; `apps/api` must answer with `Access-Control-Allow-Credentials: true`
//! and an explicit origin (never `*`, which is illegal with credentials).
//!
//! ## Cost accepted (§6.1)
//!
//! Because the architecture rejects Dioxus server functions, this layer is
//! hand-written rather than generated: roughly one function per endpoint. That
//! is the price of the clean WASM boundary.

#![forbid(unsafe_code)]

use gloo_net::http::{Request, RequestBuilder};
use rv2_api_types::{
    CreatePostRequest, ErrorResponse, PostView, SessionView, UpdatePostRequest,
    UpdateProfileRequest, UserView,
};
use uuid::Uuid;
use web_sys::RequestCredentials;

/// Where `apps/api` lives, baked in at compile time.
///
/// A WASM bundle has no environment to read at runtime, so this is an
/// `option_env!` resolved by the build. The default matches the port allocation
/// carried over from rust-v1 (api 4400, web 4401, app 4402).
#[must_use]
pub fn api_base() -> &'static str {
    option_env!("PUBLIC_API_URL").unwrap_or("http://localhost:4400")
}

/// Anything that can go wrong talking to `apps/api`.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// The request never completed — offline, DNS, CORS preflight rejection.
    #[error("network error: {0}")]
    Network(String),
    /// The server answered with a non-2xx status and a parseable body.
    #[error("{code}: {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },
    /// The server answered but the body was not what the contract promised.
    #[error("could not decode response: {0}")]
    Decode(String),
}

impl ApiError {
    /// Whether this error means "not signed in".
    ///
    /// `apps/app`'s shell uses it to decide between "redirect to /login" and
    /// "show an error", so it lives here rather than being re-derived from a
    /// status code at each call site.
    #[must_use]
    pub const fn is_unauthenticated(&self) -> bool {
        matches!(self, ApiError::Api { status: 401, .. })
    }
}

impl From<gloo_net::Error> for ApiError {
    fn from(error: gloo_net::Error) -> Self {
        ApiError::Network(error.to_string())
    }
}

/// Attach the session cookie to a request (D17).
fn authed(builder: RequestBuilder) -> RequestBuilder {
    builder.credentials(RequestCredentials::Include)
}

/// Send a request and decode the response, mapping errors uniformly.
async fn send<T: serde::de::DeserializeOwned>(
    request: gloo_net::http::Request,
) -> Result<T, ApiError> {
    let response = request.send().await?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| ApiError::Decode(e.to_string()))?;

    if !(200..300).contains(&status) {
        // Every non-2xx from `apps/api` is an `ErrorResponse`. If it is not,
        // surface the raw body rather than inventing a code — a proxy or a
        // panic-in-middleware produces HTML, and hiding that makes it much
        // harder to diagnose.
        let (code, message) = serde_json::from_str::<ErrorResponse>(&text).map_or_else(
            |_| ("unexpected_response".to_string(), text.clone()),
            |e| (e.code, e.message),
        );
        return Err(ApiError::Api {
            status,
            code,
            message,
        });
    }

    serde_json::from_str(&text).map_err(|e| ApiError::Decode(e.to_string()))
}

// ═════════════════════════════════════════════════════════════════════════════
// Auth — better-auth's own routes, mounted under `/auth` on apps/api
// ═════════════════════════════════════════════════════════════════════════════

/// `GET /me`. `Ok(None)` means "not signed in".
///
/// `apps/app` calls this once at mount inside a `use_resource`; `None` →
/// redirect to `/login`. That replaces rust-v1's Next.js `proxy.ts` middleware
/// redirect (§7 item 4).
///
/// ## Why `/me` and not better-auth's `/auth/get-session`
///
/// §5.3 names `GET /auth/get-session` as the bootstrap call, and that route
/// does exist and does work — but it answers with better-auth's **own**
/// envelope, `{"session": {...}, "user": {...}}`, with camelCase keys and its
/// `User` shape. Deserializing that into [`SessionView`] fails, and it fails as
/// a *decode* error rather than a 401, so the shell would neither show a
/// session nor redirect to login: the user would sit on a blank authenticated
/// page forever. Observed in a browser, not reasoned about.
///
/// `/me` is `apps/api`'s normalized projection of the same session — the same
/// principal the handlers see, in our own DTO, answering 401 when absent. That
/// keeps better-auth's response shape an implementation detail of the auth
/// router instead of part of the client contract.
///
/// # Errors
///
/// [`ApiError`] for anything other than a clean answer.
pub async fn get_session() -> Result<Option<SessionView>, ApiError> {
    let url = format!("{}/me", api_base());
    match send::<SessionView>(authed(Request::get(&url)).build()?).await {
        Ok(session) => Ok(Some(session)),
        Err(e) if e.is_unauthenticated() => Ok(None),
        Err(e) => Err(e),
    }
}

/// `POST /auth/sign-out`.
///
/// # Errors
///
/// [`ApiError`] if the request fails.
pub async fn sign_out() -> Result<(), ApiError> {
    let url = format!("{}/auth/sign-out", api_base());
    authed(Request::post(&url)).build()?.send().await?;
    Ok(())
}

// ═════════════════════════════════════════════════════════════════════════════
// Posts
// ═════════════════════════════════════════════════════════════════════════════

/// `GET /posts` — served from the `posts_v1` projection worker.
///
/// # Errors
///
/// [`ApiError`] if the request fails or the body does not decode.
pub async fn list_posts() -> Result<Vec<PostView>, ApiError> {
    let url = format!("{}/posts", api_base());
    send(authed(Request::get(&url)).build()?).await
}

/// `GET /posts/{id}` — folded on read.
///
/// # Errors
///
/// [`ApiError`]; a missing post surfaces as `Api { status: 404, .. }`.
pub async fn get_post(id: Uuid) -> Result<PostView, ApiError> {
    let url = format!("{}/posts/{id}", api_base());
    send(authed(Request::get(&url)).build()?).await
}

/// `POST /posts`.
///
/// # Errors
///
/// [`ApiError`]; validation failures surface as `Api { status: 422, .. }`.
pub async fn create_post(req: &CreatePostRequest) -> Result<PostView, ApiError> {
    let url = format!("{}/posts", api_base());
    send(authed(Request::post(&url)).json(req)?).await
}

/// `PATCH /posts/{id}`.
///
/// # Errors
///
/// [`ApiError`]; a non-author gets `Api { status: 403, .. }`.
pub async fn update_post(id: Uuid, req: &UpdatePostRequest) -> Result<PostView, ApiError> {
    let url = format!("{}/posts/{id}", api_base());
    send(authed(Request::patch(&url)).json(req)?).await
}

/// `DELETE /posts/{id}`.
///
/// # Errors
///
/// [`ApiError`]; a non-author gets `Api { status: 403, .. }`.
pub async fn delete_post(id: Uuid) -> Result<(), ApiError> {
    let url = format!("{}/posts/{id}", api_base());
    let response = authed(Request::delete(&url)).build()?.send().await?;
    let status = response.status();
    if !(200..300).contains(&status) {
        let text = response.text().await.unwrap_or_default();
        let (code, message) = serde_json::from_str::<ErrorResponse>(&text).map_or_else(
            |_| ("unexpected_response".to_string(), text.clone()),
            |e| (e.code, e.message),
        );
        return Err(ApiError::Api {
            status,
            code,
            message,
        });
    }
    Ok(())
}

// ═════════════════════════════════════════════════════════════════════════════
// Users
// ═════════════════════════════════════════════════════════════════════════════

/// `GET /users/{id}` — folded on read.
///
/// # Errors
///
/// [`ApiError`] if the request fails.
pub async fn get_user(id: Uuid) -> Result<UserView, ApiError> {
    let url = format!("{}/users/{id}", api_base());
    send(authed(Request::get(&url)).build()?).await
}

/// `PATCH /users/{id}` — replaces Supabase's `updateUser` (§7 item 8).
///
/// # Errors
///
/// [`ApiError`] if the request fails.
pub async fn update_profile(id: Uuid, req: &UpdateProfileRequest) -> Result<UserView, ApiError> {
    let url = format!("{}/users/{id}", api_base());
    send(authed(Request::patch(&url)).json(req)?).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_url_has_no_trailing_slash() {
        // Every call site does `format!("{}/posts", api_base())`; a trailing
        // slash would produce `//posts`.
        assert!(!api_base().ends_with('/'));
    }

    #[test]
    fn unauthenticated_is_recognised_only_for_401() {
        let unauth = ApiError::Api {
            status: 401,
            code: "unauthorized".into(),
            message: String::new(),
        };
        let forbidden = ApiError::Api {
            status: 403,
            code: "forbidden".into(),
            message: String::new(),
        };
        assert!(unauth.is_unauthenticated());
        assert!(
            !forbidden.is_unauthenticated(),
            "403 means signed in but not allowed — redirecting to /login would loop"
        );
        assert!(!ApiError::Network("offline".into()).is_unauthenticated());
    }
}
