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
- Keyboard: Tab moves through pending assignments, then agents (while a card is picked up), then transferred items. Enter or Space opens a card or an agent's assignment. With a card picked up, Enter hands it to the focused agent. On a dropped item, Enter shows the reason.
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
