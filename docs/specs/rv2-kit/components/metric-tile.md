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
</content>
