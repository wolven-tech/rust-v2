# Quota Load Scheduler

Source item: affectUI Pro `components/rate-limit-planner` (functional reference only; see README).

## Purpose
Lets a person place scheduled agent runs across a single day so that their combined token load stays under an API rate ceiling. It draws each run's planned load as a curve, sums them into a total, marks where that total peaks, and lets people shift runs in time to bring the peak down. Measured load can be overlaid to compare what happened with what was planned.

## Data
| Field | Type | Meaning |
|---|---|---|
| `run_list` | `PlannedRun[]` | Host-owned list. Each entry: `run_id`, `run_name`, `start_minute` (minutes after midnight), `peak_tokens_per_min`, `rise_minutes` (time from start to peak; the host sets it), `tail_half_life_minutes` (time for the tail to fall to half; the host sets it), `run_days` (`"daily" \| "weekdays" \| "weekend"`, default daily), `curve_emphasis?` |
| `starting_runs` | PlannedRun[] | Runs used when the host does not supply `run_list` |
| `on_runs_change` | (run_list) => void | Called after every move, spread, edit, addition or removal |
| `provider_ceiling_per_min` | number | The provider's limit, in tokens per minute |
| `safety_ratio` | number 0..1 | Fraction of the ceiling to remain below. Defaults to 0.75. Above it the status reads "close to limit" |
| `window_start_hour` | number | Hour at which the 24-hour view begins. Default 5 |
| `deadline_marker` | `{ at_minute, marker_label }`, optional | A fixed moment to test against, such as a nightly batch. Runs still active at that moment are listed |
| `now_minute` | number, optional | Current time, in minutes after midnight. Runs that have already begun stay put |
| `measured_load` | number[], optional | Measured tokens per minute, sampled each minute from the window start until now. Rendered as a solid trace; the forward plan is dashed |
| `day_kind` | `"weekdays" \| "weekend"` | Runs whose days exclude this value are left out. Default weekdays |
| `focused_run_id` | string, optional | The highlighted run, owned by the host |
| `on_focus_change` | (run_id) => void | Called when a run is picked |
| `board_title` | string | Heading. Default "Rate limit" |

Malformed upstream numbers (non-numeric, negative, or a start outside the day) are replaced with safe stand-in values, so the chart still renders.

## Anatomy
In reading order:
1. Heading with the title and a status badge: under the limit, close to it, or over it, together with the peak figure.
2. Summary line: the number of runs, the day's total tokens, and the per-minute ceiling.
3. Chart: a time axis across the window; a combined load line; each run's curve at its own emphasis; a marker at the peak with its time; a hatched area where the total exceeds the ceiling; a dashed ceiling line; a solid measured-load line when supplied; a checkpoint marker when supplied.
4. Run rows: one per run, showing its name, start, peak, and a draggable bar on a timeline; each row has controls to edit settings and to remove the run.
5. Footer controls: a "spread out" action, a "move to" action with a time choice, and an "add run" action. A footer sentence describes the plan against measured load.

## States
- **Under the ceiling:** combined peak below the safety fraction.
- **Close to the ceiling:** peak between the safety fraction and the ceiling.
- **Over the ceiling:** peak above the ceiling; the hatched overflow is shown.
- **Run focused:** that run's curve is drawn at its own emphasis and its row is highlighted; the others are muted.
- **Run started:** runs that began before `now_minute` are locked in place and cannot be moved.
- **Editing:** a settings window for one run is open and holds focus.
- **Ahead of or behind measured load:** the footer states the difference and the expected peak in words.
- **Empty:** no runs match the chosen day kind; the chart shows the ceiling line with an empty-state message.

Transitions: moving or spreading runs recalculates the combined curve and the status; adding a run appends a row and a curve; removing a run removes both.

