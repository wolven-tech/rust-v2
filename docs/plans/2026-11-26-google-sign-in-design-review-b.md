design-sha256: ef9f4d82bb2b73317bcdac355fc3723b4d1f2bfadacd18722240373fc99301ef

# Review B: Google sign-in design

Scope: design file above, checked against `apps/api/src/infrastructure/auth/oauth_glue.rs`,
`oauth_routes.rs`, `better.rs`, `apps/api/src/lib.rs`, `crates/rv2-shared/src/config.rs`,
and the better-auth 0.10.0 and axum 0.8.9 sources in `~/.cargo/registry`. The review did not
run the tests or the gate. Claims about test results ("43 passing") are unverified.

## Findings

1. **blocking. The gate does not cover HEAD on the callback.** The design says the gate
   "answers both paths itself and never lets them through" (design §Decisions, "Start and
   callback own the browser-facing behaviour"). `oauth_routes.rs:289` tests
   `request.method() == Method::GET` only. axum 0.8.9 `routing/method_routing.rs:1157-1158`
   sends HEAD to the `get` handler when no HEAD handler exists. better-auth 0.10.0
   `handlers/axum.rs:95-96` registers `get(...)` for every GET route, and its doc comment on
   `method_routing.rs:55` says the same. So `HEAD /auth/callback/google?code=C&state=S` skips
   the gate, reaches `handle_callback` (better-auth-api `oauth/handlers.rs:664-688`), which
   has no cookie check, consumes the state, creates the user and account, and returns
   `Set-Cookie` for a session. The invariant "callback without the pending cookie is
   refused" does not hold for HEAD. Fix: in `gate`, match the callback path for any method
   other than GET and answer 404 (or 405) for it. Add an integration row: HEAD callback with
   valid code and state and no cookie, expecting 404 and no session row. Add the same row for
   `HEAD /auth/sign-in/social` and `HEAD /auth/link-social`, which the POST-only gate also
   misses.

2. **should. Provider-mismatch row missing from the matrix.** The sink S4(c) check
   (`oauth_routes.rs:202-204`) is in the design's VERIFY table, and the design requires provider
   mismatch coverage, but neither the unit table (design §6, oauth_glue) nor the integration
   table has a row for it. Only `google` is offered today, so the route cannot reach the
   mismatch branch. Add a unit test that seals `provider: "github"` and calls `complete` (or
   the sealed `Pending` check) against a `google` route, expecting refusal with no session.

3. **should. Mixed-case email is not covered and may create a second user.** The design claims
   "no account linked" for a password user with the same email (design §Decisions, and integration
   row "Google email matches an existing password user"). The lookup is
   `ctx.database.get_user_by_email(&user_info.email)` (better-auth-api `oauth/handlers.rs:595`)
   with an exact string. Neither `email_password.rs` nor `oauth/handlers.rs` lowercases the
   address (rg for `to_lowercase|to_ascii_lowercase` returns nothing), and
   `require_email_verification` defaults to false (`email_password.rs:473`). A password account
   created as `Victim@…` does not match Google's `victim@…`. The callback then takes the
   "create a new user" branch (`handlers.rs:636-641`) with a verified email, so two users share one
   address. Fix: either normalise emails on both write paths, or state the limit in the design
   and add a row with mixed-case input.

4. **should. Expired pending rows are not "deleted after 10 min".** Design E5 says the
   verification row is "deleted on callback or after 10 min". In the code,
   `delete_verification` runs only on a successful lookup (`handlers.rs:433`). Nothing in
   `handlers.rs` sweeps unused rows. Correct the E5 reversibility column to "deleted on callback;
   unused rows persist until swept by the adapter (not verified)". Check the adapter for a TTL
   sweep before writing this down.

5. **should. Start route writes rows without a rate limit.** `start` is mounted on the root router
   before `merge(domain_routes)` (`lib.rs:94-98`). The rate limiter layer is applied only to
   `domain_routes` (`lib.rs:83-86`), and `/auth/*` is nested outside it too. An unauthenticated
   client can therefore create unlimited verification rows (E5) and Google redirects through
   `/auth/oauth/google/start`. Add a rate-limit layer to the OAuth routes and list E5 growth
   under Violations.

6. **should. Account-linking default is asserted, not checked.** Design §Decisions says
   "The library default is `account_linking.enabled = true`". I did not read
   `AccountLinkingConfig::default()` in better-auth-core 0.10.0. The code path (`handlers.rs:595-633`)
   refuses when linking is disabled, so the behaviour is right either way. Read the default and
   cite the line, or drop the claim.

7. **should. Design cites a decision and a doc that were not checked here.** The design says it
   "Implements decision D18 and §5.4". §5.4 at `docs/architecture/001-rust-v2-allsource-foundation.md:1032-1046`
   names the cookie `oauth_origin`, not `rv2_oauth_pending`. The name change is not an
   overturned decision, but the doc should record it, per the CLAUDE.md rule "If you overturn a
   decision, edit the decision". D18 (`001-...md`) was not read.

8. **nit. Library and cookie expiry race.** The library state is created at
   `oauth_routes.rs:115-123` and the cookie is sealed at line 153, so the cookie expires about a
   second after the library state. The result is a refusal, not a bypass. Say so in the design,
   or set the cookie TTL a few seconds shorter.

9. **nit. Duplicate cookie names.** `read_cookie` (`oauth_glue.rs:251-256`) takes the first match.
   A sibling-path cookie with the same name can shadow the real one and cause a refusal. The HMAC
   stops forgery, so this is a denial of the flow only. Mention it in Violations.

10. **nit. Test count claim.** "unit matrix (present, 43 passing)" (design §Verification) was not
    checked. The unit tests in `oauth_glue.rs:258-414` exist. The count needs a `cargo test` run
    before merge.

Verified and consistent with the code: E1/E2/E9 line numbers (`oauth_routes.rs:157`, `:174`,
`:178`); S2 `admit` and `admit_path` (`oauth_glue.rs:141`, `:164`); the decoded-form refusals
(`oauth_glue.rs:168`, tests at `:333-361`); the cookie attributes (`oauth_glue.rs:238-242`);
the `verified_google_user` refusal before any write (`better.rs:128-131`, called before the branch
at `handlers.rs:506`); the library's state delete on use (`handlers.rs:433`); the `Pending` expiry
check (`oauth_glue.rs:129`); and the config fallback `OAUTH_RETURN_ORIGINS` to `CORS_ORIGINS`
(`config.rs:280-284`).

Not checked: the `AllsourceAuthAdapter` expiry handling, the rate-limit module, the
`apps/app` login view, and the `docs/guides/` file the design says is written in Build.

verdict: findings
