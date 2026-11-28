# Google sign-in on apps/api: design

Status: draft, rescoped by the founder on 2026-11-27 (Amendment 1, revised). Implements
decision D18 and §5.4 of [the foundation doc](../architecture/001-rust-v2-allsource-foundation.md).
Nothing merges before approval. Review history is at the end.

## Scope

- Google OAuth 2.0 authorisation code flow with PKCE, end to end: start route, callback,
  session creation on the existing better-auth session, sign-out (`POST /auth/sign-out`).
- The return target travels in a sealed pending-origin cookie, bound to the OAuth `state`.
- Linking a Google identity to an existing account by email, **only when both sides proved
  the email** (Amendment 1). Otherwise the sign-in is refused.
- The seal, the cookie and the return-target check are provider-agnostic. A GitHub provider
  needs a provider entry and an `OAuthProvider::github` call in `better.rs`; it adds no
  redirect logic. GitHub is not built now.
- Out of scope: GitHub, refresh tokens (the library's refresh and access-token routes are
  unmounted), id_token verification, email verification itself (see Plan).

## Flow

```text
browser                  apps/api                              Google
  | GET /auth/oauth/google/start?return_to=/p&app=O
  |----------------------->| gate: rate limit
  |                        | admit(O, /p)                       (oauth_glue::admit)
  |                        | library POST /sign-in/social -> state S, PKCE in library record
  |  302 Location: accounts.google.com/...?state=S&code_challenge=…
  |  Set-Cookie: __Host-rv2_oauth_pending=seal({S, O, /p, exp +540s}); Path=/; Secure
  |<-----------------------|
  |------------------------------------------------------------->| user consents
  |  GET /auth/callback/google?code=C&state=S   (cookie rides, SameSite=Lax)
  |----------------------->| gate: rate limit, GET only
  |                        | pending_cookie: exactly one value, else refuse
  |                        | open(seal) -> provider == route -> state == S
  |                        | claim_local, then claim_shared (one use across instances)
  |                        | admit(O, /p) again
  |                        | library GET /callback/google: consumes state record,
  |                        |   POST token_url (code, code_verifier), GET userinfo,
  |                        |   verified Google email? link or create user, create session
  |                        | settle_email_match: existing account verified?
  |                        |   yes -> audit (password accounts), keep session
  |                        |   no  -> undo the link and session, refuse
  |  302 Location: O/p   Set-Cookie: session=…; Set-Cookie: __Host-rv2_oauth_pending=; Max-Age=0
  |  (refused: 302 Location: <home origin>/login?error=unverified_link, no session cookie)
  |<-----------------------|
```

Any other refusal on the callback path ends at `302 Location: <home>` with the pending
cookie cleared and no session cookie. `<home>` is the first entry of `OAUTH_RETURN_ORIGINS`
(default `CORS_ORIGINS`) with path `/`. The gate owns these responses.

## Decisions

**Methods.** On `/auth/callback/{provider}` only GET runs the signed flow. Every other
method, HEAD included, is 404: axum routes HEAD to the GET handler, so better-auth's own
callback would accept a HEAD without the cookie, and a HEAD that succeeded would spend the
provider's one-time code. This narrows the round-1 instruction ("GET and HEAD through the
signed flow"); recorded in the review history. On `/auth/sign-in/social` and
`/auth/link-social`, every method is 404.

**Unused library routes are unmounted** (`better.rs`, `UNMOUNTED_USER_ROUTES`,
`UNMOUNTED_OAUTH_ROUTES`, `UNMOUNTED_SESSION_ROUTES`). The client reads the session from
`/me` and signs out with `/auth/sign-out`. Unmounted by path, so every method answers 404:
`update-user`, `change-email`, `delete-user`, `delete-user-callback`, `get-access-token`,
`refresh-token`, `get-session`, `list-sessions`, `revoke-session`, `revoke-sessions`,
`revoke-other-sessions`, `sign-in/username`. Fewer doors: none of them has a caller.

**Account linking by email (Amendment 1, revised).** better-auth links a provider to an
existing account with the same email by default and does not check that account's
verification. This code keeps the library's linking on for `google` only
(`better.rs`, `trusted_providers = ["google"]`) and settles the result in
`settle_email_match` after the library answers:

- **Both sides proved the email**: Google reports `email_verified` (the library's gate
  `verified_google_user`) **and** the existing account's `email_verified` is true. The link
  stands and the library's session is kept. Every accepted Google sign-in is audited, as
  `auth.google.signed_in`, for every account (password or not). The store cannot tell a first
  link from a repeat sign-in without a read the library does not allow before it links, so
  the audit covers both. If the audit fails, the callback is refused and the rows this
  callback wrote are removed. A link from an earlier sign-in is not removed.
