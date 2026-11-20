# Decision Review Queue

Source item: affectUI Pro `components/review-center` (functional reference only; see README).

## Purpose
Lets a person work through a backlog of proposals agents want to carry out: code changes, plans, contract edits, refunds. One item appears at a time, along with its reasoning, the proposed content and any flagged spans. The person approves, edits, rejects or skips it, and can undo the most recent decision. Items are ordered by risk and age.

## Data
| Field | Type | Meaning |
|---|---|---|
| `review_items` | `ReviewItem[]` | Each: `item_id`, `item_title`, `agent_name`, `model_name?`, `item_mark?`, `item_kind` (short category label), `risk_level` (`"high" \| "medium" \| "low"`), `confidence_score` (0..1), `created_at` (epoch value), `agent_reasoning?`, `text_fields?`, `body_text?`, `body_heading?`, `body_format?` (`"text" \| "code"`), `cited_sources?`, `file_changes?`, `plan_steps?`. |
| `text_fields` / `body_text` | string, or array of strings and `{ text, flag_note, source? }` | A flagged segment carries a note explaining the concern. Editable. |
| `file_changes` | `{ path, lines_added?, lines_removed?, diff_text?, first_line? }[]` | For a code change: one row per file, with a unified diff showing line numbers. |
| `plan_steps` | `{ step_title, tool_name?, step_mark?, step_detail? }[]` | For a plan: numbered steps; a step may carry a flag. |
| `sort_order` | `"risk" \| "oldest" \| "given"` | Queue order. Defaults to risk (high first, then oldest). |
| `queue_filter` / `default_queue_filter` | `"all" \| "high" \| "unsure"` | Queue filter. "Unsure" means confidence below `unsure_below`. |
| `unsure_below` | number | Threshold for Unsure. Defaults to 0.7. |
| `reviewed_before` | number | Count already reviewed earlier in the day; the header continues counting from there. |
| `wait_limit` | duration | How long an item may wait before its wait time shows a warning and then an overdue state. Defaults to one hour. |
| `current_time` | epoch value, optional | Current time. Defaults to the real clock, refreshed periodically. |
| `reject_reasons` | string[] | Quick reasons offered when rejecting. Up to four are shown. |
| `on_decide` | (item, result) => void | Result is `{ outcome, reason?, edited_text?, edited_fields? }`; the edit fields are present only when edited. |
| `on_undo` | (item, result) => void | The most recent decision was reversed. |
| `on_skip` | (item) => void | Moved on without deciding. |
| `board_title` | string | Heading. Defaults to "Review". |

Decided items remove themselves from the queue. New ids slide into the queue at their sorted position.

## Anatomy
In reading order:
1. Header: title, progress "N of M reviewed", the oldest waiting time, and the filter control.
2. Queue column: rows in the chosen order. Each row shows title, kind and waiting time. The current row is marked.
3. Item pane: the item's title, its agent and model, its risk and confidence tags, the reasoning, the body or fields (with flags), file diffs or plan steps, and sources.
4. Decision bar: Approve, Edit, Reject and Skip, each showing its single-key shortcut, plus Undo when available.
5. Reason picker: appears after Reject; a short list of reasons, each with its number key.
6. Progress indicator for the queue.

## States
- **Reviewing:** one item shown; decision bar active.
- **Editing:** the body or fields are editable in place; Save and Cancel are available.
- **Reason picking:** the reason list is open after Reject; Escape cancels the rejection.
- **Decided:** a brief outcome mark appears where the item was; the item fades out; the next item rises in.
- **Skipped:** the row is marked skipped; the item stays in the queue at its position.
- **Flag open:** a flagged span's note is open.
- **Queue empty:** a completion message; decision bar disabled.
- **Filter applied:** the queue shows only matching items; if the current item is filtered out, the next visible one is selected.
- **Overdue:** the waiting time shows the warning or overdue treatment according to `wait_limit`.

Transitions: deciding moves to the next item; Undo restores the last decided item to its place.

## Interactions
- **Pointer:** click a queue row to select it; click decision buttons; click a flagged span to read its note.
- **Keyboard, A:** approve the current item.
- **Keyboard, E:** edit in place; Ctrl or Cmd with Enter approves with the edits; Escape cancels the edit.
- **Keyboard, R:** reject, then number keys 1 to 4 choose a reason; Escape cancels.
- **Keyboard, S:** skip to the next item without deciding.
- **Keyboard, J and K or the arrow keys:** next and previous item.
- **Keyboard, F:** moves to the next flagged span and shows its note.
- **Keyboard, Z or Ctrl or Cmd with Z:** undoes the last decision.
- **Keyboard, Escape:** closes the reason picker, editor or note.
- Shortcuts work while focus is anywhere inside the component; buttons expose their shortcut to assistive technology.
- **Touch:** the same actions are available as buttons; no key-only action is required.

## Motion (functional)
- A decided row shows its outcome mark where its time was, then fades and closes up.
- The decided item fades up while the next one rises from below.
- A pressed decision button briefly flashes its decision's emphasis.
- The progress indicator advances smoothly.
- New items arrive at their sorted spot with a brief highlight.
- Stepping between items slides briefly; a note appears by scaling up while fading in.
- Reduced motion: all changes happen at once.

## Accessibility
- Every action is a real button with a name and a shortcut hint.
- The reason list is a named group.
- Each queue row is a button whose label lists its title, kind, risk, confidence and wait; the active row also gets the current marker.
- Flagged spans are focusable and linked to their note.
- Decisions, skips and undos are read out politely, together with the updated count.
- When the row or field a person is on disappears, focus moves to the next item.

## Responsive
- Built on a container query: when wide, the queue sits beside the item; when narrow, the queue collapses above it.
- The workspace has a fixed height and scrolls internally so the decision bar is always reachable.

## Composition hints
- Split layout: list plus detail pane.
- List and list item.
- Badge for risk and confidence.
- Diff viewer for code changes.
- Plain in-place text editor for edits.
- Popover for flagged notes.
- Button group for decisions.
- Polite live region.

## Acceptance checks
1. Items are shown in risk order, high risk first, then oldest first within the same risk.
2. Pressing A calls `on_decide` with outcome approve for the current item and shows the next item.
3. Pressing R then 2 calls `on_decide` with outcome reject and the second reason.
4. Pressing Z after a decision calls `on_undo` and restores the item to its prior position.
5. Pressing S moves to the next item without calling `on_decide`, and marks the skipped row.
6. An item with confidence below `unsure_below` appears under the Unsure filter.
7. Editing a body and pressing Ctrl or Cmd with Enter calls `on_decide` with the edited text.
8. A waiting time past `wait_limit` shows the overdue treatment.
9. When no items remain, the decision bar is disabled and a completion message is shown.
</content>
