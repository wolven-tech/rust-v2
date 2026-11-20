# Agent Tool Explorer

Source item: affectUI Pro `screens/agent-tool-calls` (functional reference only; see README).

## Purpose
Presents one agent, the tools it is allowed to invoke, and every invocation it has made. A person can see which tool is being called right now and which call failed. They can set each tool to run freely, to ask first, or to stay off. They can approve or reject a waiting call, connect or disconnect tools, and pause the agent.

## Data
- `agent_info`: `{ id, entry_name, task_name, model_name, provider?: "anthropic" | "openai" | "google" | string, is_paused: boolean, agent_state }`.
- `tool_list`: list of `{ id, entry_name, function_name, mark?, source_label, entry_description, access_mode: "auto" | "ask" | "off", parameter_list: [{ entry_name, type_name, is_required? }] }`. `source_label` is a small tag such as "Built-in" or the name of the server the tool came from.
- `call_log`: list of `{ id, tool_id, started_at, duration?, call_state: "running" | "waiting" | "done" | "failed" | "rejected", input_data, output_data?, error_text?, token_count?, run_id }`.
- `now_at`, `is_live` (boolean), `time_range`: `"today" | "7d" | "30d"`, `chart_layout`: `"org" | "compact" | "canvas"`.
- `is_loading`, `is_empty`.
- Callbacks: `on_select(selection | null)`, `on_resolve(call, "approve" | "reject")`, `on_access_change(tool, access_mode)`, `on_disconnect(tool)`, `on_add_tool()`, `on_pause(paused)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Header: the agent's name and state, a Connect tool button, and a Pause agent button.
3. Summary tiles: failed calls, total calls, success share, average duration, and tokens.
4. Chart: the agent card at the top, with its tools below it in rows joined by connector lines (org layout). The compact and canvas layouts are alternatives. Each tool card shows its mark, name, source tag, and access badge. A tool card that is in use shows a live indicator.
5. Tool or agent panel (right). For a tool: a segmented control for access (Allowed, Ask first, Paused), the description, parameters, recent calls, and a Disconnect action at the foot; Disconnect requires a second confirmation. For the agent: the model, the task, and the Pause or Resume action.
6. Call table (bottom, resizable from its top edge): time, tool, input summary, state, duration, tokens, run. A waiting call's row shows Approve and Reject buttons. Opening a call row reveals its full input and output.
7. Range menu in the header: today, last seven days, last thirty days.

## States
- **access auto**: the tool card shows Allowed; the agent may call it without asking.
- **access ask**: the card shows Ask first; each call waits for approval.
- **access off**: the card shows Paused; the agent cannot call it.
- **running call**: the connector to the tool shows a moving highlight; the call's duration counts up.
- **waiting call**: the row and the panel show Approve and Reject.
- **failed call**: the row says failed in words; the error appears in the panel.
- **rejected call**: the row says rejected.
- **agent paused** (`is_paused`): the agent card shows Paused; no new calls appear.
- **agent without tools** (`is_empty`): the agent card alone, with a dashed slot for the first tool; the table offers Connect tool.
- **loading** (`is_loading`): placeholder cards, placeholder lines, and placeholder table rows, with no figures.

## Interactions
- Tab enters the chart as a single stop; focus starts on the agent card, or on the last card that had focus.
- Arrow keys move to the nearest card in that direction (from agent to tools, along a row, between rows).
- Home and End move to the agent card or the last tool.
- Enter opens the focused card's panel, or closes it. On a call row, it opens that call.
- Escape closes a picker or the panel and returns focus to where it came from.
- Inside the table, up and down arrows step through calls; Tab reaches the table as one stop.
- Canvas layout only: Alt with the arrow keys pans; plus and minus zoom; a dedicated key fits the view.
- Approve and Reject on a waiting call call `on_resolve`.
- The access segmented control calls `on_access_change`.

## Motion (functional)
- While a tool is in use, a moving highlight travels along its connector from the agent to the tool, and the call's duration counts up.
- New calls rise into the top of the table.
- Opening a panel fades the other cards and lines; the chart shifts aside to make room.
- Reduced motion: nothing flows, pulses, or slides; lines stay visible and state labels describe what is happening.

## Accessibility
- Cards are presented as a list, with name, source, state, access, and figures.
- The table names each call in full: time, tool, state, duration, and input.
- Tabs, the access control, and the range menu follow the standard ARIA patterns.
- The side panel is exposed as a landmark region; at phone width it turns into a modal sheet that traps focus and hands it back on close.
- Focus rings appear for keyboard users only; rows inside panels draw their ring inside their edge.

## Responsive
- Tools flow onto additional rows when width runs out; a connector drops from each card to the card on the row below.
- The call table is capped at half the screen height and is resized by dragging its top edge. The chart fills what remains and is kept centred.
- When the container narrows, the run column is dropped first, then the tokens column.
- Phones: the table keeps time, tool, state, and duration; buttons become icons; the panel opens as a sheet rising from below.
- Touch targets are enlarged for touch input.

## Composition hints
Org-chart nodes with connector lines; identity card; tool card with mark and access badge; segmented control; stat tiles; data table; side panel or sheet; confirmation dialog; range menu; empty dashed slot; state badges; skeleton; live region.

## Acceptance checks
1. A paused agent's card shows Paused, and no new calls appear in the table.
2. Changing a tool's access calls `on_access_change`, and the tool card's badge changes to match.
3. Approve on a waiting call calls `on_resolve(call, "approve")`, and the row changes state.
4. Disconnect asks for a second confirmation; after confirming, the tool card is removed and its calls remain in the table.
5. The table lists calls newest first.
6. The failed-calls tile equals the number of calls with state failed in the selected range.
7. An agent with no tools (`is_empty`) shows the dashed slot and the Connect tool action.
8. With focus on the agent card, the right arrow key moves focus to the nearest tool card.
