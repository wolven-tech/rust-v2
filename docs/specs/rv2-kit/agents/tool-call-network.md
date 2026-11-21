# Tool Call Network

Source item: affectUI Pro `components/agent-graph` (functional reference only; see README).

## Purpose
Presents agents, the tools they invoke, and the person they report to as a single live graph. Every invocation travels down its link and its reply travels back, so a viewer can tell which agent is occupied, which tool is being used, which link is busy, and which node is blocked on the person or has broken.

## Data
- `entities`: `{ id, name, node_role: "agent" | "tool" | "user", compact_label?, glyph?, summary_line?, condition?: "idle" | "working" | "waiting" | "error" | "done", activity?, spend?, is_centre? }[]`. Send the complete list on every update; ids keep a node where it is, and unseen ids add a node.
  - `activity`: what the node is doing at this moment, shown on its card.
  - `summary_line`: a model or tool description placed under the name.
  - `is_centre`: pins that agent to the middle; without it the first agent is used.
- `exchanges`: `{ id, from_id, to_id, sent_at, result?: "pending" | "ok" | "error", caption?, latency? }[]`, the recent ones, newest last. A new `id` launches a pulse from `from_id` toward `to_id`. Setting `result` to `"ok"` returns the reply in the success style; `"error"` returns it in the failure style and flashes the link. An exchange keeps its `id` when its result changes. Exchanges create whatever links they need.
- `links`: `[from_id, to_id][]`, links that exist before any exchange happens.
- `clock_now?`: the reference time used for per-minute counts and link activity. Defaults to the current time, refreshed each second.
- `on_focus_node(entity | null)`: fired when a node's card opens or closes.
- `box_height?`: an explicit height. Default is derived from the width, taller when the width is small.
- `format_spend(amount) => string`.
- `heading`: string, default "Agent graph".
- `extra_class?`

Keep the graph to a few agents and roughly a dozen outer nodes.

## Anatomy
From top to bottom:
1. Header bar: the heading, counts (agents, tools, exchanges per minute), and total spend. A "needs you" chip or a "failed" chip appears when any node is waiting or in error.
2. Canvas: nodes arranged on two concentric rings. The inner ring holds agents, with the centre agent in the middle. The outer ring holds tools and the user, each sitting on the side that faces the agents it talks to. Each node displays its glyph or initials, its name, and its summary line when space allows.
3. Links: a thin, plain line for each linked pair, drawn with one right-angle bend. Links that leave the same side of a node run parallel and are ordered so they never cross.
4. Node card: opens when a node is activated. It shows name, role, a condition badge, `activity`, a few of the node's most recent exchanges (each with its target and duration), and spend. It closes through its close control or Escape.

## States
- Node `idle`: at rest.
- Node `working`: a slow, breathing ring.
- Node `waiting`: attention styling with a sweeping highlight and a thin moving outline; a clock badge and the text "Needs you".
- Node `error`: critical styling with the same shape as waiting; an exclamation badge and a text line.
- Node `done`: at rest, marked as complete.
- Link: idle (plain) or carrying traffic (a moving pulse). A busy link gets thicker as its recent exchange count rises.
- Exchange: pending (outbound pulse), ok (return pulse), error (critical return pulse plus a link flash).
- Card: closed or open.
- Chips: visible while any node is waiting or in error. What activating a chip does is not specified by the reference.

## Interactions
- Activating a node opens its card. Activating a different node switches the card to that node. The close control or Escape closes the card and calls `on_focus_node(null)`.
- Keyboard: Tab moves across nodes and chips. The focused node highlights its neighbourhood (its links and the nodes they reach).
- Enter or Space on a node opens or closes its card.
- Escape closes the card and moves focus back to its node.
- Touch: a tap does what a click does.
- Hover over a link or a node: not specified by the reference; the clean room may omit it.

## Motion (functional)
- On first draw, the centre agent appears first, then the remaining agents, then the outer-ring tools. Each link extends toward the node it reaches.
- Each exchange sends a short streak along its link that fades behind it, with a soft glow. Request, reply, waiting-on-user, and error each carry a distinct glow. The receiving node ripples once.
- Working agents breathe slowly; waiting agents breathe a little faster in the attention style.
- When nodes arrive or leave, every node settles into its new place with a springy motion. A new tool slides out of the agent it connects to. A departing node shrinks while its link retracts.
- Animation runs only while the component is visible on screen and the tab is in focus.
- Reduced motion: nodes do not glide and pulses do not travel; instead each exchange lights up its link.

## Accessibility
- Each node is a button whose label combines its name, role, condition, activity, and the number of exchanges in the last minute. `aria-expanded` mirrors whether its card is open.
- The card is a dialog. Its close control receives focus on open, and Escape dismisses it.
- A visually hidden list restates the graph: each node's condition, the agents or tools it contacted and how frequently over the last minute, and its spend.
- Chip updates for "needs you" and "failed" are pushed through a polite announcement area.
- Condition is never shown by colour or motion alone: waiting and error nodes also carry a badge and a text line under the name.

## Responsive
- Layout is computed from the measured width. Ring radii, node sizes, and label room scale together.
- When the container is narrow: the graph is taller than it is wide, nodes are smaller, labels are compact, the summary line is dropped, and the detail card anchors to the lower edge of the graph.
- Labels and the outer ring stay inside the panel, and the graph never requires horizontal scrolling at phone width.
- Light and dark themes are supported through an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Container card (header and panel), canvas with absolutely positioned nodes over an SVG link layer, badge (condition), chip (waiting and error counts), dialog (node card), polite live region, visually hidden table (text equivalent), glyph or initials avatar, a single animation loop that writes positions straight to the DOM.

## Acceptance checks
1. Each node sits at its ring position and shows its name, role, and condition; the entity marked `is_centre` sits in the middle.
2. A new exchange id produces one pulse from `from_id` to `to_id`; the same id later set to `result: "ok"` produces a return pulse in the success style.
3. An exchange with `result: "error"` returns a critical pulse and flashes its link.
4. A node in `waiting` shows the "Needs you" text and a badge; a node in `error` shows the failure text and a badge.
5. Activating a node opens a card listing its recent exchanges and spend; Escape closes it and returns focus to the node.
6. Tab visits nodes and chips in order; the focused node highlights its neighbourhood.
7. The text list includes every node with its condition and its exchange count for the last minute.
8. A change to waiting or error is announced through the polite live region.
9. Removing a node moves the remaining nodes into place and shrinks the removed one away.
10. With reduced motion on, no node moves and each exchange lights its link without a travelling pulse.
