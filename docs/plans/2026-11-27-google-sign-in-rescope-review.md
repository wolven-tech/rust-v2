design-sha256: c51f1587e665b5c34701bbb87e234fcddb77f466ef0200d01cdf62feb0578c3f

# Google sign-in rescope review (Amendment 1, rescoped)

Scope: the design `docs/plans/2026-11-26-google-sign-in-design.md` against `apps/api/src/infrastructure/auth/oauth_routes.rs`, `better.rs`, `middleware.rs`, `apps/api/tests/google_sign_in.rs`, and better-auth-api 0.10.0 `src/plugins/oauth/handlers.rs`. Not compiled, not run.

## Findings

1. **blocking.** `oauth_routes.rs:414-425` and `:434-454`. An audit failure on a verified link calls `undo_link` on a verified account. `undo_link` deletes every `google` account row of that user, including rows from earlier sign-ins, and this callback's session. This path is reached on the existing-account branch (`handlers.rs:554-590`, no new link), so a verified owner with an established Google link loses it while the audit store is down. This contradicts design S7 (line 225: "Reached only when the account is unverified"), E10 (line 169: "deletes only rows this callback created"), and the comment at `oauth_routes.rs:430-433`. Fix: delete only Google account rows with `created_at` at or after this callback's start (`AuthAccountMeta::created_at`, better-auth-core `entity.rs`), or on a failed audit of a pre-existing link refuse without undoing. Correct the comment and design S7/E10. Test: verified owner with an existing Google row, `FailAudit` on a second sign-in; the row is still present afterwards.

2. **should.** `oauth_routes.rs:289-295`, `:299-304`, `:393-408`. Early `Err` returns skip `undo_link`: library error or non-200 after the library wrote the link, `NoSessionCookie` after the session was created, and settle's body, user-id and user-read failures. The browser side is fail closed (`callback` clears only the pending cookie, `:219`; no `Set-Cookie` is forwarded). The residue is a Google account row on an unverified account, which the next callback for that identity refuses and undoes. Fix: take the user id from the library body before any early return, and undo on every `Err` after the library has answered with a user id. State the residue in design Violations. Test: library reply without `Set-Cookie`, then assert no Google row remains.

3. **should.** The pre-check the focus question asks about is not available in this library. `map_user_info` is a plain `fn` pointer (better-auth-api `providers.rs:43`) with no database handle, and the link decision runs inside `callback_core` after token exchange and userinfo (`handlers.rs:506`, `:595-611`). Checking first needs a library change or a hand-rolled callback. A window-free option: disable linking (`handlers.rs:629-632` refuses with no write) and restore verified linking through a session-bound link flow (`link-social` is 404 at `oauth_routes.rs:506-508`). Keep the post-hoc settle for now, and write this choice into Decisions so the window is a stated decision.

4. **should.** Origin mismatch. Design line 45 sends the unverified refusal to `O/login` (O = admitted origin). The code (`oauth_routes.rs:214-215`) uses `home.origin`, and S3 (line 221) says home. Pick one and fix line 45.

5. **should.** Audit scope against the rule. `oauth_routes.rs:414-419` audits password accounts only. The rule says audit every verified link. A passwordless verified account is reachable when Google issues a new `sub` for the same verified address (link path, `handlers.rs:595-611`), and that link is not audited. Founder decision: audit every linked account, or record the exception in Decisions and E11.

6. **should.** Tests do not prove the undo. `tests/google_sign_in.rs:759-777` and `:1010-1035` compare `user_by_email` (returns `Option<String>`, `:317`), which is the id only. Nothing asserts that the Google account row is gone, that the callback's session row is gone, or that a second Google sign-in with the same identity is refused. The comment at `:755-757` ("exactly as it was") is not shown by the check. Fix: compare `get_user_accounts` ids and the full user row before and after, then repeat the sign-in and expect a refusal.

7. **should.** Concurrency and settle error paths are untested. The only concurrency test (`:626`) is one cookie against the single-use claim. The two-instance test is `#[ignore]` (`:879-919`). No in-process test races two distinct states on one unverified identity (both refused, no session, no residual row). No test covers the settle error paths in finding 2.

8. **nit.** Design lines 98-100 (zero-knowledge shares) do not bear on the rule and repeat Open question 5 (line 396). Delete one.

9. **nit.** Design Violations (lines 241-250) do not say that an audit row can outlive an undone link when the SDK retries after a commit. One line.

## Answers to the focus questions

- **Window (1).** Between the library's link write and `undo_link`, the linked identity is used only by another callback for the same identity. That callback takes the existing-account branch (`handlers.rs:554-590`) and settle refuses it. Nothing else reads Google accounts: `get-access-token` and `refresh-token` are unmounted, and `link-social` is 404 at the gate. The library's session token sits in the response body; it reaches the browser only through the success-path `Set-Cookie` (`oauth_routes.rs:209`, `:297-307`). The refusal is enforced before any usable session reaches the browser. An undo that fails or is skipped leaves residue, not a session: findings 1 and 2.
- **Session routes (2).** `ExtractAuthUser` (`middleware.rs:88-111`) reads the session row and the user row only, with no revocation check left. No symbols from the removed revocation remain in `apps/` or `crates/`. The route that would have needed a cutoff (verified link over a live password session) cannot occur, because an unverified account is never kept linked. Verified clean.
- **Stale or contradictory text (3).** Findings 1, 4, and the S7/E10 wording under finding 1.
- **Tests (4).** Verified link: present. Unverified refused: present, but weak (6). Unverified Google: present. Mixed case: present, verified side only. Concurrency: missing (7). Audit outage: first-link case only, so finding 1 is not covered. The live-Core test was not run.

verdict: findings
