# rust-v2

An all-Rust workspace whose **only datastore is [AllSource](https://github.com/all-source-os/all-source)**.
One datastore, one language, no TypeScript in the data path.

- `apps/api` — Axum HTTP API. The only server process.
- `apps/app` — Dioxus CSR SPA, the authenticated dashboard.
- `apps/web` — Dioxus marketing site, built SSG.
- `crates/` — the shared crate graph (events, domain, DTOs, UI kit, client, AllSource integration, analytics, email, jobs).
- `tooling/xtask` — the gate: `cargo xtask ci`, and the Tailwind compile.

The design and every decision behind it live in
[`docs/architecture/001-rust-v2-allsource-foundation.md`](docs/architecture/001-rust-v2-allsource-foundation.md).
Read it before changing anything structural — it records *why*, and the code
only records *what*.

New here? [`AGENTS.md`](AGENTS.md) is the short orientation — what this is, the
one command, and the four things most likely to catch you out.

---

## Prerequisites

| | |
|---|---|
| Rust | `1.97.1` — pinned in `rust-toolchain.toml`, so `rustup` installs it for you |
| wasm target | `wasm32-unknown-unknown` — also in `rust-toolchain.toml` |
| `dx` | `cargo install dioxus-cli --version 0.7.10 --locked` — the only way to serve a frontend |
| `meta` | `cargo install monorepo-meta` — the task orchestrator, driven by `meta.toml` |
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
# terminal 1 — Core on :3900
cargo xtask core

# terminal 2 — API on :4400
cargo run -p api --features allsource-auth

# terminal 3 — dashboard on :4402
dx serve --package app --platform web --port 4402
```

After editing a component's Tailwind classes, recompile the stylesheets:
`cargo xtask styles`. The compiled CSS is committed so a fresh clone renders
correctly with no build step; `cargo xtask ci` fails if what is committed has
gone stale.

| Service | Port |
|---|---|
| AllSource Core | 3900 |
| AllSource Query Service (optional) | 3902 |
| `apps/api` | 4400 |
| `apps/web` | 4401 |
| `apps/app` | 4402 |

Ports 4400–4402 preserve rust-v1's allocation, so existing `.env` files and
bookmarks carry over.

## Deploy

```bash
docker build -t rust-v2-api .
docker run --rm -p 4400:4400 --env-file .env rust-v2-api
```

The image builds `apps/api` only. The Dioxus frontends compile to static wasm
bundles (`dx bundle --package web --platform web --release`) and belong on a CDN
or a static file server — putting them inside the API image would couple a
frontend deploy to an API deploy for no reason.

It runs as a non-root user, ships no toolchain, and sets `LOG_FORMAT=json` so
logs arrive at an aggregator as fields rather than as text to re-parse.

### Probes

Two endpoints, because they answer two different questions. Wire both.

| | Endpoint | Answers | On failure |
|---|---|---|---|
| Liveness | `GET /health` | "Is this process wedged?" | The orchestrator restarts it |
| Readiness | `GET /ready` | "Should this instance get traffic?" | Routed around, left running |

`/health` deliberately checks **no dependency** and always answers `200` while
the process can serve. A liveness probe that reports on AllSource gets every
instance killed during a Core outage, and the restart cannot help.

`/ready` is where dependencies belong: it reports Core reachability and whether
the `posts_v1` projection has caught up, and answers `503` when either says this
instance should not be serving. During a projection replay the data is real but
stale — keeping that out of the load balancer is the entire job.

The `Dockerfile`'s `HEALTHCHECK` uses `/health` only, because Docker's sole
response to a failing check is a restart, which is the wrong answer to `/ready`.

## Test

```bash
cargo xtask ci      # exactly what CI runs — see below
cargo xtask live    # the tests that need a live Core
```

`cargo xtask ci` is the whole gate: rustfmt, clippy, build, test, a
stylesheet-freshness check, the no-predecessor grep, and the wasm32 boundary in
both directions. CI runs the same command, so the two cannot drift — there is no
shell script duplicated into YAML to keep in sync. It stops at the first
failure and prints a header per step, so a failure names itself.

The one thing it does **not** cover is `cargo deny check` (advisories, bans,
licences, sources), which needs its own tool install and has its own CI job.

**The WASM boundary** is the most dangerous line in the workspace, so `xtask ci`
proves it by real cross-compile rather than by inspection — and in both
directions: the WASM-safe crates must compile for `wasm32-unknown-unknown`, and
the server-only crates must not be reachable from either app. A crate that
accidentally pulls `tokio` with `net`, `reqwest`, or native TLS fails in seconds
instead of during a `dx build` three weeks later.

**The live tests are `#[ignore]`d**, so a plain `cargo test` skips them in
silence — which is indistinguishable from passing. `cargo xtask live` is what
actually runs them: it checks Core is reachable, tells you how to start one if
not, sets the four environment variables, and runs the contract suite before the
vertical slice.

---

## The vertical slice

One feature, end to end:

```
HTTP request → domain event → AllSource append → projection fold
             → read model → rendered in the Dioxus UI
```

### As an automated test

```bash
cargo xtask live
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
layer 1; `rv2-ui`, `rv2-client`, `rv2-allsource`, `rv2-shared`,
`better-auth-allsource`, `rv2-analytics`, `rv2-email` and `rv2-jobs` layer 2;
`apps/*` layer 3. A crate may only depend on strictly lower layers, and `apps/*`
are leaves.

**`GET /posts` is paginated** — `?limit=` (default 50, capped at 200, clamped
rather than rejected) and `?offset=`. It is bounded because an unbounded list
endpoint is a cliff that arrives without warning, at whatever point the store
grows enough. When the projection worker is down the fallback scans events
directly, and that scan has a 10,000-event ceiling it logs at `error` on
crossing: past it the folds are computed from a truncated history, so they are
*wrong* rather than merely short.

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
| No session cache | `apps/api/src/infrastructure/auth/middleware.rs` | Authenticated requests cost two AllSource round-trips. Measure p99 before adding the cache. |
| English only | — | No Rust i18n crate has been evaluated. rust-v1 shipped `en` + `fr`; this is a product regression that needs sign-off. |
| Rate limits are per-instance | `apps/api/src/infrastructure/rate_limit.rs` | In-memory, so N instances allow N× the limit, and a restart clears them. Correct for now (§9.2: a counter is not an event); a shared limiter needs a store this workspace does not have. |
| No durable job queue | `crates/rv2-jobs` | Periodic in-process work only: every instance runs every job, nothing survives a restart, and failures are not retried. See *Background jobs* below. |

**`TRUSTED_PROXY_HOPS` is not a gap but is easy to get wrong.** It defaults to
`0`, meaning no trusted proxy, under which `x-forwarded-for` is ignored entirely
and the limiter keys on the socket address. Deploy behind a proxy without
setting it and every request buckets under the proxy's own ip. Set it too high
and a caller can choose their own bucket — which both bypasses the limit and
grows the key set without bound.

---

## Platform capabilities

What rust-v1 provided as TypeScript packages, and where each lives now. Every
row is either implemented or names what is missing — a capability that is
configured but absent is worse than one that is obviously not there.

| Capability | rust-v1 | rust-v2 | State |
|---|---|---|---|
| Product analytics | `packages/analytics` (posthog-node) | `crates/rv2-analytics` — PostHog's **official** Rust SDK | **Done.** Tracks `post_published`; `Disabled` without a key |
| Transactional email | `packages/email` — React Email templates, **no sender** | `crates/rv2-email` — Tera templates + Resend's **official** Rust SDK | **Done, and more than v1 had.** v1 declared `RESEND_API_KEY` and never read it |
| Logging | `packages/logger` (pino) | `tracing` throughout, `LOG_FORMAT=json` for structured output | **Done** |
| Metrics | — (rust-v1 had none) | Prometheus scrape on `METRICS_ADDR` | **Done**, opt-in. HTTP + job metrics, labelled by matched route |
| Traces | — (rust-v1 had none) | OTLP export on `OTEL_EXPORTER_OTLP_ENDPOINT` | **Done**, opt-in. Verified against a live collector |
| Rate limiting / KV | `packages/kv` (Upstash Redis) | `allframe`'s `KeyedRateLimiter` in `AppState` | **Done**, in-memory. A counter is deliberately not an event (§9.2) |
| Server state / caching | `packages/react-query` | Dioxus `use_resource` | **Done** |
| UI kit | `packages/ui` (shadcn/React) | `crates/rv2-ui` (Dioxus) | **Done** — 29 components |
| Background jobs | `packages/jobs` (trigger.dev) | `crates/rv2-jobs` | **Periodic work only.** Not a durable queue — read the caveat below before adding a job |

### Analytics is off the request path, in both halves

`Analytics::track` is **synchronous** and fire-and-forget. That is not a detail,
it is the guarantee: a handler cannot await a vendor because there is no future
to await. An earlier version called PostHog's `capture_immediate` — which sends
inline and retries per the client's retry configuration — and awaited it from
`POST /posts`, putting the vendor's latency *and its whole retry budget* on
every publish, under a comment claiming it never blocked on a vendor.

The other half of fire-and-forget is the easy one to forget: `apps/api` calls
`Analytics::shutdown` after `axum::serve` returns. Without it, everything still
queued dies with the process, silently, on every deploy.

### Unconfigured is a supported state

Both vendor integrations degrade rather than fail:

- **No `POSTHOG_API_KEY`** → events are logged, not sent. Analytics is not
  load-bearing, and a missing key must never fail a request that would otherwise
  have succeeded.
- **No `RESEND_API_KEY`** → the template still *renders*, to the log. A broken
  template then surfaces in development rather than the first time a key exists
  in staging, and local development sees the email without a vendor account.

That is also what makes both crates testable with no network and no keys: their
tests drive the real code path, not a mock.

### Background jobs: periodic work, and nothing more

`crates/rv2-jobs` is an in-process periodic scheduler. It is the smallest thing
that is genuinely useful, and the boundary is sharp:

| | `rv2-jobs` | A durable queue |
|---|---|---|
| Survives a restart | No — schedules are in memory | Yes |
| Runs once across N instances | **No — every instance runs every job** | Yes, via a lease |
| Retries a failure | No; it runs again next period | Yes, with backoff |
| Enqueued at runtime | No; registered at boot | Yes |

The second row is the one that bites. Scale to three instances and every job
runs three times per period. Fine for refreshing a gauge, completely wrong for
"email the customer" — so **do not register a job whose second execution would
be a defect.**

What it does get right, because each of these is a way the naive version fails
silently: a panicking run does not kill the schedule (each tick runs in its own
task); a slow run does not pile up (missed ticks are skipped, not queued); jobs
do not all fire in the same millisecond after a deploy (each is offset by a
deterministic fraction of its period, derived from its name); and shutdown
awaits in-flight runs rather than cutting them off mid-write.

One job is registered today — `dependency_health`, which refreshes the
`allsource_reachable` and `projection_caught_up` gauges.

When you need durability, the seam to build against is a **leased queue over
AllSource**: append `job.claimed` / `job.finished` events and get durability
plus an audit trail from the store that already exists. That is a design, not a
line of code, and it needs a real workload before it needs writing.

### Metrics and traces

Both are opt-in and both cost nothing when off.

| Signal | Off unless | When off |
|---|---|---|
| Structured logs | `LOG_FORMAT=json` | Human-readable to stdout |
| Metrics | `METRICS_ADDR` is set | No recorder installed; every `metrics` macro is a no-op |
| Traces | `OTEL_EXPORTER_OTLP_ENDPOINT` is set | No exporter, no batching thread |

```bash
METRICS_ADDR=0.0.0.0:9090 \
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 \
  cargo run -p api
```

**`/metrics` is on its own port, not on the application router.** Scrape output
names every route and reports request volumes; it has no business being subject
to — or exempt from — the app's rate limiter and CORS policy. A separate
listener that is simply never published outside the network is a cruder control
than an auth check and a much harder one to get subtly wrong. The `Dockerfile`
does not `EXPOSE` it.

**HTTP metrics are labelled by the matched route** (`/posts/{id}`), never the
request path. Labelling by path is one time series per uuid, they never expire,
and the Prometheus instance falls over long before anyone connects it to the
line of code responsible. Unrouted requests report `unmatched` for the same
reason — 404-scanning traffic would otherwise mint unbounded labels.

A `METRICS_ADDR` that will not bind **fails the boot**, on purpose. A metrics
endpoint that silently is not there is how a dashboard ends up showing a flat
line that everyone reads as "no traffic".

---

## Docs

| Document | What it is |
|---|---|
| [`docs/architecture/001-rust-v2-allsource-foundation.md`](docs/architecture/001-rust-v2-allsource-foundation.md) | The design. 21 numbered decisions (D1–D21), the risks (R*), and the open questions (OQ-*) that the code comments cite by number. Appendix C records what the scaffold changed; Appendix D corrects D3/D4 on tenant-scoped reads. |
| [`docs/ledger/allsource-integration-corpus.md`](docs/ledger/allsource-integration-corpus.md) | The frozen list of AllSource behaviours this integration depends on (B1–B21), which are asserted, and the loop that asserted them. |
| [`docs/ledger/component-kit-autoresearch.md`](docs/ledger/component-kit-autoresearch.md) | How `rv2-ui` got its scope and why the bundle is the size it is. |
| [`docs/review/001-bootstrap-readiness.md`](docs/review/001-bootstrap-readiness.md) | A cold review of this repository as a foundation to start a project from: twelve findings, what each one actually was, and what was done about it. |
| [`AGENTS.md`](AGENTS.md) | The short orientation for anyone arriving with no context. `CLAUDE.md` points here. |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | The rules, and what enforces each one. |
| [`SECURITY.md`](SECURITY.md) | How to report a vulnerability, what is in scope, and the limitations stated rather than hidden. |
| [`CHANGELOG.md`](CHANGELOG.md) | What has changed and why. |

The architecture doc lived in the `rust-v1` repo until 2026-08-11, which left
136 references to `§`, `D*`, `R*` and `OQ-*` in this codebase pointing at a file
that was not here.
