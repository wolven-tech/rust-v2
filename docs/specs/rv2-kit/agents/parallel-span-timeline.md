# Parallel Span Timeline

Source item: affectUI Pro `components/agent-lanes` (functional reference only; see README).

## Purpose
Shows several workers, and the units of work each has done or is doing, on one shared time axis that ends at the present. A person sees who is busy, who is unassigned, and which unit is held for a decision.

## Data
- `worker_rows`: list of `{ worker_key: string (unique), worker_label: string, model_label?: string, worker_mark?: icon element }`. Rows render in the order given.
- `units`: list of `{ unit_key: string (unique), worker_key: string, unit_label: string, phase: "settled" | "active" | "held" | "broken", begins_at_ms: number, ends_at_ms?: number, hands_off_to_key?: string, decision_prompt?: string, tool_marks?: icon element[] (max 3), highlights?: string[] (max 3) }`. No `ends_at_ms` means the unit reaches the present. `decision_prompt` is required when `phase` is `held`.
- `window_span`: duration on the same clock as the unit times; how far back the axis reaches. Default: a short recent window.
- `clock_ms`: number, optional. A controlled "present". Without it the component follows real time.
- `heading`: string, default "Workers".
- `on_confirm(unit)`, `on_decline(unit)`: decision outcomes for a held unit.
- `on_details(unit)`: optional. When given, non-held units offer a details action in their tooltip.
- `class_name`: optional extra class on the root.

## Anatomy
In reading order:
1. Header: heading, count of active units, count of held units, window length.
2. Time axis: tick labels at regular intervals before the present, and a marker at the present.
3. One row per worker: identity block (mark, label, model label) and a track.
4. Units on the track as segments placed by their begin and end times. Open-ended units grow to the present.
5. Hand-off connectors: a curve from the end of a unit to the start of the unit named in `hands_off_to_key`, drawn when both are in view.
6. Held units: a decision marker at the segment's end.

A worker with no units in the window shows an "Unassigned" label on its identity block.

## States
- `settled`: static segment. Tooltip shows `highlights` and, if `on_details` is given, a details action.
- `active`: segment ends at the present and lengthens as the clock advances. A live indicator shows work in progress.
- `held`: segment ends in a decision marker. Activating it opens the decision popover with `decision_prompt`, a confirm action and a decline action. The worker row shows an "Input needed" tag.
- `broken`: segment in an error treatment. Tooltip shows the same content as `settled`.
- Units that begin before the window: clipped at the left edge, sliding out as the window moves.
- Header counts update on every phase change.

## Interactions
- Pointer hover on a unit: tooltip with time range and highlights.
- Activate a held unit: opens its decision popover.
- Confirm: calls `on_confirm(unit)` and closes the popover. Without the handler, the component sets the unit to `settled`.
- Decline: calls `on_decline(unit)`. Without the handler, the component sets the unit to `broken`. (Design choice: the source does not document the no-handler outcome.)
- Tab: moves unit to unit, row by row, left to right by time.
- Enter: opens the tooltip on a non-held unit, or the decision popover on a held unit with focus on confirm.
- Escape: closes the decision popover and returns focus to its unit.
- Unverified in demo: the decision popover open and its confirm action (headless activation of a held unit did not respond).

## Motion (functional)
- The window moves with the clock. Units and hand-off curves move together.
- Units leaving at the left edge slide out and fade.
- Active units carry a live indicator. Held units pulse to draw attention.
- Reduced motion: everything is still; only content updates.

## Accessibility
- Each worker row is a labelled group named by the worker label.
- Each unit is a button whose accessible name includes worker, unit label, phase and time range.
- A held unit's trigger exposes expanded state. The popover is a labelled dialog and confirm receives focus when it opens.
- Count of held units is announced in a polite live region when it changes.

## Responsive
- Fills its container up to a comfortable reading width.
- When the container is narrow, the worker identity collapses to its mark. The label stays in the accessible name.

## Composition hints
Panel container, list, button, popover/dialog, tooltip, badge (tags), a custom timeline track with positioned segments and connectors, a tick-label row.

## Acceptance checks
1. A unit without an end extends to the present and lengthens as `clock_ms` advances.
2. Units are placed by their times; a unit beginning before the window is clipped at the left edge.
3. Tab visits each unit in worker order, then by time.
4. Enter on a held unit opens the dialog with confirm focused; Escape closes it and returns focus to the unit.
5. Confirm calls `on_confirm` once with the unit; decline calls `on_decline` once.
6. The held count in the header equals the number of `held` units and is announced politely when it changes.
7. A connector runs from the end of a unit to the start of the unit named in `hands_off_to_key`.
8. In a narrow container the worker label is hidden visually but remains in the accessible name.
9. With reduced motion, no animation runs and content updates are visible.

## Amendment: structure and demo

**Archetype: swimlane time chart.** One row per worker on a shared time axis that ends at the present. Units are segments placed by their start and end times. A tick row runs along the top of the axis, and the right edge is the present.

**Parts that must exist**
- Header: heading, count of active units, count of held units, window length.
- Tick row: labels at regular intervals before the present, and "now" at the right edge.
- Worker rows: an identity block (mark, label, model line, tag) and a track.
- Unit segments on each track, with hand-off connectors from the end of one unit to the start of the unit it names.
- Decision marker at the end of each held unit, opening the decision window.
- Tooltip for every other unit.

**Gallery demo counts**
- 5 workers: Reader, Scout, Planner, Tester, Notifier.
- Window: 10 minutes. Tick labels at 10, 8, 6, 4 and 2 minutes before now, then "now".
- 7 units: Reader 2, Scout 2, Planner 1, Tester 2.
- Scout's first unit starts before the window.
- 1 hand-off connector, from the end of Scout's second unit to the start of Planner's unit.

**Visible at rest**
- Two active units reaching the present: Reader's second unit and Tester's second unit.
- One held unit on Planner's row, with its decision marker and an "Input needed" tag on the row.
- One broken unit on Tester's row.
- Scout's first unit clipped at the left edge.
- Notifier's row with the "Unassigned" label and no segment.

**Acceptance checks (continued)**
10. The gallery renders seven unit segments across five worker rows; Notifier's row carries "Unassigned" and no segment.
11. A unit that starts before the window is clipped at the left edge, and its accessible name gives its full time range.
12. The header reads 2 active and 1 held; the held unit's row shows an "Input needed" tag.
13. One hand-off connector runs from the end of Scout's second unit to the start of Planner's unit.
14. The last tick label reads "now" and sits at the right edge of the axis.
