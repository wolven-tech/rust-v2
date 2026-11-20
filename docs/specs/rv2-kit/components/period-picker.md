# Period Picker

Source item: affectUI Pro `components/date-range-picker` (functional reference only; see README).

## Purpose
Lets a person settle on a start and an end date, either from a list of relative presets or by clicking through calendar months, and optionally line up a comparison against an earlier span. Dashboards, reports and filters use it for any record with a start and an end date.

## Data
- `selected_range` (`{ from: Date, to: Date }`): host-owned range.
- `starting_range` (same shape; default: the 30 days ending today): used when the host does not own the range.
- `on_apply(range, comparison_range, comparison_mode)`: fired on Apply. `comparison_range` is null when comparison is off.
- `preset_list` (Preset[]): each entry is `{ preset_id, preset_label, resolve(today) => { from, to } }`. The built-in set offers the current day, the previous day, trailing windows of seven, thirty and ninety days, the current and previous calendar month, the current quarter, year to date, and a custom choice.
- `show_comparison` (boolean, default true): displays the comparison control.
- `starting_comparison` (`"none" | "previous_period" | "previous_year"`, default `"previous_period"`).
- `first_weekday` (`0 | 1`, default 1; 0 is Sunday).
- `earliest_day` / `latest_day` (Date; the latest defaults to today). Days outside those bounds are not selectable.
- `start_expanded` (boolean, default false).
- `anchor_today` (Date, default now): the day that presets resolve against. Tests and captures pin it so results stay stable.
- `extra_class` (text, optional).

## Anatomy
Trigger: a button showing the current range as text, plus the comparison state when one is active.

Panel, opened from the trigger:
1. Preset list: a left column on wide layouts, a horizontally scrolling chip row on phones.
2. Calendar area: two month grids side by side, or one on phones. Each grid carries a month heading with previous and next controls.
3. Comparison control: a labelled choice among None, Previous period and Previous year.
4. Footer: a summary of the selected range, plus Cancel and Apply.

## States
- Closed: trigger only.
- Open, nothing picked: the current range is marked.
- Start picked: the start day is marked and the end is pending. A lighter band tracks the pointer to preview the candidate range.
- Range complete: both ends are marked and the days between them are banded. Apply commits.
- Comparison on: a secondary band marks the comparison span; off: no band.
- Blocked days: outside the bounds; dimmed and unselectable.
- Today: marked with a dot and the current-day attribute.

Transitions: the first click sets the start; the second sets the end, swapping the two if the end comes first. Choosing a preset sets the range immediately and moves the view to show it.

## Interactions
Pointer:
- Click a preset: applies that range.
- Click a day: the first click sets the start, the second sets the end.
- Hover after the first click: shows the candidate band.
- Click previous or next month: moves the view.
- Click a comparison option: changes the comparison mode.
- Apply commits and closes. Cancel or an outside click closes without changes.

Keyboard:
- Arrow keys: one day left or right, one week up or down.
- PageUp and PageDown: previous or next month.
- Home and End: start or end of the week.
- Enter: picks the start, then the end.
- Escape: closes without applying.
- Closing returns focus to the trigger.

## Motion (functional)
- The panel opens out of the trigger with a short scale and fade.
- A month change slides the new month in from the side the person moved toward.
- The calendar area changes height as the number of visible weeks changes; the footer stays fixed.
- After the first click, the candidate band follows the pointer until the second click.
- With reduced motion requested, every transition is immediate.

## Accessibility
- Opening the trigger yields a dialog. Months are exposed as grids, and each day's name spells out the complete date.
- Today is exposed as current; picked days are exposed as selected.
- The month name is announced whenever the view changes.
- Arrow keys move focus; focus returns to the trigger on close.

## Responsive
- Below the phone breakpoint: presets become a horizontally scrolling chip row, only the month containing the end date is visible, and the footer stacks.
- Panel width follows the screen, capped at a set maximum. A width setting lets the host take the full width.
- On touch, day cells and buttons are larger.

## Composition hints
Popover or dialog, segmented list (presets), calendar grid (two instances), chip row, radio group (comparison), button pair (cancel and apply), text summary.

## Acceptance checks
1. Opening with `selected_range` set marks that range, and the trigger text equals the formatted range.
2. Choosing a preset sets the range and moves the view so both ends are visible.
3. Click a start, then hover: the candidate band follows the pointer; click an end and Apply commits both dates in order.
4. Clicking an end before the start swaps the two so `from <= to`.
5. Days after `latest_day` cannot be selected by pointer or keyboard.
6. With comparison set to previous period, `on_apply` receives a comparison range of equal length ending the day before `from`.
7. Escape closes without calling `on_apply`, and focus returns to the trigger.
8. Arrow keys move focus one day or one week; PageDown moves the view to the next month.
9. Below the phone breakpoint the presets render as a scrolling chip row and only one month is visible.
10. With reduced motion requested, opening and month changes appear instantly.
