design-sha256: de3fa6a694af66c0e72ebe0d79038fec65874dae0406db7b174ddb681b00e8ac

# Review B, round 3: Google sign-in design

Scope: round-3 design above, checked against `apps/api/src/infrastructure/auth/oauth_routes.rs`,
`oauth_glue.rs`, `better.rs`, `apps/api/src/lib.rs`, `crates/rv2-shared/src/config.rs`,
`apps/api/tests/google_sign_in.rs`, `docs/architecture/001-rust-v2-allsource-foundation.md`, and the
better-auth 0.10.0 / better-auth-api 0.10.0 / better-auth-core 0.10.0 sources in `~/.cargo/registry`.
The test suite was not re-run here. Counts come from `rg` over the test attributes.

## Round-2 findings and carried items

| # | Round-2 item | Sev | Status | Evidence |
|---|---|---|---|---|
| 1 | Gate does not cover HEAD on callback | blocking | closed | Non-GET on the callback path is 404: `oauth_routes.rs:321-323`. Tests `google_sign_in.rs:435` (HEAD with and without cookie, state unspent, later GET succeeds) and `:521` (POST/PUT/DELETE/HEAD on callback). |
| 2 | Provider-mismatch row missing | should | closed | Refusal at `oauth_routes.rs:226-228`. Row `google_sign_in.rs:656`. |
| 3 | Mixed-case email, possible second user | should | closed, stated limit | Credential rewrite `oauth_routes.rs:359-377`; Google identity `better.rs:144`; `normalise_email` `oauth_glue.rs:145-147`. Rows `google_sign_in.rs:725` and unit `oauth_glue.rs:437`. Legacy lockout stated in design Email form. |
| 4 | E5 "deleted after 10 min" false | should | closed | Design Decisions now says rows persist and lookups filter on expiry. No caller of `delete_expired_verifications` in the workspace. |
| 5 | Start route has no rate limit | should | closed | Gate `oauth_routes.rs:333-338`; callback `:324-326`. Row `google_sign_in.rs:801`. Growth now under Violations. Consequence behind a proxy: see N5. |
| 6 | Linking default asserted, not read | should | closed | better-auth-core-0.10.0 `config.rs:240-243` sets `enabled: true`; `better.rs:107` sets false; refusal at better-auth-api `plugins/oauth/handlers.rs:629-632`. |
| 7 | D18 / §5.4 mismatch | should | closed | `docs/architecture/001-rust-v2-allsource-foundation.md:1049-1054` records `__Host-rv2_oauth_pending` and the departures. |
| 8 | Cookie and state expiry race | nit | closed | `PENDING_TTL_SECS = 540` at `oauth_glue.rs:22`; library state 600 s. Design Decisions "Expiry" states the order. |
| 9 | Duplicate cookie shadows the real one | nit | closed | `pending_cookie` refuses any second value, empty ones included: `oauth_glue.rs:265-279`. Unit rows `oauth_glue.rs:489-498`. Residual scope: N1. |
| 10 | "43 passing" count unverified | nit | closed by count | `google_sign_in.rs` has 23 `#[tokio::test]`, matching design. `apps/api/src` has 43 test attributes, matching design "api lib 43". Not re-run. |
| C1 | HEAD row for `/auth/link-social` | should | closed | Row `google_sign_in.rs:532`; design method table line 304. |
| C2 | Duplicate-cookie wording, planted-value limit | should | reworded, residual still open (carried) | Design Violations lines 234-240 now state the single planted value is accepted. The heading "Login CSRF, refused" overstates; see N7. The impact of the accepted case is not spelled out; see N7. |
| C3 | Email uniqueness race | should | closed as stated limit | Design Violations lines 246-253; Plan bead 3. |
| C4 | Google code single-use backstop | should | closed | Design Decisions lines 122-127. Backstop is correctly described. |
| C5 | Gmail aliases | should | closed | Design squat rule and Email form; Plan bead 8. |
| C6 | ASCII-only folding | nit | closed | Design Email form lines 94-96; `oauth_glue.rs:146` is `to_ascii_lowercase`. |
| C7 | Legacy mixed-case lockout | should | closed | Design Email form lines 101-106 state the lockout as well as the non-match; Plan bead 4. |
| C8 | E5 growth under Violations | nit | closed | Design Violations lines 243-245. |
| C9 | Test run recorded | nit | closed by claim, not reproduced | Design Verification gives counts that match the tree (see 10). Build gate still needs the recorded `cargo test` output (Plan bead 12). |

