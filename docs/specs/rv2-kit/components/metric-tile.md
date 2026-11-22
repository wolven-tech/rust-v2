# Metric Tile

Source item: affectUI Pro `components/stat-card` (functional reference only; see README).

## Purpose
A single headline figure with its label and a note of how it moved against the previous period. It can carry a small chart and can act as a link or button into a detail view. Used as the KPI row at the top of a dashboard.

## Data
| Field | Type | Meaning |
|---|---|---|
| `metric_label` | string | What the figure measures. |
| `metric_value` | number or string | Numbers animate when they change and are displayed through `format_value`. Strings are shown exactly as given. |
| `format_value` | (n) => string | Formats a numeric value. Default: grouped digits. |
| `change_percent` | number, optional | Percentage change against the previous period. Shown as a badge with a direction arrow. |
| `format_change` | (absolute) => string | Formats the magnitude of the change. |
| `change_note` | string, optional | Quiet text after the change, for example "vs last week". |
| `lower_is_better` | boolean | A rise is shown as unfavourable and a fall as favourable. Defaults to false. |
| `metric_mark` | ReactNode, optional | Small icon placed before the label. |
| `inline_chart` | ReactNode, optional | Any chart; a sparkline fits. |
| `chart_position` | `"stacked" \| "inline"` | Chart beneath the figure or beside it. Defaults to stacked. |
| `footer_content` | ReactNode, optional | Last row of the tile. |
| `is_loading` | boolean | Shows placeholders in place of the value, change and chart; keeps the size. |
| `target_href` / `on_activate` | string / () => void | Makes the whole tile a link or button. |
| `corner_control` | ReactNode, optional | A small control in the top corner, such as a period menu. It stays usable when the tile is a link. |
| `is_chosen` | boolean, optional | One tile of a set is chosen; the others fade their content. |
| `target_attributes` | attributes object, optional | Extra attributes for the link or button, for example role and selected state when tiles select a chart's series. |

## Anatomy
In reading order:
1. Top row: optional icon and label; optional corner control at the far end.
2. Value: the headline figure.
3. Change row: direction badge with the percentage and the change note.
4. Chart area: beneath the value (stacked) or beside it (inline), when given.
5. Footer row, when given.

## States
- **Loading:** placeholders for value, change and chart; a slow sheen moves across them; the tile is marked busy.
- **Value changed:** the figure rolls from its old value to the new one.
- **Change favourable / unfavourable:** the badge's emphasis and arrow follow `lower_is_better`.
- **No change figure:** the change row is omitted.
- **Interactive (link or button):** focus and hover emphasis apply to the whole tile.
- **Chosen:** the tile is emphasised; other tiles in the set fade their content.
- **Not chosen:** content fades back.

Transitions: a new value triggers a roll; entering loading replaces the content with placeholders, and leaving it restores the content.

## Interactions
- **Pointer, tile (link or button):** activates the link or the handler.
- **Pointer, corner control:** operates the control without activating the tile.
- **Pointer, chart inside an interactive tile:** operates the chart independently.
- **Keyboard, Tab:** focuses the tile when it is interactive, then the chart inside it, then the corner control.
- **Keyboard, Enter:** activates the tile's link or button.
- Non-interactive tiles are not in the tab order unless their chart is focusable.

## Motion (functional)
- On first view, the figure rolls up from zero to its value.
- When the value changes, it rolls to the new value.
- Placeholders show a slow sheen while loading.
- Reduced motion: rolling and sheen are off; the final value appears at once.

## Accessibility
- Assistive tech reads the settled value and the change as words, e.g. "up 16 percent vs last week"; mid-roll numbers are never announced.
- An interactive tile's accessible name includes its label, value and change.
- The embedded chart remains independently navigable.
- While loading, the tile is busy.

## Responsive
- Occupies its grid cell entirely. The figure wraps instead of clipping; labels and comparisons end in an ellipsis.
- Works in a row of tiles that reflows into columns as the width shrinks.

## Composition hints
- Card for the container.
- Number formatting utility.
- Badge with arrow for change.
- Sparkline or chart slot.
- Skeleton placeholders for loading.
- Icon for the label.

