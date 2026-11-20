# Branch Choice Map

Source item: affectUI Pro `components/decision-tree` (functional reference only; see README).

## Purpose
Where an agent reaches a fork it should not resolve alone, this shows every option as a branch with its consequences set out beside it. Risky options are marked, the agent's own preference is highlighted, and a person can pick a path, answer a follow-up question on a branch, or write an instruction of their own.

## Data
- `pending_forks` (Fork[]): everything awaiting a choice. Decided forks leave; new ones join at the end. Each Fork:
  - `fork_id`, `question_text`, `context_text` (optional), `agent_name`, `model_name` (optional), `agent_icon` (optional node for the tool the agent uses)
  - `risk_level` (`"high" | "medium" | "low"`, optional): shown as a tag beside the agent name.
  - `branches` (2 to 4 Branch entries). A Branch has `branch_label`, `branch_icon` (optional), `consequences` (one-line effects; each is text or `{ text, flag_explanation, reference_url? }`, and a flagged one carries `flag_explanation`), `touched_apps` (optional node list), `agent_preferred` (boolean; at most one per level), and `follow_up` (optional `{ question_text, branches }`, one level deep, same shape as branches).
- `on_select(fork, selection)`: the selection is `{ index, branch, follow_up_selection? }` for a branch, or `{ custom_text }` for a written instruction. It may return an optional short answer string, or a promise of one, which is shown back to the person.
- `allow_custom` (boolean, default true): shows the custom-instruction branch.
- `control_handle`: imperative handle offering hover, choose, set_custom, send, next, previous, peek.
- `heading_text` (text, default "Decisions").
- `extra_class` (text, optional).

## Anatomy
Reading order, wide layout:
1. Header: heading and a pager ("1 of 3").
2. Current fork: risk tag, agent name with model, question text, and an optional context line.
3. Branch column: one button per branch, with its label, an agent-preferred marker where set, and a numbered key hint.
4. Consequence column: beside each branch, its consequences as lines. Flagged ones carry a marker; hovering or focusing one reveals its note.
5. Follow-up: when a branch has one, the follow-up question and its branches open to the right of the picked branch.
6. Custom branch: last in the list when allowed. Opens a multi-line text field with a send control.
7. Outcome line: after a pick, the chosen path ends in an outcome or the agent's answer.

## States
- Waiting: branches shown; none chosen.
- Hovered branch: that path is emphasised and the others recede.
- Picked: unchosen branches fade; the selected route extends to its result.
- Follow-up waiting: the picked branch has a second question, answered the same way.
- Custom open: text field open; it grows up to three lines as text is entered.
- Sending: text goes to `on_select`; the field shows a waiting state until the answer returns.
- Answered: outcome or answer line shown; then the fork leaves and the next one appears.
- Empty: no forks; nothing rendered. (Spec choice.)

## Interactions
Pointer:
- Click a branch to pick it. If it has a follow-up, the follow-up appears and waits for a second pick.
- Hover a branch to emphasise its path.
- Click a flagged consequence to show its note.
- Click the custom branch to open the text field. Send or Enter sends.
- Pager: move between forks.

Keyboard:
- Digits 1 to 4: pick that branch. The digit after the last branch opens the custom branch.
- Enter: in the text field, sends. Shift+Enter inserts a line break.
- ArrowRight / ArrowLeft (also J / K): next and previous fork.
- F: move to the next flagged consequence and show its note.
- Escape: close the custom field or a note.

## Motion (functional)
- A new fork grows in: the question first, then the branches, then their consequences in sequence.
- Hovering a branch emphasises its path.
- After a pick, unchosen branches dim, the selected route extends to its result, and only then does the next fork arrive.
- The text field opens short and grows as the person types, up to its maximum; its first line does not move.
- Reduced motion: all of the above appears at once.

## Accessibility
- Every branch is a button named by its label; when it is the agent's preference, the name says so.
- Shortcut keys are exposed as keyboard shortcuts.
- Flagged consequences are focusable; their note is linked by description.
- The text field is labelled. Its focus is shown by a stronger outline, since it has no ring.
- Picks, sent instructions and the agent's answers are announced politely.
- Decorative connector lines are hidden from assistive technology.

## Responsive
- Container-based. Wide: question, branches and consequences run left to right.
- When the container is narrow: branches hang from a vertical rail beneath the question, each with its consequences beneath it. Nothing scrolls sideways.
- Touch: targets sized for fingers.

## Composition hints
Panel, question heading, tag (risk), button list (branches), list of consequence lines with info markers, popover or inline note, textarea with send, pager, status line, live region, path drawing.

## Acceptance checks
1. A fork with three branches shows three branch buttons; pressing 2 picks the second and calls `on_select` with `index: 1`.
2. A branch with `agent_preferred` shows the preference marker, and its accessible name says so.
3. Picking a branch with a follow-up shows the follow-up; picking its branch calls `on_select` with `follow_up_selection` set.
4. Choosing the custom branch, typing, and pressing Enter calls `on_select` with `{ custom_text }`; the returned answer is shown.
5. Shift+Enter inserts a newline and does not send.
6. Pressing F moves to a flagged consequence and shows its note; Escape closes the note.
7. Picking a fork removes it from the list; a fork appended to `pending_forks` appears at the end of the pager.
8. With `risk_level: "high"`, a high-risk tag is shown beside the agent name.
9. When the container is narrow, no horizontal scrolling occurs and branches stack vertically.
10. With `allow_custom: false`, no custom branch is shown and digit keys stop at the last branch.
</content>
