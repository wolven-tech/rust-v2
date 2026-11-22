# Model Failover Flow

Source item: affectUI Pro `components/fallback-chain` (functional reference only; see README).

## Purpose
Shows one request finding a model that can answer it. The request is tried against each model in its fallback order; the person sees which attempt is running, which failed and why, and which one answered. Used by model routers, gateways, cross-provider retries and incident views.

## Data
- `attempt_list` (Attempt[]): in the order tried. Each `{ attempt_id, model_name, provider_name, attempt_state, duration?, attempt_cost?, error_text? }`. `attempt_state`: `"queued" | "in_progress" | "failed" | "answered" | "skipped"`.
- `heading_text` (text, default "Request"): what was asked; shown above the diagram with the running time and total cost.
- `routing_label` (text, optional): label on the request node.
- `origin_node` (End, optional): where the request came from. An End is `{ node_name, node_logo?, node_text?, node_details? }`, shown in a hover card.
- `result_node` (End, optional): where the answer goes. Same shape; its preview appears once an attempt has answered.
- `clock` (number, optional): clock used for the live timer. Defaults to real time.
- `on_retry()`: shown as a Retry control once every attempt has failed.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Summary: heading, "Attempt n of N", elapsed time, total cost.
2. Diagram: the request node on the left; one branch per attempt in order; the result node on the right.
3. Each branch node: model name, provider, state text and time.
4. Hover card per node: source, text preview, extra detail lines. Shown on hover and on focus.
5. Retry control: appears only after every attempt has failed.

## States
Per attempt:
- Queued ("Up next"): not yet tried.
- In progress: the running pulse travels to this node; the timer runs.
- Failed: the branch uses the error tone; the state text names the failure (for example a rate-limit code).
- Answered: the branch continues to the result; the state shows time and cost.
- Skipped ("Not needed"): a later attempt after an answer. Shown as skipped, not failed.

Whole flow:
- Running: at least one attempt is in progress or queued.
- Answered: one attempt answered; the result node shows its preview; remaining attempts are skipped.
- Exhausted: all attempts failed; Retry is shown.

## Interactions
Pointer:
- Hover a node: shows its hover card.
- Click Retry (exhausted state only): calls `on_retry`.

Keyboard:
- Tab: request node, each attempt in order, result node, then Retry when shown. Each node opens its hover card on focus.

## Motion (functional)
- The running pulse moves from the request to the attempt in progress.
- A failed branch changes tone to show the failure.
- The branch that answers extends to the result.
- The live timer counts up while an attempt runs.
- Reduced motion: pulse and drawing stop; states change at once.

## Accessibility
- The summary is a status region; the current attempt, the answer time and failures are announced.
- Each state is written as text, not tone alone.
- Attempts form an ordered list with provider, state and time per item. Diagram lines are decorative.
- Hover cards are tooltips linked to their node, so they are read on focus too.

## Responsive
- When space is limited: the provider moves from the node into the hover card.
- When the container is too narrow for side-by-side branches: the diagram becomes a tree. The request sits on top, attempts branch from one vertical line, and the result stays on that line above them.

## Composition hints
Panel, summary header, flow diagram (nodes and connectors), status badge, hover card (tooltip), button, live region, timer.

## Acceptance checks
1. With four attempts and the first failing with a 429 status, the first branch shows the error tone and that status text.
2. While attempt 2 runs, the summary reads "Attempt 2 of 4" and the timer advances.
3. When attempt 2 answers, attempts 3 and 4 show Skipped, not Failed, and the result node shows the answer preview.
4. When all four fail, Retry is visible, and pressing it calls `on_retry` once.
5. Focusing an attempt node shows its hover card and links it to the node for screen readers.
6. When space is limited, the provider text no longer appears in the node and appears in its hover card.
7. When the container is too narrow for side-by-side branches, the layout is a single vertical tree with no horizontal scrolling.
8. Each state is announced in text: a failed node has text beyond its tone.
9. Reduced motion: no moving pulse; the state still changes.
10. An attempt with no cost shows no cost value rather than zero.

## Amendment: structure and demo

Layout archetype: a left-to-right fan-out. A request node sits at the leading edge, a vertical stack of attempt nodes sits in the centre in tried order, and a result node sits at the trailing edge, level with the stack. A summary header runs above all three.

Parts that must exist:
- Summary header: heading at the leading edge; at the trailing edge, the running position as "n of N", the elapsed time and the total cost, on one line.
- Request node: a title, a routing label line and a short source line. It is the origin of the connectors.
- Attempt nodes: one card per attempt, stacked in tried order. Each card has a provider initial badge at the leading edge, the model name as its title, the provider name on a second line, and a state row with state text, time and cost.
- Connectors: one from the request to every attempt. One from the answering attempt to the result, drawn only after an answer. Connectors are decorative.
- Result node: at the trailing edge. Shows a waiting placeholder until an attempt answers; then shows its preview.
- Hover card on every node, opened by pointer or focus.
- Retry control: below the stack, present only in the exhausted state.

Demo data for the gallery (our own names and values):
- Four attempts in order: attempt 1 failed with a rate-limit status 429; attempt 2 in progress with a live timer; attempts 3 and 4 queued.
- Request node with a routing label; result node waiting.
- Each attempt has a cost, except attempt 4, which shows no cost value (check 10).
- Summary reads position 2 of 4, with an elapsed time and a total cost.

States visible at rest in the gallery: one failed attempt with its status text, one running attempt, two queued attempts, result node waiting. Not shown at rest: answered, skipped and exhausted states, hover cards and the Retry control; those are reached by interaction or by the state tests.

Acceptance checks:
11. At desktop width the gallery demo shows the three columns in order: request at the leading edge, attempts stacked in the centre, result at the trailing edge.
12. The gallery demo shows the four attempts at rest with the states named in the demo above, with no interaction.
13. At rest the result node shows a waiting placeholder and no preview text.
14. Each attempt card carries provider, state text, time and cost on one card; the gallery demo shows all four cards.
15. Connectors go from the request to every attempt; the connector to the result appears only when an attempt has answered (SSR test on both states).
16. The summary header shows position, elapsed time and total cost in that order at the trailing edge.
17. The gallery demo shows no hover card and no Retry control at rest.
</content>
