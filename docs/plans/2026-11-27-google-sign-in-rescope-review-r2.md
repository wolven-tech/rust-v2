design-sha256: 12a9401d997a367bb28ab7fe9ff67f4d94aa008c01f6f2e935e85d6891b9a035

# Google sign-in rescope review, round 2 (Amendment 1, rescoped)

Scope: design `docs/plans/2026-11-26-google-sign-in-design.md` against `apps/api/src/infrastructure/auth/oauth_routes.rs`, `oauth_glue.rs`, `apps/api/src/infrastructure/state.rs`, `apps/api/tests/google_sign_in.rs`, and better-auth-api 0.10.0 `src/plugins/oauth/handlers.rs` and `types.rs`, plus `crates/better-auth-allsource/src/adapter.rs`. Not compiled, not run.

## Status of round-1 findings

| # | Sev | Status | Evidence |
|---|---|---|---|
| 1 | blocking | closed | `oauth_routes.rs:493-496` filters on `created_at() >= started` for verified accounts. Test `an_audit_outage_on_a_repeat_sign_in_keeps_the_existing_link` (`tests/google_sign_in.rs:1083`). Clock caveat in Carried. |
| 2 | should | open (test) | Undo on non-200 (`:296`), NoSessionCookie (`:311`), settle DB read (`:432-435`) and settle fault (`:438-441`) is in code. Library `Err` (`:294`) has no body and stays residue, as the design states. No test for non-200 or missing Set-Cookie. |
| 3 | should | accepted | Design Decisions "Why there is no pre-check" and bead 16. |
| 4 | should | closed | Design line 45 and Decisions line 85 say primary origin; code `:217` uses `home.origin`. |
| 5 | should | closed | Design E11 and Decisions: every accepted Google sign-in audited. Code `:447-458` audits every verified account, password or not. |
| 6 | should | closed | Undo tests compare `account_ids` and `user_row` before and after (`:1061-1102`, `:803-828`). Callback session row not asserted; the token never reaches the browser, stated in the response table. |
| 7 | should | closed in part | Race test `:1124` and settle-fault test `:1106` exist. Non-200 and missing-cookie error paths untested (see 2). |
| 8 | nit | closed | Zero-knowledge paragraph removed. Open question 5 remains. |
| 9 | nit | closed | Design Violations: "An audit row can outlive an undone link." |

## Round-2 focus answers

- **`created_at >= started` clock.** Both sides read `chrono::Utc::now()` in one process. Account `created_at` is set in `crates/better-auth-allsource/src/adapter.rs:461` and in memory `memory_traits.rs`, both after `started`. Both keep nanoseconds (chrono serde default). A row this callback creates is therefore at or after `started`. A pre-existing row is never re-created: the existing-account branch (`handlers.rs:554-590`) only calls `update_account`. A pre-existing row can be caught only by a concurrent callback for the same identity that wrote its row after this callback's start (see Carried). Core round-trip precision is not tested; the live test has not run.
- **Error paths that leave a session.** None. `callback` (`:209-223`) forwards `Set-Cookie` only on the `Ok` path, and `complete` returns `Ok` only after `settle_email_match` passes. Library tokens that never reach the browser are not usable. Every refusal after a user id is known runs `refuse_link`, which deletes the token's session row.
- **Linked identity on an unverified account.** Residue remains only on the uncovered paths: library `Err` (`:294`), a 200 without `user.id` (`:423-424`), and a settle DB read failure (`:432-435`, where `refuse_link` runs with `all=false`). The next callback for that identity takes the existing-account branch, settle sees the account unverified, and `all=true` removes the rows. Password sign-in does not read Google rows. Accepted in the design.
- **Widened audit.** Fails closed: `record_audit` error runs `refuse_link(false)` (`:453-457`), which touches only rows created at or after `started`, and no session is issued. A pre-existing link survives a repeat sign-in refused by audit. Availability cost: a Core audit outage refuses all Google sign-ins, including Google-only accounts. Design says so.

## Blocking

None new.

## Carried to Plan

1. **Tests for finding 2 (should).** Add: library reply without `Set-Cookie`, then assert no Google row; non-200 with body, then assert no Google row. `apps/api/tests/google_sign_in.rs`, new cases.
2. **Clock step (should).** `started` (`oauth_routes.rs:281`) uses wall clock. A backward step makes a new row's `created_at` earlier than `started`, so `refuse_link` (`:495`) leaves it on a verified account. State it in design Violations, or compare against a monotonic reference.
3. **Concurrent undo (should).** `refuse_link` (`:495`) on a verified account also removes Google rows that a concurrent callback for the same identity created after this start. That callback's session survives without its link. Design E10 and S7 say "never an earlier link", which holds, but do not cover this case. State it.
4. **Round-trip precision (should).** Verify that `created_at` keeps nanoseconds through Core (`better-auth-allsource` adapter read path). Or record a measured limit. Live test not run.
5. **`Faults` in production state (should).** `state.rs:45-65` compiles the outage switches into every build. No route sets them, and the comment says "false outside tests". Gate with `cfg(test)` or a test feature, or keep the comment as the only guard and say so in Decisions.
6. **Residue with unknown verification (nit).** `refuse_residue` (`:469-473`) and settle (`:432-435`) pass `all=false` when the user read fails, so pre-existing residue on an unverified account survives. Design Violations should name it; the next callback refuses it.
7. **200 without `user.id` (nit).** `settle_email_match` (`:423-424`) returns without undo. Design Violations should name it with the library-Err residue.
8. **Compile checks, not run.** `Refusal` derives `Debug, Clone, Copy, PartialEq, Eq` (`oauth_glue.rs:75`), enough for `assert_eq!` at `oauth_routes.rs:648`. Still open: `AuthUser: Serialize` for `user_row` (`tests/google_sign_in.rs:363`), `delete_account(&str)` and `delete_session(&str)` signatures, and `AuthAccount` trait import in `oauth_routes.rs`. Listed in design Verification.

verdict: clean
