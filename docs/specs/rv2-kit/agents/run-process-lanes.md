# Run Process Lanes

Source item: affectUI Pro `components/handoff-swimlanes` (functional reference only; see README).

## Purpose
Draws one agent run as a process diagram with a horizontal lane for each participant: people, agents and tools. Each step sits as a box in its performer's lane, each hand-over between steps is a connector, and each approval is a diamond. The diagram shows, at a glance, which work is done, which is running, which waits on a person, and which failed, and where work changes hands. A waiting approval can be acted on from its card without leaving the diagram.

## Data
- `lanes`: `{ id, name, kind?, role?, mark?, color?, description? }[]`, top to bottom. `kind`: `"person" | "agent" | "tool"`, default `"agent"`. A person lane shows the person mark in the ink colour; an agent lane shows its own identity mark in its colour; a tool lane shows its logo, or a neutral mark when none is given.
- `steps`: `{ id, lane_id, title, status?, shape?, after?, duration_label?, detail? }[]`.
  - `status`: `"queued" | "running" | "waiting" | "done" | "failed" | "skipped"`. Default `"queued"`.
  - `shape`: `"step" | "approval" | "start" | "finish"`. Default `"step"`. An approval is a diamond; start and finish are circles.
  - `after`: ids of the steps that hand over to this one. Omitted, it means the step immediately before it in the array. An empty array starts a new branch.
  - `duration_label?`: free text shown in the card, for example a measured duration. Not used for layout.
  - `detail?`: longer text, shown in the card only.
- `run_title`: string, shown in the header.
- `run_status`: derived from the steps unless supplied: `"running" | "waiting" | "done" | "failed"`. Shown as a badge in the header.
- `on_select(step | null)`: fired when a step's card opens or closes.
- `on_approve(step)` and `on_reject(step)`: when supplied, the card of a `waiting` approval shows Approve and Reject.
- `heading?`: string, default "Hand-overs". Also the section's accessible name.
- `extra_class?`

Normalisation, applied before rendering:
- A step whose `lane_id` names no lane is dropped, and connectors that start or end at a dropped step are dropped with it.
- A connector from an id that names no step is dropped.
- Repeated step ids keep the first occurrence.
- A cycle is cut where it closes, so the diagram stays acyclic.

Demo values in any reference are invented for this spec; no affectUI data is reused.

## Anatomy
From top to bottom, inside one framed panel:
1. Header: `run_title`, a position counter (done steps out of total), and the run status badge.
2. Lane rail: one row per lane. Each lane label holds the lane's mark, its name and its role. A lane with no steps reads as empty.
3. Grid: boxes placed in their lane, one column per step in dependency order. Connectors run from a predecessor to a successor. A connector's head carries an arrow.
4. Approval diamonds and start and finish circles sit in the lane of their owner.
5. Card, opened from a lane label, a box, a diamond or a circle, placed beside what opened it:
   - Title, lane name and role, and status word.
   - `detail`, and `duration_label` if given.
   - Where the work came from (predecessor lanes) and where it goes (successor lanes).
   - For a `waiting` approval with handlers supplied: Approve and Reject actions.

