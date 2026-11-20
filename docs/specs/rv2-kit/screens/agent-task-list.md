# Agent Task List

Source item: affectUI Pro `screens/agent-tasks` (functional reference only; see README).

## Purpose
Shows the work an orchestrating agent carries out as tasks instead of as individual agents: where each task stands, what it has produced, what has happened so far, and which tasks are waiting for a person. Opening a task reveals its details and leads to the decision it needs.

## Data
- `task_records`: list of `{ id, entry_title, project_name, progress_state: "planning" | "running" | "paused", status_phrase?, step_records: [{ entry_label, is_done }], artifact_names?: string[], pending_asks: [{ id, question_text, ask_kind: "decision" | "pick" | "approval" | "question" }], updated_at_ms, is_done?: boolean, goal_text?, started_at_ms?, event_log: [{ at_ms, text }], decision_log: [{ question_text, answer_text, at_ms }] }`.
- `pending_asks` are the items waiting for a person in the Decision Queue; the task list counts them.
- `project_names`: list of project names; their order sets the group order.
- `initial_tab`: `"active" | "done"`. `initial_task_id`: string or null.
- `artifact_store`: a map from file name to a document, sheet, folder, or code artifact.
- `suggested_artifacts`: suggested artifacts offered for a new tab.
- `text_strings`, `account`, `workspace_name`, `is_loading`, `is_empty`.
- Callbacks: `on_select(id)`, `on_create(task)`, `on_open_decision(task_id, ask_id?)`, `on_open_artifact(task_name, entry_name)`, `on_artifact_change(entry_name, artifact)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Header: title and count, Active and Done tabs, and a New task button.
3. Task list: a tree. Group headings name each project with its count and can collapse. Each row shows a progress ring (steps done out of total), the title, a status phrase, a Needs you badge with the number of asks, the artifact count, and the time since the last update.
4. Task panel (right): title, project, state, goal, a step checklist, a Needs you block with the first question and a Decide button, artifact tabs (using the shared artifact viewers), a "what happened so far" history, and the decisions already made for this task.
5. New task line: a single-line field above the list. Enter creates the task at the top of the chosen project, in planning state.

## States
- **planning**: the ring is drawn dashed; the status phrase reads planning.
- **running**: the ring fills in proportion to steps done.
- **paused**: the status phrase reads paused.
- **needs you**: the Needs you badge shows a count; the panel shows the first question with Decide.
- **done**: the task appears under Done with its completion date.
- **new task**: the line is open; creating a task adds it to the list and keeps the line open for the next one.
- **empty** (`is_empty`): Active shows a note and a New task button; Done shows a note that nothing is finished yet; the panel shows nothing open.
- **loading** (`is_loading`): placeholder rows and panel lines.

## Interactions
- J or K moves the selection to the adjacent row, provided no text field holds focus.
- Arrow keys, Home, and End move through headings and rows; the list occupies one stop in the tab order.
- Enter or Space on a row opens the task in the panel; on a heading, it collapses or expands that group.
- Left and right arrows on a heading: collapse or expand. Left on a row moves up to its heading.
- C or N opens the New task line. Enter creates the task; Escape closes the line.
- Left and right arrows on the Active and Done tabs switch lists.
- Panel tabs: arrow keys move between them; Delete closes an artifact tab.
- Decide opens the Decision Queue at that ask. Without `on_open_decision`, the Inbox opens.
- Project picker: a menu with icons; the chosen project is checked; Escape closes it and returns focus.

## Motion (functional)
- A new task settles in at the top of its project as its row grows from nothing.
- The chevron on a group heading turns as the group collapses or expands.
- Reduced motion: rows appear without movement.

## Accessibility
- The list is a tree: headings are top-level items with an expanded state and a count; rows are child items with their position inside the group.
- Each row is announced in full: title, project, Needs you count, steps, artifacts, and last update.
- The Needs you badge is a pointer shortcut; keyboard users reach the Decide control by pressing Enter on the row and then moving into the panel.
- A polite live region announces "Task created in <project>".
- Focus rings appear only while the keyboard moves through the list; text fields show no ring.

## Responsive
- Fills its container; the host sets the height.
- Every row has the same height. Step counts, artifact counts and the time sit in columns that never move; the status phrase is the first thing to shrink.
- Below the tablet breakpoint, the header shares one line with the shell's corner buttons.
- Below the phone breakpoint, each row takes two lines (title, then status phrase); opening a row shows the task as a sheet from the right.
- Touch targets are enlarged for touch input.

## Composition hints
Tree list with group headings; progress ring; badge; text field; tabs; side panel or sheet; checklist; artifact viewers; history list; buttons; menu; empty state; skeleton; live region.

## Acceptance checks
1. Each group heading's count equals the number of tasks in that group.
2. A progress ring's fill equals steps done divided by total steps.
3. The Needs you count equals the number of asks, summed across tasks.
4. Decide opens the Decision Queue at the ask's id.
5. Enter in the New task line creates a planning task at the top of its project and keeps the line open.
6. The Done tab lists done tasks with their completion day.
7. With `is_empty` set on Active, the note and the New task button appear.
8. At phone width each row takes two lines and the page does not scroll horizontally.
