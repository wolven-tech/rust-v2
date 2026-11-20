# Agent Task Board

Source item: affectUI Pro `screens/agent-inbox-board` (functional reference only; see README).

## Purpose
Shows every background task the agents are working on, grouped by what each one needs. A person can approve, reject, retry, or open a task from its card without leaving the board.

## Data
- `task_records`: list of `{ id, entry_title, entry_description?, agent_id, task_state: "waiting" | "failed" | "running" | "done" | "stopped", started_at?, elapsed_seconds?, source_count?, tool_count?, step_records?: [{ entry_label, time_label? }] }`. Each task appears once.
- `agent_records`: list of `{ id, entry_name, identity_hue_token? }`. Each agent's identity mark is drawn in its assigned tone.
- `initial_view`: `"board" | "list"`. `initial_task_id`: string or null.
- `is_loading`, `is_empty`: booleans.
- Callbacks: `on_approve(task)`, `on_reject(task)`, `on_retry(task)`, `on_open(task)`, `on_create(task)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Top bar: tabs (All, Needs you, Running, Done) with counts; an agent filter listing the agents; a board and list toggle; a New task button.
3. Board: three columns with headings and counts: Needs you (waiting and failed), Running, and Done (done and stopped).
4. Card: agent identity mark, title, a one-line description, a state label (or a state glyph on narrow cards), and a time: a live clock while running, the final duration once done.
5. Detail panel (right) for the open task: title, state, agent, source and tool counts, a step list with times, and the action buttons that suit its state.
6. New task form: a description text area and an agent select. Starting it creates a running task.

## States
- **waiting**: the card sits in Needs you. The panel shows Approve and Reject.
- **failed**: the card sits in Needs you with a failed state. The panel shows Retry.
- **running**: the card sits in Running with a live clock and a running indicator on the agent mark.
- **done**: the card sits in Done with its final duration. The panel shows Open.
- **stopped**: the card sits in Done with a stopped label. The panel shows Open.
- **transition**: when a task changes state, the card first displays the new state where it is, then moves to its new column.
- **loading** (`is_loading`): placeholder cards under the real column headings, and placeholder lines in the panel.
- **empty** (`is_empty`): one short note with New task; no columns filled.

## Interactions
- Activating a card opens it in the panel.
- Approve or Reject (waiting) calls `on_approve` or `on_reject` with the task. Retry (failed) calls `on_retry`. Open (running or done) calls `on_open`.
- Keyboard: J and K move between tasks, column by column. Enter opens the focused task.
- Left and right arrows move between the Board and List views and between panel tabs.
- N opens the New task form. Ctrl or Cmd+Enter starts the task; Escape closes the form.
- Agent filter: a multi-select menu; filtering removes non-matching cards from every column.

## Motion (functional)
- A task that changes state shows the change in place before it moves to another column.
- The agent identity mark of a running task shows a subtle live indicator, and the card displays a running ring.
- Reduced motion: cards change column without motion.

## Accessibility
- A card is exposed as a button whose accessible name joins its title, state and agent. The open card carries the current marker.
- A polite live region states how many tasks need the person and how many are running, whenever those counts change.
- Focus rings appear for keyboard users only and sit inside the card edge, so the scrolling column does not clip them.

## Responsive
- Wide screens: three columns side by side, each with a maximum width.
- Narrower: columns collapse into a single stack, and list rows lose their supplementary line.
- Narrower still: the top bar keeps only icon controls, and New task becomes a plus button.
- Touch targets are enlarged for touch input.

## Composition hints
Tabs with counts; select menu; segmented view toggle; kanban-style columns; task card; agent identity mark; state badge; live clock; side panel (sheet on narrow screens); step list; buttons; form (text area, select); skeleton; empty state; live region.

## Acceptance checks
1. Waiting and failed tasks appear in Needs you; running tasks in Running; done and stopped tasks in Done.
2. Approve on a waiting task calls `on_approve`, and once the host sets the state to running, the card moves to Running.
3. Retry appears only on failed tasks; Open appears only on running and done tasks.
4. The header tab counts equal the number of cards in each column.
5. Switching between Board and List keeps the open task open.
6. Pressing N opens the form; Ctrl or Cmd+Enter calls `on_create` with the entered description and chosen agent.
7. With `is_empty` set, only the note and New task appear.
8. At phone width the columns stack and the page does not scroll horizontally.
