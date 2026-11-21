# Part-of-Total Card

Source item: affectUI Pro `components/compact-breakdown` (functional reference only; see README).

## Purpose
A small card that shows one total and what it is made of: a ranked list of parts with values and shares, and a single stacked bar. When the period changes, the numbers, bar and ranking move to the new values in step.

## Data
| Field | Type | Meaning |
|---|---|---|
| `part_entries` | Part[] | `{ part_key?, part_name, part_amount, part_color? }`. Ranked by amount for display. Give a `part_key` when names can change, so that parts keep their identity across periods |
| `card_title` | string | Card title. Default "Composition" |
| `period_options` | `{ period_key, period_name }[]` | Period switch options. Omitted: no switch |
| `active_period` / `initial_period` | string | Controlled or starting period |
| `on_period_change(period_key)` | callback | New period chosen; host returns matching `part_entries` and `change_pct` |
| `change_pct` | number | Change against previous period in percent (signed). Shown as a tag beside the total |
| `format_amount(value)` | function | Formats total, rows and hovered values. Default: whole numbers with separators |
| `total_caption` | string | Caption under the total while nothing is hovered. Default "Overall" |
| `trailing_row` | node \| null | Last row. Default: "Top part: <part name>". `null` hides it |
| `root_class` | string | Extra class on root |

## Anatomy
Reading order:
1. Heading row with period switch (when `period_options` is given).
2. Total figure with caption and change tag.
3. Stacked bar: one segment per part, in rank order.
4. Ranked list: colour swatch, name, amount, share.
5. Trailing row (under a hairline).

## States
- **Default**: total shown with caption; no part highlighted.
- **Part hovered or focused**: other segments dim; the chosen segment lifts; total figure changes to that part's amount and its share caption; leaving restores the total.
- **Period changed**: values roll to new numbers; bar segments glide to new widths; rows reorder; parts absent in the new period fold away.
- **Single part**: full-width bar; one row.
- **No parts**: empty line in place of the bar and rows.
- **Change negative**: tag uses the negative tint and a minus sign.

## Interactions
- Pointer: hover a row or a bar segment to focus that part.
- Tab: period switch, then each row (focusable rows give the same highlight as hover).
- Left/Right on the period switch: moves to the neighbouring option and wraps around at the ends. Home/End: first or last.
- Escape: none specified; leaving focus clears highlight.

## Motion (functional)
- On a period change, numbers roll to their values together in one shared transition, so total, rows and bar move in step.
- Bar segments keep their colour and place and only change width.
- Rows slide to their new rank.
- Hover on a part: dim others, lift the segment, roll the total to the part's amount; leaving rolls it back.
- Period switch highlight slides to the chosen option.
- Reduced motion: immediate.

## Accessibility
- Group named by the card title.
- Period switch is a radio group with one tab stop.
- Rolling numbers are hidden from assistive tech; a single sentence gives the final total and change.
- Each row states name, amount and share.
- The bar is decorative; all its information is in the list.

## Responsive
- Fills its container up to a compact width.
- Narrow: the share column hides; amounts stay visible.
- Touch: taller rows and period options.
- No horizontal scroll at phone width.

## Composition hints
Card, segmented control (period), number display with roll, stacked bar, list with swatches, tag (change), tooltip-free hover state, live region helper.

## Acceptance checks
1. Parts render in descending order of amount; equal amounts keep input order.
2. Switching period via `on_period_change` and new `part_entries` moves a part with the same `part_key` to its new rank without remounting its row.
3. Hovering a row sets the total figure to that part's formatted amount and restores the total on leave.
4. The trailing row shows the largest part's name by default and disappears when `trailing_row` is `null`.
5. The change tag shows the signed percentage and uses the negative style for negative values.
6. Period switch: Right arrow moves to the next period and wraps from last to first.
7. Screen-reader summary sentence contains the total and the change; rolling numbers are not announced.
8. Each row is focusable and shows the same highlight as hover.
9. A part absent from the new period is removed from rows and bar.
10. At phone width the card fits and no horizontal scroll occurs.
