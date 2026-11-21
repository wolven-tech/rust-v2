# Sheet Grid

Source item: affectUI Pro `components/data-table` (functional reference only; see README).

## Purpose
An editable, spreadsheet-style table for small datasets that people keep up by hand: tools, vendors, budgets, contacts. Users select cells and ranges, edit in place, move data in and out of spreadsheets through the clipboard, undo mistakes, and sort.

## Data
- `grid_heading` (text): panel heading; also the grid's accessible name.
- `grid_heading_icon` (node, optional).
- `grid_columns` (Column[]): each column has
  - `column_key` (text)
  - `column_label` (text)
  - `column_icon` (node, optional)
  - `column_width` (number, optional): a width hint; a column never shrinks below its minimum
  - `column_kind` (`"title" | "tag" | "link" | "number" | "text"`)
  - `cells_editable` (boolean, optional, default true)
  - `tag_choices` (text[], for tag columns; a fixed set of choices)
  - `number_format` (for number columns; default groups digits and uses a true minus sign)
- `grid_rows` (Row[]): starting rows. Each row has `row_id`, an optional `row_logo` (node or URL), and one value per column key.
- `tag_tones` (map of tag text to tone): tones are `info | brand | highlight | positive | caution | muted`.
- `read_only` (boolean, default false): when true, selection and copy remain but editing, paste, delete and add-row are removed.
- `on_rows_change(rows)`: receives every row after any edit, paste, clear, delete, undo or redo.
- `on_checked_change(row_ids)`: receives the ids of the checked rows.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Panel header: heading icon and heading.
2. Column header row. Every header is a sort control with a direction indicator. Rows open with a checkbox column.
3. Body: one row per record. Tag cells show chips; link cells show their target text; number cells show grouped digits; title cells may carry a logo ahead of the text.
4. Footer action: an "Add row" control.
5. The active cell and the selected range appear as outlines drawn over the cells.

## States
- Idle: no cell active.
- Active cell: one cell shows the focus outline.
- Range selection: a rectangle of cells is selected; the anchor is the active cell.
- Editing: the active cell hosts an input. The prior value is retained until the edit is committed.
- Read-only: cells can be selected and copied but not changed. Add-row is hidden.
- Sorted: one column shows an ascending or descending indicator.
- Checked rows: the header checkbox reads checked, unchecked, or mixed when only some rows are checked.
- Empty: no rows; the body holds only the add-row control. (Spec choice.)

## Interactions
Pointer:
- Click a cell: makes it active. Drag across cells: selects a range.
- Double-click a cell: begins editing.
- Click a column header: sorts by that column; repeated clicks flip the direction.
- Click a row checkbox: toggles that row. The header checkbox toggles every row.
- Click "Add row": appends an empty row and activates its first cell.
- Click a tag cell: opens a picker listing that column's choices.

Keyboard:
- Arrow keys: move the active cell.
- Shift with an arrow: extends the selected range.
- Ctrl or Cmd with an arrow: leaps to the edge of the grid in that direction.
- Tab and Shift+Tab: next or previous cell.
- Enter or F2: edits the active cell. Enter saves and drops down one row.
- Any printable character on an active cell: begins editing with that character.
- Escape: cancels an edit, or collapses a range to its anchor.
- Delete or Backspace: clears the selected cells.
- Ctrl or Cmd with C, X or V: copy, cut, paste. Copy writes tab-separated text with one line per row, which spreadsheet applications accept.
- Ctrl or Cmd with Z: undo. Adding Shift gives redo.
- Ctrl or Cmd with A: selects every cell.

## Motion (functional)
- A tag picker opens from its cell with a short drop-in.
- The sort indicator turns to the new direction.
- Checkboxes confirm briefly when toggled.
- With reduced motion requested, all of the above is removed.

## Accessibility
- The table is a grid. The active cell and the selected range are exposed so a screen reader tracks both.
- Column headers are buttons that report their sort direction.
- Each row checkbox is labelled with its row; the header checkbox reports the mixed state.
- Changes that are not evident from focus, such as the row count after a delete, are announced politely.

## Responsive
- On narrow screens the panel scrolls horizontally inside itself. Columns keep their minimum width.
- On touch screens, row checkboxes have at least a comfortable touch target.
- A visible edge signals that more content lies off to the side.

## Composition hints
Panel, icon, grid primitive with a cell focus model, sort button in the header, checkbox, chip, popover for choice picking, text input for cell editing, avatar or logo, toolbar with add-row, undo stack.

## Performance and limits
- Designed for up to a few hundred rows. Each edit is a single state update.
- The undo history keeps the most recent 100 steps.

## Acceptance checks
1. Dragging from A1 to C3 selects a 3x3 range; Shift with an arrow grows it by one row.
2. Pressing a printable key on an active cell starts editing with that key as the first character.
3. Enter in an edit commits it and moves the active cell down one row.
4. Copying a selected range and pasting into a plain text field yields tab-separated values, one line per row.
5. Pasting tab-separated text into an empty range fills cells beginning at the active cell.
6. Undo after a delete restores the cleared values; redo reapplies them.
7. Clicking a column header sorts ascending, then descending on the next click.
8. The header checkbox shows the mixed state when some, but not all, rows are checked.
9. With `read_only: true`, double-click does not start editing and Add row is absent, but copy still works.
10. `on_rows_change` fires once per edit, paste, clear, delete, undo and redo, with all rows.
