design-sha256: 6f950a726f95d26563fd76480fd8724da42ad714537ef51bf119cf625ef378c3

# Review round 3 (final): Google sign-in design, Amendment 1

Fresh-context third review. Checks the round-2 findings against design and code, then
looks for new blocking findings. Library source read from
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` (better-auth-0.10.0,
better-auth-core-0.10.0, better-auth-api-0.10.0). Nothing compiled or run.

Library facts checked:
- `before_request` runs after `run_before` middleware and the disabled-path check, for
  every request `handle_request` sees (`better-auth-0.10.0/src/core/auth.rs:263-298`).
- The axum router registers only enabled paths and every handler calls `handle_request`
  (`better-auth-0.10.0/src/handlers/axum.rs:85-117`, `:161`).
- `require_session` and `extract_session_token` take `Authorization: Bearer` first, then
  the cookie (`better-auth-core/src/plugin.rs:243-254`, `session.rs:231-250`). Neither
  checks the revocation cutoff.
- Session-reading routes mounted after the disabled list: `POST /sign-out` only
  (`better-auth-api/src/plugins/session_management.rs:55`, `:167-172`). `/change-password`
  and `/set-password` belong to `PasswordManagementPlugin`, which is not mounted.
- `auto_sign_in` defaults to true (`better-auth-api/src/plugins/email_password.rs:476`),
  so sign-up mints a session in production.

## Status of round-2 findings

| # | Round-2 finding | Status | Evidence |
|---|---|---|---|
| B1 | Library session routes read voided sessions | closed for the listed routes; the invariant is still open via N1 | `get-session`, `list-sessions`, `revoke-*` in `UNMOUNTED_SESSION_ROUTES` (`better.rs:68-75`); 404 test `tests/google_sign_in.rs:1175-1184`. The cookie check still does not cover Bearer (N1). |
| B2 | `/auth/sign-in/username` bypasses the guard | closed | Unmounted (`better.rs:74`); `the_username_sign_in_route_is_not_mounted` (`tests/google_sign_in.rs:1189-1201`). |
| C1 | Not compiled | open | Static read found no error in the items listed (`AuthConfig` derives Clone, `config.rs:24`; `create_session` at `session.rs:31`; `metadata()` at `entity.rs:53`). Not built (disk). |
| C2 | Audit outage leaves squatter live | accepted (stated limit); widened, see K3 | `oauth_routes.rs:403-412`. |
| C3 | Domain data carries to owner | accepted | Plan 15. |
| C4 | Clock skew between S and T | accepted | Plan 14. N3 is a same-process ordering bug, not skew. |
| C5 | "Verified keeps both" is test-only | accepted | Plan 17. |
| C6 | `allsource-auth` off by default | open | `apps/api/Cargo.toml` feature list; production feature set not confirmed. |
| C7 | Live-Core test not run | open | `tests/google_sign_in.rs:887-925`. |
| C8 | Test gaps | open | Bearer route (N1), sign-up window (N2) and revoke-write window (N3) have no tests. |
| C9 | Check-then-create email race | accepted | Plan 3. |
| C10 | Legacy mixed-case rows | accepted | Plan 4. |
| C11 | Scan limit | accepted | Plan 6. |

## Blocking (new)

1. **N1. The cutoff check on library routes reads a cookie only, so a voided session
   passes with `Authorization: Bearer`.**
   `revocation.rs:52-57` takes the token from the `cookie` header alone. The library
   takes Bearer first (`session.rs:233-237`). A request with `Authorization: Bearer <voided>`
   and no cookie passes `before_request` (`revocation.rs:52`, `Ok(None)`), and
   `POST /auth/sign-out` runs `require_session` (`session_management.rs:172`) and
   deletes the row, returning 200. The design's claim "a request carrying a void session
   gets 401" and "every route passes" (design lines 386-392, 407-411) is false. The
   test `a_voided_session_is_refused_on_every_route_that_reads_it`
   (`tests/google_sign_in.rs:1144-1185`) sends the cookie only.
   Fix: in `RevocationPlugin::before_request`, take the token with
   `ctx.session_manager().extract_session_token(req)` (`session.rs:231`), the same
   extractor `require_session` uses. Add a Bearer case to the test. Amend design
   lines 202 and 386-392.

2. **N2. Sign-up mints a session after the account exists, and no guard covers it. A
   revocation in that gap leaves the squatter's session live on the owner's account.**
   `handle_sign_up` calls `create_user` (`email_password.rs:322`), then
   `create_session` (`:324-327`). The Google callback can find that account by email,
   link to it, and revoke it between the two calls (`settle_email_match`,
   `oauth_routes.rs:382-420`). The sign-up session is then created after the cutoff, so
   `session_void` (`revocation.rs:67-78`) and `ExtractAuthUser` (`middleware.rs:115`) keep
   it. The guard runs only for `canonical == "/auth/sign-in/email"`
   (`oauth_routes.rs:509-511`). The design's "the account is new, so no revocation can
   predate it" (design line 200, and 290-292) overlooks a revocation that predates the
   session but follows the account.
   Fix: extend `guard_password_sign_in` to `/auth/sign-up/email` (the new account has a
   hash, so the same decision applies). This is correct only with fix N3 in place.
   Test: hold the sign-up after the library returns (`SignInHold`), revoke, release,
   assert 401 and no session.

3. **N3. The revocation cutoff is timestamped before its write lands. A password sign-in
   whose session is created in that gap survives.**
   `revoke_password` takes `revoked_at = Utc::now()` (`oauth_routes.rs:429`) and then
   `update_user` writes it with the hash clear (`:430-437`). Interleaving: T (cutoff
   taken) < s (sign-in session created) < g (gate reads the user, hash still present, no
   cutoff yet) < w (write lands). The session's `created_at` is after T, so
   `session_void` does not void it. At g the gate sees the hash and no cutoff, so
   `sign_in_survived` (`oauth_glue.rs:305-311`) keeps it. After w the hash is gone and
   the cutoff is in place, yet the session still stands. Design lines 407-416 say "a
   revocation after S is caught by the cutoff", and that holds only if the cutoff
   becomes visible before the guard reads. The race is the width of the
   `update_user` round trip, and the attacker can drive many sign-ins into it.
   Fix: the cutoff must not be earlier than the write that clears the hash. Either
   clear the hash first, then write the cutoff with a timestamp taken after that write
   returns (a second update), or use a store-assigned timestamp (Plan 14). Test: a
   `SignInHold`-style seam inside `revoke_password` between T and w, with a sign-in
   started before it. Expect no session.

## Carried to Plan

C1 to C11 as in the status table: C1, C6, C7 open; C8 open (tests for N1 to N3);
C2, C3, C4, C5, C9, C10, C11 accepted.

K1. Stale comment `apps/app/src/routes.rs:3` says `AppShell` gates on
    `GET /auth/get-session`. The client reads `/me` (`crates/rv2-client/src/lib.rs:170`).
    Not a functional break; fix the comment.

K2. Line drift in the design. `UNMOUNTED_OAUTH_ROUTES` is `better.rs:62` (design `:61`),
    `UNMOUNTED_USER_ROUTES` `:52` (design `:51`). In `oauth_routes.rs`: password clear
    and cutoff `:427-439` (design `:418`), best-effort delete `:440` (design `:427`),
    `record_audit` `:346` (design `:333`), password-hash early return `:399` (design
    `:381`), owner session `:415-419` (design `:410-413`).

K3. Settle is skipped on any library non-200 answer, a missing `Set-Cookie`, or a
    failed `get_user_by_id` after the library's link (`oauth_routes.rs:281-296`). The
    squatter then stays live until the next verified callback. This is C2 widened from
    audit outage to any post-link failure. The design should name it.

K4. `revoke_password` audit can record a revocation whose write then fails
    (`oauth_routes.rs:403-412` before `:427-439`). The audit log is then wrong about
    state. Record the audit after the write, or state it.

K5. Two callbacks for two different states on one unverified account both revoke. The
    second cutoff voids the first owner session. Availability only, not a takeover.
    Note it.

verdict: findings
