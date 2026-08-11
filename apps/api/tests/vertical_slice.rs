//! **The acceptance test.** One feature, end to end, against a real AllSource
//! Core:
//!
//! ```text
//! HTTP request → domain event → AllSource append → projection fold
//!              → read model → JSON the Dioxus app renders
//! ```
//!
//! A scaffold that compiles but has never moved a byte through AllSource proves
//! nothing, so this test does not mock anything below the HTTP layer. It runs
//! the real router, the real `better-auth` stack, the real `EventWriter`, and
//! the real `QueryClient::query_and_fold`.
//!
//! ## Running it
//!
//! It is `#[ignore]`d by default because it needs a live Core:
//!
//! ```bash
//! docker compose up -d              # or: allsource-core (see README)
//! export ALLSOURCE_CORE_URL=http://localhost:3900
//! export ALLSOURCE_QUERY_URL=http://localhost:3900
//! export ALLSOURCE_API_KEY=dev
//! export JWT_SECRET=dev-secret-key-that-is-at-least-32-characters-long
//! cargo test -p api --test vertical_slice -- --ignored --nocapture
//! ```
//!
//! It **fails loudly** if Core is unreachable rather than skipping, because a
//! silently-skipped acceptance test is indistinguishable from a passing one.

use std::sync::Arc;

use api::{build_router, infrastructure::state::build_state};
use rv2_shared::ServerConfig;
use serde_json::json;

/// Read the same environment the binary does, with test-shaped fallbacks for
/// the things that are not AllSource URLs. The AllSource URLs themselves have
/// **no** fallback — R6 says never hard-code a port, and that applies here too.
fn config() -> ServerConfig {
    for (key, value) in [
        (
            "JWT_SECRET",
            "dev-secret-key-that-is-at-least-32-characters-long",
        ),
        ("PORT", "0"),
        ("CORS_ORIGINS", "http://localhost:4402"),
        ("AUTH_BASE_URL", "http://localhost:4400/auth"),
    ] {
        if std::env::var(key).is_err() {
            // SAFETY: set before any thread is spawned in this test process.
            unsafe { std::env::set_var(key, value) };
        }
    }
    ServerConfig::from_env().expect(
        "ALLSOURCE_CORE_URL, ALLSOURCE_QUERY_URL and ALLSOURCE_API_KEY must be set — \
         see the module docs for the exact exports",
    )
}

