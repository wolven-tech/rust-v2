# Run Timing Waterfall

Source item: affectUI Pro `components/trace-waterfall` (functional reference only; see README).

## Purpose
Maps the time and cost of an agent run onto a horizontal timeline. Every model call, tool call and sub-agent gets its own bar, labelled with token counts and spend. The longest finished model or tool call is tagged, and any branch can be collapsed into its parent. Selecting a bar opens a panel with that item's particulars.

## Data
- `trace_tree` (TraceNode): the run as a nested structure, extended while the run executes. A node carries `node_id`, `node_label`, `node_type` (`agent`, `subagent`, `model` or `tool`), `node_status` (`running`, `done` or `failed`), `begin_offset`, `finish_offset` (absent while running), `node_note`, `node_tag`, `model_label`, `prompt_tokens`, `completion_tokens`, `node_cost`, `input_excerpt`, `output_excerpt`, `failure_text` and `children`.
- `clock_point` (number, optional): current position on the timeline. Supply it for replays and demonstrations.
- `started_epoch` (number, optional): wall-clock start. Lets the component advance the clock by itself while anything is still running and `clock_point` is absent.
- `collapsed_ids`: the ids of collapsed nodes. Passing it hands ownership to the host; omitting it lets the component track the set, seeded from `initial_collapsed_ids`.
- `on_collapse_change(collapsed_ids)`: fired whenever a node is collapsed or expanded.
- `selected_node_id` (string or null): the node whose panel is open. Same ownership rule as `collapsed_ids`, seeded from `initial_selected_node_id`.
- `on_selection_change(node_id, node)`: fired when a panel opens (with the node) and when it closes (with null).
- `flag_slowest` (boolean, default true): tags the longest finished model or tool call.
- `min_span` (number): the shortest window the axis covers. It widens in steps as the run lengthens.
- `format_cost(amount)`: renders each cost figure. Without it, a currency symbol with two decimals is used.
- `heading` (text, default "Trace").

Costs and token counts belong to the node that carries them. Parent totals are summed for display, so callers must not pre-sum.

## Anatomy
Reading order:
1. Heading block: run status (live, done or failed), the root label, and totals for elapsed time, cost, tokens, model calls, tool calls and errors.
2. Node list on the left: one row per node, indented by depth, with a collapse toggle wherever children exist.
3. Timeline on the right: time ticks, one bar per node spanning its start to its end, and a marker at the current time for live runs.
4. Bar captions: elapsed time and cost at the bar's end, plus a tag on the flagged call.
5. Panel, when a node is selected: label, type, status, timing, model, tokens, cost, input and output excerpts, and error text.

## States
- **Live:** a running node has no finish point yet and extends toward the current-time marker; the status reads Live.
- **Done:** every node has finished; the axis ends at the latest finish.
- **Failed:** the failing node takes failure styling, and its error text appears in the panel.
- **Collapsed:** child rows are hidden; their calls show as small ticks on the parent bar.
- **Window grown:** the range advances a step when the run exceeds its current extent.
- **Panel open:** one node's panel is visible.

Transitions: new nodes slide in as rows and their bars extend from their start point. Removed rows fold upward, one after another.

## Interactions
- **Pointer, row:** selects the node; the toggle collapses or expands its children.
- **Pointer, bar:** opens the panel.
- **Keyboard, Tab:** brings the waterfall into the tab order as a single stop, then moves on to the panel.
- **Keyboard, Up and Down:** move between rows; Home and End go to the first and last row.
- **Keyboard, Right:** expands a collapsed node, or moves to its first child.
- **Keyboard, Left:** collapses an expanded node, or moves to its parent.
- **Keyboard, Enter or Space:** opens or closes the panel for the focused row.
- **Keyboard, Escape:** closes the panel and returns focus to the row.

## Motion (functional)
- New nodes slide in as rows; their bars grow out from their start point.
- A running bar lengthens in real time, with a highlight travelling along its leading edge; the current-time marker advances with it.
- Once the finish time exceeds the axis, the range eases up one step; ticks dissolve and bars rescale with it.
- Collapsing shuts child rows smoothly and leaves their calls as ticks on the parent bar; the toggle rotates.
- Rows that go away close up in sequence, starting from the bottom.
- The panel opens and closes with a soft scale and fade from its bar.
- Figures still climbing during a live run (elapsed time, cost, tokens) carry a faint shimmer.
- With reduced motion requested, every change applies at once.

