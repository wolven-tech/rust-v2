# Inspected Agent Card Scene

Source item: affectUI Pro `illustrations/inspect-agent-card` (functional reference only; see README).

## Purpose
A product scene that shows an agent card as if a designer were inspecting it in a layout tool: the card is tilted back in space, its parts are outlined, spacing values are shown between them, and a guide walks through its rows to explain the structure.

## Data
- `card`: { title, model_label, status_label, task_line, rows: { label, value, bar_fraction? }[], footer: { cost_label, time_label } }.
- `inspect_steps`: ordered list of { row_index, note } used by the guide.
- `auto_play` (boolean, default true); `speed`: "slow" | "normal".
- `extra_classes`.

## Anatomy
In reading order:
1. Ghost cards: one faded card above and one below the focused card, giving depth.
2. Focused card: header (title, model line, status), task line, rows of metrics, footer line.
3. Selection frame: an outline around the focused card and each row being inspected.
4. Spacing markers: small numbered labels between parts giving distances in units.
5. Guide line: a line that moves down the card, pausing at each row, with a pointer mark on the active row.
6. Row label tag: names the row under the guide.

## States
- Playing: guide moves through each row in order; spacing numbers update for each row.
- Paused (on hover or focus): guide stops on the current row; spacing for that row stays visible.
- Idle when off-screen: guide stops.
- Reduced motion: guide is shown at the first row with all spacing visible; no movement.

## Interactions
- Pointer over a row: pins the guide to that row until the pointer leaves.
- Pointer leaving the scene: resumes playing.
- Keyboard focus on the scene: pauses; Left and Up move the guide back a row; Right and Down move it forward; Escape resumes play.
- No click behaviour.

## Motion (functional)
- The card settles into its tilted view on first view, then the frame draws around it.
- The guide moves smoothly from row to row and pauses on each; the row label changes as it passes.
- Spacing markers appear next to the row being inspected and fade for others.
- Ghost cards drift a small amount to show depth; they do not play the guide.
- With reduced motion preferred, the static view with all spacing shown is used.

## Accessibility
- Decorative; the scene's card content is also available as text by the parent. `aria-hidden` on the scene; the guide is not announced.
- Keyboard control is provided by the scene only when interactive is requested by the parent; otherwise it is not focusable.

## Responsive
- Scales with its container; the tilt reduces at narrow widths so the card stays readable.
- Phone width: the spacing markers are hidden and only the guide and row labels remain.

## Composition hints
- Card (focused and ghost), outline overlay, marker labels, a moving guide line, tag label.

## Acceptance checks
1. The guide visits each row in order and pauses on each.
2. Pointer over a row pins the guide to that row; leaving resumes play.
3. Spacing markers for the current row are visible and labelled with numbers.
4. With reduced motion preferred, the static view shows all rows and spacing with no movement.
5. Keyboard Right moves the guide forward one row and Escape resumes play.
6. At phone width the spacing markers are hidden and nothing scrolls sideways.

Uses no partner logos.

## Amendment 5: structure and demo

Archetype: one focused agent card, tilted back, with a faded second agent card above it and a selection overlay that walks its rows.

Visible at rest:
- Focused card with three rows: header (avatar, agent name, model line, status badge), task line (one sentence), footer (cost and elapsed time).
- A faded second agent card above the focused card, fading toward the top edge, showing its header and task line only.
- Selection frame on the header row, with corner handles, a row tag reading "Header", spacing numbers between the header and the task line, and a width label on the frame.
- Guide line at the card's left edge, with a pointer mark on the active row.

Gallery demo:
- Focused agent: "Ledger reviewer", model line "Compact model", status "Done", task "Match March invoices to bank lines", footer "$0.27 · 4 min".
- Faded agent above: "Export checker", model line "Compact model", status "Running", task "Check the totals in the export file".

Motion, settled frame: card in its tilted view; frame around the header row; guide on the header; spacing numbers beside the header.
Moves: the guide walks header, task line, footer in order. Each row's frame, handles, tag, spacing numbers and width label come in one piece after another, hold, then leave together. The faded card drifts slightly.
Reduced motion: static view, header selected, all spacing shown.

Acceptance checks (continued):
7. The focused card has exactly three rows, and the selection frame rests on the header row.
8. The faded second card sits above the focused card and never takes the guide.
9. Each row's tag names that row: "Header", "Task", "Footer".
10. The width label on the frame equals the rendered width of the row the frame is on.

## Amendment 6: structure and demo

Supersedes Amendment 5 where they differ. Replaces:
- "Visible at rest", faded-card bullet: the faded card above shows header, task line and footer, with status Done.
- "Visible at rest", guide bullet: the pointer mark sits on the focused status badge, not on the guide.
- "Gallery demo", both bullets: replaced by the data below.

Archetype: one focused agent card tilted back, one faded card above it, a selection band that walks its rows.

Visible at rest:
- One focused card: header (avatar, name, model line, status badge with a spinner glyph), task line, footer (cost and elapsed time).
- One faded card above the focused card, fading toward the top edge: header, task line, footer, status Done. It is a different agent, not a copy.
- No card below the focused card.
- Selection frame on the header row: corner handles, tag "Header", width label at the row's right end.
- Spacing tags: one on the frame's top edge, one on its left inset, one in the gap between header and task line. The gap tag reads 0 at rest.
- Pointer mark on the focused status badge.
- Guide line on the card's left edge.

Gallery demo:
- Focused: name "Patch writer", model line "Large model", status "Running", task "Fix the broken link in the setup guide", footer "$0.42 · 9 min".
- Faded above: name "Link checker", model line "Small model", status "Done", task "Check every link on the docs site", footer "$0.18 · 3 min".

Scale: the card group fills most of the stage width. Its left edge sits about a third of the way across; its outer right edge runs to the stage edge. The focused card is the largest element; the faded card sits in the upper third. No text of the focused card is cropped.

Motion, settled frame: card in its tilted view; frame on the header row; guide on the header; spacing tags visible; spinner turning.
Moves: as Amendment 5. Reduced motion: static view, header selected, all spacing shown, spinner still.

Defects, stated as checks: two copies of one card with dimension tags only (check 11); no pointer mark on the active row (check 12).

Acceptance checks (continued):
11. At rest one focused card and one faded card are drawn, with different names. The focused status reads Running; the faded status reads Done.
12. The focused header row carries the frame, the Header tag, the width label, the top, left and gap spacing tags, and a pointer mark on its status badge.
13. The focused card's text is fully visible at 1280 px and 390 px widths. Only the card's outer edge may pass the stage edge.
