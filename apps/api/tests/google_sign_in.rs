//! Google sign-in end to end, against an in-process stub OIDC provider.
//!
//! The stub stands where Google's authorise, token and userinfo endpoints stand
//! in production: `ServerConfig.google_oauth.endpoints` points at it. It checks
//! the client secret and the PKCE verifier against the challenge the library
//! sent. No network leaves the machine and no Google credential is needed.
//!
//! Runs on the memory adapter (the default feature set), so no AllSource Core
//! is required. The design's matrix is in
//! `docs/plans/2026-11-26-google-sign-in-design.md`, section 6.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use api::build_router;
use api::infrastructure::auth::oauth_glue::{self as glue, Pending};
use api::infrastructure::state::{AppState, Coordination, build_state_with};
use axum::Json;
use axum::extract::{Form, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::routing::{get, post};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use std::sync::atomic::Ordering;

use better_auth::{AccountOps, AuthAccount, AuthUser, UpdateUser, UserOps};
use rv2_shared::{GoogleOAuthConfig, OAuthEndpoints, OAuthFlowConfig, ServerConfig};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CLIENT_SECRET: &str = "test-client-secret";
const PENDING_SECRET: &str = "test-pending-secret-that-is-at-least-32-bytes";
const PRIMARY: &str = "http://localhost:4402";
const SECONDARY: &str = "http://localhost:4401";

#[derive(Clone)]
struct Grant {
    challenge: String,
    user: Value,
}

#[derive(Default)]
struct StubState {
    grants: Mutex<HashMap<String, Grant>>,
    tokens: Mutex<HashMap<String, Value>>,
}

#[derive(serde::Deserialize)]
struct TokenForm {
    code: String,
    code_verifier: String,
    client_secret: String,
}

async fn stub_token(
    State(stub): State<Arc<StubState>>,
    Form(form): Form<TokenForm>,
) -> (StatusCode, Json<Value>) {
    let grant = stub.grants.lock().unwrap().get(&form.code).cloned();
    let Some(grant) = grant else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_grant"})),
        );
    };
    let verifier_matches =
        URL_SAFE_NO_PAD.encode(Sha256::digest(form.code_verifier.as_bytes())) == grant.challenge;
    if form.client_secret != CLIENT_SECRET || !verifier_matches {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_client"})),
        );
    }
    let token = format!("at-{}", form.code);
    stub.tokens
        .lock()
        .unwrap()
        .insert(token.clone(), grant.user.clone());
    (
        StatusCode::OK,
        Json(json!({"access_token": token, "token_type": "Bearer", "expires_in": 3600})),
    )
}

async fn stub_userinfo(
    State(stub): State<Arc<StubState>>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string);
    let user = token.and_then(|t| stub.tokens.lock().unwrap().get(&t).cloned());
    match user {
        Some(user) => (StatusCode::OK, Json(user)),
        None => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "invalid_token"})),
        ),
    }
}

async fn stub_authorize() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn spawn_stub() -> (String, Arc<StubState>) {
    let stub = Arc::new(StubState::default());
    let app = axum::Router::new()
        .route("/authorize", get(stub_authorize))
        .route("/token", post(stub_token))
        .route("/userinfo", get(stub_userinfo))
        .with_state(Arc::clone(&stub));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}"), stub)
}

struct Harness {
    base: String,
    stub_url: String,
    stub: Arc<StubState>,
    state: Arc<AppState>,
    client: reqwest::Client,
}

async fn harness() -> Harness {
    let (stub_url, stub) = spawn_stub().await;
    harness_with(Coordination::Local, "http://localhost:3900", stub_url, stub).await
}

/// One API instance. `core` is the AllSource Core the instance uses; `stub_url`
/// and `stub` are the OIDC provider it talks to, which several instances may share.
async fn harness_with(
    coordination: Coordination,
    core: &str,
    stub_url: String,
    stub: Arc<StubState>,
) -> Harness {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");

    let origins = vec![PRIMARY.to_string(), SECONDARY.to_string()];
    let config = ServerConfig {
        bind_addr: addr.to_string(),
        allsource_core_url: core.to_string(),
        allsource_query_url: core.to_string(),
        allsource_api_key: "ask_test".into(),
        jwt_secret: "test-secret-key-that-is-at-least-32-characters-long".into(),
        auth_base_url: format!("{base}/auth"),
        cors_origins: origins.clone(),
        trusted_proxy_hops: 0,
        google_oauth: Some(GoogleOAuthConfig {
            client_id: "test-client".into(),
            client_secret: CLIENT_SECRET.into(),
            endpoints: OAuthEndpoints {
                auth_url: format!("{stub_url}/authorize"),
                token_url: format!("{stub_url}/token"),
                user_info_url: format!("{stub_url}/userinfo"),
            },
        }),
        oauth_flow: Some(OAuthFlowConfig {
            pending_secret: PENDING_SECRET.into(),
            return_origins: origins,
        }),
    };
    let state = Arc::new(
        build_state_with(&config, coordination)
            .await
            .expect("state builds"),
    );
    let app = build_router(Arc::clone(&state));
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    Harness {
        base,
        stub_url,
        stub,
        state,
        client,
    }
}

