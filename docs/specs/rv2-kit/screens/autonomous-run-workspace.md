# Autonomous Run Workspace

Source item: affectUI Pro `screens/agent-run` (functional reference only; see README).

## Purpose
The detail view of one autonomous run. The agent works through a plan of steps, calls tools, and waits for a person before it sends, pays or publishes anything. The person sees the plan, the tool calls, any approval pending, and the draft the agent wrote, and can accept or reject the draft's changes.

## Data
- `run_record`: `{ id, entry_title, agent_name, started_at_ms, run_state: "running" | "waiting" | "done" | "stopped" | "failed", elapsed_seconds, use_count, tool_call_count, cost_label }`.
- `step_list`: list of `{ id, entry_title, detail_text?, tool_name, step_state: "done" | "running" | "pending" | "waiting", tool_call_list: [{ entry_name, summary, args_data, result_data?, duration_ms, call_state }], approval_request?: { entry_title, entry_description, risk_level, fact_rows: [{ entry_label, entry_value }] } }`.
- `draft_record`: `{ original_text, suggested_text, entry_label? }`, shown as an inline diff.
- `file_map`: map from file name to a document, sheet, code, image or folder artifact.
- `auto_play`: boolean. When true, the built-in simulation runs; when false, the host drives the steps through its own events.
- `initial_step_id`: string or null.
- `workspace_name`, `account`, `is_loading`, `is_empty`.
- Callbacks: `on_approve(step_id)`, `on_deny(step_id)`, `on_stop()`, `on_done()`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app shell).
2. Header: run title, agent name and start time, run state badge, Stop run button.
3. Summary tiles: uses, step progress (for example "1 of 5"), tool calls, elapsed time, cost.
4. Plan: numbered steps. Each step shows its title, its detail, its tool name and a state in words. A step expands to show its tool calls with arguments and results.
5. Approval card: attached to the step that waits. Shows title, description, risk level, facts, and Approve and Deny buttons.
6. Draft section: the draft as an inline diff. Each change can be accepted or rejected. "Accept all" and "Reject all" sit above it.
7. Files list (right): files the run wrote or read. Each opens in its own tab in the work area.

## States
- **running**: step progress advances; the current step shows running.
- **waiting**: the run pauses at an approval; the approval card is active and the run state reads Waiting for you.
- **denied**: the run stops at the denied step and says so.
- **done**: every step done; `on_done` fires once.
- **stopped**: after Stop; no further steps run.
- **no approval pending**: the approval area reads "Nothing needs you yet".
- **loading** (`is_loading`): placeholders for name, figures, plan, activity, approval, draft and files.
- **empty** (`is_empty`): a run without steps shows a short note.

## Interactions
- Plan steps that have not started can be renamed (Enter on the title), moved (Alt with up and down arrows) and removed (Delete). Started steps cannot be changed.
- Approval card: keyboard focus goes to the Deny button by default, so a stray Enter does not approve. Approve needs an explicit choice (Tab then Enter, or the Approve key).
- Draft changes: keys accept or reject the focused change; "Accept all" and "Reject all" apply to every change.
- Files: Enter opens a file in its tab.
- Stop run: calls `on_stop`, after confirmation.

## Motion (functional)
- Step progress animates as steps finish.
- A waiting approval card is announced once when it appears; it does not take focus from what the person is doing.
- Reduced motion: no transitions.

## Accessibility
- The approval card is a region with its title as heading. Its buttons name the step they belong to.
- A polite live region announces "Waiting for your approval: <title>" and run completion.
- The draft diff is available as text; each change says whether it is an addition or a removal, not only by marking.
- Focus rings show for keyboard users only.

## Responsive
- Stacked layout: summary, plan, approval, draft, then files. The files list moves below the draft.
- Phone: the files list opens as a sheet.
- Touch targets are enlarged on touch input.

## Composition hints
- Run header with state badge and button; summary tiles; numbered step list with expandable tool rows; approval card (risk badge, fact list, two buttons); inline diff block with accept and reject controls; file rows; live region; skeleton; empty state.

## Acceptance checks
1. While an approval is pending, the run state reads waiting and the step progress does not advance.
2. Approve calls `on_approve(step_id)` and the run continues; Deny calls `on_deny(step_id)` and the run stops at that step.
3. Stop calls `on_stop` only after confirmation.
4. Alt with the arrow keys moves a pending step; a started step cannot be moved.
5. "Accept all" applies the suggested text; "Reject all" keeps the original text.
6. Opening a file from the list shows it in its own work area tab.
7. Enter on a focused approval card's default button does not approve.
8. When every step is done, `on_done` fires once.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** run header and summary tiles over a plan column, with a right work area of two tabs, Needs your input and Activity.

