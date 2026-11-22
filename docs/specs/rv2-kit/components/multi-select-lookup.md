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

## Amendment: structure and demo

Layout archetype: a vertical stack at field width. A caption sits above a bordered field that holds the chips and the text input. Directly under the field, the list stays open as a popover with grouped rows and a foot bar. The list is shown open in the gallery so that the grouped rows are visible at rest.

Parts that must exist:
- Caption: one line above the field, naming the field and the list.
- Field: a bordered box holding the chips, followed by the text input on the same line or the next line when chips wrap.
- Popover list: same width as the field. Holds group headings in order, then rows. Scrolls inside a fixed height when it has more rows than fit.
- Row: initials avatar at the leading edge, the option label, and a role or hint line beneath it. A check mark sits at the trailing edge of a picked row.
- Foot bar: the selected count at the leading edge, and at the trailing edge a key-hint legend naming the move key and the toggle key.
- Create row: appears at the end of the list only when `allow_create` is set and the typed text matches nothing.

Demo data for the gallery (our own names and values):
- Caption "Approvers".
- Two chips picked, drawn from different groups.
- Three groups, "Platform", "Support" and "Operations", holding three, two and two people, with an initials avatar and a role line on each.
- Two rows picked, each with a check mark.
- Foot bar reads the selected count (two) with the key-hint legend.
- No row highlighted, no typed text, no create row.

States visible at rest in the gallery: the list open under the field, group headings, picked rows with check marks, foot bar with count and key hints. No row is active and the no-match text is not shown.

Acceptance checks:
11. At rest the gallery demo shows the caption, the field with two chips, and the open list directly beneath it, all at the field's width.
12. The list shows three group headings in the order given, with the rows of each group beneath its heading.
13. Each row shows an initials avatar, the label, and a role line; the two picked rows show a check mark at the trailing edge.
14. The foot bar shows the selected count and the move and toggle key hints in the gallery demo.
15. The list has a fixed maximum height and scrolls internally when it holds more rows than fit (SSR test on the style hook, plus a gallery render with the seventh row reachable by scroll).
16. No row is highlighted at rest in the gallery demo.
17. The create row is absent from the gallery demo and appears only when typed text matches no option (SSR test on both).
</content>
