# Cause Analysis Tree

Source item: affectUI Pro `components/root-cause-tree` (functional reference only; see README).

## Purpose
Lays out how an agent looked into a failure as a tree of hypotheses. The failure sits at the top; beneath it sit candidate causes, each one open, under check, ruled out, or confirmed, together with the evidence behind each verdict. Once a root cause is confirmed, the chain from it up to the failure is highlighted.

## Data
| Field | Type | Meaning |
|---|---|---|
| `failure_node` | `Cause` | The top node. Its children are the first hypotheses. Optional `failure_detail` line beside its badge. |
| `Cause.cause_id` | string | Stable key. |
| `Cause.cause_title` | string | What the hypothesis claims. |
| `Cause.cause_mark` | ReactNode, optional | Small mark, for example the tool or service involved. Muted when ruled out. |
| `Cause.file_paths` | string[], optional | Files involved. The first is shown as a chip; the rest as a count. |
| `Cause.join_rule` | `"any" \| "all"` | Applies to its children: `any` means one is enough; `all` means they only cause the failure together. Defaults to `any`. |
| `Cause.check_state` | `"open" \| "checking" \| "ruled-out" \| "confirmed"` | Defaults to open. |
| `Cause.verdict_text` | string, optional | One sentence explaining why it was ruled out or confirmed. |
| `Cause.evidence` | `{ evidence_kind, evidence_source?, evidence_content }`, optional | `evidence_kind` is the label of the evidence (for example Log, Diff, Tool call); the content is shown as code. |
| `Cause.checked_at` | elapsed value, optional | When the check happened, measured from the start of the run. |
| `Cause.sub_causes` | `Cause[]` | Nested hypotheses. |
| `on_cause_action` | (cause, action: `"fix" \| "retry"`) => void | Offered on a root cause. Without it, evidence is read-only. |
| `format_time` | (elapsed value) => string | Formats `checked_at`. |
| `board_title` | string | Heading. Defaults to "Root cause". |

A confirmed cause that has children counts as confirmed only once its join rule is satisfied: one confirmed child for `any`, all children for `all`.

## Anatomy
In reading order:
1. Heading and a summary line: the investigation's status, the number of hypotheses, and how many are ruled out.
2. The tree: the failure at the top, children beneath; connectors show the join rule at each split (an "any" or "all" label on the split).
3. Each hypothesis card: title, state word, file chip, and its check time when present.
4. Evidence card below the tree: the selected cause's verdict, its evidence label, source and content (as code), and its file list. Fix and Retry actions appear for a root cause.

## States
- **Open:** not yet checked; neutral card.
- **Checking:** the card carries a moving highlight; the state word reads "checking".
- **Ruled out:** the card fades back with a dashed outline and a "ruled out" word; it stays in the tree.
- **Confirmed (intermediate):** the card reads "confirmed" and its children continue the investigation.
- **Root cause:** a confirmed cause with no children; its card reads "root cause" and the path up to the failure is highlighted.
- **Join not yet met:** a confirmed parent whose join rule is unmet is shown as pending.
- **Evidence selected:** the evidence card shows that cause's proof.

Transitions: a hypothesis moves from open to checking, then to ruled out or confirmed. Ruled-out nodes are never removed. A new hypothesis appears as open beneath its parent.

## Interactions
- **Pointer, card:** selects that cause and shows its evidence; clicking it again returns to the default evidence view.
- **Pointer, Fix or Retry:** calls `on_cause_action` with the cause and the action.
- **Keyboard, up and down arrows:** move to the parent (up) or to the first child (down). When the layout is narrow, they move to the previous and next row.
- **Keyboard, left and right arrows:** move along a row between branches. When the layout is narrow, they move to the parent and into the first child.
- **Keyboard, Home and End:** the failure at the top, and the last hypothesis.
- **Keyboard, Enter or Space:** shows the focused cause's evidence; pressing again returns to the default.
- **Keyboard, Tab:** moves from the tree to the evidence, then to Fix and Retry.
- **Keyboard, arrows inside the evidence code:** scroll it.
- **Keyboard, Escape:** returns from the evidence to the investigation summary.
- **Touch:** tapping a card selects it.

## Motion (functional)
- The hypothesis being checked shows a moving highlight.
- A ruled-out hypothesis fades back to a flat outline but stays visible.
- The evidence card grows and shrinks with its content, easing to its new height.
- When a root cause is confirmed, the path from it up to the failure draws itself in, one segment after another.
- Reduced motion: everything appears without animation.

## Accessibility
- The hypotheses form one tree with a single tab stop; each card announces its title, its state and, at a split, whether one child or every child is required.
- The evidence region is a polite live region; it announces when the selected evidence changes.
- State is never conveyed by emphasis alone: each state has a word, and ruled-out cards have a dashed outline.

## Responsive
- Cards keep a constant height, so a state change never shifts neighbours.
- The failure card sizes to its title. Hypothesis cards use a common width, set by the longest title but capped by available room. Overlong titles are clipped with an ellipsis; the evidence shows the full wording.
- When the tree cannot fit its narrowest cards, it becomes an indented list, two rows per card, with join rules shown as chips.

## Composition hints
- Card for the container and for each node.
- Tree layout primitive (or indented list fallback).
- Badge for state and join rule.
- Code block for evidence.
- Button group for Fix and Retry.
- Polite live region for evidence changes.

## Acceptance checks
1. A confirmed root cause with no children draws the highlighted path from itself up to the failure.
2. A parent confirmed with an `all` join rule and only one confirmed child shows the parent as pending, not confirmed.
3. A ruled-out hypothesis remains in the tree with a dashed outline and the word "ruled out".
4. Pressing Enter on a cause shows its evidence; pressing Enter again returns to the default view.
5. Down arrow moves focus from a parent to its first child; up arrow returns to the parent.
6. Fix calls `on_cause_action` with the root cause and `"fix"`; Retry calls it with `"retry"`.
7. Without `on_cause_action`, the Fix and Retry actions are not shown.
8. Adding a new open hypothesis does not change the height of existing cards.
9. When the container is narrow, the tree renders as an indented list and join rules remain visible as text.
</content>
