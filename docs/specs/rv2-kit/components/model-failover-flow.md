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
</content>
