# Product-shell extraction review

Date: 2026-09-09

## Outcome

`rv2-ui` now owns four router-neutral product-shell patterns:

- `ProductHeader` for stable identity, navigation, active-route state, mobile disclosure, and a primary action;
- `NavigationRail` for compact page or policy indexes;
- `ProcessRail` for static or active multi-step progress;
- `PageEntrance` for a finite, reduced-motion-safe route entrance.

Product routes, names, artwork, copy, legal text, and commercial rules remain product-owned.

## Checks

| Gate | Result |
| --- | --- |
| `cargo test -p rv2-ui` | 13 passed |
| `cargo xtask ci` | passed: format, clippy, build, tests, CSS freshness, predecessor scan, WASM boundary |
| Desktop browser, 1280 × 720 | no horizontal overflow; one H1; 3 section anchors; 4 process steps |
| Mobile browser, 320 × 800 | no horizontal overflow; 44px action; 45.78px navigation targets |
| Reduced-motion CSS | entrance animation disabled under `prefers-reduced-motion: reduce` |
| Browser app adoption | compiles for native and `wasm32-unknown-unknown` through workspace CI |

Browser inspection caught and fixed two issues before release: invisible mobile-action text and an empty `ProcessRail` DOM caused by nested RSX rendering.

## Boundary

Visual browser evidence exercises shared components through `apps/web`. Authenticated `apps/app` shell adoption is compile- and WASM-verified; no synthetic session was created to bypass its real authentication boundary.
