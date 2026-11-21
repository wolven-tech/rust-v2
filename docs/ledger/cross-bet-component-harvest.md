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
  under reduced motion too, because it animates nothing visible.
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

## Not landed: the catalogue

Confidence uses rock / sand / water (see the estimation rule): rock is shaped
and ready to build, sand needs its shape settled first, water needs a spike.

The rock tier is built (above). What remains:

### Sand — useful, shape not settled

| Candidate | Sources | What settles it |
|---|---|---|
| `EquationRail` (a × b ÷ c = total) | Reefdose `reefdose.css:1875-1939`, Sponsor Ledger arithmetic line | One API for both operator cells and running sums. |
| `TimeRibbon` (day blocks, now line) | chief-of-staff `report.html:170-183` | A second consumer. |
| `ActionCard` with verb operand | chief-of-staff `report.html:111-141` | Whether the verb set is general. |
| `SectionTimer` + `CueDeck` | Talk Run Card `views.rs:1082-1287`, `:1600-1641` | Needs a ticking clock: an interval in the app, not the kit. |
| `SourceChip` / `RowMenu` (`details` popovers) | chief-of-staff `report.html:201-223`, jbt-fit facet panel | A shared CSS-only popover atom first. |
| `HelpTip` (anchor-positioned popover) | Edit Scope `input.css:409-452` | Browser support for CSS anchor positioning. |
| `LedgerTable` (tabular money, voided rows) | Worthwake `worthwake.rs:636-760` | Overlap with `CompareTable`. |
| `SourceCard` (citation with captured date) | Sponsor Ledger web `main.rs:857-869` | A second consumer. |
| `AttentionQueue` | jbt-fit `crates/jbt-ui/src/states.rs:96-124` | Generic enough beyond fitness. |
| `MiniBars` sparkline, `Sparkline` | Reefdose `reefdose.css:1033-1081`, jbt-fit `metrics.rs` | One primitive or two. |
| Local assistant panel (tone switch, privacy pill, undo bar) | Edit Scope `views.rs:1147-1491`, Logo Handoff `lib.rs:516-534` | Split into atoms; ties to `rv2-ai`. |

### Water — spike first

| Candidate | Sources | Unknown |
|---|---|---|
| Scroll-driven figures (`TallyRoll`, `GrowBar`, arrival shadow) | Jobbreakdown `apps/web/assets/input.css:280-406` | `animation-timeline: view()` support; belongs in `motion` (UI9). |
| `IsoColumnChart`, `DonutScene`, `SignalBars` | Jobbreakdown `observatory.css:169-235` | Whether the kit should own charts at all. |
| `ScrubRail` (day scrubber with pointer capture) | jbt-fit `calendar.rs:80-150` | Pointer capture without `web-sys`. |
| `Radar` sweep, `RouteBeam` | control room `styles.css:335-389`, Move to Own `globals.css:492-525` | Decoration; `motion` candidates. |

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
  dialog's modal behaviour and a toast's timed dismissal happen in the browser
  and are only verified by opening the page.
- Ready Play Days itself still renders its own CSS. Moving it onto these
  instructions is a change in that repository.
