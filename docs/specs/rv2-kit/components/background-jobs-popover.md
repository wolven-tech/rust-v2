# Background Jobs Popover

Source item: affectUI Pro `components/running-now` (functional reference only; see README).

## Purpose
A compact control in a header or sidebar that condenses background agent work into a single summary ("3 running, 1 needs you"). Opening it lists each task with its progress and current step, and lets a person pause, resume or open a task from anywhere in the app. Tasks waiting on a person are flagged.

## Data
| Field | Type | Meaning |
|---|---|---|
| `job_list` | `BackgroundJob[]` | Each entry: `job_id`, `job_title`, `current_step`, `steps_done`, `steps_total`, `job_state` (`"running" \| "paused" \| "needs-you" \| "done"`). Input order is kept. |
| `on_pause` / `on_resume` | (job_id) => void | Icon control on running or paused rows. |
| `on_open_job` | (job_id) => void | Fired by activating a row, or by the open control on a job that needs the person. |
| `on_open_all` | () => void | Shows a footer link to the full job list and fires when it is activated. |
| `is_open` / `default_open` / `on_open_change` | boolean | Controlled or uncontrolled popover state. Closed by default. |
| `align_edge` | `"start" \| "end"` | Aligns the popover to this edge of the trigger. Defaults to end. |
| `label_text` | partial map | Every word the component shows, so it can be localised. |

Paused jobs are excluded from the "running" figure in the trigger. Progress is computed from `steps_done` of `steps_total`; when `steps_total` is zero the ring stays empty.

## Anatomy
Trigger (always visible): one pill carrying the summary: the running count, the needs-you count, or the paused count when only paused jobs remain, or an idle message when there are none.

Popover (when open), reading order:
1. Heading "Background tasks".
2. Rows: each shows a progress ring, title, current step, step count, and a state-dependent control (pause, resume, or open).
3. Footer link to the full list (when `on_open_all` is supplied).

## States
- **Idle (no jobs):** the trigger shows the idle message; the popover shows an empty line.
- **Running:** the progress ring fills as steps complete; the pause control is shown.
- **Paused:** ring frozen; resume control shown; not counted as running.
- **Needs you:** a badge reads "needs you"; the open control is shown; the job counts toward the needs-you figure.
- **Done:** full ring in a completed treatment; no state control.
- **Popover open / closed.**
- **Constrained trigger:** when both counts are present and space is short, only the numbers show; the accessible name still gives the full summary.

Transitions: a job's state change updates its row in place without changing the row's height; the trigger counts change in step with it.

## Interactions
- **Pointer, trigger:** opens or closes the popover.
- **Pointer, row:** opens the job (calls `on_open_job`).
- **Pointer, pause or resume:** calls `on_pause` or `on_resume` without closing the popover; focus stays on the same control.
- **Pointer, open control:** calls `on_open_job`.
- **Pointer, footer link:** calls `on_open_all`.
- **Keyboard, Enter or Space on the trigger:** toggles the list; opening it places focus on the first job.
- **Keyboard, down or up arrow on the trigger:** opens the list and focuses the first or last job.
- **Keyboard, up, down, Home, End within rows:** move between jobs.
- **Keyboard, Tab:** from a job to its control, then on to the next job.
- **Keyboard, Escape:** closes the popover and returns focus to the trigger.
- **Outside click or tabbing out:** closes the popover.

## Motion (functional)
- The progress ring fills as steps complete, and its treatment changes with the state.
- New step text replaces the old text with a short vertical crossfade; the trigger counts do the same.
- Reduced motion: everything changes at once.

## Accessibility
- The trigger's accessible name is the full summary, for example "Background tasks: 3 running, 1 needs you"; it exposes popup, expanded and controls relationships.
- The popover is a dialog named "Background tasks".
- Each row is a button whose name states its title, state, current step and step count.
- Each icon control names its job, for example "Pause <job title>".
- A polite live region announces when a job starts waiting on the person or finishes.
- Focus stays inside the popover: pause and resume leave focus where it was; a row that disappears hands focus to its nearest neighbour.
- Focus rings are visible to keyboard users only.

## Responsive
- Each row has the same height in every state, so a changing step or badge never shifts another row.
- The popover has a fixed width that never exceeds the screen width less a margin, so it fits a small phone.
- Touch targets are enlarged; the list scrolls inside the popover after a set number of rows, with a partial row showing that more exist.
- Right-to-left: the popover aligns to the logical edge and text follows its own direction.

## Composition hints
- Pill button for the trigger.
- Popover or dialog for the list.
- Progress ring primitive.
- List rows as buttons.
- Icon buttons for pause, resume and open.
- Badge for needs-you.
- Polite live region.

## Acceptance checks
1. Given two running jobs and one needs-you job, the trigger reads three running and one needs you, and its accessible name states both.
2. A paused job is not counted in the running figure.
3. Enter on the trigger opens the list with focus on the first job row.
4. Pressing Escape closes the list and returns focus to the trigger.
5. Pressing pause on a running row calls `on_pause` with that job's id and leaves the popover open, with focus on the same control.
6. A job that moves to needs-you produces an announcement in the live region.
7. All row heights are equal whether the row is running, paused or needs you.
8. A job with zero total steps shows an empty ring and no division error.
9. With a narrow viewport the trigger keeps its numbers and the popover stays within the screen width.
</content>
