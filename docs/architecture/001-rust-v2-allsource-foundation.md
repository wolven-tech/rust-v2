# rust-v2 — All-Rust / AllSource Foundation

**Status:** Design (slice 1 of 2). Prompt `002` scaffolds the workspace from this document.
**Author:** produced under `.prompts/001-rust-v2-allsource-architecture-doc.md`
**Research date:** all versions and signatures below were verified against crates.io, the live
GitHub sources, and the local filesystem **during this session**. Nothing is quoted from memory.

This is a build specification, not an essay. Every question has a **decision**. Things I could
not verify are in [§10 Open Questions](#102-open-questions), never patched over with a guess.

---

## Decisions Summary

| # | Decision | Choice | One-line rationale |
|---|---|---|---|
| D1 | Repo strategy | **Fresh repo `rust-v2`**, seeded by copying `tooling/meta` + docs from rust-v1 | rust-v1's git history is 90% TypeScript that never lands in v2; an in-place migration means a months-long broken `main`. |
| D2 | Rust edition / toolchain | **edition 2024**, `rust-toolchain.toml` channel `1.97.1` (pinned) | `allsource-core` needs 1.92, `allframe` 1.89, `dioxus` 1.83; 2024 needs 1.85. Local toolchain is 1.97.1. |
| D3 | AllSource access | **Official `allsource` crate v0.23.0 from crates.io** (`CoreClient` + `QueryClient` + `ProjectionWorker`) | An official typed client exists now; getformlab hand-rolled one because it predates it (pins `allframe` 0.1.12). Do not re-hand-roll. |
| D4 | Deployment mode | **Remote Core** — `CoreClient` → `:3900`, `QueryClient` → `:3902`, both URLs from env | User decision. `allsource-core` (embedded, longhand's mode) is **not** a rust-v2 dependency. |
| D5 | `allframe` role | `allframe 0.1.28` for router / health / openapi / resilience / rate-limit **only**. Not `cqrs-allsource`. | The `allsource` SDK owns event I/O; layering allframe's CQRS on top would give two competing event abstractions. |
| D6 | API shape | **`apps/api` stays a separate Axum service.** Dioxus apps are clients over REST. Server functions are **not** used for business endpoints. | better-auth mounts its own Axum router (OAuth callbacks, cookie scoping); allframe already owns routing; a stable REST+OpenAPI contract survives a future Tauri/mobile client. |
| D7 | `apps/app` (dashboard) | **Dioxus 0.7.10 web CSR SPA** (`features = ["web","router"]`), no `fullstack`, no SSR | Authenticated dashboard — SEO irrelevant; keeps the crate 100% `wasm32` and out of `dioxus-server`. |
| D8 | `apps/web` (marketing) | **Dioxus 0.7.10 fullstack in SSG mode**, built with `dx bundle --web --ssg`, deployed as static files | Marketing pages need crawlable HTML. SSG uses the server binary at *build* time only — nothing extra to run in prod. |
| D9 | `better-auth-allsource` | **Vendor `crates/better-auth-allsource/` from getformlab.** Do **not** use the crates.io release. | The published `better-auth-allsource 0.14.12` declares `better-auth-core ^0.8`; `better-auth 0.10.0` needs `better-auth-core 0.10.0`. Two incompatible trait copies → will not compile. Proof in [§5.1](#51-the-better-auth-allsource-port). |
| D10 | `event_type` wire format | `<domain>.<entity>.<action>`, lowercase dot-notation, e.g. `identity.user.registered` | AllSource validates event types as "lowercase, dot-notation" (`docs/current/EVENT_STORE_FEATURES.md:53`). |
| D11 | Variant↔wire mapping | Explicit `DomainEvent::event_type(&self) -> &'static str` + `from_wire`, enforced by an exhaustive round-trip test. **No `decode_event` fallback, no `normalize_event_type`.** | getformlab's fallback silently mis-decodes; the SDK's normalizer is lossy (`TwoFactorCreated` → `two.factor.created`). See [§3.2](#32-eventtype-naming-and-the-serde-tag-reconciliation). |
| D12 | Event evolution | Additive only. Every post-v1 field carries `#[serde(default)]`. Breaking change ⇒ **new** event type + retained old variant. Golden-JSON corpus test. | Old events are immutable and must keep deserializing forever. |
| D13 | Domain time | Every payload carries its own `occurred_at`. **Never** read domain time from the envelope timestamp. | AllSource assigns `timestamp` server-side at ingest, so migrated/backdated events would all read as "now". |
| D14 | Projection trait | Use the SDK's `allsource::EventFolder` for fold-on-read and `allsource::ProjectionWorker` for continuous folding. Do **not** define a bespoke `Projection` trait. | Both are verified in the SDK source; the worker already does WS subscribe, checkpoint, dedup, reconnect. |
| D15 | Read-model rebuild | Rebuild = **rename the worker's durable consumer id** (`.name("posts_v2")`), run old + new in parallel, cut reads, delete old state | Core keys the cursor server-side by consumer id, so a new id replays from zero. No manual cursor surgery. |
| D16 | Client HTTP from WASM | `gloo-net 0.7` with `RequestCredentials::Include`; **not** `reqwest` | getformlab's proven `wasm32` path; `reqwest` on wasm drags a much larger tree into the bundle. |
| D17 | Session transport | HttpOnly cookie issued by better-auth on `apps/api`; Dioxus sends `credentials: include`; Axum validates per-request | Matches `getformlab:apps/api/src/infrastructure/auth/middleware.rs`, which accepts cookie *or* `Bearer`. |
| D18 | OAuth | Google, via `better_auth::plugins::oauth::OAuthPlugin`, callback landing on `apps/api`, signed pending-origin cookie | Replaces Supabase `signInWithOAuth` + `/api/auth/callback` 1:1; getformlab's ADR-006 pattern is the reference. |
| D19 | Not event-sourced | Rate-limit counters, session-validation cache, blob/media bytes, full-text/analytics indexes, and **uniqueness constraints** | AllSource has no unique index and no cross-entity transaction. Stated honestly in [§9](#9-what-is-not-event-sourced). |
| D20 | TS packages | `logger`→`tracing`; `kv`→allframe `rate-limit`; `email`→`tera`; `react-query`→`use_resource`; `ui`→`rv2-ui` (Dioxus); `analytics`→PostHog JS snippet + AllSource events; `jobs`→**deleted** | Detail and per-package reasoning in [§8.4](#84-fate-of-the-typescript-support-packages). |
| D21 | Data migration | One-shot `tooling/pg2events` Rust binary: `posts`/`users` rows → events. **Credentials do not migrate** — users re-register or re-link OAuth. | Supabase password hashes live in `auth.users` behind the service role and better-auth 0.10 cannot verify them (unverified — see OQ-3). |

---

## 1. Workspace layout

### 1.1 Directory tree

```
rust-v2/
├── Cargo.toml                     # virtual workspace manifest
├── Cargo.lock                     # committed
├── rust-toolchain.toml            # channel = "1.97.1", targets += wasm32-unknown-unknown
├── rustfmt.toml  .clippy.toml  cargo-sort.toml   # copied from rust-v1
├── meta.toml                      # meta orchestrator (see §1.4)
├── deny.toml                      # cargo-deny: license + duplicate-version policy
├── docker-compose.yml             # AllSource Core :3900 + Query Service :3902 for local dev
│
├── apps/
│   ├── api/                       # [bin] Axum + allframe HTTP API. The ONLY server process.
│   │   └── src/
│   │       ├── main.rs            #   router assembly, CORS, better-auth mount
│   │       ├── infrastructure/
│   │       │   ├── auth/          #   better.rs (BetterAuth builder), middleware.rs (ExtractAuthUser)
│   │       │   ├── config.rs      #   ServerConfig::from_env, incl. AllSource URLs + API key
│   │       │   └── email.rs       #   tera-rendered transactional email
│   │       └── presentation/
│   │           └── handlers/      #   REST handlers; own the write path (command → event)
│   │
│   ├── app/                       # [cdylib+rlib] Dioxus CSR SPA — authenticated dashboard
│   │   ├── Dioxus.toml
│   │   ├── assets/                #   tailwind.css, favicon
│   │   └── src/{main.rs,routes/,views/}
│   │
│   └── web/                       # [bin+lib] Dioxus fullstack, built SSG — public marketing site
│       ├── Dioxus.toml
│       └── src/{main.rs,routes/,views/}
│
├── crates/
│   ├── rv2-events/                # WASM-safe. DomainEvent enum, EventEnvelope, wire-type mapping.
│   ├── rv2-domain/                # WASM-safe. Pure invariants/validation. No I/O, no clock.
│   ├── rv2-api-types/             # WASM-safe. Request/response DTOs shared api ↔ apps.
│   ├── rv2-ui/                    # WASM-safe. Dioxus component kit (Button, Input, Card, Toast…).
│   ├── rv2-client/                # WASM-safe. Typed gloo-net client for apps/api.
│   ├── rv2-allsource/             # SERVER-ONLY. allsource SDK wrapper, folders, projection workers.
│   ├── rv2-shared/                # SERVER-ONLY. ServerConfig, AuthUser/Role, cross-cutting utils.
│   └── better-auth-allsource/     # SERVER-ONLY. VENDORED from getformlab. See §5.1.
│
└── tooling/
    ├── meta/                      # copied verbatim from rust-v1:tooling/meta
    └── pg2events/                 # [bin] one-shot Supabase Postgres → AllSource migrator (§8.3)
```

### 1.2 Root `Cargo.toml`

```toml
[workspace]
resolver = "3"
members = [
    "apps/api", "apps/app", "apps/web",
    "crates/rv2-events", "crates/rv2-domain", "crates/rv2-api-types",
    "crates/rv2-ui", "crates/rv2-client", "crates/rv2-allsource",
    "crates/rv2-shared", "crates/better-auth-allsource",
    "tooling/meta", "tooling/pg2events",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"

[workspace.dependencies]
# ── AllSource ───────────────────────────────────────────────────────────────
allsource      = { version = "0.23.0", default-features = false, features = ["rustls", "projection-worker"] }
allframe       = { version = "0.1.28", default-features = false, features = ["router", "health", "openapi", "resilience", "rate-limit"] }
allframe-core  = { version = "0.1.28", default-features = false }

# ── Auth ────────────────────────────────────────────────────────────────────
better-auth            = { version = "0.10.0", features = ["axum"] }
better-auth-core       = { version = "0.10.0" }
better-auth-allsource  = { path = "crates/better-auth-allsource", default-features = false, features = ["rustls"] }

# ── Server ──────────────────────────────────────────────────────────────────
axum       = { version = "0.8", features = ["macros"] }
axum-extra = { version = "0.10", features = ["typed-header"] }
tokio      = { version = "1", features = ["rt-multi-thread", "macros", "time", "sync", "net", "signal"] }
tower      = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
reqwest    = { version = "0.13", default-features = false, features = ["json", "rustls-tls"] }

# ── Frontend ────────────────────────────────────────────────────────────────
dioxus   = { version = "0.7.10" }             # per-app features; see §6
gloo-net = { version = "0.7", default-features = false, features = ["http", "json"] }

# ── Shared, WASM-safe ───────────────────────────────────────────────────────
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
chrono     = { version = "0.4", features = ["serde"] }
uuid       = { version = "1", features = ["v4", "serde"] }
thiserror  = "2"

# ── Observability ───────────────────────────────────────────────────────────
tracing            = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[profile.release]
codegen-units = 1
lto = true
opt-level = "s"
strip = true
```

> **Why `resolver = "3"`.** Edition 2024 implies resolver 3 (MSRV-aware). getformlab is on
> `resolver = "2"` with `edition = "2024"` at the package level
> (`getformlab:Cargo.toml`); declaring it explicitly avoids the mismatch warning.
>
> **`panic = "abort"` is deliberately omitted** even though getformlab sets it. It breaks
> `#[should_panic]` tests and interacts badly with the wasm targets; revisit as a
> per-profile setting for the API binary only.

### 1.3 `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.97.1"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

Pinned rather than `stable` (getformlab uses `stable`) because the workspace mixes four
independently-versioned MSRVs and a silent stable bump is a bad way to discover that.

### 1.4 `meta.toml`

`tooling/meta`'s config model is fully generic: `tools` is a `HashMap<String, ToolConfig>` and
`ProjectConfig.project_type` is a free `String`
(`rust-v1:tooling/meta/src/config.rs:9,30`). So `dx` can be declared as a tool without
touching meta's source. The only meta code that branches on the type string is the doctor's
binary-staleness check (`rust-v1:tooling/meta/src/execution/mod.rs:332`, `!= "rust"`), which is
harmless for library and WASM crates.

```toml
version = "1"

[workspace]
name = "rust-v2"
root = "."

[tools.cargo]
enabled = true
command = "cargo"
for_languages = ["rust"]
for_tasks = ["build", "test", "fmt", "clippy", "audit", "check"]

[tools.bacon]
enabled = true
command = "bacon"
for_languages = ["rust"]
for_tasks = ["dev"]

[tools.dx]
enabled = true
command = "dx"
for_languages = ["rust"]
for_tasks = ["dev", "build"]

[tools.docker]
enabled = true
command = "docker"
for_tasks = ["dev"]

# ── infra: AllSource Core :3900 + Query Service :3902 ────────────────────────
[projects.infra]
type = "docker"
path = "."
[projects.infra.tasks]
dev = { tool = "docker", command = "compose up" }

# ── apps ────────────────────────────────────────────────────────────────────
[projects.api]
type = "rust"
path = "apps/api"
[projects.api.tasks]
dev     = { tool = "bacon",  command = "run-long" }
build   = { tool = "cargo",  command = "build --release -p api" }
test    = { tool = "cargo",  command = "test -p api" }
clippy  = { tool = "cargo",  command = "clippy -p api --all-targets -- -D warnings" }

[projects.app]
type = "rust"
path = "apps/app"
[projects.app.tasks]
dev   = { tool = "dx", command = "serve --package app --platform web --port 4402" }
build = { tool = "dx", command = "bundle --package app --platform web --release" }
test  = { tool = "cargo", command = "test -p app" }

[projects.web]
type = "rust"
path = "apps/web"
[projects.web.tasks]
dev   = { tool = "dx", command = "serve --package web --port 4401" }
build = { tool = "dx", command = "bundle --package web --web --ssg --release" }
test  = { tool = "cargo", command = "test -p web" }

# ── library crates: build/test only, no dev task (excluded from `meta dev`) ──
[projects.rv2-events]
type = "rust"
path = "crates/rv2-events"
[projects.rv2-events.tasks]
build = { tool = "cargo", command = "build -p rv2-events" }
test  = { tool = "cargo", command = "test -p rv2-events" }

# … identical five-line blocks for rv2-domain, rv2-api-types, rv2-ui,
#   rv2-client, rv2-allsource, rv2-shared, better-auth-allsource …
```

Ports preserve rust-v1's allocation (`apps/web` 4401 → `package.json:"next dev -p 4401"`,
`apps/app` 4402, api 4400) so `.env` and bookmarks carry over.

---

## 2. Crate graph

### 2.1 The graph

```
                    ┌─────────────┐
                    │ rv2-events  │  (DomainEvent, EventEnvelope, wire mapping)
                    └──────┬──────┘
                           │
           ┌───────────────┼────────────────┐
           ▼               ▼                ▼
   ┌──────────────┐  ┌────────────┐  ┌───────────────┐
   │ rv2-domain   │  │rv2-api-types│ │ rv2-allsource │──▶ allsource 0.23 (SDK)
   └──────┬───────┘  └──────┬─────┘  └───────┬───────┘
          │                 │                │
          │        ┌────────┴────────┐       │   ┌──────────────────────┐
          │        ▼                 ▼       │   │ better-auth-allsource│ (vendored)
          │  ┌──────────┐     ┌──────────┐   │   └───────────┬──────────┘
          │  │rv2-client│     │  rv2-ui  │   │               │
          │  └────┬─────┘     └────┬─────┘   │        ┌──────┴──────┐
          │       │                │         │        │ rv2-shared  │
          └───────┴────────┬───────┘         │        └──────┬──────┘
                           ▼                 └───────────────┤
                  ┌────────────────┐                         ▼
                  │ apps/app  (wasm)│                  ┌───────────┐
                  │ apps/web  (wasm)│                  │ apps/api  │
                  └────────────────┘                  └───────────┘
```

**Acyclicity rule (one sentence, mechanically checkable):**
> Crates are assigned a **layer number** — `rv2-events` = 0; `rv2-domain`, `rv2-api-types` = 1;
> `rv2-ui`, `rv2-client`, `rv2-allsource`, `better-auth-allsource`, `rv2-shared` = 2;
> `apps/*` = 3 — and a crate may only depend on **strictly lower** layers. `apps/*` are leaves
> and nothing depends on them.

CI enforces it with `cargo tree --workspace --invert -p <app>` returning nothing, plus
`cargo-deny`'s `bans.multiple-versions` to catch accidental duplicate trees.

### 2.2 The WASM boundary — the single most dangerous line in this workspace

`apps/app` and `apps/web` compile to `wasm32-unknown-unknown`. **Everything they transitively
touch must too.** The crates that cross the boundary are exactly:

| Crate | Crosses to wasm32? | Allowed dependencies |
|---|---|---|
| `rv2-events` | **yes** | `serde`, `serde_json`, `chrono`, `uuid`, `thiserror` |
| `rv2-domain` | **yes** | same, plus pure-Rust algorithm crates |
| `rv2-api-types` | **yes** | `serde` (+ `chrono`, `uuid` for ids/timestamps) |
| `rv2-ui` | **yes** | `dioxus`, `rv2-api-types`, `rv2-domain` |
| `rv2-client` | **yes** | `gloo-net`, `web-sys`, `serde_json`, `rv2-api-types` |
| `rv2-allsource` | **no** | `allsource`, `reqwest`, `tokio`, `rv2-events` |
| `better-auth-allsource` | **no** | `better-auth-core`, `reqwest`, `async-trait` |
| `rv2-shared` | **no** | anything server-side |
| `apps/api` | **no** | anything server-side |

**Forbidden inside any "yes" crate**, because each of these fails to compile or link on
`wasm32-unknown-unknown`:

- `tokio` with any of `rt-multi-thread`, `net`, `fs`, `process`, `signal` (`sync` and `macros`
  alone are fine)
- `reqwest` (use `gloo-net`), `hyper`, `axum`, `tower-http`
- native TLS in any form — `native-tls`, `openssl`, `openssl-sys`
- `std::fs`, `std::net`, `std::process`, `std::thread::spawn`
- `mio`, `socket2`, `rustls` (server-side), `sqlx`, `object_store`
- **`allsource`, `allframe`, `better-auth*`** — these are server crates, full stop

**Two concrete traps, both observed in the reference repos:**

1. **`uuid::Uuid::new_v4()` and `getrandom` on wasm.** getformlab handles it with a
   target-specific block (`getformlab:crates/jbt-domain/Cargo.toml`):

   ```toml
   [target.'cfg(target_arch = "wasm32")'.dependencies]
   getrandom = { version = "0.2", features = ["js"] }
   uuid = { workspace = true, features = ["js"] }
   ```

   That recipe is for `getrandom 0.2`. The current `getrandom` is **0.4.3** (verified
   `cargo info getrandom` this session) where the feature is renamed `wasm_js` and additionally
   requires a build cfg. **Decision:** the WASM crates do **not** generate UUIDs. Ids are
   minted server-side and travel in DTOs. `rv2-domain`/`rv2-api-types` may *parse* and *carry*
   `Uuid`, never `new_v4()`. This removes the `getrandom` dependency from the WASM half
   entirely, which is both simpler and correct-by-construction. (Exact `getrandom 0.4`
   wasm wiring is **OQ-1** should we ever need it.)

2. **`chrono`'s clock.** `chrono` with default features pulls `wasmbind`/`js-sys` for
   `Utc::now()`. getformlab's `coach-web` therefore takes
   `chrono = { version = "0.4", default-features = false, features = ["alloc"] }`
   (`getformlab:apps/coach-web/Cargo.toml`, with an explanatory comment). **Decision:** same
   here — WASM crates get `chrono` with `default-features = false, features = ["alloc", "serde"]`
   and never call `now()`; "now" comes from the server or from `js_sys::Date`.

**Enforcement:** CI runs `cargo check --target wasm32-unknown-unknown -p rv2-events -p rv2-domain
-p rv2-api-types -p rv2-ui -p rv2-client` on every PR. A violation then fails in CI in ~40s
instead of failing during a `dx build` three weeks later.

---

## 3. Event model

> This is the highest-stakes section. A bad event schema is expensive after data exists.

### 3.1 The seed slice

rust-v1's entire Supabase schema is two tables (`packages/supabase/src/types/db.ts`: `Tables`
contains exactly `posts` and `users`; `Views`, `Functions`, `Enums`, `CompositeTypes` are all
empty). The seed slice is therefore **identity** and **content**.

```rust
// crates/rv2-events/src/lib.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Envelope for every event we read back out of AllSource.
///
/// Field names mirror AllSource's wire model deliberately: `entity_id` IS the stream id —
/// "There is no separate `stream_id` field"
/// (all-source:docs/allsource-qs-api-reference.md:26).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub entity_id: String,
    /// Wire event type: `<domain>.<entity>.<action>`, lowercase dot-notation.
    pub event_type: String,
    pub data: serde_json::Value,
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Server-assigned INGEST time. NOT domain time — see `DomainEvent::occurred_at`.
    pub ingested_at: DateTime<Utc>,
    #[serde(default)]
    pub version: u64,
}

/// Top-level domain event. `#[serde(tag = "type")]` matches getformlab's
/// `DomainEvent` shape (getformlab:crates/jbt-events/src/events/mod.rs:57-60):
/// the payload itself carries the discriminator under `"type"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum DomainEvent {
    // ── identity ────────────────────────────────────────────────────────────
    UserRegistered {
        id: Uuid,
        email: String,
        /// D13: domain time lives in the payload, never in the envelope.
        occurred_at: DateTime<Utc>,
        #[serde(default)]
        full_name: Option<String>,
        #[serde(default)]
        avatar_url: Option<String>,
    },
    UserProfileUpdated {
        id: Uuid,
        occurred_at: DateTime<Utc>,
        /// Sparse patch. Absent key = unchanged; explicit `null` = cleared.
        #[serde(default)]
        full_name: Option<Option<String>>,
        #[serde(default)]
        avatar_url: Option<Option<String>>,
    },

    // ── content ─────────────────────────────────────────────────────────────
    PostCreated {
        id: Uuid,
        author_id: Uuid,
        title: String,
        content: String,
        occurred_at: DateTime<Utc>,
    },
    PostEdited {
        id: Uuid,
        occurred_at: DateTime<Utc>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        content: Option<String>,
    },
    PostDeleted {
        id: Uuid,
        occurred_at: DateTime<Utc>,
    },
}
```

**Stream (entity_id) convention.** `<entity>:<uuid>` — `user:6f1c…`, `post:9ab2…`. This is
exactly the shape the vendored auth adapter already uses for its own streams
(`getformlab:crates/better-auth-allsource/src/adapter.rs:59-62`):

```rust
fn user_entity(id: &str) -> String { format!("auth-user:{id}") }
fn session_entity(token: &str) -> String { format!("auth-session:{token}") }
```

so domain streams (`user:…`) and auth streams (`auth-user:…`) never collide, and
`entity_id_prefix` queries (`GET /api/v1/projections/:name/state?entity_id_prefix=post:`)
work as a cheap keyspace shard.

### 3.2 `event_type` naming and the serde-tag reconciliation

This is the mismatch the prompt flags, and it is real. Two facts collide:

1. **AllSource wants `lowercase.dot.notation`.** Its domain validation includes "Event type
   format validation (lowercase, dot-notation)"
   (`all-source:docs/current/EVENT_STORE_FEATURES.md:53`), and every example in the API
   reference is `user.created`, `order.placed`
   (`all-source:docs/current/API_REFERENCE.md:88`). The QS reference is blunter still:
   `"event_type": "user.created", // REQUIRED: lowercase dot-notation`
   (`all-source:docs/allsource-qs-api-reference.md:18`).

2. **`#[serde(tag = "type")]` produces PascalCase variant names** — `PostCreated`.

**How getformlab handles it (read, not guessed).** From
`getformlab:crates/jbt-allsource/src/projections.rs:18-42`:

```rust
/// If the payload is missing the `"type"` key (e.g. raw payloads emitted via
/// `emit_event` with hand-built JSON), fall back to the envelope's
/// `event_type` field. The wire-level `event_type` may use the AllSource
/// `namespace.entity.action` naming convention, which won't match the
/// `DomainEvent` variant tags — so we only use it as a fallback.
fn decode_event(envelope: &EventEnvelope) -> Option<DomainEvent> {
    let mut obj = match envelope.data.clone() {
        serde_json::Value::Object(m) => m,
        _ => return None,
    };
    if !obj.contains_key("type") {
        obj.insert(
            "type".to_string(),
            serde_json::Value::String(envelope.event_type.clone()),
        );
    }
    serde_json::from_value(serde_json::Value::Object(obj)).ok()
}
```

So getformlab has **two** conventions live simultaneously: domain events are stored with
PascalCase in the payload's `"type"` (their own tests emit `envelope("CoachRegistered", …)`,
`projections.rs:2790+`), while the auth adapter writes proper dotted wire types —
`"auth.session.created"`, `"auth.session.expiry_updated"`, `"auth.session.deleted"`
(`getformlab:crates/better-auth-allsource/src/adapter.rs:341,384,399`). The fallback branch
exists to paper over that split. It is a latent bug: `.ok()` swallows every decode failure
silently, so a renamed variant degrades to "the projection quietly stops updating".

**rust-v2's decision (D11).** Decouple the two namespaces *explicitly* and test the bijection.

```rust
// crates/rv2-events/src/wire.rs
impl DomainEvent {
    /// The AllSource wire event type. This is the ONLY place the mapping lives.
    pub fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::UserRegistered      { .. } => "identity.user.registered",
            DomainEvent::UserProfileUpdated  { .. } => "identity.user.profile_updated",
            DomainEvent::PostCreated         { .. } => "content.post.created",
            DomainEvent::PostEdited          { .. } => "content.post.edited",
            DomainEvent::PostDeleted         { .. } => "content.post.deleted",
        }
    }

    /// Every wire type this codebase has ever emitted. Append-only.
    pub const ALL_WIRE_TYPES: &'static [&'static str] = &[
        "identity.user.registered",
        "identity.user.profile_updated",
        "content.post.created",
        "content.post.edited",
        "content.post.deleted",
    ];
}

/// Decode strictly. The payload's `"type"` tag is the ONLY discriminator we trust.
/// No fallback to the envelope: a decode failure is a real error and must be visible.
pub fn decode(envelope: &EventEnvelope) -> Result<DomainEvent, DecodeError> {
    let ev: DomainEvent = serde_json::from_value(envelope.data.clone())
        .map_err(|e| DecodeError::Payload { event_type: envelope.event_type.clone(), source: e })?;
    if ev.event_type() != envelope.event_type {
        return Err(DecodeError::TypeMismatch {
            envelope: envelope.event_type.clone(),
            payload: ev.event_type(),
        });
    }
    Ok(ev)
}
```

The write path is the mirror image, and it is the only place that ever names a wire string:

```rust
// crates/rv2-allsource/src/writer.rs
pub async fn append(&self, entity_id: &str, event: &DomainEvent) -> Result<(), Error> {
    self.core.ingest_event(allsource::IngestEventInput {
        event_type: event.event_type().to_string(),          // dotted, from the mapping
        entity_id:  entity_id.to_string(),
        payload:    serde_json::to_value(event)?,            // carries "type": "PostCreated"
        metadata:   None,
    }).await
}
```

Enforced by an exhaustive test — the one thing that makes this safe rather than merely tidy:

```rust
#[test]
fn wire_mapping_is_a_bijection_and_matches_allsource_grammar() {
    for e in sample_of_every_variant() {                 // must be updated when a variant is added
        let wire = e.event_type();
        assert!(DomainEvent::ALL_WIRE_TYPES.contains(&wire));
        assert!(wire.chars().all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'));
        assert_eq!(wire.split('.').count(), 3, "must be <domain>.<entity>.<action>");
        let env = envelope_for(&e);
        assert_eq!(decode(&env).unwrap(), e);             // round-trip
    }
    let mut seen = DomainEvent::ALL_WIRE_TYPES.to_vec();
    seen.sort_unstable(); seen.dedup();
    assert_eq!(seen.len(), DomainEvent::ALL_WIRE_TYPES.len(), "duplicate wire type");
}
```

**Rejected alternative — `allsource::normalize_event_type`.** The SDK ships a converter
(`all-source:sdks/rust/src/normalize.rs`) that turns PascalCase into dotted form. It is
lossy in exactly the way that matters, per its own test:

```rust
assert_eq!(normalize_event_type("TwoFactorCreated"), "two.factor.created");
assert_eq!(normalize_event_type("ApiKeyDeleted"),    "api.key.deleted");
```

`two.factor.created` is not `two_factor.created`, and the function is not injective —
`user_created`, `userCreated`, `UserCreated` and `user-created` all collapse to `user.created`,
so it cannot be inverted. It is fine as an ingestion-time tidier for third-party feeds; it is
not a schema.

### 3.3 Versioning and forward compatibility

**The rule, stated once:**

> An event that has been written is immutable and must deserialize into every future build of
> `rv2-events`, forever. Therefore: **fields may only be added, and every added field carries
> `#[serde(default)]`.** Renaming a field, changing its type, tightening `Option<T>` to `T`, or
> removing a variant are all forbidden. A change that cannot be expressed additively becomes a
> **new** wire type (`content.post.created_v2`) plus a new variant; the old variant and its
> fold arm stay in the code permanently.

This is the same discipline getformlab uses — their `ExerciseCreated` carries
`#[serde(default)] forked_from` and `#[serde(default)] pattern` with the comment "keeps older
`exercise.created` events decodable" (`getformlab:crates/jbt-events/src/events/mod.rs:105-115`)
— made explicit and mandatory rather than case-by-case.

**Enforced by a golden corpus.** `crates/rv2-events/tests/golden/` holds one JSON file per
`(wire_type, schema_version)` ever released, captured at release time. A single test
deserializes every file and asserts `decode()` succeeds. Adding a non-defaulted field then
fails CI immediately, at the exact commit that introduced it, rather than in production six
months later when an old stream is re-folded.

**Explicitly not used: AllSource's schema registry.** Core exposes one
(`POST /api/v1/schemas`, `PUT /api/v1/schemas/{subject}/compatibility`,
`all-source:docs/current/API_REFERENCE.md:282-352`). It is a good idea and a natural follow-up,
but it is server-side state that must be provisioned and kept in sync with the Rust types; the
golden corpus gives 90% of the safety with zero operational surface. Revisit once a second
service writes to the same streams.

---

## 4. Projections

### 4.1 The trait — use the SDK's, don't write one

`allsource 0.23.0` ships the trait (`all-source:sdks/rust/src/fold.rs`, verified this session):

```rust
pub trait EventFolder: Default {
    /// The domain state produced by folding events.
    type State;
    /// Apply a single event to the accumulator. Returns `true` if the event
    /// was relevant and applied, `false` if it was ignored.
    fn apply(&mut self, event: &Event) -> bool;
    /// Finalize the folder into domain state. Returns `None` if no relevant
    /// events were applied (e.g., entity doesn't exist).
    fn finalize(self) -> Option<Self::State>;
}

/// Events should be ordered chronologically (oldest first).
pub fn fold_events<F: EventFolder>(events: &[Event]) -> Option<F::State>;
```

Compare getformlab's hand-rolled equivalent
(`getformlab:crates/jbt-allsource/src/projections.rs:14-16`):

```rust
pub trait Projection {
    fn apply(&mut self, event: &EventEnvelope);
}
```

The SDK's version is strictly better: `-> bool` distinguishes "handled" from "ignored" (which
is what lets a worker count progress honestly), and `finalize() -> Option<State>` gives you
"this entity does not exist" for free instead of an empty `HashMap` you have to interrogate.
getformlab wrote theirs because they pin `allframe 0.1.12` and the SDK did not exist for them
to use. **Decision: `rv2-allsource` implements `allsource::EventFolder`. It defines no
projection trait of its own.**

Folders live in `crates/rv2-allsource/src/folders/{user.rs,post.rs}` — server-only, because
`allsource::Event` comes from the SDK which pulls `reqwest`. Read models (the `State` types)
live in `rv2-api-types` so the Dioxus apps can deserialize them without crossing the boundary.

```rust
// crates/rv2-allsource/src/folders/post.rs
use allsource::{Event, EventFolder};
use rv2_api_types::PostView;                 // WASM-safe read model
use rv2_events::{decode, DomainEvent};

#[derive(Default)]
pub struct PostFolder { inner: Option<PostView>, deleted: bool }

impl EventFolder for PostFolder {
    type State = PostView;

    fn apply(&mut self, event: &Event) -> bool {
        let Ok(ev) = decode(&envelope_from_sdk(event)) else {
            tracing::warn!(event_type = %event.event_type, id = %event.id, "undecodable event");
            return false;                     // logged, never silently swallowed
        };
        match ev {
            DomainEvent::PostCreated { id, author_id, title, content, occurred_at } => {
                self.inner = Some(PostView { id, author_id, title, content,
                                             created_at: occurred_at, updated_at: occurred_at });
                true
            }
            DomainEvent::PostEdited { occurred_at, title, content, .. } => {
                if let Some(p) = self.inner.as_mut() {
                    if let Some(t) = title   { p.title = t; }
                    if let Some(c) = content { p.content = c; }
                    p.updated_at = occurred_at;
                }
                true
            }
            DomainEvent::PostDeleted { .. } => { self.deleted = true; true }
            _ => false,
        }
    }

    fn finalize(self) -> Option<PostView> {
        if self.deleted { None } else { self.inner }
    }
}
```

### 4.2 Fold-on-read vs. continuous folding — when each is correct

| | Fold-on-read | Continuous (`ProjectionWorker`) |
|---|---|---|
| **Mechanism** | `QueryClient::query_and_fold::<F>` — one HTTP call, fold in the handler | WebSocket subscription + Core-tracked durable consumer, state held in memory |
| **Cost** | O(events in *this* stream) per request | O(events since last ack) at cold start, then ~0 |
| **Freshness** | Perfectly fresh, always | Sub-millisecond after Core broadcast (SDK README, "Performance cheatsheet") |
| **Use when** | The query is scoped to **one** `entity_id` and the stream is bounded — a single post, a single user profile, an auth entity | The read model spans **many** entities — the post list, the user directory, dashboard counters |

**The concrete rule for rust-v2:**

- `GET /posts/{id}`, `GET /me` → **fold-on-read**. `entity_id` is known, the stream is a
  handful of events. The vendored auth adapter already works this way: `get_latest_raw` fetches
  the whole entity stream with `("limit", "1000")` and takes the newest live payload,
  with a comment explaining that per-entity streams are tiny
  (`getformlab:crates/better-auth-allsource/src/client.rs:114-150`).
- `GET /posts` (the list) → **`ProjectionWorker`**. Folding on read would mean scanning every
  `content.post.*` event in the store on every page load.

`apps/api` starts exactly one worker per cross-entity read model at boot, holds the
`ProjectionHandle` in `AppState`, and blocks readiness until `handle.is_caught_up()`:

```rust
let posts = ProjectionWorker::<HashMap<Uuid, PostView>>::builder(core.clone())
    .name("posts_v1")                                  // durable consumer id — see §4.4
    .event_types(&["content.post.created", "content.post.edited", "content.post.deleted"])
    .reducer(|state, event: &Event| { /* delegates to PostFolder */ Ok(()) })
    .checkpoint_interval(100)
    .build()?;
let handle = posts.start().await?;
```

(Builder shape quoted from `all-source:sdks/rust/README.md`, "Building custom projections".)

### 4.3 Snapshotting

Two independent mechanisms; use both, for different things.

1. **Worker checkpoints (the hot path).** The worker registers a durable consumer with Core;
   the cursor is tracked **server-side** and `save_checkpoint` is an `ack_consumer` underneath
   (SDK README, "Lifecycle notes"). Cursors survive Core restarts via
   `_system.consumer.registered` / `_system.consumer.ack_updated` / `_system.consumer.deleted`
   system events replayed from the WAL at boot (`all-source:README.md`, v0.17.3 notes). This is
   what makes a worker restart O(events-since-ack) instead of O(total).
   `checkpoint_interval = 100`.

2. **Entity snapshots (the cold path).** `POST /api/v1/snapshots {"entity_id": …}` and
   `GET /api/v1/snapshots/{entity_id}/latest` (`all-source:docs/current/API_REFERENCE.md:173-199`).
   **Decision:** rust-v2 does **not** use these at launch. No entity in the seed slice
   accumulates enough events for a per-entity fold to be slow. Introduce them per-entity-type
   when a stream is measured above ~1,000 events, not before.

### 4.4 Read-model rebuild — what happens when a projection's shape changes

This is the operation people get wrong, so it is specified as a procedure rather than a
principle.

Because Core keys the consumer cursor by the worker's `name`, **the rename *is* the rebuild
trigger** — a new name has no cursor, so Core replays from zero.

1. Change the folder / read-model type. Bump the worker name: `.name("posts_v1")` →
   `.name("posts_v2")`.
2. Deploy with **both** workers running. `posts_v2` replays history from zero while `posts_v1`
   keeps serving reads. Replay of the whole store is the expensive step; it happens with the
   old model still live, so there is no read outage.
3. Wait for `handle_v2.is_caught_up()`. Compare: for a sample of entity ids, assert
   `v1.get(id)` and `v2.get(id)` agree modulo the intended change.
4. Flip the handler to read `posts_v2`. Deploy.
5. Delete the old worker's code and its persisted state:
   `DELETE /api/v1/projections/posts_v1/{entity_id}/state`
   (`all-source:docs/current/API_REFERENCE.md:264`), enumerated via
   `GET /api/v1/projections/posts_v1/state?limit=…&offset=…` (ibid.:226).

**Corollary that constrains the design:** because a rebuild replays the *entire* store, folders
must be **pure and total** — no clock reads, no network calls, no `unwrap()` on payload shape,
and `apply` must be idempotent for an event it has already seen. The SDK worker does per-entity
version dedup ("Events with `version ≤ last_applied` are skipped", SDK README) but that only
covers replay overlap; logical idempotence is the folder's job. getformlab tests exactly this —
re-emitting `CoachFollowUpScheduled` for the same id must not duplicate
(`getformlab:crates/jbt-allsource/src/projections.rs`, `follow_up` test). Copy that test habit.

---

## 5. Auth

### 5.1 The `better-auth-allsource` port

**The prompt's premise needs correcting, and the correction strengthens its conclusion.**

`better-auth-allsource` **is** published on crates.io — `0.14.12`, "Allsource DatabaseAdapter
for better-auth-rs — event-sourced auth persistence", repository `all-source-os/all-source`
(verified via `cargo info better-auth-allsource` this session). So the stated reason to vendor
("it is not published") is no longer true.

**But it still must be vendored, for a harder reason.** Every published version declares
`better-auth-core ^0.8`:

```
$ curl -s https://index.crates.io/be/tt/better-auth-allsource | …
0.14.5  [('better-auth-core', '^0.8')]
0.14.11 [('better-auth-core', '^0.8')]
0.14.12 [('better-auth-core', '^0.8')]
```

while `better-auth 0.10.0` requires `better-auth-core 0.10.0` (crates.io index; `better-auth`
has pinned `better-auth-core` to its own exact minor for every release from 0.5.0 to 0.10.0).
A resolution test in a scratch crate this session confirms the outcome:

```
$ cargo tree -i better-auth-core@0.8.0  --depth 1
better-auth-core v0.8.0
└── better-auth-allsource v0.14.12
$ cargo tree -i better-auth-core@0.10.0 --depth 1
better-auth-core v0.10.0
├── better-auth v0.10.0
└── better-auth-api v0.10.0
```

Two copies of `better-auth-core` in one tree means two *distinct* `UserOps` / `SessionOps` /
`DatabaseAdapter` traits. `AuthBuilder::database(AllsourceAuthAdapter)` would therefore fail
the trait bound — a type error that reads as nonsense ("expected `UserOps`, found `UserOps`")
and costs an afternoon to diagnose.

This is precisely why getformlab vendored it. Their vendored manifest says so in its own
description (`getformlab:crates/better-auth-allsource/Cargo.toml`):

```toml
[package]
name = "better-auth-allsource"
version = "0.14.5"
description = "Allsource DatabaseAdapter for better-auth-rs — vendored and bumped to better-auth-core 0.10"

[dependencies]
better-auth-core = "0.10"
```

**Decision (D9): vendor.** Copy `getformlab:crates/better-auth-allsource/` into
`rust-v2/crates/better-auth-allsource/` verbatim (4 files: `lib.rs` 28 lines, `adapter.rs`
1385 lines, `client.rs` 479 lines, `error.rs` 19 lines), add a `PROVENANCE.md` recording
source repo, commit sha, upstream crates.io version, and the reason. Path-depend on it from
`[workspace.dependencies]` exactly as getformlab does.

**Exit condition, so this doesn't become permanent by accident.** A CI job runs weekly:
fetch the latest `better-auth-allsource` from the index; if its `better-auth-core` requirement
matches the `better-auth` we pin, open an issue. At that point delete the vendor and switch the
`[workspace.dependencies]` line to a version dep. The vendored copy is a **version bridge**,
not a fork — do not add features to it.

**Alternatives rejected:**

| Alternative | Why rejected |
|---|---|
| Pin `better-auth 0.8.0` + published `better-auth-allsource 0.14.12` | Genuinely works and needs no vendoring. Rejected because it strands us two minors behind the reference implementation we are porting from, and puts the *auth* crate — the one with the security-relevant changelog — on the oldest thing that compiles. |
| Git submodule | There is no upstream repo to submodule. The adapter lives inside the `all-source` monorepo (`repository: github.com/all-source-os/all-source`); a submodule would drag the whole event store. |
| Extract to our own published crate | Adds a second repo and a release cadence before rust-v2 has a single user. Reconsider only if a third service needs the adapter. |
| Patch via `[patch.crates-io]` | Cannot help: the problem is a *semver-incompatible* dependency of the crate, not a bad version of the crate itself. |

### 5.2 How auth actually works, end to end

**Storage — users and sessions as events.** The adapter maps every better-auth entity to an
AllSource stream. From `getformlab:crates/better-auth-allsource/src/adapter.rs:20-27`:

> Each auth entity is stored as an Allsource event stream:
> - Entity ID pattern: `auth-{type}:{id}` (e.g., `auth-user:abc123`)
> - Each create/update appends a full-state snapshot event
> - Deletes append a `_deleted: true` marker
> - Reads fetch the latest event and deserialize the payload

Real signatures (same file, verified):

```rust
pub struct AllsourceAuthAdapter { client: AllsourceClient }

impl AllsourceAuthAdapter {
    /// - `core_url`:  Allsource Core URL (e.g., `http://localhost:3900`)
    /// - `query_url`: Allsource Query Service URL (e.g., `http://localhost:3902`)
    /// - `api_key`:   API key for authentication (e.g., `ask_...`)
    pub fn new(core_url: &str, query_url: &str, api_key: &str) -> Self;
}

