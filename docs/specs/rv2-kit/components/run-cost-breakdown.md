# Run Cost Breakdown

Source item: affectUI Pro `components/cost-waterfall` (functional reference only; see README).

## Purpose
Shows the cost of one agent run and the steps that incurred it, so a person can spot the expensive step quickly. Fits next to a conversation, within a log of runs, or on a billing screen.

## Data
- `run_label` (text, default "Run cost"): panel heading.
- `run_summary` (text, optional): purpose of the run, shown muted beside the heading.
- `status_override` (`"in_progress" | "finished" | "failed"`, optional): replaces the computed state label. Without it, the run reads in progress while any step is still going.
- `step_list` (Step[]): steps in sequence. Each step carries:
  - `step_id` (text, stable across updates)
  - `step_name` (text)
  - `step_kind` (`"agent" | "model_call" | "tool_call"`): picks the row tone and the labels in the detail view (task and result, prompt and response, input and output)
  - `step_status` (`"in_progress" | "done" | "failed"`)
  - `model_name` (text, optional), `step_note` (text, optional), `step_icon` (node, optional)
  - `in_tokens`, `out_tokens` (number, optional)
  - `step_cost` (number, optional; a negative value is a credit)
  - `step_elapsed` (number, optional; time taken, or time so far while in progress)
  - `prompt_text`, `result_text` (text, optional; shown when the row opens)
  - `error_text` (text, optional; shown when a failed row opens)
  - `child_steps` (Step[], optional; nested sub-agent steps)
- `measure` / `starting_measure` (`"cost" | "tokens"`, default `"cost"`): what the bars encode.
- `folded_step_ids` / `starting_folded_step_ids` (text[], default empty): sub-agents currently folded.
- `detail_step_id` / `starting_detail_step_id` (text or null, default null): the step whose detail view is showing.
- `max_list_height` (number, optional): above this height the rows scroll.
- `format_money` (amount => text, default: dollars to three decimals).
- Callbacks: `on_measure_change(measure)`, `on_fold_change(ids)`, `on_detail_change(id, step?)`.
- `extra_class` (text, optional).

Data rules:
- A sub-agent's cost and tokens are the sum of its children; callers supply leaf values only for steps that have children.
- Values that are not finite numbers, or that are missing, render as a dash and count as zero.

## Anatomy
Reading order:
1. Header: run label, muted summary, and a state label (In progress, Done or Failed).
2. Measure switch: two options, Cost and Tokens.
3. Summary row: total cost, total tokens split into input and output, and the costliest step. That step is a button that jumps to it.
4. Step rows in order. Each shows an expand control when it has children, then name, kind, model, input tokens, output tokens, cost and time. A bar trails each row, sized against the largest value on the axis.
5. Detail view: opens directly beneath its row and presents that step's prompt and result, or its error.

## States
- Empty (no steps): totals read zero and the list is empty. (Spec choice.)
- In progress: the state label reads In progress. The running step's bar carries a moving highlight at its leading edge, and figures still changing are flagged as such.
- Done: every step has finished; the label reads Done.
- Failed: the failing step's bar and row take the error tone, and its error text is shown in the detail view. The label reads Failed.
- Folded sub-agent: child rows are hidden; the row shows a collapsed expand control.
- Detail open: one step at a time. Opening another closes the first.
- Cost and tokens measures: identical rows in identical order; only the measure differs.

Transitions: a new step appears as in progress and the label reads In progress; a step that finishes turns done; a step that reports a failure turns failed.

## Interactions
Pointer:
- Click a row: opens its detail. Click it again: closes the detail.
- Click a folded expand control: unfolds the sub-agent. Click an open one: folds it.
- Click a measure option: switches the measure.
- Click the costliest step: unfolds its sub-agents, opens its detail, and moves focus to its row.

Keyboard (the list is a single tab stop):
- Tab: moves through the measure switch and the costliest step, then into the rows.
- Down and Up arrows: move between rows. Home and End: first and last row.
- Right arrow: unfolds a folded sub-agent; on an unfolded one, moves to its first child.
- Left arrow: folds an unfolded sub-agent; on a closed or leaf row, moves to the parent row.
- Enter or Space: opens or closes the focused row's detail.
- Escape: closes the detail and returns focus to the row. Works from inside the detail as well.

