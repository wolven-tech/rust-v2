# Task Assignment Board

Source item: affectUI Pro `components/delegate-board` (functional reference only; see README).

## Purpose
Lets a person hand queued work to a team of agents: drag a task onto an agent, or switch to automatic assignment so each new task goes to whichever agent is free first. Each agent shows how long it stays busy and what a hand-off costs.

## Data
- `worker_list` (Worker[]): each `{ worker_id, worker_name, worker_model_name (optional), worker_avatar (optional), busy_minutes (optional), cost_per_minute (optional) }`.
  - `busy_minutes`: minutes left on the task the worker runs now; drives "busy until".
  - `cost_per_minute`: shown after the model name; the rate a hand-off is charged at, and the basis of the queue total.
- `work_items` (Item[]): each `{ work_id, work_title, estimate_minutes, required_tools (optional), assigned_worker_id (optional) }`. In each queue the first item is the running one. Beyond three items, a "+N" chip lists the rest on hover or focus.
- `on_assign(work_id, worker_id | null)`: fired on drop, keyboard pick or automatic assignment. `null` means back to unassigned. Without this callback the board moves items itself.
- `assignment_mode` / `default_assignment_mode` (`"automatic" | "manual"`, default `"manual"`). Automatic hands each unassigned item to the least busy worker, at a slow pace that can be followed.
- `on_mode_change(mode)`.
- `proposed_assignment` (`{ work_id, worker_id } | null`): a proposed hand-off. The worker is highlighted and previews its new time; the item is marked.
- `clock_time` (timestamp, default current time): reference time for "busy until".
- `format_money(amount)` (default dollars): replace to use another currency or credits.
- `heading_text` (text, default "Delegate").
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Header: heading, and an automatic-mode switch labelled with its word.
2. Unassigned section: items with no worker. Each shows its title and estimate.
3. Workers section: one column per worker. Each column shows the name, model and cost rate; a "busy until" time with the total queued cost; then the queue, running item first.
4. Queue items: title and estimate, with a small tools marker where given.

## States
- Idle: items sit in Unassigned; workers with empty queues show "free".
- Picked up: an item is held for keyboard assignment; candidate workers are highlighted in turn.
- Dragging: the dragged item follows the pointer; the worker under it highlights and previews its new time.
- Proposed: `proposed_assignment` is set; the target worker is highlighted and the item is marked. Accepting is the caller's own `on_assign` call.
- Automatic: switch on; unassigned items move to workers on a timer.
- Free worker: queue empty; shows "free" and zero cost.
- Overflow: more than three queued items; a "+N" chip lists the rest.

## Interactions
Pointer:
- Drag an item onto a worker column: assign it. Drop on Unassigned returns it.
- Click an item: pick it up (see keyboard below).
- Hover or focus the "+N" chip: list the hidden items.
- Toggle the automatic switch.

Keyboard:
- Enter on an item: pick it up. Enter again: hand it to the highlighted worker.
- Arrow Up / Down: choose the worker while an item is picked up.
- Digits 1 to 9: hand the focused item to that worker at once. Numbers show beside the names.
- Escape: put a picked-up item back.

Touch: items and overflow chips can be moved by dragging with a finger.

## Motion (functional)
- A dragged item lifts, leans a little and tracks the pointer. The worker beneath it lights up and previews the time it would be busy until.
- On drop, the card moves into the worker's queue; queued items move between queues smoothly.
- In automatic mode, hand-offs occur at a steady pace so each can be followed.
- Reduced motion: moves happen at once.

## Accessibility
- Unassigned and each worker are labelled regions; each queue is a list named after its worker.
- Each item is a button described by how to move it.
- Drag is never the sole route: keyboard pick-up, digit keys and automatic mode produce the same outcome.
- Automatic mode is a labelled switch.
- Every hand-off is announced politely ("<item> to <worker>").

## Responsive
- Fills its container up to a maximum width.
- On narrow widths: the unassigned section sits above the workers.
- Touch dragging works for items and chips.

## Composition hints
Panel or bar, switch, section list, list items as draggable buttons, drop targets, chip with overflow popover, number formatter, live region, timer for automatic mode.

## Acceptance checks
1. Dragging an unassigned item onto a worker calls `on_assign(work_id, worker_id)` and the item appears at the end of that worker's queue.
2. Enter on an item then Enter on the highlighted worker produces the same `on_assign` call.
3. Pressing digit 2 on a focused item assigns it to the second worker in one step.
4. Escape while an item is picked up returns it to its previous place without calling `on_assign`.
5. With automatic mode on, unassigned items are handed out one at a time, least-busy worker first.
6. A `proposed_assignment` highlights the worker and previews a new "busy until" time before any assignment.
7. A worker's "busy until" equals now plus `busy_minutes` plus the estimates of its queued items.
8. A queue with five items shows three items and a "+2" chip that lists the other two.
9. Changing `format_money` changes every displayed cost.
10. On narrow widths, the unassigned section renders above the worker columns.

## Amendment: structure and demo

Layout archetype: a board with three stacked bands. A top bar, then a single strip of unassigned items, then a row of worker columns. A one-line usage hint sits under the board.

