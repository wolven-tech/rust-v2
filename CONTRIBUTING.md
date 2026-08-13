# Contributing

## The short version

```bash
cargo xtask ci
```

That is the whole gate, and CI runs the same command. If it passes locally it
passes in CI, because there is nothing to keep in sync — no shell script
duplicated into YAML.

Two things it does **not** cover, each for a stated reason:

| | Command | Why it is separate |
|---|---|---|
| Supply chain | `cargo deny check advisories bans licenses sources` | Needs a tool install |
| Live Core | `cargo xtask live` | Needs a running AllSource Core |

## Before you change anything structural

Read [`docs/architecture/001-rust-v2-allsource-foundation.md`](docs/architecture/001-rust-v2-allsource-foundation.md).
The code records *what*; that document records *why*, as numbered decisions
(`D1`–`D21`), risks (`R*`) and open questions (`OQ-*`) that comments cite
directly. A comment saying "§2.2 trap 1" is pointing at a real paragraph.

If you overturn a decision, edit the decision. A code change that silently
contradicts a recorded one leaves the next person with two sources of truth and
no way to tell which won.

## Rules that are not negotiable, and what enforces each

| Rule | Enforced by |
|---|---|
| Events are append-only; fields may only be **added**, with `#[serde(default)]` | `crates/rv2-events/tests/golden/` — one captured payload per released version, all of which must still deserialize |
| Folders are pure and total: no clock, no network, no `unwrap()` on payload shape, idempotent `apply` | Review. A rebuild replays the whole store, so an impure folder produces a different answer each time |
| Nothing server-only is reachable from a Dioxus app | `cargo xtask ci`, by real cross-compile in both directions |
| No new HTTP dependency brings `native-tls` | `deny.toml` bans `openssl-sys` and `native-tls`. **Check the TLS feature of anything that speaks HTTP** — two dependencies here defaulted to native TLS, so treat it as the expectation |
| Every crate declares its layer and depends only on strictly lower ones | Review, and the crate-level doc comment on each `lib.rs` |
| A background job is idempotent and safe to skip | Review. `rv2-jobs` runs **every job on every instance** — a job whose second concurrent execution is a defect does not belong there |
| A metric label has bounded cardinality | Review. Label by matched route, never by path or entity id |

## Adding a dependency

Ask, in order:

1. **Is it official?** `posthog-rs` and `resend-rs` are vendor-maintained. That
   is why they are dependencies rather than a hand-rolled HTTP client.
2. **What TLS does it default to?** See above.
3. **Which layer does it land in, and does that keep the WASM boundary?** A
   crate reachable from `apps/app` or `apps/web` cannot pull `tokio` with `net`,
   `reqwest`, or `getrandom`.
4. **Does it need a backing store this workspace does not have?** AllSource is
   the only datastore — there is no relational database and no Redis, and
   `cargo xtask ci` greps for the names of the ones this replaced. If a line of
   yours genuinely needs one of those words, mark it `predecessor-mention-ok`.

## Stylesheets

The compiled CSS is committed, so a fresh clone renders correctly with no build
step. After editing any Tailwind class — in an app **or** in `crates/rv2-ui` —
run `cargo xtask styles` and commit the result. `cargo xtask ci` fails if what is
committed has gone stale.

## Commits and pull requests

- One reviewable change per PR. The gate is fast; small PRs are cheap.
- The commit message says *why*. The diff already says what.
- If you find a defect while doing something else, fix it in its own commit.

## Tests

Prefer a test that would have caught the bug over a test that describes the
code. The suites here that have actually earned their keep are the ones that
assert a **behaviour of something external**:

- `crates/rv2-allsource/tests/core_contract.rs` — what AllSource Core genuinely
  does, `#[ignore]`d because it needs a live server. Every AllSource defect this
  codebase has hit was an assumption written in a comment and checked nowhere.
- `crates/rv2-events/tests/golden/` — that old events still decode.

Live tests **fail loudly** when Core is unreachable rather than skipping, because
a silently-skipped acceptance test is indistinguishable from a passing one.

## Reporting a security issue

See [`SECURITY.md`](SECURITY.md). Do not open a public issue.