## States
- Step status: `queued` (quiet, default), `running` (shimmer in the owner's colour), `waiting` (attention tone), `done` (quiet with a check mark), `failed` (critical tone), `skipped` (muted, struck through in its label).
- Run status: `running`, `waiting` when any approval is `waiting`, `done` when every step is `done` or `skipped`, `failed` when any step is `failed` and none is running.
- Connector light: a connector whose successor is `running` or `waiting` carries a light from its tail to its head. The light takes the successor's colour, or the waiting tone when the successor is an approval.
- Card: closed, or open on one step. Only one card is open at a time.
- Empty: no lanes, or no steps after normalisation. The panel shows "Nothing to hand over yet" in place of the grid.

## Interactions
- Click or activation of a lane label, box, diamond or circle: opens its card and calls `on_select`.
- Approve in the card: calls `on_approve(step)` and closes the card. Reject does the same with `on_reject`.
- Click outside the card closes it.
- Escape closes the card from anywhere and returns focus to what opened it. Approve and Reject also return focus this way.
- Opening a card from the keyboard moves focus into it; a waiting approval receives focus first.
- Keyboard: Tab visits the lane labels first, then the boxes, one column at a time, reading downward.
- Enter or Space on any element opens its card.

## Motion (functional)
- On first view boxes settle into place and connectors draw column by column.
- A connector light runs along each live hand-over, full at its head and fading behind it.
- A running box shimmers and its owner's mark shows a light wave. A waiting approval breathes softly.
- A card fades in from its anchor.
- Reduced motion: all of the above is removed. Status changes apply immediately.

## Accessibility
- Each lane label and each step shape is exposed as a button carrying an expanded state that reflects its card.
- A box's accessible name combines its title, its lane name and its status, for example "<title>, <lane name>, <status word>".
- Lines, dots and lights are decorative and hidden from assistive technology. Each card states where its work came from and where it goes.
- Run status changes are announced politely once per change.
- The focus indicator shows only for keyboard focus.
- Status is never conveyed by tone alone; the status word is always present on the box or in its card.

## Responsive
- The grid fills the container's width. Spare width spreads across the columns.
- Under width pressure, gaps tighten before boxes shrink. If the grid still cannot fit, it is scaled as one unit. Horizontal scrolling is not offered.
- Below a narrow container width, lane labels show only their marks; the name and role appear in the lane's card.
- Long titles and lane names are cut short with an ellipsis, and the full text is in the card.
- Cards open below their anchor, or above it when the anchor is near the bottom of the viewport. A card never extends beyond the component's width.
- Right-to-left and other scripts render in their own direction.
- Dark theme is supported through an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Button (lane label, box, diamond, circle), badge (run status, step status), SVG layer for connectors and lights, popover or anchored card, text buttons (Approve, Reject), polite live region for run status, region for the header.

## Acceptance checks
1. A step with no `after` is connected from the step before it in the array; a step with `after: []` has no incoming connector.
2. Steps naming an unknown lane are not rendered, and connectors touching them are not drawn.
3. Changing a step's `status` updates its box and the run status without moving other boxes.
4. Clicking a box, diamond, circle or lane label opens exactly one card and calls `on_select` with that step.
5. A `waiting` approval's card shows Approve and Reject when handlers are supplied; a step that is not a waiting approval does not.
6. Approve calls `on_approve` once and returns focus to the element that opened the card.
7. Escape closes the open card from anywhere and returns focus to its opener.
8. Tab order visits lane labels first, then boxes column by column.
9. Each run status change produces one polite announcement.
10. With reduced motion requested, no connector draws, light runs, shimmer, or breathing animation plays.

## Amendment: structure and demo

**Archetype: swimlane process diagram.** One horizontal row per lane, each a person, an agent or a tool. Steps sit as boxes in their lane, placed in columns by dependency order. Approvals are diamonds; start and finish are circles. Arrowed connectors join each step to its successors. The header holds the title, a done counter and a run status badge.

**Parts that must exist**
- Header: run title, counter (done boxes and diamonds of all boxes and diamonds), run status badge.
- Lane rail: one label per lane holding its mark, name and role.
- Grid: boxes, diamonds, circles and arrowed connectors, with a live marker on each connector whose successor is running or waiting.
- Card for the opened step: title, lane, status word, detail, duration, the lanes it came from and goes to, and Approve and Reject for a waiting approval.

**Gallery demo counts**
- 5 lanes, top to bottom: Approver (person), Planner (agent), Coder (agent), Test runner (tool), Release bot (agent).
- 8 nodes: start circle and Brief received (Approver); Plan the change (Planner); Write the helper (Coder); Run tests (Test runner); Approve deploy (diamond, Approver); Ship build (Release bot); Legacy cleanup (Coder); finish circle (Release bot).
- 7 connectors: start to Plan, Plan to Write, Write to Run tests, Run tests to Approve, Approve to Ship, Ship to finish, and Write to Legacy cleanup.
- Counter denominator 6 (5 boxes and the diamond). Skipped steps count toward it and never as done.

**Visible at rest**
- Run status badge reading Waiting.
- Approve deploy waiting, with its status word; Run tests running, with its status word.
- Legacy cleanup skipped, its label struck through and its box carrying the word Skipped.
- Plan the change and Write the helper done.
- Exactly one connector with a live marker: the one into Approve deploy.

**Acceptance checks (continued)**
11. Five lane rows render in the given order; the start circle and the approval diamond sit in the Approver lane.
12. The gallery header reads "2 of 6 done" and the run status badge reads Waiting.
13. Legacy cleanup's label is struck through and its box still carries the word Skipped.
14. Exactly one connector carries a live marker at rest, the one into Approve deploy.
15. Each lane label shows its role under its name; below the narrow threshold only the marks remain and the names move into the lane's card.
