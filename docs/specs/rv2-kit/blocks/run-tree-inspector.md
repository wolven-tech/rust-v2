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