## New findings

None at `blocking` severity.

Checked and held: every sink in the S1-S8 table has its VERIFY on the path the table names.
The OAuth plugin registers only `POST /sign-in/social`, `GET /callback/{provider}` and
`POST /link-social` plus the two token routes (better-auth-api `plugins/oauth/mod.rs:55-59`).
The token routes are removed at route registration (better-auth-0.10.0 `handlers/axum.rs:89`), and
the axum adapter has no fallback to `handle_request`. The gate is layered after the nest and the
start merge (`lib.rs:94-106`), so it sees all methods on those paths. `library(` is called only from
`start` (`oauth_routes.rs:142`) and `complete` (`:261`). `admit_path` checks the raw and
percent-decoded forms with three decode passes and a host-unchanged join (`oauth_glue.rs:178-192`).
Claim order is cookie, provider, state, code, claim, admit, library (`oauth_routes.rs:221-273`), so a
refused request spends nothing. Concurrent callbacks are serialised by `claim` before the library.
Account linking is off, so a verified Google email that matches a user row is refused before any write
(`handlers.rs:595-632`). Returning Google users are matched by account first (`handlers.rs:554-590`).

## Carried to Plan

- **C2 (should, residual wording).** Accepted limit: one planted, attacker-sealed value in a browser
  that carries it, with the attacker's own code and state, signs the victim into the attacker's
  account (login CSRF). Browsers prevent planting a `__Host-` cookie from another site, so this is a
  stated assumption, not a reachable path in a normal browser. State the impact in the Violations bullet
  and retitle its heading from "refused" to "accepted residual".
- **N1 (should).** The duplicate check reads only the first `Cookie` header: `oauth_routes.rs:217-221`
  uses `headers.get(header::COOKIE)`. A second `Cookie` header with the pending value is never seen.
  The design's S4(a) and the unit name "refused wherever it sits" (`oauth_glue.rs:489`) hold only inside one
  header. A browser cannot hold two same-name `__Host-` values, so this is not reachable from a browser.
  Fix: iterate `headers.get_all(header::COOKIE)` and join; add an integration row with two `Cookie` headers.
- **N2 (should, not in design).** Non-atomic create: `handlers.rs:641` creates the user, `:647` creates the
  account. If the second write fails, a verified user row with no account remains. Every later Google
  sign-in for that identity then finds no account (`:554-557`), finds the user by email (`:595`), and is
  refused with linking off (`:630-632`). That identity has no password and no reset path, so the lock-out
  is permanent without manual store intervention. Add to Violations and a Plan bead: compensate on failure
  or make the pair idempotent.
- **N3 (should).** Taint list (design lines 204-205) names `name` and `picture` as remote data written to
  E6/E7, but S5 (line 219) has no VERIFY for them. `verified_google_user` passes them through
  (`better.rs:145-146`). Add a row: length cap, and state that the UI renders them as escaped text (or
  does not render `image` as a URL). Password sign-up already accepts an arbitrary `name`, so this adds
  no new exposure, but the VERIFY table should cover every sink it lists.
- **N4 (should).** Design Decisions and E0 say the token routes answer 404 "for every method". The
  test covers POST only (`google_sign_in.rs:533-534`). Add GET, HEAD, PUT and DELETE rows for both paths.
- **N5 (should).** The limiter keys on the peer address when `TRUSTED_PROXY_HOPS = 0` (`rate_limit.rs:45-46`,
  `:58-60`). Behind a load balancer every client shares one bucket of 60 rps and 120 burst (`state.rs:98`),
  so one abusive client throttles every sign-in. Design lines 149-150 state the key but not this launch
  consequence. State the required launch setting for `TRUSTED_PROXY_HOPS` in the launch notes.
- **N6 (nit).** Design lines 46-48 say any refusal on the callback path ends at 302 home with the pending cookie
  cleared. HEAD returns 404 (`oauth_routes.rs:321-322`) and rate limiting returns 429 (`:325`), and neither
  clears the cookie. Reword to name those two cases.
- **N7 (nit).** Design line 234 heading "Login CSRF, refused" is accurate only for the two-value case.
  Retitle with the accepted residual (see C2).

Count carried: 8 (C2, N1 to N7).

verdict: clean
