# Agent Memory Manager

Source item: affectUI Pro `screens/agent-memory` (functional reference only; see README).

## Purpose
Lets a person inspect what their agent has retained about them and their work, trace where each retained item came from and where it was applied, and then correct, scope, confirm, or delete it.

## Data
- `memory_records`: list of `{ id, text, project_id?, source_info: { source_kind: "conversation" | "decision" | "user" | "file", occurred_at, quote_text?, question_text?, file_name? }, use_count?, last_used_at?, usage_log?: [{ task_name, how, occurred_at }], is_pending_review?: boolean }`.
- `project_names`: list of project names. Their order sets the order of the groups. Memories without a project fall into the "About you" group, which applies everywhere.
- `initial_tab`: `"saved" | "review"`. `initial_memory_id`: string or null.
- `inbox_count`: number shown in the sidebar.
- `text_strings`, `account`, `workspace_name`, `is_loading`, `is_empty`.
- Callbacks: `on_select(id)`, `on_create(memory)`, `on_change(memory)` (fired when text or scope is edited, or a keep is reverted), `on_forget(id)`, `on_restore(memory, index)`, `on_keep(id)`, `on_open_source(memory)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Header: title, tabs for Saved and To review with counts, a search field, and a Remember button.
3. List: a tree. Group headings (About you first, then one per project) show their count and can collapse or expand. Rows show the memory text, source label, use count, and last-used time in fixed columns.
4. Memory panel (right) for the selected memory: editable text, an Applies-to menu (Everywhere or a single project), a source block (quote, decision question with its answer, or file name) with an Open action, a usage list, and a Forget button. Under To review, Keep and Don't keep buttons take the place of Forget.
5. Remember line: an inline single-line field that adds a memory at the top of About you with source "user".

## States
- **Saved tab**: every confirmed memory.
- **To review tab**: memories flagged `is_pending_review`. Each offers Keep and Don't keep.
- **selected**: the panel shows the memory.
- **editing**: the panel text is editable; the previous text is kept until saved.
- **recalled**: when `last_used_at` moves to a later time, the row briefly flags that it was just used, and its count and time update.
- **forgotten**: the row collapses out of the list; Undo restores it at its earlier index for a short period.
- **remember line open**: the field has focus; Enter saves and clears the field for the next entry.
- **loading** (`is_loading`): placeholder group headings and rows; placeholder lines in the panel.
- **empty, Saved tab** (`is_empty`): a note explaining where memories come from, with a Remember button.
- **empty, To review tab**: a note saying nothing needs review.
- **empty panel**: no memory open.

## Interactions
- J and K: next and previous row, unless a field has focus.
- Up and down arrows, Home and End travel through group headings and rows; the whole tree occupies one stop in the tab order.
- Enter or Space on a row loads that memory into the panel; on a group heading, it toggles the group's visibility.
- Left and right arrows on a heading: collapse or expand; right on an expanded heading steps into it. Left on a row moves up to its heading.
- Delete or Backspace on a row: forgets it. Under To review it means Don't keep. Undo restores it.
- "/" focuses search. Escape clears it.
- N opens the remember line. Enter saves; Escape closes it.
- In the panel text: Enter saves the edit; Escape restores the previous text.
- Applies-to menu: chooses Everywhere or a project; the chosen option is checked.
- Opening a source calls `on_open_source`. Without it, the thread screen opens, or the Inbox for a decision.

## Motion (functional)
- A recall lights the row for a moment: a faint sweep crosses it, the icon flickers brighter, and the count and timestamp update to "now".
- Newly saved memories slide into the first position of their group.
- A forgotten memory folds away.
- Reduced motion: no sweeps or gliding; counts and times change directly.

## Accessibility
- The list is a tree: group headings are top-level items with an expanded state and a count; rows are child items.
- Each row is announced in full: text, source, use count, and last use. Supporting columns stay out of the spoken output.
- Keep and Don't keep are pointer shortcuts; keyboard users reach the same actions through Enter and then the panel buttons, or through Delete.
- A polite live region announces "Forgotten: <text>" and "Kept: it will be used from now on".
- Text fields show no focus ring; the caret and a firmer border indicate focus.
- Long text truncates with a tooltip; right-to-left text keeps its direction.

## Responsive
- Fills its container; the host sets the height.
- Rows have one fixed height. Source, use count, and time sit in fixed columns that line up.
- Narrow: the source column shrinks, rows wrap to two lines, and a row opens its memory in a sheet from the right.
- Narrower: search collapses to an icon while idle, and the To review label is shortened.
- Counts keep their column at any size, using compact number formatting for large values.
- Touch targets are enlarged for touch input; shortcut hints are hidden.

## Composition hints
Sidebar; tabs with counts; tree view with group headings and rows; source badge; count chip; inline text field; editable text panel; menu (Applies to); button group; undo notice; skeleton; empty state; live region; tooltip.

## Acceptance checks
1. Group headings show the project name and the number of memories in it.
2. Delete on a row removes it, and the undo notice restores it at the same index.
3. On To review, Keep moves the memory to Saved and calls `on_keep`.
4. Editing the text and pressing Escape restores the previous text; pressing Enter saves and calls `on_change`.
5. Pressing N, typing, and pressing Enter creates a memory with source "user" at the top of About you and calls `on_create`.
6. When `last_used_at` changes on a row, that row shows the recall highlight once.
7. Arrow keys move through the tree, and each heading reports whether it is expanded or collapsed.
8. With `is_empty` set on each tab, the matching empty note appears.
