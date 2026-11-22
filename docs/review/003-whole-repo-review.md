# 003 — Whole-repository review

**Date:** 2026-11-22, fixes 2026-11-23
**Scope:** the whole repository at `5cc657d`, read in five lanes: auth, data
path, API and client, UI and apps, gates and docs. Static reading only; nothing
was run against a live Core during the review. Each finding marked
*confirmed* was traced end to end in the code; *reported* means a reviewer
traced it and it has not been re-checked independently.

**Status legend:** ✅ fixed in the PR that adds this document · ⏳ open, with
the reason.

---

## Verdict

The repository was not safe to put in front of users. Four defects stood out:

1. Any signed-in user could change server-owned fields on their own account,
   including the role and the stored password hash.
2. Single-use verification tokens could be redeemed again until they expired.
3. Two read paths omitted `tenant_id`, so against single-node Core they
   returned nothing, with HTTP 200, forever.
4. The post list never applied an edit or a delete, and after a restart it
   served deleted posts again.

None of the four was covered by a test that runs in `cargo test`, and `main`
had no branch protection, so a red check could merge.

---

## Auth

### 1. `/auth/update-user` wrote `role` and `metadata` from the request body ✅ confirmed

`vendor/better-auth/src/core/auth.rs` copies `role` and `metadata` from the
JSON body into `UpdateUser`; the adapter saves both. `middleware.rs` turns the
stored `role` into the request's role set, and the adapter keeps the password
hash in `metadata`. Nothing checks `Admin` yet, so the role half was latent; the
hash half let a stolen session replace the password without the old one.

**Fix:** `apps/api/src/infrastructure/auth/better.rs` no longer mounts
`/update-user`, `/change-email`, `/delete-user` or `/delete-user/callback`
(`UNMOUNTED_USER_ROUTES`). The app uses none of them; profile edits go through
`PATCH /users/{id}`. The vendored library is unchanged, per its provenance
rules. A unit test asserts all four answer 404.

### 2. Consumed verification tokens stayed redeemable ✅ confirmed

`get_verification` scanned only `auth.verification.created`; consuming a token
appends `auth.verification.consumed`, which that scan never returned. The same
`*.created`-only scan sat under every list and lookup in the adapter, so
deleted entities stayed listed and changed fields (email, session expiry,
password hash) read as their first snapshot.

**Fix:** `crates/better-auth-allsource/src/client.rs` scans the whole family
(`auth.x.`), groups by the event's `entity_id`, takes the newest snapshot by
timestamp and drops tombstones. `find_by_field` matches current state only; the
`payload_filter` fast path, which could return a past snapshot or an unrelated
row, is removed. Recorded in that crate's `PROVENANCE.md`.

**Cost:** every lookup by field (email at sign-in included) is now a family
scan of up to 10,000 events. It was already the fallback path.

### 3. Email changes skipped uniqueness and verification ✅ via 1

Reachable only through `/update-user` and `/change-email`, both now unmounted.
The lookups also now see changed emails (2).

### Open

- ⏳ **Sign-up checks the email, then appends, with no expected version.** Two
  concurrent sign-ups with one email can both succeed; organisation slugs,
  members and two-factor rows have the same shape. Needs the expected-version
  append described at the `SEAM` in `crates/rv2-allsource/src/writer.rs`.
- ⏳ **`/auth` sits outside the API rate limiter** (`apps/api/src/lib.rs`), and
  the limiter's per-IP map never evicts. better-auth's own limiter default was
  not read.
- ⏳ **Email comparison is case-sensitive in the adapter.** Whether better-auth
  lowercases before calling it was not checked.
- ⏳ **Not reviewed:** cookie flags, CSRF, password-hash parameters and session
  expiry live in `better-auth-core`, outside this repository.

---

## Data path

### 4. `GET` / `PATCH /users/{id}` always answered 404 ✅ confirmed

`handlers/users.rs` read through the SDK's `query_and_fold`, which cannot send
`tenant_id`. **Fix:** it reads through `AllSource::fold_entity`.

A live run then showed a second cause: nothing ever appended
`identity.user.registered`. Sign-up runs inside better-auth, which writes only
its private `auth-user:<id>` streams. better-auth 0.10's database hooks could
not do it: `HookedDatabaseAdapter` does not implement `PasskeyOps`, so it is not
a `DatabaseAdapter`. **Fix:** the profile handlers register the domain user from
the session the first time its owner reads or edits it, which also covers
accounts created earlier. The live test `a_profile_belongs_to_its_owner_alone`
covers read, edit, another user's 403, and `/auth/update-user` answering 404.

### 5. The `GET /posts` fallback returned `[]` ✅ confirmed

Same defect via `query_events`; the truncation alarm could never fire. **Fix:**
`AllSource::events_by_type_prefix`, tenant-scoped.

For 4 and 5, `apps/api/tests/tenant_reads.rs` scans the API source and fails
on any SDK query-client read. It runs in plain `cargo test`.

### 6. Deleted posts came back after a restart ✅ confirmed

The worker removed a deleted post from memory, the flush wrote only live posts,
and the SDK can only upsert projection KV entries. The deleted post's last
snapshot stayed in the KV and was hydrated back at boot, with the cursor past
its delete. **Fix:** `PostsReadModel` keeps deleted ids and flushes a tombstone
under the same key; hydration lets a tombstone win in either order. Snapshots
flushed before this change still deserialize.

Against a live Core this was not enough, for two more reasons:

