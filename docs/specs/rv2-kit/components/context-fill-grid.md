# Context Fill Grid

Source item: affectUI Pro `components/context-window` (functional reference only; see README).

## Purpose
Shows how full a model's context window is, as a grid of cells filled part by part (system prompt, tools, files, conversation, memory). A person can hover a part to highlight it, remove files, and compact the conversation, which visibly empties the cells of that part while it runs.

## Data
| Field | Type | Meaning |
|---|---|---|
| `part_list` | Part[] | `{ part_key, part_name, token_count?, sub_items?, is_locked?, can_compact?, part_color? }`. Cells are filled in this order. |
| `window_size` | number | Context size in tokens. |
| `panel_title` | string | Defaults to "Context usage". |
| `model_name` | string | Small tag in the header. |
| `model_mark` | node | Logo shown before the model name. |
| `on_item_remove(part_key, item_key)` | callback | Fired when an item is removed. When supplied, the card shows exactly the parts passed in; otherwise removal is kept inside the card. |
| `on_compact_request(part_key)` | callback, may return a promise | Fired on Compact; the part's cells shimmer while the promise is pending. Without it, the part contracts on its own to its reduced share. |
| `footer_action` | `{ action_text, on_activate }` | Button in the last row, for example "Start fresh". |
| `caution_share` | number | Share of the window at which the percentage switches to a caution tint and the last row states how much is left. Defaults to 0.8. |
| `full_share` | number | Share at which the window counts as full. Defaults to 0.95. |
| `cell_total` | number | Number of cells. Defaults to a count of our choosing. |
| `cells_per_row` | number | Cells per row. |
| `root_class` | string | Extra class applied to the root. |

Token rule: a part with sub-items takes its tokens from the sum of those sub-items; `token_count` is omitted for such parts. Each cell represents `window_size / cell_total` tokens. Any part with tokens receives at least one cell.

## Anatomy
Reading order:
1. Header: panel title, model tag with its mark.
2. Used figure: "N of M tokens used", with a percentage tag.
3. Cell field: a grid of cells coloured by part.
4. Part list: one row per part: colour swatch, name, token count. Rows with sub-items expand to show them, each with a remove control.
5. Last row: caption (compacting, saved amount, space left), a Compact control per compactable part, and the footer action.

## States
- **Normal:** cells filled in part order; percentage neutral.
- **Caution:** at or above `caution_share`, the percentage tag uses a caution tint; the caption states the space left.
- **Full:** at or above `full_share`, the caption says the window is full.
- **Part hovered:** other cells fade; the hovered part's cells stay lit; its figure rolls up.
- **Part expanded:** sub-items listed in place.
- **Item removed:** its cells empty from left to right, and the cells behind it move up.
- **Compacting:** that part's cells shimmer; the caption shimmers "Compacting…"; then most of the cells clear, the figure rolls down, and the button briefly confirms "Compacted".
- **Compacted, cooldown:** Compact is unavailable until the part grows again.

## Interactions
- Pointer: hover a row or its cells to highlight the part; click a part row to expand its sub-items; click Compact; click remove on an item.
- Tab: expandable parts, Compact, item remove controls, footer action.
- Enter or Space: expands or collapses a part; presses Compact; removes a focused item.

## Motion (functional)
- Any change sweeps across the cell field, cell by cell.
- Compacting: a shimmer wave passes over the part's cells, then most of them clear; the figure rolls down; the button morphs into a confirmation.
- Sub-items unfold in place; removed items fold away before leaving.
- Reduced motion: no movement; state changes are immediate.

## Accessibility
- The group is named by the panel title.
- The rolling figure is hidden; a single sentence states the used amount, the window size and the percentage.
- The cell field is decorative; every number it shows also appears in the list.
- Parts holding sub-items act as disclosure buttons; each remove control's name includes its item.
- The caption speaks courteously as the state changes: compacting, space recovered, room remaining.

## Responsive
- Fills its container up to a compact width.
- When the container is narrow: fewer cells per row so cells stay readable; the model tag shortens; Compact keeps only its icon.
- Touch: taller rows and buttons.
- No horizontal scrolling at phone width.

## Composition hints
Card, number display (roll), grid of cells (custom), list with disclosure rows, button (compact, remove), tag (model, percentage), live region helper, shimmer text.

## Acceptance checks
1. Parts render in the given order; the cells assigned to each part equal its share of `window_size` at the stated ratio of one cell per unit, with at least one cell for any part with tokens.
2. A part with sub-items shows the sum of its sub-items as its token count and has no separate `token_count` value in use.
3. At or above `caution_share` the percentage tag uses the caution tint; at or above `full_share` the caption states the window is full.
4. Removing an item calls `on_item_remove(part_key, item_key)` once; without the callback the item disappears from the card.
5. Pressing Compact calls `on_compact_request(part_key)` once, and the Compact control stays busy until the returned promise settles.
6. Without `on_compact_request`, the part's cells shrink to its compact share on Compact.
7. After compaction the Compact control is unavailable until the part's token count increases.
8. Hovering a part row fades the cells not belonging to that part.
9. Each expandable part is a button with expanded state, and its sub-items are not in the tab order while collapsed.
10. At phone width the cell field uses fewer columns, and no horizontal scrolling occurs.
</content>
