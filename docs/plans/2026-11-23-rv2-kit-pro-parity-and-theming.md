# rv2-kit: every Pro component at HIGH parity, every component themable

Goal: all 88 rv2-kit components that stand in for an affectUI Pro component
rate HIGH against it, and every rv2-kit stylesheet takes all of its colour,
shape and density from tokens an app can override. Lands on PR #35.

## Where it stands (2026-11-23)

**Parity.** Rated from side-by-side screenshots, light, 1280 px, one resting
state each: 40 HIGH, 42 MED, 6 LOW. Seven confirmed defects, independent of
parity:

| Component | Defect |
|---|---|
| agent-clash-map | Agent-kind chips sit on top of the agent names and clip them |
| agent-context-map | Memory cards stack on each other; the last runs past the frame |
| tool-call-network | Search and Sheets nodes overlap; Database clipped at the canvas edge |
| baton-pass-log | Item cards spill into neighbouring agent cards; last column clipped |
| model-failover-flow | Connectors stop at the active model; nothing reaches Reply |
| period-picker | Calendar body renders empty |
| delegation-card-tree | Card names truncate to two or three letters |

Rating definitions, used unchanged for the exit check:

- **HIGH**: same layout archetype, comparable density, the same states visible
  at rest. A user would call it the same component.
- **MED**: same idea; archetype or density clearly differs.
- **LOW**: different archetype, sparse, or broken.

**Theming.** `assets/kit.css` defines 29 `--kit-*` tokens, each an rv2
register with a light fallback, so an app's palette and dark mode reach the
kit. 57 of the component stylesheets still carry literal colours. The literals
fall into four groups:

1. An identity palette (blue, violet, teal, amber, rose, slate) pasted per file
   with a dark pair under `[data-theme="dark"]`.
2. Card shadows written as `rgb(15 23 42 / …)`.
3. Tinted marks written as `oklch(95% 0.035 var(--<prefix>-hue))`: the hue is a
   variable, but the lightness and chroma, and their dark pairs, are literal.
4. One-offs: chart series colours, scrims, the reasoning violet.

Radii are literal `12px` in most files even though `--kit-radius*` exists.

`cargo xtask ci` step `no hardcoded hues` scans `.rs` files for Tailwind
palette classes only. It does not read stylesheets, so none of the above is
caught.

## The clean-room constraint on "HIGH"

The rv2-kit components are a clean-room rebuild (AF7): builders work from the
specs and never see affectUI's Pro pages. Reaching HIGH must not route Pro
visuals to builders through a brief. The procedure:

- **Spec room only reads the public pages.** A fresh spec-room pass compares
  each MED or LOW component's public demo with its spec and writes
  **functional** amendments: the layout archetype (graph, board, tree,
  timeline), which parts exist, how many items the demo carries, and which
  states show at rest. No colours, sizes, spacing, timings, CSS or source
  prose. Each amendment is scanned for shared 8-word runs against the captured
  page text, as the original specs were.
- **Builders read the amended spec only.** The parity table and screenshots
  stay with the orchestrator and the raters.
- **Raters never brief builders.** A rater's "main gap" phrase is a visual
  description and does not enter any builder prompt.

HIGH is therefore a claim about structure, density and states. It is not a
claim of matching look. The look stays ours, set by the tokens below.

## Work

Classified rock / sand / water per the betting table.

### T1. Token layer — rock

Extend `assets/kit.css`, each token with a light and a dark value:

- `--kit-id-1` … `--kit-id-6` and `--kit-id-*-soft`: the identity palette.
  Components pick by index, never by hue name.
- `--kit-series-1` … `--kit-series-6`: chart series, distinct from identity.
- `--kit-tint-bg-l`, `--kit-tint-bg-c`, `--kit-tint-ink-l`, `--kit-tint-ink-c`:
  the lightness and chroma of tinted marks. Components keep their own hue
  variable and compose `oklch(var(--kit-tint-bg-l) var(--kit-tint-bg-c) var(--hue))`.
- `--kit-shadow-card`, `--kit-scrim`, `--kit-think`.
- `--kit-radius-card` and a density scale (`--kit-space-1` … `--kit-space-6`,
  `--kit-text-xs` … `--kit-text-lg`) so compact and comfortable variants are one
  override.

Document every token in `docs/design-system/` with its job and default.

### T2. Sweep the 57 stylesheets onto tokens — rock

Replace each literal with the token for its job. Done by the lane that owns the
component (see Lanes), in the same pass as its parity work, so no file is
touched twice.

### T3. Gate — rock

