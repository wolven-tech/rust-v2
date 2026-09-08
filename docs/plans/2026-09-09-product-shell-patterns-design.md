# Reusable product-shell patterns

## Decision

Add an additive `rv2_ui::product_shell` module. Existing `rv2-ui` is already
the renderer-neutral, WASM-safe design-system crate; creating another crate
would split ownership and force every starter app to choose between two kits.

Alternatives rejected:

1. Expanding `site::NavBar` would preserve one name but turn a small marketing
   component into a prop-heavy desktop, mobile, route-state, app-status hybrid.
2. Shipping CSS classes without components would reuse appearance while every
   product reimplemented landmarks, active state, disclosure, and progress
   semantics.

## Component contract

- `ProductHeader`: caller-owned brand mark/name, typed route items, current
  route key, optional typed CTA, optional trailing status. It renders one
  stable sticky header, desktop navigation, and native mobile `<details>` menu.
- `NavigationRail`: compact route, policy, or article index using same typed
  items and `aria-current` contract.
- `ProcessRail`: ordered stages with optional anchors and one-based active step.
  Complete and current states remain semantic; static process diagrams may omit
  active state.
- `PageEntrance`: opt-in wrapper for one finite page entrance. No render loop,
  timer, animation dependency, or mandatory transition.

Product names, logos, copy, legal identity, business rules, route enums, and
illustrations stay in product repositories. Logo Handoff’s folder art and
context-specific glyphs are not general design-system primitives.

## Styling

Add `assets/product-shell.css`, imported once by each starter stylesheet.
Semantic custom properties provide paper, ink, rule, accent, focus, width, and
shadow hooks with accessible defaults. Components expose stable `rv2-*` class
names; callers theme at product-shell boundary instead of copying component
CSS. Mobile targets remain at least 44 CSS pixels. Motion uses opacity and
transform, runs once, and disappears under `prefers-reduced-motion: reduce`.

No global preloader, cursor effect, horizontal-scroll navigation, WebGL, or
cross-document transition is part of default contract.

## Starter migration and verification

Marketing and app shells consume `ProductHeader`, proving both renderer paths.
About consumes `ProcessRail`, `NavigationRail`, and anchored editorial
sections. `ProductHeader` replaces unused internal `NavBar`; `StepList` remains
in its existing process-list coverage.

Tests cover current-route and active-step class decisions. `cargo xtask styles`
must regenerate committed CSS; `cargo xtask ci` must pass native and WASM
boundaries. Exact browser checks cover desktop header, 320px mobile disclosure,
route state, progress semantics, focus, reduced motion, and overflow.

## Sources

- `crates/rv2-ui/src/site.rs`
- `crates/rv2-ui/assets/motion.css`
- `apps/web/src/main.rs`
- `apps/app/src/views.rs`
- Logo Handoff Card UK editorial UI release and autoresearch ledger
