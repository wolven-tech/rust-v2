# Quick Action Palette

Source item: affectUI Pro `components/command-menu` (functional reference only; see README).

## Purpose
One search field that runs any action in the product: suggested actions to the tools the product connects to, general commands, and navigation. Typing narrows the list; a fallback runs the typed text as a question to the assistant.

## Data
| Field | Type | Meaning |
|---|---|---|
| `command_groups` | Group[] | `{ group_id, group_name, commands: { command_id, command_name, command_note?, command_mark?, key_combo?, match_terms? }[] }`. The group with id `suggested` gets quick-slot shortcuts 1 to 5 |
| `empty_hint` | string | Empty-field text |
| `on_command(command_id, typed_text?)` | callback | A command ran. Assistant fallback sends a reserved id with the typed text |
| `on_dismiss()` | callback | Escape pressed with an empty field; host closes its dialog |
| `focus_on_mount` | boolean | Focus the field on mount |
| `root_class` | string | Extra class on root |

## Anatomy
Reading order:
1. Search field with a leading search glyph.
2. Result list in groups, each group with a heading: Suggested, Shared, Utilities, Navigate (default names, overridable).
3. Each row: mark or icon, name, optional note, optional key combo on the right.
4. Footer row: key hints (move, jump section, open) and the last-run confirmation after a run.

## States
- **Empty field**: all groups shown.
- **Filtering**: groups with no match are hidden; matching text highlighted.
- **No matches**: a single "Ask the assistant" row with the typed text as its label is the only entry.
- **Active row**: one row highlighted.
- **Just ran**: the row presses briefly; footer shows the command name with a check.

## Interactions
- Up/Down: move row by row through all sections.
- Enter: run the active command.
- Tab / Shift+Tab: jump to next or previous section.
- Mod+1 to Mod+5 (Ctrl on Windows and Linux): run suggestion 1 to 5 immediately.
- Escape: clears the field if it has text; closes when empty (via `on_dismiss`).
- Mod+K or "/" when focus is outside the palette: focuses the field. (Host may map this globally.)
- Pointer hover moves the highlight; click runs the command.

## Motion (functional)
- Highlight glides between rows with a short, crisp move; fades when the pointer leaves the list.
- The run row gives a short press; the footer confirmation appears briefly.
- Reduced motion: immediate.

## Accessibility
- The search field is a combobox bound to the list; whichever row is highlighted is exposed as the active option.
- Groups are labelled; key combos are hidden from assistive tech and the full shortcut is in the row's accessible name.
- What just ran is announced politely.

## Responsive
- Fills its container up to a comfortable width.
- On phones key combos are hidden; rows remain tappable.
- No horizontal scroll at phone width.

## Composition hints
Dialog (host), text field (search), listbox with group headings and option rows, logo or icon tile, keycap or kbd text, live region helper.

## Acceptance checks
1. Typing narrows the list to commands whose name or match terms contain the text, and hides groups with no matches.
2. Pressing Enter runs the active command once and calls `on_command(id)` with that id.
3. Pressing Mod+3 runs the third suggestion immediately without moving the highlight first.
4. With text in the field and no matches, Enter calls `on_command` with the reserved assistant id and the typed text as `typed_text`.
5. Escape with text clears the field and does not call `on_dismiss`; Escape with an empty field calls `on_dismiss` once.
6. Tab moves the highlight to the first row of the next section.
7. Groups appear in the order given; the suggested group is first when provided.
8. Key combos are not announced by screen readers in the row names beyond the single full shortcut string.
9. Running a command announces its name through a polite live region.
10. At phone width the list and field fit and no horizontal scroll occurs.
