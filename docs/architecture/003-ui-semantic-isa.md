# UI semantic instruction set

**Status:** Proposed. Work is tracked in beads: epic `rv2-pql`, label `ui-isa` (`bd ready -l ui-isa`).

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

The kit is small (≈2,700 lines) and still drifts, because nothing says which
spelling is the canonical one:

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
| UI3 | Hook variables (`--rv2-shell-*`, `--rv2-command-*`) stay, and default to registers: `--rv2-shell-ink: var(--color-ink)`. | Downstream bets theme through the hooks, so removing them breaks a published contract. Defaulting them to hex is what makes one page carry three accents. |
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
| Verdict | `takeable`, `caution`, `fault`, each with `-ink` | Health and judgement; never decoration |
| Type | `--font-sans`, `--font-display` | Body and display families; `display` defaults to `sans` |
| Product | e.g. `index-*`, `rust-*`, `clears-*` | App-local. Declared in the app, never used by the kit |

Shape and space are operands, not registers: radius is `rounded-md` for a
control, `rounded-lg` for a surface, `rounded-full` for a pill or dot, and
nothing else in kit source.

## Operands

| Operand | Values | Used by |
|---|---|---|
| `Variant` | `Primary`, `Secondary`, `Outline`¹, `Danger`, `Ghost`, `Link`¹ | `Button`, `LinkButton` |
| `Size` | `Small`¹, `Default`, `Large` | `Button`, `LinkButton` |
| `HeadingSize` | `Display`, `Section`, `Card` | `Heading` |
| `Tone` | `Default`, `Muted`, `Small`, `Lead`, `OnInk`¹ | `Text` |
| `StatusTone` | `Ok`, `Warn`, `Bad`, `Muted` | `Status` |
| `Width` | `Narrow`¹ (`sm`), `Prose` (`2xl`), `Default` (`5xl`), `Wide` (`7xl`), `Dense`¹ (`110rem`, data-dense screens) | `Container` |
| `Space` | `Tight`, `Default`, `Loose` | `Section` |
| `TabsVariant` | `Pill` (`Line` has no consumer; UI8) | `Tabs` |

¹ Added by this decision; each closes a gap with existing raw call sites.

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
| Reveal | `Disclosure`, `Faq` | `details` | 1 hand-rolled `details` |
| Say | `Heading`, `Text`, `Eyebrow` | `h1`–`h4`, `p` | raw headings in `hiring`, `app` |
| Signal | `Badge`, `Status`, `Swatch`, `Bar` | `span`/`div` | — |
| Report state | `EmptyState`, `Skeleton`, `ErrorBanner` | — | — |
| Navigate | `ProductHeader`, `NavigationRail`, `ProcessRail`, `Tabs`, `Footer` | `header`/`nav` | — |
| Market | `Hero`, `FeatureCard`, `StepList`, `FactList`, `PricingCard` | composites (UI6) | — |

`NavItem`, `ProductNavItem` and `ProductHeaderAction` carry the same
`label + href + external` record three times. They become one `NavLink`, with
`key` optional.

`TextLink` also exports its class string as `TEXT_LINK`, because the kit has no
router (see `lib.rs`): an app's router `Link` takes the class rather than
re-typing it.

`Row` and `Stack` have no consumer; delete them (UI8).

## Exceptions

A raw element is legitimate when it is the **subject** of the page rather than
its chrome — a skip link whose whole job is to be the first focusable thing,
an element whose markup is the thing being demonstrated — and the
`isa-raw-ok` marker says which. "The instruction did not support my case" is not an
exception; it is an operand to add.

## Not decided here

- Whether `motion` keeps earning its weight. `lib.rs` already says delete the
  module and `/motion` together when it stops.
- Dark mode for `apps/app`/`apps/web`. UI2 makes it a values-only change; it
  needs a product reason first.