/// Boot the whole service on an ephemeral port and return its base URL.
async fn serve() -> String {
    let config = config();
    let state = Arc::new(
        build_state(&config)
            .await
            .expect("could not build app state — is AllSource Core running?"),
    );
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

/// Register a user through better-auth's own route and return the session token.
///
/// Real credential auth, not a fabricated principal: this is what makes the
/// `403`/`401` assertions below mean anything.
async fn register(client: &reqwest::Client, base: &str, email: &str) -> String {
    let response = client
        .post(format!("{base}/auth/sign-up/email"))
        .json(&json!({ "email": email, "password": "password123", "name": "Test User" }))
        .send()
        .await
        .expect("sign-up request");

    let status = response.status();
    let body: serde_json::Value = response.json().await.expect("sign-up body is JSON");
    assert!(status.is_success(), "sign-up failed ({status}): {body}");

    body.get("token")
        .and_then(|t| t.as_str())
        .or_else(|| body.pointer("/session/token").and_then(|t| t.as_str()))
        .unwrap_or_else(|| panic!("no session token in sign-up response: {body}"))
        .to_string()
}

#[tokio::test]
#[ignore = "requires a live AllSource Core; see the module docs"]
async fn an_event_travels_the_full_path() {
    let base = serve().await;
    let client = reqwest::Client::new();

    // ── 0. The service is up and Core is reachable ───────────────────────────
    let health: serde_json::Value = client
        .get(format!("{base}/health"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("health: {health}");
    assert_eq!(health["status"], "ok");
    assert_eq!(
        health["allsource_reachable"], true,
        "Core is not reachable — the rest of this test would be meaningless"
    );

    // ── 1. Real auth ─────────────────────────────────────────────────────────
    let email = format!("slice-{}@example.com", uuid::Uuid::new_v4());
    let token = register(&client, &base, &email).await;

    let me: serde_json::Value = client
        .get(format!("{base}/me"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("me: {me}");
    assert_eq!(me["email"], email);

    // ── 2. HTTP request → domain event → AllSource append ────────────────────
    let created = client
        .post(format!("{base}/posts"))
        .bearer_auth(&token)
        .json(&json!({
            "title": "  On the Analytical Engine  ",
            "content": "Note G."
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201);
    let post: serde_json::Value = created.json().await.unwrap();
    println!("created: {post}");

    // The title is trimmed, which proves the request went through
    // `rv2_domain::post::create` rather than being echoed back.
    assert_eq!(post["title"], "On the Analytical Engine");
    let id = post["id"].as_str().expect("id").to_string();

    // ── 3. Projection fold → read model (fold-on-read) ───────────────────────
    let fetched: serde_json::Value = client
        .get(format!("{base}/posts/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("fold-on-read: {fetched}");
    assert_eq!(fetched["id"], post["id"]);
    assert_eq!(fetched["content"], "Note G.");

    // ── 4. An edit folds on top rather than replacing ────────────────────────
    let edited: serde_json::Value = client
        .patch(format!("{base}/posts/{id}"))
        .bearer_auth(&token)
        .json(&json!({ "title": "On the Analytical Engine (rev. 2)" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("after edit: {edited}");
    assert_eq!(edited["title"], "On the Analytical Engine (rev. 2)");
    assert_eq!(
        edited["content"], "Note G.",
        "an absent patch field must be unchanged, not cleared"
    );

    // ── 5. The list read model contains it ───────────────────────────────────
    let list: Vec<serde_json::Value> = client
        .get(format!("{base}/posts"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    println!("list: {} post(s)", list.len());
    assert!(
        list.iter().any(|p| p["id"] == post["id"]),
        "the new post is missing from GET /posts"
    );

    // ── 6. Delete removes it from both read paths ────────────────────────────
    let deleted = client
        .delete(format!("{base}/posts/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(deleted.status(), 204);

    let after = client
        .get(format!("{base}/posts/{id}"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        after.status(),
        404,
        "a deleted post must fold to 'does not exist'"
    );
}

/// R4 is "the risk most likely to produce a real incident", and the doc calls
/// the cross-user test non-optional. So it is here, not deferred.
#[tokio::test]
#[ignore = "requires a live AllSource Core; see the module docs"]
async fn a_different_user_cannot_edit_or_delete_someone_elses_post() {
    let base = serve().await;
    let client = reqwest::Client::new();

    let author = register(
        &client,
        &base,
        &format!("author-{}@example.com", uuid::Uuid::new_v4()),
    )
    .await;
    let stranger = register(
        &client,
        &base,
        &format!("stranger-{}@example.com", uuid::Uuid::new_v4()),
    )
    .await;

    let post: serde_json::Value = client
        .post(format!("{base}/posts"))
        .bearer_auth(&author)
        .json(&json!({ "title": "Mine", "content": "Not yours" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = post["id"].as_str().unwrap();

    let edit = client
        .patch(format!("{base}/posts/{id}"))
        .bearer_auth(&stranger)
        .json(&json!({ "title": "Hijacked" }))
        .send()
        .await
        .unwrap();
    println!("stranger PATCH -> {}", edit.status());
    assert_eq!(edit.status(), 403);

    let delete = client
        .delete(format!("{base}/posts/{id}"))
        .bearer_auth(&stranger)
        .send()
        .await
        .unwrap();
    println!("stranger DELETE -> {}", delete.status());
    assert_eq!(delete.status(), 403);

    // And the post is genuinely untouched.
    let still: serde_json::Value = client
        .get(format!("{base}/posts/{id}"))
        .bearer_auth(&author)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(still["title"], "Mine");
}

/// The unauthenticated case, separated from the forbidden case on purpose: the
/// client redirects to `/login` on 401 and would loop forever if 403 came back
/// the same way.
#[tokio::test]
#[ignore = "requires a live AllSource Core; see the module docs"]
async fn domain_routes_reject_anonymous_callers_with_401() {
    let base = serve().await;
    let client = reqwest::Client::new();

    for (method, path) in [("GET", "/posts"), ("GET", "/me")] {
        let response = client
            .request(method.parse().unwrap(), format!("{base}{path}"))
            .send()
            .await
            .unwrap();
        println!("anonymous {method} {path} -> {}", response.status());
        assert_eq!(response.status(), 401, "{method} {path} was not protected");
    }
}
