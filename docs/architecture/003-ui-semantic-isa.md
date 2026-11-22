# UI semantic instruction set

**Status:** Accepted, and implemented across `rv2-ui`, `apps/app` and `apps/web`.
UI7 is enforced by `cargo xtask ci`. `apps/hiring`, which the evidence below
cites, was removed on 2026-10-20: it ships from `uk-tech-hiring-dashboard`.

This extends, without replacing, D7/D8 in
[`001-rust-v2-allsource-foundation.md`](001-rust-v2-allsource-foundation.md),
the token rules in [`../design-system/ledger.md`](../design-system/ledger.md),
and the hook contract in
[`../design-system/product-shell.md`](../design-system/product-shell.md).

An app does not style elements. It states intent in a closed vocabulary, and
`rv2-ui` is the only thing that turns intent into markup and classes. The
vocabulary has three parts, named as an instruction set because it behaves like
one: **registers** (tokens) hold values, **operands** (enums) select among
fixed choices, and **instructions** (components) are the only way to emit UI.
An app that writes a raw `button` with a hand-made class string is writing
inline assembly — sometimes necessary, never invisible.

## Why

When this was written the kit was small (≈2,700 lines) and still drifted,
because nothing said which spelling was the canonical one. Recorded as it was
found, so the rules below keep their reasons:

- Three colour sources. `@theme` tokens, `--rv2-shell-*` hard-coded to its own
  blues in `product-shell.css`, and `--rv2-command-*` hard-coded to a third
  palette in `command-select.css`. No app in this repo sets either hook, so a
  `Button` focus ring is `#0f172a`, the header's is `#2347c6`, and
  `CommandSelect`'s is `#315bc4`. In `apps/hiring`'s dark theme a
  `CommandSelect` would render as a cream box.
- `apps/app` and `apps/web` carry byte-identical `@theme` blocks, copied by hand.
- Kit composites restate atoms instead of using them: `Hero` re-types
  `Eyebrow` and display-`Heading` classes, `FeatureCard`/`PricingCard` are
  hand-rolled cards at `p-6`/`p-8` beside `Card`'s `p-4`, `Footer` re-types
  `Container`. Changing an atom does not change half its uses.
- Apps bypass the kit where it has gaps: the same outline-button string seven
  times across `web` and `hiring` (as buttons and as button-styled links); two
  buttons styled as text links; four raw inputs, two of which (`/motion`) copy
  the field style and drop its focus ring; ten raw anchors in
  `hiring/results.rs`; nine hand-rolled `rounded-lg border border-rule-soft
  bg-surface` surfaces at four different paddings; widths (`max-w-4xl`,
  `max-w-sm`, `max-w-[110rem]`) that `Width` cannot express.
- `class` cannot change colour. `Text`'s tone classes beat a passed class, so
  `apps/web` documents twice that it stopped using `Text` for exactly that
  reason, and `Badge { class: "bg-rust-strong …" }` depends on stylesheet order.

## Decisions

| # | Decision | Why |
|---|---|---|
| UI1 | A colour is named for its job, never its hue. | Already enforced by `no hardcoded hues`. Restated because every rule below assumes it. |
| UI2 | Registers have one source: `crates/rv2-ui/assets/theme.css`. Apps `@import` it and override **values only**. | A kit component that gains a token today must also be added to two hand-copied blocks, or it renders unstyled in one app. |
| UI3 | Hook variables (`--rv2-shell-*`, `--rv2-command-*`) stay, and every hook that has a register defaults to it, with the old hex as fallback: `--rv2-shell-ink: var(--color-ink, #172033)`. The shell's decoration — `accent`, `accent-soft`, `highlight` — has no register and keeps its hex. | Downstream bets theme through the hooks, so removing them breaks a published contract. Defaulting them to hex is what made one page carry three focus colours. The shell's blue is decoration, not focus, so mapping it to `accent` would have repainted the header for no rule's sake. |
| UI4 | Kit source uses canonical register names only. Registry aliases (`muted`, `ring`, `primary`…) stay defined, for borrowed code. | One concept, one spelling in the code we own. The aliases exist so copied shadcn components work unmodified; `tabs.rs` is ours and should not lean on them. |
| UI5 | Operands are enums. `class` carries layout only — margin, width, grid placement — never colour, type or radius. A caller needing a different colour has found a missing operand; add it. | A class that fights a component's own classes wins or loses by stylesheet order, which no test sees. |
| UI6 | Composites are built from instructions. A composite never restates an atom's classes. | So changing `Heading`, `Card` or `Container` changes every use of it. |
| UI7 | Apps emit no raw `button`, `input`, `select`, `textarea`, `h1`–`h4` or styled `a`, outside an instruction. An exception is marked `isa-raw-ok: <reason>`. Enforced by `cargo xtask ci`. | The gap list above is what happens without a rule. A visible exception is a design note; an invisible one is drift. |
| UI8 | An instruction exists when two call sites need it, and is deleted when no consumer does. External consumers count only when named in this doc. | Same ledger as the component-kit scope: unused code is bundle weight and upkeep with nothing on the other side. |
| UI9 | `rv2_ui::motion` is outside the instruction set. | It is a showcase justified by `/motion`. No app may depend on it to carry meaning. |

