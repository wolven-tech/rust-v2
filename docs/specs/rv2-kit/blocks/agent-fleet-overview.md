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

## Amendment 5: structure and demo

Archetype: control room: a status filter and activity summary across the top, a fleet tree at the left, an inbox at the right.

Replaces the wide-width placement in Anatomy item 3: sub-agents sit to the right of their lead. Narrow widths keep them under the lead.

Visible at rest:
- Heading "Mission control" and a segmented status filter: All 7, Running 3, Needs you 2, Failed 1.
- Summary card "Since you looked": a time range label ("Since 9:10 AM · 2h"), tallies (4 done, 2 need you, 1 failed), an activity curve across that period with seven event markers, each marked by outcome, and two time labels at its ends.
- Fleet card: heading "Agents" and a summary line "3 leads · 4 sub-agents · 3 running · 2 waiting on you · $3.22". Lead cards in a column; each lead's sub-agents stacked to its right and joined by an elbow line from the lead's right edge. Each card: name, model line, status badge, current task, footer with cost and elapsed time. Leads show a sub-agent count.
- Inbox: heading "Inbox 3" and three item cards, each with a kind word, title, detail line, waiting time and action buttons.

Gallery demo:
- Leads: Planner (large model, running, "Splitting the launch into three checks", $1.42 · 21 min, 3 sub-agents); Support lead (mid model, running, "Answering open tickets in the queue", $1.02 · 29 min, 1 sub-agent); Data clerk (small model, needs you, "Rotating the webhook secret", $0.05 · 11 min).
- Sub-agents under Planner: Researcher (done, "Comparing four rival listings"), Drafter (running, "Writing three plan descriptions"), Publisher (needs you, "Waiting for approval to send"). Under Support lead: Reply writer (failed, "Reply draft blocked by policy check").
- Inbox: Decision "Send the launch email to 240 accounts?" (Publisher, waiting 14 min, buttons Reject and Approve); Decision "Rotate the webhook secret now?" (Data clerk, waiting 11 min, Reject and Approve); Failure "Reply draft blocked by policy check" (Reply writer, waiting 6 min, Retry and Dismiss).

Motion, settled frame: all cards shown; nothing focused.
Moves: an answer in the inbox updates the agent's card, the curve and the counts at once. Focusing an agent dims the others. A filter keeps the row height constant and removes hidden items without reflow.
Reduced motion: all changes apply in one step.

Acceptance checks (continued):
9. At wide widths sub-agents sit to the right of their lead, and each lead's sub-agent count matches the sub-agents shown.
10. The activity curve shows seven markers, each marked by outcome, and rises and falls across the period.
11. The filter counts match the agent cards: All 7, Running 3, Needs you 2, Failed 1.