## Acceptance checks
1. A value change from one number to another rolls through intermediate values on screen and ends on the new value.
2. A change of positive 8 percent with `lower_is_better` false shows a favourable upward badge; with `lower_is_better` true the same change shows an unfavourable badge.
3. Passing `target_href` makes the whole tile a link: Enter follows it.
4. The corner control inside a link tile can be operated without following the link.
5. With `is_loading` true, the value, change and chart are replaced by placeholders, and the tile carries the busy state.
6. A string value is displayed exactly as given with no animation.
7. Screen readers announce the value and the change in words once the roll completes.
8. In a set where one tile is chosen, the chosen tile shows full content and the others fade their content.

## Amendment: structure and demo

Layout archetype: a KPI row. Four equal-width tiles sit side by side in one row; each tile is a vertical stack of top row, value line, change line and footer. Only one tile carries an inline chart beside its value, and only one carries a corner control. The row reflows into a grid as width shrinks.

Parts that must exist:
- Top row: icon and label at the leading edge; the corner control at the trailing edge when given.
- Value line: the headline figure, large. When an inline chart is given, the chart sits to its trailing side, aligned with the value.
- Change row: a badge with a direction arrow and the percentage, then the change note in muted text.
- Footer row: the reading date for the value, in small muted text.

Demo data for the gallery (our own names and values):
- Four tiles: "Open tasks", "Median latency", "Cost", "Failed runs".
- Tile one holds the corner control: a two-segment period toggle reading "7 days" and "30 days", with the 7-day segment active. This tile also carries an inline sparkline.
- Change badges: "Open tasks" rises and is favourable; "Median latency" falls and is favourable (lower is better); "Cost" rises and is unfavourable (lower is better); "Failed runs" falls and is favourable.
- Each tile carries a change note of our own wording.
- Footer date identical across tiles.

States visible at rest in the gallery: four tiles settled; one favourable rise, one favourable fall on a lower-is-better tile, one unfavourable rise on a lower-is-better tile, one favourable fall; the corner control on tile one; the inline chart on tile one; no tile in loading state and no tile chosen.

Acceptance checks:
9. At desktop width the gallery demo shows four equal-width tiles in one row.
10. Each tile shows its top row, value line, change row and footer row in that order from top to bottom.
11. The corner control sits at the trailing edge of the first tile's top row and shows two segments with the first active.
12. The inline chart sits beside the value on the first tile only.
13. The change badges match the direction and favourability in the demo data: the lower-is-better tiles show a rise as unfavourable and a fall as favourable.
14. The gallery demo shows no loading placeholder and no busy state at rest.
15. With the first tile as a link (SSR test), the corner control still renders as a separate control.

## Amendment 2: structure and demo

Replaces amendment 1 in four places. The four tiles form a two-by-two grid, not one row. The period toggle leaves the first tile and sits in a header above the grid. Every populated tile carries its chart beneath its change row, not beside its value. The loading example moves into its own group below the populated grid. The tile's corner-control prop is unchanged; the gallery simply does not use it on a tile.

**Layout archetype:** a group with a header row and a two-by-two grid of tiles. Below it, a separate labelled group holding one loading tile on its own.

**Tile anatomy, top to bottom:** top row with icon and label at the leading side; the value, large; change row with the direction badge and its percentage, then the change note in muted text; the chart spanning the tile width beneath the change row; footer row with the reading date in small muted text.

**Group header:** group label at the leading edge; a two-segment period toggle at the trailing edge, first segment active.

**Demo data (ours):**
- Four populated tiles. "Open tasks": 42, rises, favourable, line chart. "Median latency": 1.2 s, falls, favourable, lower is better, line chart. "Cost": $18.40, rises, unfavourable, lower is better, column chart. "Failed runs": 3, falls, favourable, line chart.
- Change notes in our own wording: "against the prior seven days".
- Footer reading date identical across the four tiles.
- Loading group: one tile in loading state, under the label "Loading example".

**Visible at rest:** the populated grid with four settled tiles and four charts; the group header toggle; no placeholder in the populated grid; the loading group below it.

**Acceptance checks (continued):**
16. At desktop width the populated tiles form a two-by-two grid: two tiles on each row.
17. Each populated tile shows its top row, value, change row, chart and footer, in that order from top to bottom.
18. The period toggle sits in the group header at the trailing edge, not inside any tile, and its first segment is active.
19. Four charts render at rest, three of line kind and one of column kind, each beneath its tile's change row and none beside a value. Replaces amendment 1 check 12.
20. The populated tiles show no placeholder and no busy state. The loading example sits in its own labelled group below the grid (replaces amendment 1 check 14).
21. Change badges match direction and favourability for the four tiles (check 13).
22. A corner control on a link tile still renders as a separate control (check 15).

