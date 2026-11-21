# Multi-Select Lookup

Source item: affectUI Pro `components/combobox` (functional reference only; see README).

## Purpose
Lets a person pick one or several values from a long, grouped, searchable list, and optionally create a value that is not in the list. Selected values appear as removable chips in the field.

## Data
| Field | Type | Meaning |
|---|---|---|
| `option_list` | Option[] | `{ option_value, option_label, option_hint?, option_image?, option_swatch?, option_mark?, option_group?, match_terms? }`. Options sharing an `option_group` appear under one heading. `option_image` is an image URL, or `true` to show initials. |
| `allow_many` | boolean | Defaults to true. Single mode closes the list after a pick. |
| `selected_values` | string[] | Controlled selection (at most one entry in single mode). |
| `initial_selection` | string[] | Uncontrolled starting value. Defaults to empty. |
| `on_selection_change(values, options)` | callback | Called on every pick, removal or creation. |
| `empty_hint` | string | Shown while nothing is picked. |
| `allow_create` | boolean | Offers a create row when the typed text matches no option exactly. |
| `build_new_option(text)` | callback returning option | Builds the new option. Default `{ option_value: text, option_label: text }`. |
| `create_row_text(text)` | function | Text of the create row. |
| `max_chip_count` | number | Upper limit on visible chips; the rest collapse into a "+N" chip. |
| `field_caption` | string | Visible label; also names the list. |
| `start_open` | boolean | Open at mount. |
| `no_match_text` | string | Shown when nothing matches. |
| `root_class` | string | Extra class applied to the root. |

## Anatomy
Reading order:
1. Caption (optional).
2. Field: chips of the selected values, then the text input. Single mode shows the selected label in the field instead of chips.
3. "+N" chip when chips are limited.
4. Popover list (when open): optional group headings, rows with image or swatch, label, hint, and a check for picked rows; a create row at the end when applicable; a count row at the foot.

## States
- **Closed, empty:** empty hint.
- **Closed, with picks:** chips or the single label.
- **Open, filtering:** rows filtered by the typed text; matching text is highlighted in each row.
- **Open, no matches:** no-match text (and the create row when `allow_create` is set).
- **Highlighted row:** one row active, following the keyboard or pointer.
- **Chip about to be removed:** after Backspace in an empty field, the last chip is highlighted; a second Backspace removes it.
- **Overflow:** some chips hidden behind "+N".

## Interactions
- Typing opens the list and filters it.
- Up/Down: opens the list if it is closed, then moves the active row.
- Enter: picks or unpicks the active row, or creates when the create row is active.
- Backspace on an empty input: selects the trailing chip; a second press deletes it.
- Escape: closes the list; when already closed, clears the typed text.
- Tab: closes the list and moves on.
- Pointer: clicking a row picks it without moving focus out of the input; clicking a chip's remove control removes it.

## Motion (functional)
- Chips appear with a small grow-in; removal shows a short flash before folding away.
- The active-row highlight moves between rows and fades when the pointer leaves the list.
- Check marks appear on pick.
- The list resizes smoothly while filtering.
- Reduced motion: immediate.

## Accessibility
- Implements the combobox pattern: the input exposes combobox role, expansion state and active-descendant linkage; the list is a grouped listbox.
- Picked rows carry a selected state; the listbox is multi-selectable in multiple mode.
- Picks, removals and creations are announced politely along with the new count.
- Focus stays in the input while interacting with rows and chip controls.

## Responsive
- The list matches the field's width.
- Chips wrap onto new lines; long names truncate.
- Touch: larger row and chip targets; input text large enough to avoid zoom on focus.
- Works at phone width.

## Composition hints
Input (text field), popover (list container), listbox and option rows, avatar, badge or chip (selected values), checkbox-style check mark, button (chip remove), live region helper.

## Acceptance checks
1. Typing "ma" filters options to those whose label or match terms contain "ma", and highlights the matched part in each row.
2. Enter on an active row adds it to `selected_values` and calls `on_selection_change` with the new list and option objects.
3. Enter on an already-picked row removes it from `selected_values`.
4. With `allow_create` and text matching no option, the create row appears; choosing it calls `build_new_option(text)` and adds the returned option.
5. Backspace in an empty field highlights the last chip; a second Backspace removes it and calls `on_selection_change`.
6. In single mode, picking an option closes the list and leaves exactly one value.
7. With `max_chip_count` set below the number of picks, the "+N" chip shows the count of hidden picks.
8. Escape while open closes the list; Escape while closed clears the typed text.
9. Options in the same `option_group` appear under one heading, in the order given.
10. At phone width, chips wrap and labels truncate without horizontal scrolling.
</content>