/// A `Set-Cookie` value for the named cookie, if one was set and not cleared.
fn cookie_pair(resp: &reqwest::Response, name: &str) -> Option<String> {
    resp.headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|raw| {
            let pair = raw.split(';').next()?;
            let (key, value) = pair.split_once('=')?;
            (key == name && !value.is_empty()).then(|| pair.to_string())
        })
        .next()
}

/// The better-auth session cookie: any cookie set other than the pending one.
fn session_cookie(resp: &reqwest::Response) -> Option<String> {
    resp.headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|raw| {
            let pair = raw.split(';').next()?;
            let (key, value) = pair.split_once('=')?;
            (!key.is_empty() && key != glue::PENDING_COOKIE && !value.is_empty())
                .then(|| pair.to_string())
        })
        .next()
}

fn location(resp: &reqwest::Response) -> String {
    resp.headers()
        .get(header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("<no location>")
        .to_string()
}

struct Started {
    state: String,
    challenge: String,
    pending_cookie: String,
}

/// Start a flow and return what the browser would hold after the redirect.
async fn start(h: &Harness, return_to: Option<&str>, app: Option<&str>) -> Started {
    let mut query: Vec<(&str, &str)> = Vec::new();
    if let Some(r) = return_to {
        query.push(("return_to", r));
    }
    if let Some(a) = app {
        query.push(("app", a));
    }
    let resp = h
        .client
        .get(format!("{}/auth/oauth/google/start", h.base))
        .query(&query)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FOUND, "start refused");
    let url = url::Url::parse(&location(&resp)).unwrap();
    assert!(
        url.as_str().starts_with(&h.stub_url),
        "start redirected to {url}"
    );
    let pairs: HashMap<String, String> = url.query_pairs().into_owned().collect();
    let pending_cookie = cookie_pair(&resp, glue::PENDING_COOKIE).expect("pending cookie set");
    Started {
        state: pairs["state"].clone(),
        challenge: pairs["code_challenge"].clone(),
        pending_cookie,
    }
}

/// Make the stub accept `code` for this flow's challenge and user.
fn grant(h: &Harness, code: &str, started: &Started, user: Value) {
    h.stub.grants.lock().unwrap().insert(
        code.to_string(),
        Grant {
            challenge: started.challenge.clone(),
            user,
        },
    );
}

fn verified_user(email: &str) -> Value {
    json!({"sub": format!("sub-{email}"), "email": email, "email_verified": true, "name": "Test"})
}

/// Hit the callback as the browser does after Google redirects back.
async fn callback(
    h: &Harness,
    code: Option<&str>,
    state: &str,
    cookie: Option<&str>,
) -> reqwest::Response {
    let mut query = vec![("state", state.to_string())];
    if let Some(c) = code {
        query.push(("code", c.to_string()));
    }
    let mut req = h
        .client
        .get(format!("{}/auth/callback/google", h.base))
        .query(&query);
    if let Some(c) = cookie {
        req = req.header(header::COOKIE, c);
    }
    req.send().await.unwrap()
}

async fn me_status(h: &Harness, session: &str) -> StatusCode {
    h.client
        .get(format!("{}/me", h.base))
        .header(header::COOKIE, session)
        .send()
        .await
        .unwrap()
        .status()
}

async fn user_by_email(h: &Harness, email: &str) -> Option<String> {
    h.state
        .auth
        .database()
        .get_user_by_email(email)
        .await
        .unwrap()
        .map(|u| u.id().to_string())
}

async fn password_signin_succeeds(h: &Harness, email: &str, password: &str) -> bool {
    h.client
        .post(format!("{}/auth/sign-in/email", h.base))
        .json(&json!({"email": email, "password": password}))
        .send()
        .await
        .unwrap()
        .status()
        .is_success()
}

