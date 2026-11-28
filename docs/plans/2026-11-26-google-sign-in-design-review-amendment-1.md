design-sha256: 9c35671c6e7ea8745e977fd6cb9efe6984e5c512bf107216699e94a8fdb56700

# Review: Google sign-in design, Amendment 1

Fresh-context review of the founder's Amendment 1 (design section "## Amendment 1 (founder, 2026-11-26)" and the Decisions, Violations and Plan text it changed). Code read in the worktree at `/Users/decebaldobrica/Projects/founder-mode/rust-v2-google-signin`. Library source read from `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` (better-auth-api-0.10.0, better-auth-core-0.10.0, allsource-0.24.0, allsource-core-0.30.2) and from the vendored tree `vendor/better-auth` and `crates/better-auth-allsource`. Paths below are repo-relative unless marked as registry paths.

Nothing was compiled or run. The Core-backed test was read, not executed.

## Checked and holds

- **Single use at Core.** The first append at expected version 0 is accepted; a second gets `VersionConflict`. Core's entity version starts at 0 and stores `current + 1` (`allsource-core-0.30.2/src/store.rs:816-833`). The single-event HTTP path forwards `expected_version` (`allsource-core-0.30.2/src/infrastructure/web/api.rs:260`). The client maps 409 `version_conflict` to `Error::VersionConflict` (`allsource-0.24.0/src/client.rs:249-256`).
- **Email verification gates the link.** `verified_google_user` errors before any library write. The library turns that error into a failure, not a skipped mapping (`better-auth-api-0.10.0/src/plugins/oauth/handlers.rs:506-507`).
- **Password hash location.** The hash lives in user metadata (`better.rs:44-46`, `email_password.rs:302-309`). The adapter's `update_user` replaces metadata wholesale (`crates/better-auth-allsource/src/adapter.rs:234-236`), so `metadata: Some({})` does clear it.
- **Sign-up sets no verified flag.** Password sign-up does not set `email_verified` (`email_password.rs:311-320`), so squatter accounts start unverified.
- **Settle covers repeat callbacks.** The existing-account branch returns the linked user before any link logic (`handlers.rs:554-590`), so `settle_email_match` also runs on repeat sign-ins.
- **Target safety.** The raw target must be printable ASCII (`oauth_glue.rs:223-228`, `:236`). `#` is refused (`:189`). The host check is at `:206-210`. A raw CR or LF cannot reach `Location`. `redirect()` also fails closed on an invalid header value (`oauth_routes.rs:504-515`). The decode loop's bound is correct (`oauth_glue.rs:252`).
- **Claim ordering.** The claim is taken before the library call (`oauth_routes.rs:247-251`), so a failed callback spends the state and nothing retries the code.

## Findings

1. **blocking. Revocation order lets a live squatter session survive.** `settle_email_match` deletes sessions (`oauth_routes.rs:394`) before it clears the hash (`:397`). A password sign-in verifies the hash and then creates a session (`better-auth-api-0.10.0/src/plugins/email_password.rs:365` onward). Argon2 verification takes long enough that a sign-in can read the old hash before the clear, then create its session after the delete. The window between `:394` and `:397` is deterministic, not just a race. A change-password in flight can also write a full metadata snapshot with a new hash after `:397` (`adapter.rs:234-236`).
   Fix: clear the hash and set verified first (`:397`), then delete sessions. Close the in-flight case with a revocation cutoff that the session read checks (for example `sessions_revoked_at` in metadata, with sessions created before it rejected), or write down the residual window as a launch limit. Add a test where a password sign-in is held between hash check and session create, then the callback runs. The squatter's session must be dead afterwards. Amendment text at design line 344 and the sequence in `oauth_routes.rs:394-397` both need the change.

