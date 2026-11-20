# Agent Question Card

Source item: affectUI Pro `components/decision-card` (functional reference only; see README).

## Purpose
Presents an agent's open questions directly above the chat input, so a person can answer by pointer or keyboard without leaving the conversation. It handles three question types: a free choice among options, an approve-or-decline, and a pick among options that carry previews.

## Data
- `pending_questions` (Question[]), oldest first. The caller removes questions once answered. Each Question:
  - `question_id` (text)
  - `question_text` (text)
  - `task_name` (text, optional): the work item this question belongs to.
  - `question_kind` (`"choice" | "pick" | "approval"`, default `"choice"`)
  - `answer_choices` (3 to 5 Choice entries, for choice and pick). Choice: `choice_id`, `choice_title`, `choice_summary` (optional), `is_suggested` (boolean; at most one per question), `preview_node` (pick only), `extra_blocks` (node, optional).
  - `detail_rows` (2 or 3 `{ row_name, row_text }` pairs, for approval).
- `on_answer(question_id, answer)`: answer is `{ chosen_id }`, `{ free_text }`, `"approve"`, or `"decline"`.
- `on_fold(question_id)`: fired by the fold control or by Escape.
- `start_folded` (boolean, default false): begins collapsed to one line showing the waiting count.
- `reveal_token` (number, optional): changing the value brings the card up; it unfolds, pulses once, and focuses the selected row.
- `text_map` (partial map): every visible string, including the section title, fold, show, suggested tag, approve, decline, the free-text placeholder, the waiting count, and the "n of N" pager label.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Header: section label, a pager reading "1 of 3" with previous and next controls, and a Later control.
2. Question text. The task name sits above it in muted text when supplied.
3. Body, varying by kind:
   - Choice: numbered rows. Each row shows its number, title, a one-line summary, and a Suggested tag where applicable.
   - Pick: a row of preview tiles, one per choice. Tiles stack into rows when the container is narrow.
   - Approval: the question, the detail rows, and two actions, Approve and Decline.
4. Free-text field at the bottom. Enter sends the text.

## States
- Waiting: card visible, first question selected.
- Several waiting: the pager shows the current position; the others are reached through the pager.
- Selected row: one row (or tile) shows the focus ring.
- Answering: the chosen row is acknowledged and `on_answer` fires. The caller removes the question, and the card advances to the next.
- Folded: one line with the waiting count and a Show control.
- Empty: when `pending_questions` is empty, the card is not rendered.
- No disabled state (spec choice): the caller hides the card instead.

Transitions: answering or Later advances to the next question. After the last answer the card folds and then leaves the layout. Later folds the card to one line; Show restores it.

## Interactions
Pointer:
- Activate a choice row or tile to answer with it at once.
- Approve or Decline answers at once.
- Click the free-text field to type; Enter sends.
- Pager arrows move between questions. Later folds the card. Show unfolds it.

Keyboard:
- Number keys 1 to 5 (including the number row) answer with the choice at that position at once.
- Arrow Up / Down move between rows. Left / Right move between tiles in pick mode. Down from the last row moves to the free-text field; Up from an empty field returns to the rows.
- Home / End: first or last row.
- Enter or Space: answer with the focused row. In the field, Enter sends the typed text.
- Escape: Later; folds the card to one line.

Focus rules:
- After an answer, focus moves to the next question.
- The pager and Later keep focus inside the card; focus never falls back to the page body, even when the question holding focus is removed.
- After the last answer, the caller moves focus to the composer.

## Motion (functional)
- A new question fades in as the current one leaves.
- After the last answer, the card's height collapses to nothing, then the card is removed.
- When revealed through `reveal_token`, the card pulses once.
- Reduced motion: everything appears without transition.

## Accessibility
- The card's accessible name is its current question.
- Choices form a listbox with a single tab stop; arrow keys move within it.
- Each choice exposes its number key as a keyboard shortcut.
- Answers and Later are announced politely, including how many questions remain waiting.
- Focus rings appear only for keyboard focus. The free-text field uses a stronger edge instead.

## Responsive
- Takes the full width of the chat input column.
- Long titles and detail values wrap once, then truncate with an ellipsis. Summaries and task names stay on one line.
- Pick tiles: when there is room for one or two tiles, they keep their full size; when the container is narrow, tiles become rows.
- Touch: rows are at least a comfortable touch target tall; buttons are at least a comfortable touch target to hit.

## Composition hints
Panel, group label, pager (icon buttons), listbox, numbered badge, tag (suggested), preview tile, text input with send, detail list, collapse animation, live region.

## Acceptance checks
1. With three choice questions, the pager reads "1 of 3"; answering the first moves the pager to "1 of 2" and shows the next question.
2. Pressing key 2 answers with the second choice immediately, without Enter.
3. Escape folds the card to one line showing the waiting count; Show restores it with the same selected row.
4. A `question_kind: "approval"` question shows its detail rows and exactly Approve and Decline actions; Approve calls `on_answer(id, "approve")`.
5. A pick question renders one tile per choice; Enter on a tile calls `on_answer` with that choice.
6. Typing in the free-text field and pressing Enter calls `on_answer(id, { free_text })` with the typed string.
7. Setting `reveal_token` to a new number on a folded card unfolds it and focuses the selected row.
8. After the last question is answered, the card collapses and is removed; no focus is left on a removed node.
9. When the container is narrow, pick tiles become full-width rows.
10. Reduced motion: questions swap without fade or collapse animation.
</content>
