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
