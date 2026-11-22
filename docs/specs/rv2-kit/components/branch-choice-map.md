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

## Amendment: structure and demo

**Archetype:** a decision tree read left to right. The question sits at the left. Each branch is a button with its consequence lines beside it. A picked branch can open a second question to its right. Narrow layouts hang the branches from a rail under the question.

**Parts:**
- Header: heading and pager.
- Question block: risk tag, agent name with model name, question, optional context line.
- Branch column: one numbered button per branch; a preference mark on at most one of them.
- Consequence column: lines beside each branch; a flagged line carries a marker and a note.
- Follow-up region: beside a picked branch. Before the pick, a preview of it sits beside its branch (check 12).
- Custom row: last in the list, labelled, opens a multi-line field with a send control.
- Outcome line: appears after a pick, once the answer is known.

**Gallery demo:** two forks; the pager reads "1 of 2". Fork one: agent "Migration agent" on "model A", high risk, asking whether to apply a schema change to a live table now or after the replica catches up. Three branches, "Apply now", "Wait for replica" and "Stop and report", with two consequence lines each. One line under "Apply now" is flagged. "Wait for replica" is the preferred branch and has a follow-up question with two branches. Fork two exists in the data and shows only in the pager.

**Visible at rest in the gallery:** pager, risk tag, agent and model names, all three branches, the preference mark, the flagged marker, the follow-up preview, the custom row. No outcome line and no open note or field.

**Acceptance checks (continued):**
11. At rest the gallery shows the pager reading "1 of 2", the high risk tag beside the agent and model names, three branch buttons each with a numbered key hint, the preference mark on "Wait for replica", and the custom row after the branches.
12. A branch with a follow-up shows that follow-up beside it as a dimmed, read-only preview. The preview's controls are not in the tab order and cannot be picked until the branch is picked.
13. Each branch's consequence lines sit in the same row band as its button, so a reader matches them without a connecting line.
14. A second gallery instance shows the same fork after the first branch was picked. Unpicked branches are dimmed, the follow-up is live and in the tab order, and no outcome line appears until the answer arrives.
15. A flagged consequence shows its marker at rest with its note hidden. Focusing it or pressing F reveals the note, and the consequence refers to the note through its accessible description.

## Amendment 2: structure and demo

This amendment makes the links between parts visible. The first amendment described a left-to-right tree; the gallery drew a flat list, so the question-to-branch and branch-to-follow-up links are now drawn as elbow connectors. It adds a second follow-up preview to the gallery and a waiting count to the header. Checks 16 to 20 are added.

**Archetype: a left-to-right tree with drawn elbows.** The question card sits at the left, centred on the branch column. Elbow connectors run from it to each branch node. Each node has its consequence lines to its right, in the same row band. A branch with a follow-up has its follow-up group to the right of its consequence lines, joined to the node by an elbow connector. The custom row is the last node in the column and is joined to the question like the others.

**Parts that must exist**
- Header: heading, count of forks waiting for you, and a pager (dots and "n of N").
- Question block: risk tag, agent name with model, question, optional context line.
- Branch nodes: numbered, one per branch. The preferred node carries a preference tag. A node with a follow-up carries a follow-up tag.
- Consequence lines beside each node; a flagged line carries a marker and a note.
- Follow-up group: its question and option nodes, to the right of the consequences. Dimmed and read-only until its branch is picked.
- Custom row: last in the column.
- Outcome line, after a pick.

**Gallery demo counts**
- Fork one: three branches and the custom row. Apply now carries a follow-up preview ("Notify the owner first?", two options). Wait for replica is the preferred branch and carries a follow-up preview ("Run the backfill in the same window?", two options). Stop and report has consequence lines and no follow-up.
- Fork two exists in the data and shows only in the pager.

**Visible at rest**
- Pager "1 of 2"; header count two.
- Risk tag, agent and model names.
- Three branch nodes and the custom row, each joined to the question by an elbow connector.
- Two dimmed follow-up previews, their option nodes outside the tab order.
- Preferred tag on Wait for replica; follow-up tags on the two branches that carry one.
- No outcome line; no open note or field.

**Acceptance checks (continued)**
16. Elbow connectors run from the question card to each branch node and to the custom row. The connectors are hidden from assistive technology.
17. Each branch with a follow-up shows its follow-up group to the right of its consequence lines, joined to the node by its own elbow connector.
18. At rest both follow-up previews show, dimmed and read-only, and their option nodes are not in the tab order.
19. The preferred node carries the preference tag, each node with a follow-up carries the follow-up tag, and the header count equals the number of forks in the pending list.
20. In a second instance with the first branch picked, its follow-up group is live and in the tab order, the other previews stay dimmed, and no outcome line appears until the answer arrives.
</content>
