# Decision Queue

Source item: affectUI Pro `screens/inbox` (functional reference only; see README).

## Purpose
Lets a person answer, one item at a time and mostly from the keyboard, the questions an orchestrating agent is waiting on, so the agent can keep going without a change of screen.

## Data
- `request_list`: list of `{ id, request_kind: "decision" | "pick" | "approval" | "question", question_text, context_text?, task_name, asked_at, is_unread?, option_list?: [{ id, entry_title, summary?, is_recommended?, advantages?: string[], drawbacks?: string[], preview_text? }], fact_rows?: [{ entry_label, entry_value }], suggested_answers?: string[], answer_record?: { answer_text, answered_at, entry_label? } }`.
- `request_kind` meanings: decision (choose one option); pick (choose one variant, which may show a preview); approval (approve or decline, with supporting facts); question (free-text answer, with optional suggestions).
- `task_details`: a map from task name to `{ goal?, project_name?, task_state?, step_list?, artifact_names? }`, shown in the Task panel.
- `artifact_store`: a map from file name to a document, sheet, or folder artifact.
- `initial_tab`: `"open" | "done"`. `initial_request_id`: string or null.
- `running_count`: number of background tasks still running, shown as a footer count.
- `workspace_name`, `account`, `text_strings`, `is_loading`, `is_empty`.
- Callbacks: `on_answer(id, answer)`, where answer is `{ option_id?, free_text? }` or `"approve" | "decline"`; `on_later(id)`; `on_reopen(id)`; `on_select(id | null)`; `on_artifact_change(entry_name, artifact)`; `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. List column: title, Open and Done tabs with counts, and the rows. Each row shows the request kind, a short question, the task name, its age, and an unread dot.
3. Reading column (centre): the question as a heading, a context paragraph, then the answer area for the request kind:
   - decision or pick: option cards in a radio group; the recommended option is marked and pre-selected when the item opens; the selected card reveals its advantages and drawbacks.
   - approval: a fact list with Approve and Decline buttons.
   - question: a text area, with suggestion chips above it.
   Beneath the answer area: Later and Send buttons.
4. Task panel (right): the task's goal and state, a checklist of steps, the artifacts as tabs (document, sheet, folder), and a "+" tab that suggests artifacts to open. Artifacts use the shared artifact viewers.
5. Footer line: "N tasks still running", or a caught-up message when the open list is empty.

## States
- **open list, item selected**: the reading column shows the item's answer area.
- **done list**: rows are answered items; selecting one shows the answer that was given, with a Reopen action.
- **sending**: the send control shows a confirmation mark. Focus stays on the heading, then moves to the next item.
- **after send**: the answered item leaves the open list and the next item is selected.
- **option selected**: the card expands to show its advantages and drawbacks.
- **later**: the item is moved to the back of the open queue.
- **all caught up**: no open items; the reading column shows one sentence and a Go to Thread action.
- **empty** (`is_empty`): no items at all. The list header and tabs stay; the reading column shows the empty message; the Task panel states that nothing is open.
- **loading** (`is_loading`): placeholder rows, a placeholder question, and placeholder lines in the Task panel.

## Interactions
- J moves to the next item and K to the previous one, from anywhere on the screen, unless a field has focus.
- Up and down arrows (or Home and End) step through the items in the list; the reading column tracks the selection.
- 1, 2, 3, and so on: pick that option; arrow keys move between options in a radio group.
- Enter: sends. On an option, it picks and sends. On an approval, it approves. On a question, it sends the typed answer. Enter also works from inside the reply field.
- L: Later for the open item.
- R or /: focus the reply field (or the answer field on a question).
- Shift+Enter: new line in a question's answer.
- Escape: leaves the reply field. On phones, goes back from the item to the list.
- Left and right arrows: switch between Open and Done.
- Task panel tabs: arrow keys move between tabs; Delete closes the focused tab. Closing the last tab folds the panel and moves focus to its open control.
- Artifact sheet (spreadsheet-like table): arrows move the active cell; Tab and Enter move across and down; typing starts editing; F2 or a double-click opens an in-place editor; Enter or Tab commits it; Escape cancels; a leading equals sign starts a formula. Copy, cut, and paste use tab-separated text.
- Artifact document: standard inline formatting shortcuts (bold, italic, underline, link), list and checklist shortcuts, and Markdown-style line starts.
- Artifact folder: arrow navigation; Enter opens; type-ahead jumps to a name.
- The context menu key or Shift+F10 opens the context menu for the focused element.
- Shortcuts never fire while the person is typing in a field, except Enter and Escape.

## Motion (functional)
- Sending turns the answer control into a confirmation mark; then the row leaves the list and the next item comes in.
- The chosen option expands to show its advantages and drawbacks.
- The unread dot fades once the item has been seen.
- Menus and tips fade in. The spreadsheet selection moves without animation.
- Reduced motion: no sliding, folding, or drawing; the confirmation simply appears.

## Accessibility
- The list uses listbox semantics and a roving tab stop; each row is named in full: kind, question, task, time, and unread state.
- Option cards form a radio group; the chosen one is checked, and its advantages and drawbacks are read with it.
- After a send, focus moves predictably (see states), so it is never lost.
- A polite live region reports results, for example "Sent. 4 left." or "Later. Next: <question>."
- Each button advertises its shortcut through aria-keyshortcuts.
- The artifact toolbars form a toolbar with one tab stop and arrow-key movement. The sheet is a grid, with the active cell announced.
- Menus and dialogs close with Escape and return focus to their trigger.

## Responsive
- Fills its container; the host sets the height.
- Every row is the same height. The reading column is capped at a maximum width and centred.
- Below the phone breakpoint, the list fills the screen; opening an item shows it full width with a back control.
- At phone width the reply field drops to a line of its own; variant options stay three to a row; shortcut hints are hidden.
- Touch targets are enlarged for touch input.
- Long text is cut short with a tooltip holding the full text. Right-to-left text keeps its own direction.

## Composition hints
App frame; tabs with counts; listbox rows with badges; radio card group; approval card with a fact list; text area; suggestion chips; buttons; task panel with checklist and history; artifact viewers (document, sheet, folder); context menu; live region; tooltip; empty state; skeleton.

## Acceptance checks
1. J and K move the selection and update the reading column.
2. With an option item open, pressing 2 selects the second option; Enter then sends that option, and `on_answer` receives `{ option_id: <id> }`.
3. Enter on an approval calls `on_answer(id, "approve")`.
4. L moves the open item to the end of the open list and selects the next one.
5. After a send, focus moves to the next item's heading, and the live region reports how many items are left.
6. In the Done list, Reopen moves the item to the top of the open list.
7. While typing in the reply field, J, K, L, and 1 to 3 do not trigger their actions.
8. With no items, the caught-up or empty message appears, and its action navigates to Thread.
