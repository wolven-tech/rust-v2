# Work Queue Flow

Source item: affectUI Pro `components/agent-queue` (functional reference only; see README).

## Purpose
Shows jobs waiting for work slots, running in them, and finishing or failing, with the backlog over time. A person can see how fast work moves and steer it: pause intake, change the slot count, move a waiting job to the front, or withdraw it.

## Data
- `job_rows`: list of `{ job_key: string (unique), job_label: string, job_kind: "model_call" | "tool_call" | "sub_agent" | "human_review", job_state: "enqueued" | "active" | "succeeded" | "errored", enqueued_at_ms: number, started_at_ms?: number, ended_at_ms?: number, slot_key?: string, lane_priority: "urgent" | "standard" | "deferred" (default "standard"), progress_fraction?: number (0 to 1, while active), expected_run_ms?: number, attempt_number: number, max_attempts?: number, detail_text?: string, job_mark?: icon element, error_text?: string }`.
- `slot_rows`: list of `{ slot_key: string, slot_label: string, slot_detail?: string, slot_mark?: icon element }`. The number of slots is the concurrency.
- `clock_ms?`: number, read on the same timeline as the job timestamps. When absent, the component reads the wall clock itself.
- `throttle_windows?`: list of `{ from_ms: number, until_ms: number, reason_label?: string }`. While one is current, the provider is rate-limiting.
- `is_paused?` with `on_paused_change(paused: boolean)`.
- `on_slot_count_change(count: number)`, `min_slots` (default 1), `max_slots` (default 8).
- `backlog_line?`: number. Above this waiting depth, the waiting count is flagged and a line is drawn on the chart.
- `chart_span`: duration of recent history the chart shows, on the same clock as the job times. Default: a short recent window.
- `lifetime_totals?`: `{ completed_total?: number, errored_total?: number }`. Used when `job_rows` holds only recent jobs.
- `open_job_key?` with `on_open_change(key | null, job?)`.
- `on_promote(job)` and `on_withdraw(job)`: offered in the pinned card.
- `queue_title`: default "Queue". `queue_name?`: muted text beside the title.
- `class_name?`.

## Anatomy
Wide layout, left to right: waiting line, slots (each holding its active job as a growing bar), then completed and errored tiles.
Above the flow: heading and queue name, throttle indicator, pause control, slot stepper. Stat tiles: waiting count, active count, throughput per minute, 95th-percentile wait.
Below the flow: depth chart with the backlog line.
A pinned card shows one job in detail.

Narrow layout (when the container is narrow): waiting line on top, slots below as marks only, completed and errored side by side.

## States
- `enqueued`: in line; position shown.
- `active`: in a slot; bar grows with progress (or with elapsed time against `expected_run_ms`).
- `succeeded`: brief highlight, then moves to the completed tile.
- `errored`: brief highlight, then moves to the errored tile, or returns to the line as a retry when attempts remain.
- Retry: same key, back to `enqueued` with `attempt_number` plus one, a new enqueue time, and no start time or slot.
- Throttled: while a throttle window is current, active jobs show a paused-by-throttle state with a countdown to the window's end. The chart line climbs.
- Paused intake: no waiting job enters a slot. The pause control reads as resume.
- Backlog above `backlog_line`: waiting count flagged.
- Pinned card: detail, attempt number and priority, with promote and withdraw. (Design choice: promote is offered only for `enqueued` jobs.)

## Interactions
- Tab: one stop on the job list, then into the pinned card's controls.
- Arrow keys move through the busy slots first, then along the line starting at its head. Home and End go to either extreme.
- Enter or Space on a job: pins its card, or closes it if already pinned.
- Escape: closes the pinned card and returns focus to its job.
- Pause control: toggles intake. Slot stepper: changes slot count within min and max.
- Unverified in demo: the pinned card. Headless activation of a job row by label did not respond. Pause and slot-count controls activated without errors.

## Motion (functional)
- New jobs join at the back of the line. A high-priority job moves past the others to the front.
- The front job moves into a free slot and its bar grows with progress.
- A finished job gets a brief highlight and moves to its tile. A failed job gets a brief error cue and moves to its tile or back to the line.
- During a throttle window, bars stop and show their throttled state, and the countdown runs.
- Numbers ease to new values. The chart scale glides when it changes.
- Reduced motion: everything appears in place.

## Accessibility
- Each job is a button named with its label, kind, wait or run time, queue position, priority and attempt.
- Tab reaches a single job; the remaining jobs are reached with arrows.
- The pinned card is a labelled dialog; Escape closes it.
- Changes in throttling, job failures, requeues and the backlog threshold are reported through a polite announcement region.
- The depth chart exposes a text alternative stating the current backlog, its maximum, its mean and every throttled interval.
- The pause control is a toggle with pressed state. Stepper buttons are named by their action and the count is announced.

## Responsive
- Container-width based. Wide: horizontal flow. When the container is narrow: stacked flow, stat tiles in two columns. When it is narrower still: pause and stepper take a row of their own.

## Composition hints
Card, list, toggle button, stepper buttons, dialog, progress bars (custom), stat tiles, line chart (custom, with text summary), live region, icon element, badges for attempt and priority.

## Acceptance checks
1. Jobs appear in line order; an urgent job moves to the front.
2. Active jobs never exceed the slot count; changing slots updates the count within min and max.
3. Pausing stops waiting jobs entering slots; resuming restores intake.
4. A retry keeps the job key, increments the attempt, and clears slot and start time.
5. A throttle window shows the countdown and the paused bars until it ends.
6. Roving focus: one tab stop; arrows walk the slots, then the line.
7. Escape closes the pinned card and returns focus to its job.
8. The chart text summary matches the counts on screen.
9. Pinned card, promote and withdraw are unverified in demo, as noted above.
