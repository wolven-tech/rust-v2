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

## Amendment: structure and demo

Layout archetype: a card with a heading row, a tab row and a map panel. The map is radial: the agent card sits at the centre, granted tools sit in two groups on either side of it, and no-access tools sit at the outer edge of the same groups. A calls panel is closed at rest.

Parts that must exist:
- Heading row: title at the leading edge; a counts line at the trailing edge (running count, used-of-total for the selected agent, total calls).
- Agent tab row: one tab per agent. Each tab label is the agent name followed by its state word. The selected tab carries an active indicator.
- Map panel: the tab's panel. Holds the agent card at the centre.
- Agent card: mark, model name, state badge, and the current task line.
- Tool cards: each with mark, name, state badge, and a detail line (call count and last-call time, or "Not used yet", or "No access"). Granted cards are solid; no-access cards are faded and dashed-bordered.
- Connectors: one from the agent card to each tool card.
- Calls panel: opens over the map on a tool's activation. Closed at rest.

Demo data for the gallery (our own names and values):
- Three agents in tabs, one in each state word: running, queued, needs you.
- Eight tools in the catalogue. The selected agent is granted five: three with calls and two not yet used. The remaining three are no-access.
- Of the three with calls, one call is running.
- Counts line: one running, three of five used, and a total call count for the run.

States visible at rest in the gallery: one running tool with its running badge and a moving highlight on its connector; two "Not used yet" cards; three faded no-access cards with dashed connectors; the selected tab marked active; calls panel closed.

Acceptance checks:
9. At desktop width the heading row shows the title at the leading edge and the counts line at the trailing edge, on one row.
10. The gallery demo shows three agent tabs, each with its state word, and the selected tab marked active.
11. The gallery demo shows the agent card at the centre, with the first half of the granted tools on the left and the second half on the right.
12. The gallery demo shows eight tool cards: five with solid connectors, three faded with dashed connectors.
13. At rest the gallery shows one card with the "Running" badge, two cards reading "Not used yet", and no calls panel.
14. The counts line in the gallery demo reads one running, three of five used, and the total calls (SSR text test).
15. Changing a call's state leaves every card in its position and at its size (layout comparison of two renders).

## Amendment 2: structure and demo

Replaces amendment 1 in four places. The radial arrangement becomes three columns: tools the agent has called sit in the leading column, the agent sits in the centre, and the remaining tools sit in the trailing column. Connectors are elbow lines rather than spokes. The demo catalogue grows to nine tools. The counts line measures use against the whole catalogue.

**Layout archetype:** a card with a heading row, a tab row and a map panel. Inside the map panel: a leading column of tool cards, the agent card at the centre, and a trailing column of tool cards. Connectors run from each leading card into the agent card, and from the agent card to each trailing card. The calls panel is closed at rest.

**Parts that must exist:**
- Heading row: title at the leading edge; counts line at the trailing edge (running count, used-of-catalogue count for the selected agent, total calls).
- Agent tab row: one tab per agent, label the agent name followed by its state word. The selected tab carries the active indicator.
- Leading column: every granted tool that has at least one call, in catalogue order.
- Trailing column: granted tools with no calls yet, then no-access tools, in catalogue order.
- Agent card: mark, model name, state badge, current task line.
- Tool cards: mark, name, state badge, and a detail line (call count and last-call time, "Not used yet", or "No access"). No-access cards use the faded form.
- Connectors: one per tool. Granted connectors are solid; no-access connectors are faded.
- Calls panel: opens over the map on a tool's activation. Closed at rest.
- Narrow widths: the two columns stack beneath the agent card and connectors are removed; every state word stays visible.

**Demo data (ours):**
- Three agents in tabs, with state words "Running", "Queued" and "Needs you".
- Nine tools in the catalogue. The selected agent holds six grants: five with calls, one not yet used. The remaining three are no-access.
- Of the five with calls, one call is running.
- Counts line: one running, five of nine used, and the total call count for the run.

**Visible at rest:** leading column of five cards with one "Running" badge and its moving highlight; trailing column with one "Not used yet" card and three faded "No access" cards; the selected tab marked; calls panel closed.

**Acceptance checks (continued):**
16. At desktop width the heading row shows the title at the leading edge and the counts line at the trailing edge, on one row (check 9).
17. The gallery shows three agent tabs, each with its state word, and the selected tab marked active (check 10).
18. The agent card sits between two columns: the leading column holds five tool cards, the trailing column holds four (one "Not used yet", three "No access"). Replaces amendment 1 checks 11 and 12.
19. Exactly one card reads "Running", it sits in the leading column, and its connector carries the moving highlight (check 13, restated).
20. The counts line reads one running, five of nine used, and the total calls (SSR text test).
21. No-access cards and their connectors use the faded form, and their tools cannot be opened (check 1).
22. Changing a call's state leaves every card in its column position and at its size (check 15).
23. With a narrow map, the columns stack beneath the agent card, connectors are removed, and every state word remains visible (check 8).

