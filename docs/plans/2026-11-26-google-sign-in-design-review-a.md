design-sha256: ef9f4d82bb2b73317bcdac355fc3723b4d1f2bfadacd18722240373fc99301ef

# Review A: Google sign-in design (2026-11-26)

Reviewed against: `apps/api/src/infrastructure/auth/oauth_routes.rs`, `oauth_glue.rs`, `better.rs`, `apps/api/src/lib.rs`, `crates/rv2-shared/src/config.rs`, `crates/better-auth-allsource/src/{adapter,client}.rs`, `vendor/better-auth/src/handlers/axum.rs`, `better-auth-api-0.10.0` OAuth plugin (`src/plugins/oauth/{mod,handlers}.rs` in the cargo registry), `axum-0.8.9` method routing, and `docs/architecture/001-rust-v2-allsource-foundation.md` §5.4 and D18.

Not checked: `better-auth-api` email/password signup normalisation beyond a grep; the sign-out handler in better-auth core; the HKDF in `encryption.rs`; the Core query ordering for `scan_family` (client.rs:240-250 not fully read); the claimed "43 passing" count.

## Findings

1. **blocking. The gate does not cover HEAD on the callback.** `oauth_routes.rs:289` tests `request.method() == Method::GET` only. The better-auth plugin registers the callback as `get(...)` (`axum.rs:96`), and axum routes HEAD to `get` handlers (`axum-0.8.9/src/routing/method_routing.rs:55-56`, `1157-1158`). `HEAD /auth/callback/google?code=C&state=S` with no `rv2_oauth_pending` cookie therefore reaches `handle_callback`, runs `callback_core`, and returns the session `Set-Cookie`. This breaks the invariant the design states at lines 40-41, 72-77 and S4 ("never lets them through"). Fix: match `Method::GET | Method::HEAD` in `gate`, and add an integration row: HEAD callback without cookie → 302 home, no session cookie.

2. **should. Expired verifications are never deleted.** E5 says the verification row is "deleted on callback or after 10 min". Callback does `delete_verification` (a tombstone, `handlers.rs:433`). Nothing deletes rows that expire unused. `delete_expired_verifications` is defined at `adapter.rs:660` and declared in `better-auth-core/src/adapters/traits.rs:103`, and I found no caller in `better-auth-api`, `better-auth-core`, `vendor/better-auth`, `apps/api/src` or `crates/rv2-jobs`. Abandoned flows accumulate in `auth-verification`, and every callback scans them. Fix: correct E5, and add an idempotent purge to `rv2-jobs` (safe on every instance per CLAUDE.md) or call the adapter method from an existing job.

3. **should. Pre-registration squats a Google sign-in.** `better.rs:97` disables linking. `handlers.rs:630-632` then refuses any Google sign-in whose email matches an existing user. Anyone who registers a victim's address with a password first locks that victim out of Google sign-in with no recovery path. The design says "the user keeps signing in the way they did before", which is false for a user who never had an account. Either decide this is accepted for launch and say so in Decisions, or require verification of the password account before the squat matters. Add a row for "new Google user, email already held by an unverified password account".

4. **should. Email matching is case-sensitive.** `client.rs:309` compares `payload.get(field) == Some(value)` exactly. `handlers.rs:595` calls `get_user_by_email` with Google's address. A grep of `better-auth-api/src/plugins/email_password.rs` found no lowercasing. A password account created as `Victim@x.com` does not match Google's `victim@x.com`, so `callback_core` creates a second user. That contradicts S5's "cannot attach to an existing email account" for case variants. Fix: lowercase on write and on lookup (adapter or `verified_google_user`), and add a mixed-case row to the matrix.

5. **should. S4(d) overstates single use.** `callback_core` does `get_verification_by_identifier` (`handlers.rs:403-405`), then `delete_verification` (`handlers.rs:433`). These are separate non-atomic operations. Two concurrent callbacks with the same cookie can both pass before the tombstone lands. Sequential replay is refused, as the integration row says. Concurrent replay is not covered by the claim. Fix: state the window honestly, or serialise on `state` in the gate.

6. **should, unverified. Lookup may miss new verifications at scale.** `client.rs:15` sets `SCAN_LIMIT = "10000"` and `client.rs:247` passes it as `limit`. `client.rs:56-59` says events are read oldest-first. If Core returns the first 10000 events, a new `auth.verification` cannot be found once the family passes that count (each flow is two events: create plus tombstone), and new callbacks fail closed. I did not confirm the ordering parameter in the query. Fix: confirm the Core ordering, then either paginate or record the ceiling as a launch limit in Verification.

7. **should. E6's reversibility cites a path that does not exist.** E6 says "deletable through the existing admin path". `UNMOUNTED_USER_ROUTES` (`better.rs:49-54`) removes `DELETE_USER`, and no user-delete handler exists under `apps/api/src/handlers` (only `posts::delete` at `lib.rs:72`). Fix: say no delete path exists, or name the one that does.

8. **should. Test matrix gaps.** Missing rows:
   - provider mismatch between sealed `provider` and route (checked at `oauth_routes.rs:202`, untested);
   - HEAD method (see 1);
   - no `code` in the callback, e.g. consent declined or `error=access_denied` (`oauth_routes.rs:214-216`);
   - allowlist shrunk after sealing, which re-runs `admit` at `oauth_routes.rs:218`;
   - duplicate `rv2_oauth_pending` cookies (`read_cookie`, `oauth_glue.rs:251-256`, takes the first; a sibling subdomain can set one);
   - path longer than `MAX_PATH_LEN` (`oauth_glue.rs:24`, `181`);
   - `\` not at index 1 (`/posts\x`) and a NUL byte.
   Admitted cases are thin: one unit row (`/posts/abc?tab=1`). Add `/`, a query string, and a percent-encoded safe value.

9. **should. Legitimate encoded targets are refused.** `admit_path` decodes the whole path including the query (`oauth_glue.rs:168`) and then rejects spaces and non-ASCII. `/posts?q=hello%20world` and `/posts/caf%C3%A9` are refused and land on home. This fails closed, so it is safe, but the design's "cleared inputs admitted" intent does not hold for these. Either document it as a stated limitation (CLAUDE.md asks for this) or allow decoded query characters.

10. **nit. Sink verification by grep is weak.** `rg -n 'handle_request' apps/api/src` matches the library calls in `start` (`oauth_routes.rs:115`) and `complete` (`:233`), which is expected. It does not prove that only the gate reaches them. Use the HEAD and method tests from 1 and 8 as the proof.

11. **nit. Citation drift.** E2 cites "`oauth_glue::set_cookie`, line 157". The call is `oauth_routes.rs:157`. `set_cookie` is defined at `oauth_glue.rs:238`.

12. **nit. "primary origin" is ambiguous.** `ReturnTarget::home` uses `allowlist.first()` (`oauth_glue.rs:63`), and the allowlist is `OAUTH_RETURN_ORIGINS` (`config.rs:273-285`), not `CORS_ORIGINS`. Say so.

13. **nit. The id_token claim is incomplete.** The design says there is "no id_token to validate". `handlers.rs:478` reads `id_token` from the token response and stores it encrypted (`handlers.rs:645`). State that it is stored unvalidated.

14. **nit. Taint sources omit the `provider` path parameter.** `oauth_routes.rs:91` takes it and it reaches the library body (`:112`) and the sealed payload (`:153`). It is bounded by `offers()` at `:94`. Add it to §3.

## Counts

blocking: 1. should: 8 (2-9). nit: 5 (10-14).

verdict: findings

verdict: findings
