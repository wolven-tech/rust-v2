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
