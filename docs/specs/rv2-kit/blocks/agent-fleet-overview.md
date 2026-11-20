# Agent Fleet Overview

Source item: affectUI Pro `blocks/mission-control` (functional reference only; see README).

## Purpose
Gives a person one screen on every agent they run: what changed since they last looked, which agents run under which, which ones need a decision from them, and one filter that narrows all three views together.

## Data
- `agents`: list of { id, name, model_label, parent_id?, status: "running" | "waiting" | "failed" | "done", cost_usd, elapsed_secs, current_task, child_count }.
- `inbox_items`: list of { id, agent_id, kind: "decision" | "failure" | "review", title, detail, waiting_secs, actions: { id, label }[] }.
- `activity`: list of { at, agent_id, outcome: "done" | "needs_you" | "failed" }, covering the period since `last_seen_at`.
- `last_seen_at` (timestamp).
- `status_filter`: "all" | "running" | "waiting" | "failed"; `on_status_filter_change(value)`.
- `spotlight_agent_id` (controlled, nullable), `on_spotlight_change(id | null)`.
- `on_inbox_answer(item_id, action_id)`, `on_clear_failed(agent)` for failed agents only.
- `heading_text` (default "Agent fleet").

## Anatomy
In reading order:
1. Heading and the status filter, a segmented control with counts: all, running, waiting, failed.
2. Summary strip (top, full width): a short activity curve across the period since `last_seen_at`, with counts of done, needs-you and failed events.
3. Fleet (centre left): agent cards arranged by parent. Each card shows name, model line, status badge, current task, elapsed time and cost. Child agents sit under their parent; a parent with children shows a child count.
4. Inbox (right): cards for each item needing attention, with title, detail, how long it has waited and its action buttons.

## States
- Default: filter on all; no focus.
- Filtered: the chosen status narrows the fleet, the activity curve and the inbox together; counts in the filter update; a filter with no matches keeps the layout and shows an empty message in the fleet and inbox.
- Agent focused: its card and its inbox item are emphasised; other cards and items fade; its activity points are highlighted on the curve.
- Failed agent: status badge shows failed; a dismiss action appears on its card.
- Inbox empty: a short line that nothing needs attention.
- Answering an inbox item: its action buttons show progress; the item leaves the inbox when the answer is accepted.

## Interactions
- Click an agent card: focuses it; click again to clear.
- Hover an activity point: shows its time, agent name and outcome; click the point to focus that agent.
- Click a filter segment: applies it.
- Inbox action button: calls `on_inbox_answer`.
- Clear on a failed agent: removes it from the fleet after confirmation inside the card, via `on_clear_failed`.
- Keyboard: Tab order is filter, fleet cards, inbox items. Left and Right move between filter segments; Home and End jump to first and last. Enter or Space expands an inbox card to reveal its buttons. Escape clears focus.

## Motion (functional)
- An answer from any part (inbox or fleet) updates the agent everywhere at once: the activity curve, the fleet card and the inbox counts.
- Focusing an agent fades the other cards and inbox items; both sides change together.
- A filter keeps the row height constant, so items below do not jump; hidden items are removed, not reflowed.
- With reduced motion preferred, all changes apply in one step.

## Accessibility
- The status choices form a radio set named for the agent list, every option stating its tally.
- The summary sits in a named region; its figures and curve are spoken when they shift.
- Inbox cards are buttons with an expanded state; their action buttons are real buttons.
- Status is a text badge, not a visual cue alone.
- Parent and child relationships are exposed as a tree.

## Responsive
- Wide: fleet and inbox side by side.
- Narrower: inbox moves under the fleet and takes its full height.
- Phone width: single column; the filter wraps onto two lines if needed; action buttons are full width.

## Composition hints
- Segmented control (filter), chart primitive (activity curve), card (agent and inbox items), badge (status), tree list (parent and child), button (actions), empty-state block.

## Acceptance checks
1. Selecting the failed filter shows only failed agents in the fleet and only failed-related inbox items.
2. Focusing an agent highlights its card and inbox item and dims other agents.
3. Answering an inbox action updates that agent's status in the fleet and removes the item from the inbox.
4. Dismissing a failed agent asks for confirmation and removes it only after confirming.
5. Child agents appear under their parent, and the parent's child count matches.
6. Activity points outside the selected filter are not counted in the summary.
7. Left and Right on the filter move between options and change the selection.
8. At phone width the inbox sits below the fleet and nothing scrolls sideways.