/// The ids of every account row of a user, in the store's order.
async fn account_ids(h: &Harness, user_id: &str) -> Vec<String> {
    h.state
        .auth
        .database()
        .get_user_accounts(user_id)
        .await
        .unwrap()
        .iter()
        .map(|account| account.id().to_string())
        .collect()
}

/// The whole user row, for before and after comparisons.
async fn user_row(h: &Harness, user_id: &str) -> Value {
    let user = h
        .state
        .auth
        .database()
        .get_user_by_id(user_id)
        .await
        .unwrap()
        .expect("user row");
    serde_json::to_value(user).unwrap()
}

async fn verify_email(h: &Harness, user_id: &str) {
    h.state
        .auth
        .database()
        .update_user(
            user_id,
            UpdateUser {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();
}

async fn register_password(h: &Harness, email: &str) -> reqwest::Response {
    h.client
        .post(format!("{}/auth/sign-up/email", h.base))
        .json(&json!({"email": email, "password": "password123", "name": "Pre"}))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn a_google_sign_in_lands_on_the_cleared_target_with_a_session() {
    let h = harness().await;
    let started = start(&h, Some("/posts?tab=1"), Some(PRIMARY)).await;
    grant(&h, "c1", &started, verified_user("new@example.com"));

    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::FOUND);
    assert_eq!(location(&resp), format!("{PRIMARY}/posts?tab=1"));
    let session = session_cookie(&resp).expect("session cookie set");
    assert_eq!(me_status(&h, &session).await, StatusCode::OK);
    assert!(user_by_email(&h, "new@example.com").await.is_some());
}

#[tokio::test]
async fn the_root_path_is_admitted_on_the_primary_origin() {
    let h = harness().await;
    let started = start(&h, Some("/"), None).await;
    grant(&h, "c1", &started, verified_user("root@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(session_cookie(&resp).is_some());
}

#[tokio::test]
async fn a_target_on_the_second_origin_is_admitted() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), Some(SECONDARY)).await;
    grant(&h, "c1", &started, verified_user("second@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert_eq!(location(&resp), format!("{SECONDARY}/posts"));
}

#[tokio::test]
async fn an_encoded_query_value_is_admitted_as_written() {
    let h = harness().await;
    let started = start(&h, Some("/posts?q=hello%20world"), None).await;
    grant(&h, "c1", &started, verified_user("enc@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert_eq!(location(&resp), format!("{PRIMARY}/posts?q=hello%20world"));
    assert!(session_cookie(&resp).is_some());
}

/// Double encoding and an encoded `//` in the path are refused: the target falls
/// back to home, and the sign-in itself still completes.
#[tokio::test]
async fn double_encoded_and_encoded_authority_targets_fall_back_to_home() {
    let h = harness().await;
    for (i, target) in ["/posts?q=%252F", "/%2F%2Fevil.example"].iter().enumerate() {
        let started = start(&h, Some(*target), None).await;
        let code = format!("enc{i}");
        grant(
            &h,
            &code,
            &started,
            verified_user(&format!("enc{i}@example.com")),
        );
        let resp = callback(
            &h,
            Some(&code),
            &started.state,
            Some(&started.pending_cookie),
        )
        .await;
        assert_eq!(location(&resp), format!("{PRIMARY}/"), "target {target}");
    }
}

#[tokio::test]
async fn tainted_return_targets_fall_back_to_the_home_page() {
    let h = harness().await;
    let targets = [
        "https://evil.example/x",
        "//evil.example",
        "/\\evil.example",
        "/%2F%2Fevil.example",
        "/%5Cevil.example",
    ];
    for (i, target) in targets.iter().enumerate() {
        let started = start(&h, Some(target), None).await;
        let code = format!("t{i}");
        grant(
            &h,
            &code,
            &started,
            verified_user(&format!("taint{i}@example.com")),
        );
        let resp = callback(
            &h,
            Some(&code),
            &started.state,
            Some(&started.pending_cookie),
        )
        .await;
        assert_eq!(location(&resp), format!("{PRIMARY}/"), "target {target}");
        assert!(
            session_cookie(&resp).is_some(),
            "sign-in itself still completes for {target}"
        );
    }
}

#[tokio::test]
async fn a_callback_without_the_pending_cookie_creates_no_session() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("nocookie@example.com"));
    let resp = callback(&h, Some("c1"), &started.state, None).await;
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(session_cookie(&resp).is_none());
    assert!(user_by_email(&h, "nocookie@example.com").await.is_none());
}

/// HEAD is refused with or without the cookie, and a refused HEAD spends nothing:
/// the state still completes a later GET.
#[tokio::test]
async fn a_head_callback_is_not_found_and_spends_nothing() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("head@example.com"));

    for cookie in [None, Some(started.pending_cookie.as_str())] {
        let mut req = h
            .client
            .head(format!("{}/auth/callback/google", h.base))
            .query(&[("code", "c1"), ("state", started.state.as_str())]);
        if let Some(c) = cookie {
            req = req.header(header::COOKIE, c);
        }
        let resp = req.send().await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "cookie: {}",
            cookie.is_some()
        );
        assert!(session_cookie(&resp).is_none());
    }
    assert!(user_by_email(&h, "head@example.com").await.is_none());

    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert_eq!(location(&resp), format!("{PRIMARY}/posts"));
    assert!(session_cookie(&resp).is_some());
}