- **The existing account's email is not verified**: the sign-in is refused. Every Google row
  of that account is removed, because no legitimate link can sit on an unverified account, and
  the session this callback issued is deleted (`refuse_link`). The browser is sent to
  `/login?error=unverified_link` on the primary origin. The account's user row, password and
  other sessions are unchanged. No verification mail is sent yet (Plan bead 14).
- **Any other failure after the library has answered** with a user id (a settle fault, a
  missing cookie, a non-200 answer) runs the same undo (`refuse_residue`), decided by the
  account's verification as above.

**Why there is no pre-check.** The link decision happens inside the library's `callback_core`,
after the token exchange and the userinfo fetch. `map_user_info` is a plain `fn` pointer with
no database handle, so this code cannot look up the account before the library links. A
pre-check needs a library change or a hand-rolled callback. The window between the library's
link write and the undo is therefore a stated decision: the linked identity is reached only by
another callback for the same identity, which takes the same settle and is refused. Nothing
else reads Google accounts, because the token routes are unmounted and `link-social` is 404.
The window-free alternative is to disable library linking (the library refuses with no write)
and restore verified linking through a session-bound "connect Google" flow (Plan bead 16).

`update_user_info_on_link` is off, so a refused link leaves the profile as it was.

**Who is affected.** No email-verification flow is mounted, and password sign-up sets no
verified flag. So every password account is unverified today, and **password accounts never
auto-link to Google until email verification ships.** Their password sign-in is unaffected.
A Google sign-in for their email is refused with the message above. Google-only accounts,
created by Google sign-in with a verified email, are unaffected.

**Takeover.** A password account registered with a victim's address is unverified, so
Google never links to it. A squatter can therefore no longer be overtaken, and the owner
cannot sign in with Google to that account either: the sign-in is refused. That is a denial
for the owner, not a takeover. It stops when email verification ships (Plan bead 5).

**Email form.** Account emails are stored and looked up lower-cased, using ASCII lower-casing
only (`normalise_email`, `oauth_glue.rs`). Non-ASCII letters are not folded. Password
sign-up and sign-in are rewritten by the gate (`normalise_credentials`, `oauth_routes.rs`),
and the Google identity by `verified_google_user`. Accounts created before this change with
an upper-case letter are not matched by Google and cannot sign in with a password, because
the sign-in email is lower-cased before lookup. No release exists (`CHANGELOG.md`), so none
are expected; a migration bead is in the Plan.

**Unverified Google emails are refused.** `verified_google_user` rejects `email_verified !=
true` before any row is written.

**Tokens at rest are encrypted** (`encrypt_oauth_tokens = true`), keyed from `JWT_SECRET` by
the library's HKDF. No new secret.

**State, PKCE and single use.** The library stores the PKCE verifier and state in a
verification record keyed `oauth:{state}`, with a 10-minute expiry, and deletes it on a
successful lookup. The lookup and delete are two calls, so two concurrent callbacks can both
pass the lookup. Two layers close that: `claim_local` (in-process fast path) and `claim_shared`
(an append `auth.oauth.claimed` to stream `oauth-claim:{state}` at expected version 0, so
AllSource accepts exactly one). A `VersionConflict` is a refusal; any other store failure is
refused as `ClaimUnavailable`. The shared claim holds across instances only with the
`allsource-auth` feature (see Verification). The provider's one-time code is a backstop.
The cookie lives 540 s, shorter than the library's 600 s state.

**Verification rows are not swept.** `delete_expired_verifications` has no caller. Lookups
ignore rows past `expires_at`, so an expired row cannot be used, but the rows persist
(Plan bead 1).

**Redirect URI is derived, not configured:** `AUTH_BASE_URL` + `/callback/google`, the
library's default. Accepted by the founder.

**No nonce.** Identity comes from the userinfo endpoint over TLS with the access token. The
library stores the token response's `id_token`, encrypted, without validating it. Nothing
reads it.

**Rate limit.** Start and callback are limited per client address by the existing limiter
(`rate_limit::within_limit`, called from the gate). 60/s sustained, 120 burst.

**Provider mismatch.** A cookie sealed for another provider is refused on a route for this one.

**Config.** `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` (both or neither),
`OAUTH_PENDING_SECRET` (required with Google, at least 32 bytes), `OAUTH_RETURN_ORIGINS`
(default `CORS_ORIGINS`). Endpoints are fixed in code, overridable only in tests.

