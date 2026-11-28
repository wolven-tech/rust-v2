design-sha256: 5176d66f195efa7664348217add97dddab98355b60108f89653b1ec14335b1ab

# Review round 2: Google sign-in design, Amendment 1

Fresh-context second review of Amendment 1 on the design above. It checks the
first review's 16 findings against the design and the code, then looks for new
blocking findings. Code read in the worktree at
`/Users/decebaldobrica/Projects/founder-mode/rust-v2-google-signin`. Library source read from
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` (better-auth-api-0.10.0,
better-auth-core-0.10.0). `vendor/better-auth` is a facade over those crates.

Nothing was compiled or run. The live-Core test was read, not executed.

## Status of the round-1 findings

Status is `closed`, `open`, or `accepted` (a stated limit that carries to the Plan).

| # | Round-1 finding | Status | Evidence |
|---|---|---|---|
| 1 | Live squatter session survives revocation (order, in-flight sign-in) | **open** | Order fixed: audit (`oauth_routes.rs:403`), one update clears hash and writes cutoff (`:412`, `:427-438`), session deletes best-effort (`:440`), owner session after (`:415-418`). In-flight `/auth/sign-in/email` closed by `guard_password_sign_in` (`:546-576`) and test `a_password_sign_in_racing_a_revocation_yields_no_session` (`tests/google_sign_in.rs:1094-1137`). Still open: library session routes ignore the cutoff (B1), and `/auth/sign-in/username` is unguarded (B2). |
| 2 | "Every session ends" not guaranteed (10k scan) | **open** | The cutoff is enforced only in `ExtractAuthUser` (`middleware.rs:113-120`). The library's `require_session` has no cutoff (B1). The design's claim "the read path enforces it" is false for library routes. |
| 3 | Base design contradicts Amendment 1 | **closed** | Scope now lists linking in scope (design lines 17-19). S5 says linking is on for Google only (`better.rs:107-108`). Old rows replaced by the auto-match and revoke rows. |
| 4 | Audit after link leaves squatter live on audit outage | **accepted** | Stated as a known limit and Plan 16. Asserted both ways by `an_audit_outage_refuses_the_owner_and_leaves_the_squatter_live` (`tests/google_sign_in.rs:1022-1033`). Carried (C2). |
| 5 | "Never retried" claim wrong | **closed** | Corrected in design S4(e), Amendment 2, and `claim_shared` doc (`oauth_routes.rs:330-335`). Audit retry duplicate stated in Violations. |
| 6 | S10 VERIFY claims an email check it does not do | **closed** | S10 row now names S9 as the only email gate. |
| 7 | Squatter name, picture and domain data carry over | **closed** for name and picture (`better.rs:111` sets `update_user_info_on_link`; library applies it at `better-auth-api` `plugins/oauth/handlers.rs:613-620`). Domain carry-over **accepted** as Plan 15 (C3). |
| 8 | Refusal arms untested, live test not run | **closed** for the refusal arms (`Coordination::FailClaims` and `FailAudit`, tests at `tests/google_sign_in.rs:1008` and `:1022`). **open** for the live-Core test (C7). |
| 9 | Not compiled | **open** (C1). |
| 10 | Stale text (beads 2, 5, 7, open items, S4(e)) | **closed**. |
| 11 | Gate path variants untested | **closed**. Trailing-slash and encoded variants tested (`oauth_routes.rs:494-498`, `tests/google_sign_in.rs:1038-1068`). Test asserts outcome only, not library behaviour (C8). |
| 12 | `from_str_radix` accepts `+` | **closed**. `hex_value` requires two hex digits (`oauth_glue.rs:258`, `:270-277`), unit test at `oauth_glue.rs:606-613`. |
| 13 | Stale unit test name | **closed** (`oauth_glue.rs:527-528`). |
| 14 | Comment does not match test | **closed**. The comment at `tests/google_sign_in.rs:417-418` now matches the assertion. |
| 15 | Refusal docs say "any percent-escape" | **closed** (`oauth_glue.rs:96-98`). |
| 16 | Unreachable "verified keeps both" branch | **accepted**. Plan 17 says no production path sets it. |

## Blocking

1. **B1. A voided session still reads and lists the owner's live session tokens.**
   The library's session routes authenticate with `require_session`, which checks
   only that the session row exists and is not expired (`better-auth-core-0.10.0`
   `src/plugin.rs:243-254`). It never reads the cutoff. `SessionManagementPlugin`
   is mounted (`apps/api/src/infrastructure/auth/better.rs:117`) with
   `/get-session`, `/list-sessions`, `/revoke-session`, `/revoke-sessions`, and
   `/revoke-other-sessions` (`better-auth-api-0.10.0` `src/plugins/session_management.rs:51-60`).
   `handle_list_sessions` returns every session of the user (`:178-186`), and
   `Session` serialises `token` (`better-auth-core` `src/types.rs:43-48`). The gate
   passes these paths through (`apps/api/src/infrastructure/auth/oauth_routes.rs:515`).

   Event order: the squatter's session S1 exists before the revocation time T. The
   revocation's best-effort delete (`delete_user_sessions`, `crates/better-auth-allsource/src/adapter.rs:361-364`)
   misses S1. The design admits this miss (Open item 5, Amendment "Revocation
   invariant"). The squatter then calls `GET /auth/list-sessions` with S1. The
   library returns 200 with the owner's sessions, including the owner's post-T
   token. The squatter presents that token to any `ExtractAuthUser` route, such as
   `/me`, and holds the owner's account. The cutoff does not stop this because only
   `ExtractAuthUser` (`middleware.rs:115`) checks it.

   This is an account takeover, and the design's "every session created before T is
   void" and "the read path enforces it" are false for these routes.

   Fix: enforce the cutoff at the one lookup every session path shares. Apply
   `session_revoked` in `crates/better-auth-allsource/src/adapter.rs:347` `get_session`
   and in the memory adapter, so a voided row returns `None`. `require_session`,
   `ExtractAuthUser`, and the sign-in guard then all reject it. Cheaper alternative
   if the app does not use them: add `/get-session`, `/list-sessions`,
   `/revoke-session`, `/revoke-sessions`, and `/revoke-other-sessions` to
   `UNMOUNTED_USER_ROUTES` in `better.rs:51-56`. Check `apps/app` for any use of
   `/auth/get-session` first. Test: create S1, skip the delete, revoke, then assert
   `GET /auth/list-sessions`, `GET /auth/get-session`, and `GET /me` with S1 all
   return 401. Amend the design's "read path enforces it" sentence and the
   Violations text to name every path covered.

2. **B2. `/auth/sign-in/username` is a second password sign-in that bypasses the
   in-flight guard.**
   `EmailPasswordPlugin` always mounts `POST /sign-in/username`
   (`better-auth-api-0.10.0` `src/plugins/email_password.rs:489-492`). It runs
   `sign_in_username_core` (`:454-467`), which checks the hash and creates a
   session. Sign-up accepts a `username` and stores it (`:314-315`), so the squatter
   can set one. The gate's guard covers only `canonical == "/auth/sign-in/email"`
   (`oauth_routes.rs:500-510`). The username route falls through to `next.run`
   unguarded.

   Event order: the squatter's username sign-in checks the hash before T, and its
   session row is written after T. `created_at > T`, so the read-path cutoff does
   not void it, and no guard runs. The squatter holds a live session on the owner's
   account. Amendment "Known limits" says "Every interleaving is covered", which is
   false for this route.

   Fix: add `/sign-in/username` to the disabled paths in `better.rs:51-56`. The
   product has no username sign-in. Alternatively, extend the guard branch at
   `oauth_routes.rs:500-510` to cover it. Test: the `SignInHold` race against
   `/auth/sign-in/username`, asserting no session, plus a 404 on the path. Correct
   the "Every interleaving is covered" sentence to match what is tested.

## Carried to Plan

C1. **Not compiled.** Check these first, since they are likely compile errors:
   `oauth_routes.rs:415` needs `AuthConfig: Clone` (also `:596`);
   `oauth_routes.rs:416` `create_session(&user, None, None)` arity;
   `oauth_routes.rs:419` `create_session_cookie(token, config)` argument types;
   `middleware.rs:116` `session.created_at()` on `AuthSession`;
   `oauth_routes.rs:605` `user.metadata()` on `AuthUser`;
   `oauth_routes.rs:323` `state.allsource.core` visibility and
   `IngestEventInput::with_expected_version`;
   `oauth_routes.rs:339` and test `:657` `SdkError::VersionConflict` and `SdkError::Api` field shapes;
   `state.rs:54` `tokio::sync::Barrier` needs the tokio `sync` feature;
   `oauth_routes.rs:327` `Result::inspect_err` needs Rust 1.76 or later.

C2. **Audit outage leaves the squatter live** (`oauth_routes.rs:380-381`, test `:1022`).
   Stated limit, Plan 16. The victim is refused and the squatter keeps the account.
   Founder to decide whether a fail-closed order is possible before the library link.

C3. **Domain data keyed by user id carries to the owner** (`apps/api/src/presentation/handlers/users.rs:96`).
   Plan 15. The owner inherits the squatter's records.

C4. **Multi-instance clock skew** between `S` and `T` (Plan 14). Stated.

C5. **"Verified account keeps both credentials" is test-only** (`oauth_routes.rs:399-401`,
   `tests/google_sign_in.rs:792`). Plan 17. Stated.

C6. **`allsource-auth` is off by default** (`apps/api/Cargo.toml:20-24`, `default = []`).
   Without it the library state and sessions are per-process memory, and the
   `Coordination::Shared` claim does not hold across instances. Confirm the production
   feature set before the Plan calls cross-instance single use closed.

C7. **Live-Core test not run.** `two_instances_sharing_one_core_admit_one_session`
   (`tests/google_sign_in.rs:887-925`, `#[ignore]`). Tenant scoping of the claim write
   is unverified (design line on tenant scoping).

C8. **Test gaps.** No test of library session routes after a revocation (B1). No test
   of the username route race (B2). `gate_path_variants_never_sign_anyone_in`
   (`tests/google_sign_in.rs:1038`) asserts the outcome, not what the library does
   with each spelling.

C9. **Check-then-create race** on email uniqueness (Violations; Plan 3).

C10. **Legacy mixed-case rows** cannot sign in (Decisions, Email form; Plan 4).

C11. **Scan limit** (`better-auth-allsource/src/client.rs:247`, `limit=10000`, design Open
   item 5). After B1 is fixed the cutoff covers the scan miss, but the scan still
   decides which rows are deleted. Keep the launch limit.

Carried count: 11 items (C1 to C11).

verdict: findings
