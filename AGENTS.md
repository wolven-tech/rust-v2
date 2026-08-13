# Orientation

For anyone — human or agent — arriving with no context. Read this first; it is
short on purpose and points at the things that are long.

## What this is

An all-Rust workspace whose only datastore is
[AllSource](https://github.com/all-source-os/all-source), an event-sourcing
database. There is no relational database anywhere in the data path, no
TypeScript, and no JavaScript package manager.

- `apps/api` — Axum HTTP API. The only server process.
- `apps/app` — Dioxus CSR SPA, the authenticated dashboard.
- `apps/web` — Dioxus marketing site.
- `crates/` — the shared graph: events, domain, DTOs, UI kit, client, AllSource
  integration, analytics, email.
- `tooling/xtask` — the gate.

## One command

```bash
cargo xtask ci
```

That is the whole local gate, and CI runs the same command. Run it before you
believe anything you have changed works. It is: rustfmt, clippy, build, test,
stylesheet freshness, a predecessor-stack grep, and the wasm32 boundary in both
directions.

`cargo xtask live` runs the tests that need a running AllSource Core. Those tests
are `#[ignore]`d, so `cargo test` skips them **in silence** — which looks exactly
like passing.

## The four things most likely to catch you out

**1. Reads must be tenant-scoped.** Core answers a query with no `tenant_id`
with `{"events":[],"count":0}` and **HTTP 200** — no error. A read path that
omits it silently returns nothing forever, which looks like "the write failed".
The SDK's own `QueryEventsParams` has no field for it; that is why
`crates/rv2-allsource/src/tenant_query.rs` exists. Use it.

**2. The WASM boundary is real and is enforced by cross-compile.** `rv2-events`,
`rv2-domain`, `rv2-api-types`, `rv2-ui`, `rv2-client` and both apps compile to
`wasm32-unknown-unknown`. Nothing that pulls `reqwest`, `tokio` with `net`,
native TLS, or `getrandom` may be reachable from them. Server-only crates —
`rv2-allsource`, `rv2-shared`, `better-auth-allsource`, `rv2-analytics`,
`rv2-email` — are asserted *unreachable* from the apps, not merely undeclared.

**3. Events are immutable forever.** Fields may only be added, each with
`#[serde(default)]`. `crates/rv2-events/tests/golden/` holds one captured payload
per released schema version and every one must keep deserializing. Anything not
expressible additively becomes a new wire type plus a new variant; the old
variant and its fold arm stay in the code permanently.

**4. New HTTP dependencies default to the wrong TLS.** `deny.toml` bans
`native-tls` and `openssl-sys`. Two dependencies here (`better-auth`,
`resend-rs`) default to native TLS and needed `default-features = false` plus an
explicit rustls feature. Assume the next one does too.

## Where the *why* lives

`docs/architecture/001-rust-v2-allsource-foundation.md` — 21 numbered decisions
(`D1`–`D21`), risks (`R*`) and open questions (`OQ-*`). Code comments cite them
by number, so `§2.2 trap 1` and `R6` point at real paragraphs. **If you overturn
a decision, edit the decision** — a code change that silently contradicts one
leaves two sources of truth.

Also:

- `docs/review/001-bootstrap-readiness.md` — an outside review of this repo as a
  foundation, and what was done about each finding.
- `docs/ledger/` — how the component kit got its scope, and which AllSource
  behaviours are asserted rather than assumed.
- `CONTRIBUTING.md` — the rules and what enforces each.

## House style, as observed

Comments say *why*, not what. Where a defect was fixed, the comment records the
defect — several of the sharpest comments in this codebase exist because someone
lost a day to the thing they describe. Test names are sentences describing the
behaviour under test. Docs state limitations plainly rather than omitting them:
a capability that is configured but absent is treated as worse than one that is
obviously missing.

## Things deliberately not built

- **Background jobs.** No runner is wired, and no seam has been faked, because
  the options differ enough operationally that choosing without a real workload
  is guessing.
- **Google OAuth.** Marked with a `SEAM` comment. Wiring it without the
  HMAC-signed pending-origin cookie glue produces an open redirect.
- **`apps/web` SSG.** It renders CSR today.
- **Metrics and traces.** Logs are structured; nothing is exported. Picking a
  backend is an operator decision.

Do not quietly fill one of these in as a side effect of another change.