**Cookie name.** `__Host-rv2_oauth_pending` (`oauth_glue.rs`). The `__Host-` prefix makes a
browser accept it only with `Secure`, `Path=/` and no `Domain`, so a sibling subdomain cannot
set or shadow it. Two values under the name are refused (`DuplicateCookie`). Recorded in the
foundation doc §5.4.

**Alignment with the foundation doc.** D18 and §5.4 say "signed pending-origin cookie" and name
it `oauth_origin`. The implemented name is `__Host-rv2_oauth_pending`; §5.4 is updated.

## Semantic ISA

### 1. Side-effect inventory

| # | Side effect | Where | Reversibility |
|---|---|---|---|
| E1 | 302 to the provider | `oauth_routes.rs` `start` | none persisted |
| E2 | Sealed pending cookie written to the browser | `oauth_glue.rs` `set_cookie`, applied in `start` | expires in 540 s; cleared on callback |
| E3 | Outbound POST to Google token endpoint (code, verifier, client secret) | library `callback_core`, reached via `library(...)` in `complete` | none; one-time code |
| E4 | Outbound GET to Google userinfo with the access token | library `callback_core` | none |
| E5 | Verification row (state, PKCE verifier, callback URL) written | library `initiate_oauth_flow_core`, via `start` | consumed on successful callback; unused rows persist (Plan 1) |
| E6 | User row created for a new verified Google email | library `callback_core`, after `verified_google_user` | none in this change: no delete route is mounted or called |
| E7 | Account row created (Google, tokens encrypted) | library `callback_core` | removed with the user (E6), or by undo (E10) |
| E8 | Session row created and session cookie written to the browser | library, forwarded in `complete` | sign-out |
| E9 | 302 to the admitted URL, pending cookie cleared | `callback` | none |
| E10 | **Refusal undo**: Google account rows and the session of a refused callback are deleted | `refuse_link`, reached from `settle_email_match` and `refuse_residue` | unverified account: every Google row of that user (no legitimate link can sit there). Verified account (audit or settle fault): only Google rows created at or after this callback's start. Never an earlier link. |
| E11 | **Sign-in audit record** `auth.google.signed_in`, for every accepted Google sign-in (every account) | `record_audit` | append-only; retried by the SDK, so a duplicate is possible; can outlive an undone link (Violations) |

No file write, deploy, message or payment. Data writes are E5–E8, E10 and E11 into the
existing `auth-*` streams and the `audit-auth:{user}` stream.

### 1a. Session routes and enforcement

| Route | Creates or reads a session | Enforcement | Tested by |
|---|---|---|---|
| `POST /auth/sign-in/email` | creates (password) | none beyond the normal library check; no link is involved | the password tests |
| `POST /auth/sign-in/username` | creates (password) | **unmounted** | `the_username_sign_in_route_is_not_mounted` (404) |
| `POST /auth/sign-up/email` | creates, for a new account | none needed; the account is new | `register_password` in the password tests |
| `GET /auth/callback/{p}` | creates (Google), via library `callback_core` | gate (GET only); `settle_email_match` links only verified accounts and undoes a refused link | the callback and link tests |
| `POST /auth/sign-out` | reads (`require_session`) | library's own check | `sign_out_ends_a_google_session` |
| `GET`/`POST /auth/get-session`, `GET /auth/list-sessions`, `POST /auth/revoke-*` | read or mutate sessions | **unmounted** | `the_session_management_routes_are_not_mounted` (404) |
| `POST /auth/refresh-token`, `POST /auth/get-access-token` | reads, returns stored provider tokens | **unmounted** | the disabled-route unit test |
| `POST /auth/sign-in/social`, `POST /auth/link-social` | creates (social), reads (link) | gate: 404 for every method | `other_methods_on_the_oauth_paths_are_not_found` |
| `GET /me` and domain routes that take a session (`posts`, `users`, `a2ui`) | reads | `ExtractAuthUser`: the session row must exist and be unexpired | `/me` tests |
| `GET /health`, `/ready`, `/openapi.json` | none | n/a | n/a |

A session is valid until it expires or is signed out. There is no revocation cutoff in
this design. Sessions of a password account are not ended by a Google sign-in, because a
Google sign-in never links to an unverified account (see Decisions).

### 2. Instructions

No model call and no prompt is added, so there are no instruction cores.

### 3. Taint

Sources:

