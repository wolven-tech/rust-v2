# Working Trail Panel

Source item: affectUI Pro `components/thinking-state` (functional reference only; see README).

## Purpose
Shows what an assistant is doing while it works: a live elapsed timer, the sources it is searching, and the steps of its reasoning appearing one after another. It builds trust during longer waits and ends with a completion marker and the time taken.

## Data
| Field | Type | Meaning |
|---|---|---|
| `trail_steps` | `Step[]` | Ordered. Each step is either a search (a query label plus sources) or a thinking line. Sources: `source_label`, `source_href?`, `source_mark?`. |
| `hidden_source_count` | number, optional, per step | Sources hidden behind a "more" control. |
| `done_label` | string | Final line. Default "Finished thinking". |
| `step_pause` | number | Wait between steps. A configuration default, not a visual value. |
| `repeat_sequence` | boolean | Restarts the sequence after it finishes, for demonstrations. |
| `fold_when_done` | boolean | Folds the steps away when finished; the header stays. |
| `on_finished` | (elapsed_secs) => void | Called once when all steps are complete, with the elapsed seconds. |

## Anatomy
In reading order:
1. Header row: a live elapsed-time label, a status word ("Thinking" while working), and a small animated mark. Acts as a disclosure control for the steps.
2. Timeline: steps in order, joined by a line that grows down to each new step.
3. Each step: its label and any sources as small chips; the active step has a pulsing marker.
4. "More sources" control where sources are hidden.
5. Final row when done: a completion check and the done label with the total time.

## States
- **Working:** header shows the timer and the working word; the mark animates; the active step is marked.
- **Searching step:** shows its query and sources appearing one by one.
- **Thinking step:** shows its text.
- **Done:** completion check and done label with the elapsed time; `on_finished` fired.
- **Collapsed:** steps hidden; header remains and shows the done summary.
- **Expanded:** steps visible.
- **Sources partly revealed:** the "more" control shows the remaining count; activating it reveals them.

Transitions: a step appears after the previous one; the timer stops at done; collapsing and expanding toggles the steps only.

## Interactions
- **Pointer, header:** toggles the steps open or closed.
- **Pointer, source chip:** follows its link when it has one; hover or focus shows the link as a small tooltip.
- **Pointer, "more" control:** reveals the hidden sources.
- **Keyboard, Enter or Space on the header:** toggles the steps.
- **Keyboard, Enter or Space on "more":** reveals hidden sources.
- **Keyboard, Tab:** moves through source links in order, each showing its tooltip on focus.
- **Touch:** tapping a source follows its link; a tap on a source with a tooltip shows the tooltip first.

## Motion (functional)
- Steps fade in one after another; the timeline grows down to each new step.
- Sources appear one by one as small chips.
- The active step has a pulsing marker and a moving highlight.
- Hovering the header lightens its words and reveals the disclosure chevron.
- On completion a check appears and draws itself.
- Reduced-motion preference: everything appears without animation; the timer still updates its figure.

## Accessibility
- The header is a button with an expanded state; collapsing hides only the steps.
- New steps are announced politely through a live list.
- Source links are focusable; their tooltip shows on focus as well as hover.
- Completion is announced once.

## Responsive
- No card; sits on the page and works from a narrow width. Tighter padding on narrow containers and larger tap targets on touch screens.

## Composition hints
- Plain container (no card).
- Disclosure header button.
- Vertical timeline with nodes.
- Chip list for sources, with tooltip.
- Live list for announcements.
- Check icon for completion.

## Acceptance checks
1. Steps appear in order, one after another, with the configured pause between them.
2. The header timer keeps counting until the last step is done, then stops and shows the done label with the elapsed time.
3. Pressing Enter on the header collapses the steps and leaves the header visible.
4. `on_finished` is called exactly once with the elapsed seconds.
5. "More sources" reveals the hidden count and is removed once all are shown.
6. A source chip with a link opens its link on activation; focusing it shows the link text.
7. With `repeat_sequence` true the sequence restarts after completion.
8. With reduced motion requested, steps appear without fade and the timer still updates.

## Amendment: structure and demo

**Archetype.** A plain vertical timeline laid straight on the page, with no card around it. A header row sits at the top; steps hang from a rail beneath it, one node per step; a completion row closes the list once done. It is not a card and not a progress bar.

**Parts that must exist:**
- Header row: a live timer, a status word, a small mark, and a disclosure chevron. The whole row toggles the steps.
- Steps on a rail, each with a node. Two kinds: a search step (a query label plus source chips) and a thinking line (one sentence).
- Source chips on search steps; a "more" control when some sources are hidden, showing the hidden count.
- Active step: its node is marked.
- Completion row: check mark, done label, elapsed time. Absent until done.

**Gallery demo data:** four steps. Step 1 is a thinking line. Step 2 is a search with two visible chips and one hidden source. Step 3 is a thinking line. Step 4 is a thinking line and is the active step.

**States visible at rest in the gallery:** working. Timer running, status word shown, step 4 active, search step partly revealed, no completion row. A gallery control moves the panel to finished.

**Acceptance checks, continued:**
9. Gallery at rest: the timer is running, the status word reads as working, the active step has its marker, and no completion row is rendered.
10. A search step with hidden sources shows the hidden count on its "more" control; activating the control reveals those sources and removes the control.
11. Collapsing the steps leaves the header visible with the done summary when the panel has finished.
12. The gallery's finish control stops the timer at its last value, renders the completion row with a check, the done label and the elapsed time, and calls the finish handler once.

## Amendment 2: structure and demo

Replaces amendment 1 in three places. The panel holds no action controls: the gallery's finish and restart controls sit in a separate row outside the component frame. While working, the rail shows the active step and up to three steps before it; earlier steps are folded away and return when the header is expanded. The header is a single line. The timeline stays a plain column with no card around it.

**Layout archetype:** a plain column on the page. A single header line, then a short rail of the most recent steps, the active step at its foot. A completion line closes the rail once done. No action controls sit inside the column.

**Parts that must exist:**
- Header line: live timer, status word, mark, and disclosure chevron. The whole line toggles the rail.
- Rail: the active step and up to three steps before it.
- Steps: a search step (a query label and source chips) or a thinking line (one sentence). The active step carries its marker.
- "More" control on a search step with hidden sources, showing the hidden count.
- Completion line: check, done label, elapsed time. Absent until done.

**Gallery demo (ours):** four steps. Step 1 is a thinking line. Step 2 is a search with a query label, two visible source chips and one hidden source. Step 3 is a thinking line. Step 4 is a thinking line and is active. The timer runs and the status word reads as working. No completion line. Beneath the frame, outside the component, a control row offers "Finish" and "Restart".

**Acceptance checks (continued):**
13. At rest the header timer is running, the status word reads as working, the active step is marked, no completion line is rendered, and the component's own markup contains no button or input element (SSR test).
14. With five or more steps, the rail shows the active step and three before it. Earlier steps are absent from the rail until the header is expanded (SSR test with five steps).
15. Finish, from the outside control, stops the timer at its last value, adds the completion line with check, done label and elapsed time, and calls `on_finished` once. Replaces the in-panel control of amendment 1 (check 12).
16. The "more" control on a search step shows the hidden count; activating it reveals the hidden sources and removes the control (check 10).
17. Collapsing the header leaves the header visible with the done summary when the panel has finished (check 11).
