design-sha256: 7022e12aa29825c2b442a4cd96a6562b836c89586cb3ad7495c54e044963f81e

# Review B, round 2: Google sign-in design

Scope: round-2 design file above, checked against `apps/api/src/infrastructure/auth/oauth_routes.rs`,
`oauth_glue.rs`, `better.rs`, `apps/api/src/lib.rs`, `crates/rv2-shared/src/config.rs`,
`apps/api/tests/google_sign_in.rs`, `docs/architecture/001-rust-v2-allsource-foundation.md`,
and the better-auth 0.10.0, better-auth-api 0.10.0, better-auth-core 0.10.0 and
better-auth-allsource 0.14.15 sources in `~/.cargo/registry`. The tests were not run. Test counts are
from `rg` over the test file.

## Round-1 findings

| # | Round-1 finding | Sev | Status | Evidence |
|---|---|---|---|---|
| 1 | Gate does not cover HEAD on callback | blocking | closed | Gate matches GET and HEAD only, all other methods 404: `oauth_routes.rs:318-323`. Sign-in social and link-social 404 for every method: `oauth_routes.rs:340-342`. Tests: `google_sign_in.rs:433` (HEAD, no cookie, state unspent), `:462` (HEAD with cookie), `:479` (POST/PUT/DELETE callback; POST/GET/HEAD sign-in social; POST link-social). HEAD on link-social has no test row (see C1). |
| 2 | Provider-mismatch row missing | should | closed | Refusal at `oauth_routes.rs:226-228`. Integration row `google_sign_in.rs:610-625` seals `github`, calls the google route, expects home and no session. |
| 3 | Mixed-case email, possible second user | should | closed, with stated limit | Gate rewrites credential email: `oauth_routes.rs:344-351`, `:359-377`. Google identity: `better.rs:136`. Function: `oauth_glue.rs:142-144`. Row `google_sign_in.rs:678-705`. Pre-change mixed-case rows are not matched: accepted (design lines 84-86; `CHANGELOG.md` says nothing is released). See C6, C7. |
| 4 | E5 "deleted after 10 min" is false | should | closed | Design E5 and "Verification rows are not swept" now say rows persist and lookups filter on expiry. `delete_expired_verifications` (`better-auth-allsource` `adapter.rs:660`) has no caller in the library or the workspace (only the definition matches). Lookups filter on `expires_at` at `adapter.rs:606`, `:635`. |
| 5 | Start route has no rate limit | should | closed | Start and callback both checked in the gate: `oauth_routes.rs:324-326` (callback), `:333-336` (start). Row `google_sign_in.rs:754`. E5 growth is not listed under Violations (see C8). |
| 6 | Linking default asserted, not read | should | closed | `better-auth-core-0.10.0` `config.rs:241-243`: `enabled: true` in the `AccountLinkingConfig` default. Refusal with linking off: `better-auth-api-0.10.0` `plugins/oauth/handlers.rs:629-632`. |
| 7 | D18 and §5.4 mismatch | should | closed | §5.4 now records `rv2_oauth_pending`, the derived redirect URI and the departures: `docs/architecture/001-rust-v2-allsource-foundation.md:1049-1054`. D18 (line 34) says "signed pending-origin cookie" with no name, so it does not conflict. |
| 8 | Cookie and state expiry race | nit | closed | Cookie TTL 540 s: `oauth_glue.rs:22`. Library state 10 min: `better-auth-api-0.10.0` `plugins/oauth/handlers.rs:359`. Design "Expiry" paragraph states the ordering. |
| 9 | Duplicate cookie shadows the real one | nit | accepted as stated limit | Design Violations, first bullet. `read_cookie` takes the first match: `oauth_glue.rs:262-267`. The "cannot forge one" part holds for HMAC forgery. The planted-cookie case is not covered (see C2). |
| 10 | "43 passing" count unverified | nit | closed | Design no longer carries the 43 figure. `google_sign_in.rs` has 22 `#[tokio::test]` functions, matching the design's "22 passed". The api lib count (38) was not re-run here (see C9). |

