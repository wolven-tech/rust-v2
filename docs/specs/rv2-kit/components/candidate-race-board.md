# Candidate Race Board

Source item: affectUI Pro `components/parallel-attempts` (functional reference only; see README).

## Purpose
Shows several candidate results for the same task side by side while they run, lets a person compare each candidate's progress and score, and lets them choose which result to use. The best finished candidate is pre-selected unless the person changes it.

## Data
| Field | Type | Meaning |
|---|---|---|
| `candidate_list` | `Candidate[]` | One entry per candidate, in display order. |
| `candidate_key` | string | Stable key; keeps rows in place across updates. |
| `display_name` | string | Usually the model name. |
| `method_summary` | string | One phrase describing how this candidate approaches the task. |
| `model_id` | string, optional | Model identifier, if different from the display name. |
| `mark` | ReactNode, optional | Small mark shown before the name. |
| `run_state` | `"running" \| "finished" \| "halted"` | Current outcome state. |
| `fraction_done` | number 0..1, optional | Fraction complete. A halted candidate keeps the value where it stopped. |
| `current_action` | string, optional | What the candidate is doing right now. |
| `quality_score` | number 0..100, optional | Set once finished. Only the ordering matters for choosing the best. |
| `outcome_note` | string, optional | One line describing the outcome or the error. |
| `elapsed_secs` | number, optional | Time the candidate took. |
| `run_cost` | number, optional | Cost in the caller's currency. |
| `task_text` | string | What every candidate is trying to do. |
| `chosen_key` | string or null | Controlled choice. Null means the board chooses the best finished candidate. |
| `on_choose` | (candidate) => void | A candidate other than the current one was chosen. |
| `format_spend` | (amount) => string | Formats every spend value. |
| `board_title` | string | Heading. Default: "Best of N" where N is the candidate count. |

## Anatomy
In reading order:
1. Heading with the title and the task text.
2. A count line: how many candidates are finished out of the total.
3. One row per candidate, in the order given. Each row holds: the mark and display name, the method phrase, a progress bar, the current action (while running) or the outcome note (once finished), the quality score, the spend and the elapsed time.
4. For each finished candidate, a control that adopts that candidate's result.
5. A "best" marker on the best finished candidate, and a separate "in use" marker on the chosen candidate.

## States
- **Waiting (no candidate started):** rows show zero progress and a neutral action text.
- **Running:** the progress bar grows; the action text describes the current step.
- **Finished:** the bar shows full, the note replaces the action text, the score appears. The candidate becomes choosable.
- **Halted:** the bar stops at its last progress value, marked as halted; the note holds the error line. Not choosable.
- **Best:** the highest-scoring finished candidate carries the best marker. Only one candidate carries it.
- **In use:** the chosen candidate is visually emphasised; the others are de-emphasised.
- **Controlled vs uncontrolled choice:** when `chosen_key` is supplied, the board shows exactly that choice. When omitted, the best candidate is in use until the person chooses another.
- **Empty:** no candidates; shows the heading and the task text with a "nothing to compare" line.

Transitions: a candidate moves from running to finished or halted when its `run_state` changes. Scores arriving late re-evaluate the best marker. Changing `chosen_key` or pressing a choose control moves the in-use state.

## Interactions
- **Pointer, finished candidate:** the choose control adopts that candidate and calls `on_choose`.
- **Pointer, hovering a row:** brings that row's details (method, note, spend) forward if they were condensed.
- **Keyboard, Tab:** moves through the choose controls of finished candidates only. Running and halted candidates have no choose control.
- **Keyboard, Enter or Space on a choose control:** adopts that candidate.
- **Touch:** tapping a choose control adopts it. Anything shown on hover must also be reachable by tap or focus.
- **Escape and outside click:** no effect on the board.

## Motion (functional)
- New rows appear one after another in display order so the eye can follow the start of the race.
- Running progress grows continuously with a light moving highlight to show that work is live.
- When a candidate finishes, its emphasis state changes in place and its outcome text replaces the action text in the same place, so the row never changes height.
- The best marker and in-use emphasis move between rows with a short transition.
- Reduced-motion preference: everything appears at once; no moving highlights.

## Accessibility
- The rows are a group labelled with the title.
- Each finished candidate's choose control is a radio inside a radio group; its accessible name includes the candidate name and score.
- Progress bars are decorative when the text beside them already states progress; otherwise they expose the percentage.
- State is conveyed in words ("finished", "halted", "best", "in use") as well as by emphasis.
- Status changes are announced politely; a halt is announced as an alert when it happens.
- Focus is not moved when candidates update.

## Responsive
- Fills its container. When narrow, the progress bar moves under the name, the method phrase is hidden, and the score sits beside the name.
- Controls keep a touch-sized target on touch input.

## Composition hints
- Card or panel for the container.
- Badge for best, in-use and state markers.
- List (or radio group) for the rows.
- Progress bar primitive.
- Radio control for choosing.
- Icon tile for the candidate mark.

## Acceptance checks
1. Given three candidates with scores, the highest finished one shows the best marker and is in use when no `chosen_key` is given.
2. Choosing a different finished candidate calls `on_choose` once with that candidate and moves the in-use state to it.
3. A running candidate has no choose control; Tab order skips it.
4. A halted candidate keeps its progress value where it stopped and shows its error note.
5. Changing a candidate's state from running to finished does not change the row's overall height.
6. With `chosen_key` set to a candidate that is still running, the board shows it as in use and offers no choose control on it.
7. Empty input renders the heading and the empty line without errors.
8. With reduced motion requested, rows appear without animated transitions.
9. At phone width the progress bar sits below the name and the method phrase is no longer visible.

## Amendment: structure and demo

**Archetype:** a ranked row list. One row per candidate, all rows sharing one column layout. Only finished rows carry a choice control.

**Parts:**
- Heading, task line, and a count line such as "2 of 4 finished".
- Row: mark tile, name, method phrase, progress bar with the percent as text, one status slot (current action while running, outcome note once finished or halted), score (finished rows only), spend, elapsed time, and the choice control on finished rows.
- Best marker and In use marker, each a word as well as a style.
- Footer hint stating that only finished candidates can be chosen.

**Gallery demo:**
- Instance A (at rest): three candidates, all running, at 20, 45 and 70 percent: "Retry queue" on "model A", "Mock the gateway" on "model B", "Wait for settle" on "model C". Count line "0 of 3 finished". Spend and elapsed time already shown.
- Instance B: four candidates. "Retry queue" finished with a score of 72 (Best, In use). "Wait for settle" finished with a score of 64 (choosable). "Mock the gateway" halted at 35 percent with an error note. "Stub the clock" running at 60 percent.

**Acceptance checks (continued):**
10. Instance A at rest shows three running rows, each with its percent as text, its current action, spend and elapsed time. No Best marker, no choice control, and the count line reads "0 of 3 finished".
11. Instance B: the highest-scoring finished row carries Best and In use. The other finished row offers a choice control. The halted row keeps its bar where it stopped and shows its error note. The running row shows its percent.
12. Every row uses one column layout: mark, name, method phrase, bar, status, score, spend, elapsed time. The same field sits in the same column on every row, so rows stay aligned when one changes state.
13. A choice control is a radio inside a radio group. Its accessible name includes the candidate's name and score.
14. Scores appear only on finished rows.
