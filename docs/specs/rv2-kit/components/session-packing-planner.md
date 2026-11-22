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

## Amendment: structure and demo

**Archetype:** a week grid with a holding pile above it. Columns are days, rows are sessions. Each cell holds the cards placed in that session, and each session shows how much of its budget is used.

**Parts:**
- Heading, note line, and the Auto and Manual radio pair.
- Agent strip: small overlapping agent marks. Pressing one filters the cards.
- Holding pile: unplaced cards with a count, above the grid.
- Week grid: day headers, session-row headers, a marker on the current session, and elapsed cells visibly dimmed.
- Cards: title, agent mark, usage share as text. A pinned card shows a pin mark. A split card shows its part as "n of m".
- Capacity bar per session with the used share as text.
- Card panel opened by click or Enter.

**Gallery demo:** four day columns (Mon to Thu) and three session rows labelled "Block A", "Block B", "Block C". Current session: Mon Block B. Elapsed: Mon Block A. Seven tasks in total: three placed by auto mode (Tue A, Tue C, Wed B); one pinned booking (Thu A); one task split across Tue B (part 1 of 2) and Wed A (part 2 of 2) with the two shares adding up to the original; two tasks in the pile that no session has room for. Mode is Auto and slotting has finished.

**Visible at rest in the gallery:** four day columns, three labelled rows, the current-session marker, the dimmed elapsed cell, the split pair, the pinned card, the pile count with its two unplaceable tasks, the capacity bars, the mode pair set to Auto.

**Acceptance checks (continued):**
10. The gallery at rest shows four day columns and three labelled session rows, a current-session marker on one cell, one dimmed elapsed cell, and Auto selected.
11. The split task appears as two cards, "part 1 of 2" and "part 2 of 2", in two sessions, and their shares add up to the original share.
12. The two unplaceable tasks stay in the pile. The pile shows its count, and each unplaceable card says in words that no session has room for it.
13. Each session row shows its capacity bar with the used share written as text.
14. The Auto and Manual pair is a radio group. Arrow keys switch it, and the new mode is announced politely.

## Amendment 2: structure and demo

Replaces amendment 1 in four places. The holding pile moves from above the grid to a backlog column at the grid's leading edge. Session rows are labelled with start times instead of block names. A now line crosses the session in progress. The demo grows to eleven tasks. The rule that an unfit task stays whole in the pile is kept, and restated in check 20.

**Layout archetype:** a week grid with a backlog column at its leading edge. Columns left to right: backlog, row labels, four day columns. Rows are sessions in start-time order, top to bottom. The heading, note, Auto and Manual pair, and agent strip sit above.

**Parts that must exist:**
- Heading row: title, note line, and the Auto and Manual radio pair at the trailing edge.
- Agent strip: overlapping agent marks, with the task count at its trailing side.
- Backlog column: a header with the word "Unplaced" and its count, then the unplaced cards stacked. Each card: title, agent mark, usage share as text, and, for a card that no session can take, a line saying no session has room for it.
- Row labels: one per session row, showing its start time.
- Day headers: one per day column.
- Week grid: the cards of each session in its cell. The cell of the session in progress carries the words "In progress". Elapsed cells are dimmed and show their used share as text.
- Now line: a line at the current time inside the cell of the session in progress only. It never crosses cards in other sessions' cells.
- Capacity bar per cell, with the used share as text.
- Cards: title, agent mark, usage share as text. A pinned card shows a pin mark. A split card shows "part n of m".
- Card panel, opened by click or Enter, as in amendment 1.

**Demo data (ours):**
- Four day columns, Monday to Thursday. Three session rows starting at 07:00, 12:00 and 17:00.
- Mode Auto; slotting finished.
- Session in progress: Monday 12:00. Elapsed: Monday 07:00.
- Eleven tasks. Six placed by auto mode: Monday 17:00, Tuesday 07:00, Tuesday 17:00, Wednesday 12:00, Wednesday 17:00 and Thursday 12:00. One pinned booking at Thursday 07:00. One task divided into two parts: part 1 of 2 at Tuesday 12:00 and part 2 of 2 at Wednesday 07:00, with the two shares adding up to the original. Three unplaced tasks, none of which fits any session's free room.

**Visible at rest:** four day columns, three start-time rows, the "In progress" cell with the now line in its row, one dimmed elapsed cell, the split pair, the pinned card, the backlog with its count of three, capacity bars with text, Auto selected.

**Acceptance checks (continued):**
15. At rest the backlog is a column at the leading edge of the grid with the count three. No unplaced card appears above the grid. Replaces amendment 1's placement of the pile.
16. Each row label is a start time (07:00, 12:00 and 17:00 in the gallery). The in-progress cell carries the words "In progress", and the now line sits only inside that cell. Replaces the block labels of amendment 1.
17. The split task appears as "part 1 of 2" and "part 2 of 2" in two sessions, and the shares add up to the original (check 11).
18. The gallery shows twelve cards for eleven tasks: six auto-placed, one pinned, two split parts, three in the backlog. Count by SSR text.
19. Each backlog card says in words that no session has room for it, and the backlog header count equals the number of such cards (check 12).
20. A task that fits within one session's budget but has no free room stays whole in the backlog. Only a task larger than one full session's budget is divided. SSR test with one task of each kind.
21. Each cell shows its capacity bar with the used share as text, and elapsed cells render dimmed (checks 10 and 13).
22. The Auto and Manual pair is a radio group: arrow keys switch it and the new mode is announced politely (check 14).
