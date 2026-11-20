# Agent Timeline Overview

Source item: affectUI Pro `blocks/mission-timeline` (functional reference only; see README).

## Purpose
Shows every agent on one shared clock: what changed since the person last looked, how each agent's work is spread across time, and which items need a decision. The same status filter applies to all three views.

## Data
- `agents`: list of { id, name, model_label, status: "running" | "waiting" | "failed" | "done" }.
- `spans`: list of { id, agent_id, label, start_at, end_at?, status }. A span with no end is still running.
- `inbox_items`: list of { id, agent_id, kind: "decision" | "failure" | "review", title, detail, waiting_secs, actions: { id, label }[] }.
- `window`: { from, to }; `now` timestamp; default window is the last thirty minutes.
- `last_seen_at` (timestamp).
- `status_filter`: "all" | "running" | "waiting" | "failed"; `on_status_filter_change(value)`.
- `spotlight_agent_id` (controlled, nullable), `on_spotlight_change(id | null)`.
- `on_inbox_answer(item_id, action_id)`.

## Anatomy
In reading order:
1. Heading and the status filter with counts.
2. Summary strip: activity since `last_seen_at` with counts of done, needs-you and failed.
3. Timeline (centre): a time axis across the top with ticks relative to now; one lane per agent; spans as bars on their lane, labelled with their task; a marker at now.
4. Inbox (right): cards for items needing a decision, with their action buttons.

## States
- Default: window of the last thirty minutes, filter on all.
- Filtered: lanes and inbox narrow together; lanes with no matching spans stay in place as empty lanes so positions do not shift.
- Agent focused: its lane is highlighted and its inbox item emphasised; other lanes fade.
- Running span: its end is the moving now marker; its bar grows.
- Waiting span: bar shows a waiting label; the agent's inbox item is shown.
- Failed span: bar shows a failure mark and its error line appears on hover.
- Empty window: lanes show their axis with no spans and a short note.

## Interactions
- Hover a span: shows its label, start, end or elapsed time, and status.
- Click a span: focuses its agent.
- Click a lane label: focuses that agent.
- Scroll or drag the time axis: moves the window; the axis snaps to whole minutes.
- Click a filter segment: applies it.
- Inbox action: calls `on_inbox_answer`.
- Keyboard: Tab order is filter, lanes (each lane's spans in time order), inbox items. Left and Right move between spans on a lane; Up and Down move between lanes; Enter focuses the agent; Escape clears focus.

## Motion (functional)
- Running spans extend toward the now marker as time passes; the marker moves steadily.
- Changing the window slides the spans rather than redrawing them.
- An answer from any part updates the span status and the inbox counts at once.
- Focusing an agent fades the other lanes and inbox items together.
- With reduced motion preferred, the marker updates in steps and window changes apply instantly.

## Accessibility
- The timeline is a grid: lanes are rows, time is the column axis; the focused span is announced with agent, label, start and status.
- Spans are buttons with their label and status as the accessible name.
- The status control is a radio set whose name reads "Show agents".
- The summary is a labelled region; counts are announced on change.
- Status uses text badges in addition to bar markers.

## Responsive
- Wide: timeline with lanes and inbox side by side.
- Narrower: inbox moves under the timeline; the time axis shows fewer ticks.
- Phone width: lanes become a vertical list with a compact time bar per lane; the window can be changed by a control instead of dragging.

## Composition hints
- Segmented control, grid or timeline primitive (lanes and spans), card (inbox items), badge (status), button (actions), tooltip (span detail).

## Acceptance checks
1. A running span extends toward the now marker and stops at it.
2. Filtering to failed hides non-failed spans and keeps lane positions unchanged.
3. Focusing an agent highlights its lane and inbox item and fades other lanes.
4. Hovering a span shows its label, start and status.
5. Answering an inbox action updates the span status and removes the item.
6. Left and Right move between spans on a lane in time order.
7. At phone width each lane is a list entry and nothing scrolls sideways.
