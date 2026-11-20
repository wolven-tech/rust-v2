# Ranking Bracket

Source item: affectUI Pro `components/judge-bracket` (functional reference only; see README).

## Purpose
Shows a judge model picking the strongest of several answers to one task. Candidates are paired off inside a knockout tree, and each pairing sends the judge's favourite forward. Opening any pairing reveals both answers next to each other, with their scores and the judge's rationale. Fits LLM-as-judge selection, best-of-n code fixes and similar pick-one workflows.

## Data
- `entrants` (Candidate[]): every attempt, in seeding order. Count is 2, 4, 8 or 16; adjacent seeds meet first. Each entry is `{ entrant_id, entrant_name, entrant_icon?, entrant_meta?, entrant_text? }`:
  - `entrant_name`: what distinguishes this attempt.
  - `entrant_icon`: optional mark, such as the model maker's logo.
  - `entrant_meta`: a quiet line such as word count and duration.
  - `entrant_text`: what the attempt produced; may be a multi-line code block.
- `verdicts` (map keyed `"round:pairing"`, zero-based): `{ winner_id, rationale?, scores? }`. Scores are supplied for the upper and lower entrant. The key `"0:2"` names the third pairing of round one.
- `pairing_in_flight` (pairing key, optional): the pairing being adjudicated right now. Both names animate.
- `judge_label` (text): who is judging, typically a model name.
- `brief` (text): what the attempts are meant to achieve; shown muted beside the heading.
- `on_adopt(entrant)`: fires from the "use this one" action. Without it the final has no button.
- `heading` (text, default "Judge").
- `extra_class` (text, optional).

Rules:
- A verdict is recorded only once both entrants of that pairing are known, and its winner has to be one of them.
- Keep `entrant_name` to what sets the attempt apart, and `rationale` to one sentence.

## Anatomy
Reading order:
1. Header: heading, muted brief, judge label, and a status line that reads "Judging" with the pairing, or the result.
2. Knockout tree: rounds laid out left to right and named (quarterfinal, semifinal, final). Each pairing is a two-slot block showing both names and meta lines. The winning slot keeps full emphasis; the other is dimmed and joined by a dashed connector.
3. Winning route: a highlighted connector through every pairing the winner has taken, with a trophy on the final's winner.
4. Detail area under the tree: opened by selecting a pairing. Shows both answers in two columns (stacked on narrow layouts), each with its name, meta, text, scores and the rationale.
5. Adopt button: beside the final's winner, when `on_adopt` is supplied.

## States
Per pairing:
- Waiting: entrants not yet decided, or no verdict yet.
- In flight: the key matches `pairing_in_flight`; both slots animate.
- Settled: a verdict exists; winner emphasised, loser dimmed.

Whole tree:
- In flight: the status line names the pairing under judgement.
- Complete: the final has a verdict; the winning route is drawn and adopt is available.
- Pairing open: the detail area shows that pairing.

## Interactions
Pointer:
- Click a settled pairing: opens its answers below the tree. Click again: closes them.
- Click the adopt action on the final: calls `on_adopt(winner)`.

Keyboard:
- Tab: enters the tree and moves pairing by pairing, then to the answers and the adopt action.
- Left and Right: step to the pairing this one flows into, or back to the earliest pairing flowing into the current one.
- Up and Down: move to the neighbouring pairing in the current round.
- Enter or Space: shows that pairing's answers; repeating the key hides them.
- Escape: returns focus to the pairing under judgement.

## Motion (functional)
- The pairing under judgement animates.
- Once a verdict is in, the favoured entrant glides onward to the next round, while the other entrant dims in place and stays on screen.
- When the final is settled, the winning route draws in round by round and a trophy appears.
- The detail area eases to its height when a different pairing opens.
- With reduced motion requested, everything appears at once.

## Accessibility
- Each pairing is a button whose name includes its round, both entrants, and either the winner or the fact that it is being judged.
- The status line is announced politely as it changes; the header states who currently leads.
- Victory is never signalled by emphasis alone: the winner carries a check or score mark, and the loser is dimmed with a dashed connector.

## Responsive
- Slots hold a constant height, so the layout stays still during judging.
- Rounds share the width. When they cannot sit side by side, the tree becomes a list per round with two entrants per row, and answers stack.

## Composition hints
Panel, header with status, tree columns (pairing blocks with slots), connectors, badge (round name), detail panel with two columns, score list, button, trophy icon, animated highlight, live region.

## Acceptance checks
1. With eight entrants, the first round shows four pairings, the second two, and the final one; adjacent seeds meet first.
2. A verdict for pairing `"0:2"` whose `winner_id` is the third entrant marks that entrant as winner and advances it.
3. Setting `pairing_in_flight` to a key animates that pairing's two slots, and the status reads Judging with that pairing.
4. Clicking a settled pairing shows both texts, the scores and the rationale; clicking again hides them.
5. A verdict whose winner is neither entrant is ignored and not displayed.
6. The final's winner shows a trophy and the adopt button; pressing it calls `on_adopt` with that entrant.
7. Without `on_adopt`, no adopt button renders after the final is settled.
8. Left or Right on a focused pairing moves focus along the tree to the pairing it flows into.
9. Below the narrow breakpoint the tree becomes a list per round with no horizontal scroll.
10. With reduced motion requested, the winning route appears without a drawing animation.
