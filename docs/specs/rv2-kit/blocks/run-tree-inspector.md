# Run Tree Inspector

Source item: affectUI Pro `blocks/run-inspector` (functional reference only; see README).

## Purpose
Shows one agent run on one screen: which agent started which, what each agent did, and where a failure happened. A person can open a single call to see its error in place.

## Data
- `run`: { id, title, status, duration_secs, cost_usd, token_count, model_call_count, tool_call_count, error_count }.
- `agents`: list of { id, name, model_label, status, parent_id?, cost_usd, elapsed_secs, current_step? }.
- `calls`: list of { id, agent_id, kind: "model" | "tool", name, status: "ok" | "running" | "error", duration_secs, error_message? }.
- `selected_call_id` (controlled, nullable), `on_select_call(id | null)`.
- `extra_classes`.

## Anatomy
In reading order:
1. Run header: title, status, and a figure row (duration, cost, tokens, model calls, tool calls, errors).
2. Tree area: each agent is a card; children hang beneath their parent with connecting lines. A root card sits at the top.
3. Agent card: name, model line, status badge, elapsed time, cost, and a row of small bars, one per call, with its length showing the call's duration.
4. Call detail: when a call is chosen, its row expands in place on its agent card, showing its name, duration and, for an error, the error message directly under its row.

## States
- Running: status badge shows running; the tree's latest call bar grows.
- Completed: status badge shows done; all bars are at final length.
- Error: the errored call's bar shows an error mark; the run's error count is non-zero; the card shows an error badge.
- Call selected: its row is expanded; other calls on the same card remain; the card stays in view.
- Nothing selected: header figures read run totals.
- Empty run: tree shows the root card with no calls.

## Interactions
- Click an agent card: expands or collapses its call list.
- Click a call bar or row: selects it; click again to clear.
- Hover a call bar: shows its name, duration and status.
- Hover an agent card: highlights its parent and children lines.
- Escape: clears the selected call.
- Keyboard: Tab order is the header figures (none focusable), then agent cards in tree order, then each call inside an expanded card. Enter or Space expands a card or selects a call. Arrow Down and Up move between cards in tree order; Right expands a card; Left collapses it.

## Motion (functional)
- Connecting lines draw as the tree grows; new child agents appear beneath their parent.
- A running call's bar grows over time.
- Selecting a call expands its row in place; the rows below move down, nothing is covered.
- With reduced motion preferred, nothing grows or draws; final lengths show.

## Accessibility
- The tree is a tree view: agents are tree items with level and expanded state; calls are child items.
- Each call's accessible name includes its name, status and duration.
- Error messages are in the call's row text, not only in a tooltip.
- Status is a text badge, not a visual cue alone.

## Responsive
- Wide: tree spans the width with children side by side under their parent.
- Narrower: children stack under their parent with an indent per level.
- Phone width: a single indented list; the figure row wraps to two lines.

## Composition hints
- Tree view (agent hierarchy), card (agent), badge (status), bar sparkline (calls), disclosure (expand), stat row (run figures), expandable row (call detail).

## Acceptance checks
1. Clicking an agent card expands its calls; clicking again collapses them.
2. Selecting a failed call shows its error message directly under its row.
3. The error count in the run header equals the number of calls with error status.
4. Child agents appear beneath their parent with the parent's expanded state reflected in the tree.
5. Arrow keys move between agent cards in tree order; Right and Left expand and collapse.
6. A running call's bar length increases while the run is running and stops at its final length when it completes.
7. At phone width the tree is an indented list and nothing scrolls sideways.

## Amendment 5: structure and demo

Archetype: run header, a figure strip, and a node graph of agent cards.

Visible at rest:
- Run header: title, status badge, agent count with running count, total cost.
- Figure strip, one row: Duration, Cost, Tokens, Model calls, Tool calls, Errors.
- Node graph: a root card top centre; two child cards beneath it, one left and one right, joined by elbow lines from the root's lower edge. A peer card beside the root, joined to it by a dashed horizontal link.
- Each agent card: name, model line, status badge, then its call rows.
- Each call row: a small kind mark, the call name, a status mark, a duration bar with its duration figure.
- An errored call row carries an error mark at rest.

Gallery demo:
- Title "Ship the spring newsletter", status "Running".
- Figures: Duration 8m 40s, Cost $0.31, Tokens 96k, Model calls 4, Tool calls 6, Errors 1.
- Root "Planner" (large model, running), call row "Draft the send plan" ($0.12).
- Peer "Budget checker" (small model, running), call row "Read the spend limit".
- Child "Researcher" (mid model, running), calls "Search price pages", "Search partner terms", "Query price history" (error).
- Child "Writer" (mid model, waiting), no calls, drawn faded.

Motion, settled frame: connectors drawn; running bars at their current length.
Moves: connectors draw as the tree grows, and new children appear under their parent. A running call's bar grows. Bars stop at final length on completion.
Reduced motion: all bars at final length; nothing draws.

Acceptance checks (continued):
8. Agents render as cards joined by elbow connectors; no agent is shown as a bar row.
9. Each call row shows its name, a status mark and a duration bar; the errored row shows its error mark at rest.
10. The Writer card shows a waiting status and no running mark.

## Amendment 6: structure and demo

Supersedes Amendment 5 where they differ. Replaces:
- "Visible at rest", error bullet ("An errored call row carries an error mark at rest"): no call row carries an error mark at rest. Error marks appear only on calls whose status is error.
- "Gallery demo", Errors figure 1: Errors reads 0.
- "Gallery demo", Researcher calls: three calls, all running. "Query price history (error)" is removed.
- "Gallery demo", Writer: mid model, running, drawn faded, with two running calls. Replaces "waiting, no calls".
- Check 9: the error-row clause is replaced by "each call row shows name, status mark, duration bar and figure; none shows an error mark at rest".
- Check 10: replaced by check 13 below.

Archetype: run header, figure strip, and a node graph of agent cards.

Visible at rest, added to Amendment 5:
- Run header: sparkle mark and title at left; at right, agent count with running count, and total cost.
- Each agent card footer: cost and elapsed time at left; a disclosure count with a chevron at right giving the number of calls not shown.
- Figure strip: six figures in one bordered row.
- Peer card joined to the root by a dashed horizontal link.

Scale: root card top centre-left. Peer card to its right at the same height. Left child card below the root, about a quarter of the stage width. Right child, faded, further right and lower. Each agent card is about a quarter of the stage width. Call rows take the card's full inner width, with name, bar and figure on one line.

Gallery demo:
- Title "Ship the spring newsletter", status Running.
- Figures: Duration 8m 40s, Cost $0.31, Tokens 96k, Model calls 4, Tool calls 6, Errors 0.
- Root "Planner" (large model, running), call "Draft the send plan" ($0.12).
- Peer "Budget checker" (small model, running), call "Read the spend limit".
- Child "Researcher" (mid model, running), calls "Search price pages", "Search partner terms", "Query price history", all running.
- Child "Writer" (mid model, running, drawn faded), calls "Read the style guide", "Read the price frames", both running.

Motion, settled frame: connectors drawn; running bars at their current length.
Moves and reduced motion: as Amendment 5.

Defects, stated as checks: planner and tool-call rows squeezed into a narrow column with labels wrapped one word or one character per line (check 11).

Acceptance checks (continued):
11. Call names never break to one word or one character per line. At 1280 px and 390 px they stay on one line or end with an ellipsis.
12. Each agent card is about a quarter of the stage width at 1280 px or wider. Each call row shows its name, bar and figure.
13. The Writer card is faded and reads Running. Its two call rows show running durations and no error mark.
