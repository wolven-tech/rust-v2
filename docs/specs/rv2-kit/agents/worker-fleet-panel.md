# Worker Fleet Panel

Source item: affectUI Pro `components/agent-fleet` (functional reference only; see README).

## Purpose
Lays out several top-level agents in a column of cards. Each card condenses its sub-agents into one line plus a progress ring. Opening a card unfolds its sub-agent tree alongside, so a viewer can spot the agent that is waiting on them, the one that failed, and the fleet-wide cost, without every tree being unfolded at once.

## Data
- `heading`: string, default "Agents".
- `lead_agents`: `{ id, name, assignment, model_label?, picture?, tool_marks?: node[], state, spend?, started_at?, ended_at?, sub_agents?: SubAgent[] }[]`. Send the whole fleet on every change; stable `id`s keep each card in its slot.
- `SubAgent`: `{ id, name, assignment, model_label?, state, work_items?: { label, state?, tool_name?, duration? }[], spend?, question?, failure_text? }`.
- `state` (lead agent and sub-agent alike): `queued` | `running` | `waiting` | `done` | `failed` | `stopped`. `waiting` is shown as "Needs you". `queued` and `stopped` are rendered muted.
- `spend` on a lead agent is that agent's own spend. The panel adds up sub-agent spend for each branch and the grand total for the header. Callers must not pre-sum.
- `expanded_id` / `default_expanded_id`: the id of the open lead agent, or null to fold everything. Controlled or uncontrolled.
- `on_expand_change(id | null)`.
- `on_pick(sub_agent)`: turns on selection for sub-agent cards.
- `on_respond(sub_agent, action)`: `approve` | `reject` | `retry` | `dismiss`. Without this handler the response actions are not offered.
- `clock_now?`: the reference time for running durations. Defaults to the current time, ticking.
- `format_spend(amount) => string`. Defaults to a dollar format.
- `extra_class?`

## Anatomy
From top to bottom:
1. Header bar: `heading`, summary counts (agents, sub-agents, running, needs you), and total spend.
2. A vertical column of lead agent cards. Each card, top to bottom:
   - picture, name, model line
   - state badge (top right)
   - assignment line; tool marks drawn as small overlapping tiles ahead of it
   - footer: a summary phrase (the most urgent state among its sub-agents), a progress ring showing the share of sub-agents finished, the card's total spend, and elapsed time
3. The open tree, placed beside the column when there is room and under its card otherwise. It holds the lead agent's sub-agent cards, linked to it by connector lines. Each sub-agent card shows name, state, assignment, and a footer with spend and either elapsed time or the current step.
4. Work window (for a sub-agent that has work items): its items with their state and duration. For a `waiting` or `failed` sub-agent, a response area sits above the action buttons.

## States
- Lead agent: running, waiting (needs you), failed, done, queued, stopped.
- The summary phrase reports the most urgent state, in this order: needs you, failed, running, done.
- Open tree or folded.
- While one lead agent is open, the others are dimmed; hovering a dimmed card brings it back to full strength.
- Sub-agent: queued, running, waiting, done, failed, stopped. Dim the queued and stopped ones, connectors included.
- Work window: closed or open.
- Work items: an item without a state counts as done and shows its duration; running items read as in progress; waiting or failed items stand out; queued or skipped items are faint.

## Interactions
- Activating a lead agent card opens its tree, and the previously open tree folds back into its card. Activating the already-open card folds it.
- Activating a sub-agent card that has work items opens its work window. Activating it again, or pressing Escape, closes the window and returns focus to the card.
- Activating a sub-agent card without work items calls `on_pick`.
- Approve or Reject (on a waiting sub-agent), and Retry or Dismiss (on a failed one), call `on_respond`.
- Keyboard: Up and Down move between lead agents; Home and End jump to the first and last. Enter or Space opens or folds the focused lead agent. Tab enters the cards of the open tree (those with work items, a response to give, or selection). Inside a response window, Tab cycles through its buttons.
- Escape closes the work window and returns focus to its card.
- A click outside closes a work window.
- Touch: a tap does what a click does.

## Motion (functional)
- Opening a tree: the connectors of the previously open tree retract first, then the new connectors extend and its cards rise in one after another.
- The open tree centres on its lead agent as far as the column permits. If it grows while open, it moves smoothly to its new position.
- Lead agents that are not open fade to a lower opacity.
- Spend that is still climbing carries a faint shimmer.
- A newly added sub-agent rises into place with its connector drawn out.
- A work window drops down from its card, and its item rows fade in one after another.
- Reduced motion: trees switch instantly, with no staggered entrance.

## Accessibility
- Each lead agent is a button with `aria-expanded`, and `aria-controls` points at its open tree.
- The open branch is exposed as a tree whose label names the lead agent; each child is a tree item carrying its name and state.
- Summary totals (agents, running, needs you, spend) are announced politely when they change.
- State is never conveyed by tone alone: every card names its state in words, and the phrase under each lead agent explains what the ring represents.
- Sub-agents that have work items, or that need a response or have failed, expose `aria-haspopup` and `aria-expanded`. Their work window is a dialog labelled "Work of <name>", dismissed by Escape or an outside click.
- The small action buttons on a card are shortcuts. Keyboard users reach the same actions from the work window.

## Responsive
- When the container is roomy enough for the column plus the deepest branch, the branch opens to the right.
- In a tighter container the branch drops below its card, shifting lower cards downward, and folds away before another opens.
- Every card has a fixed height (one line for the assignment, one for the figures), so state changes never shift the layout.
- Light and dark themes are supported through an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Card (lead agent, sub-agent), badge (state), progress ring with a pie fill (sub-agent share), picture, tool tile group, tree with connectors, dialog or popover (work window), text buttons (Approve, Reject, Retry, Dismiss), polite live region (totals), list (column), shimmer text (spend).

## Acceptance checks
1. Each lead agent card shows name, model, assignment, state badge, sub-agent summary, spend, and elapsed time.
2. The summary phrase on a card reports the most urgent sub-agent state, checked in the order needs you, failed, running, done.
3. Activating a lead agent opens its tree; activating another lead agent folds the first and opens the second.
4. Activating a sub-agent with work items opens a work window that lists each item with its state and duration; Escape closes it and focus returns to the card.
5. Approve and Reject appear only on `waiting` sub-agents; Retry and Dismiss only on `failed` ones; each calls `on_respond` with the matching action.
6. Header totals equal the sums across agents and sub-agents, and spend is never counted twice.
7. Up, Down, Home, and End move between lead agents; Enter opens or folds the focused one.
8. A sub-agent changing state updates its card in place without moving sibling cards.
9. Each change to a total produces one polite live announcement.
10. Below the wide threshold, the tree opens under its card and the cards beneath it move down.
