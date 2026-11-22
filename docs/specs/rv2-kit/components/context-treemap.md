# Context Treemap

Source item: affectUI Pro `components/context-map` (functional reference only; see README).

## Purpose
Renders an agent's context window as a treemap: one tile per part (instructions, capability definitions, recall, documents, conversation, tool output, digest), with unused room drawn as a dashed remainder. Before and after views show what a compaction changed.

## Data
| Field | Type | Meaning |
|---|---|---|
| `block_list` | Block[] | `{ block_key, block_label, block_kind, token_count, block_note?, block_mark?, sub_items? }`. The order is the tile order and must remain stable. |
| `block_kind` | `"instructions" \| "capabilities" \| "recall" \| "document" \| "conversation" \| "tool_output" \| "digest"` | Selects the tint of the tile. |
| `window_size` | number | Context size in tokens. The remaining space is the free area. |
| `model_name` / `model_mark` | string / node | Shown in the title area, with the window size as a small tag. |
| `agent_options` / `active_agent_id` / `on_agent_change(id)` | list / string / callback | Changes the agent in view. The bar presents the choice as a menu; the chosen agent's window size takes the place of `window_size`. |
| `compaction_record` | `{ pre_blocks, folded_keys?, removed_keys?, digest_text? } \| null` | Snapshot of blocks prior to the latest compaction, plus which were merged into a digest or dropped. Enables a before/after toggle. `null` renders a "no compaction has run" note at unchanged height. |
| `active_view` / `on_view_change(view)` | `"before" \| "after"` / callback | Controlled side of the compaction. When omitted, the component keeps its own state. |
| `status_text` | string | Short status shown in the bar, for example "Compacting". |
| `on_block_pick(block \| null)` | callback | Fired when a tile is pressed; `null` for free space. |
| `panel_title` | string | Defaults to "Context". |
| `root_class` | string | Extra class applied to the root. |

## Anatomy
Reading order:
1. Bar: panel title, model tag, agent menu (if any), used-of-window text, fill percentage, status text, and the before/after switch (when compaction data exists).
2. Treemap area: tiles sized by token count, with a dashed free-space region.
3. Compaction line below the map: "no compaction has run", or a count of folded and removed blocks.
4. Hover card for the tile under the pointer: label, kind, token count, sub-item list.

## States
- **Idle:** tiles at rest; free area dashed.
- **Tile hovered or focused:** its card shows; a click holds the card open; a second click, Escape, or the pointer leaving the map releases it.
- **Compacting:** status shows; tiles being squeezed out shrink to nothing while a digest block grows.
- **Compacted, before view:** pre-compaction blocks shown.
- **Compacted, after view:** kept blocks remain; folded and removed ones shrink away; the digest block is shown.
- **Over window:** free area is zero; the bar uses a caution tint and the fill text says full.
- **No blocks:** the map shows only the free area.

## Interactions
- Pointer: hover a tile for its card; click to hold it open; click free space to report `null`.
- Tab: each tile in order, then the agent menu and the before/after switch.
- Enter or Space on a tile: holds its card open or releases it.
- Escape: releases a held card.
- Left/Right on the before/after switch: changes the view.

## Motion (functional)
- Tiles keep their order and glide to new places and sizes as tokens change.
- New tiles fade in at their layout position; leaving tiles shrink away.
- A shine passes continuously over the used-tokens figure.
- Text changes after compaction roll: the old line leaves upward and the new one arrives from below.
- The before/after switch glides between maps: kept blocks stay, folded and removed blocks shrink as the digest grows, and the reverse happens on the way back.
- Reduced motion: tiles jump to their place.

## Accessibility
- The map is a list named by its panel title; every tile is an item whose name carries label, kind and token count.
- Fill level is conveyed as text, not colour alone.
- A held card is announced through its name when it opens.

## Responsive
- The map area holds one height; tiles too small for text show colour alone and surface their details on hover or focus.
- When the container is narrow, the area is shorter and figures move onto their own line.
- No horizontal scrolling at phone width.

## Composition hints
Card (container), toolbar or bar (title, menu, switch), toggle group or segmented control (before/after), treemap (custom layout), tooltip or popover (tile card), list (accessible tiles), live region helper.

## Acceptance checks
1. Tiles render in the order given; their areas sum to the used tokens, and the free area fills the rest of `window_size`.
2. Adding a block to `block_list` moves existing tiles and inserts the new one without reordering existing keys.
3. With `compaction_record` null, the "no compaction has run" line appears and the map height is unchanged compared with a non-null record.
4. Switching to the after view removes the removed blocks from the map and adds a digest block; switching back restores them.
5. Hovering a tile shows its label, kind, token count and sub-item list; moving the pointer off the map hides the card.
6. Clicking a tile holds its card open; a second click releases it.
7. Clicking free space calls `on_block_pick(null)`.
8. Changing `active_agent_id` through the menu calls `on_agent_change` with the new id and uses that agent's window size for the fill figure.
9. A tile too small for its label still has an accessible name containing its label.
10. At phone width the map keeps its height and no horizontal scrolling occurs.
</content>
