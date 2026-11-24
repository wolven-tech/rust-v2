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

## Amendment 6: structure and demo

Replaces: none (no earlier amendment). Overrides the at-rest state of Anatomy item 6: the configuration panel is closed at rest.

**Archetype:** full-width board with a palette column at its left, a breadcrumb bar above it, a run area along its bottom, and no configuration panel open.

**Parts visible at rest (gallery demo):**
- Sidebar: nine destinations, Workflows active, Runs showing the count 3.
- Breadcrumb bar: workspace, project, branch (Main reads Production), a status chip, and a primary Publish button.
- Palette, left column, no search field. Four groups: Triggers (4 items), Tools (5), Logic (4), Actions (5). Each item has its kind marker and label.
- Board: seven step cards. Top row: two trigger cards, the second set to the right of the first. Centre column, top to bottom: an agent card (tool count and a model mode tag), a tool card (method and address line), a branch card (two paths and its condition line). Below the branch card: two action cards side by side, the left one reached from the branch's yes output and the right one from its no output. Six cards read Ready. The trigger with no source line reads Not configured.
- Five links, all curves: first trigger to agent; agent to tool; tool to branch; branch yes to the left action; branch no to the right action. The unconfigured trigger has no link. The branch's two outputs carry yes and no tags.
- Control strip at the board's lower left: select, comment, zoom in, zoom out with the current zoom percent, fit to view, and run.
- Run area along the bottom: tabs Payload, Triggers, Errors (count 1), Timeline (active) and Runs. Timeline reads "No runs yet", with a one-line hint and a Start test run button at its right.
- Configuration panel closed. No step is focused, and the board uses the full width.

**States at rest:** no step selected; one unconfigured trigger; run area on Timeline with no runs.

**Arrangement:** sidebar, then the palette column, then the board, with the breadcrumb bar above the board and the run area along its bottom. No right column is open.

**Motion, settled frame:** nothing moves. No traveling highlight and no pulse on any status.

**Defects to fix in ours:**
- Ours shows four cards at rest. The at-rest demo has seven.
- Ours shows an empty configuration placeholder on the right. At rest the panel is closed.
- Ours shows one item per palette group. The groups hold four, five, four and five items.

**Checks (continued):**
11. At rest the board holds seven step cards: two triggers, one agent, one tool, one branch and two actions. The trigger without a source reads Not configured. The other six read Ready.
12. Five links join the cards as described. The branch's outputs carry yes and no tags, and each output can connect to a different step.
13. The palette shows four groups with 4, 5, 4 and 5 items, and no search field.
14. The run area shows tabs Payload, Triggers, Errors (with count 1), Timeline and Runs. Timeline is active and reads No runs yet, with the start-test button.
15. The sidebar's Runs entry shows the count 3, and the configuration panel is closed.
16. At 1280 x 720 without page scroll, the breadcrumb bar, the palette, all seven cards, the control strip and the run area tabs are visible.

## Amendment 8: structure and demo

