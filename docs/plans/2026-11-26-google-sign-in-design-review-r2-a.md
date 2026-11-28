design-sha256: 7022e12aa29825c2b442a4cd96a6562b836c89586cb3ad7495c54e044963f81e

# Review A, round 2: Google sign-in design (2026-11-26)

Checked against the round-2 design and the tree at this head: `apps/api/src/infrastructure/auth/{oauth_routes,oauth_glue,better}.rs`, `apps/api/src/lib.rs`, `crates/rv2-shared/src/config.rs`, `apps/api/tests/google_sign_in.rs` (test names only), `better-auth-api-0.10.0` (`src/plugins/oauth/{mod,handlers}.rs`, `email_password.rs` routes), `crates/better-auth-allsource/src/adapter.rs`.

Not checked: tests were not run; the Core query ordering (open item 5); the `set-cookie` header shape of the library's session response; the §5.4 edit in the foundation doc.

## Round-1 findings

| # | Round-1 finding | Status | Evidence |
|---|---|---|---|
| 1 | HEAD on callback bypasses gate (blocking) | closed | `oauth_routes.rs:321` matches GET and HEAD; tests `a_head_callback_without_the_pending_cookie_creates_no_session` and `a_head_callback_with_the_cookie_runs_the_signed_flow` |
| 2 | Expired verification rows never swept | accepted as stated limit | Design "Verification rows are not swept" states rows persist; `delete_expired_verifications` has no caller (defined only in `crates/better-auth-allsource/src/adapter.rs`). Purge is a follow-up |
| 3 | Pre-registration squat | open | Design lists options (A)/(B)/(C) and asks the user to choose (Open item 1). Same-case squat test exists (`a_google_identity_cannot_take_over_a_password_account`). No decision recorded. Non-blocking while stated, but a launch decision owed |
| 4 | Email matching case-sensitive | closed | `oauth_glue.rs:142` `normalise_email`; `better.rs:136` applies it to the Google identity; `oauth_routes.rs:359-377` `normalise_credentials` rewrites the sign-up and sign-in `email`; test `a_mixed_case_password_account_is_not_duplicated_by_google` |
| 5 | Non-atomic single use | accepted as stated limit | In-process `OAuthRuntime::claim` at `oauth_routes.rs:74`, called at `:242` before the library. Concurrency test `concurrent_callbacks_with_one_cookie_admit_one_session`. Cross-process window stated in Decisions |
| 6 | Core scan limit may miss rows at scale | accepted as stated limit | Design Open item 5 states it as a launch limit. Ordering not verified against Core |
| 7 | E6 reversibility cites a nonexistent path | closed | E6 now reads "none in this change"; `better.rs:51` `UNMOUNTED_USER_ROUTES` (`DELETE_USER`, `DELETE_USER_CALLBACK`) |
| 8 | Test matrix gaps | closed | Rows added and present: provider mismatch (`a_cookie_naming_another_provider_is_refused`), HEAD, no code (`a_declined_consent_with_no_code_is_refused`), tainted targets, mixed case. Allowlist-shrunk is a unit row (`an_origin_removed_after_sealing_is_refused_at_the_callback`). Three unit rows declined, see C4. Duplicate cookie stated in Violations (but see B1) |
| 9 | Legitimate encoded targets refused | accepted as stated limit | Open item 4. `admit_path` `oauth_glue.rs:175-189` decodes the whole path and refuses space and non-ASCII |
| 10 | Grep as the sink proof | closed | Design §4 names the integration rows as the proof; grep kept as a check |
| 11 | Citation drift (E2) | closed | E2 cites `oauth_glue.rs:249` (`set_cookie`, verified at 249) and the call at `oauth_routes.rs:184` (verified) |
| 12 | "Primary origin" ambiguous | closed | Review history A12 and design text; `oauth_glue.rs:61-66` `ReturnTarget::home` uses `allowlist.first()`; `config.rs:273-285` defaults `OAUTH_RETURN_ORIGINS` to `CORS_ORIGINS` |
| 13 | id_token claim incomplete | closed | Design "No nonce" section states it is stored encrypted and not validated |
| 14 | `provider` path parameter missing from taint | closed | Design §3 "Path `provider` on `start` and `callback`" |

Counts: closed 9 (1, 4, 7, 8, 10, 11, 12, 13, 14; finding 8 has three unit rows declined, see C4), open 1 (3), accepted as stated limit 4 (2, 5, 6, 9).

## New findings, blocking

**B1. Planted valid pending cookie completes a login CSRF. The design's "denies the flow; cannot forge" claim omits this.**
- Design §5 (Violations, lines 203-205) says a duplicate `rv2_oauth_pending` "denies the flow; it cannot forge one". Forgery is indeed impossible. But the cookie is not bound to the browser beyond its name, so a valid sealed cookie minted by the attacker's own `start` is accepted if a sibling subdomain can set it.
- Code: `oauth_glue.rs:251` sets `Path=/auth` with no `Domain` (correct for this host), but `read_cookie` (`oauth_glue.rs:262-267`) accepts any cookie with that name and takes the first. A sibling subdomain of the same registrable domain can set `rv2_oauth_pending` with `Domain=<parent>` and `Path=/auth`.
- Attack, given sibling-subdomain control: attacker runs `start` in their own browser (540 s window), keeps the sealed cookie, plants it in the victim's browser from the sibling host, then sends the victim `/auth/callback/google?code=C_attacker&state=S_attacker`. With no real cookie for the victim, `open`, `state`, `claim` and the library all pass. The victim's browser ends with the attacker's Google session. The design says this path is "denial only".
- Fix: rename the cookie to `__Host-rv2_oauth_pending`. The `__Host-` prefix requires `Secure`, `Path=/`, and no `Domain`, so a sibling subdomain cannot set it. Change `oauth_glue.rs:18`, `:251`, `:257`, `read_cookie`, the unit test at `:450-453`, and design §3, §5 and E2. Alternatively bind the sealed value to a browser-held nonce echoed in the callback. Restate Violations to name login CSRF as the consequence of a duplicate or planted cookie. Add a unit row that the callback reads only the `__Host-` name.

## Carried to Plan

- C1 (should). `/auth/refresh-token` and `/auth/get-access-token` are mounted and not gated (`better-auth-api` `oauth/mod.rs:58-59`; not in `better.rs:51` `UNMOUNTED_USER_ROUTES`). `refresh_token_core` (`handlers.rs:233-263`) posts to the configured `token_url` with the session user's stored refresh token. Not a tainted sink (`provider_id` is checked against the configured providers; account is session-scoped), but the design says refresh tokens are out of scope and that every library call goes through the gate. Either disable both paths or list them in the §1 inventory and §4.
- C2 (should). A HEAD with a valid cookie runs the flow and consumes the provider code and the state claim. A later GET with the same cookie lands on home (`AlreadyUsed`). Design row "HEAD callback with the cookie" should state this, or the gate should reject HEAD on the signed path.
- C3 (should). `oauth_routes.rs:283-288` forwards only the first `set-cookie` header from the library response. Confirm the library emits exactly one session cookie, or forward all of them.
- C4 (nit). Unit rows declined in round 2: path longer than `MAX_PATH_LEN` (`oauth_glue.rs:24`, `:192`), `\` at index greater than 1, NUL byte. Review history records them as not added. Add them to the glue tests (`oauth_glue.rs:344-436`) to close the gap.
- C5 (should). Squat decision (finding 3, open item 1) must be recorded before launch. Design recommends (C); user decides.
- C6 (nit). Verification section reports test counts from a run; this review did not rerun them.

verdict: findings