## Accessibility
- Rows form a tree with levels. Each node is a tree item labelled with its name, type, status, elapsed time and cost. Nodes with children report their expanded state.
- Roving focus keeps a single row in the tab order, so the whole waterfall is one stop.
- The panel is a dialog named after its node. Escape dismisses it and returns focus to the row.
- Changes in run status are announced politely.
- Ticks, grid lines and the current-time marker are decorative and hidden from assistive technology.

## Responsive
- Container-driven: wide containers split into a node list (left) and a timeline (right).
- At moderate widths each row becomes a stack: name, elapsed time and cost on one line, the bar full-width beneath, and figures spread across three columns.
- At narrow widths the muted detail after each name is dropped so names keep their space.

## Composition hints
- Card for the container.
- Tree view with collapse toggles.
- Timeline bar chart with axis.
- Badges for type and the flagged tag.
- Dialog for the panel.
- Code block for input and output excerpts.
- Polite live region for run status.

## Acceptance checks
1. A parent with two children spans from the earliest child start to the latest child finish, and its cost equals the sum of the children's costs.
2. A running node's bar lengthens toward the current-time marker as `clock_point` increases.
3. Collapsing a node hides its child rows and shows their calls as ticks on its bar; expanding restores them.
4. The longest finished model or tool call carries the tag when `flag_slowest` is true, and no tag otherwise.
5. Enter on a node opens its dialog; Escape dismisses the dialog and hands focus back to the originating row.
6. A retried call appears as its own node with a distinct id, and the failed attempt remains in the tree.
7. Arrow keys move between rows; Right on a collapsed node expands it.
8. When the finish time passes the axis end, the axis widens to the next step.
9. At narrow widths each row stacks and the muted detail text is hidden.

## Amendment 3: structure and demo

**Replaces:** none. This spec had no earlier amendment, so checks 1 to 9 stand. The changes below alter the gallery's resting state and its heading.

**Archetype:** one live run as a full tree, every branch open at rest, with its timeline beside it.

**Parts, top to bottom:**
- Heading block: a status tag (in flight, done or failed), the root label, and six totals in one row: Clock, Charges, Token volume, Model turns, Tool runs, Failures.
- Node list (left): up to three depth levels. Every node with children is expanded at rest.
- Timeline (right): ticks, one bar per node, and a current-time marker for a live run. Each bar caption carries its elapsed time and charge. The longest finished model or tool call carries its tag.
- Panel: closed at rest.

**Gallery demo (our data):** heading "Digest run", a live run. Seven nodes:
1. Agent root, "Draft the weekly digest", in flight; its bar runs to the current-time marker.
2. Model, "Outline sections", finished, depth 1.
3. Subagent, "Fact checker", in flight, depth 1.
4. Tool, "lookup_figures", finished, depth 2, child of Fact checker.
5. Model, "Verify claims", in flight, depth 2, child of Fact checker.
6. Tool, "Fetch last week's tickets", finished, depth 1.
7. Model, "Write the draft", finished, depth 1. It is the longest finished model call and carries the tag.

Totals at rest: Clock 9 seconds, Charges 0.31, Token volume 6,200, Model turns 3, Tool runs 2, Failures 0.

**Acceptance checks (continued):**
10. At rest the gallery shows seven nodes on three depth levels, with every node that has children expanded.
11. The heading shows the six totals. Each equals the sum over the demo nodes: Model turns 3, Tool runs 2, Failures 0, and the root's charge equals the sum of the charges below it.
12. The status tag reads "In flight" while any node is running. The running bars end at the current-time marker. No panel is open.
13. At rest the flag tag sits on exactly one bar: "Write the draft".
14. Collapsing Fact checker hides its two child rows and draws their two calls as ticks on its bar; expanding restores them. When the host supplies the collapsed set, the component follows it rather than its own record.