Replaces, from Amendment 6:
- "Arrangement": the breadcrumb bar spans the full content width, above the palette and the board, not above the board alone.
- "Control strip at the board's lower left": the strip is centred on the board's bottom edge.
- "Run area along the bottom" (the run area's tabs and "No runs yet" text) as a parts-at-rest item: the run area is closed at rest.
- "Five links, all curves": the three stacked links are straight; the two branch outputs are curves.
- "Six cards read Ready": status words are shown on triggers and actions only (check 20).
- Checks 14 and 16 (run area tabs visible at rest). Checks 11, 12, 13 and 15 stand.
Overrides Anatomy item 5 (strip position) and item 7 (run area visible at rest).

**Archetype:** a full-height board with a palette column at its left, a breadcrumb bar across the top of the content area, a row of seven cards in a tree-like column filling most of the board, and nothing open on the right.

**Layout** (fractions of the content area to the right of the sidebar, origin top left):
- Breadcrumb bar: full content width, top 0 to about 0.06 of content height.
- Palette: x 0 to about 0.19 of content width, from below the breadcrumb to the bottom edge.
- Board: x about 0.19 to 1.00, from below the breadcrumb to the bottom edge. Dotted backdrop.

**Graph on the board** (fractions of board width and height):
- Bounding box of the seven cards: x 0.11 to 0.88, y 0.03 to 0.89. About 0.77 of board width and 0.86 of board height. Left margin about 0.11, right margin about 0.12.
- Zoom readout at rest: 94%.

**Cards** (x left, y top, width 0.275 of board width; height 0.15 for triggers, 0.12 for the others):
- "New order", trigger: x 0.27, y 0.03.
- "Every morning", trigger, unconfigured: x 0.605, y 0.03.
- "Ask an assistant", agent: x 0.27, y 0.23.
- "Look up the order", tool: x 0.27, y 0.40, height 0.125.
- "Amount over 500", logic: x 0.27, y 0.57.
- "Ask a manager", action: x 0.11, y 0.74.
- "Send receipt", action: x 0.43, y 0.74.
Stacked cards leave about 0.05 of board height between them.

**Card anatomy, all seven:**
- Kind tag: a small uppercase label (TRIGGER, AGENT, TOOL, LOGIC, ACTION) straddling the card's top-left edge.
- Header: a small outlined icon tile with a line glyph, the title on one line, and a right-side chip.
  - Triggers and actions: a status chip. "Ready" on the configured trigger and on both actions. "Not configured" in warning tone on the unconfigured trigger.
  - Agent: "3 tools" chip. Tool: method chip "GET". Logic: "2 paths" chip.
- Subtitle: one line, aligned to the title.
- Bottom band: an inset strip near the card's bottom edge.
  - "New order": one monospace source line, "shop webhook · order created", truncated.
  - "Every morning": two chips, "Weekdays" and "09:00"; the subtitle position holds a warning-tone line "Set a schedule to start".
  - "Ask an assistant": three chips: a mode chip with a small icon reading "Auto", then "Order API" and "Web search".
  - "Look up the order": one monospace request line "GET orders/{id}".
  - "Amount over 500": one monospace line "Yes → Ask a manager · No → Send receipt", truncated.
  - Actions: one monospace payload line, truncated.
- No text wraps inside a band; overflow ends in an ellipsis.

**Connectors:** dots at each endpoint.
- New order to Ask an assistant, Ask an assistant to Look up the order, Look up the order to Amount over 500: straight vertical lines from the bottom centre of the upper card to the top centre of the lower card.
- Amount over 500 has two outputs at about 0.29 and 0.71 of its width. Two curves run from them down and outward to the top centres of "Ask a manager" (left) and "Send receipt" (right). A YES tag sits at the midpoint of the left curve; a NO tag at the midpoint of the right curve.
- Unlinked outputs show a small add button under their output dot: "Every morning" and both action cards.

**Palette:** a "Nodes" title with a close control at its right. Four groups, each with its count: Triggers 4, Tools 5, Logic 4, Actions 5. Items: a line glyph and a label. No letter tiles, no search field.

**Control strip:** floating, centred on the board's horizontal centre, about 0.26 of board width wide, in the bottom 0.06 of board height (y about 0.94 to 0.98). Buttons left to right: add node, select, comment, fit to view, zoom out, zoom readout, zoom in, run. Run is the only filled button.

**Run area at rest:** closed. No tabs and no text are visible; the board runs to the bottom edge. The run control in the strip opens the run area, rising from the bottom edge and keeping the current zoom.

**Breadcrumb bar:** workspace with a small team tag, project, branch "Main" with a "Production" status chip, and a filled Publish button at the right end.

**Motion, settled frame:** nothing moves (unchanged from Amendment 6).

**Acceptance checks (continued):**
17. At rest the board holds seven cards within 0.02 of the positions above: two triggers across the top, a column of three below them, and two actions at the bottom, one under each branch output.
18. The seven cards' bounding box covers about 0.77 of board width and about 0.86 of board height. The zoom readout reads 94%.
19. Each card has its kind tag, header with icon tile, title and right chip, subtitle, and bottom band as described above. Bands truncate with an ellipsis.
20. Status words show on triggers and actions only. The configured trigger and both actions read Ready; the unconfigured trigger reads Not configured in warning tone. Agent, tool and logic cards show their chip instead; their status is still stated in their accessible name.
21. The three stacked links are straight with dots at both ends. The two branch outputs are curves with YES and NO tags at their midpoints, leaving the branch card at about 0.29 and 0.71 of its width.
22. Unlinked outputs (the unconfigured trigger and both actions) show an add button under their output dot.
23. Palette item markers are line glyphs. Each group header shows its item count. No letter tiles, no search field.
24. The breadcrumb bar spans the full content width above both palette and board. The palette has a "Nodes" title and a close control.
25. The control strip is centred on the board's bottom edge. Run is its only filled control.
26. At rest no run area is visible; the board reaches the bottom edge. At 1280 x 720 without page scroll, the breadcrumb bar, palette, all seven cards and the control strip are visible. (Replaces check 16's run-area tabs.)
