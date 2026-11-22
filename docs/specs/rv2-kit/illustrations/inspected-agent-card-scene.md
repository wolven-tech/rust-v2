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