/// The login-CSRF shape: the attacker's valid cookie, planted in front of the
/// victim's real one, with the attacker's own code and state.
#[tokio::test]
async fn a_planted_second_pending_cookie_is_refused() {
    let h = harness().await;
    let victim = start(&h, Some("/posts"), None).await;
    let attacker = start(&h, Some("/posts"), None).await;
    grant(
        &h,
        "attacker-code",
        &attacker,
        verified_user("planted@example.com"),
    );

    let header = format!("{}; {}", attacker.pending_cookie, victim.pending_cookie);
    let resp = callback(&h, Some("attacker-code"), &attacker.state, Some(&header)).await;
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(session_cookie(&resp).is_none());
    assert!(user_by_email(&h, "planted@example.com").await.is_none());
}

#[tokio::test]
async fn the_pending_cookie_is_host_prefixed_and_hardened() {
    let h = harness().await;
    let resp = h
        .client
        .get(format!("{}/auth/oauth/google/start", h.base))
        .send()
        .await
        .unwrap();
    let raw = resp
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("__Host-rv2_oauth_pending="))
        .expect("host-prefixed pending cookie is set")
        .to_string();
    let attributes: Vec<&str> = raw.split(';').map(str::trim).collect();
    for wanted in ["Path=/", "Secure", "HttpOnly", "SameSite=Lax"] {
        assert!(attributes.contains(&wanted), "{raw}");
    }
    assert!(
        attributes
            .iter()
            .all(|a| !a.to_ascii_lowercase().starts_with("domain=")),
        "{raw}"
    );
}

#[tokio::test]
async fn other_methods_on_the_oauth_paths_are_not_found() {
    let h = harness().await;
    let cases = [
        (reqwest::Method::POST, "/auth/callback/google"),
        (reqwest::Method::PUT, "/auth/callback/google"),
        (reqwest::Method::DELETE, "/auth/callback/google"),
        (reqwest::Method::HEAD, "/auth/callback/google"),
        (reqwest::Method::POST, "/auth/sign-in/social"),
        (reqwest::Method::GET, "/auth/sign-in/social"),
        (reqwest::Method::HEAD, "/auth/sign-in/social"),
        (reqwest::Method::POST, "/auth/link-social"),
        (reqwest::Method::HEAD, "/auth/link-social"),
        (reqwest::Method::POST, "/auth/refresh-token"),
        (reqwest::Method::POST, "/auth/get-access-token"),
    ];
    for (method, path) in cases {
        let resp = h
            .client
            .request(method.clone(), format!("{}{path}", h.base))
            .query(&[("code", "c1"), ("state", "s")])
            .json(&json!({"provider": "google"}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{method} {path}");
    }
}

#[tokio::test]
async fn a_replayed_cookie_after_success_creates_no_second_session() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("replay@example.com"));
    let first = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert!(session_cookie(&first).is_some());
    let second = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert_eq!(location(&second), format!("{PRIMARY}/"));
    assert!(session_cookie(&second).is_none());
}

#[tokio::test]
async fn concurrent_callbacks_with_one_cookie_admit_one_session() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("race@example.com"));
    let (a, b) = tokio::join!(
        callback(
            &h,
            Some("c1"),
            &started.state,
            Some(&started.pending_cookie)
        ),
        callback(
            &h,
            Some("c1"),
            &started.state,
            Some(&started.pending_cookie)
        ),
    );
    let sessions = [&a, &b]
        .iter()
        .filter(|r| session_cookie(r).is_some())
        .count();
    assert_eq!(sessions, 1);
}

#[tokio::test]
async fn a_state_that_does_not_match_the_cookie_is_refused() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("mismatch@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        "not-the-state",
        Some(&started.pending_cookie),
    )
    .await;
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(session_cookie(&resp).is_none());
}

