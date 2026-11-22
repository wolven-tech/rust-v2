# Agent Context Map

Source item: affectUI Pro `components/memory-graph` (functional reference only; see README).

## Purpose
Displays what an assistant knows and which tools it can reach, laid out around the agent itself. Every memory shows its origin; every tool shows what it is allowed to do and where. A person can switch items off, forget a memory, or attach files and tools from the agent.

## Data
- `agent_node` (`{ agent_id, agent_name, agent_subtitle, agent_logo? }`): the agent at the centre. The subtitle changes with activity (for example "Using <tool>", "Learning", "Recalling 2 memories").
- `memory_items` (Memory[]): on the left side, in six positions filled from the centre outward. Past six, the last position reads "+N more". Each Memory:
  - `memory_id`, `memory_text`, `memory_kind` (optional), `memory_source_line` (optional)
  - `source_artifact` (`{ artifact_name, artifact_kind?, artifact_logo?, artifact_link? }`): where it came from; a link adds an open action to its details.
  - `learned_via_tool_id` (optional): the tool that taught it; a new memory's pulse travels in from that tool.
  - `replaces_memory_id` (optional), `is_outdated` (boolean): a replacing memory marks the older one outdated, and the two are linked.
  - `related_memory_ids` (text[], optional)
  - `learned_at`, `use_count`, `is_disabled` (boolean, optional)
- `tool_items` (Tool[]): on the right side; a logo row beneath the agent lists them too. Each has `tool_id`, `tool_logo` (node; first letter shown when absent), `tool_name`, `access_level`, `scope_text`, `last_activity_text`, `last_used_at`, `call_count`, `is_disabled` (optional).
- `active_tool_ids` (text[]): tools the agent is using right now. Each one sends a pulse out to it. Switched-off tools are never used.
- `recalled_memory_ids` (text[]): memories being recalled right now.
- `on_switch_change(kind, id, enabled)`: adds a switch to each card's details. Switching off dims the card and stops the agent from using it.
- `on_forget(memory)`: adds a Forget action to a memory's details.
- `available_tools`, `on_add_tool(tool)`, `on_add_files(files)`: activating the agent opens an Add panel with a drop area for files and the tools that can be connected.
- `on_select_memory(memory | null)`.
- `heading_text` (text).
- `clock_ms` (number, defaults to now): reference point for relative times such as "3h ago".
- `format_date(ms)` (defaults to a short month and day).
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Header: heading with the memory and tool counts.
2. Ring: the agent at the centre. Memories on the left half, tools on the right half, each joined to the agent by a connector.
3. Memory cards: text, source line, source mark, and state words (outdated, switched off, recalled).
4. Tool cards: logo, name, access level and scope, last activity.
5. Details panel for the selected card; Add panel opened from the agent.

## States
- Idle: cards are static; the agent subtitle shows its default.
- Using a tool: that tool's card is marked in use, and its connector carries a pulse.
- Recalling: recalled memories are marked and their connectors animate.
- Learning: a new memory arrives in its position with a pulse travelling in from its tool.
- Memory outdated: card shown more quietly; linked to its replacement.
- Disabled item: card faded; its connector is idle; the agent does not use it.
- Adding: Add panel open beneath the agent.
- Empty: no memories and no tools; the ring shows only the agent and an invitation to add. (Spec choice.)

## Interactions
Pointer:
- Activate a memory or tool card to open its details. Activate the agent to open Add.
- In details: toggle a switch, forget a memory, or open its source.
- Escape or an outside click closes the details or the Add panel.

Keyboard:
- Tab: three stops, covering the memories, the agent, and the tools.
- Up / Down: move through one side, top to bottom.
- Home / End: first or last card on a side.
- Left / Right: move from a side to the agent and onward to the other side.
- Enter or Space: opens the details of the focused card, or the Add panel when focus is on the agent.
- Escape: close the details or Add; focus returns to the card.

## Motion (functional)
- A pulse travels along a connector: outward to a tool the agent starts using, inward from a tool through the agent to whatever it learns. It is strongest at the centre and fades toward both ends.
- The card the pulse reaches is highlighted by a fill that rises from the side the pulse came from.
- A new memory or tool rises into position on arrival.
- Items being recalled or used carry a mark, and their connectors shimmer.
- The agent's mark animates while it works.
- Add opens as a small panel beneath the agent and slides into the layout.
- Reduced motion: all of this is off; items simply appear.

