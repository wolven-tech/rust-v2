# Baton Pass Log

Source item: affectUI Pro `components/agent-handoff` (functional reference only; see README).

## Purpose
Follows a task as it moves through a sequence of agents, one after another. At each handoff it shows what was forwarded to the next agent and what was lost along the way, so a person can locate where context dropped out of a pipeline and can assign a task to an agent manually.

## Data
- `relay_agents`: `{ id, name, model_label?, picture?, role_text? }[]`, in relay order.
- `assignments`: `{ id, caption, picture?, stage: "todo" | "working" | "done" | "warning", holder_id?, memo? }[]`. Transfers identify assignments by `caption`, which determines the picture and current position. The first agent displays its next `todo` assignment as "Up next".
- `transfers`: `{ id, from_agent_id, to_agent_id, task_ref, payload: { id, text, entry_type?: "file" | "source" | "fact" | "decision" | "note", dropped?, drop_reason? }[] }[]`. Each transfer is drawn as a card in the column between its two agents, newest first. A new `id` plays the transit animation.
- `on_hand_over(assignment, agent)`: fires when a card is released over a target agent that sits further along the relay than the card's current column. The host updates the assignment and the transfers.
- `on_reinstate(transfer, payload_item)`: enables "Add back" on a dropped item.
- `extra_class?`

The component only draws; the host drives the relay. On `on_hand_over` the host sets the assignment to `working` under that agent. When the work finishes, the host pushes a transfer to the next agent and advances the assignment, or sets it to `done` or `warning` at the end of the chain. Cards only ever move forward through the relay, never backward.

## Anatomy
From top to bottom:
1. Agent row: one card per agent in relay order. Each card shows name, model label, a state badge (Idle, Working, or Ready for a task), role text, and a bottom slot displaying "Up next" with the upcoming assignment, or the assignment the agent holds with its stage.
2. Relay area: between each adjacent pair of agents, a column of transfer cards. Connectors run from each sender card and into each receiver card.
3. Transfer card: a header showing `from → to` and the number of payload items; the assignment caption; a summary of the items (first few texts); and a status line reading "With <agent>" while in progress, or "Looks good" or warning text once reviewed.
4. Payload items: a labelled list. A dropped item is struck through and tagged "dropped"; its reason appears on hover or on activation.
5. Assignment card (pending list): can be dragged or otherwise handed to an agent.

## States
- Agent: idle, working, or ready for a task.
- Assignment stage: todo (waiting or up next), working, done, or warning.
- Transfer: in transit (a short light runs along its line when new), or settled.
- Item: normal, dropped (struck through, reason available), or restored (after "Add back").
- Card: collapsed (default) or open (shows everything that travelled along; opening a dropped item reveals its reason).
- Drag: the card is lifted and follows the pointer; agents it can move to are highlighted; agents it has already passed are muted.

## Interactions
- Releasing a dragged assignment card over a target agent further along the relay calls `on_hand_over(assignment, agent)`. Dropping it on an agent to the left or in the same column does nothing.
- Clicking a transfer card expands it in place to show all payload items.
- Clicking a dropped item shows why it was dropped and offers "Add back", which calls `on_reinstate`.
- Keyboard: Tab moves through pending assignments, then agents (while a card is picked up), then transferred items. Enter or Space expands a card, or lists the assignments an agent holds. With a card picked up, Enter hands it to the focused agent. On a dropped item, Enter shows the reason.
- Non-drag alternative: activate an assignment card to pick it up, then activate an agent to its right to hand it over.
- Escape cancels a picked-up card. Whether Escape closes an open card is not specified by the reference.

## Motion (functional)
- On each hand-off, a brief glint runs down the rounded connector into the receiving column's head. The card settles in, and the glint carries on to the next agent.
- A card expands in place to reveal its contents; a dropped item expands to show its reason.
- A dragged card follows the pointer. Agents it can move to rise and show a highlight; agents it has already passed step back.
- Cards keep a fixed size; the columns fade out toward the bottom.
- Reduced motion: everything appears at once.

## Accessibility
- Assignments can be handed over without dragging: activate a card to pick it up, then activate the target agent.
- Each column and each transfer's payload list is a labelled list.
- Cards and dropped items are buttons that expose `aria-expanded`.
- A dropped item is not conveyed by strike-through alone: its label includes "dropped", and its reason can be reached.

## Responsive
- Below a medium width, the status is shown as an icon only.
- Below a narrow width, the relay stacks into a single column: agent, its outgoing transfers, the next agent, and so on.
- Light and dark themes are supported through an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Card (agent, transfer, assignment), badge (agent state), status line, list (payload), connector drawing (SVG path with rounded corners), drag and drop with a keyboard alternative, dialog or inline expansion (card and item detail), tooltip (dropped reason).

