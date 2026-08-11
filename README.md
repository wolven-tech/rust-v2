# rust-v2

An all-Rust workspace whose **only datastore is [AllSource](https://github.com/all-source-os/all-source)**.
No Postgres, no Supabase, no TypeScript in the data path.

- `apps/api` — Axum HTTP API. The only server process.
- `apps/app` — Dioxus CSR SPA, the authenticated dashboard.
- `apps/web` — Dioxus marketing site, built SSG.
- `crates/` — the shared crate graph (events, domain, DTOs, UI kit, client, AllSource integration).
- `tooling/` — `meta` (the orchestrator) and `pg2events` (the one-shot migrator).

The design and every decision behind it live in
[`docs/architecture/001-rust-v2-allsource-foundation.md`](docs/architecture/001-rust-v2-allsource-foundation.md).
Read it before changing anything structural — it records *why*, and the code
only records *what*.

---

## Prerequisites

| | |
|---|---|
| Rust | `1.97.1` — pinned in `rust-toolchain.toml`, so `rustup` installs it for you |
| wasm target | `wasm32-unknown-unknown` — also in `rust-toolchain.toml` |
| `dx` | `cargo install dioxus-cli --version 0.7.10 --locked` — the only way to serve a frontend |
| `meta` | `cargo install --path tooling/meta` — the task orchestrator |
| `bacon` | `cargo install bacon` — used by `meta dev` for the API |
| AllSource Core | see below |

### Getting an AllSource Core running

**Recommended: the native binary.** Core is published on crates.io, so this
needs no Docker and no registry credentials:

```bash
cargo install allsource-core --version 0.23.0 --locked

ALLSOURCE_HOST=0.0.0.0 \
ALLSOURCE_PORT=3900 \
ALLSOURCE_DATA_DIR=.allsource-data \
ALLSOURCE_DEV_MODE=true \
  allsource-core
```

`ALLSOURCE_DEV_MODE=true` bypasses API-key auth. **Local development only.**
Without `ALLSOURCE_DATA_DIR` Core runs in-memory and everything is lost on
restart.

Verify:

```bash
curl -s http://localhost:3900/health
# {"status":"healthy","role":"leader","service":"allsource-core", ...}
```

**Alternative: Docker.** `docker-compose.yml` is set up for it, using the
public Apache-2.0 **community** images:

```bash
docker pull ghcr.io/all-source-os/allsource-core-community:latest
```

Use the `-community` tags. The unsuffixed `allsource-core` and
`allsource-query-service` images are the enterprise builds (BSL 1.1) and return
`denied` without a `docker login ghcr.io` token — that denial is an auth
failure, not a missing image.

One caveat: as of 2026-08-11 the community images publish **linux/amd64 only**,
so on Apple Silicon they run under emulation. The native binary above is
faster there, and is what `meta dev`, CI, and the vertical slice all use.

**Do you need the Query Service?** Not for the read paths this codebase uses —
which is a narrower claim than "not for local development", and the distinction
matters.

`QueryClient` is a thin `HttpTransport` wrapper aimed at whatever URL you give
it; it is not bound to the Query Service. The methods used here — `query_events`,
`get_entity_events`, `query_and_fold` — hit `/api/v1/events/query`, which the
SDK documents as *Core's* endpoint, and the vertical slice reads data back with
`ALLSOURCE_QUERY_URL` pointing at Core on `:3900`.

Four `QueryClient` methods do **not** work against Core:
`list_prime_projections`, `define_prime_projection`, `project_node` and
`node_field_provenance` all deserialize a gateway-only `{"data": ...}` envelope.
Bring the service up (`docker compose --profile gateway up`) before using any of
them, and for the gateway concerns it owns: per-tenant rate limits, quotas,
billing, and any external or untrusted client.

> **On ports.** Never hard-code one. Three upstream sources give three different
> pairs for the same services (`:3900`/`:3902`, `:3280`/`:3283`,
> `:3854`/`:3855`). `ServerConfig::from_env` therefore **requires**
> `ALLSOURCE_CORE_URL` and `ALLSOURCE_QUERY_URL` with no fallback, so a
> misconfiguration fails at boot with a named variable instead of silently
> talking to the wrong port.

---

## Setup

```bash
git clone <this repo> && cd rust-v2
cp .env.example .env
set -a && source .env && set +a
```

## Run

```bash
meta dev      # tmux: Core + api (bacon) + app (dx :4402) + web (dx :4401)
```

or individually:

```bash
# terminal 1 — Core
ALLSOURCE_DATA_DIR=.allsource-data ALLSOURCE_DEV_MODE=true allsource-core

# terminal 2 — API on :4400
cargo run -p api --features allsource-auth

# terminal 3 — dashboard on :4402
dx serve --package app --platform web --port 4402
```

| Service | Port |
|---|---|
| AllSource Core | 3900 |
| AllSource Query Service (optional) | 3902 |
| `apps/api` | 4400 |
| `apps/web` | 4401 |
| `apps/app` | 4402 |

Ports 4400–4402 preserve rust-v1's allocation, so existing `.env` files and
bookmarks carry over.

## Test

```bash
meta test                                    # every member
cargo test --workspace                       # same, without meta
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo audit --deny warnings                  # one lockfile ⇒ covers all 13 members
```

**The WASM boundary** is the most dangerous line in the workspace, so it is
proven by a real cross-compile rather than by inspection:

```bash
cargo check --target wasm32-unknown-unknown \
  -p rv2-events -p rv2-domain -p rv2-api-types -p rv2-ui -p rv2-client \
  -p app -p web
```

A crate that accidentally pulls `tokio` with `net`, `reqwest`, or native TLS
fails here in seconds instead of during a `dx build` three weeks later.

---

## The vertical slice

One feature, end to end:

```
HTTP request → domain event → AllSource append → projection fold
             → read model → rendered in the Dioxus UI
```

### As an automated test

```bash
export ALLSOURCE_CORE_URL=http://localhost:3900
export ALLSOURCE_QUERY_URL=http://localhost:3900
export ALLSOURCE_API_KEY=dev
export JWT_SECRET=dev-secret-key-that-is-at-least-32-characters-long

cargo test -p api --features allsource-auth --test vertical_slice \
  -- --ignored --nocapture --test-threads=1
```

It is `#[ignore]`d by default because it needs a live Core, and it **fails
loudly** rather than skipping when Core is unreachable — a silently-skipped
acceptance test is indistinguishable from a passing one.

### By hand — the one command worth running yourself

The automated test above asserts all of this. This is the single step it cannot
show you: the stored event, straight out of Core.

```bash
# `tenant_id` is NOT optional. Core scopes every read to a tenant and answers a
# tenant-less query with {"events":[],"count":0} and HTTP 200 — no error. Omit
# it and this prints nothing, which looks exactly like "the write failed".
curl -s "http://localhost:3900/api/v1/events/query?entity_id=post:$ID&tenant_id=default" \
  | python3 -m json.tool
```

```json
{
  "event_type": "content.post.created",
  "entity_id": "post:4862f98b-1c2e-422f-8210-1b3aa76389bd",
  "payload": {
    "type": "PostCreated",
    "id": "4862f98b-…",
    "author_id": "cf1714ec-…",
    "title": "Hello from the vertical slice",
    "content": "One event, all the way through.",
    "occurred_at": "2026-08-11T12:04:35.007276Z"
  },
  "timestamp": "2026-08-11T12:04:35.008130Z",
  "version": 1
}
```

Three things in there are load-bearing:

- **`event_type` is dotted; the payload's `"type"` is PascalCase.** Two different
  namespaces, mapped explicitly in `rv2_events::wire` and *checked* on decode.
  Nothing guesses one from the other.
- **`occurred_at` (payload) and `timestamp` (envelope) are different values.** The
  envelope timestamp is assigned by Core at ingest. All domain time is read from
  the payload, which is what lets a backdated or migrated event keep its real
  date instead of collapsing to "now".
- **The tenant scoping above is asserted**, not just documented, in
  `crates/rv2-allsource/tests/core_contract.rs`. Every defect this codebase has
  hit in AllSource was an assumption written in a comment and checked nowhere.

### In the browser

```bash
dx serve --package app --platform web --port 4402
```

Open <http://localhost:4402/posts>. Unauthenticated you are redirected to
`/login` (the client-side replacement for rust-v1's Next.js `proxy.ts`
middleware). After signing in, the list renders `PostView`s folded from
`content.post.*` events — **not fixtures** — and the Publish form appends a new
event and re-reads through the API rather than patching a local cache, so a
render proves the fold actually happened.

---

## Architecture notes worth knowing before you edit

**The crate layers.** `rv2-events` is layer 0; `rv2-domain` and `rv2-api-types`
layer 1; `rv2-ui`, `rv2-client`, `rv2-allsource`, `rv2-shared` and
`better-auth-allsource` layer 2; `apps/*` layer 3. A crate may only depend on
strictly lower layers, and `apps/*` are leaves.

**Events are immutable, forever.** Fields may only be **added**, and every added
field carries `#[serde(default)]`. `crates/rv2-events/tests/golden/` holds one
captured JSON payload per released schema version, and every one must keep
deserializing into the current build. Anything that cannot be expressed
additively becomes a **new** wire type plus a new variant; the old variant and
its fold arm stay in the code permanently.

**Folders must be pure and total.** A read-model rebuild replays the entire
store, so no clock reads, no network calls, no `unwrap()` on payload shape, and
`apply` must be idempotent for an event it has already seen.

**Rebuilding a projection** = renaming the worker's durable consumer id
(`posts_v1` → `posts_v2`). Core keys the cursor by that name, so a new name
replays from zero. Run both in parallel, compare, flip the handler, then delete
the old state. Do not attempt cursor surgery.

**`crates/better-auth-allsource` is vendored and is a version bridge, not a
fork.** Bug-for-bug ports only; never add features. See its `PROVENANCE.md`, and
the weekly `vendor-check` workflow that opens an issue when it can be deleted.

---

## Known gaps

These are marked, not hidden. Each has a `SEAM` comment at the site.

| Gap | Where | Why |
|---|---|---|
| Google OAuth is not wired | `apps/api/src/infrastructure/auth/better.rs` | The plugin needs the HMAC-signed pending-origin cookie glue; without it the callback is an open redirect. Credential auth is complete end to end. |
| `apps/web` SSG is not wired | `apps/web/src/main.rs` | Needs the `static_routes` server function + `IncrementalRendererConfig`. The app builds and cross-compiles; it currently renders CSR. |
| `pg2events` has no Postgres reader | `tooling/pg2events/src/main.rs` | Phase-8 work; needs the real Supabase replica. The row→event mapping and its guarantees are implemented and tested. |
| No session cache | `apps/api/src/infrastructure/auth/middleware.rs` | Authenticated requests cost two AllSource round-trips. Measure p99 before adding the cache. |
| English only | — | No Rust i18n crate has been evaluated. rust-v1 shipped `en` + `fr`; this is a product regression that needs sign-off. |

---

## Docs

| Document | What it is |
|---|---|
| [`docs/architecture/001-rust-v2-allsource-foundation.md`](docs/architecture/001-rust-v2-allsource-foundation.md) | The design. 21 numbered decisions (D1–D21), the risks (R*), and the open questions (OQ-*) that the code comments cite by number. Appendix C records what the scaffold changed; Appendix D corrects D3/D4 on tenant-scoped reads. |
| [`docs/ledger/allsource-integration-corpus.md`](docs/ledger/allsource-integration-corpus.md) | The frozen list of AllSource behaviours this integration depends on (B1–B21), which are asserted, and the loop that asserted them. |
| [`docs/ledger/component-kit-autoresearch.md`](docs/ledger/component-kit-autoresearch.md) | How `rv2-ui` got its scope and why the bundle is the size it is. |

The architecture doc lived in the `rust-v1` repo until 2026-08-11, which left
136 references to `§`, `D*`, `R*` and `OQ-*` in this codebase pointing at a file
that was not here.
