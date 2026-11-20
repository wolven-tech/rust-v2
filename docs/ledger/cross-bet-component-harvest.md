# Cross-bet component harvest

**Date:** 2026-11-20. **Scope:** every repository under `founder-mode/` that
ships a UI, read for components worth moving into `rv2-ui`.

Ready Play Days was the starting point: its `apps/web` had grown a tactile
button language — controls resting on a hard ledge, a clay star that tilts on
hover and focus, a save toggle whose gem pulses — written as `class` overrides
on `Button`, which UI5 forbids. The rest of the portfolio was then surveyed
for the same thing: shapes drawn by hand in more than one bet.

## Landed in this change

| Instruction | Module | Drawn from | Consumers seen |
|---|---|---|---|
| `PressButton`, `PressLink`, `Press::{Primary, Soft, Offset}` | `tactile` | Ready Play Days `.action-button`; offset from Logo Handoff, Talk Run Card, Sponsor Ledger, Solar Quote Check | 5 |
| `SaveToggle` | `tactile` | Ready Play Days `.save-button` + `.save-gem` | 1, plus the saved-list shape in Saved Ledger |
| `ChipGroup`, `Chip` | `tactile` | Ready Play Days `.filters`; Jobbreakdown filter chips | 2 |
| `MediaCard` | `tactile` | Ready Play Days `.idea` | 1 |
| `Masthead` | `tactile` | Ready Play Days `.masthead` | 1 |
| `Sparkle` | `tactile` | Ready Play Days `.paper-accent` | 1 |
| `StatStrip`, `Stat`, `Lean`, `Density` | `readout` | A Place Ahead overview strip, control-room signal rail, Move to Own position strip, Solar Quote metric grid, Viewing Ledger summary, Sponsor Ledger numbers, Talk Run budget, Worthwake metrics | 8 |
| `ResultPanel` | `readout` | Chargewindow result, Viewing Ledger primary result, Solar Quote results head | 3 |
| `Meter` (thresholds, ticks) | `readout` | All Source usage meter, Reefdose portion rail, Move to Own question progress, Sponsor Ledger slot strip | 4 |
| `SplitBar`, `Part` | `readout` | Chargewindow energy/fees bar, Edit Scope cost stack | 2 |
| `Callout`, `Verdict` | `feedback` | Move to Own applicable/stopped, Edit Scope boundary strip, A Place Ahead offer note, Logo Handoff context boundary, Viewing Ledger readiness note | 5 |
| `Receipt` | `feedback` | Edit Scope draft and completion receipts, Logo Handoff draft review, Talk Run Card summary | 3 |
| `UnitField` | `form` | Chargewindow, Solar Quote, Viewing Ledger, Reefdose, Edit Scope affix inputs | 5 |
| `SegmentedControl` | `form` | A Place Ahead view and scope toggles, All Source billing toggle, Jobbreakdown lens switch | 3 |

The catalogue's rock tier landed in the same change:

| Instruction | Module | Drawn from | Consumers seen |
|---|---|---|---|
| `Dialog`, `ConfirmDialog`, `DialogKind::{Centered, Drawer}` | `overlay` | jbt-fit `crates/jbt-ui/src/dialog.rs:14,91` | 1 (named external) |
| `ToastProvider`, `Toaster`, `use_toast`, `ToastQueue` | `overlay` | jbt-fit `crates/jbt-ui/src/toast.rs` | 1 (named external) |
| `ArmedDeleteButton`, `DeleteClick` | `primitives` | jbt-fit `dialog.rs:114-154` | 1 (named external) |
| `BadgeFill::Tint` (count chip) | `primitives` | A Place Ahead `workboard-components.css:14` | 1, plus every tab and heading count |
| `Stamp` | `feedback` | Logo Handoff `.status`, Sponsor Ledger verdict stamp | 2 |
| `Surface::Slab` on `Card` | `feedback` | Logo Handoff, Talk Run Card, Sponsor Ledger, Solar Quote | 4 |
| `GateList`, `Gate` | `readout` | control room gate rows, Logo Handoff asset rows, Reefdose verification grid | 3 |
| `RedlineDiff`, `Redline` | `readout` | Edit Scope `apps/app/assets/input.css:669-675` | 1 (named external) |
| `CompareTable`, `CompareRow` | `readout` | A Place Ahead `comparison.css`, All Source `ComparisonTable.tsx`, Move to Own responsive table | 3 |
| `ChoiceGroup`, `Choice` | `form` | Move to Own `.action-option:has(input:checked)`, Logo Handoff context choice, Reefdose parameter grid | 3 |
| `RailLayout::{Row, Column}` on `ProcessRail` | `product_shell` | Edit Scope timeline, Reefdose stepper | 2 |

Decisions made while building them:

- **`Dialog` uses `document::eval`.** `showModal()` has no attribute form, and
  only the modal state gives the backdrop, `inert` on the page behind and
  Escape. `eval` ships in the `lib` feature the kit already takes, so no
  `web-sys` dependency was added. The drop path evaluates only after an effect
  has run, so a server render never calls it.
- **Focus.** jbt-fit cycled Tab inside the dialog by script. The kit relies on
  the native modal's `inert` page instead, and first focus goes to a child with
  `autofocus` (`ConfirmDialog` puts it on the cancel button). Tab can still
  leave to the browser chrome, which jbt-fit's cycle prevented.
- **No timer for toasts.** jbt-fit spawned a cancellable delay per toast. Here
  the lifetime is a CSS animation (`rv2-toast-life`, length per verdict) that
  `:hover` and `:focus-within` pause, and its `animationend` dismisses. It runs
  under reduced motion too, because it animates nothing visible. Only `Clear`
  and `Note` toasts time out: a `Caution` or `Stop` stays until dismissed,
  because a timeout on something the reader must act on fails WCAG 2.2.1.
- **Toast level is `Verdict`.** jbt-fit's `Success / Info / Warning / Error`
  map onto `Clear / Note / Caution / Stop`, so a toast and a callout for the
  same outcome share one colour.
- **`Count` became `BadgeFill::Tint`** and **`Stamp` its own instruction**: a
  count is a pill like any badge, a stamp is square and in capitals, and
  folding it into `BadgeFill` would have given one operand two shapes.
- **ProcessRail's `:target` variant was not built.** The step that matches a
  `:target` section is a different element from the link in the rail, so the
  highlight needs a `:has(#id:target)` selector per step id, generated per
  page. `active_step` already carries the current step.

Ready Play Days, jbt-fit and Edit Scope components with a single consumer
count under UI8 because `003-ui-semantic-isa.md` names each. `/kit` on
`apps/web` mounts every instruction in this section;
`crates/rv2-ui/tests/tactile.rs`, `tests/readout.rs` and `tests/records.rs`
pin the accessibility contracts (aria-pressed, role=meter with value text,
aria-live results, one tab stop per card, hidden affixes, dialog naming, first
focus on the safe action, spoken gate state, `del`/`ins`, a column name on
every stacked compare cell, native radios).

Theme: `assets/tactile.css` reads `--rv2-tactile-*` hooks with register
fallbacks at each use; `assets/themes/playful.css` is Ready Play Days' palette
as a class-scoped theme (`.rv2-theme-playful`), usable on `body` or on one
region.

The clay icons (`action-star-icon.png`, `save-heart-icon.png`) are copied into
`apps/web/assets` for the gallery only. The kit takes an icon URL and ships no
artwork.

## The catalogue's sand and water tiers

Confidence uses rock / sand / water (see the estimation rule): rock is shaped
and ready to build, sand needs its shape settled first, water needs a spike.
Each sand item was shaped and each water item spiked against its sources. Only
Dioxus bets count as consumers: chief-of-staff renders HTML strings, Reefdose
and Move to Own are React, and the control room is static JavaScript, so their
patterns informed shapes but cannot mount a kit instruction.

### Built

| Instruction | Module | Was | Consumers |
|---|---|---|---|
| `Tally` | `arrive` | water: scroll-driven figures | Jobbreakdown (named) |
| `Arrive::{Static, Grow}` on `Bar`, `SplitBar` | `arrive` | water: `GrowBar`, share segments | Jobbreakdown (named) |
| `LedgerTable`, `LedgerRow`, `LedgerCell` | `ledger` | sand: `LedgerTable` | Worthwake, Edit Scope web, A Place Ahead calculator, Talk Run Card web |
| `Citation`, `Dated` | `ledger` | sand: `SourceCard` | Sponsor Ledger web, A Place Ahead desk and history, Jobbreakdown newsletters |
| `AttentionQueue`, `QueueItem` | `queue` | sand | jbt-fit coach dashboard (named) |
| `Sparkline`, `spark_points` | `sparkline` | sand: `Sparkline` | jbt-fit metric cards, 3 sites (named) |
| `Popover`, `RowMenu`, `HelpTip` | `overlay` | sand: `SourceChip`/`RowMenu`, `HelpTip` | jbt-fit facet panels and sort menu (named), Edit Scope tip (named) |
| `TimerReadout`, `SectionTimer`, `pace`, `clock` | `timer` | sand: `SectionTimer` | Talk Run Card runner (named), jbt-fit rest countdown |
| `BoundaryPill`, `GroundingNote` | `feedback` | sand: assistant panel atoms | Edit Scope, Logo Handoff |
| `FactWeight::{Item, Total, Chosen}` on `Fact` | `site` | sand: `EquationRail`, running-sum form | Sponsor Ledger web, Edit Scope web and app |
| `SegmentFlow::{Row, StackNarrow}` on `SegmentedControl` | `form` | sand: assistant tone switch | Edit Scope, Logo Handoff |

