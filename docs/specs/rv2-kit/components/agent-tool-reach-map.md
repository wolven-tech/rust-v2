# Agent Tool Reach Map

Source item: affectUI Pro `components/tool-access-map` (functional reference only; see README).

## Purpose
Places one agent at the centre of every tool it is able to reach. A person can see which tools the agent has already used, which have failed, and which it is calling at this moment. Opening a tool lists its calls, and any single call can be replayed, either with its original input or with an edited version of it.

## Data
| Field | Type | Meaning |
|---|---|---|
| `agent_list` | `MapAgent[]` | Each entry: `agent_id`, `agent_name`, `model_name?`, `agent_mark?`, `agent_state` (`"queued" \| "running" \| "waiting" \| "done" \| "failed"`), `agent_task?`, `granted_tools`. |
| `tool_catalog` | `AccessTool[]` | Every tool in the run: `tool_id`, `tool_name`, `tool_mark?`, `tool_summary?`. Laid out in sequence; the first half sits on the left. |
| `granted_tools` | `{ tool_id, call_list? }[]` | Per agent: every tool it may use. A tool with no calls reads as not used yet; a tool missing from this list reads as no access. |
| `call_list` | `ToolCall[]` | Per tool, oldest first: `call_id`, `call_state` (`"running" \| "done" \| "failed"`), `call_input`, `call_output?`, `call_error?`, `called_at`, `duration?`, `is_replay?`. |
| `selected_agent_id` | string, optional | Controlled selection of the agent shown on the map. |
| `default_agent_id` | string | Agent shown on first render. Defaults to the first agent. |
| `on_agent_pick` | (agent_id) => void | Fired when an agent tab is chosen. |
| `on_replay` | (call, input, tool, agent) => void | Fired when replay is activated. `replay_input` is the call's own input, or the edited input parsed as JSON when the original was an object. |
| `format_time` | (time) => string | Formats the moment a call was made. |
| `board_title` | string | Heading. Defaults to "Tool access". |

## Anatomy
In reading order:
1. Heading and a counts line: the number of tools currently running, the used-of-total count for the selected agent, and the number of calls.
2. Agent tabs across the top; the map is the tab panel.
3. The map: the agent card at the centre, with tool cards arranged around it and a connector from the agent to each tool. Connectors to granted tools are solid; the rest are dashed.
4. Tool card: name, mark, a state word ("Running", "Failed", "Not used yet", "No access"), the call count, and the time of the last call.
5. Calls view (when a tool is opened): a list of its calls, each showing input, output or error, and a replay control with an input editor.

## States
- **Granted, used:** solid connector; card shows call count and last call time.
- **Granted, not used:** solid connector; card reads "Not used yet".
- **Granted, call running:** the connector carries a moving highlight from the agent toward the tool; card reads "Running".
- **Failed call:** card reads "Failed" with failure emphasis.
- **No access:** dashed connector; card is de-emphasised and cannot be opened.
- **Calls view open:** the calls panel sits over the map; the map behind it takes no input.
- **Replay with edited input:** the replay control runs the changed input; the resulting call is marked as a replay.
- **Waiting agent:** the agent card reads "Needs you".

Transitions: switching agent tabs moves the connectors to that agent's grants; a call starting or finishing updates the tool card without changing its size.

## Interactions
- **Pointer, agent tab:** switches the map to that agent.
- **Pointer, granted tool card:** opens its calls view.
- **Pointer, no-access tool card:** no effect.
- **Pointer, call:** expands its input and output.
- **Pointer, replay:** replays the call with its input.
- **Keyboard, left and right arrows:** move between agent tabs.
- **Keyboard, Tab:** moves through tool cards; Enter opens the calls of the focused one.
- **Keyboard, up and down arrows in the calls view:** move through calls and show each one.
- **Keyboard, Escape:** leaves the calls view for the map and puts focus back on the originating tool card.
- **Keyboard, Ctrl or Cmd with Enter in the input editor:** runs the changed input.

## Motion (functional)
- Each granted tool has a solid connector; the others are dashed and faint.
- During an in-flight call, a brighter spark moves along the connector toward the tool.
- Changing agent: connectors redraw outward from the centre, the agent card rises into view, and tools fade according to their new grants.
- The calls view slides over the map, and the map grows to accommodate it.
- Reduced motion: everything appears at once and the moving highlight is off.

## Accessibility
- Each agent is a tab; the map acts as that tab's panel and names the agent whose tools it lists.
- Each tool is a button whose name gives its name, its state and its call count. A tool without access says so and cannot be opened.
- Opening a tool hands focus to its call list; closing it hands focus back. While the list is open, the map beneath is not interactive.
- State is never conveyed by emphasis alone: running and failed are written as words; unused tools say "Not used yet", and others say "No access".
- Counts in the heading are announced politely when they change.

## Responsive
- Tool cards never change dimensions, so a call beginning, completing or failing leaves the layout untouched.
- Layout is driven by the map's own width. Under a narrow width, tools reflow into a grid under the agent and the connector lines are dropped.
- In the calls view, the narrow layout places the list above the selected call.

## Composition hints
- Card for the container.
- Tab list for agents.
- Node-and-connector diagram for the map.
- Card per tool with badge.
- Code block for input and output.
- Editable text area for replay input.
- Polite live region.

## Acceptance checks
1. A tool present in `tool_catalog` but absent from an agent's `granted_tools` is shown with a dashed connector and "No access", and cannot be opened.
2. A granted tool with no calls reads "Not used yet".
3. A call whose state is running shows the running word on its tool card and a moving highlight on its connector.
4. Pressing Enter on a granted tool opens its calls; Escape returns focus to that tool card.
5. Replaying a call with edited JSON input calls `on_replay` with the parsed input and marks the new call as a replay.
6. Switching agent tabs with the arrow keys calls `on_agent_pick` and shows that agent's grants.
7. Calls are listed oldest first; a new call appears at the end of its tool's list.
8. When the container is narrow, the tools move into a grid beneath the agent card and all state words remain visible.
</content>
