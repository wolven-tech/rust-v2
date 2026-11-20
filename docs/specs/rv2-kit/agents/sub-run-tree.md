# Sub-Run Tree

Source item: affectUI Pro `components/agent-tree` (functional reference only; see README).

## Purpose
Gives a person a hierarchy view of a job split across a coordinating unit and the units it spawned: each unit's progress, its own cost and how long it has been going. Makes a stalled branch, a unit waiting on a decision and an expensive subtree stand out at a glance.

## Data
| Field | Type | Meaning |
|---|---|---|
| `root_node` | Node | Full hierarchy. The host regenerates it on every progress event. |
| `heading` | string | Label for the summary strip. |
| `clock_ms` | number | Present moment, supplied by the host. Every elapsed counter derives from it, so the component never schedules its own ticks. |
| `folded_keys` | string[] | Branch keys currently folded. Controlled. |
| `default_folded_keys` | string[] | Starting fold set when the component manages its own state. |
| `format_spend` | (amount: number) => string | Renders every cost figure. Default: two decimal places with a currency symbol. |
| `on_folded_change` | (folded_keys: string[]) => void | Fires after a branch is folded or unfolded. |
| `on_node_pick` | (node: Node) => void | Fires when a card is activated. Cards become selectable only when this is supplied. |
| `on_node_resolve` | (node: Node, choice: "confirm" \| "refuse" \| "retry" \| "dismiss") => void | Fires when the person answers from a node's window. The host applies the change. |

Node:

| Field | Type | Meaning |
|---|---|---|
| `node_key` | string | Identity that persists across updates. Layout position follows this key. |
| `caption` | string | Display name of the unit. |
| `brief` | string | One-line statement of the unit's objective. |
| `model_tag` | string? | Model name shown as a small tag. |
| `tool_marks` | ToolMark[] | Marks for tools the unit can use. |
| `phase` | "pending" \| "active" \| "awaiting_input" \| "complete" \| "errored" \| "halted" | Lifecycle position. |
| `spend` | number | Cost incurred by this unit alone. Subtree totals are computed by the component. |
| `began_at_ms` / `ended_at_ms` | number? | Elapsed time is the end, or `clock_ms` while still open, minus the start. |
| `needs_text` | string? | For `awaiting_input`, the decision the unit is asking for. |
| `fault_text` | string? | For `errored`, the failure description. |
| `branches` | Node[] | Units started by this one. |

ToolMark: `{ mark_source, caption }`.

## Anatomy
1. Summary strip: the `heading`, then counts for units, active units, units awaiting input, errored units and total spend, in that order.
2. Canvas: the top-level node is centred at the head. Each parent drops links to its children.
3. Node body, read top to bottom: phase glyph, caption, optional model tag, tool marks, brief. On the right: spend and elapsed time.
4. Fold toggle: appears on any node with children, states how many children sit below it, and shows the subtree's total spend.
5. Folded subtree: a single row with the caption and rolled-up figures.
6. Badge: nodes in `awaiting_input` or `errored` show one. Activating it reveals `needs_text` or `fault_text` and the available answers.

## States
- `pending`: dimmed body; no elapsed figure.
- `active`: running glyph; elapsed figure increases with `clock_ms`; spend figure animates while it is still accruing.
- `awaiting_input`: waiting glyph; badge with the word for waiting; window offers confirm and refuse.
- `complete`: finished glyph; elapsed figure stops at `ended_at_ms`.
- `errored`: failure glyph; badge with the word for failure; window offers retry and dismiss. The failed node stays where it is, and a retry adds a new node under the same parent with a new key.
- `halted`: dimmed, with a stop glyph.
- Folded branch: descendants leave the layout; the parent displays rolled-up figures.
- Empty hierarchy: summary strip only, all counts at zero.

Every transition is caused by the host changing `root_node`.

## Interactions
- Tab: visits fold toggles first, then cards (only when `on_node_pick` is supplied). Hidden descendants are skipped.
- Enter on a card: calls `on_node_pick`.
- Enter or Space on a fold toggle: flips that branch.
- Enter or Space on a badge: opens its window and places focus on the first answer.
- Esc inside a window: closes it and returns focus to the badge.
- A pointer press outside an open window: closes it.

## Motion (functional)
- A newly added node draws its link from the parent first, then its body rises into place. Several additions arrive in tree order, one after another.
- Layout changes travel: nodes slide to their new positions and links redraw during the slide.
- Removed nodes fade out starting from the deepest level, and their links retract toward the parent.
- The only continuous motion is the elapsed clock and a rising spend figure. With reduced motion requested, all changes apply instantly and the spend shimmer is disabled.

## Accessibility
- Hierarchy exposed as nested groups; each unit is a tree item whose name includes caption and phase word.
- Fold toggles report `aria-expanded` and an action label that states the count, such as "Show 2 units under <caption>".
- A badge works as a disclosure control reporting its open state; the window it reveals takes focus on its leading answer.
- Summary counts are announced politely when they change.
- Phase is always expressed as a word inside the accessible name, never by glyph alone.

## Responsive
- When the container can hold the first tier side by side, first-level children sit in a row under the root and deeper tiers hang beneath their parent. All cards share one width.
- When the container is narrow, each tier indents under its parent and links run down a left rail. Cards narrow together so the deepest tier still fits.
- Link geometry is read from rendered card positions, so it stays correct at any width or zoom level.
- At phone width the page never scrolls sideways; the tree scrolls inside its own container if needed.
- Theme-aware: follows the host's light or dark setting.

## Composition hints
Card, status badge, glyph, toggle button, popover or dialog, nested list with tree roles, summary strip of stat cells, link layer, currency formatter.

## Acceptance checks
1. Folding a branch with two children removes both from layout and shows one summary line whose spend equals the sum of the descendants' spend.
2. Moving a node to `awaiting_input` shows its badge; activating the badge opens a dialog with focus on the first answer; Esc closes it and returns focus to the badge.
3. Advancing `clock_ms` raises the elapsed figure of every `active` node and leaves `complete` nodes unchanged.
4. Retrying an errored node by supplying a new key adds a node under the same parent and keeps the errored node visible.
5. Tab order lists fold toggles ahead of cards and never reaches descendants of a folded branch.
6. Changing a node's phase updates the summary counts and triggers a polite announcement.
7. Enter on a card passes that node to `on_node_pick`; Enter on a fold toggle does not.
8. At phone width there is no horizontal page overflow, and every card remains readable.
