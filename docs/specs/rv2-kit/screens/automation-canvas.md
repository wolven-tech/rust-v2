# Automation Canvas Screen

Source item: affectUI Pro `screens/workflow-builder` (functional reference only; see README).

## Purpose
Users compose a flow from linked steps. Every flow starts at a trigger and grows through tool calls, AI steps, branching logic and actions. Each step is configured in a side panel. A test execution feeds a sample payload through the flow and reports each step as it finishes.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_name | string | Organisation named in the breadcrumb and the sidebar. |
| project_name | string | Project named in the breadcrumb. |
| path_branch | string | Branch named in the breadcrumb. The branch called "Main" appears under the label "Production". |
| canvas_nodes | { id; node_type: string; node_title: string; subtitle?: string; pos_x: number; pos_y: number; node_config?: object; node_status?: "unset" \| "ready" \| "running" \| "done" \| "failed" }[] | Steps on the board; `node_type` is one of the palette kinds. |
| canvas_edges | { id; source_node: node_id; source_port: "out" \| "yes" \| "no"; target_node: node_id }[] | Links between steps. A branch step exposes a yes output and a no output. |
| canvas_comments | { id; pos_x; pos_y; comment_text; author_name?: string }[] | Pinned notes. |
| step_results | Record<node_id, { duration?: number; credit_cost?: number; error_message?: string }> | Outcome of each step during a test execution. A failing step halts the execution. |
| payload | object | Sample input for a test execution; shown on the Payload tab of the run area. |
| node_palette | { group_name: string; group_items: { node_type; item_label; item_kind: "trigger" \| "agent" \| "tool" \| "logic" \| "action"; item_icon; item_description }[] }[] | Palette sections. |
| is_loading | boolean | Set while an existing flow is opened: placeholder cards in step positions, muted links, test execution disabled; palette and toolbar remain usable. |
| is_empty | boolean | Brand-new flow: a blank board with one centred "Add a trigger" action. |
| on_change | (nodes, canvas_edges) => void | Fired after any structural edit. |
| on_run | () => void | Fired when a test execution starts. |
| on_publish | () => void | Fired by the publish control in the breadcrumb bar. |
| on_navigate | (label: string) => void | Fired by a sidebar entry. |

The shell sidebar holds nine destinations. Two sit at the top, Workflows and Runs (which shows a count); the rest, read bottom to top, are Settings, Knowledge, Workspace, Data tables, Variables, Integrations and Templates.

## Anatomy
Reading order:
1. Sidebar on the left.
2. Breadcrumb bar above the board: workspace, project, branch, and a publish control.
3. Palette, the board's left column (turns into a sheet when space is tight). Lists the groups Triggers, Tools, Logic and Actions with no search box; each item shows its kind marker and label.
4. Board: a dotted backdrop; step cards show the kind, the title, a status word and an optional subtitle; curves join output and input ports; branch outputs carry yes and no tags; comment pins sit on top.
5. Control strip at the board's lower left: select, comment, zoom in, zoom out, fit to view.
6. Configuration panel on the right (rises from the bottom when space is tight). Opens for the focused step; holds fields for its settings and a list of its outgoing links, each with a connect control and a remove control.
7. Run area along the bottom of the board: tabs for Payload, Triggers, Errors, Timeline and Runs. The timeline lists each step with its elapsed time. A start-test button sits in this area.

Card content by kind: trigger (its source line, or "not configured"); agent (tool count and model mode); tool (method and address line); logic (branch count and condition line); action (target line and status).

## States
- **Node status.** Unset (shown as "Not configured"), ready, running, done (with elapsed time), or failed (with the error). Status is always stated in words on the card.
- **Board loading.** Placeholder cards occupy step positions; links are muted; test execution is disabled.
- **Board empty.** One centred "Add a trigger" action.
- **Step selected.** The card shows a selection frame; the configuration panel may open.
- **Configuration open.** Panel visible; focus moves into it.
- **Link being dragged.** A dashed curve tracks the pointer; compatible ports are highlighted; the port it snaps to fills in.
- **Node picker open.** Opened from a step's first free output or from an add control on a link.
- **Run in progress.** Links show a highlight moving toward the target, drawn in the sending step's marker; the receiving step lights up as the run reaches it; each finished step is added to the timeline.
- **Run failed.** The failing step shows its message and the run stops.
- **Comment open.** A comment is being composed or is still a draft; resolved comments collapse.

