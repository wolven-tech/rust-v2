# Task Branch Scene

Source item: affectUI Pro `illustrations/project-flow` (functional reference only; see README).

## Purpose
An animated scene of a client's task splitting into subtasks, linked notes, and the people assigned to them. It shows how one piece of client work becomes a set of owned pieces.

## Data
- `client`: { label, connection_status }.
- `task`: { title, assignees: { id, avatar }[], tool_icons: node[] (supplied by caller) }.
- `notes`: list of { id, label }.
- `roles`: list of { id, title, person: { name, avatar } }.
- `extra_classes`.

## Anatomy
In reading order (top to bottom):
1. Client node at the top, with a connection status word.
2. Task card: title, assignee avatars, tool icon row.
3. Notes node: a side node linked to the task with a label.
4. Assigned-roles connector: a branch from the task down to a roles node.
5. Role card: the role title, the person, and their tool icon row.
6. Background rings: faint concentric shapes behind the scene.

## States
- Building: connectors draw from the client to the task, then to notes and roles in order.
- Settled: all nodes in place; connectors static.
- Reduced motion: settled view.

## Interactions
- Hover a node: highlights it and its connectors.
- No click behaviour by default.

## Motion (functional)
- The client node appears first; the connector to the task draws; the task card rises in.
- The notes node and the roles connector follow in order.
- The role card appears last, with its person's avatar.
- Background rings expand slowly and fade, suggesting activity.
- With reduced motion preferred, all nodes appear in their final place without drawing.

## Accessibility
- Decorative; the scene is `aria-hidden`. The parent must state the same information in text.

## Responsive
- Scales with its container; nodes may move closer together at narrow widths.
- Phone width: scene scales down; labels may be hidden.

## Composition hints
- Node cards, connector drawing, avatar, icon row, background shapes.

## Acceptance checks
1. Connectors draw in the order client, task, notes and roles.
2. Hovering a node highlights it and its connectors.
3. With reduced motion preferred, all nodes are placed without drawing.
4. Assignee avatars on the task card match the `assignees` list.
5. At phone width the scene scales down and nothing scrolls sideways.

Uses partner logos (supplied by caller) for the tool icon rows. Does not describe any brand mark.

## Amendment 8: structure and demo

Replaces: Anatomy items 3 and 4 (the side node and the roles connector as a branch of the same scene). Anatomy items 1, 2, 5 and 6 stand, detailed below. Acceptance checks 1 to 5 stand. Checks 6 to 12 are new.

**Archetype:** a tree. One stem drops from the task card and splits into two branches: the left branch ends at a stack of subtask cards, the right branch at a chain of role cards. A side node hangs off the task card's right edge. Not a ring of cards around a hub.

**Frame:** the demo fills its card, with no inner stage. The scene spans about 0.24 to 0.76 of the card width and 0.17 to 0.84 of its height, centred.

**Elements at rest (gallery demo, our names and data):**
- Client node "Client: Northwind Foods", status word "Connected". Faded (see Fade).
- Task card "Task: Q4 onboarding deck". Owners row: two round initials avatars (AL, SR). Tool-icon row.
- Side node "Client notes": small icon, label, and a meta text at its right that runs into a fade at the node's right edge.
- Subtasks card "Subtasks: draft the outline". Owners row: three avatars. No tool-icon row.
- Nested card "Subtask 1: draft the outline", in front of the lower half of the subtasks card. Owners row: one avatar. Tool-icon row.
- Role split chip "Role split", sitting on the roles connector.
- Role card "Project manager": round portrait-style avatar (DM) at its left, title, subline "Owns the timeline", tool-icon row.
- Ghost role card "Design lead": avatar (ML), title, subline, tool-icon row. Faded (see Fade).

Counts at rest: 7 cards and 1 chip. 6 connectors. 1 connector carries text. 4 tool-icon rows. 8 avatars across the cards (task 2, subtasks 3, nested 1, role 1, ghost 1).

**Tool-icon row:** at the card's bottom edge, left-aligned, spanning about 0.8 of the card width. A small tag chip reading "Stack", then square tool tiles in one row. Task card: 5 tiles. Nested card: 4. Role card: 3. Ghost role card: 3. Demo tiles are neutral glyph tiles, no brand marks; caller-supplied icons replace them.

**Positions** (fractions of the card's width and height, origin top left):
- Client node: x 0.40 to 0.59, y 0.17 to 0.24.
- Task card: x 0.41 to 0.58, y 0.27 to 0.41. Aspect about 2 to 1.
- Side node: x 0.60 to 0.76, y 0.31 to 0.37.
- Subtasks card: x 0.24 to 0.44, y 0.50 to 0.67.
- Nested card: x 0.25 to 0.44, y 0.58 to 0.72.
- Role split chip: x 0.60 to 0.68, y 0.48 to 0.52.
- Role card: x 0.55 to 0.72, y 0.55 to 0.69.
- Ghost role card: x 0.55 to 0.72, y 0.71 to 0.84.

**Connectors:** one hairline in the accent token, a small dot at each endpoint, rounded corners at elbows.
1. Client bottom centre to task top centre: straight vertical.
2. Task right-edge middle to side node left edge: straight horizontal, about 0.02 of width, no text.
3. Stem: from the task's bottom centre straight down to y 0.45, then a horizontal bar from x 0.34 (subtasks centre) to x 0.64 (role chip centre). The left leg drops from the bar to the subtasks card top. The right leg drops from the bar to the role split chip top.
4. Role split chip bottom to role card top: straight vertical, dots at both ends.
5. Role card bottom to ghost role card top: straight vertical, dots at both ends.
No connector joins the subtasks card and the nested card; the nested card overlaps it.

**Labels:** the only text on a connector is the "Role split" chip. It sits on the right leg's vertical run between the bar and the role card and interrupts the line. The side connector carries no text.

**Fade:** a fade band across the top fifth of the frame, and one across the bottom band from y 0.70 down. The client node and the ghost role card sit inside those bands and read as faded. Every other element is at full strength.

**Background:** three or four faint hairline rings, concentric on the scene's centre (0.5, 0.5). The largest reaches about 0.9 of the card width. Static in the settled frame.

**Motion, settled frame:** every element in the positions above, every connector fully drawn, the two ghosts held in the fade, rings at rest. Build order stays as in Motion: client, task, side node, subtasks leg, roles leg, role card, ghost role card.

**Acceptance checks (continued):**
6. At rest the scene is a tree: one stem from the task's bottom centre splits into a left branch to the subtasks card and a right branch to the role split chip. No ring of cards around a hub.
7. The side node hangs off the task's right edge on a short connector with no text. The only text on a connector is the "Role split" chip, which interrupts the roles connector.
8. The task, nested, role and ghost role cards each carry a tool-icon row at their bottom edge: a tag, then 5, 4, 3 and 3 tiles. The subtasks card and the side node carry none.
9. Owners rows: task card 2 avatars, subtasks card 3, nested card 1. The role card has one portrait-style avatar at its left.
10. Each element sits within 0.02 of the fractions above. The scene's bounding box is centred within 0.02.
11. The client node and the ghost role card read as faded. No other element is faded at rest.
12. Three or four faint concentric rings sit behind the scene; the largest reaches about 0.9 of the card width.
