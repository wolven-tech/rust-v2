# Session Packing Planner

Source item: affectUI Pro `components/scheduled-runs` (functional reference only; see README).

## Purpose
Fits a backlog of agent tasks into a week of sessions, each with a usage ceiling. Every session has a fixed budget: short tasks fill leftover room, oversized tasks are divided across several sessions, and placed tasks can be relocated by hand. The person can see where queued work will run before it starts.

## Data
- `task_queue` (ScheduledTask[]): each entry is `{ task_id, task_title, usage_share, agent_name?, task_detail?, agent_logo?, pinned_slot? }`. `usage_share` is the fraction of one session's budget the task consumes. `pinned_slot` is `{ day_index, session_index }` and marks a task already booked into a position. Unpinned tasks wait in a holding pile and are placed in list order.
- `day_headers` (string or `{ day_name, day_date? }`, per column): three or four columns read best.
- `row_headers` (string[]): the start time of each session row.
- `current_slot` (`{ day_index, session_index }`): the session in progress. Sessions before it are dimmed and accept nothing new.
- `session_budget` (number): what one session can carry, in the unit used by `usage_share`.
- `smallest_fragment` (number): a split never leaves a fragment below this size in one session.
- `agent_roster` (`{ agent_name, agent_logo? }[]`): shown in the header; choosing an entry highlights its tasks.
- `mode` / `initial_mode` (`"auto" | "manual"`): auto slots pending tasks in on its own; manual sends them back to the pile. The mode can change while a run is underway.
- `auto_delay` (number or boolean): in auto mode, the wait before slotting begins.
- `loop_demo` (boolean): clears and re-slots on repeat, for demonstrations.
- `title` / `note` (string): heading and a quiet line beside it.
- `on_plan_settled(placed_parts)`: called once every fragment has a home after slotting. Each part reports `planned_task`, `day_index`, `session_index`, `usage_share`, `part_number` and `part_count`.
- `on_drop(planned_task, placed_parts)`: called when a task is dropped into a session.
- `on_relocate(part, target_slot)`: called when a pinned card moves to another session. The host is expected to persist the change.

## Anatomy
Reading order:
1. Heading, note, and the mode switch (Auto and Manual as a radio pair).
2. Agent strip: small overlapping logos for the agents involved.
3. Week grid: day columns and session rows. Each cell holds the cards for that session. A marker flags the session in progress.
4. Holding pile: unplaced tasks, shown as a stack above or beside the grid.
5. Budget bar for each session, showing how much is in use.
6. Card panel: opened from a card, with the task's agent, detail and placement.

## States
- **Pile full, grid empty:** tasks are waiting; the mode is manual, or auto before slotting starts.
- **Slotting (auto):** cards leave the pile one at a time and settle into sessions in order.
- **Settled:** every task is placed or reported as unable to fit; `on_plan_settled` has fired.
- **Divided task:** a task too large for any session's free room is split; every fragment shows its part number.
- **Pinned card:** fixed to its session; the person may move it.
- **Elapsed session:** dimmed; takes no new work.
- **Lifted card:** picked up for relocation; valid destinations are highlighted.
- **Blocked move:** a placement that does not fit is refused, and the refusal states its reason.
- **Agent filter active:** cards belonging to other agents recede.

Transitions: a card moves from the pile into a session on slotting or drop. A lifted card lands in a session on drop, or returns to its origin on cancel.

## Interactions
- **Pointer drag:** drag a pile card onto a session; drag a pinned card onto another session.
- **Pointer click on a card:** opens its panel.
- **Pointer on an agent logo:** highlights that agent's tasks.
- **Keyboard, Tab:** walks the cards, pile first, then sessions in day order.
- **Keyboard, Enter:** opens or closes the focused card's panel.
- **Keyboard, Space:** lifts the focused card; pressing Space again drops it onto the aimed session.
- **Keyboard, Left and Right while lifted:** aim one day earlier or later.
- **Keyboard, Up and Down while lifted:** aim at the neighbouring session on the same day.
- **Keyboard, Escape:** returns a lifted card to its place; also closes the panel.
- **Keyboard, Left and Right on the mode switch:** toggle between Auto and Manual.
- Elapsed-session cards announce that they cannot be moved.

## Motion (functional)
- In auto mode, cards leave the pile in order and travel into their sessions, so each placement can be followed.
- A divided task shows its fragments appearing in their sessions one after another.
- A dragged card tracks the pointer; the destination session lights up.
- Cards that leave a position close the gap, starting from the bottom.
- With reduced motion requested, placement happens without travel.

## Accessibility
- Every card is a focusable button; its hint explains what Enter and Space do, and elapsed-session cards say they are locked.
- Keyboard moves are announced politely: the card lifted, each session it is aimed at, whether that session has room (and if not, why), and where it finally lands.
- Once dropped, the card receives focus in its new session; Escape returns focus to its starting point.
- The week layout is a grid whose columns and rows are labelled. The mode switch is a radio group.
- Agent logos used as filters are toggle buttons named after their agent.

## Responsive
- On narrow screens the week grid scrolls sideways, and the session column stays pinned in view.
- Cards keep a constant size, so nothing shifts as tasks land.
- On touch, dragging works with a finger; a tap opens the panel.

## Composition hints
- Card for the container.
- Grid layout with headers.
- Draggable card with a keyboard pick-up mode.
- Capacity bar.
- Segmented control or radio group for mode.
- Overlapping avatar group for agents.
- Dialog or side panel for details.
- Polite live region.

## Acceptance checks
1. A task whose `usage_share` exceeds a session's budget is divided into parts whose usage adds back to the original, each part at or above `smallest_fragment`.
2. Space on a pile card, Right arrow, then Space places the card in the session one day later and calls `on_drop`.
3. Escape while a card is lifted returns it to its origin without calling `on_drop` or `on_relocate`.
4. A session before `current_slot` refuses new placements and says so in the live region.
5. Switching from auto to manual returns unpinned, placed cards to the pile and leaves pinned cards where they are.
6. Moving a pinned card calls `on_relocate` with the part and the target.
7. A task with a `pinned_slot` appears in its session before any slotting happens.
8. Choosing an agent logo fades the cards of other agents.
9. When the week is wider than the screen, the grid scrolls sideways and the session column stays visible.
