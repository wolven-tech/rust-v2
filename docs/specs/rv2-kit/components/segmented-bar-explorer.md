# Segmented Bar Explorer

Source item: affectUI Pro `components/breakdown-chart` (functional reference only; see README).

## Purpose
Shows how a set of records splits across one category axis, optionally stacked by a second category, and lets the reader switch which metric, which axis and which stacking they look at. A linked table repeats every number.

## Data
| Field | Type | Meaning |
|---|---|---|
| `tally_rows` | Row[] | Flat rows, one per item counted. Each has a value per axis key, and optional numeric fields for metrics |
| `grouping_axes` | Axis[] | `{ axis_key, axis_label, axis_values: string[], value_swatches?, value_marks? }` |
| `metric_defs` | Metric[] | `{ metric_key, metric_label, unit, source_field? }`. Without `source_field`, rows are counted |
| `heading_text` | string | Title |
| `start_metric` | string | Metric key shown first |
| `start_axis` | string | Axis key along the bars |
| `start_stack` | string | Axis key that stacks each bar, or `"none"` |
| `root_class` | string | Extra class on card |

## Anatomy
Reading order:
1. Heading and total line (total for the current metric).
2. Three pickers in a row: Metric, Axis, Stack. Each opens a list of options.
3. Bar chart: one bar per axis value, stacked segments per stack value. Vertical value scale on the left, horizontal axis labels below.
4. Legend of stack values, each a toggle that hides or shows that segment.
5. Linked table: one row per axis value, columns per stack value and a total column.

## States
- **Default**: first metric, first axis, first stack.
- **Picker open**: list of options; highlighted option follows keyboard.
- **Segment hidden**: legend toggle off; its segment is removed from the bars and totals.
- **Hovered or focused bar**: other bars dim; matching table row is highlighted; a small breakdown shows each stack value for that bar.
- **Single bar (axis with one value)**: bar is centred; no degenerate layout.
- **No rows**: chart shows an empty-axis message and the table shows a single line.

## Interactions
- Pointer: pickers open on click; hovering a bar shows its breakdown; clicking a legend item toggles it.
- Tab: through pickers, bars, then legend items.
- Up/Down in an open picker: previous or next option; Enter picks; Escape closes and returns focus to the picker.
- Focusing a bar shows the same breakdown as hovering it.
- Hover over a table row highlights the matching bar.

## Motion (functional)
- When metric, axis or stack changes, bars grow or shrink and restack smoothly.
- Hovering dims unrelated bars and emphasises the hovered bar.
- Reduced motion: immediate.

## Accessibility
- Each bar is a button whose name states its axis value and total.
- The table carries every number so no information depends on colour or hover.
- Legend items are toggle buttons with a pressed state.
- Pickers are labelled listboxes.

## Responsive
- Chart fills its container; below a narrow width the table scrolls inside its own box, not the page.
- Legend wraps.
- No horizontal page scroll at phone width.

## Composition hints
Card, select or listbox (pickers), bar chart (custom SVG or canvas-free layout), legend (toggle buttons), table, tooltip-like breakdown panel, live region helper.

## Acceptance checks
1. Switching the metric changes every bar height to the new metric's values and updates the total line.
2. Choosing a stack axis restacks each bar in the order of that axis's values.
3. Toggling a legend entry removes that segment from bars and from the total, and sets its toggle to pressed=false.
4. Focusing a bar with Tab shows its breakdown and dims the other bars.
5. The table's per-axis totals equal the bar totals for every combination of visible segments.
6. With no `source_field` on a metric, the metric counts rows.
7. Escape closes an open picker and returns focus to its button.
8. An axis with a single value renders one bar without layout errors.
9. Hovering a table row highlights the matching bar.
10. At phone width the page does not scroll horizontally; the table scrolls within its box.

## Amendment: structure and demo

**Archetype:** a stacked column chart with three pickers in a row above it (metric, axis, stack), a legend row beneath, and a linked table under that. Every number in the chart appears in the table.

**Parts:**
- Heading and a total line for the active metric.
- Picker row: Metric, Axis, Stack, each a labelled listbox.
- Bar chart: one bar per axis value, stacked segments per stack value, a value scale on the left, axis labels below.
- Legend row: one toggle per stack value. Pressed means shown; unpressed removes that segment from bars and totals.
- Linked table: one row per axis value, a column per stack value, a total column, and a totals row.
- Breakdown panel for the hovered or focused bar, listing each stack value for it.

**Gallery demo:** 118 rows counted across four axis values ("North", "South", "East", "West") and three stack values ("Open", "In review", "Closed"). Two metrics: "Tickets" (counted rows) and "Hours" (summed from an hours field).
- Instance A (at rest): Tickets, Region axis, Status stack, all three legend toggles pressed, no focus on the chart.
- Instance B: Hours metric, Region axis, Priority stack, one legend toggle unpressed, and the "East" bar focused so its breakdown panel shows with the other bars dimmed.

**Acceptance checks (continued):**
11. Instance A at rest shows four bars, three pressed legend toggles, and a table with four axis rows and a totals row. The totals equal the bar totals.
12. Instance B shows the second metric and the second stack, one unpressed legend toggle, the focused bar's breakdown panel, dimmed unrelated bars, and the matching table row highlighted.
13. Stacked segments follow the axis's value order in both instances. Hiding a segment does not reorder the remaining ones.
14. Each picker is a labelled listbox that lists every option for its axis or metric. Picking an option updates the bar heights, the total line and the table in one step.
