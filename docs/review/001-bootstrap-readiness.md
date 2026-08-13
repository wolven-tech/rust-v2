# 001 — Bootstrap readiness review

**Date:** 2026-08-13
**Scope:** the whole repository, read cold, with one question — *is this a
foundation someone can start a new project from?*
**State at review:** 104 tracked files, ~9.6k lines of Rust, 110 tests, 12
commits, one branch.

---

## Verdict

The architecture and the gate discipline are unusually strong. The repository
was **not bootstrappable**, for two reasons that have nothing to do with the
architecture:

1. Nothing existed between `cargo run` and production — no container build, no
   licence, no readiness contract, nothing watching the pins.
2. One shipped defect contradicted its own comment.

Both are now addressed. This document records each finding, what it actually
was, and what was done — including the two places where the fix was to write
down a decision rather than write code.

**Status legend:** ✅ fixed · 📝 recorded as a decision · ⏳ deferred with a
stated reason.

---

## P0 — defects

### 1. Analytics blocked the request path, under a comment saying it did not ✅

`POST /posts` awaited `Analytics::track` before returning `201`. The comment
above the call read:

> `track` never errors and never blocks the response on a vendor

The implementation called `posthog_rs::Client::capture_immediate`. That method's
own documentation:

> Sends inline (bypassing the background worker) and **retries transient
> failures per the client's retry configuration**.
> […] prefer fire-and-forget everywhere else.

So a PostHog outage added the vendor's latency *and its entire retry budget* to
every publish — precisely the failure mode the comment claimed to prevent. The
API had been chosen by the plausibility of its name.

**Fixed.** `track` now calls `Client::capture`, which enqueues on the SDK's
background worker and returns. It is no longer `async`, which makes the mistake
unrepeatable: there is nothing for a handler to await. Delivery failures surface
through an `on_error` hook registered at construction, since fire-and-forget
reports nothing to the caller by design.

The test that covers the disabled path is deliberately a plain `#[test]` rather
than `#[tokio::test]`. That is the assertion — it stops compiling the moment
someone puts a round-trip back on the request path.

### 2. Queued analytics events died on shutdown ✅

The corollary, and the reason fixing #1 alone would have been a downgrade.
Fire-and-forget means events exist only in an in-process queue until the worker
drains them. Nothing called `flush` or `shutdown`, so every deploy silently
dropped whatever was in flight.

rust-v1's `setupAnalytics` returned `{ track, shutdown }`. The port kept `track`
and dropped `shutdown` — an omission with no symptom except missing data around
each restart.

**Fixed.** `Analytics::shutdown` exists and `apps/api` calls it after
`axum::serve` returns, so nothing tracked by an in-flight request is lost. It is
safe to call on the disabled variant, so the call site needs no branch — the one
code path that runs on every deploy should not be the one nobody exercises.

---

## P0 — could not deploy

### 3. No container build ✅

No `Dockerfile`, no `.dockerignore`, no deployment manifest. `docker-compose.yml`
started AllSource Core and nothing else. The repository went from `git clone` to
`localhost` and stopped — a fine starting point, and a bad place to leave a
foundation other projects are meant to start from.

**Fixed.** A two-stage `Dockerfile`: manifests copied first so the dependency
layer caches against source edits, `cargo fetch --locked` to populate it, then
`--locked --offline` for the real build so the image can never resolve a
different dependency set than the one CI checked. The runtime stage is
`debian:bookworm-slim` with `ca-certificates` (rustls verifies against the system
trust store — without it every outbound call fails with an opaque certificate
error), a non-root user, and a `HEALTHCHECK` on `/health`.

Two deliberate omissions, both documented in the file: `pkg-config`/`libssl-dev`
are absent, because needing them would mean the `native-tls` ban had been
breached; and the Dioxus frontends are not in the image, because static wasm
bundles belong on a CDN and bundling them would couple a frontend deploy to an
API deploy.

### 4. No `LICENSE` file ✅

`Cargo.toml` declared `license = "MIT"`. There was no licence text in the
repository. On a public repository that is not a formality — absent an explicit
grant, the default is all rights reserved, so nobody could legally fork the
thing that exists to be forked.

**Fixed.** MIT text added, `Copyright (c) 2026 Wolven Tech`. *If that is not the
right legal entity, it is a one-line change and worth making now rather than
after the first external contribution.*

---

## P1 — operability

### 5. No readiness probe ✅

Only `/health` existed, and it always answered `200`. There was no way for an
orchestrator to distinguish "restart this" from "route around this".

