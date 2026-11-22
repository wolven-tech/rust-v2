# Series Trend Chart

Source item: affectUI Pro `components/trend-chart` (functional reference only; see README).

## Purpose
A self-contained chart for embedding in any card or layout. It renders one or more series as lines, filled areas, columns or a compact sparkline, presents a tooltip that sets the current period beside the earlier one, and can publish its highlighted point so several charts on a page stay in step.

## Data
- `x_labels` (string[]): one entry per data point. The first, middle and final entries are printed on the axis.
- `tooltip_headings` (string[], optional): fuller headings for the tooltip. `x_labels` serves when absent.
- `series` (Series[]): each entry is `{ name, values, emphasis?, is_earlier? }`. `values` lines up with `x_labels`, and a null breaks the line at that point. An earlier-period series is dashed in line mode and faded in column mode.
- `variant` (`"line" | "area" | "bar" | "sparkline"`): drawing style. Sparklines drop their axes and replace the tooltip with a short tag. Line is the starting value.
- `fmt_value(n)`: formats tooltip values and, unless `fmt_axis` is given, axis ticks.
- `fmt_axis(n)` (optional): a compact formatter for axis ticks.
- `height` (number): plot height. A sensible default applies, and sparklines default to something smaller.
- `axis` (boolean or `"x"`): `true` draws value ticks on the left and dates below; `"x"` draws only dates; `false` draws neither. Starts on, or off in sparkline mode.
- `show_tooltip` (boolean): when false, the surrounding card displays the value and the point stays highlighted.
- `from_zero` (boolean, default true): anchors the value axis at zero. Setting it false zooms in on the spread of averages or rates. Columns always start at zero.
- `active` / `on_active_change` (number or null): the highlighted point, owned by the host. Pass one shared value to several charts to link them.
- `initial_active` (number, optional): a point kept highlighted while the pointer is elsewhere.
- `label` (string, default "Chart"): accessible name.
- `empty_message` (string): displayed in place of the series whenever nothing, or only zeros, is available.

## Anatomy
Reading order:
1. Plot: value axis on the left, date labels underneath, series drawn at their emphasis levels, a guide line and a point marker at the active position.
2. Tooltip (a tag in sparkline mode): the point heading, then one row per series.
3. Legend, placed by the caller when wanted.

## States
- **Idle:** nothing active; the `initial_active` point stays highlighted when supplied.
- **Engaged (hover or focus):** guide, marker and tooltip track the active point.
- **Empty:** the message appears in the plot; no tick labels.
- **Earlier period visible:** that series is dashed or faint.
- **Linked:** the active position comes from a shared value, so a move on one chart carries to the others.

Transitions: pointer or key movement updates the active position and reports it to `on_active_change`; leaving the chart reports null.

## Interactions
- **Pointer over the plot:** the highlight snaps to the nearest label; leaving reports null.
- **Touch drag:** the highlight follows the finger.
- **Keyboard, Tab:** enters the chart as one stop.
- **Keyboard, Left and Right:** move to the previous or next point.
- **Keyboard, Home and End:** go to the first or last point.
- **Keyboard, Escape:** hides the tooltip.

## Motion (functional)
- Lines and areas sweep in from the left on first render; columns rise from the baseline.
- Data changes interpolate: each point and column travels to its new position, including when the number of points changes.
- The guide, marker and tooltip hop between points quickly.
- With reduced motion requested, nothing animates.

## Accessibility
- The chart is one focusable stop and is navigable with the arrow keys.
- The tooltip sits in a polite live region, so the current values are spoken.
- The accessible name comes from `label`.

## Responsive
- Scales to the container width while keeping strokes crisp.
- On narrow layouts the middle date disappears; on narrower ones, tooltip values drop beneath their series names.
- Touch users tap or drag across the chart to read values.

## Composition hints
- Plain container; the chart supplies no card of its own.
- Line, area and column series primitives.
- Axis primitive with value and date labels.
- Popover tooltip, or a tag in sparkline mode.
- Shared highlight state for linked charts.
- Polite live region.

