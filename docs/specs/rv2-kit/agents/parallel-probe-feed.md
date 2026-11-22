# Parallel Probe Feed

Source item: affectUI Pro `components/agent-findings` (functional reference only; see README).

## Purpose
Shows a set of agents running checks in parallel. One compact row per agent shows its progress; results they produce appear in a short feed, newest first. Lets a person follow a swarm of checks without reading each agent's detail.

## Data
- `heading`: string, default "Verification".
- `workers`: `{ id, name, current_step?: string, completed: number, planned: number, state?: "running" | "done" | "failed", counts?: { critical?, warning?, info? } }[]`. State defaults to `running` until `completed` reaches `planned`.
- `results`: `{ id, worker_id, severity: "critical" | "warning" | "info", title, location?, found_ms: ms }[]`, newest first.
- `visible_count`: number, default 4. Maximum number of feed entries shown; the feed grows to this height as results arrive.
- `clock_ms?`: clock for relative times. When absent, real time is used.
- `on_select_result(result)`.
- `extra_class?`

## Anatomy
Top to bottom:
1. Header bar: heading; a summary "N of M checks" (completed of planned across workers); severity totals, each with a count and a text label.
2. Worker rows, one per worker, in the order given. Each row: name, current step (one line, muted), a thin progress bar, `completed/planned`, and per-severity counts for that worker.
3. Feed section: a small label "Latest findings" and filter chips. A chip shows the active worker filter. Below: up to `visible_count` results, newest at top. Each result: severity marker, title, worker name, location (monospace), relative time.

## States
- Worker running: normal row, progress bar advancing.
- Worker done: row muted; shows completion, no step text.
- Worker failed: failure label on the row.
- Worker filter active: feed shows only that worker's results; the chip is shown; other rows are muted but remain selectable.
- No results yet: feed area empty with a neutral placeholder line (not observed in the public demo; clean room decides the text).
- Severity: critical, warning and info each have a distinct marker and a text label.

Transitions: progress changes when `completed` changes; a new result appears at the top of the feed and pushes the rest down, the oldest leaving when the feed is full.

## Interactions
- Click or Enter on a worker row: toggles the feed filter to that worker; clicking again shows all.
- Click or Enter on a result: calls `on_select_result(result)`.
- Keyboard: Tab moves through worker rows and then results in the feed. Enter or Space behaves as click.
- Escape: no effect defined.
- Touch: tap equivalent to click.

## Motion (functional)
- Progress bars grow smoothly when `completed` changes.
- A new result slides into the top of the feed; existing results shift down.
- Filter change: feed content updates without a page-level shift.
- Reduced motion: results appear without sliding.

## Accessibility
- The feed is a polite live region; each new result is announced.
- Every severity has a text label, not only a marker.
- Each worker row is a toggle button exposing its pressed state for the filter.
- Progress bars expose completed and planned values.

## Responsive
- Below a narrow breakpoint, check counts and location text are hidden so each row stays on one line.
- Supports light and dark themes; visual values come from theme variables scoped to the component root.

## Composition hints
Container card, header bar, progress bar, toggle button (worker row), badge with counts (severity), chip (active filter), list (feed), monospace text (location), relative-time text, polite live region.

## Acceptance checks
1. Each worker row shows its name, current step, progress bar and `completed/planned`.
2. Header summary equals the sums of `completed` and `planned` across workers.
3. A new result appears at the top of the feed; the feed never exceeds `visible_count` entries.
4. Clicking a worker row shows only that worker's results and adds a removable filter chip; clicking it again clears the filter.
5. Enter on a focused worker row toggles the same filter.
6. Clicking a result calls `on_select_result` with that result.
7. Each new result is announced through a polite live region.
8. Each severity shows its text label, not only a visual marker.
9. When `completed` reaches `planned` for a worker, its state becomes done and its row mutes.
10. Below the narrow breakpoint, each worker row stays on one line.
