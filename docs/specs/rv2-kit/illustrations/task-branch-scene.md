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
