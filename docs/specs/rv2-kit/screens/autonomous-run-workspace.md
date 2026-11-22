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
