# Single Run Status Card

Source item: affectUI Pro `components/agent-monitor` (functional reference only; see README).

## Purpose
Keeps one agent run in view while people work elsewhere: the current step, the elapsed time without paused stretches, the spend so far, and the approval the run needs before a risky action. It can fold down to a small bar.

## Data
- `run_phase`: one of `standby`, `active`, `awaiting_grant`, `held`, `complete`, `errored`, `stopped`.
- `worker_identity`: `{ display_label: string, role_label?: string, mark?: icon element }`. Without a mark, the first letter of the label is shown.
- `step_rows`: list of `{ step_key: string (unique), step_label: string, step_kind?: "inspect" | "find" | "modify" | "execute" | "reason" | "network", step_state: "pending" | "in_progress" | "succeeded" | "errored", started_at_ms?: number, elapsed_ms?: number, tool_mark?: icon element, tool_label?: string, detail_text?: string, target_text?: string }`. `target_text` names the command run or the place touched.
- `started_at_ms?`, `ended_at_ms?`, `paused_total_ms?`: numbers. The clock counts from start to end, minus paused time.
- `spend_usd?`: number.
- `pending_request?`: `{ request_title: string, request_command?: string, request_icon?: icon element }`. Shown while `awaiting_grant`.
- `result_summary?`: `{ lines_added?: number, lines_removed?: number, items_touched?: number, note_text?: string }`. Shown once `complete`.
- `error_text?`: string. Shown when `errored`.
- `focus_label?`: string. Overrides the current-step wording while `active`.
- `on_grant()`, `on_refuse()`, `on_hold()`, `on_resume()`, `on_stop()`, `on_retry()`, `on_open_log()`.
- `start_collapsed`: boolean, default false.
- `class_name?`.

## Anatomy
Expanded:
1. Header: worker identity, phase tag, spend figure.
2. Current-step line with the elapsed clock.
3. Time strip: one segment per step, widths by duration. Each segment opens that step's details.
4. Step list: finished steps as rows, most recent at the top. Scrolls after three rows.
5. Footer: controls for the current phase.

Collapsed: a single bar with identity, phase, clock and a stack of the tool icons used so far. Activating it restores the expanded card.

## States
- `standby`: no steps yet. Clock stopped. No footer controls.
- `active`: current-step line shows the in-progress step. Clock runs. Footer: hold, stop.
- `held`: clock frozen. Footer: resume, stop.
- `awaiting_grant`: the pending request is shown with its command. Footer: grant, refuse.
- `complete`: result summary shown. Clock stopped at the end time. Footer: none.
- `errored`: error text shown. Footer: retry, open log.
- `stopped`: final time shown. Footer: open log.
- Step details may be open in any phase; they close on outside press or Escape.

## Interactions
- Tab order: collapse control, step list (when scrollable), footer controls.
- Up or Down with the step list focused: scrolls through finished steps.
- Enter or Space on a step row: toggles its details; focus moves to the details' close control.
- Enter with the card focused in `awaiting_grant`: grant.
- Escape with the card focused in `awaiting_grant`: refuse. If a details panel is open, Escape closes the panel and returns focus to its row, and does not refuse.
- Hover or focus on a step row or its time segment: highlights the partner element.
- Press outside an open details panel: closes it.

## Motion (functional)
- A new step enters the current position from below. A finished step moves to the top of the list and the others shift down.
- The phase tag changes with a short vertical transition.
- Collapsing and expanding keep the clock readable throughout.
- Reduced motion: no transitions.

## Accessibility
- The region is named after the worker, its phase and the step in progress.
- The collapsed bar is a button with the same accessible name.
- The time strip is presented as a single graphic with a text label that enumerates steps and durations. Step rows form a list; icons are skipped by assistive technology.
- Each row is a button with expanded state. Details are a labelled dialog.
- Spend and elapsed time are spoken in words.

## Responsive
- A compact fixed width that narrows with its container.
- Long step labels and commands end in an ellipsis.

## Composition hints
Card, list, buttons, dialog, segmented progress strip (custom), tag (badge), icon element, timer display, collapsed bar as a button.

## Acceptance checks
1. The clock excludes `paused_total_ms` and stops while `held`.
2. Grant in `awaiting_grant` calls `on_grant` once; refuse calls `on_refuse` once.
3. Escape with a details panel open closes only the panel.
4. Activating a step row opens its details, focus moves to the close control, and closing returns focus to the row.
5. A finished step appears at the top of the list; the list scrolls after three rows.
6. `complete` shows the result summary; `errored` shows the error text and a retry control.
7. Collapsing and re-expanding keeps phase, clock and steps.
8. Spend and elapsed time are readable by assistive technology in words.
9. Unverified in demo: the `awaiting_grant` state and its grant and refuse actions (not triggered). Headless hold and step-details activation ran without page errors; visual result of hold not inspected.