#[tokio::test]
async fn a_tampered_cookie_is_refused() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("tamper@example.com"));
    let value = started
        .pending_cookie
        .split_once('=')
        .map(|(_, v)| v.to_string())
        .unwrap();
    let (payload, signature) = value.split_once('.').unwrap();
    let mut forged_sig = signature.to_string();
    let flipped = if forged_sig.ends_with('A') { 'B' } else { 'A' };
    forged_sig.pop();
    forged_sig.push(flipped);
    let cookie = format!("{}={payload}.{forged_sig}", glue::PENDING_COOKIE);
    let resp = callback(&h, Some("c1"), &started.state, Some(&cookie)).await;
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(session_cookie(&resp).is_none());
}

#[tokio::test]
async fn an_expired_cookie_is_refused() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("expired@example.com"));
    let now = chrono::Utc::now().timestamp();
    let target = glue::ReturnTarget {
        origin: PRIMARY.into(),
        path: "/posts".into(),
    };
    let mut pending = Pending::new("google", &started.state, &target, now);
    pending.expires_at = now - 1;
    let sealed = glue::seal(PENDING_SECRET.as_bytes(), &pending).unwrap();
    let cookie = format!("{}={sealed}", glue::PENDING_COOKIE);
    let resp = callback(&h, Some("c1"), &started.state, Some(&cookie)).await;
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(session_cookie(&resp).is_none());
}

#[tokio::test]
async fn a_cookie_naming_another_provider_is_refused() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("provider@example.com"));
    let target = glue::ReturnTarget {
        origin: PRIMARY.into(),
        path: "/posts".into(),
    };
    let now = chrono::Utc::now().timestamp();
    let pending = Pending::new("github", &started.state, &target, now);
    let sealed = glue::seal(PENDING_SECRET.as_bytes(), &pending).unwrap();
    let cookie = format!("{}={sealed}", glue::PENDING_COOKIE);
    let resp = callback(&h, Some("c1"), &started.state, Some(&cookie)).await;
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(session_cookie(&resp).is_none());
}

#[tokio::test]
async fn a_declined_consent_with_no_code_is_refused() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    let resp = callback(&h, None, &started.state, Some(&started.pending_cookie)).await;
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(session_cookie(&resp).is_none());
}

#[tokio::test]
async fn an_unverified_google_email_creates_no_user() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(
        &h,
        "c1",
        &started,
        json!({"sub": "s", "email": "unverified@example.com", "email_verified": false}),
    );
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert!(session_cookie(&resp).is_none());
    assert!(user_by_email(&h, "unverified@example.com").await.is_none());
}