## Interactions
- **Pointer drag on a run bar:** moves the run's start while dragging; the curves follow the pointer directly.
- **Pointer, Spread out:** redistributes unlocked runs across the day to bring the peak down.
- **Pointer, Move to (footer):** shifts the focused run to the chosen time.
- **Pointer, Add run:** appends a run with default settings.
- **Pointer, row settings:** opens the settings window.
- **Pointer, remove:** deletes the run.
- **Keyboard, Tab:** visits Spread out, then each run's bar followed by its remove and settings controls, then Add run, then Move to.
- **Keyboard, Left and Right on a bar:** shifts the run earlier or later by one small fixed step.
- **Keyboard, Shift with arrows, or Page Up and Page Down on a bar:** shifts by a larger step.
- **Keyboard, Home and End on a bar:** jumps to the earliest unlocked start, or to the latest start that still fits within the day.
- **Keyboard, Enter on a bar or its settings control:** opens the run's settings window.
- **Keyboard, Delete on a bar:** removes the run and moves focus to the next run.
- **Keyboard, Escape:** closes the settings window and returns focus to its opener.
- **Touch:** dragging works; the page still scrolls when a swipe starts outside a bar. The picked row exposes its remove and settings controls without hover.

## Motion (functional)
- Curves ease to their new shape after each move; the peak marker and hatched overflow travel with them.
- Spread out and Move to glide bars to their new positions; direct drags follow the pointer without easing.
- New rows slide open and their bars grow; removed rows fold away.
- The settings window appears with a brief scale and fade.
- With reduced motion requested, every change jumps to its end state and the running indicator stops.

## Accessibility
- Each bar is a slider named after its run; its value text states the start and the peak.
- Runs that have started are disabled, and the control says so.
- The footer sentence sits in a polite region and narrates the result of the latest move.
- The settings window is a dialog; focus enters its first field and returns to the opener on close.
- The chart is decorative; its information is repeated in the status badge, the rows and the footer.

## Responsive
- Fills its container. Below a moderate width, the plot and its bars stretch edge to edge, and each run's name moves onto a line of its own above its bar.
- Malformed or missing values are guarded as described under Data.
- On touch, dragging works with a finger; the remove and settings controls appear on the picked row.

## Composition hints
- Card for the container.
- Line chart with area fill for the load and per-run curves.
- Slider (custom timeline bars) per run.
- Badge for status.
- Dialog for settings.
- Buttons and select for Spread out, Move to and Add run.
- Polite live region for the footer.

## Acceptance checks
1. With two runs whose combined peak lies above the safety fraction but below the ceiling, the status reads "close to limit".
2. Pressing Spread out lowers the combined peak or leaves it unchanged, and calls `on_runs_change` once with the new starts.
3. A run that started before `now_minute` cannot be moved by drag or keyboard.
4. Pressing Right on a focused, unlocked bar moves its start later by one step and announces the new time.
5. Overflow above the ceiling is hatched and the status reads "over limit".
6. Supplying `measured_load` draws a solid line up to the current minute and a dashed plan after it.
7. A run whose `run_days` is weekend is excluded when `day_kind` is weekdays.
8. A NaN or negative peak value does not break the chart; that run's contribution counts as zero.
9. Escape closes the settings window and focus returns to the settings control that opened it.

## Amendment: structure and demo

**Archetype:** a day-long load chart above a list of run rows. The chart and the rows share one hour axis, so moving a row moves its curve.

**Parts:**
- Heading, status badge, and the peak figure with its time.
- Summary sentence: run count, the day's total, and the ceiling per minute.
- Chart: an hour axis over a 24-hour window starting at 06:00; a combined load line; one curve per run, each identifiable without colour alone; a peak marker with its time; the portion above the ceiling marked as over-limit; a dashed ceiling line; a solid measured trace up to the current time, continuing as a dashed plan; a labelled checkpoint marker.
- Run rows: colour key, name, start time, peak, a bar on the hour axis, remove control, settings control. A started run is locked.
- Footer: a spread control, a move-to control with a time choice, an add-run control, and a sentence comparing plan with measured load (ahead or behind, and the expected peak).

**Gallery demo:**
- Instance A (at rest): four runs: "Ledger sync" at 09:00 (already started, locked), "Index rebuild" at 13:00, "Report writer" at 15:30, "Invoice render" at 19:00. Ceiling 100k tokens per minute, safety fraction 0.8, now 10:30, checkpoint "Nightly close" at 23:00, measured load from 06:00 to 10:30. Combined peak sits between the safety fraction and the ceiling.
- Instance B: same four runs, ceiling 60k tokens per minute, no measured trace. Combined peak exceeds the ceiling.

