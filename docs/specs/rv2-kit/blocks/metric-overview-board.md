# Metric Overview Board

Source item: affectUI Pro `blocks/kpi-board` (functional reference only; see README).

## Purpose
Gives a person the headline numbers of a dashboard in one glance: how each key figure moved, how the current period compares with the previous one, and what a single day contributed. The person can change the time range and a segment filter and watch every number follow.

## Data
- `stat_cards`: list of { id, label, total (number), current_series: number[], previous_series: number[], lower_is_better?: boolean }. Each card shows a headline total and a small trend.
- `overview_series`: list of { label, values: number[] } for the comparison chart, one entry per segment shown in the overview.
- `period_options`: list of { id, label }. The first entry is the default.
- `selected_period`, `default_period`, `on_period_change(id)`.
- `segment_options`: list of { id, label }, with `selected_segment`, `default_segment`, `on_segment_change(id)`. The segment filter only changes the overview chart, never the stat cards.
- `current_value_label`, `previous_value_label`: words used in the tooltip and the focus card.
- `format_amount(value)`: formatting for money values; default is the locale currency.
- `heading`, `leading_marker` (node), `setup_action` (label + on_activate), `footer_action` (label + on_activate), `extra_classes`.

## Anatomy
In reading order:
1. Header row: optional heading and the period picker at its right edge.
2. Stat row: three trend cards. Each shows its name, the total, a delta badge with a percentage against the previous period, and a sparkline of the current period.
3. Overview region: the segment filter at its top-right, the overview total and delta, and a comparison chart with two lines (current and previous period; the previous line is drawn lighter).
4. Focus card, beside the overview on wide layouts: the highlighted day, its current and previous value, and the `setup_action` button.
5. Footer: the `footer_action` button, when given.

## States
- Default: period set, no day hovered, focus card shows the latest day.
- Hovering or focusing a day: the focus card shows that day; the tooltip shows the date, the current value and the previous value.
- Period changed: every number rolls from its old value to the new one; sparklines and overview redraw.
- Segment changed: only the overview chart and its totals change; stat cards keep their values.
- Stat card selected: the other stat cards fade back and the overview switches to that metric.
- Empty series: the chart shows its baseline and totals read zero; no error state.
- Disabled: the period picker and segment filter can be disabled as a whole; the chart stays visible but cannot be focused.

## Interactions
- Pointer hover over a day on the overview: marks the day and shows the tooltip; leaving the chart clears the mark.
- Pointer over a stat card: a tooltip shows the same numbers, expanded.
- Click on a period option: applies it; the picker closes.
- Keyboard, pickers: Enter, Space or Down opens the menu; Up and Down move through options; Home and End jump to the first or last; Enter picks the highlighted option; Escape closes without changing the value.
- Keyboard, chart: focus enters the chart as one group; Left and Right step through the days; Home and End jump to the first and last day; Escape leaves the group and the focus card returns to the latest day.
- Tab order: period picker, each stat card, segment filter, overview chart, focus card button, footer action.
- Outside click closes any open menu.

## Motion (functional)
- On first view the stat cards appear one after another; each sparkline draws in after its card appears.
- Changing the period rolls each number to its new value, so the eye follows the change.
- Changing the segment fades the old overview lines out and draws the new ones in.
- Hovering a day places the marker on the line; the marker slides rather than jumps.
- With reduced motion preferred, every change applies instantly.

## Accessibility
- Each stat card is a labelled group: name, total and delta are read together.
- The overview chart is one labelled group; its value at the chosen day is announced when the day changes, politely.
- Delta badges carry a word for direction so a visual cue is never the only signal. For `lower_is_better` cards the word states whether the change is good or bad.
- The period and segment controls use listbox roles with the selected option announced.
- The focus card is a live region for the chosen day.

## Responsive
- Wide containers: the three stat cards sit in one row; the overview and focus card sit side by side.
- Narrower containers: the focus card moves below the overview; the period picker stays at the top right of the header.
- Narrow containers: stat cards stack to one column and the overview keeps full width.
- Touch input: the tooltip is shown on tap of a day and dismissed by tapping elsewhere.

## Composition hints
- Card (each stat card and the overview), badge (delta), sparkline and chart primitive with two series, listbox for the period and segment pickers, button for the actions, tooltip.

## Acceptance checks
1. Hovering a day on the overview shows its date, current value and previous value, and moves the marker to that day.
2. Choosing a different period changes all three stat totals and redraws the overview; numbers animate from their old values.
3. Changing the segment changes only the overview totals; stat card values stay the same.
4. Arrow keys on the focused chart step through days in order, and the announced value matches the focus card.
5. Escape on the chart returns focus to the group and resets the focus card to the latest day.
6. A stat card with `lower_is_better` set shows the delta word as good or bad, not up or down alone.
7. With reduced motion preferred, a period change shows final values with no intermediate frames.
8. At narrow width, the focus card appears below the overview and nothing scrolls sideways.
9. An empty series renders zeros and a flat baseline without an error.
