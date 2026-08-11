# Component kit + dependency strip — autoresearch ledger

Loop shape borrowed from [karpathy/autoresearch](https://github.com/karpathy/autoresearch):
propose one change, run it under a fixed budget, score it on a single scalar,
keep or discard, append a row, repeat.

## Setup

**Frozen corpus.** A component audit of a real marketing page,
[chargewindow-web.fly.dev](https://chargewindow-web.fly.dev/): nav with a
call-to-action, hero with dual actions, three-column numbered feature grid,
numbered process steps, itemised cost breakdown, one-off pricing block, FAQ,
multi-column footer. That page uses **no** modal, tab, tooltip or carousel, and
is light-mode only — so neither does the kit.

**Scalar.** `apps/web` release wasm, in bytes. Lower is better. Chosen because
it is the one number a component library can make worse without anyone
noticing, and because it responds to both halves of the task: adding components
pushes it up, removing dependencies pulls it down.

**Gate.** A proposal is only eligible to be kept if all of these hold:
`cargo build --workspace`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo fmt --all -- --check`, `cargo audit --deny warnings`, the
`wasm32-unknown-unknown` cross-compile, and `apps/web` still compiles as the
coverage fixture. A smaller bundle that drops coverage is a discard, not a win.

**Baseline.** Measured before any change:

| | |
|---|---|
| `apps/web` release wasm | **510,535 B** |
| compiled CSS | 23 B — *see R1, this was broken* |
| `rv2-ui` components | 9 |
| `cargo-machete` findings | 10 |
| workspace lockfile | 533 crates |

## Ledger

| # | Proposal | Scalar | Δ vs baseline | Verdict |
|---|---|---|---|---|
| 0 | Baseline | 510,535 B | — | — |
| 1 | Restructure `rv2-ui` into 6 modules; add 20 marketing components; move `PostCard` to `apps/app` to drop the `rv2-api-types` dependency | 557,653 B | +47,118 (+9.2%) | **Keep** — the scalar is the *price* of coverage, and the gate now includes a fixture page that did not previously compile. Kit dependencies went 2 → 1 (`dioxus` only). |
| 2 | Compile Tailwind for real via `tooling/tailwind` + the standalone CLI | 557,653 B (wasm unchanged); CSS 23 B → 22,266 B | 0 on the scalar | **Keep** — not a scalar move, a correctness fix. See R1. |
| 3 | Strip the 9 confirmed-unused dependencies found by `cargo-machete` | 557,653 B | 0 on the scalar | **Keep** — every one was server-side, so none touch the wasm tree. `cargo-machete` now reports clean. Kept anyway: build hygiene, not bundle size. |
| 4 | Drop the `logger` feature from `apps/web` | **490,481 B** | **−20,054 (−3.9%)** | **Keep** — −67,172 B (−12.0%) against proposal 3. A static marketing page has nothing to log at runtime. |

**Net: −20,054 B (−3.9%) against baseline, while adding 20 components.**

## R1 — the finding that mattered more than the scalar

`assets/tailwind.css` shipped as **23 bytes**: the literal, uncompiled
`@import "tailwindcss";`. Every command reported success — `cargo build`,
`cargo test`, `cargo clippy`, `dx bundle` — and the site rendered with **no
styling at all**. The architecture doc's claim that "`dx` has automatic Tailwind
detection in 0.7, so each app owns an `assets/tailwind.css` and `dx serve` runs
the build" does not hold for `dx build` / `dx bundle`, which only *copy* the
file.

This is the failure mode a component kit is most exposed to: the entire visual
layer is class strings that no compiler checks. `tooling/tailwind` now compiles
both apps and **fails the build** if the output is under 2 KB, so the silent
version cannot come back.

Two sub-findings:

- Tailwind cannot discover Rust files. `crates/rv2-ui` has to be named as a
  `@source`, or every class used only by the kit is tree-shaken away — leaving
  the app's own classes working and the components bare, which is a nasty thing
  to debug.
- Tailwind v4 scans up to the git root by default, so both apps were emitting
  byte-identical CSS containing each other's classes. `source(none)` plus
  explicit globs scoped it properly: 19,967 B → 18,118 B (`web`) and 18,512 B
  (`app`).

## The kit

29 components, `dioxus`-only, no router, no domain types, no JavaScript.

**Layout** — `Container` (3 widths), `Section` (3 rhythms), `Grid`, `Stack`,
`Row`, `Divider`
**Typography** — `Heading` (size and semantic level decoupled), `Text` (4 tones),
`Eyebrow`
**Primitives** — `Button`, `LinkButton`, `ArrowLink`, `Badge`, `StepMarker`,
`Variant`, `Size`
**Site** — `NavBar`, `Hero`, `FeatureCard`, `StepList`, `FactList`,
`PricingCard`, `Faq`, `Footer` (+ `NavItem`, `Step`, `Fact`, `QandA`,
`FooterColumn`)
**Form** — `TextField`, `TextArea`
**Feedback** — `Card`, `PageHeader`, `EmptyState`, `Skeleton`, `ErrorBanner`

Deliberate choices worth keeping:

- **`Faq` is native `<details>`/`<summary>`.** Correct expand/collapse
  semantics, keyboard operation and in-page find with zero JavaScript, so it
  works in the SSG build before hydration. A hand-rolled accordion needs state,
  an effect and `aria-expanded` wiring to reach the same place.
- **A call-to-action that navigates is `LinkButton` (an `<a>`), not `Button`.**
  It must be middle-clickable, openable in a new tab, and crawlable.
- **No router dependency.** Navigation components take plain `href` strings, so
  the kit stays renderer-agnostic; apps wrap with `Link` if they want
  client-side routing.
- **`apps/web`'s home page is a coverage fixture.** It renders every element the
  audit found, so a regressed component or removed prop breaks the build — the
  cheapest available test for a library whose output is otherwise only
  checkable by eye.

## Open

- **The `fullstack` feature is still in `apps/web`** even though the SSG seam
  (OQ-10) is unimplemented, so the app is effectively CSR and paying for
  fullstack anyway. Removing it would very likely beat 490,481 B, but it has to
  go back when SSG lands. Not run — the loop should not optimise the scalar by
  deleting a decision the architecture doc made.
- **The lockfile is unchanged at 533 crates.** The stripped dependencies were
  all still reachable transitively, so nothing left the graph. Direct-dependency
  hygiene improved; the graph did not shrink.
- **`wasm-opt` was never run.** `dx` may or may not apply it in release; not
  verified, and it is the most likely remaining single-digit-percent win.
- **Dark mode is absent**, matching the audited page. Adding it later means
  touching every colour class in the kit.