2. **blocking. "Every session of that account ends" is not guaranteed.** `delete_user_sessions` reads all sessions with `find_all_by_field("auth.session.created", "user_id", ...)` (`crates/better-auth-allsource/src/adapter.rs:361-364`, `client.rs:299-312`). That is a family scan with the 10,000 ceiling (design Open item 5). Past that ceiling, or if a session written moments earlier is not yet visible to the scan, the squatter's session is missed and nothing reports it. The design does not check read-after-write on this scan.
   Fix: either a per-user index that is read-after-write safe, or the cutoff from finding 1 (which does not depend on the scan). Until one of these holds, state the revocation as bounded by the scan limit, not as complete, in Amendment 1 and in the Open items.

3. **blocking. The base design contradicts Amendment 1 and its own test table.** Scope (design line 17) puts "linking Google to an existing account" out of scope. The S5 row (line 217) says "Linking is disabled (`better.rs:107`)", but `better.rs:107` now enables it. Test rows at lines 311-312 expect "no session, password account unchanged" and "no session, no second user" for an email match. `google_sign_in.rs:758-878` asserts the opposite. The approval artifact should not carry two rules for the same decision.
   Fix: change line 17 to in scope, rewrite S5 to name `verified_google_user` and the link as the VERIFY, and replace rows 311-312 with the auto-match and revoke rows that the test file already has.

4. **should. Audit after link leaves the squatter live during an audit outage.** The library writes the link and creates a session (`oauth_routes.rs:268`). Settle writes the audit only after that (`:385`). If the audit fails, the callback refuses (`:392`), so the victim gets no session. The link is already written, and the squatter's password and sessions are untouched. Fail-closed holds for the victim's session only.
   Fix: state this in Decisions as a known limit: during an audit outage the victim cannot sign in and the squatter stays live. Add a test that makes the audit fail (see finding 8) and asserts both halves.

5. **should. The shared claim is not "never retried".** The design says "a failed write is refused, never retried" (line 124) and "Nothing is retried" (line 357). The SDK's `ingest_event` goes through `transport.post` (`allsource-0.24.0/src/client.rs:561-563`, `:141`), and `with_retry` retries transport errors and 5xx up to three times (`client.rs:193-210`). The effect is fail-closed: a retry after a timed-out append that actually committed gets 409, is refused as `AlreadyUsed`, and no code is reused. The audit write is retried too, so a duplicate `auth.account.revoked` row is possible.
   Fix: correct both sentences and the doc on `claim_shared` (`oauth_routes.rs:306`). If "never retried" is the intent, use a non-retrying call for the claim.

6. **should. S10 VERIFY claims a check the code does not do.** Design line 344 says "the email matched the Google identity's normalised email (S9)". `settle_email_match` (`oauth_routes.rs:364-414`) checks only the hash and the verified flag of whatever user the library returned. The only email match is the library's (`handlers.rs:595`), gated by `verified_google_user` (`better.rs:139-150`). No takeover path was found, because the S9 gate is the only email match.
   Fix: change the VERIFY text to name S9 as the email gate, or add an assertion. Do not leave a claim in the table that no line of code makes.

7. **should. Revocation keeps the squatter's user id, name and data.** The owner gets the squatter's user row. The name was set at registration from the squatter's body (`email_password.rs:311-313`). `update_user_info_on_link` defaults to false (`better-auth-core-0.10.0/src/config.rs:247`), so the name stays the squatter's. Domain data keyed by `actor.id` (`apps/api/src/presentation/handlers/users.rs:96`, `load_or_register`) also carries over. The owner inherits the squatter's profile and any records the squatter created under that id.
   Fix: decide and write down. Either reset name and image from the verified Google identity on revoke and clear or detach the domain profile, or give the owner a new user id and move the Google link to it. The design is silent on this today.