**At rest (gallery demo):**
- Header: run title, agent name and start time, state badge reading Running, Stop run button.
- Five summary tiles: uses, progress (1 of 5), tool calls, elapsed, cost.
- Plan: five steps. Step 1 is done with its two tool calls expanded beneath it. Step 2 is running. Steps 3 to 5 are pending.
- Draft section below the plan: inline diff with five changes. Accept all and Reject all sit above it.
- Work area open on the Needs your input tab, whose count is 0. Its body holds the empty message, one line on what the run will ask here, and a Files list with three rows.
- Second work area tab, Activity: six entries, newest first.

**States at rest:** running; no approval pending; Needs your input tab active.

**Arrangement:** header across the top; plan and draft in the main column; work area on the right beside them.

**Motion, settled frame:** progress is at its current value. The running step shows its state as text; under reduced motion it has no moving highlight.

**Checks (continued):**
9. At rest the Needs your input tab comes first in the work area and shows count 0; its body states that no item is waiting.
10. When an approval is pending, the run reads waiting, the approval card appears under Needs your input, and that tab's count reads 1.
11. The Activity tab lists six entries, newest first, each with its time and text.
12. The Files list has three rows. Opening one adds its tab after the fixed tabs.
13. The draft shows five changes at rest. Accept all and Reject all sit above the diff.
14. Each step's tool calls appear under that step in the plan. No separate tool-call panel sits below the plan.
15. At 1280 wide the plan, the draft and the work area are visible side by side.

## Amendment 6: structure and demo

Replaces, from Amendment 5 (this amendment wins where they differ):
- The Archetype line. The work area is one fixed tab, Needs your input. Activity is a section of the main column, not a second tab.
- "Five summary tiles". The at-rest figures are four (steps done, tool calls, elapsed time, cost) and a row of app marks for uses.
- "Step 1 is done ... Step 2 is running". At rest step 1 is running and steps 2 to 5 are pending.
- "Draft section below the plan" and the Arrangement line. The draft moves to the right pane.
- The files list inside the tab body. The files list sits in the right pane under the draft.
- "Second work area tab, Activity: six entries". Replaced by an Activity section with one entry at rest.
- "Five changes". The draft shows nine changes.
- Checks 11, 13 and 15 are superseded by 17, 18 and 20. Checks 9, 10, 12 and 14 stand. Checks 16 and 19 are new.

**Archetype:** run header and figures over a plan column, with an activity section beneath the plan; a right pane holds the approval tab, the draft and the files.

**Parts visible at rest (gallery demo):**
- Header: run title; agent name with start time; state badge reading Running; Stop run button; a row of app marks for the apps the run uses.
- Four figures: steps done as "0 of 5"; tool calls, 2; elapsed time; cost.
- Plan: a one-line key hint, then five numbered steps, each with a tool-name tag. Step 1 is running, with its two tool calls beneath it: the first done, the second running. Steps 2 to 5 are pending. Plan footer: running, "1 of 5", elapsed time.
- Activity: in the main column under the plan. One entry, for step 1, with its tool-call count and duration.
- Right pane: one tab, Needs your input, with count 0 and an add-tab control. Its body states that nothing is waiting. Below it, the Draft section: a title, a count of nine changes, Accept all and Reject all above an inline diff of a short reminder email. Below the draft, a files list with three rows, each with name and kind.

**States at rest:** running; step 1 running; no approval pending; Needs your input tab active.

**Arrangement:** header across the top and figures beneath it. Plan and Activity sit in the main column. The right pane sits beside them at about a quarter of the width.

**Motion, settled frame:** progress is at its current value. The running step shows its state in words. Under reduced motion it has no moving highlight.

**Defects to fix in ours:**
- The draft block runs past the bottom of the stage and is clipped.
- Activity is hidden behind a tab and does not show at rest.

**Checks (continued):**
16. At rest step 1 reads running, with its two tool calls beneath it, one done and one running. Steps 2 to 5 read pending.
17. The Activity section sits in the main column under the plan, lists its entries newest first, and at rest shows one entry without any tab.
18. The draft in the right pane shows nine changes, with Accept all and Reject all above the diff.
19. The files list (three rows) sits in the right pane under the draft.
20. At 1280 x 720 without page scroll, the header, figures, all five steps, the Activity section, the draft with its buttons and the files list are all visible.
