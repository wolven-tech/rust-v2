# What travels from `rust-v2` to the register, and in what order

`uk-tech-hiring-register.fly.dev` is served from `uk-tech-hiring-dashboard`, not
from here. That repo deploys `apps/web` — hand-written HTML, CSS and JS — with
nginx, after a Rust stage regenerates `register.data.js`. This repo is the
generalisable starting point; the register is where everything lands.

This is the source side of that migration: what changed here on 2026-09-21, what
it depends on that the destination does not have, and the four frictions that are
cheaper to know about than to rediscover.

## `rv2-hiring` exists in both, and they have diverged in both directions

This is not a copy waiting to be carried across. `uk-tech-hiring-dashboard`
took this repo's layout at `b117abd` and has been developing its own register
since — judging how central Rust is to a posting, attaching three AIM and
SmallCap boards, recording an application against a role. This repo grew the
UI side over the same period.

Both copies hold work the other does not:

| `crates/rv2-hiring/src` | here | the register |
|---|---|---|
| Files | 12 | 4 |
| `breakdown`, `dashboard`, `litmus`, `mandate`, `view`, `work_pattern`, `csv`, `engagement` | present | absent |
| `lib.rs` | 2.6K | **7.4K** |
| `links.rs` | **4.5K** | 2.3K |
| `model.rs` | **22.5K** | 16.2K |
| `text.rs` | **6.9K** | 6.2K |

Every shared file differs, and not in one direction, so neither side is an
ancestor of the other. Treating this as a copy would silently drop the
register's own recent work.

**Which means the split this repo should settle into is components, not
products.** `crates/rv2-ui` is a component kit and belongs here. The register's
domain and its page are product, they are being actively developed in the
repository that deploys them, and continuing to grow them here widens a fork
that somebody has to merge by hand. The UI work recorded below was done here
and is on the wrong side of that line — it is listed so it can be moved, not as
a claim that it should have been.

## The gap, measured

| | `uk-tech-hiring-dashboard` | here |
|---|---|---|
| Workspace members | 5 `rv2-*` crates, 2 tooling crates. No app is a member | apps and crates, including `rv2-ui` |
| `apps/` | `api`, `app`, `web` — static assets, not Cargo crates | Dioxus apps |
| `rv2-ui` | absent | the component kit |
| Tailwind | none. `styles.css`, `graph.css`, `skills.css` are hand-written | `assets/input.css` per app, `cargo xtask styles`, committed output |
| Async | none, deliberately: `ureq`, no tokio | the Axum api needs it |
| `xtask ci` | fmt, clippy, build, test, `register_js_is_current`, pure-crate | the same four plus `styles_are_current`, `no hardcoded hues`, `no_predecessor`, `wasm32 boundary` |

## What has to travel

In dependency order. Each step is useless before the one above it.

1. **`rv2-ui`.** A new crate there. Everything below assumes it.
2. **The Tailwind pipeline.** Per-app `assets/input.css`, `cargo xtask styles`,
   and the generated `tailwind.css` committed beside it. Without this the token
   work is inert: a semantic class with no `@theme` behind it is a declaration
   the browser drops, and the element renders with whatever it inherited.
3. **Two gates:** `styles_are_current` and `no hardcoded hues`. The second is
   what caught a `bg-white` that had been drawing a white card on a dark page.
4. **`apps/hiring` as a workspace member.** No app is one there today.

## `apps/hiring` fits the destination better than it looks

Its manifest already says it: *no `fullstack`, no `server`, no `tokio`, no
`reqwest` — the register is a file compiled into the binary, so the crate stays
100% wasm32*. That matches the destination's deliberate no-async stance rather
than fighting it, and it is why `rv2-hiring` can cross without dragging a
runtime behind it.

The deploy shape is not an obstacle either. A wasm bundle is static files, so
nginx serves it unchanged. Measured here: 883,320 bytes raw, **311,383 gzipped**,
against a few hundred KB of hand-written JS. Cold release build 153s.

## Four frictions, each with the cost attached

**A gate ordered ahead of another hides it.** Proving `no hardcoded hues` could
fail meant planting an offender. Placed under `src/`, it failed the
*stylesheet freshness* step instead — Tailwind's `@source` generated a utility
for it — and the hue check never ran. Proving a check fails means proving it
fails at its own step; the probe has to live where Tailwind does not scan.

**`accent` means two different things.** shadcn spends it on a hover surface.
Ledger spends it on the focus ring, chosen lighter than the FTSE 100 blue so a
ring never reads as market data. One CSS variable name, two incompatible jobs,
so the registry aliases deliberately omit it and a borrowed component wanting
shadcn's gets `bg-raised` written in. A test asserts it stays omitted.

**A borrowed component that picks a shade rather than a role breaks in whichever
theme its author was not looking at.** The tab list filled its current tab with
`background` on a `muted` track — correct on a light theme, and on a dark one
`background` is darker than the track, so the current tab became the recessive
thing in the row. `primary` / `primary-foreground` inverts with the theme
instead of assuming one.

**A dark surface inside a light app needs its own ink scale.** The marketing
site has two — a foil card and a chip — each carrying three levels of text. The
token set modelled text on the page at three levels and text on an ink fill at
one, so there was no way to say "secondary text on this fill" and the site said
`text-slate-200`. `on-ink-muted` and `on-ink-faint` exist now, measured against
each theme's own `--color-ink` as the fill.

## What changed here, by commit

| | |
|---|---|
| `4f551b2` | Coverage counted over the company-grain pool, so a role filter still reports the boards it could not read |
| `004a543` | Two-arm ordinal palette: green for buckets clearing the filters, warm neutral for those missing them |
| `115abe6` | `rv2-ui::tabs` — controlled, no `TabsContent`, with the ARIA the pattern needs |
| `bc78d0a` | The current tab is the filled control |
| `327b5ee` | 64 classes in the kit stop naming hues |
| `c028f15` | 34 more in the apps, and the scan moves to `xtask` |

## Limits

Two steps per arm is a measured ceiling, not a preference: four steps cleared
every check the validator makes and still rendered as one flat colour, and
raising the bar from 6px to 10px did not rescue it. An arm has roughly 0.2 of
OKLCH lightness to spend before it collides with a reserved token or stops
clearing its own track.

`apps/web` here has not been re-rendered since the 34 app-level changes. The
`on-ink` values were chosen to preserve its appearance and the contrast numbers
say they do, but that is arithmetic rather than a look. Its foil card and chip
are the two that moved onto a new scale, so they are what to check.