- Query `return_to` and `app` on `start` (browser-chosen).
- Path `provider` on `start` and `callback` (bounded by `offers()`).
- Query `state` and `code` on `callback` (provider-supplied; a crafted link can carry any value).
- `Cookie` header `__Host-rv2_oauth_pending` on `callback` (browser-supplied).
- Credential body `email` on `sign-up/email` and `sign-in/email` (rewritten, not trusted).
- Google userinfo JSON: `sub`, `email`, `email_verified`, `name`, `picture` (remote; `name`
  and `picture` are written only on account creation, never on a link).
- Google token response: access token, refresh token, `id_token` (remote, stored encrypted;
  `id_token` unvalidated).

### 4. VERIFY before every sink

Line numbers are for the tree at this round's head and are approximate after the rescope
rebuild; the Review re-derives them with `rg`.

| Sink | Call site | VERIFY | Deterministic? |
|---|---|---|---|
| S1 Location to provider | `start` | URL built by the library from configured `client_id` and fixed `auth_url`; its `state` is parsed and sealed. No request value reaches it. | yes |
| S2 Location to app URL, success | `callback` | `target` is `glue::admit(allowlist, origin, path)`. The path passes `admit_path`: no `%` in the path, printable ASCII, no `\`, `#` refused; the query is decoded once and must be UTF-8 with no control characters and no escape left in it (`check_query`); the raw value is returned; host must not move. The origin must equal an allowlist entry. | yes |
| S3 Location to home or login, refusal | `callback` | `ReturnTarget::home(allowlist)` plus `/login?error=unverified_link` for an unverified link. No taint. | yes |
| S4 Session cookie write | `complete` (library `set-cookie`) | (a) `pending_cookie`: exactly one `__Host-` value, non-empty. (b) `glue::open`: HMAC-SHA256 with `verify_slice` before the payload is parsed; expiry. (c) provider equals the route. (d) `state` equals the sealed state. (e) `claim_local` then `claim_shared` admit the first use across instances. (f) the library consumes the state record. | yes, shared store |
| S5 Account link and user write from Google identity | library `callback_core` (linking on for `google`), reached from `complete` | **Link sink.** VERIFY: (1) Google `email_verified == true` (`verified_google_user`); (2) the existing account's `email_verified == true` (checked in S6, the settle); (3) the email equality is the library's lookup on the normalised email, both sides normalised (`normalise_email`, `normalise_credentials`); (4) provider id is `google` (`trusted_providers`). The library writes the link before S6 runs; S6 decides whether it stands. | yes |
| S6 Link settle | `settle_email_match` | Reads the linked user. If `email_verified` is false: refused, `refuse_link` with all Google rows (S7). If true: the audit (S8) is written; a failed audit refuses and `refuse_link` removes only this callback's rows. Otherwise the link stands. Every refusal path runs `refuse_link` once the user id is known. | yes |
| S7 Refusal undo | `refuse_link`, `refuse_residue` | Unverified account: every Google row of the user and the callback's session token. Verified account: only Google rows created at or after the callback's start, and the session token. Never a row from an earlier sign-in. | yes |
| S8 Audit write | `record_audit` | Shared coordination writes `auth.google.signed_in` to `audit-auth:{user}` for every accepted Google sign-in. A failed write refuses (S6). Local coordination logs it (tests only). | yes, retried by the SDK (Violations) |
| S9 Credential write (sign-up, sign-in) | `gate`, `normalise_credentials` | Only the `email` string changes, to its ASCII-lower-cased form. The password field is untouched. | yes |

Sink-to-VERIFY check for Review: `rg -n 'redirect\(' apps/api/src/infrastructure/auth/oauth_routes.rs`,
`rg -n 'library\(' apps/api/src/infrastructure/auth/oauth_routes.rs`, and the integration rows.
Every library call must be reached only through `gate`.

### 5. Violations

Recoverable, not panics. Each refusal is logged at WARN with its enum name. The browser gets
a 302 to home, or to the sign-in page for an unverified link, with no session.

- **Unverified accounts are refused, not linked** (Decisions). The owner of an unverified
  password account cannot sign in with Google until email verification ships. Denial, not
  takeover.
- **The undo is not atomic, and the window is a decision.** The library writes the link before
  settle refuses it (Decisions: no pre-check is possible). For that window the Google row exists
  for an unverified account. A concurrent callback for the same identity goes through the same
  settle and is refused, so no session issues from it. Two distinct states racing one
  unverified identity are tested (`two_callbacks_racing_one_unverified_identity_both_refuse`).