- **Hydration ran after `start()`** and raced the replay. An edit or delete
  replayed into empty state found no post, and the older snapshot merged in
  afterwards won. Hydration now runs before `start()`, into
  `ProjectionWorker::state()`, and a delete for an unknown post is still
  recorded.
- **The worker never applied an edit or a delete at all.** Core 0.23's append
  response numbers an entity's events 1, 2, 3, but queried and streamed events
  all carry `version: 1`, and the SDK worker skips any event at or below the
  last version it applied for that entity. Observed: after a restart the list
  showed deleted posts with their pre-edit titles.

**Fix:** the worker is not started (foundation doc C.2-f), and `GET /posts`
folds on read with the tenant-scoped scan. The consumer is renamed `posts_v2`,
so a worker restarted later rebuilds from zero instead of hydrating
`posts_v1`'s stale snapshots. Contract test B18 pins the version behaviour and
fails the day Core reports a real sequence. The vertical slice now asserts the
list shows the edit and drops the deleted post. **Cost:** every `GET /posts`
reads up to 10,000 `content.post.*` events.

### Also fixed

- ✅ A failed post read answered 404 "post not found"; it now answers 502
  `store_unavailable` (`ApiError::Read`).
- ✅ `GET /users/{id}` returned any user's email to any signed-in user; only
  the owner may read a profile now (`rv2_domain::user::can_view_profile`).
- ✅ Tenant-scoped reads built a new HTTP client per call and read
  `ALLSOURCE_TENANT_ID` per call; `TenantReader` holds one client and reads the
  tenant once.

### Open

- ⏳ **Appends are unconditional**, so two concurrent `PATCH /posts/{id}` can
  overwrite each other. Same `SEAM` as the sign-up race.
- ⏳ **Before the posts worker is restarted:** a failed hydrate is logged and the
  worker starts empty, already past older events; every instance shares one
  consumer and KV keys, so a lagging instance can flush an older view over a
  newer one; and the worker, checkpoint and KV calls send no `tenant_id`.
- ⏳ **Contract test B4** lists `content.post.updated`; the code emits
  `content.post.edited`.

---

## App and site

### 7. No way to sign out ✅ confirmed

**Fix:** the shell's header tail has a Sign out button; a refused sign-out says
the user is still signed in. `ProductHeader` now repeats its tail inside the
mobile menu, because the tail is hidden below 48rem.

### Also fixed

- ✅ If `GET /me` failed outright, the app showed an empty page forever; it
  now shows the error with Retry.
- ✅ A 401 mid-visit showed raw text with a Retry that could only fail; it now
  returns to `/login`.

### Open

- ⏳ Footer links to `/privacy` and `/terms`, which have no route.
- ⏳ `/kit` and `/motion` are in the sitemap with no `noindex`.
- ⏳ The crumple and vacuum animations ignore `prefers-reduced-motion`.
- ⏳ `ConsentFooter` and `Mailer::send` have no callers.
- ⏳ The client falls back to `http://localhost:4400` when `PUBLIC_API_URL` is
  unset at build time; malformed `CORS_ORIGINS` entries are dropped silently.
- ⏳ `rv2-ai` validation checks that citations exist and are unique, not that
  the text is supported by them.

---

## Gates

### 8. `main` had no branch protection ✅

The protection API answered 404 "Branch not protected", and the repository has
no git hooks, so every check was advisory. Protection with the three CI checks
required is set once this document's PR shows their names.

### 9. CI and the local gate differed ✅

CI set `RUSTFLAGS=-D warnings`; `cargo xtask ci` did not. CI no longer sets it
(clippy in the gate already denies warnings). The gate's clippy, build and test
steps pass `--locked`. Every CI job has a `timeout-minutes`.

**Still over the five-minute rule:** the `check` job measured 10m on a cold
cache (2026-11-22, run 45045462421), and `live Core` ran past 5m compiling
`allsource-core` on a cache miss (2026-11-23, run 45092120595). Both have a
15-minute timeout as a hang guard; the Core binary is now cached by version
alone. Meeting the rule needs `check` split or sharded.

### 10. `main`'s supply-chain check was red ✅

RUSTSEC-2026-0592, stack overflow in `url::Url::origin()` on nested `blob:`
URLs, `url 2.5.8`. The lockfile now pins 2.5.9.

### Open

- ⏳ The wasm32 boundary check matches workspace crate names only; `reqwest`,
  `posthog-rs` and `resend-rs` reachability is not checked.
- ⏳ The ISA scan misses `, button {` and `button{` (`tooling/isa/src/isa.rs`).
- ⏳ `deny.toml` sets `multiple-versions = "warn"` while its header says a new
  duplicate fails CI.
- ⏳ The Tailwind binary is downloaded and run without a checksum
  (`tooling/xtask/src/styles.rs`).

---

## How the fixes were verified

- `cargo xtask ci`: all checks passed.
- `cargo xtask live` against a private Core 0.23.0: 14 contract tests and 4
  vertical-slice tests passed, including B18 and
  `a_profile_belongs_to_its_owner_alone`.
- `apps/app` under `dx serve`, against `apps/api` and that Core, in a headless
  Chromium:
  - `/posts` signed out redirects to `/login`;
  - sign in, publish a post, see it listed first;
  - session cookie cleared, then Publish: back to `/login`, no post created;
  - at 375px the menu shows the email and Sign out, and signing out there ends
    the server session;
  - API stopped: "Could not reach the API" with Retry.
- Not checked: Firefox and Safari; the desktop Sign out button was seen but
  only the mobile one was clicked.
