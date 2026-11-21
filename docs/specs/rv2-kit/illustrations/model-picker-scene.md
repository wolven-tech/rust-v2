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
