# Parallel Work Grid

Source item: affectUI Pro `components/fan-out-grid` (functional reference only; see README).

## Purpose
One agent divides a large job among many workers. Every task is a cell on a grid that fills in as workers complete them. The grid shows progress, failures, retries, cost and an estimate of remaining time, and any task can be inspected. Suited to batch enrichment, translation, labelling and evaluation runs.

## Data
- `job_items` (Item[]): one per task, in cell order. Each `{ item_id, item_label, item_state, worker_name?, start_offset?, end_offset?, item_cost?, attempt_count?, output_value?, error_text? }`.
  - `item_state`: `"queued" | "in_progress" | "retrying" | "done" | "failed"`.
  - `start_offset` / `end_offset`: time offsets from the start of the job, used only for their difference and for remaining time. They are not timestamps.
  - `attempt_count`: a value of 2 or more is shown in the details.
- `job_name` (text): what the job does; shown beside the heading.
- `concurrency` (number, optional): workers running at once. Remaining time is calculated from it. Defaults to the number of items in progress.
- `clock_offset` (number, default 0): current position in the job, on the same scale as the offsets.
- `on_retry(item_ids)`: drives the "Retry failed" control, shown while any item has failed. Receives those ids.
- `on_pick(item | null)`: fired on pick (click or Enter) and on release.
- `format_money(amount)` (default dollars).
- `heading_text` (text, default "Fan-out"); `heading_icon` (node, optional).
- `extra_class` (text, optional).

Rules:
- Remaining time needs at least three finished items. Without `concurrency`, the estimate is hidden.
- Pass the full item list on every change. Ids keep each cell in its place; a cell animates when its state changes.
- A retried item keeps its id. Set its state back to in progress and raise its `attempt_count`.

## Anatomy
Reading order:
1. Header: optional icon, heading, and job name.
2. Stats row: done count, in-progress count, retrying count (plus failed count when non-zero), total cost, remaining time.
3. Grid: wrapped square cells, one per item, in order. Tone follows state.
4. Details window: opens for the focused or picked cell and shows label, state, worker, duration, cost, attempts, and output or error.
5. Counts line under the grid ("done of total"). Each count doubles as a filter for one state.
6. Retry failed control, when relevant.

## States
Per cell: queued (muted), in progress (primary tone with a moving highlight), retrying (caution tone), done (success tone), failed (error tone).

Whole job:
- In progress: counts and remaining time update; progress is announced at each tenth.
- Complete: every cell is done or failed; a final progress announcement is made; remaining time is hidden.
- Filtered: one count is active; cells outside it are dimmed.
- Empty: no items; the grid shows nothing and counts read zero. (Spec choice.)

## Interactions
Pointer:
- Hover a cell: its details window appears.
- Click a cell: picks it and keeps details open. Click again: releases it.
- Click a count: filters to that state. Click again: clears the filter.
- Click Retry failed: calls `on_retry` with the failed ids.

Keyboard:
- Tab: enters the grid as a single stop, then the counts, then Retry.
- Arrow keys: move between cells; details follow focus. Home and End: first and last cell.
- Enter or Space: keep details open, or release them.
- Escape: release the details.

## Motion (functional)
- Cells appear as a quick wave in order.
- In-progress cells carry a highlight that sweeps across them, offset from cell to cell.
- Retrying cells pulse in the caution tone.
- Done and failed cells give one small pop as they land.
- The details window opens out of its cell with a short scale and fade.
- With reduced motion requested, all of this happens instantly.

## Accessibility
- The whole grid occupies one tab stop; whichever cell is focused supplies its accessible description.
- Count controls expose pressed state and show one state at a time.
- Progress is announced politely at every ten percent and at completion.

## Responsive
- The grid fills the width it is given and wraps cells into rows.
- On narrow widths, cells shrink and each number moves onto its own line.

## Composition hints
Panel, header with icon, stat tiles, grid of square status cells, popover or details panel, segmented toggle buttons, icon button, live region, progress announcer.

## Acceptance checks
1. With 240 items, 29 done, 13 in progress and 3 retrying, the counts read 29, 13 and 3, and the total cost updates when any `item_cost` changes.
2. Changing one item from in progress to done changes only that cell's tone; every other cell stays put.
3. Hovering a cell shows its label, state, duration and cost; an item with `attempt_count: 2` shows its attempts.
4. Enter on a focused cell keeps its details open; Escape releases them.
5. Clicking the "done" count shows only done cells; the rest are dimmed.
6. "Retry failed" appears when at least one item has failed and calls `on_retry` with exactly those ids.
7. Remaining time stays hidden until three items have finished.
8. Progress is announced at each tenth of completion and once at the end.
9. At narrow widths the numbers sit on their own line and cells are smaller; no horizontal scroll.
10. With reduced motion requested, there is no wave and no highlight sweep; cells show their final state at once.
