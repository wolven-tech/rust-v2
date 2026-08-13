# Changelog

Notable changes to this workspace. Format loosely follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Nothing here is released yet — the workspace is at `0.1.0` and every crate is
`publish = false`.

## [Unreleased]

### Fixed

- **Analytics no longer blocks the request path.** `Analytics::track` called
  PostHog's `capture_immediate`, which sends inline and retries per the client's
  retry configuration; awaited from a handler, that put the vendor's latency and
  full retry budget on every publish — under a comment claiming it never blocked
  on a vendor. `track` is now synchronous and fire-and-forget, so the mistake
  cannot recur without changing its signature.
- **Queued analytics events are now flushed on shutdown.** Fire-and-forget
  capture means events live in an in-process queue; the process exited without
  draining it, losing them silently on every deploy.
- **`/health` no longer calls AllSource Core.** It was one unauthenticated,
  unrate-limited request in and one request out — an amplification path onto
  Core — and logged a line per failure, so a Core outage produced a log flood
  from the probe path.
- **The rate limiter no longer trusts `x-forwarded-for` by default.** Keying on
  the leftmost entry let any caller mint a fresh bucket per request with a fake
  ip, which both bypassed the limit and grew the in-memory key set without
  bound. `TRUSTED_PROXY_HOPS` now selects the hop, and the default of `0`
  ignores the header entirely.
- **`GET /posts` is paginated.** It returned every post, with a response size
  that grew with the store and no signal until it was already a problem.
- **The `GET /posts` fallback scan reports truncation.** Its 10,000-event
  ceiling was silent, and a truncated scan produces folds that are *wrong*
  rather than merely short. Crossing it now logs at `error`.
- **The stylesheet freshness check no longer writes to the working tree.** It
  compiled over the committed files and restored them on mismatch, so an
  interruption left generated CSS staged over the committed CSS.
- **The WASM boundary check runs `cargo tree` twice instead of ten times.** The
  tree depends only on the app, but the loop was nested per server crate.

### Added

- `Dockerfile` and `.dockerignore` — the repository had no container build, so
  it went from `git clone` to `localhost` and stopped.
- `LICENSE` (MIT). `Cargo.toml` declared the licence; the file was absent, which
  on a public repository means all rights reserved.
- `GET /ready` — readiness, reporting Core reachability and whether the
  `posts_v1` projection has caught up, `503` when it should not receive traffic.
  Liveness and readiness answer different questions and now have different
  endpoints.
- `LOG_FORMAT=json` — structured logs, so the fields already attached to spans
  reach an aggregator as fields.
- `TRUSTED_PROXY_HOPS` configuration.
- `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODEOWNERS`, a pull request
  template, `.editorconfig`, and Dependabot.
- An opt-out marker (`predecessor-mention-ok`) for the predecessor-stack grep,
  so a document can discuss what this replaced without failing CI.

## [0.1.0] — foundation

The initial all-Rust workspace on AllSource: event schemas with golden fixtures,
the domain and DTO layers, the Dioxus component kit, the AllSource integration
with a live contract suite, better-auth over AllSource, and `cargo xtask ci` as
the single gate.