Extend `no hardcoded hues` in `tooling/xtask` with a stylesheet scan:
`crates/rv2-kit/assets/*.css` except `kit.css` may not contain `#hex`, `rgb(`,
`hsl(`, or an `oklch(` whose lightness or chroma is a literal. Opt-out is the
existing `hue-scan-ok` marker. Text scan, so it runs before any compile.

It lands before the sweep, so it carries a checked-in list of the files that
offend today. The list may only shrink: a file not on it that offends fails,
and a file on it that is clean fails until it is removed. The work is done when
the list is empty. Stylesheets outside the four lanes (today only
`dictation-capture-panel.css`) are swept by a residual task.

### T4. Proof of theming — rock

- The gallery gets a theme picker: light, dark, and a third test theme that
  overrides only the rv2 registers and the T1 tokens (different accent,
  identity palette, radius, density).
- A screenshot pass renders every item under all three themes and fails any
  item whose pixels are identical between two themes. An identical render
  means the item ignores the tokens.
- Contrast check: text against its background is at least 4.5:1 in every theme.

### P0. Fix the seven defects — rock

One commit per lane, per component, re-shot and checked at 1280, 390 and 320 px
in light and dark.

### P1. Spec amendments for the 48 MED and LOW components — sand

Run by the spec room as described above. The MED/LOW list falls into three
kinds:

| Kind | Count | Examples |
|---|---|---|
| Thin demo (more rows, more states at rest) | ~20 | data-table, stepper-form, kpi-chart, scheduled-runs, repo-map |
| Missing part (a pane, a control, a connector) | ~18 | prompt-library preview, agent-trail rail, agent-tree branch lines, review-center diff pane |
| Different archetype | ~10 | agent-fleet (list vs graph), agent-recap (list vs board), model-router (bars vs flow), agent-cursors (no live cursors) |

The archetype kind moves the most: each is a rebuild of the component's
layout. It becomes a rock once the amendments are written and each names its
archetype.

### P2. Build from the amended specs — sand

Builders implement amendments, the T2 sweep and any P0 fix in one pass per
component. New tests come from each amendment's acceptance checks (SSR
structure, item counts, states).

### P3. Re-rate — sand

Fresh raters, who did not build, rate all 88 pairs at light and dark, 1280 and
390 px. Anything below HIGH returns to P1 with the rater's gap translated into
a functional amendment by the spec room. At most three rounds. A component
still below HIGH after three rounds is listed in the PR body with the reason,
not hidden.

### D1. Design review — rock

A senior UI design review of the kit after P3 and T4, by a reviewer with fresh
context. It judges the kit on its own merits and never against affectUI,
because its findings go to builders. Scope: every rv2-kit component under
light, dark and the test theme, at 1280, 390 and 320 px. It covers hierarchy,
type scale, spacing, colour and contrast, state design, density, alignment,
motion, and consistency across all 88. Findings are ranked by severity, each
with an item, a theme, a width, a screenshot and a fix. They are written to
`docs/review/2026-11-23-rv2-kit-design-review.md`.

### D2. Resolve the review — sand

Every finding is fixed or recorded in the review as accepted with its reason.
T3 and T4 are re-run afterwards.

## Lanes

Split by disjoint file sets. Each lane owns its components' `src/**/<name>.rs`,
`assets/<name>.css`, gallery demo and tests. `kit.css`, `agents.rs`,
`components.rs` and the gallery `main.rs` belong to the orchestrator.

| Lane | Components |
|---|---|
| A | Agent components (23) |
| B | AI interaction: answers, approvals, chat, memory, reasoning, review (22) |
| C | Charts, metrics and planning (22) |
| D | Inputs, pickers, cards, settings-style product components (21) |

T1 and T3 land first, from the orchestrator, so every lane builds on the
tokens and the gate catches regressions from the start. Builds run one at a
time on the machine; lanes use cloned target directories.

## Exit criteria

- 88 of 88 rated HIGH, or each exception listed with its reason.
- Zero confirmed defects at 1280, 390 and 320 px, light and dark.
- `no hardcoded hues` passes with the stylesheet scan; no `hue-scan-ok` in
  rv2-kit assets beyond those justified inline.
- The three-theme screenshot pass reports every item changing between themes
  and no text under 4.5:1.
- Every design review finding has a status: fixed, or accepted with a reason.
- `cargo xtask ci` green on the final head.

Progress is tracked in Chronis under epic `t-cedb7a`.

## Not in scope

- The 51 rv2-affect components: they follow affectUI Clean (AF8) and theme
  through `--aui-*` and the rv2 bridge already.
- Matching affectUI's colours, sizes or motion.
- The 75 non-component Pro items (blocks, screens, illustrations, motion). The
  same procedure applies to them in a later pass.
