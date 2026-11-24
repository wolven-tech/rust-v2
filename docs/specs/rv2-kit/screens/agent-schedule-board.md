# Agent Schedule Board

Source item: affectUI Pro `screens/agent-schedules` (functional reference only; see README).

## Purpose
Shows the recurring jobs an agent runs on a regular rhythm: when each next runs, how the most recent runs ended, and what they cost. A person can pause a schedule, skip one upcoming run, or start a run immediately.

## Data
- `schedule_list`: list of `{ id, entry_name, project_name?, brief_summary?, cadence_text, run_times: [{ at_ms, run_state: "upcoming" | "running" | "done" | "needs-you" | "failed", progress_percent? }], duration_seconds?, is_paused, history_log: [{ at_ms, outcome: "done" | "needs-you" | "failed", summary, cost_label? }], cost_per_run_label? }`.
- `run_times` carries timestamps as epoch milliseconds. The timeline shows runs near the current moment; the panel lists the next five.
- `now_ms`: the current time, supplied by the host.
- `selected_schedule_id`: string or null.
- `is_loading`, `is_empty`.
- Callbacks: `on_pause(id, paused)`, `on_run_now(id)`, `on_skip(id, at_ms, skipped)`, `on_select(id)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Header: title and the count of active schedules out of the total.
3. Timeline: a horizontal axis centred on the present. Past runs sit to the left and upcoming runs to the right. Each run is a marker. A now line sits at the present. A trace above the axis shows agent activity, rising while a run is in progress.
4. Status line beneath the timeline: names what is running now with its percentage, or the next run.
5. Schedule list: one row per schedule, with name, cadence in words, next run, last outcome badge, average cost, and a pause switch.
6. Schedule panel (right): name, brief, cadence, the next five runs each with a Skip toggle, recent runs with outcome badges, cost per run, and buttons for Run now and Pause or Resume.

## States
- **running now**: the marker for the active run fills in proportion to its progress; the status line names it.
- **upcoming**: the marker is hollow.
- **skipped**: the marker is struck through in the panel and removed from the timeline's upcoming set.
- **paused** (`is_paused`): the row is dimmed, its switch is off, and the timeline shows no upcoming markers for that schedule.
- **outcome**: recent outcomes appear as word badges: Done, Needs you, Failed.
- **loading** (`is_loading`): placeholder chart, placeholder rows, and a placeholder panel.
- **empty** (`is_empty`): a note on how to create a schedule, with an Ask in Thread button that opens the Thread screen.

## Interactions
- J or K, or the up and down arrows: move between schedules.
- Enter opens the schedule in the panel. On a timeline marker, Enter opens the schedule that marker belongs to.
- Space on a switch pauses or resumes.
- Focus order: timeline markers first, then each row with its switch, then the Run now, Pause, and Skip controls in the panel.
- Run now starts a run immediately. The run appears on the timeline at once and fills across the schedule's duration. Calls `on_run_now`.
- Skip removes one upcoming run from the schedule; a Restore action puts it back. Calls `on_skip`.

## Motion (functional)
- When the present reaches a run, the activity trace climbs at its moving head, and a small status chip shows what is running and how far along it is.
- When a run ends, the trace settles and the marker keeps its final state.
- The timeline moves with the clock: the past extends at the present, and upcoming markers slide toward it.
- Reduced motion: no glow or blinking; the trace and the chip still show the run.

## Accessibility
- Markers are buttons. Each name lists the schedule, the moment, and the state: finished, failed, in progress with a percentage, or yet to come.
- The status line is text, for example "Running now: <name>, 38 percent".
- Each switch is a switch control named with its schedule and its action.
- Outcomes are text badges, never a mark alone.
- Focus rings appear for keyboard users only.

## Responsive
- Expands to the container. The text column keeps a comfortable measure, and the timeline spans the whole width.
- Each row packs its fields onto one line; below phone width, outcome and cost retreat into the panel.
- Narrower still, a row stacks into three brief lines (title, cadence, upcoming time), with the mark beside the title.
- Long names truncate with a tooltip. Large costs use compact notation in their column.
- Touch targets are enlarged for touch input.

## Composition hints
Timeline chart with markers and a now line; status line; badge; switch; list rows; side panel; buttons; skeleton; empty state.

## Acceptance checks
1. Markers left of the now line are past runs and show their outcome or failure; markers to the right are upcoming.
2. Run now adds a running marker immediately and calls `on_run_now`.
3. Skip removes that upcoming run from the timeline; Restore brings it back and calls `on_skip` with the skipped flag.
4. Toggling the switch calls `on_pause`, and the row shows as paused.
5. Outcomes are announced as words such as "Failed", not only as a mark.
6. While a run is in progress, the status line names it and shows its progress.
7. With `is_empty` set, the note and the Ask in Thread action appear.
8. At phone width each row takes three lines and the page does not scroll horizontally.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** horizontal run timeline centred on the present, a status line, a schedule list, and a right panel for one selected schedule.

**At rest (gallery demo):**
- Header: title and active count (6 of 7).
- Timeline: an axis spanning ten hours around now, with ticks at round two-hour intervals. The now line sits in the central third of the axis.
- Markers: four past runs to the left of the now line (three done, one failed), and six upcoming runs to the right. No run is in progress at rest.
- Agent activity trace above the axis, flat at rest.
- Status line beneath the timeline: names the next run and its moment in words.
- Schedule list: seven rows. Each row shows name, cadence in words, next run, last outcome badge, average cost and a pause switch. One row is paused: dimmed, switch off, no upcoming markers on the timeline.
- Panel open on one schedule with, in order: name, brief, cadence, Run now and Pause buttons, the next five runs each with a Skip control, recent runs with outcome badges, cost per run.

**States at rest:** one paused schedule; panel open; no run in progress.

**Arrangement:** timeline across the full width at the top; status line under it; schedule list on the left below; panel on the right.

**Motion, settled frame:** the activity trace is flat and the now line is at its position. Nothing moves at rest.

**Checks (continued):**
9. Four markers sit left of the now line, each with its outcome word. At least six markers sit to its right.
10. The schedule list has seven rows, each with cadence in words, next run, last outcome badge, average cost and a switch.
11. The paused schedule's row is dimmed, its switch is off, and the timeline shows no upcoming markers for it.
12. The panel lists the next five runs for the selected schedule, each with a Skip control.
13. The status line names the next run and its moment in words. The header reads the active count over the total.
14. At 1280 wide the timeline spans the full width, and its ticks and the now line are visible.