Decisions made while shaping them:

- **Ranges end at `entry 100%`.** Scroll progress follows position, so a range
  into `cover` leaves a figure short for a reader who stops above it;
  Jobbreakdown measured 147 under a denominator of 174. A staggered bar's
  range is clamped with `min(100%, …)` however many parts it has. A test pins
  both.
- **The static state is the end state.** `@property --rv2-tally` starts at the
  destination, and the animations sit inside `@supports (animation-timeline:
  view())` and `prefers-reduced-motion: no-preference`.
- **Popovers are native.** `popover="auto"` gives light dismiss, Escape and
  the top layer, replacing jbt-fit's `web-sys` outside-pointer task. `RowMenu`
  is a list of buttons, not `role="menu"`, which would promise arrow-key
  roving. `HelpTip` is a button and a `role=note` panel, not
  `aria-describedby`, so touch readers can open it.
- **Timers render props.** The app owns the clock. Digits are `role=timer`
  (not announced); the pace word is a separate `role=status` that changes only
  with the band. Talk Run Card's source put `aria-live` on the seconds, which
  announces every tick.
- **`Boundary` is kit-local.** Apps map `rv2_ai::DataBoundary` onto it; Edit
  Scope has no `rv2-ai`, and the kit does not depend on it.
- **Ledger voids are said in words.** Every cell of a voided row starts with
  sr-only "Voided:", because a reader moving cell by cell may not hear the row
  header; the cells after the header are `del` and the row is muted ink.
  Worthwake's read-only view said it by opacity alone, which also drops
  coloured figures below text contrast.
- **`Tally` degrades to text.** The number is a plain span that
  `assets/arrive.css` hides visually when it draws the counter, so a product
  that never imports the stylesheet still shows the figure.

### Still sand

| Candidate | What is missing |
|---|---|
| `EquationRail`, horizontal operator cells | A Dioxus consumer; Reefdose, its only source, is React. The running-sum form landed as `FactWeight`. |
| `TimeRibbon` | A Dioxus consumer with day rows on an hour axis, and a decision on a shared positioned-`Track` atom first. |
| `ActionCard` | A second consumer with a typed verb; chief-of-staff's Decide/Do/Delegate/Batch is its own. `Receipt` covers the shape meanwhile. |
| `CueDeck` | A second bet; both call sites are Talk Run Card. |
| `SourceChip` | A Dioxus consumer; chief-of-staff cannot mount one. `Popover` is ready for it. |
| `MiniBars` | A Dioxus consumer; Reefdose is React and `Bar` covers labelled compares. |
| Undo bar, assistant panel composite | A second undo consumer, and one review-state type both products share, which would live in `rv2-ai`, not the kit. |
| `ScrubRail` | A second consumer, a browser check of a drag shield against `eval` pointer capture (Dioxus 0.7 has no capture API), and a tick-state operand that is not fitness-specific. |
| Arrival shadow | A second consumer, its hex gold as a hook, and `@starting-style` checked across engines. |

### Rejected

| Candidate | Why |
|---|---|
| `SignalBars` | Already `Bar`. |
| `IsoColumnChart`, `DonutScene` | Product art plus product filter state, one consumer, and data `Bar` and `SplitBar` already show with text. The kit's readouts are bars with figures; axes and SVG geometry are a charting library. |
| `Radar` sweep, `RouteBeam` | Decoration on an infinite loop, one non-Dioxus consumer each, product palette, no information. |

### Skipped

Template copies with no bespoke UI: conversion-feasibility, rust-v2-affect,
family-home-decision, family-travel-flip, property-offer-evidence,
renovation-investment, close-evidence-desk-uk, worthwake `apps/web`,
move-to-own-electrical-seo (fork of Move to Own). `crux` (upstream examples on
Bulma) and `performance-coach` (empty web template) have nothing to take.
Product art — Reefdose tank and vial, Logo Handoff folder, A Place Ahead map
sprites, Worthwake ledger art — stays with its product.

## Limits

- The survey was three read-only passes over source; candidates outside the
  rock tier were not read line by line, and file:line pointers come from those
  passes.
- `/kit` is checked by compiling and by SSR tests. Hover, press, pulse, the
  dialog's modal behaviour, a toast's timed dismissal, popover placement and
  scroll-driven arrival happen in the browser and are only verified by opening
  the page.
- Anchor positioning, `animation-timeline: view()` and `@property` support
  across engines is not established here; every use sits behind `@supports` or
  falls back to the end state.
- The bets still render their own versions. Moving each onto these
  instructions is a change in that bet's repository, tracked by an issue
  there.
