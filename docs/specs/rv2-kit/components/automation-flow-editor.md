# Automation Flow Editor

Source item: affectUI Pro `components/flow-builder` (functional reference only; see README).

## Purpose
Lets a person assemble an automation in place as a top-down flow: a trigger, then steps, with forks where a condition divides the flow into named paths. People add, move, remove and undo steps, and can run a test that marks each step as it executes. It serves onboarding sequences, routing, follow-ups and agent workflows.

## Data
- `flow_name` (text, default "Untitled workflow").
- `flow_steps` / `default_flow_steps` / `on_flow_change(steps)`: the sequence; index zero is the trigger. Entry shape: `{ step_id, step_kind, step_title, step_detail?, step_note?, step_icon?, branch_paths? }`.
  - `step_kind`: `"trigger" | "action" | "agent" | "branch" | "wait"`.
  - `step_note`: a line shown while the step runs.
  - `branch_paths` (branch only): named paths, each holding its own steps.
- `step_icon` (enum): one of webhook, schedule, form, mail, message, chat, database, http, wait, branch, user, task, flash.
- `insert_templates` (Template[]): what the add menu offers. Each is `{ step_kind, step_title, step_detail?, step_icon?, branch_paths? }`.
- `trigger_templates` (Template[]): entries offered by the add-trigger action when no steps exist.
- `run_results` (map from step id to `"idle" | "in_progress" | "done" | "failed" | "skipped"`): the test run result. A path set to skipped fades out entirely.
- `selected_step_id` / `default_selected_step_id` / `on_select(id | null)`.
- `on_run()` / `on_stop()`: run the test and stop it. Running the test requires `on_run` and at least two steps.
- `canvas_height` (default: a moderate fixed height): the canvas scrolls and pans inside this height.
- `extra_class` (text, optional).

## Anatomy
Reading order, top to bottom:
1. Header: flow name, step count, and the Run test / Stop control.
2. Canvas: a vertical line running down from the trigger. Each step is a card with an icon tile, title, detail line, state tag and time.
3. Between steps: a "+" control that adds a step at that point.
4. Branch step: its paths are drawn side by side beneath it, each with a name, its own steps, and "+" controls.
5. Add menu: opens from a "+" and lists the templates; a branch template creates named paths.

## States
Per step:
- Idle; In progress (a pulse enters the step and its note animates); Passed; Failed; Skipped (faded, titles struck through, sitting in a skipped path).
Flow:
- Editing: no run active; Run test available.
- Running: Stop available; steps update as their run results arrive.
- Empty: only the add-trigger control is shown.
- Selected: one step carries a selection ring; the caller can open an inspector from `on_select`.

## Interactions
Pointer:
- Click a "+" to open the add menu for that position. Pick a template to insert it.
- Click a step to select it. Click empty space to clear the selection.
- Drag the canvas to pan. Scroll moves within the canvas.
- Move a step up or down within its path, or remove it, using the controls on the step.
- Undo control: reverts the most recent change.

Keyboard:
- Tab: steps and "+" controls in order.
- Up / Down / Home / End: move between steps in reading order, including into paths.
- Enter or Space: picks the focused step, or opens the insert menu when focus is on a "+".
- Alt/Option + Up / Down: move the focused step within its path.
- Backspace: remove the focused step; focus moves to the step before it. Removing a branch hands its first path's steps back to the flow.
- Cmd/Ctrl + Z: undo.
- Escape: close the menu, then clear the selection.

## Motion (functional)
- Steps rise in one after another as their connecting lines draw downward.
- A step that begins running gets a soft highlight; a pulse travels down the line into it; its elapsed time counts up.
- State tags swap with a small pop. A skipped path fades.
- Reduced motion: all of this is instant.

## Accessibility
- Each step is a toggle button with a pressed state; its run result is read along with its name.
- Each "+" is named for where it inserts ("Add a step after <step>").
- The insert menu is a proper menu: focus lands on its first entry, arrows traverse it, and Escape hands focus back to the "+".
- Run outcomes, deletions and undos are read out courteously.

## Responsive
- Transparent; it inherits whatever surface hosts it.
- The canvas keeps its height and scrolls in both directions; it opens centred on the flow.
- Each step has a single size, so nothing shifts during a test run; long names truncate, and the full name appears on hover.
- In a narrow container, steps shrink yet a branch's two paths stay adjacent.
- The insert menu drops below its anchor, or flips upward when space runs out, and never leaves the panel.
- Touch screens: key hints are hidden and "+" controls get larger targets.
- Right-to-left layouts mirror.

## Composition hints
Panel, header with title and buttons, scroll-and-pan canvas, step card (icon tile, title, detail, tag), connector lines, menu (listbox), icon buttons, undo stack, live region.

## Acceptance checks
1. An empty flow shows only the add-trigger control; choosing a trigger template creates the first step.
2. Adding a step after step 2 inserts it between steps 2 and 3 and calls `on_flow_change` once with the new order.
3. Adding a branch creates two named paths; each path accepts steps independently.
4. Undo after a removal restores the step to its original position.
5. Run test is disabled with fewer than two steps; with `on_run` supplied, activating it calls `on_run`.
6. During a run, a step with run result `in_progress` shows its note and the others show their results.
7. A path with run result `skipped` renders faded, and its step titles are struck through.
8. Backspace on a focused step removes it, and focus lands on the previous step.
9. Escape with the add menu open closes it and leaves focus on its "+".
10. When the container is narrow, the two paths of a branch still sit side by side with no clipping of titles.
</content>