/// Both sides must prove the email. A Google identity whose verified email matches an
/// unverified password account is refused: no session, the browser is sent to the
/// sign-in page with a message, and the user row and its accounts are byte-identical
/// before and after, including after a repeat sign-in with the same identity.
#[tokio::test]
async fn an_unverified_password_account_refuses_google_and_is_unchanged() {
    let h = harness().await;
    let signup = register_password(&h, "victim@example.com").await;
    assert!(signup.status().is_success());
    let own_session = session_cookie(&signup).expect("sign-up sets a session");
    let id = user_by_email(&h, "victim@example.com")
        .await
        .expect("registered");
    let row_before = user_row(&h, &id).await;
    let accounts_before = account_ids(&h, &id).await;

    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("victim@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert!(session_cookie(&resp).is_none(), "a session was issued");
    assert_eq!(
        location(&resp),
        format!("{PRIMARY}/login?error=unverified_link")
    );
    assert_eq!(user_row(&h, &id).await, row_before, "the user row changed");
    assert_eq!(
        account_ids(&h, &id).await,
        accounts_before,
        "a Google row survived"
    );

    let again = start(&h, Some("/posts"), None).await;
    grant(&h, "c2", &again, verified_user("victim@example.com"));
    let resp = callback(&h, Some("c2"), &again.state, Some(&again.pending_cookie)).await;
    assert!(
        session_cookie(&resp).is_none(),
        "a repeat sign-in was accepted"
    );
    assert_eq!(
        location(&resp),
        format!("{PRIMARY}/login?error=unverified_link")
    );
    assert_eq!(account_ids(&h, &id).await, accounts_before);

    assert_eq!(me_status(&h, &own_session).await, StatusCode::OK);
    assert!(password_signin_succeeds(&h, "victim@example.com", "password123").await);
}

/// Amendment 1: a verified password account keeps both credentials after the link.
#[tokio::test]
async fn a_verified_password_account_is_linked_and_keeps_its_password() {
    let h = harness().await;
    register_password(&h, "owner@example.com").await;
    let id = user_by_email(&h, "owner@example.com")
        .await
        .expect("registered");
    h.state
        .auth
        .database()
        .update_user(
            &id,
            UpdateUser {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("owner@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    let session = session_cookie(&resp).expect("the owner is signed in");
    assert_eq!(me_status(&h, &session).await, StatusCode::OK);
    assert!(password_signin_succeeds(&h, "owner@example.com", "password123").await);
    assert_eq!(user_by_email(&h, "owner@example.com").await, Some(id));
}

/// An unverified Google email never reaches the link, so the password account is
/// untouched: its session still works and its password still signs in.
#[tokio::test]
async fn an_unverified_google_email_leaves_a_password_account_alone() {
    let h = harness().await;
    let signup = register_password(&h, "unverified@example.com").await;
    let session = session_cookie(&signup).expect("sign-up sets a session");
    let before = user_by_email(&h, "unverified@example.com").await;

    let started = start(&h, Some("/posts"), None).await;
    grant(
        &h,
        "c1",
        &started,
        json!({"sub": "s", "email": "unverified@example.com", "email_verified": false}),
    );
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert!(session_cookie(&resp).is_none());
    assert_eq!(me_status(&h, &session).await, StatusCode::OK);
    assert!(password_signin_succeeds(&h, "unverified@example.com", "password123").await);
    assert_eq!(user_by_email(&h, "unverified@example.com").await, before);
}

/// Mixed case is matched by the normalised form: a verified owner whose address was
/// stored lower-cased links to the same user when Google reports it in another case.
#[tokio::test]
async fn a_mixed_case_verified_owner_links_the_same_user() {
    let h = harness().await;
    register_password(&h, "Mixed@Example.com").await;
    let before = user_by_email(&h, "mixed@example.com")
        .await
        .expect("stored lower-cased");
    let id = before.clone();
    h.state
        .auth
        .database()
        .update_user(
            &id,
            UpdateUser {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("Mixed@Example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    let session = session_cookie(&resp).expect("the owner is signed in");
    assert_eq!(me_status(&h, &session).await, StatusCode::OK);
    assert_eq!(user_by_email(&h, "mixed@example.com").await, Some(before));
    assert!(password_signin_succeeds(&h, "mixed@example.com", "password123").await);
}

/// Two API instances on one Core. The claim is shared, so of two callbacks with
/// the same cookie only one signs anyone in. Needs a live Core and the
/// `allsource-auth` feature, which keeps the auth state in the same store:
/// `ALLSOURCE_CORE_URL=... cargo test -p api --features allsource-auth \
///   --test google_sign_in two_instances -- --ignored`
#[tokio::test]
#[ignore = "requires a live AllSource Core: set ALLSOURCE_CORE_URL and the allsource-auth feature"]
async fn two_instances_sharing_one_core_admit_one_session() {
    let core = std::env::var("ALLSOURCE_CORE_URL").expect("ALLSOURCE_CORE_URL is set");
    let (stub_url, stub) = spawn_stub().await;
    let a = harness_with(
        Coordination::Shared,
        &core,
        stub_url.clone(),
        Arc::clone(&stub),
    )
    .await;
    let b = harness_with(Coordination::Shared, &core, stub_url, Arc::clone(&stub)).await;

    let started = start(&a, Some("/posts"), None).await;
    grant(
        &a,
        "live-code",
        &started,
        verified_user("live-race@example.com"),
    );
    let (ra, rb) = tokio::join!(
        callback(
            &a,
            Some("live-code"),
            &started.state,
            Some(&started.pending_cookie)
        ),
        callback(
            &b,
            Some("live-code"),
            &started.state,
            Some(&started.pending_cookie)
        ),
    );
    let sessions = [&ra, &rb]
        .iter()
        .filter(|r| session_cookie(r).is_some())
        .count();
    assert_eq!(sessions, 1);
}

#[tokio::test]
async fn sign_out_ends_a_google_session() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("out@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    let session = session_cookie(&resp).expect("session");
    assert_eq!(me_status(&h, &session).await, StatusCode::OK);

    let out = h
        .client
        .post(format!("{}/auth/sign-out", h.base))
        .header(header::COOKIE, &session)
        .send()
        .await
        .unwrap();
    assert!(out.status().is_success());
    assert_eq!(me_status(&h, &session).await, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unconfigured_providers_are_not_found() {
    let h = harness().await;
    let start_resp = h
        .client
        .get(format!("{}/auth/oauth/github/start", h.base))
        .send()
        .await
        .unwrap();
    assert_eq!(start_resp.status(), StatusCode::NOT_FOUND);
    let callback_resp = h
        .client
        .get(format!("{}/auth/callback/github", h.base))
        .query(&[("code", "c"), ("state", "s")])
        .send()
        .await
        .unwrap();
    assert_eq!(callback_resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_start_route_is_rate_limited_per_client() {
    let h = harness().await;
    // Waves of 30 in flight: fast enough that the refill (60/s) cannot cover
    // the 120-token burst, small enough not to overflow the listen backlog.
    let url = format!("{}/auth/oauth/google/start", h.base);
    let mut limited = 0;
    for _ in 0..6 {
        let mut wave = tokio::task::JoinSet::new();
        for _ in 0..30 {
            let client = h.client.clone();
            let url = url.clone();
            wave.spawn(async move { client.get(url).send().await.unwrap().status() });
        }
        while let Some(status) = wave.join_next().await {
            if status.unwrap() == StatusCode::TOO_MANY_REQUESTS {
                limited += 1;
            }
        }
    }
    assert!(limited > 0, "180 starts from one client were never limited");
}

/// A single-use claim that cannot be written refuses the callback before the
/// library runs, so nothing is created.
#[tokio::test]
async fn an_unavailable_claim_refuses_the_callback_and_writes_nothing() {
    let h = harness().await;
    h.state.faults.claims_down.store(true, Ordering::Relaxed);
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("claimfail@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(session_cookie(&resp).is_none());
    assert!(user_by_email(&h, "claimfail@example.com").await.is_none());
}

/// A verified owner's first Google sign-in with the audit down: refused, and the Google
/// row the callback wrote is removed. The user row is unchanged.
#[tokio::test]
async fn an_audit_outage_on_a_new_link_removes_the_new_link() {
    let h = harness().await;
    register_password(&h, "newlink@example.com").await;
    let id = user_by_email(&h, "newlink@example.com")
        .await
        .expect("registered");
    verify_email(&h, &id).await;
    let row_before = user_row(&h, &id).await;
    h.state.faults.audit_down.store(true, Ordering::Relaxed);

    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("newlink@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert!(
        session_cookie(&resp).is_none(),
        "a session was issued during an audit outage"
    );
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(
        account_ids(&h, &id).await.is_empty(),
        "the new Google row survived"
    );
    assert_eq!(user_row(&h, &id).await, row_before);
}

/// A verified owner with a Google link from an earlier sign-in. The audit is down on a
/// second sign-in: that sign-in is refused, but the earlier link is not undone and no
/// session is issued for the refused callback.
#[tokio::test]
async fn an_audit_outage_on_a_repeat_sign_in_keeps_the_existing_link() {
    let h = harness().await;
    register_password(&h, "linked@example.com").await;
    let id = user_by_email(&h, "linked@example.com")
        .await
        .expect("registered");
    verify_email(&h, &id).await;

    let first = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &first, verified_user("linked@example.com"));
    let resp = callback(&h, Some("c1"), &first.state, Some(&first.pending_cookie)).await;
    assert!(
        session_cookie(&resp).is_some(),
        "the first link did not sign in"
    );
    let linked = account_ids(&h, &id).await;
    assert_eq!(linked.len(), 1);

    h.state.faults.audit_down.store(true, Ordering::Relaxed);
    let second = start(&h, Some("/posts"), None).await;
    grant(&h, "c2", &second, verified_user("linked@example.com"));
    let resp = callback(&h, Some("c2"), &second.state, Some(&second.pending_cookie)).await;
    assert!(
        session_cookie(&resp).is_none(),
        "a session was issued during an audit outage"
    );
    assert_eq!(
        account_ids(&h, &id).await,
        linked,
        "the earlier Google link was undone"
    );
}

/// A settle failure after the library wrote a new verified link: the link is removed
/// and no session is issued.
#[tokio::test]
async fn a_settle_failure_removes_the_link_this_callback_wrote() {
    let h = harness().await;
    register_password(&h, "settle@example.com").await;
    let id = user_by_email(&h, "settle@example.com")
        .await
        .expect("registered");
    verify_email(&h, &id).await;
    h.state.faults.settle_down.store(true, Ordering::Relaxed);

    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("settle@example.com"));
    let resp = callback(
        &h,
        Some("c1"),
        &started.state,
        Some(&started.pending_cookie),
    )
    .await;
    assert!(session_cookie(&resp).is_none());
    assert_eq!(location(&resp), format!("{PRIMARY}/"));
    assert!(
        account_ids(&h, &id).await.is_empty(),
        "the link survived a settle failure"
    );
}

/// Two distinct states racing one unverified identity: both are refused, neither issues
/// a session, and neither leaves a Google row or changes the user row.
#[tokio::test]
async fn two_callbacks_racing_one_unverified_identity_both_refuse() {
    let h = harness().await;
    register_password(&h, "race2@example.com").await;
    let id = user_by_email(&h, "race2@example.com")
        .await
        .expect("registered");
    let row_before = user_row(&h, &id).await;

    let a = start(&h, Some("/posts"), None).await;
    grant(&h, "ca", &a, verified_user("race2@example.com"));
    let b = start(&h, Some("/posts"), None).await;
    grant(&h, "cb", &b, verified_user("race2@example.com"));
    let (ra, rb) = tokio::join!(
        callback(&h, Some("ca"), &a.state, Some(&a.pending_cookie)),
        callback(&h, Some("cb"), &b.state, Some(&b.pending_cookie)),
    );
    for resp in [&ra, &rb] {
        assert!(session_cookie(resp).is_none(), "a session issued in a race");
        assert_eq!(
            location(resp),
            format!("{PRIMARY}/login?error=unverified_link")
        );
    }
    assert!(
        account_ids(&h, &id).await.is_empty(),
        "a Google row survived the race"
    );
    assert_eq!(user_row(&h, &id).await, row_before);
}

/// The gate matches exact spellings. A trailing slash must not reach a gated path
/// under a different spelling, and an encoded spelling must not reach the library's
/// callback. None of them may sign anyone in.
#[tokio::test]
async fn gate_path_variants_never_sign_anyone_in() {
    let h = harness().await;
    let started = start(&h, Some("/posts"), None).await;
    grant(&h, "c1", &started, verified_user("variant@example.com"));
    for path in [
        "/auth/callback/google/",
        "/auth/%63allback/google",
        "/auth/callback/%67oogle",
    ] {
        let resp = h
            .client
            .get(format!("{}{path}", h.base))
            .query(&[("code", "c1"), ("state", started.state.as_str())])
            .header(header::COOKIE, &started.pending_cookie)
            .send()
            .await
            .unwrap();
        assert!(session_cookie(&resp).is_none(), "{path} signed someone in");
    }
    for path in ["/auth/sign-in/social/", "/auth/link-social/"] {
        let resp = h
            .client
            .post(format!("{}{path}", h.base))
            .json(&json!({"provider": "google"}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{path}");
    }
}

/// A trailing slash on sign-up is normalised like the plain path, or refused by the
/// library's routing. Either way no mixed-case account is stored.
#[tokio::test]
async fn a_trailing_slash_on_sign_up_is_normalised_or_refused() {
    let h = harness().await;
    let resp = h
        .client
        .post(format!("{}/auth/sign-up/email/", h.base))
        .json(&json!({"email": "Trail@Example.com", "password": "password123", "name": "T"}))
        .send()
        .await
        .unwrap();
    if resp.status().is_success() {
        assert!(user_by_email(&h, "trail@example.com").await.is_some());
    } else {
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
    assert!(user_by_email(&h, "Trail@Example.com").await.is_none());
}

/// The session-management routes are not mounted: the client reads the session from
/// `/me`, so each of these is a door with no caller.
#[tokio::test]
async fn the_session_management_routes_are_not_mounted() {
    let h = harness().await;
    let signup = register_password(&h, "doors@example.com").await;
    let session = session_cookie(&signup).expect("sign-up sets a session");
    for path in ["/auth/list-sessions", "/auth/get-session"] {
        let resp = h
            .client
            .get(format!("{}{path}", h.base))
            .header(header::COOKIE, &session)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND, "{path} is mounted");
    }
    let revoke = h
        .client
        .post(format!("{}/auth/revoke-other-sessions", h.base))
        .header(header::COOKIE, &session)
        .send()
        .await
        .unwrap();
    assert_eq!(revoke.status(), StatusCode::NOT_FOUND);
}

/// The username route is a second password sign-in. It is not mounted.
#[tokio::test]
async fn the_username_sign_in_route_is_not_mounted() {
    let h = harness().await;
    register_password(&h, "nameuser@example.com").await;
    let resp = h
        .client
        .post(format!("{}/auth/sign-in/username", h.base))
        .json(&json!({"username": "nameuser", "password": "password123"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert!(session_cookie(&resp).is_none());
}