## New findings

None at `blocking` severity.

What I checked for a new blocking finding, and found closed:
- Every route that reaches better-auth's social code is gated. The OAuth plugin registers only
  `POST /sign-in/social`, `GET /callback/{provider}` and `POST /link-social`
  (`better-auth-api` `plugins/oauth/mod.rs:55-57`). The axum adapter registers no fallback
  (`better-auth-0.10.0` `handlers/axum.rs:85-115`). The gate is layered after the nest and after the
  `/auth/oauth/{provider}/start` merge (`lib.rs:94-106`), so it sees every method on those paths.
- `library(` is called only from `start` (`oauth_routes.rs:142`) and `complete` (`:261`). Both are
  reachable only through the gate.
- Path shape: `admit_path` (`oauth_glue.rs:175-189`) runs the shape check on the raw and the fully
  decoded path, then a host-unchanged check. The sink at `oauth_routes.rs:201` uses only the re-admitted
  `target`.
- Credential email rewrite cannot be bypassed by a form body: the library parses JSON only
  (`better-auth-core-0.10.0` `error.rs:271-285`).
- Token encryption key is HKDF over `jwt_secret` (`better-auth-api-0.10.0`
  `plugins/oauth/encryption.rs:23-27`), as the design says.
- `OAUTH_PENDING_SECRET` is required with Google and at least 32 bytes (`config.rs:234-236`, `:266-271`).

## Carried to Plan

- **C1 (should).** Add a HEAD row for `/auth/link-social` to the method table (`google_sign_in.rs:479`). The
  gate already answers it (`oauth_routes.rs:340`).
- **C2 (should).** The Violations bullet on duplicate cookies says the case "denies the flow; it cannot
  forge one." That is true for forgery but incomplete. A valid sealed value from an attacker's own
  `start`, planted under the same name (for example from a sibling subdomain), is accepted when it sits
  first. The victim's own consent then completes under the attacker's state and signs the victim into
  the victim's own account. Reword the bullet and add it as a stated limit.
- **C3 (should).** The email uniqueness check is check-then-create. Password sign-up checks at
  `better-auth-api` `plugins/email_password.rs:294`, Google callback at `plugins/oauth/handlers.rs:595`
  and `:641`, and the adapter's `create_user` at `better-auth-allsource` `adapter.rs:321-323`. Two
  concurrent sign-ups or callbacks can create two users for one address. The design's "no second user"
  claim holds only sequentially. Add it to Violations, or add a concurrency row.
- **C4 (should).** Open item 3 says the single-use window is open across processes. State the backstop:
  Google codes are single-use, so a second exchange of the same code fails at the token endpoint. This
  sizes the window correctly for the decision on scaling out.
- **C5 (should).** Provider-canonical aliases (Gmail dots) are a different string, so they are not
  matched. Such an alias creates a second user for the same mailbox. The squat rule and Violations
  should say that aliases fall outside matching.
- **C6 (nit).** `normalise_email` uses `to_ascii_lowercase` (`oauth_glue.rs:143`). Non-ASCII local parts are
  not folded, so "every account email is lower-cased" (design line 78) is slightly overstated. State
  the ASCII limit.
- **C7 (should).** Design lines 84-86 say pre-change mixed-case rows are "not matched". The same
  normalisation also breaks password sign-in for those rows, because the sign-in email is lowercased
  before lookup. State the lockout as well as the non-match.
- **C8 (nit).** Round-1 finding 5 asked for E5 growth under Violations. It is in Decisions only. Add it to
  Violations: unauthenticated `start` (and HEAD on `start`) writes a verification row per request, rate
  limited per client address.
- **C9 (nit).** The api lib count (38) and the `cargo xtask ci` result are unverified by this review. Run
  `cargo test -p rv2-shared -p api` and record the output before the Build gate.

verdict: clean
