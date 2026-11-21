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
