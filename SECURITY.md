# Security

## Reporting a vulnerability

Report privately through GitHub's [private vulnerability
reporting](https://github.com/wolven-tech/rust-v2/security/advisories/new).
Please do not open a public issue — a public report is a working exploit
published to everyone running this code before anyone can act on it.

Include what you would want if you received the report: what you did, what
happened, and what you think an attacker gets from it. A proof of concept helps
and is not required.

## What is in scope

This is a scaffold. The things most worth looking at are the ones a fork
inherits without reading:

- **Auth** — `apps/api/src/infrastructure/auth/`, and the vendored
  `crates/better-auth-allsource` (a version bridge; see its `PROVENANCE.md`).
- **Authorization** — every mutating handler re-reads current state before
  calling a named predicate in `rv2_domain`. A path that authorizes on the
  actor's *claim* instead is a bug.
- **The tenant boundary** — Core answers a query with no `tenant_id` with an
  empty result and HTTP 200. Any read path that reaches Core without one is
  silently wrong; see `crates/rv2-allsource/src/tenant_query.rs`.
- **CORS and session cookies** — `*` is rejected at boot because it is illegal
  alongside credentialed requests.
- **The rate limiter's client key** — `apps/api/src/infrastructure/rate_limit.rs`.

## Known limitations, stated rather than hidden

- **Rate limiting is abuse-blunting, not a security control.** It is in-memory
  and per-instance, so N instances mean N times the limit, and it is lost on
  restart. `TRUSTED_PROXY_HOPS` **must** be set to the number of proxies in
  front of the service; the default of `0` ignores `x-forwarded-for` entirely,
  which is the safe choice but keys every request behind a proxy on that proxy.
- **Google OAuth is not wired.** The seam is marked. Wiring it without the
  HMAC-signed pending-origin cookie glue produces an open redirect at the
  callback.
- **`ALLSOURCE_DEV_MODE=true` bypasses Core's API-key auth.** It appears in the
  README, `.env.example` and CI. It is for local development only.
- **Analytics and email degrade silently by design** when unconfigured. That is
  intentional, and it means a missing key is not an error you will see.

## Supply chain

`cargo deny check advisories bans licenses sources` runs on every pull request
against one root `Cargo.lock` covering every workspace member. CI asserts no
nested lockfile exists, because a member outside the workspace would not be
covered by a root-level check while appearing to be.

`deny.toml` carries a small number of `ignore` entries. Each names the crate that
pulls the advisory in, and each is an unmaintained-crate notice on a transitive
dependency rather than a vulnerability. Removing one when the path disappears is
part of the dependency bump.
