# Assistant Memory Manager

Source item: affectUI Pro `components/memory-panel` (functional reference only; see README).

## Purpose
Gives a person a view of what an assistant remembers about them, grouped by category, with the ability to edit, pause and delete entries. A master switch prevents the assistant from using or storing memories in new conversations. Deletions can be reversed. It fits either a settings page or a panel beside the chat.

## Data
- `memory_items` (Memory[]): each entry is `{ memory_id, memory_text, memory_group, source_label?, date_label?, is_paused? }`. Source and date are displayed exactly as supplied.
- `on_list_change(memories)`: called with the complete list after any change. When supplied, the panel displays exactly the memories passed in; otherwise changes remain internal to the panel.
- `on_add(memory)`: a memory was added.
- `on_text_edit(memory_id, text)`: a memory's text was changed and saved.
- `on_pause_change(memory_id, is_paused)`.
- `on_forget(memory_id)`: a memory was removed; it can come back through Undo.
- `on_restore(memory)`: a removed memory was reinstated at its earlier position.
- `use_memory_on` (boolean, controlled), `initial_use_on` (boolean, default true), and `on_use_change(enabled)`.
- `group_order` (text[]): order of groups and of choices for new memories. Defaults to the order in which categories first appear.
- `heading_text` (text, default "Memory").
- `list_max_height` (number, a moderate fixed height): the list scrolls once it exceeds this.
- `undo_window` (duration, default five seconds): how long Undo remains available after a deletion.
- `added_source_label` (text, default "You added"): source given to memories created inside the panel; their date is today.
- `footer_action` (`{ action_label, on_action }`, optional): a button in the last row. Undo takes its place briefly after a deletion.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Header: heading, plus the master switch "Use memory" with a note beneath it that expands when the switch is off.
2. Search field and group chips: All, followed by each group with its count.
3. List: groups in order, each with a label and its memories. Each memory shows its text, source and date, a paused tag where relevant, and actions (Edit, Pause, Forget).
4. New memory row: opens an editor with text, a group choice, Save and Cancel.
5. Footer row: a count summary ("8 memories · 1 paused") and the status line, or the Undo action.

## States
- Use memory on: the list is interactive.
- Use memory off: the list is dimmed and the note is shown; memories are not used by the assistant but remain listed.
- Filtered: groups that do not match are hidden; the chip shows the selected group.
- Editing a memory: a text field with Save and Cancel replaces the text.
- Paused memory: text in muted emphasis; a paused tag is shown.
- Forgotten: the row folds away; the footer shows Undo for the undo window.
- Empty group: "No memories here". (Spec choice: an empty list shows the same message.)

## Interactions
Pointer:
- Toggle the master switch.
- Click a chip to filter to that group.
- Click Edit to edit in place. Save commits; Cancel discards.
- Click Pause to toggle the paused state.
- Click Forget to remove the row and show Undo.
- Click New memory to open the editor.

Keyboard:
- Tab order: the master switch, the chosen chip, the search box, then per-memory Edit, Pause and Forget controls, then New memory and the footer action.
- Left / Right: move between group chips (and between groups inside the new-memory editor).
- Enter: save an edited or new memory. Shift+Enter inserts a line break.
- Escape: cancel editing, close new memory, or close the search field.

## Motion (functional)
- Filtering folds non-matching rows closed and fades them.
- Deleting collapses the row; Undo expands it back into its prior slot and brings it on screen.
- Pausing mutes the wording and fades in a paused badge.
- Counts roll to new values; status text cross-fades.
- Switching memory off dims the list and reveals the note; the New memory button widens into the editor where it sat.
- Reduced motion: all of this is instant.

## Accessibility
- The panel is a group named by its heading. The master control is a switch with a checked state; it controls the note.
- Group chips form a radio group with a single tab stop; arrow keys move between them.
- Every memory group is a named region.
- Each action button is named after its memory, for example "Forget <memory text>". Actions show on keyboard focus.
- Hidden or collapsing rows accept no input, keeping focus off invisible items.
- Counts, forgotten memories with Undo, and editing hints are announced politely.

## Responsive
- Fills its container up to a comfortable reading width; remains usable at very narrow widths.
- When there is little room: chips drop their counts. At the narrowest width: the switch keeps only its track.
- Group chips scroll sideways when they do not fit, with an edge fade while more are hidden.
- Touch: actions are always visible, with larger hit areas.

## Composition hints
Panel, header with switch, search input, chip group (radio), section lists, list rows with tag and icon buttons, text editor, footer with live status and action button, live region.

## Acceptance checks
1. With eight memories in three groups, the "All" chip shows 8, and the summary reads "8 memories · 1 paused" when one is paused.
2. Turning the master switch off dims the list, shows the note, and calls `on_use_change(false)`; the memories remain in the list.
3. Clicking a group chip shows only that group's memories; other groups fold closed.
4. Editing a memory and pressing Enter calls `on_text_edit` with the new text; Escape cancels without a call.
5. Forgetting a memory removes it from view, calls `on_forget`, and shows Undo for the `undo_window`.
6. Activating Undo within the window calls `on_restore` and returns the memory to its original position.
7. A memory added through New memory carries `added_source_label` as its source and today's date.
8. Pausing a memory calls `on_pause_change(id, true)`; it remains in the list and in the count.
9. At the narrowest width the master switch shows only its track.
10. Reduced motion: no fold or blur transitions.
</content>