## Acceptance checks
1. Agents render in the order given; each shows its name, model label, state, and current or next assignment.
2. A transfer renders as a card in the column between its two agents, newest on top.
3. Adding a new transfer id plays one transit animation; re-rendering with the same ids plays none.
4. A dropped item is struck through, labelled "dropped", and its reason can be reached by activation.
5. "Add back" appears only on dropped items and calls `on_reinstate(transfer, payload_item)`.
6. Dragging an assignment onto an agent to the right calls `on_hand_over(assignment, agent)`; dropping it onto an agent to the left does nothing.
7. Picking up and handing over work from the keyboard alone (Enter to pick up, Enter on the target to hand over).
8. Clicking a transfer card expands it to show all its payload items in place.
9. Below the narrow width, the relay is a single column in relay order.
10. With reduced motion on, no connector light or card drop animates.

## Amendment: structure and demo

**Archetype: relay row with hand-over columns.** Agent cards sit in one row in relay order. Between each adjacent pair of agents is a column that collects the transfer cards for that hand-over, newest on top. Connector lines run from each sender card into each receiver card. Below the row sits a list of assignments that have not started.

**Parts that must exist**
- Agent row: one card per agent, showing name, model line, state badge (Idle, Working or Ready for a task), role text, and a bottom slot that reads "Up next" or names the held assignment with its stage.
- Hand-over columns: one between each adjacent pair of agents.
- Transfer card: header reading "from, to" with the item count; the assignment caption; a summary of the first items; a status line.
- Payload list: a labelled list of items. A dropped item is labelled "dropped" and gives its reason on activation.
- Pending list: one card per assignment in the todo stage, with its stage word.

**Gallery demo counts**
- 4 agents in relay order: Intake, Drafter, Checker, Publisher.
- Transfer cards: 2 in the Intake-to-Drafter column, 2 in the Drafter-to-Checker column, 1 in the Checker-to-Publisher column.
- Items: 2 or 3 per transfer card; 1 dropped item, in the Drafter-to-Checker column.
- Assignments: 2 in the pending list; 3 held by agents.

**Visible at rest**
- Agent words: Intake Idle with "Up next"; Drafter Working; Checker Ready for a task; Publisher Idle.
- Transfer status lines: "With Checker" on one card, "Looks good" on another, warning text on the Checker-to-Publisher card.
- One dropped item, labelled "dropped".
- Assignment stages todo, working, done and warning, each with its stage word.

**Acceptance checks (continued)**
11. The four agent cards render in relay order; the three hand-over columns hold 2, 2 and 1 transfer cards from left to right.
12. Every assignment shows its stage word; at rest the words for todo, working, done and warning all appear.
13. The dropped item's label contains "dropped"; its reason appears on activation; "Add back" is offered on that item only.
14. A transfer card's header names both agents and the item count, and that count equals the number of items the card lists.
15. The pending list holds only assignments in the todo stage.

## Amendment 2: structure and demo

This amendment replaces the first amendment's column archetype, its agent count, and check 11. Checks 16 to 19 replace it. Check 12 of the first amendment still applies.

**Archetype: agent row above, hand-over lanes beneath it.** Agent cards sit in one row, in relay order, across the top. Beneath the row sits one lane for each adjacent pair of agents, centred under the gap between its two agents. Connector lines run down from the sender card and from the receiver card into that lane's header. The pending list sits below the lanes.

**Parts that must exist**
- Agent row: one card per agent, showing name, model line, state badge (Idle, Working or Ready for a task), role text, and a bottom slot reading "Up next" or the held assignment with its stage.
- Lanes: one per adjacent pair. Each lane header reads the pair and the item count of the cards it holds. Transfer cards stack beneath it, newest first, and the lane dims toward its lower edge.
- Transfer card, payload list and dropped-item rules, unchanged from the first amendment.
- Pending list: one card per todo assignment.

**Gallery demo counts** (these replace the first amendment's four agents and three lanes)
- Three agents in relay order: Intake (Idle, with "Up next"), Drafter (Working), Checker (Ready for a task).
- Two lanes: Intake to Drafter holds two transfer cards; Drafter to Checker holds two.
- Each card lists two or three items. One item, in the Drafter to Checker lane, is dropped.
- Two todo assignments in the pending list.

**Visible at rest**
- The agent row with its three state words.
- Two lanes of two cards each. One card reads its warning text, one reads "Looks good", and one reads "With" followed by the receiving agent.
- The stage words todo, working, done and warning each appear at least once.
- One dropped item, labelled "dropped".

**Acceptance checks (continued)**
16. Agent cards render in relay order in one row. No transfer card sits between two agent cards in that row.
17. Each adjacent pair has one lane beneath the row, centred under the gap. The Intake to Drafter and Drafter to Checker lanes hold two transfer cards each at rest.
18. Connectors run from the sender card and the receiver card into the lane header. No connector crosses another lane.
19. A lane dims toward its lower edge, so no transfer card in the gallery sits wholly within the dimmed part.
