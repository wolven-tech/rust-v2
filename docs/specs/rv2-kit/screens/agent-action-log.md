# Agent Action Log

Source item: affectUI Pro `screens/agent-activity` (functional reference only; see README).

## Purpose
Lets a person read, as plain sentences, each step an agent took inside the apps it is connected to. The screen shows who authorised each step and lets the person reverse the steps that are still reversible.

## Data
- `app_list`: list of `{ id, entry_name, account?, mark? }`. `mark` is the app's own logo; without one, the screen shows a neutral letter tile.
- `step_list`: list of `{ id, occurred_at, app_id, sentence, task_name?, actor: "automatic" | "approved", rule_label?, approved_at?, detail_rows: [{ entry_label, entry_value }], preview_lines?, undo_window?: { available_until, entry_description }, final_reason?, is_undone?, undone_at?, cost_label? }`.
- `selected_step_id`: string or null. The step the detail panel shows on start.
- `filter_value`: `"all" | "automatic" | "approved" | "undone"`.
- `text_strings`: all visible copy, replaceable.
- `is_loading`, `is_empty`: booleans.
- Callbacks: `on_undo(step)`, `on_open_task(task_name)`, `on_select(step_id)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame), with the activity entry active.
2. Header: title, a segmented filter control with a count per filter, and a search field.
3. Step list, grouped under day headings, newest first. Each row, left to right: time, app mark, the sentence (one line, truncated, with the full text in the panel and on hover), the task name with an arrow button, the actor label, and a trailing slot that holds either the Undo button, a lock mark for final steps, or an "Undone" tag.
4. Detail panel (right) for the selected step: the full sentence, time, app, task, actor, a "why it could" block (the rule it ran under, or when it was approved, with a link to permissions), a detail table, a preview (first six lines, then "Show all"), and an undo block. The undo block either shows the undo deadline and what Undo will do, or the reason the step cannot be reversed.
5. Empty state: a sentence describing where steps will appear, and a button that navigates to permissions.

## States
- **loading** (`is_loading`): placeholder rows under placeholder day headings; placeholder lines in the panel.
- **populated**: the list plus a selected step. The first step is selected unless `selected_step_id` overrides it.
- **filtered**: the list shows only matching steps. The selected step remains open even if it has dropped out of the list, until the person opens another step.
- **searching**: the list narrows by sentence and task name. Clearing the query restores it.
- **undoable**: Undo is visible and enabled until `available_until`.
- **undoing**: the row shows "Undone" right away (optimistic). If the host sets `is_undone` back to false, the row reverts to its earlier state and an error line appears in the panel.
- **undone**: sentence struck through, "Undone" tag, no Undo button.
- **expired**: Undo is removed; the panel explains that the window for reversing the step has closed.
- **final**: no Undo button; a lock mark on the row; the panel shows `final_reason`.
- **empty** (`is_empty`): the list is replaced by the empty message; the panel shows nothing open.

## Interactions
- Activating a row opens that step in the panel.
- Activating the task arrow calls `on_open_task`. Without it, the task screen opens.
- Activating Undo on a row or in the panel reverses the step (see states).
- Activating the "why it could" link navigates to Permissions.
- Keyboard, with the list focused: next and previous row (J/K or arrow keys); Enter opens; U reverses the focused or open step when that is possible; "/" focuses search; Escape in search clears it and returns focus to the list.
- Left and right arrows on the filter control move between filters.
- Narrow screens: choosing a row opens the detail full width with a back control. Escape or back returns to the list.
- Shortcuts are suppressed while a text field has focus.

## Motion (functional)
- Reversing a step: the sentence is struck through from its start, and the status changes to "Undone" where the button was. This confirms the change took effect.
- New steps arrive at the top of their day group.
- Reduced motion: state changes happen without animation.

## Accessibility
- Each row is a button whose name combines time, sentence, app, task, actor, and undo state.
- Undo is a separate button whose name includes the step sentence.
- A polite live region announces "Undone: <sentence>" after a reversal, and reports failures.
- The actor is stated in words, not only by a mark.
- Focus indicator appears only for keyboard focus.
- Filters use a radio-group or tab pattern that supports arrow keys.

## Responsive
- Fills its container; the host sets the height.
- Narrow: the time column shrinks, the sentence wraps to two lines, Undo becomes an icon button with an accessible name, and search collapses to an icon until activated.
- Long sentences truncate with an ellipsis and a tooltip; long previews are capped behind "Show all".
- Touch targets are enlarged for touch input.
- Each row supports right-to-left text.

## Composition hints
App frame with sidebar; segmented control (filters); search field; day-grouped list; list row with app mark, badge, and icon button; side detail panel (sheet on narrow screens); key-value detail table; preview block with expand; skeleton; empty state; live region; tooltip.

## Acceptance checks
1. Rows are grouped under day headings, newest first.
2. Activating Undo on an undoable row shows "Undone" at once and calls `on_undo` with that step.
3. If the host then sets `is_undone` back to false, the row reverts to its earlier state and an error appears.
4. Once `available_until` has passed, the Undo control is gone and the panel explains why.
5. The "Undone" filter lists only undone steps; left and right arrows cycle through the filters.
6. Pressing "/" focuses search; Escape clears the query and returns focus to the list.
7. After an undo, the live region announces "Undone: <sentence>".
8. With reduced motion requested, no strike-through animation plays; the final state is visible at once.
9. With `is_empty` set, the empty message appears with a button that navigates to Permissions.
10. At phone width the page does not scroll horizontally, and Undo stays reachable.