- **A refusal after the library answered is undone, when the body names the user.** The
  settle-fault, missing-cookie and non-200 paths all run `refuse_residue`. A library error that
  returns no body (a transport failure inside the library) cannot be undone, because the body
  names no user. Accepted residue: a Google row on an account that is then refused, which the
  next callback for that identity refuses and removes. For a new user the library created from
  a verified email, a failed settle leaves the user row (no delete path exists), and the next
  sign-in finds that row and settles it. Accepted.
- **An audit row can outlive an undone link.** When the SDK retries an audit after a commit,
  or when a refusal undoes the link after its audit was written, the audit row stays. It records
  an attempt, not a standing link.
- **Audit retry duplicate.** The SDK retries a write after a transport error or 5xx
  (`allsource-0.24.0` `client.rs:141`, `:193-210`). If the first attempt committed, the retry
  appends a second `auth.google.signed_in` row. The duplicate is harmless and visible as two rows.
  Accepted; not deduplicated.
- **The audit covers every accepted Google sign-in, not only links.** The store does not separate
  a first link from a repeat without a read the library does not allow before it links.
- **Claim retry.** The shared claim is not retried by this code. The SDK retries transport
  errors and 5xx before returning, so a retry whose first attempt committed reads as a conflict
  and the callback is refused as a second use. The code is never exchanged twice.
- **E5 growth.** An unauthenticated `start` writes one verification row per request. Rows
  persist (Plan 1). The rate limit is per client address, so total growth is not bounded.
- **Check-then-create race.** The uniqueness check for an email is check-then-create in the
  library (password sign-up; the Google callback). Two concurrent sign-ups, or a sign-up and a
  Google callback, can each pass the check and create two users for one address (Plan 3).
- **Login CSRF.** Two values under the `__Host-` name are refused (`DuplicateCookie`).
- **Voided cookie:** none. There is no revocation in this design.
- **Library `Set-Cookie` count.** The library returns headers as a `HashMap`; one value per
  name is all it carries. The callback emits one `Set-Cookie`; `complete` refuses with
  `NoSessionCookie` if there is none.
- **Refused HEAD leaves the state unspent** (integration row).
- Rate limit exceeded: 429. A credential body over 16 KiB: 413, before the library sees it.

### 6. ISA test matrix

**Unit**, `oauth_glue.rs`:

| Input | Expected |
|---|---|
| `https://evil.example/x`, `//evil.example`, `/\evil.example`, `/\\evil.example` | refused |
| `/%2F%2Fevil.example`, `/%5Cevil.example`, `/%09/…`, `/%0d%0aSet-Cookie:x` | refused (percent-escape in the path) |
| `/x y`, `/\tevil.example`, raw non-ASCII, `javascript:…`, `posts`, `` | refused |
| path longer than 2048, `/posts\x`, `/a/b\c`, NUL in the path | refused |
| `/`, `/posts?tab=1` on the primary origin | admitted, exact URL |
| `/posts?q=hello%20world`, `/posts?q=caf%C3%A9`, `/posts?q=a+b&tab=1`, `/posts?q=100%25` | admitted as written |
| `/posts?q=%252F` (double-encoded), `/posts?q=%2541`, `/posts?q=%0d%0a`, `/posts?q=%00`, `/posts?q=a b`, `/posts?q=%FF`, `/posts#frag` | refused (`UnsafeQuery` or path refusal) |
| `/posts?q=%+1` (`+` is not a hex digit) | admitted as literal text |
| origin not on allowlist; origin removed from the allowlist after sealing | refused, UnknownOrigin |
| tampered payload; payload sealed with another secret; `now >= expires_at` | refused |
| one `__Host-rv2_oauth_pending` value | read |
| two `__Host-` values, either order | refused, DuplicateCookie |
| empty `__Host-` value; only the unprefixed `rv2_oauth_pending` | refused, Missing |
| cookie attributes | `__Host-` name, `Path=/`, HttpOnly, Secure, SameSite=Lax, `Max-Age=540`, no `Domain=` |
| `normalise_email("  Victim@Example.COM ")` | `victim@example.com` |
| claim outcome: `Ok` → first writer; `VersionConflict` → second use; other store error → `ClaimUnavailable` | the mapping |
| unmounted routes (`UNMOUNTED_*`) answer 404 for POST | unit |

**Integration**, `apps/api/tests/google_sign_in.rs`, against an in-process stub OIDC provider.
Memory adapter; no Core needed.