Parts that must exist:
- Top bar: heading at the leading edge, automatic switch at the trailing edge, on one line.
- Unassigned strip: one wrapping row of item chips. Each chip shows its title and its estimate only.
- Worker columns: one per worker, equal width, in `worker_list` order. Top to bottom: position number, worker name, model name at the trailing edge when given, rate line, status line, then the queue.
- Status line: for a busy worker, the busy-until time and the queue's total cost. For an empty queue, the word "free" and a zero cost.
- Preview line: while a proposal targets a worker, the status line gains a second line showing that worker's busy-until time with the proposed item counted in.
- Queue: item cards, running item first. A tools list, when given, sits at the trailing edge of its card.
- Overflow chip: a "+N" chip after the third card of any queue longer than three.
- Usage hint: one line under the board naming the three routes (pointer drag, keyboard pick-up, number keys).

Demo data for the gallery (our own names and values):
- Four workers, three busy and one free.
- Four unassigned items in the strip.
- Queues of two, one, two and zero items.
- One item carries a tools list; at least one other item carries none.
- Automatic switch off.
- One proposed hand-off from an unassigned item to the second worker.

States visible at rest in the gallery: the proposed hand-off with its preview line, the free worker, and a tools list on one card. Dragging, pick-up and automatic mode are not shown at rest; they are reached by interaction.

Acceptance checks:
11. The bar shows the heading and the automatic switch on one line at desktop width.
12. The gallery demo shows four worker columns of equal width, exactly one of which reads "free" with a zero cost.
13. The gallery demo shows the unassigned strip with four chips in a single row.
14. The gallery demo shows the proposed hand-off at rest: target column highlighted, proposed item marked, preview line present, no interaction needed.
15. Each worker column shows its position number, and that number is the digit that targets the column (check 3).
16. A queue item with a tools list shows the list at its trailing edge; an item without one shows no list. The gallery demo contains both.
17. The usage hint line is present below the board in the gallery demo.
18. The proposed state appears as text (preview line and a marker on the item), not only as a highlight; SSR output contains both.

## Amendment 2: structure and demo

Replaces amendment 1 in three places: the unassigned strip becomes a vertical pool beside the workers; the worker columns become worker rows in a trailing list; the proposal is shown as text as well as a highlight. Amendment 1 checks 11, 16 and 17 stand. Checks 12, 13 and 14 are superseded by 20 to 23 below.

**Layout archetype:** two columns under the top bar. The leading column is the unassigned pool, a vertical list of item cards. The trailing column is the worker list, one row per worker, stacked top to bottom. The usage hint sits under both columns.

**Parts that must exist:**
- Top bar: heading and automatic switch, as in amendment 1.
- Unassigned pool (leading column): a header line with the word "Unassigned" and a count, then one card per unassigned item in list order. Each card: title, estimate, and the tools list on its own line when given.
- Worker list (trailing column): a header line with the word "Workers" at its leading edge and the total queued cost of all workers at its trailing edge.
- Worker row: a header line with position number, worker name, and model name, with the rate beneath. At the trailing edge, a status badge. Below the header, the queue as a wrapping row of item cards, running item first.
- Status badge: for a busy worker, "Busy until" with the time, and that queue's total cost on the rate line. For an empty queue, the word "Free" with a zero cost.
- Preview badge: while a proposal targets a row, that row's status badge becomes the busy-until time with the proposed item counted in, and the added cost as text. The proposed card in the pool carries a proposal marker and the word "proposed".
- Overflow chip: a "+N" chip after the third card of any queue longer than three.
- Usage hint: one line under both columns naming the three routes: pointer drag, keyboard pick-up, number keys.

**Demo data (ours):**
- Five unassigned items in the pool.
- Four workers in `worker_list` order: three busy, one free. Queues of two, one, two and zero items.
- One item carries a tools list; at least one other carries none.
- Automatic switch off.
- One proposed hand-off from the first unassigned item to the second worker.

**Visible at rest:** the proposal with its preview badge and proposal marker, the free worker with its zero cost, one tools list, the pool with five cards, and the worker list total. Dragging, pick-up and automatic mode are reached by interaction only.

**Acceptance checks (continued):**
19. At desktop width the pool sits in the leading column and the worker rows in the trailing column, side by side below the top bar.
20. The gallery shows exactly five cards in the pool, each with title and estimate, in one vertical list.
21. The gallery shows four worker rows in `worker_list` order. Exactly one reads "Free" with a zero cost.
22. Each worker row shows its position number, and that number is the digit that targets the row (check 3).
23. At rest the proposal is shown as text: the target row's status badge is a preview with the busy-until time counting the proposed item, and the proposed card carries the word "proposed". SSR output contains both.
24. A queue item with a tools list shows the list on its own line at the trailing side of its card; an item without one shows none. The gallery contains both.
25. The worker list header shows the sum of every row's queue total at its trailing edge, and that figure changes when `format_money` changes (check 9).
26. SSR test: a queue of five renders three cards and a "+2" chip. Gallery queues never exceed three, so no overflow chip shows at rest.
27. The usage hint is present under both columns in the gallery demo.