#[async_trait]
impl SessionOps for AllsourceAuthAdapter {
    type Session = Session;

    async fn create_session(&self, input: CreateSession) -> AuthResult<Session> {
        let token = format!("session_{}", Uuid::new_v4());
        // …
        self.client
            .append_event(&session_entity(&token), "auth.session.created", payload)
            .await
            .map_err(AuthError::from)?;
        Ok(session)
    }
    async fn get_session(&self, token: &str) -> AuthResult<Option<Session>>;
    async fn delete_session(&self, token: &str) -> AuthResult<()>;   // appends "auth.session.deleted"
    // + get_user_sessions, update_session_expiry, delete_user_sessions,
    //   delete_expired_sessions, update_session_active_organization
}
```

and the low-level client (`getformlab:crates/better-auth-allsource/src/client.rs:68-76`):

```rust
pub async fn append_event(
    &self,
    entity_id: &str,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<(), AllsourceAuthError>;
```

with the "current state" rule that a naive implementation gets wrong (ibid.:33-50) — worth
quoting because it is the single most instructive bug in the reference codebase:

```rust
/// Given an entity's events in AllSource order (OLDEST-first), return the
/// current-state payload: the LAST event's payload, or `None` if the stream is
/// empty or the latest event is a delete tombstone (`_deleted: true`).
///
/// Regression guard: a previous version took `.first()` (with `limit=1`), which
/// returned the `*.created` event and silently dropped every later
/// `*.updated`/delete — breaking user role updates and session sign-out.
fn latest_live_payload(events: &[StoredEvent]) -> Option<&serde_json::Value> { … }
```

**Note the naming asymmetry, and accept it.** The adapter writes `auth.session.created` —
dotted, correct AllSource grammar — but it uses *full-state snapshot* events rather than
deltas, i.e. it is event-*storage*, not event-*sourcing*. That is fine: auth entities are
small, mutable, and read on every request. rust-v2 does **not** try to unify the auth streams
with `DomainEvent`; they are the adapter's private namespace (`auth-*` entity prefix, `auth.*`
wire prefix) and our projections never read them. `rv2-events` owns `identity.user.*`
separately — see §5.4 for how the two stay consistent.

**Construction (`apps/api/src/infrastructure/auth/better.rs`).** Ported from
`getformlab:apps/api/src/infrastructure/auth/better.rs`, whose structure we keep, including the
test/prod adapter split:

```rust
#[cfg(feature = "allsource-auth")]
pub type ApiAuthDb = AllsourceAuthAdapter;
#[cfg(not(feature = "allsource-auth"))]
pub type ApiAuthDb = MemoryDatabaseAdapter;   // fast, hermetic tests
pub type ApiBetterAuth = BetterAuth<ApiAuthDb>;

pub async fn build_auth(cfg: &ServerConfig) -> Arc<ApiBetterAuth> {
    let auth_cfg = AuthConfig::new(&cfg.jwt_secret)
        .base_url(base_url)                    // must include the `/auth` segment: the OAuth
                                               // plugin appends `/callback/{provider}`
        .password_min_length(8)
        .trusted_origin("http://localhost:4401")   // apps/web
        .trusted_origin("http://localhost:4402");  // apps/app
    // + CORS_ORIGINS-derived production origins
    // plugins: EmailPasswordPlugin, SessionManagementPlugin, OAuthPlugin(Google)
}
```

The `allsource-auth` cargo feature keeping `MemoryDatabaseAdapter` in tests is the reason
`apps/api`'s test suite does not need a running Core. Keep it.

**Validation on the Axum side.** Direct port of
`getformlab:apps/api/src/infrastructure/auth/middleware.rs`:

```rust
pub struct ExtractAuthUser(pub AuthUser);

impl FromRequestParts<Arc<AppState>> for ExtractAuthUser {
    type Rejection = Response;
    async fn from_request_parts(parts: &mut Parts, state: &Arc<AppState>)
        -> Result<Self, Self::Rejection>
    {
        let cookie_name = &state.auth.config().session.cookie_name;
        let token = extract_token(parts, cookie_name)          // Bearer header OR cookie
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing auth token").into_response())?;
        let session_manager = SessionManager::new(
            Arc::new(state.auth.config().clone()),
            state.auth.database().clone(),
        );
        let session = session_manager.get_session(&token).await …?;
        let user = state.auth.database().get_user_by_id(&session.user_id).await …?;
        Ok(ExtractAuthUser(AuthUser { id: user.id.parse()?, roles: Role::parse_set(…) }))
    }
}
```

Handlers take `ExtractAuthUser` as an argument; whole route trees are gated with
`require_authenticated` as a `middleware::from_fn_with_state` layer. `AuthUser` and `Role` live
in `rv2-shared`.

**Cost, stated plainly:** that is **two** HTTP round-trips to AllSource per authenticated
request (`get_session`, then `get_user_by_id`). At 11.9 µs Core read latency plus network this
is acceptable, but it is a per-request tax with no cache. Mitigation is in §9.

### 5.3 How a Dioxus client obtains, stores, and refreshes a session

**Decision (D17): HttpOnly cookie, not localStorage.** The WASM client never sees the session
token. This removes the entire XSS-token-theft class, which matters more in a SPA than in an
SSR app.

- **Obtain.** `POST {API}/auth/sign-in/email` (better-auth's own route, mounted under `/auth`)
  with `RequestCredentials::Include`. better-auth's response sets the session cookie.
- **Store.** The browser stores it. `rv2-client` sets
  `.credentials(web_sys::RequestCredentials::Include)` on every request. This is the same
  mechanism getformlab's Leptos client uses — `RequestCredentials` is in its `web-sys` feature
  list (`getformlab:apps/coach-web/Cargo.toml`).
- **Refresh.** better-auth's `SessionManagementPlugin` handles rolling expiry server-side
  (`update_session_expiry` → `auth.session.expiry_updated` event). The client does nothing.
- **Bootstrap on load.** `apps/app` calls `GET {API}/auth/get-session` once at mount inside a
  `use_resource`; `None` → redirect to `/login`. This replaces rust-v1's Next.js
  `proxy.ts` middleware redirect (§7).
- **Sign out.** `POST {API}/auth/sign-out`. The adapter appends `auth.session.deleted` and
  better-auth clears the cookie.

**CORS is load-bearing and easy to get wrong.** `apps/api` must send
`Access-Control-Allow-Credentials: true` and an explicit `Access-Control-Allow-Origin`
(never `*`, which is illegal with credentials), listing `http://localhost:4401`,
`http://localhost:4402`, and the production origins from `CORS_ORIGINS`. `cookie` must be in
`Access-Control-Allow-Headers`. getformlab does exactly this
(`getformlab:apps/api/src/main.rs:559-600`).

**Cookie domain scoping.** If `apps/api` and the SPAs are on different hosts in production, the
session cookie needs `SameSite=None; Secure` and a parent-domain scope. getformlab hit both
halves of this and documents the sharp edge (`apps/api/src/main.rs:195-200`): if login scopes
the cookie to the parent domain but `/auth/sign-out` clears only a host-scoped cookie, *the
user can never log out*. **Decision:** deploy `apps/api` and the SPAs under one parent domain
and apply the same `scope_session_cookies` layer to *both* set and clear.

### 5.4 OAuth

Google only at launch, replacing Supabase's `signInWithOAuth({provider: "google"})`
(`rust-v1:apps/app/src/components/google-signin.tsx`) one-for-one.

- `better_auth::plugins::oauth::{OAuthPlugin, OAuthProvider}`, configured when
  `ServerConfig.google_oauth` is `Some`.
- Callback lands on **`apps/api`** at `/auth/callback/google` — not on a frontend route.
  rust-v1's `apps/app/src/app/api/auth/callback/route.ts` disappears entirely.
- The API then redirects back to the SPA origin. The origin cannot be trusted from a query
  param; getformlab binds it in a short-TTL HMAC-signed `oauth_origin` cookie (their ADR-006,
  implemented in `apps/api/src/infrastructure/auth/oauth_glue.rs`, with `hmac` + `subtle` for
  constant-time comparison). **Port that pattern.** It is the difference between a working
  multi-origin OAuth flow and an open redirect.
- `redirect_uri` registered in Google Cloud Console must equal `{base_url}/callback/google`,
  where `base_url` **includes the `/auth` segment** — the plugin appends only
  `/callback/{provider}`.

**Role assignment.** Supabase had no roles. better-auth stores `user.role` as a string;
getformlab parses it as a comma-separated *set* (`Role::parse_set("coach,trainee")`). rust-v2
adopts the set model from day one even with a single role, because widening a scalar to a set
later means rewriting every stored user record.

---

## 6. Dioxus application architecture

Verified against the **current stable release, `dioxus 0.7.10`** (`cargo info dioxus@0.7.10`;
crates.io index shows `0.8.0-alpha.1` as newest and `0.7.10` as newest stable). `dioxus-cli`
0.7.10 exists and is the matching `dx`. **We do not pin the alpha.** Docs consulted:
`dioxuslabs.com/learn/0.7/**` and the 0.7 release post.

### 6.1 The fork: separate Axum service, not fullstack server functions

**Decision (D6): `apps/api` remains a standalone Axum service. Dioxus apps talk to it over
REST. Server functions are not used for business logic.**

Reasoning, in order of weight:

1. **better-auth needs to own an Axum router.** `better_auth::AxumIntegration` produces a
   router that getformlab mounts with `.nest("/auth", better_auth_router…)`
   (`getformlab:apps/api/src/main.rs:492`) and then wraps in layers for OAuth-origin capture
   and cookie scoping. Reproducing that inside `dioxus-server`'s hosting model is possible but
   is a novel integration nobody has proven.
2. **`allframe` is the beachhead and already works.** rust-v1's `apps/api` depends on
   `allframe 0.1` today (`apps/api/Cargo.toml`) and gets router, health, OpenAPI, and
   resilience from it. Subsuming the API into a Dioxus server binary discards that.
3. **One contract, many clients.** A documented REST + OpenAPI surface is consumable by a
   future Tauri or mobile client. getformlab's `trainee-app` (Tauri) is exactly that case.
   Server functions serialize over a Dioxus-internal protocol.
4. **It cuts the WASM boundary cleanly.** With no `fullstack` feature, `apps/app` never links
   `dioxus-server`, and the "does this crate reach `tokio::net`?" question has a trivial answer.
5. **`dioxus-server 0.7.10` depends on `axum ^0.8.4`** (crates.io index) — the same major as
   `allframe` and `better-auth`, so this is not a compatibility forced move. It is a
   deliberate architectural one, and it could be revisited without a version fight.

**Rejected:** a single fullstack crate exposing `#[get("/api/posts")]` server functions. The
0.7 macro syntax is genuinely nice —

```rust
#[get("/api/hello-world")]
async fn hello_world() -> Result<String> { Ok("Hello world!".to_string()) }
```

— but it puts server dependencies (AllSource client, better-auth, tokio) inside the same crate
as the WASM UI, cfg-gated per item. That is the configuration in which "the workspace becomes
unbuildable" happens, and it is a one-way door once handlers proliferate.

**Cost accepted:** we hand-write the client-side fetch layer (`rv2-client`) instead of getting
it generated. That is ~1 function per endpoint and it is the price of the boundary.

### 6.2 App split

| App | Kind | Dioxus features | Build | Replaces |
|---|---|---|---|---|
| `apps/app` | CSR SPA, authenticated dashboard | `["web", "router"]` | `dx serve --package app --platform web` / `dx bundle --package app --platform web --release` | rust-v1 `apps/app` (Next.js) |
| `apps/web` | Static marketing site | `["web", "router", "fullstack"]` + `server` feature | `dx bundle --package web --web --ssg` | rust-v1 `apps/web` (Next.js) |

**Why `apps/web` is different.** It is public and needs crawlable HTML; a CSR SPA gives search
engines an empty `<div>`. Dioxus 0.7 SSG works by running the app locally, asking it for a
sitemap, rendering each route, and caching the HTML to a `public/` directory. It needs
`ServeConfig::builder().incremental(IncrementalRendererConfig::new().static_dir(…))` and a
server function at endpoint `"static_routes"` returning `Route::static_routes()`. Output:
`dx bundle --web --ssg` → a `public/` folder deployable to any static host.

The `server` feature therefore exists in `apps/web` **for the build only**; nothing runs it in
production. This keeps "one server process" true.

`apps/web/Cargo.toml`:

```toml
[dependencies]
dioxus = { workspace = true, features = ["router", "fullstack"] }
rv2-ui = { path = "../../crates/rv2-ui" }

[features]
default = ["web"]
web    = ["dioxus/web"]
server = ["dioxus/server"]        # SSG build only
```

`apps/app/Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
dioxus       = { workspace = true, features = ["web", "router"] }
rv2-ui       = { path = "../../crates/rv2-ui" }
rv2-client   = { path = "../../crates/rv2-client" }
rv2-api-types= { path = "../../crates/rv2-api-types" }
rv2-domain   = { path = "../../crates/rv2-domain" }
```

Note: **no `fullstack`, no `server`, no `tokio`, no `reqwest`.**

### 6.3 Routing

`dioxus/router` (a feature of the `dioxus` crate itself in 0.7: `router = [dep:dioxus-router]`).

```rust
#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[layout(AppShell)]
        #[route("/")]
        Dashboard,
        #[route("/posts")]
        Posts,
        #[route("/posts/:id")]
        PostDetail { id: Uuid },
    #[end_layout]
    #[route("/login")]
    Login,
}
// rendered as `Router::<Route> {}`; navigation via `Link { to: Route::Posts, … }`
```

Route params use `:name`. Auth gating is a `use_effect` inside `AppShell` that redirects to
`Route::Login` when the session resource resolves to `None` — the direct analogue of rust-v1's
`proxy.ts` redirect, moved client-side.

### 6.4 State management

- **Local:** `use_signal`.
- **Shared / app-wide:** `use_context_provider` at `AppShell` for the session, plus 0.7's
  **`Stores`** primitive for nested reactive state where a single signal would over-invalidate.
- **Server data:** `use_resource` in `apps/app` (CSR — data is always fetched client-side);
  `use_loader` in `apps/web` where SSG needs the value at render time.
- **Explicitly not:** a Redux-shaped global store. There is no `react-query` equivalent and we
  do not build one; `use_resource` plus a `Signal<HashMap<Uuid, PostView>>` cache in context is
  sufficient at this scale. Revisit if cache invalidation across routes becomes a real problem.

### 6.5 The client crate (`rv2-client`)

WASM-safe, `gloo-net`-based, one function per endpoint, all sharing a base URL read from a
build-time env var.

```rust
// crates/rv2-client/src/lib.rs
use gloo_net::http::Request;
use rv2_api_types::{PostView, CreatePostRequest};

pub async fn list_posts() -> Result<Vec<PostView>, ApiError> {
    Request::get(&format!("{}/posts", api_base()))
        .credentials(web_sys::RequestCredentials::Include)   // session cookie rides along
        .send().await?
        .json().await
        .map_err(Into::into)
}

pub async fn create_post(req: &CreatePostRequest) -> Result<PostView, ApiError> { … }
```

`gloo-net 0.7` verified on crates.io this session. Chosen over `reqwest`'s wasm backend
because it is a thin `fetch` wrapper — materially smaller bundle — and because it is the path
getformlab's WASM frontends already run in production.

### 6.6 Assets and styling

- **Tailwind.** Dioxus 0.7's `dx` has automatic Tailwind detection (0.7 release post). Each app
  keeps `assets/tailwind.css`; `dx serve` runs the Tailwind build. A shared preset lives at
  `style/tailwind.config.base.js` at the repo root, mirroring getformlab's layout.
- **Static assets.** The `asset!()` macro (manganis, re-exported through `dioxus`'s `asset`
  feature, on by default via `lib`). 0.7 supports hashless `asset!()` usage.
- **Component kit.** `rv2-ui` — Dioxus components rewritten from `packages/ui`'s shadcn
  React components. This is a **rewrite, not a port**; there is no mechanical path from JSX to
  RSX. getformlab's `crates/jbt-ui` (button, card, input, label, toast, skeleton,
  empty_state, page_header, …) is the shape and scope to copy.