## Accessibility
- Memory items, tool items and the overflow entry form button lists; each name states the item's identity, origin and status.
- The details view and the Add view are dialogs titled after the item they concern; toggles are true switches reporting on or off.
- A polite status region announces newly learned facts, tools that begin running, and newly attached tools.
- Focus rings show only for keyboard focus.
- Status always appears as text too: outdated, switched off and in-use are spelled out, never shown by shade alone.

## Responsive
- Every card holds a fixed slot on the ring and every connector has its own anchor on the agent; adding an item leaves the others where they are.
- In a narrow container the groups collapse beneath the agent: memories first, then tools, both aligned to the left edge.
- Overlong labels end in an ellipsis; the complete wording surfaces on hover and inside the details view.

## Composition hints
Panel, header with counts, central avatar with status line, card list items (memory and tool), connector lines (SVG), tags for state, detail dialog with switches, add dialog with drop area, live region, pulse animation.

## Acceptance checks
1. With `active_tool_ids` holding one tool, that tool's card shows the in-use state and its connector carries a pulse.
2. Switching a memory off through `on_switch_change("memory", id, false)` dims its card, and it is no longer listed among recalled ids.
3. Forgetting a memory calls `on_forget` with that memory.
4. A memory with `replaces_memory_id` marks the replaced memory as outdated, and both show a link between them.
5. Seven memories fill six positions with a "+1 more" entry in the last one; adding an eighth does not move the first six.
6. Activating the agent opens Add; dropping a file calls `on_add_files` with the file list.
7. Escape closes the open details, and focus returns to the card that opened them.
8. When the container is narrow, memory and tool cards stack beneath the agent with no horizontal scrolling.
9. Reduced motion: no pulse animation; a new memory appears without rising.
10. A long source name truncates with an ellipsis, and its full text is available in the details.

## Amendment: structure and demo

Layout archetype: a three-column ring. Memory cards run down the leading column, the agent sits in the centre column, and tool cards run down the trailing column. Every card is joined to the agent by its own connector. The header runs across the top.

Parts that must exist:
- Header: title at the leading edge; at the trailing edge, the memory count and the tool count.
- Memory column: one card per memory, in slot order, top to bottom. Each card holds its text, a source line, a source mark (the artifact's logo, or its first letter when none), a learned-time line, and state words when outdated, switched off or recalled.
- Agent card: centred in the middle column. Holds the agent mark, the agent name and the activity subtitle.
- Tool logo row: beneath the agent card, one mark per tool. Tools in use render at full strength; the rest render faded.
- Tool column: one card per tool, in list order. Each card holds its logo, name, access level with scope on one line, a last-activity line with call count, and an "in use" tag while the tool is in use.
- Connectors: one per card, from the card to the agent. Connectors to in-use cards are drawn in the in-use style.
- Details panel: opens for one selected card, below the ring. Closed at rest.
- Add panel: opens from the agent. Closed at rest.

Demo data for the gallery (our own names and values):
- Four memories, each with a different artifact kind and source mark.
- Five tools, each with an access level, a scope and a last-activity time. One tool is in use.
- Agent subtitle names the tool in use.
- Header reads four memories and five tools.
- At least one memory is shown as outdated, linked to its replacement; that link is drawn between two memory cards.

States visible at rest in the gallery: the in-use tool with its tag and its connector, the agent subtitle naming that tool, the outdated memory link. Details and Add panels are closed.

Acceptance checks:
11. At desktop width the gallery demo shows memories in the leading column, the agent centred, and tools in the trailing column, with no card overlapping another.
12. The header shows four memories and five tools in the gallery demo.
13. The tool logo row beneath the agent shows five marks, and only the in-use one is at full strength.
14. Each memory card shows its source line and source mark; each tool card shows access level with scope and a last-activity line with call count.
15. The agent subtitle names the in-use tool while one is in use, and shows its default subtitle when none is (SSR test on both).
16. The gallery demo shows no details panel and no Add panel at rest.
17. Cards keep their slots: adding a tool or memory leaves existing cards in the same positions (SSR test comparing two renders).
18. An outdated memory is drawn with a link to its replacement, and its state word is present in its text, not only in its shade.
</content>