## Acceptance checks
1. A null value leaves a gap at that point and draws no segment through it.
2. A series flagged as earlier is dashed in line mode and faint in column mode.
3. Right arrow moves the active point to the next label and reports it through `on_active_change`.
4. Two charts sharing `active` move together when either one changes.
5. In sparkline mode there is no axis, and the tooltip is replaced by a small tag.
6. With `from_zero` false, a line chart's value axis does not start at zero; column charts still do.
7. With no values, the empty message is shown and no axis labels are drawn.
8. Tooltip values use `fmt_value`, and axis ticks use `fmt_axis` when supplied.
9. On narrow layouts the middle date label is hidden and the rest stay readable.

## Amendment: structure and demo

**Archetype:** a standalone chart block with three drawing modes over the same data: line (with optional area), columns, and sparkline. The tooltip sets this period beside the previous one for each label.

**Parts:**
- Plot: a value axis on the left with ticks, date labels below showing only the first, middle and last labels.
- Series: lines or columns. A previous-period series is dashed in line mode and faint in column mode.
- Guide line and point marker at the active index.
- Tooltip: the point's heading, then one row per series with its name and value. In sparkline mode, a small tag replaces it.
- Legend, placed by the host.

**Gallery demo:** seven labels ("W1" to "W7"). Two series: "Completed" (current period, with a null at W4) and "Opened" (previous period, dashed in line mode). The null leaves a gap in "Completed" at W4 with no segment drawn across it. This is intended and stays.

**Three gallery instances, same data:**
- Line, with the axis and the tooltip.
- Column, with the axis and the tooltip.
- Sparkline under a headline number, with no axis and a tag in place of the tooltip.

**Acceptance checks (continued):**
10. The gallery shows the three instances. The sparkline has no axis, and its tooltip is a small tag.
11. In the line instance, "Completed" has a gap at W4 with no segment drawn across it.
12. With no pointer on the chart, the highlight rests on the last label and the tooltip shows both periods for that label.
13. The line and column instances both start their value axis at zero.

## Amendment 2: structure and demo

Replaces amendment 1 in three places. The three stacked instances become one composition: a headline block with a mode control, and a row of three sparkline tiles beneath it. No tooltip is open at rest; the headline row states the last point's two periods as text, so no tooltip covers a neighbouring element. Column mode is reached through the mode control, not a separate instance. The null-gap rule is unchanged.

**Archetype:** a headline block above a row of three small sparkline tiles. The block has its own header row, a plot beneath it, and a legend beneath the plot.

**Parts that must exist:**
- Header row: title at the leading edge; at the trailing edge, a three-option mode control (line, area, column) with the active option marked.
- Headline row: the latest value of the current-period series; its change against the previous-period value as a directional badge; the previous value in muted text.
- Plot: value axis on the left with ticks, starting at zero; date labels at the first, middle and last label only; series; guide and marker at the active index; tooltip shown only on engagement.
- Legend beneath the plot, placed by the host.
- Sparkline row: three tiles, each with a small label, a headline number and a sparkline with no axis.

**Demo data (ours):**
- Seven labels, "W1" to "W7".
- Two series: "Completed" (current period, null at W4) and "Opened" (previous period, the secondary style in line mode and faint in column mode).
- Mode control set to area at rest.
- The highlight rests on the last label. The tooltip is not shown until the chart is engaged.
- Sparkline tiles: "Open items", "Reopened", "Average age", each seven values with no nulls; each headline is its last value.

**Visible at rest:** one headline block in area mode with the axis, the highlight on W7, no tooltip, and the legend; a row of three sparkline tiles beneath.

**Acceptance checks (continued):**
14. The gallery shows one headline block and three sparkline tiles beneath it. The sparkline tiles have no axis, and no tooltip or tag is open at rest.
15. The block opens in area mode. Choosing column mode draws columns, the value axis still starts at zero, and the previous-period series is faint (checks 2 and 13).
16. In area or line mode, "Completed" has a gap at W4 and no segment crosses it. This is the null rule and it stays.
17. At rest no tooltip or tag is open anywhere in the gallery. The headline row states the last point's current and previous values as text. Replaces amendment 1 check 12.
18. When the block is engaged, its tooltip sits inside the block and does not extend onto the sparkline row.
19. Tooltip values use the value formatter, and axis ticks use the compact formatter when one is given (check 8).
20. At narrow widths the sparkline tiles stack into one column and the middle date label is hidden (check 9).