### 6.7 `dx` alongside `cargo`

They coexist because `dx` *is* a cargo driver: it invokes `cargo` with the right target,
features, and post-processing (wasm-bindgen, asset collection, Tailwind), then serves.

- `cargo build --workspace` builds everything **for the host target**, which for `apps/app`
  produces an `rlib`/`cdylib` nobody uses but which does typecheck. That is exactly what we
  want from `meta build` and CI: cheap, fast breakage detection.
- `cargo test --workspace` runs all non-WASM tests, including every folder and event test.
- `dx serve --package app --platform web` is the *only* way to get a running frontend.
- `dx bundle --package web --web --ssg --release` is the *only* way to produce the marketing
  site.
- `dx serve @client --package app @server --package api` is 0.7's multi-package syntax (0.7
  release post). We do **not** use it: our server is a plain Axum binary run by `bacon`, not a
  `dioxus-server` app, so `meta dev` runs `bacon` and `dx` as independent processes — the same
  arrangement getformlab uses with `bacon` + `trunk` (`getformlab:meta.toml`).

---

## 7. Supabase teardown

Every file in `rust-v1:packages/supabase/src/` and every call site of `@rust-v1/supabase`,
enumerated from the source. Nothing in that package is left without a named home or an explicit
open question.

| # | Current responsibility | Where it lives today | rust-v2 replacement |
|---|---|---|---|
| 1 | Browser Supabase client | `src/clients/client.ts` → `createBrowserClient` | **Deleted.** `rv2-client` (gloo-net) + HttpOnly session cookie. No client-side DB handle exists in rust-v2 by design. |
| 2 | SSR server client with cookie store | `src/clients/server.ts` → `createServerClient` + `next/headers` cookies | **Deleted.** `apps/api` reads the session cookie in `ExtractAuthUser`; all data access is server-side against AllSource. |
| 3 | Session refresh in middleware | `src/clients/middleware.ts` → `updateSession(request, response)`, calls `supabase.auth.getUser()` and rewrites cookies | **`better-auth` `SessionManagementPlugin`**, server-side rolling expiry (`update_session_expiry` → `auth.session.expiry_updated`). No middleware needed; the client never touches the cookie. |
| 4 | Route guard: unauth → `/login`, auth-on-login → `/dashboard` | `apps/app/src/proxy.ts:17-27` | **`AppShell` `use_effect`** in `apps/app`, driven by a `use_resource` over `GET /auth/get-session`. Same two redirects, client-side. |
| 5 | i18n middleware chaining | `apps/app/src/proxy.ts` (`next-international`) | **Open question (OQ-5).** No i18n crate chosen. rust-v1 ships `en`/`fr` locale files. Candidate: `dioxus-i18n` (unverified). Until decided, `apps/app` and `apps/web` ship **English only**. |
| 6 | `getUser()` | `src/queries/index.ts` → `supabase.auth.getUser()` | **`GET {API}/auth/get-session`** (better-auth), or `ExtractAuthUser` server-side. |
| 7 | `getPosts()` | `src/queries/index.ts` → `from("posts").select("*")` | **`GET {API}/posts`**, served from the `posts_v1` `ProjectionWorker` read model (§4.2). |
| 8 | `updateUser(userId, data)` | `src/mutations/index.ts` → `from("users").update()` | **`PATCH {API}/users/{id}`** → validates in `rv2-domain` → appends `identity.user.profile_updated`. |
| 9 | Generated DB types (`Tables<"posts">`, `TablesUpdate<"users">`, …) | `src/types/db.ts`, `src/types/index.ts` | **`rv2-api-types`** (hand-written Rust DTOs) + **`rv2-events`** (the write model). No codegen: AllSource has no schema to generate from. The types are now the source of truth, not a derivative of one. |
| 10 | Google OAuth initiation | `apps/app/src/components/google-signin.tsx` → `signInWithOAuth({provider:"google", redirectTo})` | **`GET {API}/auth/sign-in/social/google`** via `OAuthPlugin` (§5.4). |
| 11 | OAuth code exchange | `apps/app/src/app/api/auth/callback/route.ts` → `exchangeCodeForSession(code)` | **`{API}/auth/callback/google`**, handled by better-auth; origin recovered from a signed pending cookie. The Next.js route is deleted. |
| 12 | Sign out | `sign-out.tsx`, `user-avatar.tsx` → `supabase.auth.signOut()` | **`POST {API}/auth/sign-out`** → appends `auth.session.deleted`, clears the cookie. |
| 13 | Auth gate on server actions | `apps/app/src/actions/safe-action.ts` (`next-safe-action` + `getUser()` + rate limit) | **`ExtractAuthUser` extractor + `require_authenticated` layer** on `apps/api` routes; rate limiting via allframe's `rate-limit` feature. |
| 14 | Sentry↔Supabase tracing integration | `sentry.client.config.ts`, `sentry.server.config.ts` (`supabaseIntegration`) | **Dropped.** Replaced by `tracing` + OpenTelemetry on `apps/api` (allframe `otel` feature). No client-side Sentry SDK in the Dioxus apps at launch — **OQ-6**. |
| 15 | Row-level security | Supabase RLS policies (in the hosted project, not in this repo) | **Application-level authorization** in `apps/api` handlers, using `AuthUser.roles` + ownership checks against the read model. **This is a genuine downgrade in enforcement location** and is listed as a risk (R4). |
| 16 | Storage / buckets | — | **Not used.** `grep -rn "supabase.storage\|\.storage\."` over `apps/` and `packages/` returns **zero** hits. Nothing to replace. If media is added later: S3-compatible object storage via the `object_store` crate (getformlab's choice), with only the object key stored in an event. |
| 17 | Realtime subscriptions | — | **Not used.** No `.channel(` / `realtime` usage in the repo. Available if needed: AllSource WebSocket streaming (`GET /api/v1/events/stream`). |
| 18 | Env vars `NEXT_PUBLIC_SUPABASE_URL`, `NEXT_PUBLIC_SUPABASE_ANON_KEY`, `SUPABASE_SERVICE_KEY` | `apps/app/src/env.mjs:15,22-23,30-31,35` | **Replaced by** `ALLSOURCE_CORE_URL`, `ALLSOURCE_QUERY_URL`, `ALLSOURCE_API_KEY` (`ask_…`), `JWT_SECRET`, `GOOGLE_CLIENT_ID`/`_SECRET`, `CORS_ORIGINS`, `PUBLIC_API_URL` — all server-side only. **No AllSource credential is ever exposed to a browser**, unlike the Supabase anon key. |

**Bonus: what already isn't Supabase.** `apps/app` already calls the AllFrame API directly for
orders/products/shipping/metrics via `src/infrastructure/api/client.ts`
(`NEXT_PUBLIC_API_URL`, default `http://localhost:4400`). Those call sites map 1:1 onto
`rv2-client` functions and are the least risky part of the port.

---

## 8. Cutover plan

### 8.1 Fresh repo, not in-place — and why

**Decision (D1): a new repository, `rust-v2`.**

- **In-place would mean a long-lived broken `main`.** Deleting `packages/supabase` breaks
  `apps/app` immediately; the Dioxus replacement is weeks of work. Either `main` is red for
  weeks or a giant feature branch diverges — both are worse than a new repo.
- **The two build systems don't overlap.** rust-v2 has no `bun.lock`, `turbo.json`,
  `biome.json`, `package.json`, or `node_modules`. Keeping them around "just during migration"
  is exactly how a "temporary" polyglot repo becomes permanent.
- **The history that matters is small.** `tooling/meta` (which rust-v2 keeps verbatim) and
  `docs/` are worth carrying; the TypeScript history is not. Copy those two trees with their
  history via `git subtree`/`filter-repo` if desired, or just copy the files.
- **rust-v1 stays alive and deployable** during the whole migration. That is the property that
  makes this reversible.

**Not executed here.** Per the constraint, this document *recommends* the repo; it does not
create one, add a remote, or push. Prompt `002` scaffolds the workspace **inside this repo's
branch** so it can be reviewed as one PR; promoting it to a standalone repo is the user's call
afterwards.

### 8.2 What gets abandoned outright

| Abandoned | Reason |
|---|---|
| `apps/app`, `apps/web` (Next.js) | Rewritten in Dioxus. No incremental path from React to RSX. |
| `packages/supabase` | The point of the exercise. |
| `packages/jobs` (trigger.dev) | Contains exactly one task — `helloWorldTask`, which logs "Hello, world!" and waits 5 seconds (`packages/jobs/trigger/example.ts`). It is template scaffolding, not a workload. **Deleted, not replaced.** |
| Turborepo, Biome, bun, `sherif`, `tsconfig.json` | No TypeScript remains. `meta` + `cargo` + `clippy` + `rustfmt` replace all of it. |
| Supabase RLS policies | Enforcement moves into `apps/api`. See R4. |
| Supabase auth credentials | Password hashes do not migrate (§8.3). |
| `@supabase/sentry-js-integration` | No Supabase to instrument. |

### 8.3 Data migration: Postgres rows → events

This is a **translation**, not a copy. `tooling/pg2events` is a one-shot Rust binary
(`sqlx` or `tokio-postgres` → `allsource::CoreClient`), run once against a Supabase read
replica, idempotent, and deleted from the workspace after cutover.

**Ordering.** Users before posts (`posts.user_id` has an FK to `users.id`;
`packages/supabase/src/types/db.ts` declares `fk_posts_user`). The migrator processes
`users` fully, then `posts` ordered by `created_at ASC`.

**Mapping.**

| Postgres | Event | Stream (`entity_id`) |
|---|---|---|
| `users` row | `identity.user.registered` — `{id, email, full_name, avatar_url, occurred_at: created_at}` | `user:{id}` |
| `users.updated_at > created_at` | one `identity.user.profile_updated` at `updated_at` | `user:{id}` |
| `posts` row | `content.post.created` — `{id, author_id: user_id, title, content, occurred_at: created_at}` | `post:{id}` |
| `posts.updated_at > created_at` | one `content.post.edited` at `updated_at` | `post:{id}` |

**The trap this design already avoids (D13).** AllSource assigns the event `timestamp`
server-side at ingest ("Automatic UUID generation and timestamping",
`all-source:docs/current/EVENT_STORE_FEATURES.md:30`; and it rejects future timestamps, ibid.:56).
So a migrated post created in 2024 would carry a 2026 envelope timestamp. Because **every
payload carries its own `occurred_at`**, and because folders read domain time only from the
payload, the migration is lossless. Had we followed getformlab's pattern of taking
`event.timestamp` from the envelope inside projections
(`getformlab:crates/jbt-allsource/src/projections.rs`, `created_at: event.timestamp`), the
entire migrated corpus would have collapsed to a single instant. This is the concrete payoff of
D13.

**Provenance.** Every migrated event carries
`metadata: {"source": "supabase-migration", "migrated_at": "<iso>", "pg_table": "posts"}`.
That makes "which of these are real?" answerable forever, and makes a re-run detectable.

**Idempotence.** Before appending, the migrator queries
`GET /api/v1/events/query?entity_id=post:{id}` and skips non-empty streams. A partial run can
be resumed.

**Verification gate before cutover:** `SELECT count(*) FROM posts` must equal the size of the
`posts_v1` projection state, and a random sample of 50 rows must round-trip field-for-field.

**Credentials do not migrate.** Supabase stores password hashes in `auth.users`, reachable only
with the service role, and better-auth 0.10 has its own hashing scheme. **Decision:** users
re-register or sign in with Google (which re-links by email, so their `identity.user.*` history
survives). Whether a service-role export plus a better-auth custom verifier could preserve
passwords is **OQ-3** — do not assume it can.

### 8.4 Fate of the TypeScript support packages

| Package | What it does today | rust-v2 |
|---|---|---|
| `logger` | `pino` wrapper | **`tracing` + `tracing-subscriber`** with `env-filter`. Already in `apps/api`. Direct replacement. |
| `kv` | Upstash Redis + `@upstash/ratelimit`, used only by `safe-action.ts`'s rate limiter | **allframe's `rate-limit` feature** (verified present in `cargo info allframe`: `rate-limit = [allframe-core/rate-limit]`). Upstash is dropped; if a distributed limiter is later needed, allframe also offers `cache-redis`. |
| `react-query` | TanStack Query client | **`use_resource`** (+ `use_loader` for SSG). No third-party cache layer. |
| `ui` | shadcn/Tailwind React components | **`rv2-ui`**, Dioxus components, rewritten. Tailwind config carries over; JSX does not. |
| `email` | `react-email` + `@react-email/tailwind`; one template, `emails/welcome.tsx` | **`tera` templates** in `apps/api/src/infrastructure/email.rs`, mirroring getformlab (`tera = "1"` in `getformlab:apps/api/Cargo.toml`, alongside an `infrastructure/email.rs`). One template to port. |
| `analytics` | PostHog — `posthog-js` (client) + `posthog-node` (server), plus `@vercel/functions` | **Split.** Client-side: the PostHog JS snippet in each app's `index.html` — no Rust crate, no WASM bindings, and the same script PostHog ships. Server-side: product events become **AllSource events** (`analytics.*` namespace), which is the whole point of having an event store; export to PostHog later via a batch consumer if the product team needs it. `posthog-rs 0.23.2` exists but is unevaluated — deliberately **not** adopted (see OQ-7). |
| `jobs` | trigger.dev, one placeholder task | **Deleted** (§8.2). When real background work appears, the first candidate is a `tokio` interval task inside `apps/api` driven by an AllSource projection; `apalis` is a heavier option but its current release is `1.0.0-rc.9`, a release candidate, so it is not a launch dependency. |

### 8.5 Sequencing

Each phase ends with something demonstrable. Phases 1–4 are the scaffold prompt's territory.

| Phase | Work | Done when |
|---|---|---|
| **0** | This document reviewed and accepted | Decisions ratified |
| **1** | Workspace skeleton: `Cargo.toml`, toolchain, `meta.toml`, `docker-compose.yml` with Core + QS, empty crates | `cargo check --workspace` green; `meta dev` starts infra |
| **2** | `rv2-events` + wire mapping + golden corpus + round-trip tests | `cargo test -p rv2-events` green with a real bijection test |
| **3** | `rv2-allsource`: `CoreClient`/`QueryClient` wrappers, `PostFolder`, `UserFolder`, `posts_v1` worker | Integration test appends an event to a live Core and reads it back folded |
| **4** | Vendor `better-auth-allsource`; `apps/api` with `/auth/*` + `ExtractAuthUser` | Register → sign in → `GET /auth/get-session` returns a user, against a live Core |
| **5** | `apps/api` domain endpoints: posts CRUD, user profile | OpenAPI served; every endpoint has an integration test |
| **6** | `rv2-ui` + `rv2-client` + `apps/app`: login, dashboard, posts list/detail/create | `dx serve --package app` → full logged-in flow in a browser |
| **7** | `apps/web` SSG marketing site | `dx bundle --web --ssg` produces `public/` with real HTML in the source |
| **8** | `tooling/pg2events` + dry run against a Supabase replica | Row counts and a 50-row sample match the projection |
| **9** | Cutover: DNS, real migration run, rust-v1 frozen read-only | rust-v2 serving production; rust-v1 archived |

**Rollback.** Through phase 8, rollback is "keep using rust-v1" — it is untouched and
deployable. After phase 9, rollback means replaying AllSource events back into Postgres, which
nobody wants to write; so phase 9 is the point of no return and should not start until phase 8's
verification gate is green.

---

## 9. What is NOT event-sourced

A design that claims everything is events fails at implementation. Here is the boundary,
honestly drawn.

**First, a correction to the brief.** The premise that "AllSource's own README concedes that for
transactional OLTP, Postgres remains useful" **could not be verified**. I read
`all-source:README.md` in full this session; it says the opposite —
"No Postgres in the event path", "Core IS the database … Zero external dependencies", and the
Query Service is described as "Fully stateless, no PostgreSQL dependency". There is no
"when not to use" section. The honest position below is therefore **mine, argued from the
API surface**, not a quote from the vendor.

### 9.1 The real gap: no unique index, no cross-entity transaction

AllSource's write API is `POST /api/v1/events` with `{entity_id, event_type, payload}`. There is
no unique constraint, no `SELECT … FOR UPDATE`, and no multi-entity transaction. Concretely:

- **Email uniqueness at signup is a check-then-write race.** better-auth calls
  `get_user_by_email`, sees `None`, and appends `auth.user.created`. Two concurrent signups with
  the same email both see `None`. Postgres would have rejected the second with a unique index;
  AllSource cannot.
- **Any invariant spanning two entities** ("a user may own at most N posts") has the same shape.

**Mitigations, in the order we apply them:**

1. **Make the entity id derive from the unique key where possible.** Auth entities already do
   this for sessions (`auth-session:{token}`). Where the natural key is the identity, a
   duplicate write becomes an append to the *same* stream rather than a second entity — which
   the fold then resolves deterministically (last-write-wins).
2. **Optimistic concurrency control.** AllSource v0.14.0 lists "Optimistic concurrency control"
   (`all-source:README.md`, previous-releases list). If it exposes an expected-version guard on
   append, it turns lost updates into retryable conflicts. **I could not verify the wire
   parameter for it** — `docs/current/API_REFERENCE.md`'s `POST /api/v1/events` shows no
   `expected_version` field. **OQ-2.**
3. **Serialize registration.** Signup is low-volume. A single-writer path (one task, or a
   short-lived in-process lock keyed by lowercased email) closes the practical window on a
   single API instance. Multi-instance needs (2).
4. **Accept and detect.** A `duplicate_emails` check in the user-directory projection alarms if
   the race ever fires. The SDK even ships `QueryClient::detect_duplicates` for this shape of
   problem.

**Decision (D19): we do not introduce Postgres.** For a two-entity seed slice at this volume,
mitigations 1+3 are sufficient, and adding a second datastore would defeat the single-source-of-
truth goal that motivates rust-v2. **This decision has a trigger for revisiting:** if a workload
appears that needs a real uniqueness or balance invariant under concurrency — payments,
inventory, seat allocation — that workload gets Postgres, and this document gets amended. That
is a product decision, not an architectural surprise.

### 9.2 Things that are deliberately not events

| Workload | Why not events | Where it lives |
|---|---|---|
| **Rate-limit counters** | High write rate, TTL semantics, zero audit value. Writing them to an immutable log is pure cost. | In-memory via allframe `rate-limit`. |
| **Session-validation cache** | Every authenticated request does `get_session` + `get_user_by_id` = 2 AllSource round-trips (§5.2). | Short-TTL (≤30s) in-process cache keyed by session token; AllSource stays the source of truth. Sign-out invalidates the entry directly. |
| **Blob / media bytes** | Parquet + WAL are the wrong storage for megabytes of opaque binary. | S3-compatible object storage (`object_store`, as getformlab does). The **event stores only the key**. |
| **Full-text search / analytics aggregates** | These are derived indexes, not facts. | AllSource's own EventQL/DataFusion (`/api/v1/analytics/*`) and tantivy/BM25 — read paths over the same events, not new streams. |
| **Static assets, build artifacts** | Not domain facts. | CDN / static host. |
| **The `apps/web` marketing site content** | Prose, not state. | Rust source, SSG'd. |

### 9.3 Where event sourcing genuinely pays here

Stated for balance, so the boundary above reads as engineering rather than apology: posts and
user profiles get free audit history and time-travel (`?as_of=`); the auth store gets a complete
sign-in/sign-out trail without a separate audit table; and read models can be reshaped after the
fact by re-folding, which is exactly the flexibility a Postgres schema migration does not give
you.

---

## 10. Risks and open questions

### 10.1 Risks, ranked

**R1 — Dioxus is the least-charted decision, and there is no reference implementation.**
Both reference repos use Leptos 0.8. Dioxus 0.7's fullstack layer was substantially rewritten
in the 0.7 release (new `#[get]`/`#[post]` macros, `use_loader`, new server architecture), which
means most Dioxus material online predates it. Expect the SSG path in `apps/web` in particular
to be under-documented.
*Mitigation:* build `apps/app` (plain CSR — the best-trodden Dioxus path) **first** and prove
the full logged-in flow before starting `apps/web`. If SSG proves painful, `apps/web` degrades
to a CSR SPA and loses SEO — a contained, reversible loss, since it shares no code with the
dashboard beyond `rv2-ui`. **Do not** pin `0.8.0-alpha.1` to chase fixes; alpha churn on the
least-charted decision compounds the risk.

**R2 — Event schema mistakes are expensive after data exists.**
Once events are written they must deserialize forever (D12).
*Mitigation:* the golden corpus test (§3.3) plus the bijection test (§3.2) are cheap and both
fail at the introducing commit. Additionally: keep the seed slice **small**. Five variants over
two entities is deliberately less than we could model. Add events after the read paths are
proven, not before.

**R3 — The vendored `better-auth-allsource` becomes an unmaintained fork.**
1,911 lines of security-relevant code with no upstream sync.
*Mitigation:* the weekly index-check job (§5.1), a `PROVENANCE.md` recording exactly what was
copied, and a hard rule that the vendored crate takes **bug-for-bug ports only**, never new
features. If we need behaviour it does not have, that behaviour goes in `apps/api`.

**R4 — Authorization moves from the database to the application.**
Supabase RLS enforced access control inside Postgres, below the application. In rust-v2 an
`apps/api` handler that forgets an ownership check leaks data, with nothing behind it.
*Mitigation:* every read handler takes `ExtractAuthUser` (not `Option<…>`) so "unauthenticated"
is a type error rather than an omission; ownership checks live in `rv2-domain` as named
functions (`can_edit_post(user, post) -> bool`) so they are unit-testable and greppable; and
every endpoint gets an integration test that asserts a *different* user gets 403. This is the
risk most likely to produce a real incident — treat the test as non-optional.

**R5 — Session validation costs two AllSource round-trips per request.**
No caching in the reference implementation.
*Mitigation:* the ≤30s session cache (§9.2). Measure p99 on the authenticated path in phase 5;
if it is unacceptable, the next step is a signed JWT carrying `user_id` + roles with AllSource
consulted only on refresh — a bigger change, so measure first.

**R6 — Port drift between the AllSource docs and reality.**
Three sources give three different port pairs for the same services: the README says Core
`:3900` / Control Plane `:3901` / QS `:3902`; `docs/allsource-qs-api-reference.md:7-9` says
`:3280` and `:3283` ("maps to container port 3902"); getformlab's client defaults to `:3854`
and `:3855` (`getformlab:crates/jbt-allsource/src/client.rs:14-17`).
*Mitigation:* **never hard-code a port.** `ALLSOURCE_CORE_URL` and `ALLSOURCE_QUERY_URL` are
required env vars with no defaults, validated at boot by `ServerConfig::from_env`, and the
`docker-compose.yml` is the single place that fixes local values. Note also getformlab's
`AllSourceConfig::new` derives the query URL by string-replacing `:3854`→`:3855` — a footgun we
do not copy.

**R7 — Duplicate `reqwest` in the dependency tree.**
Verified this session in a scratch resolution:

```
reqwest v0.13.4  └── allsource v0.23.0
reqwest v0.12.28 ├── better-auth-api v0.10.0
                 └── dioxus-fullstack v0.7.10
```

Two TLS stacks and two HTTP clients compiled into one binary: slower builds, bigger binary,
two CVE surfaces. It **does** resolve and build.
*Mitigation:* accept for now; `deny.toml` records it as a **known, allowed** duplicate so a
*new* duplicate still fails CI. Revisit when `better-auth` moves to `reqwest 0.13`.
(Good news from the same test: **`axum` resolves to a single version, 0.8.9**, shared by
`allframe`, `better-auth`, and `dioxus-server`; and `tokio` is single-versioned too.)

**R8 — Two people must learn AllSource *and* Dioxus *and* event sourcing at once.**
*Mitigation:* the phase sequencing (§8.5) deliberately front-loads AllSource (phases 2–5) and
defers Dioxus (phases 6–7), so only one novel thing is in flight at a time.

### 10.2 Open questions

These are unresolved. **They are not implicit decisions** — prompt `002` must either resolve
them or route around them, and must not invent an answer.

- **OQ-1 — `getrandom 0.4` on `wasm32-unknown-unknown`.** getformlab's recipe
  (`getrandom 0.2` + `features = ["js"]`) is two majors stale; current is `0.4.3` with a
  `wasm_js` feature that also wants a build cfg. D19's "no UUID generation in WASM crates"
  avoids needing it — but if a WASM crate ever pulls `getrandom` transitively, the exact
  incantation is unverified.
- **OQ-2 — Optimistic concurrency control on append.** The README credits v0.14.0 with it, but
  `docs/current/API_REFERENCE.md`'s `POST /api/v1/events` documents no `expected_version`
  parameter and the `allsource` SDK's `IngestEventInput` (as shown in its README) has only
  `{event_type, entity_id, payload, metadata}`. If OCC exists on the wire, §9.1 mitigation (2)
  becomes available. **Verify against `allsource` 0.23's `types.rs` or Core's handler before
  relying on it.**
- **OQ-3 — Supabase password-hash migration.** Whether `auth.users` password hashes are
  exportable with the service role, and whether better-auth 0.10 can be given a custom verifier
  for them. Assumed **no** (D21). If yes, the cutover becomes materially less disruptive.
- **OQ-4 — AllSource operational ownership.** Self-hosted (docker-compose / Fly / K8s via the
  repo's Helm charts) or the hosted `api.all-source.xyz`? This changes backup/DR, the
  `CoreClient` vs `QueryClient` split (the SDK is explicit that "Core should never be on public
  DNS"), and cost. **Not an engineering decision — needs the user.**
- **OQ-5 — Internationalisation.** rust-v1 ships `en` + `fr` via `next-international`. No Rust
  i18n crate was evaluated this session. Until one is chosen, rust-v2 is English-only; that is a
  **product regression** and needs sign-off.
- **OQ-6 — Frontend error reporting.** rust-v1 uses Sentry in both browser and server. The
  server side maps onto `tracing`/OTel cleanly; a WASM Sentry story was not researched. Launch
  without it, or research `sentry-rust`'s wasm support.
- **OQ-7 — Server-side product analytics.** `posthog-rs 0.23.2` exists (verified on crates.io)
  but was not evaluated for maturity, batching, or async-runtime fit. D20 routes around it by
  emitting AllSource events; if the product team needs PostHog dashboards on day one, this needs
  a real evaluation.
- **OQ-8 — `apps/api` migration path from allframe's own server.** rust-v1's `apps/api` builds
  a `hyper-util` server directly with allframe's router; getformlab uses plain `axum::serve`
  with allframe as a library. Which shape rust-v2's `main.rs` takes was not settled — it is a
  30-line decision but it should be made explicitly in phase 4, not drifted into.
- **OQ-9 — `allframe`'s `cqrs-allsource` feature.** `cargo info allframe` shows
  `cqrs-allsource = [allframe-core/cqrs-allsource]`, but the all-frame README does not document
  it and I did not read its source. D5 sidesteps it in favour of the `allsource` SDK. If it
  turns out to be a first-class, maintained integration, D5 deserves a second look.
- **OQ-10 — Dioxus SSG mechanics in a workspace.** `dx bundle --web --ssg` and the
  `IncrementalRendererConfig` + `static_routes` server-function requirement are documented, but
  I did not verify the exact invocation **with `--package`** in a multi-app workspace, nor
  whether the SSG server function coexists with an app that otherwise defines none.

---

## Appendix A — Version verification log

Every version below was checked **this session** with `cargo info <crate>`, the crates.io sparse
index, or a real `cargo generate-lockfile` resolution. None is from memory.

| Crate | Pinned | Verified as | Method / note |
|---|---|---|---|
| `dioxus` | **0.7.10** | latest **stable** | `cargo info dioxus` reports `0.8.0-alpha.1` as newest; sparse index confirms `0.7.10` is the newest non-prerelease. `rust-version: 1.83.0`. Alpha deliberately not pinned. |
| `dioxus-cli` (`dx`) | **0.7.10** | exists | `cargo info dioxus-cli@0.7.10`. Latest overall is `0.8.0-alpha.1`. |
| `allsource` (SDK) | **0.23.0** | latest | `cargo info allsource`. Features: `default = [rustls, projection-worker]`, plus `ws`, `native-tls`. **Not mentioned in the brief — this is a new finding.** |
| `allframe` | **0.1.28** | latest | `cargo info allframe`. `rust-version: 1.89`. Confirms `rate-limit`, `resilience` (via core), `cqrs-allsource`, `cache-redis` features exist. |
| `allframe-core` | **0.1.28** | latest | `cargo info allframe-core`. |
| `allsource-core` | 0.23.0 | latest | `cargo info allsource-core`, `rust-version: 1.92`. **Not a rust-v2 dependency** (embedded mode only; longhand pins 0.20.1). |
| `better-auth` | **0.10.0** | latest | `cargo info better-auth`. Requires `better-auth-core ^0.10.0`, `better-auth-api ^0.10.0`. |
| `better-auth-core` | **0.10.0** | latest | `cargo info better-auth-core`. |
| `better-auth-allsource` | **vendored** | crates.io has **0.14.12** | `cargo info better-auth-allsource` — **contradicts the brief's "NOT published"**. Sparse index shows 0.14.5/0.14.11/0.14.12 all declare `better-auth-core ^0.8` → incompatible with `better-auth 0.10`. |
| `gloo-net` | **0.7** | latest 0.7.0 | `cargo info gloo-net`. getformlab is on 0.6. |
| `axum` | **0.8** | resolves to 0.8.9 | `cargo generate-lockfile` — **one** version across allframe, better-auth, and `dioxus-server` (which requires `axum ^0.8.4`). |
| `reqwest` | 0.13 (direct) | **two** in tree | `cargo tree -i`: 0.13.4 ← `allsource`; 0.12.28 ← `better-auth-api`, `dioxus-fullstack`. See R7. |
| `tokio` | 1.x | single version | `grep -c '^name = "tokio"' Cargo.lock` → 1. |
| `getrandom` | n/a | latest **0.4.3** | `cargo info getrandom`; `wasm_js` feature (renamed from 0.2's `js`). See OQ-1. |
| `posthog-rs` | not used | 0.23.2 exists | `cargo info posthog-rs`. Unevaluated — OQ-7. |
| `apalis` | not used | 1.0.0-rc.9 | `cargo info apalis`. Release candidate → not a launch dependency. |
| Local toolchain | — | `rustc 1.97.1` | `rustc --version` (2026-07-14). |

**Full-set resolution proof.** A scratch crate with `allsource 0.23.0` + `allframe 0.1.28` +
`allframe-core 0.1.28` + `better-auth 0.10.0` + `better-auth-core 0.10.0` + `axum 0.8` +
`dioxus 0.7.10 [fullstack, router, server]` resolved cleanly:
`Locking 339 packages to latest Rust 1.97.1 compatible versions`. **All from crates.io — no
custom registry configuration is required.**

## Appendix B — Source index

Every claim in this document traces to one of these. Prompt `002` should follow them.

**AllSource** (`github.com/all-source-os/all-source`, read via `gh api`)
- `README.md` — three tiers and ports (:3900 / :3901 / :3902), WAL+Parquet+DashMap, "No Postgres
  in the event path", QS "fold-on-read and continuous folding via PubSub", v0.17.3 WAL-backed
  consumer cursors and `_system.consumer.*` events, v0.14.0 optimistic concurrency control.
- `docs/current/API_REFERENCE.md` — `POST /api/v1/events` (:88), `GET /api/v1/events/query`
  (:114), entity state + `?as_of=` time travel (:152), snapshots (:173-199), projection state
  CRUD + `entity_id_prefix` paging (:205-269), schema registry (:282-352), replay, pipelines,
  analytics, compaction.
- `docs/current/EVENT_STORE_FEATURES.md` — event structure (:39-49), **"Event type format
  validation (lowercase, dot-notation)"** (:53), projection types and lifecycle (:113-168),
  snapshot structure (:213-241).
- `docs/allsource-qs-api-reference.md` — write schema with `"lowercase dot-notation"` (:18),
  **"`entity_id` IS the stream ID. There is no separate `stream_id` field"** (:26), QS REST
  endpoints (:34-39), port mapping 3280/3283 → 3900/3902 (:7-9).
- `sdks/rust/README.md` — `CoreClient` vs `QueryClient` decision table, `ProjectionWorker`
  builder example, lifecycle (checkpoint/reconnect/dedup), API-surface table, `ask_` API keys.
- `sdks/rust/src/lib.rs` — public exports: `CoreClient`, `QueryClient`, `EventFolder`,
  `fold_events`, `normalize_event_type`, `ProjectionWorker`, `ProjectionHandle`.
- `sdks/rust/src/fold.rs` — the `EventFolder` trait, quoted verbatim in §4.1.
- `sdks/rust/src/normalize.rs` — `normalize_event_type` and its lossiness, §3.2.
- `sdks/rust/src/types.rs:21-33` — the `Event` struct.

**getformlab** (`get-form-lab/getformlab`, **private** — read via
`gh api repos/.../contents/<path> --jq '.content' | base64 -d`)
- `Cargo.toml` — workspace deps; the `better-auth-allsource = { path = … }` line.
- `meta.toml` — bacon + trunk + cargo + tauri tool declarations; per-app task shape.
- `CLAUDE.md` — repo layout, port allocation (:3850 api, :3852/:3853 web, :3854/:3855 AllSource).
- `crates/jbt-events/src/events/mod.rs` — `EventEnvelope` (:46-54), `#[serde(tag="type")]
  DomainEvent` (:56+), additive `#[serde(default)]` evolution comments (:105-115).
- `crates/jbt-allsource/src/client.rs` — `AllSourceConfig`, `AllSourceClient::append_event`,
  `read_stream`, `query_events`, `IngestEvent{entity_id,event_type,payload}`, retry policy.
- `crates/jbt-allsource/src/projections.rs` — `trait Projection` (:14-16), **`decode_event`
  (:18-42)** — the naming-reconciliation source, quoted in §3.2 — `UserProjection` (:273-329).
- `crates/better-auth-allsource/Cargo.toml` — `version = "0.14.5"`, "vendored and bumped to
  better-auth-core 0.10", `rustls`/`native-tls` features.
- `crates/better-auth-allsource/src/adapter.rs` — entity-id scheme (:20-27, :59-90),
  `AllsourceAuthAdapter::new` (:38), the `UserOps`/`SessionOps`/`AccountOps`/`VerificationOps`/
  `OrganizationOps`/`MemberOps`/`InvitationOps`/`TwoFactorOps` impls, `"auth.session.created"`
  (:341), `"auth.session.expiry_updated"` (:384), `"auth.session.deleted"` (:399).
- `crates/better-auth-allsource/src/client.rs` — `append_event` (:68), `get_latest_raw` and the
  oldest-first/`.last()` regression note (:33-50, :114-150), `query_all`, `find_all_by_field`.
- `apps/api/src/infrastructure/auth/better.rs` — `build_auth`, `ApiAuthDb` cfg split, trusted
  origins, OAuth plugin wiring.
- `apps/api/src/infrastructure/auth/middleware.rs` — `ExtractAuthUser`, `extract_token`
  (cookie or Bearer), `require_authenticated`/`require_coach`/`require_trainee`.
- `apps/api/src/main.rs` — better-auth router nesting (:492), cookie scoping (:195-202), OAuth
  pending-origin cookie (:632-800), CORS (:559-600).
- `apps/coach-web/Cargo.toml` — the WASM dependency set: `gloo-net`, `web-sys` feature list
  incl. `RequestCredentials`, and `chrono` with `default-features = false`.
- `crates/jbt-domain/Cargo.toml`, `crates/jbt-api-types/Cargo.toml` — the
  `[target.'cfg(target_arch = "wasm32")'.dependencies]` pattern.

**longhand** (`~/Projects/fractional-projects/longhand-main`, local)
- `Cargo.toml:26-36` — `allframe-core 0.1.28 ["cqrs","security"]`, `allframe-macros`,
  `allframe-tauri`, `allsource-core 0.20.1 ["embedded","embedded-projections"]`.
- `crates/longhand-allsource/Cargo.toml` — `prime-full`/`prime-recall` feature gating.
- `crates/longhand-allsource/src/lib.rs` — re-exports
  `allsource_core::application::services::projection::Projection` — **the embedded-mode trait,
  a different type from the SDK's `EventFolder`**. This is the mode-specific seam D3/D4 avoid.
- `crates/longhand-allsource/src/events.rs` — ~400 `pub const X: &str = "domain.action"`
  constants. Confirms the dotted convention independently, and is the pattern rust-v2 replaces
  with a typed `event_type()` mapping (D11).

**Dioxus** (official docs, `dioxuslabs.com/learn/0.7/**` + the 0.7 release post)
- `/learn/0.7/essentials/fullstack/project_setup` — the `web`/`server` feature split and
  optional server-only deps.
- `/learn/0.7/essentials/fullstack/server_functions` — `#[get]`/`#[post]`/`#[put]`/`#[delete]`/
  `#[patch]`, async + `Result<T,E>` requirements.
- `/learn/0.7/essentials/fullstack/static_site_generation` — `IncrementalRendererConfig`,
  `static_routes`, `dx bundle --web --ssg`.
- `/learn/0.7/essentials/router/` — `Routable` derive, `#[route("/user/:id")]`, `Router::<Route>{}`,
  `Link`.
- `/blog/release-070/` — `dx serve @client --package xyz @server --package xyz`, Stores,
  subsecond hot-patching, `#[wasm_split]`, automatic Tailwind detection.

**rust-v1** (this repo)
- `apps/api/Cargo.toml` — the `allframe 0.1` beachhead.
- `meta.toml`, `tooling/meta/src/config.rs`, `tooling/meta/src/execution/mod.rs` — meta's
  generic tool/project model.
- `packages/supabase/src/**` — every responsibility in the §7 teardown table.
- `apps/app/src/proxy.ts`, `.../api/auth/callback/route.ts`, `.../components/{google-signin,
  sign-out,user-avatar}.tsx`, `.../actions/safe-action.ts`, `.../infrastructure/api/client.ts`,
  `src/env.mjs` — the call sites.
- `packages/{analytics,email,kv,logger,jobs,react-query,ui}/package.json` — §8.4.

---

## Appendix C — Implementation notes (from the `002` scaffold)

**Written after building the workspace, 2026-08-11.** The scaffold lives at
`~/Projects/open-source/rust-v2` as a new local git repo (D1), one commit,
no remote.

This section records where reality diverged from the design above. It is
deliberately specific: each item says what the doc said, what was actually
found, and what changed. Where the doc was right, nothing is recorded — the
absence of an entry is the signal.

### C.1 What the build proved

The vertical slice is **real and was executed**, twice: once with the
in-memory auth adapter and once with `--features allsource-auth` so the
vendored `better-auth-allsource` was exercised too. `POST /posts` →
`content.post.created` in Core → fold → `GET /posts/{id}` and `GET /posts` →
rendered in the Dioxus dashboard, including a post written *from the browser
form* and then read back out of Core as a raw event.

`cargo build --workspace`, `cargo test --workspace` (155 tests),
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --
--check`, `cargo audit --deny warnings`, the `wasm32-unknown-unknown`
cross-compile of all seven WASM-facing crates, and `meta doctor` / `meta build`
/ `meta test` all pass.

### C.2 Corrections to the design

**C.2-a — `docker-compose.yml` is not the way to get a Core, and Docker is not
required at all.** §1.1 and the prerequisites assumed a compose stack.
`ghcr.io/all-source-os/allsource-core:latest` returns `denied` to an
authenticated pull, and `ghcr.io/all-source-os/chronos-core:latest` returns
`manifest unknown`. Neither published image was reachable.

`allsource-core 0.23.0` is on crates.io **with a `server` binary**, so
`cargo install allsource-core` gives a working Core with no Docker, no
registry credentials, and no compose file. That is now the documented path,
in the README, in `meta.toml`, and in CI. `docker-compose.yml` is kept for
when the images become pullable. *(Note: the installed binary reports
`v0.22.0` at startup despite being crate 0.23.0 — an upstream version-string
mismatch, harmless but worth knowing when reading logs.)*

**C.2-b — the `allsource` SDK normalizes event types on ingest
unconditionally, and D11 survives only by luck.** §3.2 rejects
`normalize_event_type` as a schema and specifies an explicit mapping instead.
But `CoreClient::ingest_event` runs `input.event_type =
normalize_event_type(&input.event_type)` on **every** append
(`allsource-0.23.0/src/client.rs`). There is no way to opt out.

We are safe because our grammar is a **fixed point** of that function: a type
containing a `.` is only lowercased, and ours are already lowercase dotted.
That is a property, not a coincidence to be assumed — so
`EventWriter::append` now checks it per-append and **refuses** to write a type
the normalizer would rewrite, and `writer.rs` has a test asserting the
identity holds for every entry in `ALL_WIRE_TYPES`. D11 stands; it just needed
a guard it did not know it needed.

**C.2-c — `Option<Option<T>>` does not round-trip through serde, so "clear this
field" silently became "leave it alone".** §3.1 specifies sparse patches as
`#[serde(default)] full_name: Option<Option<String>>` with "absent = unchanged,
explicit `null` = cleared". serde's derived impl maps a JSON `null` onto the
**outer** `None`, so `Some(None)` serializes to `null` and deserializes back as
`None`. The clear is lost. Caught by a folder test against a real payload, not
by inspection.

Both `rv2-events::DomainEvent::UserProfileUpdated` and
`rv2-api-types::UpdateProfileRequest` now carry all three of
`#[serde(default, skip_serializing_if = "Option::is_none", deserialize_with =
"double_option")]`. All three are load-bearing: without `skip_serializing_if`
both states serialize identically; without `deserialize_with` a present `null`
collapses; without `default` an absent key errors. **Any future
`Option<Option<_>>` field in this codebase needs the same three.**

**C.2-d — `ProjectionWorker` does not hydrate its state, so a restart silently
serves a partial read model.** §4.3 says worker checkpoints make a restart
"O(events-since-ack) instead of O(total)". True of the *replay cost*; not true
of *correctness*. `ProjectionWorker::start` registers the consumer and streams
from the server-side cursor into a freshly `Default`-constructed in-memory
state. Everything folded before the last ack is gone. Observed live: after a
restart `GET /posts` returned 2 of 4 posts, with no error anywhere.

Fixed with the mechanism Core already provides, wired in both directions:
`state_flush_entities` pushes each entity's state to Core's projection KV as
the worker folds, and `get_projection_state_summary` reads it back at boot
before the stream attaches. Sound because the flushed state is always at or
before the cursor, the worker replays forward from the cursor, and the reducer
is idempotent. **§4.3 should be amended:** a worker checkpoint alone is not a
durable read model, and any new `ProjectionWorker` in this codebase must wire
flush + hydrate together or it will have the same bug.

**C.2-e — D5's allframe role is wrong about what allframe is.** D5 reads
"`allframe 0.1.28` for router / health / openapi / resilience / rate-limit".
allframe is a **complete framework** with its own `App::new().route(…).run()`
and a built-in Hyper server, not a set of axum-compatible layers. Adopting its
server would mean re-hosting better-auth's axum `Router` inside it — a novel
integration nobody has proven, and precisely the risk D6 avoids elsewhere.

`apps/api` is therefore plain `axum::serve` with **allframe used as a
library** — `allframe::resilience::KeyedRateLimiter` for the D20 rate limiter.
Health is a plain handler; OpenAPI is a hand-written served document (allframe's
generator keys off *its* handler macros). **This resolves OQ-8** in favour of
getformlab's arrangement.

**C.2-f — `GET /auth/get-session` cannot be the client's session bootstrap.**
§5.3 names it. The route works, but it answers with better-auth's own envelope
(`{"session": …, "user": …}`, camelCase, better-auth's `User` shape), which
does not deserialize into `SessionView`. Worse, it fails as a *decode* error
rather than a 401, so the shell neither showed a session nor redirected — a
blank authenticated page, forever. Observed in a browser.

`rv2-client::get_session` now calls **`GET /me`**, `apps/api`'s normalized
projection of the same session, which answers 401 when absent. better-auth's
response shape stays an implementation detail of the auth router instead of
part of the client contract.

### C.3 Version corrections to Appendix A

| Crate | Doc says | Actual | Note |
|---|---|---|---|
| `axum-extra` | `0.10` | **`0.12.6`** | 0.12.6 requires `axum ^0.8.9`, so it is the correct pairing for our `axum 0.8`. |
| `tower-http` | `0.6` | `0.6.11` used; `0.7.0` exists | Stayed on 0.6 to avoid a second copy alongside allframe's. |
| `reqwest` TLS feature | `rustls-tls` | **`rustls`** | Renamed in reqwest 0.13. `rustls-tls` does not resolve and fails the build. |
| `tera` | `1` | 1.20.1 (2.1.1 exists) | Not yet used — email is not implemented in the scaffold. |
| `uuid` | `["v4","serde"]` workspace-wide | `["serde"]` workspace-wide | `v4` is opted into per-crate on the server side only, which is what actually keeps `getrandom` out of the WASM tree. Stronger than the doc's version. |
| `chrono` | `["serde"]` with default features | `default-features = false, features = ["alloc","serde"]` workspace-wide | Same reasoning: the WASM-safe form is the default and server crates opt into `clock`. Inverts the doc's polarity, deliberately. |

Also: `dioxus` is declared `default-features = false` at the workspace level,
because cargo forbids a member from overriding an inherited
`default-features = true` and `rv2-ui` needs the renderer-agnostic `lib` set.

### C.4 Smaller deviations

- **`rustfmt.toml` dropped nine nightly-gated options** (`imports_granularity`,
  `group_imports`, `wrap_comments`, `format_strings`, `normalize_comments`,
  `normalize_doc_attributes`, `comment_width`, `format_macro_matchers`,
  `format_macro_bodies`). rust-v1 sets them; on the pinned stable toolchain
  rustfmt ignores every one and prints a warning **per file**, so
  `cargo fmt --all -- --check` emitted hundreds of lines of noise. Keeping them
  would also make stable and nightly formatting differ, turning `--check` into a
  coin flip.
- **`cargo audit` needs `.cargo/audit.toml`.** CI runs `--deny warnings`, which
  is the right strictness given rust-v1's six-unnoticed-advisories lesson — but
  the tree carries four unavoidable transitive warnings (`backoff` and `instant`
  via allframe, `proc-macro-error2` via better-auth's `validator`, and an
  unsound `lru` via `dioxus-server`). Each is listed with what it is, why it is
  reachable, the real risk, and what removes it. Without that file someone drops
  the flag and the next *real* vulnerability sails through.
- **The "no Supabase" check is not a bare `grep -ri supabase`.** That grep is
  non-empty and always will be: `tooling/pg2events` exists to read Supabase, and
  several modules carry a comment naming the §7 responsibility they replace.
  Deleting those comments removes the explanation, not the dependency. CI
  asserts the real invariant instead — no dependency, no client SDK, no
  credential variable, no `*.supabase.co` host — plus no `package.json` /
  `bun.lock` / `tsconfig.json` / `turbo.json`.
- **`apps/app` is `lib` + `bin`, not just `cdylib`+`rlib`.** `dx serve` builds
  the bin target; the lib exists so routes and views are unit-testable.
- **`Dioxus.toml` has no `[web.resource]` block.** Dioxus 0.7 requires a `dev`
  field there, and stylesheets are injected from Rust via `asset!()` +
  `document::Link` anyway — declaring both loads them twice.
- **better-auth's `User.id` is a `String`, not a UUID.** Our domain streams are
  `user:<uuid>`, so `apps/api` maps it with a deterministic UUIDv5 hash
  (identity when it already parses as a UUID). Parsing-or-failing would 500 on a
  valid user.
- **`meta.toml` declares an `allsource` tool** so `meta dev` starts Core
  directly, replacing the doc's `docker compose up` infra task (see C.2-a).
- **`bacon.toml` was added at the root.** `meta doctor` warns without it, and
  `meta dev`'s API task is `bacon run-long`.

### C.5 Open questions: what moved

- **OQ-8 (`apps/api` server shape) — RESOLVED.** Plain `axum::serve`, allframe
  as a library. See C.2-e.
- **OQ-2 (optimistic concurrency on append) — still open, and now narrower.**
  `allsource 0.23.0`'s `IngestEventInput` is exactly
  `{event_type, entity_id, payload, metadata}` (read from the crate source, not
  the README). There is **no** `expected_version` field on the SDK's write path.
  If Core supports OCC on the wire, the SDK does not expose it, so §9.1's
  mitigation (2) is unavailable to us today.
- **OQ-1 (`getrandom 0.4` on wasm) — moot, as predicted.** No WASM crate pulls
  `getrandom`; the workspace-level `uuid` has no `v4`, and ids are minted
  server-side.
- **OQ-10 (Dioxus SSG in a workspace) — still open.** `dx bundle --package web
  --platform web --release` works and produces a bundle. The `--ssg` path was
  not wired: it needs a `static_routes` server function plus
  `IncrementalRendererConfig`, and that is left as a marked SEAM rather than
  guessed at. `apps/web` currently renders CSR, which is R1's stated fallback.
- **OQ-3, OQ-4, OQ-5, OQ-6, OQ-7, OQ-9 — untouched.** None was reachable from a
  scaffold.

### C.6 What is deliberately not implemented

Each has a `SEAM` comment at the site, so it is a marked gap rather than a
silent one.

| Gap | Where |
|---|---|
| Google OAuth (needs the HMAC-signed pending-origin cookie; without it the callback is an open redirect) | `apps/api/src/infrastructure/auth/better.rs` |
| `apps/web` SSG wiring | `apps/web/src/main.rs` |
| `pg2events`' Postgres reader (the row→event mapping and its guarantees **are** implemented and tested) | `tooling/pg2events/src/main.rs` |
| The ≤30s session cache for R5 | `apps/api/src/infrastructure/auth/middleware.rs` |
| Transactional email (`tera`) | not present |
| i18n — English only, OQ-5 | not present |

## Appendix D — D3/D4 correction: Core reads are tenant-scoped

Found while fixing a red CI run on the pushed repo, against a live Core.

### What is wrong

D3 adopts the official `allsource` SDK, and D4 says a single-node stack can point
`ALLSOURCE_QUERY_URL` at Core because `QueryClient::query_events` calls Core's own
`/api/v1/events/query`. The endpoint claim is right — the SDK's own doc comment says so — but
the conclusion drawn from it is wrong.

**Core scopes every read to a tenant, and answers a tenant-less query with an empty result set
rather than an error:**

```
GET /api/v1/events/query?entity_id=probe:rw-test                    -> {"events":[],"count":0}   HTTP 200
GET /api/v1/events/query?entity_id=probe:rw-test&tenant_id=default  -> {"events":[{...}]}
```

`allsource::QueryEventsParams` (v0.23.0) has **no `tenant_id` field**, so the SDK cannot send it.
`tenant_id` appears on the response type and in test fixtures only. A `QueryClient` aimed at
Core therefore reads nothing, forever, with no error anywhere.

The Query Service derives the tenant from the API key — the SDK documents it as "validates the
API key … applies tenant". That is precisely the concern D4 dismissed as gateway-only, and it is
why SDK reads work through the gateway and silently fail against Core.

### How it presented

Not as a read bug. `POST /posts` returned **404** in CI while the append succeeded: the handler
appends, then reads back through the fold to prove the write landed, and the read-back returned
empty. Auth failed the same way — sessions are events, the session lookup read nothing, and
every authenticated request 401'd with a valid token.

`GET /posts` kept working throughout, because the list path is served by the `posts_v1`
**worker** over a WebSocket stream, which carries no tenant parameter. That is why the original
vertical slice looked green: the one read path that was exercised end to end was the one path
that does not use the query endpoint.

### What changed

- `crates/rv2-allsource/src/tenant_query.rs` — a tenant-scoped read that folds with the same
  `allsource::EventFolder` the SDK path uses, exposed as `AllSource::fold_entity`. `load` in the
  posts handler uses it instead of `QueryClient::query_and_fold`.
- `crates/better-auth-allsource/src/client.rs` — all four query sites now send `tenant_id`.
- Tenant is `default`, overridable via `ALLSOURCE_TENANT_ID`.

Pointing `ALLSOURCE_QUERY_URL` at a real Query Service still works and needs none of this; the
change is what makes "one Core, no Query Service" true rather than merely untested.

### The generalisable lesson

An API that returns `200 {"events":[],"count":0}` for a malformed-but-parseable query is
indistinguishable from one reporting "no such entity". Every layer above it — the SDK, the auth
adapter, the handler — faithfully propagated "nothing here" and produced a plausible 404/401.
Where a read can be silently scoped, assert on a known-present fixture rather than trusting an
empty result.
