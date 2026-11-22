# Multi-Agent Run Canvas

Source item: affectUI Pro `screens/agent-orchestration` (functional reference only; see README).

## Purpose
Shows a live run of several cooperating agents as a graph: which agent started which, what is running, what waits for a person, and what it costs so far. A person can open any agent to see its tasks, context and cost, and follow every call in a trace below.

## Data
- `run_record`: `{ id, entry_title, run_state: "running" | "paused" | "done" | "failed", elapsed_seconds, totals: { cost_label, token_count, agents_done, agents_total }, progress_percent }`.
- `agent_nodes`: list of `{ id, entry_name, node_role: "orchestrator" | "research" | "analysis" | "writing" | "review" | string, model_name, node_state: "running" | "done" | "queued" | "needs-you" | "failed" | "retrying", parent_id?, cost_label?, duration_seconds?, token_count? }`. `parent_id` defines the edges of the graph.
- `trace_rows`: list of `{ id, agent_id, entry_label, start_seconds, duration_seconds?, row_state, error_text? }`. A running item has no duration.
- `is_live`: boolean. When false, the clock and the flow are frozen.
- `initial_open_agent_id`: string or null.
- `is_loading`, `is_empty`.
- Callbacks: `on_pause(paused)`, `on_stop()`, `on_open_agent(id)`, `on_navigate(entry_label)`, `on_resolve(agent_id, answer)` for a waiting agent.

## Anatomy
1. Sidebar (shared app shell) with run-related entries.
2. Header: run name and short id, live badge, Pause and Stop buttons.
3. Canvas: agents laid out as a tree with the orchestrator at the top. Each node is a card with name, role lane, state badge, model, cost and duration. Edges run from parent to child. Zoom and fit controls sit in the corner.
4. Summary tiles: agents done out of total, elapsed time, tokens, total cost.
5. Lower section with tabs: Trace (time bars per agent, with a now line while live), Tokens and cost (table), Events (list).
6. Agent panel (right) for the open agent: its tasks, the context it was given, its cost and tokens, its model, and any approval it waits on.

## States
- **running**: edges show flow toward running nodes; the clock advances; the now line moves.
- **paused**: clock and flow stop; the Pause button becomes Resume.
- **done**: static graph; totals final.
- **failed node**: node shows failed state and its error; a retry appears as a sibling step in the trace, not as a replacement.
- **queued node**: dimmed; shows "queued" and no cost.
- **needs-you node**: highlighted; its panel shows the question and answer controls.
- **empty** (`is_empty`): no agents yet; the canvas shows one placeholder node.
- **loading** (`is_loading`): placeholder nodes and placeholder bars.

## Interactions
- Click a node or press Enter on it: opens the agent panel.
- Arrow keys move focus to the nearest node in that direction; Home and End go to the first and last node.
- Escape closes the panel and returns focus to the node.
- Zoom in, zoom out and fit: buttons, plus keys (+, -, and a fit key). Panning with the keyboard when the canvas has focus.
- Pause and Resume: call `on_pause`. Stop asks for confirmation first, then calls `on_stop`.
- Trace rows are selectable; selecting one opens its agent.
- Tabs move with arrow keys.

## Motion (functional)
- Flow lines run along the edges into running nodes, showing where work moves.
- A node changing state updates its badge in place.
- New trace bars grow from their start point.
- The now line moves only while the run is live.
- Reduced motion: static graph, no flow, no growing bars.

## Accessibility
- Each node is a focusable button whose name includes the agent name, role, state, model and cost.
- The graph has a list-equivalent view: the trace table gives the same information without the canvas.
- A polite live region announces changes in the number of agents done and the run state, not every clock tick.
- State is always shown in words, not mark alone.
- Stop requires a confirmation that is reachable by keyboard.

## Responsive
- The canvas scrolls and pans inside itself; the page does not scroll sideways.
- The trace uses the full width. Narrow screens narrow the label column and show the four numbers in a two-by-two grid.
- The agent panel becomes a sheet on narrow screens.

## Composition hints
- Graph canvas with tree layout; agent node card; state badge; progress bar; buttons; tabs; time-bar chart; data table; metric tiles; side panel or sheet; confirmation dialog; zoom controls; live region.

## Acceptance checks
1. Every edge goes from a node to its `parent_id`; there are no edges for nodes without a parent.
2. A queued node shows "queued" and no cost.
3. The run's elapsed time equals the end of the last trace row while the run is live.
4. Pause calls `on_pause`; Stop asks for confirmation and calls `on_stop` only after confirmation.
5. Selecting a node opens the panel with that agent's tasks and cost.
6. With `is_live` false, the clock and the flow do not change between two renders.
7. A failed agent shows its error; the retry is a separate trace row.
8. At phone width the page has no horizontal scroll; only the canvas scrolls inside itself.
