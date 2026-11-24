# Model Picker Scene

Source item: affectUI Pro `illustrations/model-menu` (functional reference only; see README).

## Purpose
A model picker for an AI product: a list of model names, each with its vendor mark, where the person picks one and a highlight glides to the chosen row. The list fades at its lower edge to suggest more models below.

## Data
- `groups`: list of { label, models: { id, name, vendor_mark: node (supplied by caller), available: boolean }[] }.
- `selected_id`, `on_select(id)`.
- `default_id`.
- `extra_classes`.

## Anatomy
In reading order:
1. Group heading (for example "Recommended").
2. Model rows: vendor mark, model name, and a check on the selected row.
3. Selection highlight: a band behind the selected row that moves between rows.
4. Fade region: the lower edge and trailing edge of the list fade into the page background.

## States
- Default: first model selected; highlight on that row.
- Hover: row shows a subtle highlight; the check does not move.
- Selected: check shown on the row; highlight rests on it.
- Unavailable model: row is shown with de-emphasised text and cannot be picked; the word "unavailable" is shown.
- Keyboard focus: the focused row shows a focus ring separate from the selection highlight.

## Interactions
- Click a row: selects it; the highlight glides to it; the list may close if it is a popover.
- Arrow Down and Up: move focus between rows; Home and End jump to the first and last.
- Enter or Space: selects the focused row.
- Escape: closes the picker without changing selection (when a popover).
- Outside click: closes the picker when it is a popover.

## Motion (functional)
- The selection highlight slides from the previous row to the new one in one movement.
- The check appears on the new row as the highlight arrives.
- The fade regions do not move; they reveal more rows as the list scrolls.
- With reduced motion preferred, the highlight jumps to the new row.

## Accessibility
- The list is a listbox with one selected option; the selected option is announced.
- Each row's name is its accessible name; the vendor mark is decorative.
- Unavailable rows are disabled options with a text note.
- Group headings are grouping labels for the listbox.

## Responsive
- Phone width: the picker fills the width; the fade regions stay on the bottom and trailing edges.

## Composition hints
- Listbox, option rows, group label, check indicator, image slot for vendor marks, popover container.

## Acceptance checks
1. Clicking a row moves the selection highlight to that row and shows the check there.
2. Arrow keys move focus between rows; Enter selects the focused row.
3. An unavailable row cannot be selected and reads "unavailable".
4. Escape closes the picker without changing the selection.
5. With reduced motion preferred, selection changes without intermediate highlight positions.
6. At phone width the picker fills the width and nothing scrolls sideways.

Uses partner logos (supplied by caller) for each model's vendor mark. Does not describe any vendor mark.

## Amendment 7: structure and demo

Replaces: none; no earlier amendment exists. Base checks 1 to 6 stand; checks 7 to 11 are new.

Archetype: one single-group list column, offset right of centre, with a faded tail.

Stage: the card, a light neutral surface, edge to edge. Width to height about 1.85 to 1.

List column: one panel with a light surface, a hairline edge and a soft shadow.
- Left edge at 35% of stage width. Its unfaded right edge at 77%. The right 10% of the panel fades into the stage.
- Top at 20% of stage height. The bottom fades out between 80% and 92% of stage height.

Group heading: one, at 25% of stage height, left-aligned with the row content, muted. Gallery text: "Fast models".

Rows: seven. Vertical centres at 32%, 40.5%, 49%, 57%, 65.5%, 74% and 82% of stage height. Each row spans the panel's full width and holds, left to right:
- a leading mark slot, 5% to 10% of panel width, with a placeholder glyph;
- the model name, starting at 13% of panel width;
- on the selected row only, a check glyph at 82% of panel width.

Placeholder glyphs: our own simple shapes, one per row (circle, diamond, triangle, square, star, hexagon, ring), in the ink token. Not brand marks.

Gallery names, top to bottom: "Flash 2", "Deep reason", "Large context", "Compact 3", "Vision pro", "Code mini", "Legacy v1".

Selected: row 2, "Deep reason". Its band is one shape spanning the panel's content width, in the accent token at low strength, with the check at 82%.

Faded tail: row 6 at about 60% opacity; row 7 at about 35%. Rows 1 to 5 at full strength. Rows 6 and 7 stay selectable. No row at rest carries the word "unavailable".

Settled frame: row 2 selected, band and check on it, rows 6 and 7 faded, bottom and right edges fading. Selecting another row moves the band in one movement; reduced motion jumps it.

Acceptance checks (continued):
7. The gallery list has one group heading and seven rows. Each row has a leading mark slot holding its own placeholder glyph.
8. At rest exactly one row (row 2) carries the band and the check.
9. Rows 6 and 7 are visibly fainter than rows 1 to 5, and row 7 is fainter than row 6. No row at rest reads "unavailable".
10. The list's left edge is at 35% of stage width and its unfaded right edge at 77%. The right fifth of the stage is empty.
11. The list has faded out before 92% of stage height, and the seventh row is not cut by the stage edge.
