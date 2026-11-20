# Metric Trend Card

Source item: affectUI Pro `components/kpi-chart` (functional reference only; see README).

## Purpose
A metric card: a headline figure, its change against the previous period, and a multi-line chart of its parts over a short span. Hovering or touching the chart shows each part's value at that point. Used on dashboards to show one number alongside how its channels moved.

## Data
- `metric_name` (text): label above the figure.
- `headline_value` (text): already formatted.
- `change_value` (number): change against the previous period. A positive value uses the success tone; a negative value uses the error tone.
- `axis_labels` (text[]): one label per point; the axis shows only the opening, central and closing entries.
- `tooltip_labels` (text[], optional): longer labels for the tooltip heading; falls back to `axis_labels`.
- `data_series` (Series[]): each `{ series_name, series_tone, series_values: number[] }`; one value per label.
- `format_value(n)` (default: compact, for example 1.2k). Used for the axis, the tooltip and the change value.
- `footer_text` (text, optional): line beneath the chart.
- `resting_index` (number, optional): point kept highlighted while nobody hovers.
- `empty_message` (text, default "No data yet"): rendered inside the plot whenever there is no data or nothing but zeros.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Metric name.
2. Headline value, with the change as a small signed chip beside it.
3. Plot: horizontal grid, one line per series, and a guide line with a marker at the highlighted point. Axis labels at the first, middle and last points.
4. Tooltip: heading (the point's label), then one row per series with its tone marker, name and value.
5. Ranked list of series by value at the highlighted point (spec choice).
6. Footer line.

## States
- Resting: highlight at `resting_index` (or the last point); the tooltip shows it.
- Hover or touch on the plot: the highlight follows the pointer to the nearest point; the tooltip follows.
- Focused: the highlight moves with the arrow keys.
- Empty: `empty_message` shown in the plot. All-zero data counts as data and draws a flat line.
- Change positive, negative or zero: tone and sign reflect it.

## Interactions
Pointer:
- Move over the plot: highlight and tooltip follow.
- Leave the plot: highlight returns to the resting point.
- Touch: tap or drag across the chart.

Keyboard:
- Tab: focuses the chart.
- Left / Right: previous or next point.
- Escape: hides the tooltip.

## Motion (functional)
- On mount, each line draws in from the left.
- The guide line, markers and tooltip hop from point to point in quick increments.
- Reduced motion: no drawing and no step.

## Accessibility
- The plot takes focus as a single unit; arrows walk its points.
- The tooltip sits in a courteous status region, so moving between points reads out the new values.
- The change carries its sign in text, not only in tone.

## Responsive
- Fills its container up to a maximum width; the plot scales while line widths stay constant.
- Touch: tap and drag on the plot.

## Composition hints
Panel, headline text, change chip, SVG line chart, axis labels, tooltip, guide marker, keyboard focus on plot, live region.

## Acceptance checks
1. With three series over seven labels, the axis shows the first, middle and last labels only.
2. Hovering the plot moves the tooltip to the nearest point and lists every series' value for it.
3. Leaving the plot returns the highlight to `resting_index`.
4. A negative `change_value` renders with a minus sign and the error tone; a positive one with a plus sign and the success tone.
5. `format_value` changes every displayed value on the axis, tooltip and change chip.
6. With no values at all, the plot shows `empty_message` and no lines.
7. Right arrow on the focused chart moves the highlight to the next point and updates the tooltip.
8. Escape hides the tooltip while focus stays on the chart.
9. The tooltip uses `tooltip_labels` when given.
10. Reduced motion: lines appear without drawing.
</content>