| Case | Expected |
|---|---|
| full flow, `return_to=/posts?tab=1`, primary origin | 302 to `…/posts?tab=1`, session cookie, `GET /me` 200, user created |
| root target; target on the second origin | 302 with session to exact URL |
| encoded query value `/posts?q=hello%20world` | 302 to exact URL with session |
| tainted targets (`https://…`, `//…`, `/\…`, `/%2F%2F…`, `/%5C…`, double-encoded `/posts?q=%252F`, encoded `//` in path) | sign-in completes, 302 to home, session set |
| GET callback without the cookie | 302 to home, no session, no user |
| HEAD callback with and without the cookie | 404 each; a later GET with the cookie still succeeds |
| planted second `__Host-` cookie (attacker's first, victim's second) | 302 to home, no session, no user |
| `Set-Cookie` for the pending cookie | `__Host-` name, `Path=/`, Secure, HttpOnly, SameSite=Lax, no `Domain` |
| POST, PUT, DELETE, HEAD on the callback; POST, GET, HEAD on sign-in/social; POST, HEAD on link-social; POST on refresh-token, get-access-token | 404 each |
| replayed cookie after success | 302 to home, no second session |
| two concurrent callbacks, one cookie | exactly one session |
| `state` mismatch; tampered signature; expired cookie; cookie for provider `github`; no `code` | 302 to home, no session |
| `email_verified: false` from Google | no user created, 302 to home |
| **verified owner**: password account with `email_verified` set, verified Google email | linked; session issued; both credentials work (password sign-in succeeds) |
| **unverified existing account**: password account unverified, verified Google email | refused: 302 to `/login?error=unverified_link` on the primary origin, no session; the user row and the account ids are identical before and after; a repeat sign-in with the same identity is refused the same way; the password and its own session still work |
| **mixed case**: password stored `mixed@…`, verified Google `Mixed@…`, owner verified | links the same user; session; password works |
| **audit outage on a new link** (`audit_down`), verified owner | refused: no session; the Google row the callback wrote is gone; the user row is unchanged |
| **audit outage on a repeat sign-in** (`audit_down`), verified owner with an earlier link | refused: no session; the earlier Google row survives |
| **settle fault** (`settle_down`), verified owner | refused: no session; the Google row the callback wrote is gone |
| **two states racing one unverified identity** | both refused; no session; no Google row; the user row unchanged |
| **claim outage** (`claims_down`) | refused before the library: no user created |
| unit: callback body parsing (`callback_identity`) | user id and token read; malformed body yields none |
| unverified Google email against a password account | no session; password and session unchanged |
| gate path variants (trailing slash, encoded) | no session; trailing-slash sign-in and link-social 404 |
| trailing-slash sign-up with mixed case | normalised, or refused by routing; no mixed-case account |
| sign-out after Google sign-in | session cleared, `GET /me` 401 |
| `/auth/oauth/github/start`, `/auth/callback/github` | 404 |
| session-management routes (`list-sessions`, `get-session`, `revoke-other-sessions`) | 404 |
| `/auth/sign-in/username` | 404, no session |
| 180 starts from one client in waves of 30 | at least one 429 |
| two instances sharing one Core (`#[ignore]`) | one session; needs a live Core and `allsource-auth` |

## Verification

Round 3 run, before the Amendment 1 rescope, exact results: `cargo test -p rv2-shared -p api`:
api lib 43 passed, `google_sign_in` 23 passed, rv2-shared 8 passed; clippy `-D warnings` clean.
**Those counts are stale.** The rescope removed the revocation code and tests; the new code is
**not compiled or tested**. The disk was at 3 GiB free and the build is held on the founder's
decision.

Build checklist, to run first on the next build:
- `AuthConfig: Clone` where settle and the password path use it (none remain after the rescope).
- `create_session` arity: none remain. `create_session_cookie`: none remain.
- `AccountOps::get_user_accounts`, `delete_account`; `AuthAccount::provider_id`, `id`: imported
  in `oauth_routes.rs`.
- `SessionOps::delete_session(&str)`: used by `refuse_link`.
- `AuthAccount::created_at` (`DateTime<Utc>`): used by `refuse_link`'s row filter.
- `AuthUser: Serialize`: the tests compare whole user rows with `serde_json::to_value`.
- `state.allsource.core` visibility; `IngestEventInput::with_expected_version`.
- `SdkError::VersionConflict` and `SdkError::Api` field shapes.
- `Result::inspect_err` needs Rust 1.76+ (toolchain 1.97).
- `ReturnTarget` fields `origin`, `path` are `pub`.
- `json!` keys in parentheses where a constant is used (none remain).
- `AppState.faults` (`Faults`: `audit_down`, `claims_down`, `settle_down`): read on each use,
  flipped by tests on a running app. `Coordination` is now `Shared | Local` only.

