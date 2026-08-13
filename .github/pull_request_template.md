## What and why

<!-- The diff says what changed. Say why it changed. -->

## Gate

- [ ] `cargo xtask ci` passes locally
- [ ] `cargo xtask live` passes, **or** this touches nothing that talks to Core

## If this touches any of these, say how

- [ ] **An event schema** — fields added only, `#[serde(default)]` on each, a new
      golden fixture for the new version, old fixtures still decoding
- [ ] **A folder** — still pure and total (no clock, no network, no `unwrap()` on
      payload shape, `apply` idempotent)
- [ ] **A dependency** — official upstream? which TLS does it default to? which
      layer does it land in?
- [ ] **A Tailwind class** — `cargo xtask styles` run and the result committed
- [ ] **A recorded decision** (`D*`, `R*`, `OQ-*`) — the architecture doc updated
      in the same PR

## Anything a reviewer would otherwise have to discover

<!-- Known gaps, deliberate omissions, things you tried that did not work. A
     "SEAM" comment in the code is better than a note here, but a note here is
     much better than nothing. -->
