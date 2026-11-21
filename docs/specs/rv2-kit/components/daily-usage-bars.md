# Daily Usage Bars

Source item: affectUI Pro `components/metering-chart` (functional reference only; see README).

## Purpose
A usage card for metered services: one bar per day for a quantity such as API calls, tokens, events or minutes, with totals above. While today is still accumulating, its bar grows live and a status badge says so. Used in usage-based billing views.

## Data
- `chart_heading` (text): heading of the card.
- `day_values` (Day[]): `{ day_label, day_value }` for each day, oldest first. A missing or invalid value leaves an empty slot.
- `total_figures` (Figure[]): `{ figure_label, figure_value, figure_format? }`. Figures roll to their new values when they change.
- `period_text` (text): the span shown at the right, for example "Sep 1 – Sep 21".
- `is_today_live` (boolean, default false): the final bar stands for today and is still filling. Switches that bar to the accent tone, adds a moving sheen, and spins the badge's indicator.
- `live_badge_text` (text, default "Real-time" when live; empty for none).
- `format_figure(n)` (default: thousands separators, e.g. 1,284): renders the tag and totals.
- `format_axis(n)` (shorter numbers for the value axis, for example 4.5k).
- `bar_palette` (text[]): tones the bars move through from left to right.
- `live_bar_tone` (text): tone of the live bar.
- `plot_height` (number, a moderate fixed height).
- `empty_message` (text, default "No usage yet").
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Title row: heading, live badge (when live), and the period text.
2. Totals row: each figure with its label; figures roll to new values.
3. Plot: one bar per day, with a value axis on the left and day labels at the first, middle and last positions. The live bar, when on, is the rightmost.
4. Hover or focus tag: day label and value for the active bar.

## States
- Static: all bars use the palette; no badge.
- Live: the last bar uses the live tone with a sheen; the badge shows with its turning indicator.
- Empty: `empty_message` in the plot area; axis hidden.
- Partial data: invalid days render as empty slots with zero height.

## Interactions
Pointer:
- Hover a bar: shows its tag with the day label and value.
- Touch: tap a bar to show its tag.

Keyboard:
- Tab: focuses the chart.
- Left / Right: previous or next day.
- Home / End: first day or today.
- Escape: closes the tag.

## Motion (functional)
- On first display, bars climb in a rippling sequence starting from the left.
- When values change, heights and the axis ease to their new values.
- The live bar carries a slow sheen; the badge's indicator turns.
- Reduced motion: all of this is off.

## Accessibility
- The whole chart is a single focus target; arrows step across its days.
- The hover tag reports through a courteous status region.
- Assistive readers receive totals only once they settle.

## Responsive
- Fills its container. Bars keep a fixed width and thin out on narrow panels; the period text moves beneath the title.

## Composition hints
Panel, badge (live indicator), totals row with rolling numbers, bar chart (SVG or elements), axis labels, tag, live region, sheen animation.

## Acceptance checks
1. With 21 days of data, the chart shows 21 bars; the axis labels read the first, middle and last day.
2. With `is_today_live: true`, the last bar uses the live tone, the badge reads "Real-time", and the sheen runs.
3. Changing a figure's value rolls it to the new value; the read-out after the roll shows the final value.
4. An invalid entry in `day_values` renders as an empty slot and does not break the other bars.
5. With no data, `empty_message` shows and no bars render.
6. Right arrow moves focus to the next day, and the tag shows its label and value.
7. `format_figure` changes the totals and tag; `format_axis` changes only the axis labels.
8. When the container is narrow, the period text sits beneath the title.
9. With `is_today_live: false`, no badge is rendered by default.
10. Reduced motion: values update without wave or sheen.
</content>
