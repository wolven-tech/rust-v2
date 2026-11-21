# Traffic Split Router

Source item: affectUI Pro `components/model-router` (functional reference only; see README).

## Purpose
Gives a person a view of, and control over, how an AI gateway distributes requests among models. They pick a strategy, compare the blended cost against the policy currently live, nominate a fallback model, review each route's share, latency and price, and push the policy out. Intended for gateway operations and cost decisions.

## Data
- `route_list` (Route[]), top to bottom: `{ route_id, route_name, route_provider?, route_logo?, route_tone?, p95_seconds, cost_per_1k }`. `cost_per_1k` is dollars per thousand tokens; `p95_seconds` is the 95th-percentile latency.
- `strategy_list` (Strategy[]): `{ strategy_id, strategy_label, share_map: { [route_id]: number } }`. Shares are normalised to a total of 100 percent; a strategy without shares divides traffic evenly.
- `live_policy` (`{ active_strategy, fallback: { enabled, from_route_id, to_route_id } }`): the policy in force right now.
- `policy_revision` (number): shown as "Policy v<n>"; increases by one with each deployment.
- `environment_label` (text): the environment the gateway runs in, for example a production name.
- `on_deploy(policy)`: pushes the policy out. While the returned promise is pending, the button reads Deploying.
- `on_draft_change(policy)`: fired on every change to the strategy or fallback.
- `heading` (text).
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Header: heading and the environment label.
2. Summary: the blended cost per thousand tokens for the current selection, set against the live policy.
3. Strategy switch: a choice among strategies (for example Cost, Balanced, Quality).
4. Flow: one line per route fanning out from the gateway, each line's width matching its share; pulses of light travel from the gateway into each route.
5. Route table: each route's share, p95 latency and cost per thousand tokens, with row and column headers.
6. Fallback row: a switch and two model pickers (from, to).
7. Footer: live state text (Live, Unsaved changes, or Deploying), Revert, and Deploy.

## States
- Live: the selection matches the live policy; the footer reads Live; Deploy is disabled.
- Unsaved changes: any edit; the footer reads Unsaved changes; Deploy is enabled; Revert is available.
- Deploying: `on_deploy` is pending; the footer reads Deploying; controls are locked.
- One route only: the fallback row is hidden.
- No routes: the text "No models to route to" appears.
- Fallback names a route that is not in the list: the fallback resolves to the first available routes, never the same route twice.

## Interactions
Pointer:
- Click a strategy: changes the shares; lines widen or narrow; figures update.
- Toggle the fallback; choose the fallback's source and target in its pickers.
- Hover a flow line or table row: highlights that route everywhere, fading the rest.
- Revert: restores the live policy. Deploy: invokes `on_deploy`.

Keyboard:
- Strategy: radio group; arrow keys move between options.
- Fallback switch: Space toggles it.
- Model pickers: Enter, Space or an arrow opens a menu; arrows and Home/End move within it; Enter selects; Escape closes and returns focus.
- If Revert is removed from the footer, focus moves to Deploy.

## Motion (functional)
- Pulses of light run along each line from the gateway to its route, at a pace matching the share; each pulse fades in, peaks in the middle, and fades out.
- A strategy change slides each band and share bar to a new width, while cost and shares tick over to their fresh values.
- Hovering a line or row highlights that route everywhere.
- With reduced motion requested, pulses and rolling stop; numbers change at once.

## Accessibility
- The strategy control is a radio set that occupies a single tab stop.
- The fallback switch announces its state; each model picker is a menu.
- The routes form a real table with row and column headers.
- The footer state is announced when it changes. Deploy stays focusable but marked aria-disabled until something has changed.
- The flow is decorative and hidden from assistive technology; the table carries the figures.
- Focus rings appear for keyboard use only.

## Responsive
- Fills the container width; the lines stretch; route names stay at the right.
- On moderate widths the strategy switch moves beneath the cost summary; the table keeps share and price.
- On narrow widths the fallback pickers show only the route's logo; the name lives in the label and the menu.
- Long names and labels truncate, with the full text shown on hover.

## Composition hints
Panel, header, segmented control (radio), summary figures with rolling numbers, flow diagram (SVG), data table, switch, two select pickers, footer with status text and buttons, live region, animated pulses.

## Acceptance checks
1. With the live policy active, the footer reads Live and Deploy is disabled.
2. Choosing a different strategy changes the route shares, and the shares of all routes sum to 100 percent.
3. The blended cost equals the sum, across routes, of share times cost per thousand tokens for the selected strategy.
4. Any edit changes the footer to Unsaved changes and enables Deploy; Revert restores the live policy.
5. Pressing Deploy calls `on_deploy` with the edited policy; while pending the button reads Deploying and the controls are locked.
6. With one route, the fallback row is not rendered.
7. A fallback whose target equals its source is never shown; the next route is used instead.
8. Strategy options are reachable with the arrow keys and occupy one tab stop.
9. On moderate widths the strategy switch sits under the cost summary.
10. With reduced motion requested, pulses do not animate and numbers change without rolling.