Touch: the measure options and expand controls have enlarged targets.

## Motion (functional)
- A new step enters as a row; its bar grows from the point where the previous step ended.
- An in-progress bar carries a moving highlight. Figures still changing are flagged.
- Once the total exceeds the axis ceiling, the axis advances to the following round figure and all bars shift with it.
- Switching measure moves each bar to its new length.
- Details open and close beneath their row. Folding a sub-agent removes its child rows.
- With reduced motion requested, every change appears at once.

## Accessibility
- The rows form a tree: each step is a tree item at its level, labelled with name, kind, state, cost, tokens and time. Sub-agents expose their expanded state.
- Roving focus: a single row holds a place in the tab order at any moment.
- The measure switch is a labelled group of two toggle buttons with pressed state.
- The run state is announced politely when it changes.
- Axis lines and bar decoration are hidden from assistive technology.
- Long input and output text scrolls inside its own box, which is itself a focus stop.

## Responsive
- Container-based. Wide: steps on the left, bars in the middle, numbers on the right.
- On moderate widths the input and output columns are removed. In tokens mode, the step's total tokens replace the cost.
- On narrow widths each row stacks: name, cost and time share one line, and the bar spans the full width beneath.
- Touch-sized targets for the switch and expand controls.

## Composition hints
Panel (container), segmented toggle, tree list with disclosure controls, horizontal bar, status badge, collapsible detail panel, scroll area, number formatter, live region.

## Acceptance checks
1. With three steps costing 0.04, 0.01 and 0.00, the summary names the 0.04 step as costliest, and it is the first row.
2. Switching to tokens redraws each bar from its token total and keeps the row order.
3. A failed step uses the error tone; opening it shows its error text.
4. Left arrow on an unfolded sub-agent hides its child rows; Right arrow reopens them.
5. A negative cost renders with a credit style and reduces the running total in the summary.
6. A step with `step_cost: NaN` shows a dash and contributes 0 to totals.
7. Escape while focus is inside the detail closes it and focus lands on the row.
8. On narrow widths the bar sits under its row and spans its full width.
9. A long `result_text` scrolls inside its box, and that box is reachable with Tab.
10. With reduced motion requested, a new step appears without a sliding animation.

## Amendment 2: structure and demo

This spec had no first amendment, so this is its first structural note. Checks continue from 11.

**Archetype: waterfall table.** A summary strip sits above a table with three zones, left to right: the step tree with name and kind; a cost axis across the middle, with its tick labels along the top of that zone; and the figures (input, output, cost, time) at the right. Each top-level bar starts where the running total of the rows above it ends and runs for its own value. A child's bar starts where its parent's bar starts and runs for the child's own value, so children sit inside their parent's span.

**Parts that must exist**
- Header: run label, muted summary, state label.
- Summary strip: total cost, total tokens split into input and output, and the costliest step as a button that jumps to its row.
- Table header: step, axis, input, output, cost, time.
- Axis: starts at zero and labels a round ceiling. Tick labels sit above the bar zone, not inside the rows.
- Rows as in the first spec text, with their bars placed on the waterfall.
- Detail view, opening beneath its row.

**Gallery demo counts**
- Instance one: three rows. Plan the report (agent, the costliest step, first row), Researcher (agent, expanded), and its child web search tool call.
- Instance two: four rows. Summarise thread (expanded) with Read messages (done) and Write summary (failed, detail open beneath it), and Index documents (folded).

**Visible at rest**
- Instance one: the summary strip naming Plan the report as costliest; three rows in order; Researcher's bar starting where Plan's bar ends; the child bar starting at Researcher's start.
- Instance two: the failed row with its detail open beneath it; its label reads Failed.

**Acceptance checks (continued)**
11. In instance one each top-level bar starts where the one above it ends and runs for its own cost. The axis labels zero and its ceiling along the top of the bar zone.
12. A child's bar starts at its parent's start and runs for the child's own cost, so no child bar extends past its parent's end.
13. The axis ceiling is a round figure at or above the summed total. When the total passes it, the axis moves to the next round figure and every bar shifts with it.
14. In instance two the failed row's detail is open beneath its row, and the row's label reads Failed.
15. Switching the measure to tokens rebuilds every bar's start and length from token totals and keeps the row order.