Other checks:
- `cargo xtask ci` and `cargo xtask live`: not run.
- Production feature set: `allsource-auth` is off by default (`apps/api/Cargo.toml`). Without
  it, library state and sessions are per process, and the shared single-use claim does not
  hold across instances. Confirm the production build turns it on before the Plan calls
  cross-instance single use closed.
- The sign-up trailing-slash test accepts either a normalised success or a 404; which one the
  library gives is unverified.
- The live-Core test has not run.

## Plan (beads)

1. **Verification sweeper.** Idempotent purge of rows past `expires_at`, in `rv2-jobs`.
2. **Cross-instance single use.** Written (`claim_shared`). Closes when the ignored live-Core
   test passes with `--features allsource-auth`, and the production feature set is confirmed.
3. **Identity uniqueness guard.** Make sign-up and the Google callback atomic on the email.
   Needs a unique constraint or a store lock.
4. **Legacy mixed-case migration.** Lower-case existing account emails, or match both forms
   once, before any release that contains such rows.
5. **Email verification ships.** Until it does, password accounts never auto-link (Decisions).
   Verification unlocks linking for the owners who verify.
6. **Core family-scan limit.** Confirm Core's ordering and the `limit=10000` ceiling
   (`better-auth-allsource` `client.rs:247`). Paginate or record a measured launch limit.
7. **Encoded query values.** Written: admitted with a decode-once check. Test run open.
8. **Alias and Unicode folding.** Decide whether Gmail aliases and non-ASCII folding are in
   scope for matching. Currently neither.
9. **Google Cloud setup guide.** `docs/guides/google-sign-in-setup.md`: authorised JavaScript
   origins, redirect URIs per environment, scopes `openid email profile`, consent screen.
10. **`.env.example`.** `OAUTH_PENDING_SECRET`, `OAUTH_RETURN_ORIGINS`, commented
    `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET`. No real values.
11. **Login view.** "Continue with Google" on `apps/app`, built from `rv2-ui` / `rv2-kit`
    components. It must also show the `unverified_link` message at `/login`. Screenshots light
    and dark.
12. **Gates.** `cargo xtask ci` and `cargo xtask live` (with a Core) run and recorded.
13. **Founder action.** Create the Google OAuth client and enter its values.
14. **Verification mail on refusal.** The refusal should send a verification mail to the
    address, rate-limited. better-auth's `email_verification` plugin exists
    (`better-auth-api` `plugins/email_verification`) but is not mounted, and `rv2-email` has
    no better-auth `EmailProvider`. Not built: mount the plugin, write the provider on
    `rv2-email`, and rate-limit the send.
15. **Audit duplicate rows.** Deduplicate the sign-in audit if the store offers an idempotency
    key. Accepted for now (Violations).
16. **Connect flow, window-free linking.** Disable library linking (the library then refuses with
    no write, `handlers.rs:629-632`) and restore verified linking through a signed-in "connect
    Google" flow: the session proves the account, the Google identity proves the email, and the
    link is written by this code after both checks. Removes the post-hoc window (Decisions).
17. **Clock step.** `started` is read from the wall clock. A backward step can make a new row's
    `created_at` earlier than `started`, so `refuse_link` leaves that row on a verified account.
    Compare against a monotonic reference or a store-assigned time.
18. **Concurrent undo.** On a verified account `refuse_link` also removes Google rows that a
    concurrent callback for the same identity created after this callback's start. That
    callback's session survives without its link. "Never an earlier link" holds; this case is
    not covered by E10 and S7.
19. **Round-trip precision.** Verify that `created_at` keeps nanoseconds through Core's read path
    (`better-auth-allsource` adapter), or record a measured limit. The live test has not run.
20. **Residue with unknown verification.** When the user read fails, `refuse_residue` and settle
    pass `all = false`, so residue on an unverified account survives until the next callback for
    that identity refuses it.
21. **200 without a user id.** `settle_email_match` returns without undo when the library's 200
    body has no `user.id`. Same residue class as a library `Err`.

## Open for review

1. Founder: confirm the rescope and the refusal copy at `/login?error=unverified_link`.
2. Founder: is the denial for unverified password accounts acceptable until bead 5 ships, or
   should password accounts be verified by a manual step first?
3. Single instance at launch, or the shared claim (bead 2)? Written; needs the live test.
4. Core scan limit (bead 6).
5. Zero-knowledge shares: considered, not used. Correct the interpretation if it differed.

## Review history

Round 1: spike committed as `2e2fa12`; reviewed in
[review A](2026-11-26-google-sign-in-design-review-a.md) and
[review B](2026-11-26-google-sign-in-design-review-b.md). Both found the same blocker: the
gate matched GET only, so HEAD reached better-auth's callback without the cookie.

