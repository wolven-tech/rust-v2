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

## Amendment 6: structure and demo

No earlier amendment exists, so no line is replaced.

Archetype: a since-you-looked curve across the top, a lanes panel beside an inbox panel, one status filter over all three.

Visible at rest:
- Header row: heading at left. At right, a four-segment status filter: "All 7", "Running 3", "Waiting 2", "Failed 1". Each segment applies the spec's status_filter value.
- Summary panel, full width: label "Since you looked"; start time and elapsed time beneath it. At right, three counts with icons: done, waiting, failed. Below, a curve across the panel's width with one dot per event, faint horizontal gridlines, and start and end time labels at its ends.
- Lanes panel, about two thirds of the width: header "Agents" with "3 running" and "2 waiting" counts, and a window label "Last 30m". A time axis across the top with labels at five-minute steps and a "now" label at the right end. One lane per agent, each with name, model line, a status pill where the status is waiting or done, and its spans as labelled bars.
- Inbox panel, about one third of the width: header "Inbox" with count 3. Three cards, each with title, detail line, waiting or failure line, and actions: Approve and Reject for decisions, Retry for a failure.
- The inbox count equals waiting agents plus failed agents. "All" equals the lane count.

Gallery demo:
- Lanes, top to bottom:
  - Planner, large model, running: "Outline the launch", from 25 minutes ago to now.
  - Scout, mid model, running: "Compare vendor prices", from 18 minutes ago to now.
  - Drafter, mid model, done: "Draft launch copy", from 20 to 8 minutes ago.
  - Publisher, mid model, waiting: "Stage the release", from 9 minutes ago to now, waiting label.
  - Support, small model, running: "Answer 38 tickets", from 25 minutes ago to now.
  - Invoicer, small model, failed: "Send invoice batch", from 12 to 6 minutes ago, failure mark.
  - Monitor, small model, waiting: "Watch the error budget", from 5 minutes ago to now, waiting label.
- Curve events: four done (Planner, Scout, Drafter, Support), two waiting (Publisher, Monitor), one failed (Invoicer). Seven dots in total.
- Inbox:
  - Publisher, decision: "Ship the release notes?", detail "Copy passed review. Publish to the blog?", waiting 14 min.
  - Invoicer, failure: "Invoice batch stopped", detail "Ledger rejected row 212: tax code missing", action Retry.
  - Monitor, decision: "Raise the alert threshold?", waiting 11 min.

Motion, settled frame: running spans reach the now marker; curve complete; waiting spans show their waiting label; the failed span shows its mark; no lane focused, all at full strength.
Moves and reduced motion: as stated in Motion (functional).

Defects, stated as checks: four lanes instead of seven (check 8); no since-you-looked curve (check 9); status counts that do not match the lanes (check 10).

Acceptance checks (continued):
8. At rest seven lanes are drawn, one per agent, and the All count equals the lane count.
9. The summary draws a curve with one dot per event, and its done, waiting and failed counts match those events.
10. Waiting equals the number of waiting agents. Failed equals the number of failed agents. The inbox count equals waiting plus failed.
