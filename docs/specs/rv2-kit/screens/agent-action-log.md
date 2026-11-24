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

## Amendment 8: structure and demo

Replaces: nothing; no earlier amendment exists. Makes Anatomy items 3 and 4 (list and panel contents) concrete. The gallery demo is populated at rest; the empty state is not shown.

**Archetype:** a two-pane screen: a day-grouped activity list on the left, about two-thirds of the content width, and a step panel on the right, about one-third, with a single open-step tab strip above the panel.

**List pane** (fractions of the content area to the right of the sidebar):
- Header line, one row: the title "Activity" at left; a segmented filter about 0.44 of the list pane's width, starting about 0.07 in from the list's left edge; a search field about 0.29 of the list pane's width, at the list's right end.
- Filter segments, left to right: All 9, Automatic 4, Approved 5, Undone 1. The count follows each label. "All" is selected at rest.
- Day groups, newest first: "Today" with count 6, then "Yesterday" with count 3. A day heading is a band across the full list width: label at left, count in a small pill.
- Nine rows at rest: six under Today, three under Yesterday.
- The list is not stretched. Below the last row (Yesterday, 09:42) the pane stays empty to its bottom edge.

**Row layout**, left to right, two lines per row:
- Time column, about 0.05 of list width. Times descend within each group.
- App mark: a small logo or a neutral letter tile when the app has no logo.
- Sentence on line one, one line, truncated; starts about 0.13 of list width and ends about 0.84.
- Line two: an actor dot and actor word ("Automatic" or "You approved") and the task name, in muted text.
- Trailing slot, about 0.86 to 0.99 of list width, fixed column: an Undo button, or a "Final" label with a lock glyph, or an "Undone" tag.

**Rows at rest** (ours):
Today
1. 16:30, mail, "Sent the status email to Dana Ortiz", You approved, task "Weekly update". Undo. Selected.
2. 16:04, billing, "Issued a refund of 38 to Mara Lind", Automatic, task "Billing". Final.
3. 15:38, chat, "Posted the staging link in the team channel", Automatic, task "Release notes". Undo.
4. 15:07, code, "Opened pull request 482 to retry failed webhooks", Automatic, task "Webhook retries". Undo.
5. 13:42, calendar, "Booked a 30-minute call with Ben Ortega", You approved, task "Onboarding". Undo.
6. 12:42, docs, "Created the folder for Acme Q4 in Clients", Automatic, task "Ingest pipeline". Undo.
Yesterday
7. 14:42, code, "Merged pull request 479 into main", You approved, task "API migration". Final.
8. 11:42, mail, "Sent a payment reminder to Lumen Studio", You approved, task "Billing". Undone.
9. 09:42, chat, "Asked Ben Ortega for the staging keys", You approved, task "Billing". Final.

Counts at rest: 5 rows show Undo, 3 show Final, 1 shows Undone.

**Undone row (row 8):** the sentence is struck through across its full length, in muted text. Its time, app mark and line two stay as on other rows. The trailing slot holds a neutral "Undone" tag and no Undo button. It is not selected and sits in the Yesterday group, between the two Final rows.

**Undo control:** a small outlined button with a return-arrow glyph and the word "Undo", at the right end of the trailing slot. Each undoable row carries its own.

**Selected row:** row 1 carries a filled background across the full list width at rest. Its step opens in the panel.

**Panel** (fractions of the content area; about 0.33 of width, with a small gap from the list):
- Tab strip at the top: one open-step tab with the step's short title, truncated, and a close control; a pane toggle at the far right.
- Header block: app mark, the full sentence (may wrap to two lines), and a second line with the app name and the account.
- Details table, three key-value rows: When ("Today 16:30" and "12 minutes ago"), Task (the task name with an arrow button), Cost ("0.04 EUR").
- Content block, heading "Content": To, Subject, then a body preview. Up to six lines show; a "Show all" control appears only when the body runs longer than six lines.
- Authority block, heading "Authority": an approval line with a dot, "You approved" and "approved 14 minutes ago"; then a rule row with the rule name ("Client email") and a "Permissions" link with an arrow at its right.
- Reversal block, heading "Reversal": a state line ("Recall the email if it is still unread"), a second line with the time left ("Possible for 18 more minutes"), and the Undo button at the block's right. For a final step, the block states the reason and has no button. For an undone step, it states when the step was undone.
- Below the Reversal block the panel stays empty.

**Motion, settled frame:** nothing moves. The struck-through row is already struck through; no animation runs at rest.

**Acceptance checks (continued):**
11. The Today group holds six rows and Yesterday holds three. Each group's heading shows its count, and rows run newest first within each group.
12. The filter segments read All 9, Automatic 4, Approved 5 and Undone 1, with All selected.
13. At rest five rows show an Undo button, three show "Final" with a lock glyph, and one (Lumen Studio reminder, Yesterday) shows Undone: struck-through sentence, muted, neutral tag, no button.
14. Each Undo button sits in its row's trailing slot, at the right end. The trailing column is fixed, so Undo, Final and Undone align across rows.
15. Row 1 is selected at rest, with a filled background across the list width, and its step shows in the panel.
16. The panel's tab strip holds one open-step tab with a close control and a pane toggle at the far right.
17. The panel sections appear in this order: header, details table (When, Task, Cost), Content, Authority, Reversal.
18. The list pane is about 0.66 of content width and the panel about 0.33. The header line holds the title, the filter and the search on one line.
19. The list does not stretch to fill the pane. The space below the last row is empty.