## Interactions
- Pointer: drag a palette item onto the board to place a step; drag from an output port to another step's input to connect; click a curve to select it; Delete removes the selected step or curve.
- Panning: drag the empty backdrop, or hold Space and drag; on touch, a two-finger drag.
- Zoom: control-strip buttons, Ctrl/Cmd with the wheel, or pinch. Fit shows the whole flow and never enlarges past actual size.
- Keyboard focus: the board is one tab stop. Arrow keys move between steps along links (down follows the first outgoing link, up returns along an incoming one); left and right move to the nearest step in the same row; Home and End go to the first and last step.
- Alt with an arrow key nudges the focused step a small amount; Alt+Shift with an arrow key moves it further.
- Enter on a step opens its configuration and moves focus there; Enter on a palette item adds it below the selected step.
- Pressing A on a step brings up the new-node picker, anchored to that step's first unused output.
- E on a step focuses its first link; left and right cycle through that step's links; Delete removes the focused link; Escape returns focus.
- Delete on a step removes it; Ctrl/Cmd+Z restores it.
- C selects the comment tool; a click on the board places a comment; Enter posts it.
- Escape closes the topmost layer first: a link drag in progress, then the node picker, then an open comment, then the configuration panel, then the palette sheet, then the run area, then the comment tool, and finally the selection.
- Top edge of the run area: arrow keys resize it; Home and End jump to its smallest and largest sizes.

## Motion (functional)
- Backdrop dots move with the board when it pans or zooms, but never change size.
- Opening the run area keeps the current zoom. During a test, the view follows the running step without changing zoom.
- Palette and configuration panel slide in; the run area rises from the bottom and the board makes room.
- Test run: a highlight travels along each link in the sending step's marker; the arriving step's input lights up; the running card gets a surrounding trace, then turns into a check mark with its elapsed time.
- Reduced motion: no traveling highlight and no trace; the running step shows a steady ring in its kind's marker; links light up without drawing in; nothing slides.

## Accessibility
- The board is one tab stop with roving focus across steps.
- Each step is a labelled button stating its kind and status; the selected step is marked pressed.
- Links are focusable and named by their two endpoints.
- Links can be created and removed without a pointer, from the step's settings list (a "Connect to…" control and a remove button per entry).
- Menus open with Enter, Space or Down, move with arrows, Home and End, and close with Escape.
- Focus rings show only for keyboard focus.
- Test progress goes to a polite live region: one message per completed step, one at the end, and any failure is spoken with its message. Animation frames never produce a message.
- Comment pins are buttons that name their author and text.
- Each control-strip button has a label and a tooltip that includes its shortcut.
- Names too long for their space are cut short, and hovering reveals the complete text. File names keep their suffix. Durations are spoken as minutes and seconds.

## Responsive
- The flow runs top to bottom and opens at a legible zoom, never below a legibility floor on desktop.
- When width is short: the palette becomes a sheet opened by an "Add node" control; the configuration panel becomes a sheet from the bottom; the breadcrumb bar shifts past the sidebar toggle; the run area keeps only the figures that matter; touch targets grow; text fields are large enough that focusing one does not zoom the page.
- Panning is done by dragging the backdrop; on a page that scrolls, the wheel scrolls the page until the board is clicked.
- The screen fills its container.

## Composition hints
- Workspace frame (sidebar and page area).
- Breadcrumb bar with segments, a status chip and a primary button.
- Node-graph board: pan and zoom surface, draggable cards, curved edges with port labels, selection without a marquee, roving focus.
- Palette list with kind markers; node picker popover.
- Side configuration panel with form fields and an outgoing-links list.
- Tabbed run area with a timeline list, a payload viewer and an error list; resizable with a separator.
- Comment pins with a popover.
- Toolbar with icon buttons and tooltips.
- Toast or live region for run progress.

## Acceptance checks
1. Dropping a palette item on the board adds a step at the drop point and calls `on_change`.
2. Dragging from a step's output port to another step's input creates a link and calls `on_change`.
3. A branch step shows yes and no outputs, and each can connect to a different step.
4. Arrow keys move focus from step to step along links; Home and End reach the first and last step.
5. Alt+Right nudges the focused step a small amount to the right and calls `on_change` with the new position.
6. Pressing E on a step focuses its first link; Delete removes that link.
7. Starting a test run where `step_results` holds an error stops the run at that step and announces the failure once.
8. A new flow with `is_empty` set shows one centred "Add a trigger" action.
9. With `is_loading` set, steps appear as placeholder cards, the run control is disabled, and the palette stays usable.
10. With reduced motion requested, the running step shows a steady ring and no traveling highlight.
