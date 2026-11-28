design-sha256: de3fa6a694af66c0e72ebe0d79038fec65874dae0406db7b174ddb681b00e8ac

# Review A, round 3: Google sign-in design (2026-11-26)

Checked against the round-3 design and the tree at this head: `apps/api/src/infrastructure/auth/{oauth_routes,oauth_glue,better}.rs`, `apps/api/src/lib.rs`, `apps/api/tests/google_sign_in.rs` (test names and bodies for the rows listed), and the better-auth 0.10.0 / better-auth-api 0.10.0 / better-auth-core 0.10.0 sources in the cargo registry.

Not checked: tests were not run in this review. The `api` lib count (43) was not recounted. Core query ordering (open item 5) was not checked.

## Round-2 findings and carried items

| # | Round-2 item | Status | Evidence |
|---|---|---|---|
| B1 | Planted valid pending cookie, login CSRF (blocking) | closed | Cookie renamed: `oauth_glue.rs:19` `PENDING_COOKIE = "__Host-rv2_oauth_pending"`. `pending_cookie` (`oauth_glue.rs:265-279`) counts only that exact name and refuses two or more values (`DuplicateCookie`). Unit rows `the_unprefixed_name_is_not_read` (`:472`) and `a_second_pending_cookie_is_refused_wherever_it_sits` (`:489`). Integration `a_planted_second_pending_cookie_is_refused` (`google_sign_in.rs:473`) with attacker cookie first. Attributes row `the_pending_cookie_is_host_prefixed_and_hardened` (`:492`). Violations and §5.4 (`docs/architecture/001-rust-v2-allsource-foundation.md:1050`) updated |
| C1 | `/auth/refresh-token`, `/auth/get-access-token` ungated (should) | closed | `better.rs:61` `UNMOUNTED_OAUTH_ROUTES`, applied at `better.rs:93-98`. better-auth 0.10.0 `handlers/axum.rs:86-91` skips disabled plugin routes at registration, so no route exists for either path. `handle_request` also checks at dispatch; unit `the_user_routes_that_write_role_or_email_are_not_mounted` (`better.rs:210`). HTTP row `other_methods_on_the_oauth_paths_are_not_found` (`google_sign_in.rs:521`, POST on both paths) |
| C2 | HEAD with valid cookie consumes code and state (should) | closed | Gate refuses non-GET on the callback before any library call: `oauth_routes.rs:321-323`. Row `a_head_callback_is_not_found_and_spends_nothing` (`google_sign_in.rs:435`): HEAD with and without cookie gives 404, then a GET with the cookie succeeds. Method table rows `:527`, `:532` |
| C3 | Only first `set-cookie` forwarded (should) | accepted as stated limit | `oauth_routes.rs:283-288` takes the first match. Library response headers are `HashMap<String, String>` (better-auth-core 0.10.0 `types.rs:213`), so one value per name is all it can carry. Stated in design Violations |
| C4 | Unit rows for path length, `\` later, NUL (nit) | closed | `a_path_over_the_length_cap_is_refused` (`oauth_glue.rs:500`), `a_backslash_later_in_the_path_or_a_nul_byte_is_refused` (`:509`) |
| C5 | Squat decision not recorded (should) | open | Design Open item 1 and Plan bead 5 still ask the user to choose (A), (B) or (C). Owed before launch; a user decision, not a design defect |
| C6 | Test counts from a run (nit) | open, not verified here | Integration: 23 `#[tokio::test]` in `google_sign_in.rs` (lines 321-800), matches "23 passed". The api lib count of 43 and the other counts were not recounted or run |
| R1-2 | Expired verification rows never swept | accepted as stated limit | `delete_expired_verifications` (`better-auth-allsource` `adapter.rs:660`) has no caller in `apps/` or `crates/`. Design Decisions, Violations (E5) and Plan bead 1 state it |
| R1-3 | Pre-registration squat | open | Same as C5 |
| R1-5 | Non-atomic single use across processes | accepted as stated limit | In-process `claim` at `oauth_routes.rs:74`, called at `:242` before the library. Cross-process window in Decisions |
| R1-6 | Core scan limit | accepted as stated limit | Open item 5 and Plan bead 6 unchanged. Not verified against Core |
| R1-9 | Encoded or non-ASCII targets refused | accepted as stated limit | Open item 4, Plan bead 7. Refuses at `oauth_glue.rs:178-192` (fails closed) |

Design claims checked for line accuracy, all match the tree: `oauth_glue.rs` 19, 22, 145, 155, 178, 252, 265; `oauth_routes.rs` 74, 121-127, 184, 201, 205, 242, 261, 286, 321, 324, 334, 359; `better.rs` 51, 61, 107, 108, 138.

## New findings, blocking

None. Checked the sinks in design §4 (S1-S8) against the code. Each Location and Set-Cookie write is reached only through `gate` and its VERIFY. The gate covers every method on the callback, sign-in/social and link-social paths. No request value reaches S1 or S7. The callback cannot complete without a cookie whose provider, state and signature match, and without a single unused `claim`.

## Carried to Plan

1. (should) `oauth_routes.rs:217-220` reads `headers.get(COOKIE)`, which is the first `Cookie` header field only. Use `headers.get_all(COOKIE)`, join the values with `; `, and pass the result to `pending_cookie`, so the duplicate check sees every field. Browsers send one `Cookie` header, so this is not reachable from a browser today. It is a gap in the stated VERIFY (S4a: "exactly one value").
2. (should) A refused callback clears the browser's pending cookie (`oauth_routes.rs:205`, `glue::clear_cookie`). A top-level navigation to `/auth/callback/google?code=x&state=y` with a mismatched state, sent to a victim who has a live flow, aborts that flow and leaves no session. Add it to Violations as a stated denial, next to "Denial by duplicate".
3. (should) Two starts in one browser (two tabs, or a double click on "Continue with Google") overwrite the single pending cookie (same name, `Path=/`). The earlier tab's callback then gets `StateMismatch` and lands on home. Not a security issue. State it as a known limit in the sign-in view bead (Plan bead 11).
4. (nit) `Refusal::Missing` is used for a declined consent with no `code` (`oauth_routes.rs:237-240`). Its doc comment (`oauth_glue.rs:77-78`) says "No cookie, or no pending value". Add a `NoCode` variant, or fix the doc, so the WARN log names the cause.
5. (nit) The "answer 404 for every method" claim (design Decisions, "Library token routes disabled") is true by the registration skip above, but the row covers POST only (`google_sign_in.rs:533-534`). Add GET and PUT on both paths to pin it.
6. (nit) Verification counts were not re-run in this review. Re-run `cargo test -p rv2-shared -p api` and record the output in the Build gate.
7. (should) Squat decision (round-2 C5, round-1 finding 3, design Open item 1). Record (A), (B) or (C) before launch.

verdict: clean