Round 2: design `7022e12a…f81f`, commit `8423a36`. Reviewed in
[review-r2-a](2026-11-26-google-sign-in-design-review-r2-a.md) (findings) and
[review-r2-b](2026-11-26-google-sign-in-design-review-r2-b.md) (clean). Blocker: a planted
pending cookie completing a login CSRF. Fixed by the `__Host-` rename and duplicate refusal.

Round 3: design `de3fa6a6…00e8ac`, commit `97727a6`. Reviews `r3-a` and `r3-b` clean. Founder
approval recorded: "Approve", 2026-11-26, digest `de3fa6a6…00e8ac`.

Amendment 1 (founder, 2026-11-26), first version: automatic match by verified email with
revocation of an unverified password account. Reviewed in
[review-amendment-1](2026-11-26-google-sign-in-design-review-amendment-1.md) (3 blocking, 8
should, 5 nit) and [review-amendment-1-r2](2026-11-26-google-sign-in-design-review-amendment-1-r2.md)
(2 blocking, same defect through new routes). The response added a revocation cutoff, a
read-path check, a sign-in guard, and unmounted routes. The third review,
`amendment-1-r3`, found three new blocking findings inside revoke-on-link (N1 Bearer
bypass, N2 sign-up gap, N3 cutoff stamped before the clear).

**Approved by the founder 2026-11-27 (answer: "Approve"), digest 12a9401d…9a035**, after the
rescope review round 2 came back clean ([r2](2026-11-27-google-sign-in-rescope-review-r2.md)).
The r2 carried items are Plan beads 17–21.

**Changed after the approval:** Plan beads 17–21 are in this file. The approved digest was not
reproduced by a plain SHA-256 of the file text, and the method behind `12a9401d…` is not
recorded, so the approved text cannot be checked against this file.
Approved digest 12a9401d… covers the design text; later edits are review-history and Plan-bead additions only.

**Rescope review (2026-11-27)**, [rescope review](2026-11-27-google-sign-in-rescope-review.md),
verdict `findings` (1 blocking, 6 should, 2 nit). Responses:

| Finding | Response |
|---|---|
| 1 blocking: undo deleted a pre-existing Google link when the audit failed on a verified account | `refuse_link` removes only Google rows created at or after the callback's start, for a verified account. Every Google row goes only for an unverified account, where none can be legitimate. Test `an_audit_outage_on_a_repeat_sign_in_keeps_the_existing_link`. |
| 2 should: error paths skipped the undo | Every path after the library has answered with a user id runs the undo (`refuse_residue`, `settle_email_match`). Test `a_settle_failure_removes_the_link_this_callback_wrote`. The no-body library error is stated as residue. |
| 3 should: no pre-check possible | Decisions states why (the fn-pointer mapper, the decision inside `callback_core`) and the window. Window-free alternative: Plan bead 16. |
| 4 should: origin mismatch | The refusal goes to the primary (home) origin, as the code does. Flow line corrected. |
| 5 should: audit scope | Every accepted Google sign-in is audited (`auth.google.signed_in`), for every account. Stated, with the reason. |
| 6 should: tests did not prove the undo | Assertions compare the user row and the account ids before and after, repeat the sign-in, and assert a refusal each time. The callback's session row is not visible to any test (the token never reaches the browser); the refusal path deletes the token, and that deletion is asserted only through the refusal. |
| 7 should: concurrency and error paths | `two_callbacks_racing_one_unverified_identity_both_refuse`; settle fault test; audit outage on a new link; audit outage on a repeat. |
| 8 nit: zero-knowledge paragraph | Removed. Open question 5 stays. |
| 9 nit: audit outlives undo | Violations. |

**Rescope (founder, 2026-11-27).** After the cap of the review cycle, the founder chose to
rescope Amendment 1 rather than continue the revocation design. Decision: auto-match only when
both sides proved the email; refuse otherwise with the message above. **Removed:** the
revocation design end to end: `RevocationPlugin`, `revoke_password`, the `sessions_revoked_at`
cutoff and its key, `session_revoked`, `revoked_at`, `sign_in_survived`,
`guard_password_sign_in`, the `sign_in_hold` test seam, the cutoff check in `ExtractAuthUser`,
and the revocation tests. **Kept:** the `__Host-` pending cookie and duplicate refusal, the
shared AllSource single-use claim, encoded query targets, the GET-only callback gate, email
normalisation, the unmounted unused routes, and the audit on a verified link.