The existing comment in `health.rs` had already identified the fix and left it
undone: *"Readiness is a separate concern; when it is added, that is where
`is_caught_up()` belongs."*

**Fixed.** `GET /ready` reports Core reachability and whether the `posts_v1`
projection has caught up, and answers `503` when either says this instance
should not serve. A worker that never started reports `null` and does **not**
fail readiness — that is a supported degraded mode, since `GET /posts` folds on
read. A worker that is running but still replaying does fail it, because the
instance is serving stale data and holding it out of the load balancer for that
window is the entire job.

### 6. `/health` made an outbound call per hit ✅

It called Core's `/health` on every request. That endpoint is unauthenticated
and mounted outside the rate limiter, so any caller could amplify traffic onto
Core through it — and each failure logged a warning, meaning a Core outage also
produced a log flood from the probe path.

**Fixed.** `/health` is now pure liveness: no outbound call, no log line, no
dependency state in the response body. Dependency state moved to `/ready`, where
the orchestrator can actually act on it. The vertical-slice test asserts the
liveness body carries *no* `allsource_reachable` field, so the two cannot merge
back together by accident.

### 7. `GET /posts` had no pagination, and its fallback truncated silently ✅

The endpoint returned every post — a response size growing with the store, with
no signal until it was already a problem. Worse, the fold-on-read fallback
capped its scan at 10,000 events and said nothing. A truncated scan and a
complete one have the same shape, so past that line the folds are computed from
partial history: the posts come back **wrong**, not merely missing.

**Fixed.** `?limit=` (default 50, capped at 200, clamped rather than rejected —
a client asking for too much wants as much as it can have) and `?offset=`, both
documented in the OpenAPI spec with a test asserting they stay there. The
fallback now logs at `error` when it hits its ceiling, naming what is wrong and
what to do about it.

Paginating in memory is deliberate, not laziness: the read model *is* the full
map, held in process by the worker, so there is no cheaper source to page from.
What the limit bounds is the serialized response and the client's parse, which
are the parts that scale with the store.

### 8. No metrics or traces exported 📝

`tracing` wrote to stdout as human-readable text. The README's "Logging: Done"
row overstated what was actually available to an operator.

**Partly fixed, partly recorded.** `LOG_FORMAT=json` now emits one JSON object
per event, so the fields the code already attaches reach an aggregator as fields
rather than as a line to re-parse with a regex. The `Dockerfile` sets it; local
development does not, because a developer wants the readable form. It is
deliberately not auto-detected from a TTY — a container run interactively must
log the way it does in production, or the format silently differs between the
place you debug and the place it matters.

Metrics and traces are **not** wired, and that is now a stated gap rather than
an omission. Choosing between an OpenTelemetry collector and a Prometheus scrape
has real operational cost either way, and picking one without knowing where this
will run would be guessing. It needs a decision before it needs code.

### 9. The rate limiter trusted a client-controlled header ✅

The client key came from `x-forwarded-for`'s **leftmost** entry whenever the
header was present. That header is appended to by each hop, so its leftmost
entry is whatever the client sent — arbitrary, unauthenticated text on any
deployment not behind a proxy that overwrites it.