8. **should. Refusal arms for audit and claim are untested, and the shared claim is not run.** Local coordination makes the audit a log line (`oauth_routes.rs:351-354`) and makes the claim always succeed (`:312-314`). So `AuditUnavailable` (`:392`), `ClaimUnavailable` (`:326`) and the `VersionConflict` to `AlreadyUsed` mapping (`:323`) have no test. The only shared-mode test, `two_instances_sharing_one_core_admit_one_session` (`apps/api/tests/google_sign_in.rs:885-925`), is `#[ignore]` and was not run (design Verification, line 403-405).
   Fix: add a small seam (a trait or a test-only enum) so the audit and claim can fail on demand, with unit tests for each refusal arm and the mapping to `Refusal`. Run the ignored test against a live Core before any Plan item is marked done.

9. **should. Not compiled.** Design line 403 says so. Names checked and present: `SessionManager::new` and `create_session` (`better-auth-core-0.10.0/src/session.rs:26`, `:31`), `types_mod` re-export (`vendor/better-auth/src/lib.rs:32`), `SessionOps`/`UserOps` traits in scope (`oauth_routes.rs:25-28`). Not checked: `AuthConfig: Clone` for `config.clone()` at `oauth_routes.rs:409`, the `Error` re-export used at `:19`, and the three-argument `create_session` call at `:410-412`. The Plan marks bead 2 and bead 5 done, which needs `cargo check -p api` and the test run first.

10. **should. Stale text after the amendment.** Plan beads 2, 5 and 7 still read as open (design lines 420-436). Open items 1, 3 and 4 still ask the question the amendment answered (lines 453-458). S4 (e) still says the claim "admits the first use in this process" (line 216). Plan and Verification both say the amendment is done or pending in different words. Fix each to match the state.

11. **should. Gate path variants are untested.** The gate compares exact strings (`oauth_routes.rs:442`, `:464`, `:468-469`). The library strips `base_path` and then matches its own routes (`vendor/better-auth/src/core/auth.rs:274-279`). Percent-encoded and trailing-slash forms of `/auth/callback/google`, `/auth/sign-in/social`, `/auth/link-social` and the two sign-up and sign-in email paths are not in the test table. Add rows, and check what the library does with each.

12. **nit. `from_str_radix` accepts a leading `+`.** `u8::from_str_radix("+1", 16)` succeeds, so `%+1` decodes to `0x01` (`apps/api/src/infrastructure/auth/oauth_glue.rs:254`). It is refused today only because it is a control character. Fix: require `is_ascii_hexdigit` on both characters before parsing.

13. **nit. Stale unit test name.** `oauth_glue.rs:436` is `root_query_and_encoded_safe_paths_are_admitted`, and its body now tests query values only. Rename, and check the name against the one the design gives (`an_encoded_query_value_is_admitted_as_written`, design line 392).

14. **nit. Comment does not match the test.** `apps/api/tests/google_sign_in.rs:414-415` says "no session is issued for the refused part". The test asserts only the `Location` of home, and the sign-in itself completes (the tainted-target test at `:465-469` asserts that). Fix the comment.

15. **nit. Refusal docs out of date.** `oauth_glue.rs:95-97` says `NotSameOriginPath` covers "any percent-escape" and "a backslash" in the path. Escapes are now allowed in the query (`:197-199` refuses `%` only in the path). Say "in the path".

16. **nit. Unreachable branch covered only by a forced state.** In production no route sets `email_verified` true on a password account except settle, which also clears the hash. The "verified account keeps both credentials" branch (`oauth_routes.rs:381-383`) is reached in tests only through a direct `update_user` (`google_sign_in.rs:795-806`). Say so in the design, so no one reads that branch as a supported state.

## Not checked

- Compilation, `cargo test`, and the live-Core test.
- Tenant scoping of the claim write under the API key. Design line 360 already says so. Core's version check is per entity id, not per tenant (`store.rs:818`). The stream name carries a random state, so collisions are not a concern here.
- Whether `get_user_by_email` is case-sensitive in the adapter (design legacy-row text, lines 100-103).
- Google's own behaviour on `email_verified` and on email changes for an existing `sub`.

verdict: findings
