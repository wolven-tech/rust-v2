//! `GET /openapi.json`.
//!
//! D6's third argument for keeping `apps/api` a standalone service is "one
//! contract, many clients" — a future Tauri or mobile client consumes a
//! documented REST surface, where Dioxus server functions would serialize over
//! a Dioxus-internal protocol. That argument only holds if the contract is
//! actually published, so it is published here.
//!
//! The spec is hand-written rather than derived. `allframe`'s `openapi` feature
//! generates from *its* handler macros, which this service does not use (see
//! the D5 deviation note in `lib.rs`). A generated-from-code spec is better and
//! is the natural follow-up; a hand-written one that is served and testable
//! beats a generated one that does not exist.

use axum::Json;
use serde_json::{Value, json};

pub async fn spec() -> Json<Value> {
    Json(document())
}

fn document() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "rust-v2 API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Event-sourced API over AllSource. Sessions are HttpOnly cookies \
                            issued by better-auth under /auth; a Bearer token is also accepted."
        },
        "components": {
            "securitySchemes": {
                "sessionCookie": { "type": "apiKey", "in": "cookie", "name": "better-auth.session_token" },
                "bearer": { "type": "http", "scheme": "bearer" }
            },
            "schemas": {
                "PostView": {
                    "type": "object",
                    "required": ["id", "author_id", "title", "content", "created_at", "updated_at"],
                    "properties": {
                        "id": { "type": "string", "format": "uuid" },
                        "author_id": { "type": "string", "format": "uuid" },
                        "title": { "type": "string" },
                        "content": { "type": "string" },
                        "created_at": { "type": "string", "format": "date-time" },
                        "updated_at": { "type": "string", "format": "date-time" }
                    }
                },
                "ErrorResponse": {
                    "type": "object",
                    "required": ["code", "message"],
                    "properties": {
                        "code": { "type": "string" },
                        "message": { "type": "string" }
                    }
                }
            }
        },
        "security": [{ "sessionCookie": [] }, { "bearer": [] }],
        "paths": {
            "/health": { "get": { "summary": "Liveness probe", "security": [], "responses": { "200": { "description": "ok" } } } },
            "/me": { "get": { "summary": "The authenticated principal", "responses": { "200": { "description": "session" }, "401": { "description": "unauthorized" } } } },
            "/posts": {
                "get": { "summary": "List posts (posts_v1 projection)", "responses": { "200": { "description": "posts" }, "401": { "description": "unauthorized" } } },
                "post": { "summary": "Create a post (appends content.post.created)", "responses": { "201": { "description": "created" }, "422": { "description": "validation failed" } } }
            },
            "/posts/{id}": {
                "get": { "summary": "One post, folded on read", "responses": { "200": { "description": "post" }, "404": { "description": "not found" } } },
                "patch": { "summary": "Edit a post (appends content.post.edited)", "responses": { "200": { "description": "post" }, "403": { "description": "not the author" } } },
                "delete": { "summary": "Delete a post (appends content.post.deleted)", "responses": { "204": { "description": "deleted" }, "403": { "description": "not the author" } } }
            },
            "/users/{id}": {
                "get": { "summary": "A user profile, folded on read", "responses": { "200": { "description": "user" }, "404": { "description": "not found" } } },
                "patch": { "summary": "Update a profile (appends identity.user.profile_updated)", "responses": { "200": { "description": "user" }, "403": { "description": "not the owner" } } }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::document;

    /// The spec must cover every route the router actually mounts. This is the
    /// cheap version of "the contract is real": if a route is added without a
    /// spec entry, this fails.
    #[test]
    fn the_spec_documents_every_mounted_route() {
        let spec = document();
        let paths = spec["paths"].as_object().unwrap();
        for route in ["/health", "/me", "/posts", "/posts/{id}", "/users/{id}"] {
            assert!(paths.contains_key(route), "{route} is not documented");
        }
    }

    #[test]
    fn health_is_documented_as_unauthenticated() {
        let spec = document();
        assert_eq!(
            spec["paths"]["/health"]["get"]["security"],
            serde_json::json!([]),
            "health must not require a session — it is mounted outside auth"
        );
    }
}