The existing comment was honest about half of it ("abuse-blunting, not a
security control") and missed the half that matters more:

1. A caller sending a different fake ip per request gets a fresh bucket every
   time, so the limiter is off for exactly the traffic it exists to blunt.
2. Every fake ip becomes a **permanent key in an in-memory map**. One scripted
   loop grows the process's memory without bound. A rate limiter that is also a
   memory-exhaustion vector is worse than no rate limiter.

**Fixed.** `TRUSTED_PROXY_HOPS` states how many right-hand entries the
operator's own infrastructure appends; the honest client ip is that many from
the end. The default is `0` — no trusted proxy — under which the header is
ignored outright and the key is the socket address, which cannot be forged. A
chain shorter than the configured hop count falls back rather than picking an
arbitrary entry.

This has to be configured rather than inferred: behind one proxy the client ip
is the last entry, behind two it is second-from-last, and only the operator
knows which. It is called out in the README and `.env.example` because it is
also easy to get wrong in the other direction.

---

## P1 — repository hygiene ✅

Missing: `LICENSE`, `CONTRIBUTING.md`, `SECURITY.md`, `CODEOWNERS`, a pull
request template, `.editorconfig`, `CHANGELOG.md`, dependency automation, and
any agent- or newcomer-facing orientation.

Two of those mattered more than the rest.

**Dependency automation.** The toolchain is pinned to an exact version,
`allsource` is pinned exactly, and `deny.toml` carries three dated RUSTSEC
ignores. None of that drifts loudly — it drifts by staying identical while the
world moves, and the first signal is an advisory nobody has looked at in six
months. Dependabot now runs weekly, grouped into one PR (individually-opened PRs
against a 500-crate tree are noise that gets bulk-closed, which is the same as
no updates but with more notifications). `allsource` minor and major bumps are
explicitly ignored, because that crate's version tracks a running server and
bumping it alone produces a client speaking an API the server does not.

**Orientation.** The value of this repository is concentrated in 1,885 lines of
architecture document, cited from code comments by number (`§2.2`, `D5`, `R6`,
`OQ-8`). Someone arriving cold read all of it or none of it. `AGENTS.md` is now
the short path: what this is, the one command, the four things most likely to
catch you out, and where the *why* lives. `CLAUDE.md` symlinks to it.

---

## P2 — the gate itself

### 10. The WASM boundary check ran `cargo tree` ten times for two answers ✅

The tree output depends only on the app, but the invocation sat inside a loop
over five server-only crates: five crates × two apps = ten resolutions for two
distinct results, on every CI run and every local gate.

**Fixed.** Loops swapped — one `cargo tree` per app, filtered in memory.

### 11. The predecessor grep had no escape hatch ✅

`no_predecessor()` greps the whole repository for three words with zero
tolerance. Correct for the migration it was written for. For anyone forking this
as a base it is a landmine: a design document explaining what the stack replaced
fails CI with no way to say "I meant that". The usual outcome of an unarguable
check is that somebody deletes the check.

**Fixed.** A line carrying the marker `predecessor-mention-ok` is exempt. It is
deliberately ugly and deliberately per-line — an opt-out that is easy to apply
broadly stops being an exception. The failure message names it, so the way out
is discoverable at the moment you need it.

### 12. The stylesheet freshness check mutated the tree to check it ✅

`styles_are_current()` compiled over the committed CSS, compared, and restored
the originals on mismatch. So the one command whose entire job is *verify
nothing changed* was also the command most likely to leave a dirty tree: a
panic, a second app failing to compile, or a Ctrl-C between the write and the
restore each left generated CSS staged over the committed CSS.

**Fixed.** It compiles into `target/style-check/` and compares. The working tree
is never written to.

---

## Deferred, with reasons ⏳

Unchanged by this review, listed so they are not mistaken for oversights:

| Item | Why it is still open |
|---|---|
| Background jobs | Needs a decision (in-process scheduler vs durable queue vs external scheduler), not code. No seam has been faked. |
| Google OAuth | Marked `SEAM`. Wiring it without the HMAC-signed pending-origin cookie glue produces an open redirect. |
| `apps/web` SSG | `OQ-10`. Renders CSR; needs `static_routes` + `IncrementalRendererConfig`. |
| i18n | rust-v1 shipped `en` + `fr`. A product regression that needs sign-off, not a technical choice. |
| Session cache | Authenticated requests cost two AllSource round-trips. Measure p99 before adding a cache. |
| Shared rate limits | Per-instance and in-memory, so N instances allow N× the limit. A shared limiter needs a store this workspace does not have. |

---

## What is worth keeping if you fork this

The findings above are the parts that were wrong. These are the parts that were
right, and they are the reason the repository was worth reviewing closely:

- **One gate command.** CI runs `cargo xtask ci` verbatim — the same command a
  developer runs. There is no shell script duplicated into YAML, so local and CI
  cannot drift. This is rare and it is correct.
- **The WASM boundary is proven by cross-compile, in both directions**, not by
  convention and not by review.
- **One golden JSON fixture per released event schema version**, every one of
  which must still deserialize.
- **Configuration fails at boot naming the variable.** No default ports for the
  datastore (three upstream sources give three different pairs), `*` CORS
  rejected because it is illegal with credentialed requests, half-configured
  OAuth rejected because it produces a sign-in button that fails at the callback.
- **`deny.toml` records the two-`reqwest` duplication** with who pulls each one,
  so a *new* duplicate still fails. The cost is accepted, not hidden.
- **`ServerConfig`'s `Debug` redacts secrets** — and it is the thing logged at
  boot.
- **Live tests fail loudly when Core is unreachable** rather than skipping. A
  silently-skipped acceptance test is indistinguishable from a passing one.
- **Comments record defects, not descriptions.** Several of the sharpest ones
  exist because someone lost a day to the thing they describe. That is the
  habit that makes the rest of this reviewable at all.
