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

## Amendment: structure and demo

Layout archetype: a stacked control panel with five bands, top to bottom: header, strategy row, flow, route table, fallback row, footer. Each band spans the full panel width.

Parts that must exist:
- Header band: title and environment label at the leading edge; at the trailing edge, the blended cost per thousand tokens with the live policy's figure beside it for comparison.
- Strategy row: a segmented control with one segment per strategy, the active one marked.
- Flow band: a gateway node at the leading edge; one line per route, running to the route label at the trailing edge, each label in `route_list` order. Line thickness follows the route's share. Lines carry the pulses.
- Route table: one row per route, with a colour dot and name at the leading edge, then share, p95 latency and cost per thousand tokens. Column headers are present.
- Fallback row: a toggle labelled with its word, followed by a "from" picker and a "to" picker on one line.
- Footer band: policy version and environment at the leading edge; at the trailing edge, the state text, then Revert, then Deploy.

Demo data for the gallery (our own names and values):
- Four routes: three in use, one with a zero share that stays in the table and in the flow as a zero-width line.
- Three strategies: "Even", "Lowest cost", "Lowest latency".
- Active strategy in the draft: "Lowest cost". Live policy: "Even".
- Shares in the draft sum to 100 percent.
- Fallback toggle on, from one route to another.
- Policy version and environment label as data.

States visible at rest in the gallery: the draft differs from the live policy, so the footer reads "Unsaved changes" with Revert and Deploy enabled, and the header shows both the draft and the live blended cost. Lines are drawn at their share widths.

Acceptance checks:
11. At desktop width the header holds title and environment at the leading edge and both blended-cost figures at the trailing edge.
12. The gallery demo shows the strategy control as one row of three segments with one marked as active.
13. The flow shows one line per route in route-list order, each ending at its route label, with the zero-share route drawn as a zero-width line.
14. The route table shows four rows with share, p95 and cost columns; the shares in the gallery demo add to 100 percent.
15. The fallback row sits between the table and the footer, holding the toggle and both pickers on one line.
16. The footer shows the policy version, the state text, Revert and Deploy, in that order from left to right at the trailing edge.
17. At rest the gallery footer reads Unsaved changes with Revert and Deploy both enabled, and the header shows the live blended cost next to the draft figure.
18. A route whose share is zero is still listed in the table (SSR test).

## Amendment 2: structure and demo

Replaces amendment 1 in three places. The flow band is a fan-out drawing with no handles, sliders or focusable elements; shares change only through the strategy row. The blended-cost figures leave the header and sit in a summary row that also holds the strategy control. The route table gains a share bar beside each share figure. Amendment 1 checks 14 to 18 stand, with check 13 and 11 restated below as 19 and 21.

**Layout archetype:** a control panel of six bands, top to bottom: header, summary row, flow band, route table, fallback row, footer. Each band spans the panel width.

**Parts that must exist:**
- Header band: title at the leading edge, with the environment label beneath it.
- Summary row: at the leading edge, the draft blended cost per thousand tokens as the main figure, with the live policy's figure beside it under a short caption. At the trailing edge of the same row, the strategy segmented control, one segment per strategy, active one marked.
- Flow band: a gateway marker at the leading edge; one smooth line per route, running from the gateway to a route label at the trailing edge, in `route_list` order. Each line's thickness follows its share. Each label carries a marker matching its line. Pulses travel along the lines as in amendment 1.
- Route table: one row per route: marker and name at the leading edge, then share with a share bar, p95 latency, and cost per thousand tokens. Column headers present.
- Fallback row: a toggle labelled with its word, then the "from" picker and the "to" picker, on one line.
- Footer band: policy version and environment at the leading edge; at the trailing edge, the state text, then Revert, then Deploy.

**Demo data (ours):**
- Four routes in `route_list` order: "Fast model", "Deep model", "Batch model", "Edge model".
- Strategies "Even", "Lowest cost", "Lowest latency". Draft active: "Lowest cost". Live policy: "Even".
- Draft shares: Fast 20, Deep 0, Batch 60, Edge 20 (sum 100). Live "Even" shares: 25 each.
- Fallback toggle on, from "Fast model" to "Deep model".
- Policy version and environment label as data.

**Visible at rest:** the draft differs from the live policy, so the footer reads "Unsaved changes" with Revert and Deploy enabled. The summary row shows the draft and live blended cost. The flow shows three visible lines; the zero-share route keeps its label at the trailing edge and has no visible line.

**Acceptance checks (continued):**
19. At desktop width the summary row holds the draft figure and the live figure at the leading side and the strategy control at the trailing side, on one row. Replaces amendment 1 check 11.
20. The strategy control shows three segments in one row with "Lowest cost" marked active. Its options are reached by arrow keys and occupy one tab stop (checks 8 and 12).
21. The flow band shows one labelled line end per route in `route_list` order, with the gateway at the leading edge. The zero-share route has a label and no visible line. Replaces amendment 1 check 13.
22. The flow band contains no input, slider or button element (SSR test). The flow carries no per-route control.
23. The route table shows four rows, each with share bar, share, p95 and cost; the gallery's shares add to 100 percent (check 14).
24. The fallback row sits between the table and the footer, with the toggle and both pickers on one line (check 15).
25. The footer shows, from leading to trailing, the policy version and environment, then the state text, Revert and Deploy (check 16).
26. At rest the footer reads Unsaved changes with Revert and Deploy both enabled (check 17).
27. A route whose share is zero is still listed in the table (check 18).
