# Kanban Task Board

Source item: affectUI Pro `blocks/task-board` (functional reference only; see README).

## Purpose
A board where tasks move between status columns. People drag or keyboard-move cards, add new tasks quickly, open a task's detail panel, search, filter by label, and switch to a compact view to fit more cards.

## Data
- `breadcrumb_segments`: list of breadcrumb strings.
- `status_columns`: ordered list of { id, label, icon? }.
- `work_items`: list of { id, title, status_id, priority (0 to 4), estimate?, label_ids: string[], assignee_id?, due_on?, subtasks: { done, total }, pull_request_ref?, created_at }.
- `tag_options`: list of { id, label, rank }.
- `member_options`: list of { id, name, avatar }.
- `on_items_update(work_items)`: called with the full list after each move, edit, add or delete.
- `initial_view`: "comfortable" | "compact".
- `extra_classes`.

## Anatomy
In reading order:
1. Breadcrumb and the board name.
2. Toolbar: task count, search field, filter control (labels), and a view toggle (comfortable or compact).
3. Columns side by side, each with its status name, its count, and a quick-add control.
4. Task cards in each column: title, identifier line, label chips, and a footer with status, due text, priority mark and assignee.
5. Detail panel: opens from the right over the board; holds the editable title and description, status and priority as radio groups, labels, assignee, and the subtask list.

## States
- Default: all tasks shown, comfortable view, no filter.
- Searching: cards not matching are hidden; column counts show matches.
- Label filter: only tasks with the chosen labels are shown.
- Compact view: cards show the title on one line; footer details move to a hover or focus tooltip.
- Dragging: a card lifts; a line shows the landing position; the target column is emphasised.
- Adding: a new card with an empty title is shown in the column with the field focused.
- Detail open: the card is marked selected; the panel holds focus.
- Overflow: a column with more than fifty cards shows a "show more" control at its end.

## Interactions
- Drag a card: moves it to another column or position; the drop line shows the landing point. Escape cancels a drag in progress.
- Quick add: type a title and press Enter to add to that column; Escape cancels and returns focus to the column's add control.
- Click a card: opens the detail panel.
- Panel: Escape closes it and returns focus to its card; status and priority are radio groups where Left and Right change the value and the card moves immediately.
- Labels: opening the label filter lists labels; Up and Down move; Enter toggles; Home and End jump to ends.
- Search: live filter; Escape clears text when it has content.
- Keyboard card move: focus a card, then Left and Right move it to the previous or next column; Up and Down move it within the column.
- Keyboard order: breadcrumb, toolbar controls, columns in order, cards in each column, panel when open.

## Motion (functional)
- On first view, columns and then their cards fade upward in sequence.
- A drag shows a thin line at the landing point; the target column strengthens its background while a card is over it.
- The detail panel slides in from the right.
- Moving a card between columns animates it into its new position; other cards shift to fill the gap.
- With reduced motion preferred, these are instant.

## Accessibility
- Each card is a button that opens its dialog panel; the panel is a labelled dialog.
- Status and priority controls are radio groups; the card move from keyboard is announced.
- Due-date text carries its own words ("today", "2 days late") so a visual cue is never the only signal.
- Column headings are headings; counts are part of the heading name.
- Toggles (star, compact view, filter) expose a pressed state.

## Responsive
- Wide: all columns visible; panel overlays the right side.
- Narrower: columns hold a floor width, and the board pans horizontally within its container.
- Phone width: the board scrolls horizontally inside its region; the detail panel is full screen; touch controls are larger; quick-add is always visible on each column.
- Touch: drag is supported with a hold to pick up; the status control in the panel is the alternate way to move a card.

## Composition hints
- Board (columns and cards), card, badge (label chips), toolbar with search input and segmented view toggle, popover list (labels), dialog or side panel (detail), radio group, checkbox list (subtasks), avatar.

## Acceptance checks
1. Dragging a card into another column moves it and calls `on_items_update` with the updated status.
2. Typing a title in quick add and pressing Enter creates a card in that column and focuses it.
3. Escape during a drag returns the card to its starting position without calling `on_items_update`.
4. Search hides non-matching cards and column counts update to match.
5. Left and Right on a focused card move it between columns from the keyboard.
6. The detail panel's status control changes the card's column when a new status is chosen.
7. Due-date text reads "today" or "late" with a day count, not a visual cue alone.
8. At phone width the board scrolls inside its own region and nothing else scrolls sideways.

## Amendment 5: structure and demo

Archetype: board with four status columns under a toolbar.

Visible at rest:
- Breadcrumb "Meridian / Storefront / Sprint 9" and the board name "Storefront".
- Toolbar: task count "11 tasks", search field, filter control, compact-view toggle.
- Four columns side by side, in order: Todo (3), In review (3), In progress (3), Done (2). Each heading shows the status name and count, with a quick-add control.
- Each card: identifier line ("ST-214 · Size M · Created 3 Oct"), title, one or two label chips, and a footer with age or due text, a subtask count where present, a pull-request number where present, a priority mark, and an assignee avatar.

Gallery demo, 11 cards:
- Todo: "Add the spring banner to the home page" (ST-214, Size M, label Web, 2 days); "Fix rounding on annual invoices" (ST-209, Payments, Bug); "Ask beta shops about the new tier" (ST-197, no label, subtasks 2/4, 2 days).
- In review: "Tighten the onboarding copy" (ST-221, Web, 1 day); "Cap stock alerts at one per day" (ST-205, Size L, Web, Feature); "Push opt-in on iOS" (ST-188, App, #412).
- In progress: "Write the help-centre export guide" (ST-230, Web); "Prorate seat changes mid-month" (ST-201, Payments, 4 days); "Offline mode for the task list" (ST-176, Size XL, App, #398).
- Done: "Dark mode for the dashboard" (ST-160, Web, Feature); "Crash on large avatar upload" (ST-171, App, Bug).

Motion, settled frame: all cards shown; none selected; no drag.
Moves: on first view the columns fade in one after another, then their cards. A drag shows a landing line, and a card moving to a new column animates into place.
Reduced motion: instant.

Acceptance checks (continued):
9. Columns read Todo, In review, In progress, Done, with counts 3, 3, 3, 2 that match the cards in each.
10. Every card shows an identifier line and an avatar; a card with no assignee shows a neutral avatar without initials.
11. A pull-request reference appears on its card as text, with its number.