**Acceptance checks (continued):**
10. Instance A at rest shows the badge reading "close to limit", four run rows, one row marked as started and disabled, and a summary sentence giving the run count, daily total and ceiling per minute.
11. In instance A the measured trace is solid up to the current time and dashed after it. The checkpoint marker carries its name and time.
12. Instance B shows the portion above the ceiling marked as over-limit, the badge reading "over limit", and a footer sentence stating the expected peak in words.
13. A started run's bar is still announced, but as disabled. Its remove and settings controls stay in the tab order after the bar.

## Amendment 2: structure and demo

Replaces amendment 1 in four places. The gallery shows one large chart rather than two stacked state cards, and the over-limit state is reached through a control beside the card. The run rows sit on the chart's own hour axis, and a Now line crosses both. The run names are replaced by the names below. Checks 10 and 12 of amendment 1 are restated as 14 and 20.

**Layout archetype:** one card. A title row, a summary line, a full-width chart, a list of run rows aligned under the chart, and a footer with controls and a sentence. The chart and the rows share one horizontal hour axis: a run's bar starts at the axis position of its start time.

**Parts that must exist:**
- Title row: heading and status badge at the leading edge; at the trailing edge, the peak figure with its time.
- Summary line: run count, the day's total, and the ceiling per minute.
- Chart, full card width: an hour axis across a 24-hour window beginning at 06:00, labelled at a regular interval; a combined load line with a soft fill; one curve per run, each identified by its run number at its peak; a peak marker with its time; the portion above the ceiling set apart by its own fill; a ceiling line whose value is written at its leading end; the measured trace solid up to now and continuing in the plan line style after it; a labelled checkpoint vertical line; a labelled Now vertical line at the current minute.
- Run rows: beneath the chart, in run order. Each row has a leading label column (run number, name, start time) whose width equals the chart's value-axis gutter, so bars align with the axis. Then the bar on the hour axis, the peak figure, and settings and remove controls. The chart's guide lines and the Now line run through the rows.
- Footer: spread control, move-to control with a time choice, add-run control, and the sentence comparing plan with measured load.

**Gallery demo (ours):** one instance at rest, four runs in order: "Ledger sync" at 09:00 (already started, so locked), "Index rebuild" at 13:00, "Report writer" at 15:30, "Invoice render" at 19:00. Peaks of 40k, 90k, 50k and 30k tokens per minute. Ceiling 100k per minute, safety fraction 0.8, now 10:30, checkpoint "Nightly close" at 23:00, measured load from 06:00 to 10:30. The combined peak sits between the safety fraction and the ceiling, so the badge reads "close to limit".

A two-option control sits above the card in the gallery frame, outside the component: "Ceiling 100k" (default) and "Ceiling 60k". Selecting the second sets the ceiling to 60k per minute: the peak now exceeds it, the over-limit portion is set apart, the badge reads "over limit", and the footer sentence states the expected peak in words.

**Visible at rest:** one chart instance; the badge reading "close to limit"; the Now line at 10:30; the measured trace solid up to it; the checkpoint line; four run rows with the started row locked; the footer.

**Acceptance checks (continued):**
14. At rest the gallery shows exactly one quota chart instance. The two-instance split of amendment 1 is removed; the over-limit state is reached through the ceiling control (check 20).
15. The chart spans the full card width, and the Now line at 10:30 crosses the chart and every run row.
16. Each run's bar starts at the hour-axis position of its start time: the "Index rebuild" bar begins on the 13:00 guide line, and the others likewise.
17. The measured trace is solid up to the Now line, and the plan continues after it in the plan line style (check 6).
18. The checkpoint line carries its name and time (check 11).
19. At rest the badge reads "close to limit", and the summary line gives the run count, daily total and ceiling per minute (check 10, first half).
20. Selecting the 60k ceiling control shows the over-limit portion set apart, the badge reads "over limit", and the footer sentence gives the expected peak in words (check 12).
21. The "Ledger sync" row is announced as disabled, and its settings and remove controls stay in tab order after its bar (check 13).
