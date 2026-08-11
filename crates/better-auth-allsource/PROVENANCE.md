# PROVENANCE — `better-auth-allsource`

This crate is **vendored**, not authored here. It is a **version bridge**, not a
fork.

| | |
|---|---|
| Copied from | `github.com/get-form-lab/getformlab` (private), `crates/better-auth-allsource/` |
| Source commit | `29adc2009fe1dc8ccf397cbd698f805234426acb` (branch `main`) |
| Copied on | 2026-08-11 |
| Files | `src/lib.rs`, `src/adapter.rs`, `src/client.rs`, `src/error.rs` — copied verbatim, 1,911 lines total |
| Upstream package | `better-auth-allsource` on crates.io, latest **0.14.12**; upstream repo `github.com/all-source-os/all-source` |
| Local `Cargo.toml` | Rewritten (not copied): `publish = false` added, and the description marked as vendored. Dependency versions match getformlab's. |

## Why this is vendored rather than a crates.io dependency

The published crate **exists** — `cargo info better-auth-allsource` reports
0.14.12 — so "it is not published" is not the reason.

The real reason is a semver conflict that cannot be patched around. Every
published version declares `better-auth-core ^0.8`:

```
$ curl -s https://index.crates.io/be/tt/better-auth-allsource | jq -r '"\(.vers) \(.deps[]|select(.name=="better-auth-core")|.req)"'
0.14.5  ^0.8
0.14.11 ^0.8
0.14.12 ^0.8
```

while `better-auth 0.10.0` requires `better-auth-core 0.10.0`. Both would land
in the tree, giving **two distinct copies** of the `UserOps` / `SessionOps` /
`DatabaseAdapter` traits, and `AuthBuilder::database(AllsourceAuthAdapter)`
would then fail its trait bound with an error that reads as nonsense ("expected
`UserOps`, found `UserOps`").

`[patch.crates-io]` cannot help: the problem is a semver-incompatible
*dependency of* the crate, not a bad version of the crate itself.

## Rules for this directory

1. **Bug-for-bug ports only.** Never add a feature here. Behaviour we need that
   upstream lacks goes in `apps/api`.
2. **Record every edit below.** A vendored file that silently diverges from
   upstream is the failure mode this file exists to prevent (risk R3).
3. **Delete this crate when the bridge is no longer needed.** The exit condition
   is: upstream's `better-auth-core` requirement matches the `better-auth` we
   pin. CI checks this weekly (`.github/workflows/vendor-check.yml`); when it
   fires, delete `crates/better-auth-allsource/` and change the
   `[workspace.dependencies]` entry to a version dependency.

## Local edits to the vendored sources

| Date | File | Change | Why |
|---|---|---|---|
| — | — | none yet | — |
