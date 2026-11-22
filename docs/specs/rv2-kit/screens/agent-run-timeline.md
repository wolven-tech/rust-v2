# Agent Run Timeline

Source item: affectUI Pro `screens/agent-run-detail` (functional reference only; see README).

## Purpose
Shows a single agent run one step at a time along a time axis, so a person can see which steps consumed the time, which ones failed, what each received and returned, and what the run cost. While the run is live, the timeline extends as steps begin and finish.

## Data
- `run_record`: `{ id, entry_title, agent_info: { id, entry_name, identity_hue_token }, model_name, started_at, total_seconds?, step_records: [{ id, entry_label, step_type: "model" | "tool" | "sub-agent", start_seconds, duration_seconds?, nesting_depth: 0 | 1, app_info?: { entry_name, mark? }, sub_agent_name?, has_failed?, error_text?, token_counts?: { input, output }, cost_label?, input_text?, output_text? }] }`.
- `total_seconds` is absent while the run is live.
- `elapsed_seconds`: a number, supplied while the run is live. Absent once the run has finished.
- `initial_step_id`: string or null.
- `is_loading`, `is_empty`.
- Callbacks: `on_stop()` (live runs only), `on_rerun()` (finished runs only), `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Header: run title, agent and model, start time, a state badge, and either Stop (live) or Rerun (finished).
3. Summary tiles: elapsed time, cost, tokens, and step count with the failure count.
4. Timeline: a label column on the left (step name, type mark, app mark for tool steps, indented for steps run by a sub-agent). A time axis on the right, with a bar running from each step's start to its end. Bars differ by step type. Failed steps use a distinct failed variant. Sub-agent steps sit beneath their parent, indented.
5. Detail panel (right) for the selected step: name, type, start time and duration, tokens, cost, the input and output blocks (line breaks preserved, copyable), and the error message when the step failed.

## States
- **live**: running steps show a bar that is still growing and carries a moving highlight. A now line marks the current moment. The elapsed and cost tiles keep counting up.
- **finished**: every bar is at its final length; the total duration is displayed; Rerun is available.
- **failed step**: the bar uses the failed variant; the panel shows the error. A retry, when present, is a separate step after the failed one.
- **running step selected**: the panel shows what has arrived so far and marks the rest as pending.
- **empty** (`is_empty`): no steps yet; a single line says so.
- **loading** (`is_loading`): placeholder bars under the real axis; placeholder lines in the panel.

## Interactions
- J or K, or the up and down arrows: move between steps.
- Enter opens the focused step in the panel.
- Clicking a bar or a label selects that step.
- Stop calls `on_stop` while the run is live. Rerun calls `on_rerun` once it has finished.
- Copy buttons on the input and output blocks copy the exact text.
- Keyboard focus rings appear for keyboard users only.

## Motion (functional)
- Bars grow out from their start point as steps begin.
- A running bar carries a moving highlight to show that work is in progress.
- While the run is live, a now line moves along the axis.
- The agent mark may show a subtle live indicator while the run is live.
- Reduced motion: bars appear at their final length; the now line and the highlight are not shown.

## Accessibility
- Every step exposes a button whose label gives its name, kind, app, and either its duration or the words "running" or "failed".
- A polite live region announces when the run starts, finishes, or stops, and when the step count changes. It does not announce every clock tick.
- Input and output are selectable text and wrap rather than truncate.
- State is conveyed in words as well as by the bar variant.

## Responsive
- The timeline uses the full width. The axis chooses round tick intervals for any run length, and ticks near the right edge never overlap bars.
- Narrow: the label column narrows, the four figures sit in a two-by-two grid, and step names move above their bars.
- Phone: the detail panel becomes a sheet.

## Composition hints
Header with state badge and button group; summary tiles; timeline (gantt-style bars with indented rows); type and app marks; side panel or sheet; copyable code block; live region; skeleton; empty state.

## Acceptance checks
1. While `elapsed_seconds` is set, a step without a duration shows as running, and its bar extends with the elapsed time.
2. A failed step's bar uses the failed variant, and its error appears in the panel when selected.
3. A sub-agent's steps appear indented beneath their parent step.
4. Stop appears only while the run is live; Rerun appears only once it has finished.
5. Copy on the input block copies the exact input text.
6. Axis tick labels are round values suited to the run length.
7. With reduced motion requested, no bar animation or highlight runs.
8. At phone width the page does not scroll horizontally.