## Registers

| Group | Registers | Meaning |
|---|---|---|
| Surface | `ground`, `surface`, `raised` | Page, card, hover/selected fill |
| Ink | `ink`, `ink-hover`, `ink-muted`, `ink-faint` | Text emphasis, strong to weak |
| On ink | `on-ink`, `on-ink-muted`, `on-ink-faint` | Text on an ink-filled control or dark plate |
| Rule | `rule`, `rule-soft` | Control borders, structural dividers |
| Accent | `accent` | **Focus and selected controls only** (see ledger) |
| Verdict | `takeable` (with `-soft` fill), `caution`, `fault`, each with `-ink` | Health and judgement; never decoration |
| Type | `--font-sans`, `--font-display`, `--font-weight-heading` | Body and display families, and the heading weight; `display` defaults to `sans` at 600 |
| Product | e.g. `index-*`, `rust-ink`, `clears-*`, `takeable-wash` | App-local. Declared in the app, never used by the kit |

Shape and space are operands, not registers: radius is `rounded-md` for a
control, `rounded-lg` for a surface, `rounded-full` for a pill or dot, and
nothing else in kit source.

## Operands

| Operand | Values | Used by |
|---|---|---|
| `Variant` | `Primary`, `Secondary`, `Outline`¹, `Danger`, `Ghost`, `Link`¹ | `Button`, `LinkButton` |
| `Size` | `Small`¹, `Default`, `Large` | `Button`, `LinkButton` |
| `HeadingSize` | `Display`, `Section`, `Title`¹ (one entry's name), `Card` — the first three in `--font-display` | `Heading` |
| `Tone` | `Default`, `Muted`, `Small`, `Lead`, `OnInk`¹ | `Text` |
| `StatusTone` | `Ok`, `Warn`, `Bad`, `Muted` | `Status` |
| `Width` | `Narrow`¹ (`sm`), `Prose` (`2xl`), `Default` (`5xl`), `Wide` (`7xl`), `Dense`¹ (`110rem`, data-dense screens) | `Container` |
| `Space` | `Tight`, `Default`, `Loose` | `Section` |
| `Pad` | `None`, `Default` (`p-4`), `Roomy`¹ (`p-6`) | `Card` |
| `Surface` | `Flat`, `Slab` (2px ink border, hard offset shadow) | `Card` |
| `BadgeFill` | `Default`, `Takeable`¹, `Strong`¹, `Quiet`¹, `Tint` (a count in its parent's colour) | `Badge` (beside `colour`, for category hues) |
| `Press` | `Primary`, `Soft`, `Offset` | `PressButton`, `PressLink` |
| `Lean` | `Neutral`, `Good`, `Close`, `Bad` | `Stat`, `ResultPanel`, `Meter`, `LedgerCell` |
| `Density` | `Compact`, `Hero` | `StatStrip` |
| `Verdict` | `Note`, `Clear`, `Caution`, `Stop` | `Callout`, `Receipt`, `Stamp`, `Gate`, `Toast` |
| `DialogKind` | `Centered`, `Drawer` | `Dialog` |
| `RailLayout` | `Row`, `Column` | `ProcessRail` |
| `Arrive` | `Static`, `Grow` (grows from its baseline on scroll) | `Bar`, `SplitBar` |
| `FactWeight` | `Item`, `Total`, `Chosen` (a set figure standing in for the computed one) | `Fact` |
| `SegmentFlow` | `Row`, `StackNarrow` | `SegmentedControl` |
| `PopoverAlign` / `PopoverRole` | `Start`, `End` / `Content`, `Note` | `Popover` |
| `ClockDirection` | `Up`, `Down` | `TimerReadout` |
| `Boundary` | `OnDevice`, `RemoteWithConsent` | `BoundaryPill`, `GroundingNote` |
| `ColumnKind` / `RowState` | `Text`, `Figure` / `Normal`, `Voided`, `Milestone` | `LedgerTable` |
| `Dated` | `Captured`, `Checked`, `Published` | `Citation` |
| `ReasonStyle` | `Prose`, `Evidence` | `AttentionQueue` |

¹ Added by this decision; each closes a gap with existing raw call sites.

`Tabs` lost its `TabsVariant` operand: the underline variant had no consumer
(UI8). `Variant::Link` keeps the size's type scale and drops its padding, so it
sits on the baseline of the sentence it is in. `Card` is flat: every
hand-made surface it replaced was, and the hiring ledger spends its one shadow
on a verdict. `Surface::Slab` is the exception the printed-slab bets share: a
hard offset shadow paired with `Press::Offset`, chosen per card, never a
default.

## Instructions

| Intent | Instruction | Emits | Replaces in apps |
|---|---|---|---|
| Do something | `Button` | `button` | 6 hand-styled buttons |
| Go somewhere, as a CTA | `LinkButton` | `a` | 3 button-styled anchors |
| Go somewhere, inline | `TextLink`¹, `ArrowLink` | `a` | underlined raw anchors in `hiring` |
| Enter a value | `TextField`, `TextArea`, `Select`, `Input`¹ | `label` + control | 4 raw inputs |
| Pick from many | `CommandSelect` | button + combobox | — (named external consumer: A Place Ahead) |
| Toggle a facet | `Checkbox`, `FieldSet` | native controls | — |
| Contain | `Card`, `Section`, `Container`, `Grid`, `Divider` | `div`/`section`/`hr` | 9 hand-rolled surfaces, raw `max-w-*` wrappers |
| Reveal | `Disclosure`, `Faq` | `details` | — (the hiring filter rail keeps its own `details` inside a `Card`: its summary holds a heading, a count and an action, which `Disclosure`'s text summary cannot) |
| Say | `Heading`, `Text`, `Eyebrow` | `h1`–`h4`, `p` | raw headings in `hiring`, `app` |
| Title a page, with its actions | `PageHeader` | `header` with `h1`, subtitle and an action slot | — |
| Label a custom control | `FieldShell` | `label` + help and error lines the caller binds by id | — |
| Number a step | `StepMarker` | `span`, zero-padded | — (inside `FeatureCard` and `StepList`) |
| Enter a page | `PageEntrance` | `div` with a finite CSS entrance, none under reduced motion | — |
| Ask for consent | `ConsentFooter` | the shared consent stylesheet and script; a product adapter mounts it ([`../design-system/consent-footer.md`](../design-system/consent-footer.md)) | — |
| Signal | `Badge`, `Status`, `Swatch`, `Bar` | `span`/`div` | — |
| Report state | `EmptyState`, `Skeleton`, `ErrorBanner` | — | — |
| Navigate | `ProductHeader`, `NavigationRail`, `ProcessRail`, `Tabs`, `Footer` | `header`/`nav` | — |
| Market | `Hero`, `FeatureCard`, `StepList`, `FactList`, `PricingCard` | composites (UI6) | — |
| Do or go, with weight | `PressButton`, `PressLink` | `button`, `a` on a ledge | Ready Play Days: `action-button` overrides on `Button` (UI5) |
| Keep for later | `SaveToggle` | `button[aria-pressed]` | Ready Play Days: `save-button` + `save-gem` |
| Narrow a list | `ChipGroup`, `Chip` | `div[role=group]` + `button[aria-pressed]` | Ready Play Days: `.filters button` |
| Browse | `MediaCard` | `article` | Ready Play Days: `.idea` cards |
| Identify an editorial site | `Masthead` | `header` | Ready Play Days: `.masthead` |
| Decorate | `Sparkle` | `span[aria-hidden]` | Ready Play Days: `.paper-accent` |
| Report a figure | `StatStrip`, `ResultPanel`, `Meter`, `SplitBar` | `dl`, `section[aria-live]`, `div[role=meter]`, `figure` | hand-drawn KPI bands and calculator results in 8 bets |
| Judge | `Callout`, `Receipt` | `aside`, `article` | verdict strips and receipts in 5 bets |
| Enter a measure | `UnitField` | `label` + affixed input | affix inputs in 5 bets |
| Narrow one view | `SegmentedControl` | `div[role=group]` + `button[aria-pressed]` | view and billing toggles in 3 bets |
| Choose one, with consequences | `ChoiceGroup`, `Choice` | `fieldset` + `label` + `input[type=radio]` | radio cards in 3 bets |
| Ask before acting | `Dialog`, `ConfirmDialog` | `dialog` opened with `showModal()` | jbt-fit's `Dialog`, `ConfirmDialog` |
| Delete without a modal | `ArmedDeleteButton` | `button[data-armed]` | jbt-fit's `TwoStateDeleteButton` |
| Tell after acting | `ToastProvider`, `Toaster`, `use_toast` | `div[role=region]` of `role=status`/`alert` | jbt-fit's toast queue |
| Mark a record's state | `Stamp` | `span` | outline status tags in 2 bets |
| Check readiness | `GateList`, `Gate` | `ul` with spoken state | gate and asset rows in 3 bets |
| Show an edit | `RedlineDiff`, `Redline` | `dl` of `del`/`ins` | Edit Scope's draft redline |
| Compare options | `CompareTable`, `CompareRow` | `table`, stacked cards below `sm` | comparison tables in 3 bets |
| Count a figure in | `Tally` | text number + `aria-hidden` CSS counter | Jobbreakdown's `Tally` |
| Show a trend | `Sparkline` | `svg` polyline, decorative unless named | jbt-fit's metric sparkline |
| Keep a ledger | `LedgerTable`, `LedgerRow`, `LedgerCell` | `table` in a focusable scroll region, total in `tfoot` | record tables in 4 bets |
| Cite a source | `Citation` | `p` with `time[datetime]` | dated source lines in 3 bets |
| Queue work for someone | `AttentionQueue`, `QueueItem` | `ol` of links or stated rows | jbt-fit's review queue |
| Offer more | `Popover`, `RowMenu`, `HelpTip` | `button[popovertarget]` + `div[popover=auto]` | jbt-fit's facet panels and sort menu, Edit Scope's scope-change tip |
| Run against a plan | `TimerReadout`, `SectionTimer` | `role=timer` + `role=status` pace word | Talk Run Card's rehearsal runner, jbt-fit's rest countdown |
| Say where an assistant runs | `BoundaryPill`, `GroundingNote` | `span`, `p` | Edit Scope's and Logo Handoff's local composers |

The tactile rows (`rv2_ui::tactile`) have a named external consumer, Ready Play
Days, whose `apps/web` wrote every one of them as class overrides. Its look
lives in `assets/tactile.css` behind `--rv2-tactile-*` hooks that fall back to
registers (UI3), read at each use rather than declared, so a class-scoped theme
such as `assets/themes/playful.css` reaches them. `/kit` on `apps/web` mounts
all of them; `apps/app` excludes `tactile.rs` from its Tailwind sources.
The readout, judge, measure and segment rows each have three or more consumers
across the product bets, listed with sources in
[`../ledger/cross-bet-component-harvest.md`](../ledger/cross-bet-component-harvest.md);
`/kit` mounts them too.

The ask, delete and tell rows (`rv2_ui::overlay`, plus `ArmedDeleteButton`)
have a named external consumer, jbt-fit, whose `jbt-ui` they were drawn from.
`Dialog` is the kit's first use of `document::eval`: `showModal()` has no
attribute form, and the modal state is what gives the backdrop, `inert` on the
page behind and Escape. A toast's lifetime is a CSS animation in
`assets/overlay.css` and its `animationend` dismisses it, so the kit still has
no timer; `Caution` and `Stop` toasts have none and wait to be dismissed. `apps/app` excludes `overlay.rs` from its Tailwind sources until it
mounts one. `RedlineDiff` has one consumer, Edit Scope, named here as its
external consumer (UI8).

The catalogue rows from `Tally` down came out of shaping the harvest's sand
and water tiers. Named external consumers (UI8): Jobbreakdown for `Tally` and
`Arrive`; jbt-fit for `Sparkline`, `AttentionQueue`, `Popover` and
`TimerReadout`; Talk Run Card for `SectionTimer`; Edit Scope for `HelpTip`.
`LedgerTable`, `Citation`, `BoundaryPill` and `GroundingNote` have two or more
Dioxus consumers each.

`Tally` and `Arrive::Grow` move, and UI9 keeps motion out of the ISA. They sit
in it anyway because neither carries meaning by moving: the number is text in
the DOM, and the end state is the static state, so a browser without scroll
timelines, a printout and a reader with reduced motion all get the figure.
`assets/arrive.css` holds both and is imported by both apps. Without it
`Tally` shows its number as plain text and nothing counts.

`Popover` is built on the native `popover` attribute, which supplies light
dismiss, Escape, one-open-at-a-time and the top layer with no script. CSS
anchor positioning places it under its trigger where supported; elsewhere the
browser centres it. `RowMenu` is the second use of `document::eval`: a native
popover stays open on a click inside it, so choosing an action calls
`hidePopover()`. `Disclosure` stays the instruction for inline expansion.
The timers render props only: each app owns its clock, so the kit still has no
timer.

`NavItem`, `ProductNavItem` and `ProductHeaderAction` carried the same
`label + href + external` record three times. They are one `NavLink`:
`NavLink::route(key, label, href)` for something that can be the current page,
`NavLink::new(label, href)` for anything else, `.external()` for either.

`TextLink` also exports its classes as `TEXT_LINK` and `TEXT_LINK_STRONG`,
because the kit has no router (see `lib.rs`): an app's router `Link` takes the
class rather than re-typing it.

`Row` and `Stack` had no consumer and are deleted (UI8).

The A2UI catalog in [`004-a2ui-surface.md`](004-a2ui-surface.md) publishes a
subset of these instructions to agents. `rv2-a2ui-dioxus` is a consumer of
every instruction in that subset, and a catalog entry keeps its instruction
alive under UI8 for as long as `rv2:a2ui/catalog/v1` is served. Its `Column`
is a layout `div` in the renderer, not a kit instruction.

## Exceptions

A raw element is legitimate when it is the **subject** of the page rather than
its chrome — a skip link whose whole job is to be the first focusable thing,
an element whose markup is the thing being demonstrated — and the
`isa-raw-ok` marker says which, on the element's line or in the comment block
directly above it. "The instruction did not support my case" is not an
exception; it is an operand to add.

The check (`apps speak the kit`, `tooling/isa/src/isa.rs`) scans
`apps/*/src`. Repositories built on the kit run the same scan with
`rv2-isa check`; each finding names the instructions that replace the raw
element. It does not scan `examples/`, which preview the kit's own markup,
or the kit itself, whose job is emitting markup. At the time of writing no
exception exists; the first one was `apps/hiring`'s skip link.

## Not decided here

- **The components only `apps/hiring` used.** With it removed, nothing in this
  repo renders `Badge`/`BadgeFill`, `Bar`, `Disclosure`, `Status`, `Swatch`,
  `TextLink`/`TEXT_LINK_STRONG`, `Checkbox`, `FieldSet`, `Select`, `Tabs`, or
  the operands `Width::Dense`, `Variant::Link` and `Pad::None`. UI8 says delete
  them; they are kept for now only because a product bet may be the consumer,
  and UI8 counts an external consumer only once it is named here.

- Whether `motion` keeps earning its weight. `lib.rs` already says delete the
  module and `/motion` together when it stops.
- Dark mode for `apps/app`/`apps/web`. UI2 makes it a values-only change; it
  needs a product reason first.
- A brand register. The shell's decoration stays its own blue (UI3); if a
  product wants it to follow a brand colour, that is a new register with a job
  of its own, not a reuse of `accent`.
