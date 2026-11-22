# Parallel Edit Overlap Grid

Source item: affectUI Pro `components/conflict-radar` (functional reference only; see README).

## Purpose
Tracks several coding agents that are each editing their own branch of one repository at the same time. Rows are files, columns are contributors, and each cell records the line span that contributor changed in that file. Where two contributors changed overlapping lines, the row is flagged and the person chooses which version survives. Merge stays locked until every flagged row has a choice and no contributor is still running.

## Data
| Field | Type | Meaning |
|---|---|---|
| `contributors` | Contributor[] | `{ contributor_id, contributor_name, model_name?, branch_name?, contributor_mark?, is_active }`. One column each |
| `file_rows` | FileRow[] | `{ file_path, edits: { contributor_id, line_range: [start, end], is_editing?, edit_note? }[], conflict?: { options: { contributor_id, option_text }[] } }`. One row per file. Two or more contributors whose line spans overlap make the row a conflict |
| `section_title` | string | Defaults to "Parallel work" |
| `chosen_options` | Record<file_path, contributor_id> | Pick per conflicting file. When omitted, the component keeps picks itself |
| `on_choose_option(file_path, contributor_id)` | callback | A pick was made |
| `on_merge_request()` | callback | Merge activated. Enabled only when every conflict has a pick and no contributor is active |
| `root_class` | string | Extra class on the root element |

## Anatomy
Reading order:
1. Title bar: section title, a status word ("N still active" or "Ready to combine"), and the Merge control.
2. Column header row: contributor name, model or branch, mark, and whether it is still active.
3. File rows: the path (folder dimmed, file name emphasised), then a cell per contributor showing its line span and a bar. A conflicting row draws a connector between the cells involved.
4. Conflict detail, shown beneath a focused conflicting row: a radio group of options, one sentence each, each naming its contributor.
5. Footer: file count, open conflicts, and settled files per contributor.

## States
- **Contributors active:** bars grow in; a sheen moves across bars of files still being edited; the status word counts the remaining active contributors.
- **Conflict open:** the connector glows until the conflict is settled; the row carries a conflict tag.
- **Conflict settled:** the row names the kept contributor; the connector stops glowing.
- **All settled, none active:** Merge is enabled and the status word reads "Ready to combine".
- **No conflicts:** no connectors are drawn; Merge is enabled once no contributor is active.
- **Contributor hovered or focused:** its counts (files, open conflicts, kept) appear.

## Interactions
- Pointer: hovering a bar reveals which contributor changed which lines; hovering a column header shows that contributor's counts; clicking an option picks it.
- Tab order: Merge, then conflicting rows, then the options of the focused conflict.
- Enter or Space on a conflicting row: opens its options. Enter or Space on an option: keeps that option.
- Arrow keys inside the option group: move the radio selection.
- Escape: closes an open conflict detail and returns focus to its row.

## Motion (functional)
- Bars grow in from the left when a contributor first touches a file; a sheen runs while the file is being edited.
- A conflict draws a line between the involved bars, with a glow travelling along it until the conflict is settled.
- Settling a conflict stops the glow and marks the kept contributor.
- With reduced motion requested, glows and sheens are removed and state changes apply immediately.

## Accessibility
- The grid is a table. Contributors are column headers. Each cell states who changed the file, the line span, and whether that version was kept.
- Conflict and kept states are spelled out in text, never by colour alone.
- The options for a conflict form a radio group.
- The comparison detail sits in a polite region; focusing a conflict triggers an announcement of its state.

## Responsive
- On narrow widths, contributor names and folder prefixes are dropped, columns compress, and each option's sentence wraps beneath its name.
- The grid scrolls inside its own container when it does not fit. The page itself never scrolls sideways at phone width.

## Composition hints
Card, table (grid), progress or status text, bar (cell), connector line (SVG), radio group (options), button (Merge), badge (conflict, kept), live region helper.

## Acceptance checks
1. A file edited by two contributors over overlapping line spans is flagged as a conflict and shows a connector between those two cells.
2. Merge is disabled while any conflict is unsettled or any contributor is active, and enabled once both clear.
3. Choosing an option calls `on_choose_option(file_path, contributor_id)` once and marks that contributor as kept in its cell.
4. Hovering a bar shows the contributor's name and line span for that file.
5. The footer counts (files, open conflicts, kept per contributor) update when a conflict is settled.
6. Enter on a conflicting row reveals its options; Escape hides them and returns focus to the row.
7. Without overlapping edits, no connector is rendered.
8. Conflict state is conveyed by the word "Conflict" in text, not colour alone.
9. A contributor still active is labelled "Active" in its column header.
10. At phone width the grid scrolls inside its box and the page does not scroll horizontally.
