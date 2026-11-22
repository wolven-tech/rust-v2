# Schedule Variance Timeline

Source item: affectUI Pro `components/plan-vs-actual` (functional reference only; see README).

## Purpose
Shows how an agent run is going against its plan, step by step. Each step has a planned length and an actual length. Overruns are visible, and the effect of an overrun on later steps and on the expected finish is shown.

## Data
| Field | Type | Meaning |
|---|---|---|
| `step_list` | `Step[]` | Ordered list of steps. |
| `step_key` | string | Stable key. |
| `step_label` | string | Name of the step. |
| `planned_span` | number | Planned length of the step, in the run's time units. |
| `waits_for` | string[], optional | Keys of steps that must finish first. A step with none is planned to start at zero. |
| `actual_start` | number, optional | Actual start, as an offset from the start of the run. |
| `actual_end` | number, optional | Actual end, as an offset from the start of the run. Omit while running or waiting. |
| `step_mark` | ReactNode, optional | Mark for the step. |
| `clock_point` | number | Current position in the run, as an offset from its start. Running steps grow to this point. |
| `run_purpose` | string, optional | What the run is for. |
| `format_time` | (offset) => string | Formats axis labels and numbers. Default: minutes and seconds. |
| `board_title` | string | Heading. Default "Plan vs actual". |

Callers convert their own event timestamps to offsets from the start of the run, all in one shared unit.

## Anatomy
In reading order:
1. Heading with the title, the purpose text, and a summary line: elapsed time against planned finish, and the estimated finish with its difference from plan.
2. A time axis across the top.
3. One row per step, in the order given. Each row: the step's mark and label, a planned bar (drawn as an outline), an actual bar over it, the duration figure with its variance in words, and a state word.
4. A planned-finish marker and an expected-finish marker on the axis.
5. Dependency links drawn between rows, shown when a row is focused or hovered.

## States
- **Waiting:** a dependency has not finished. The row shows a dashed planned outline only. The expected start moves when a dependency moves.
- **Running:** the actual bar grows with `clock_point`. Where its length passes the planned length, the overrun portion is shown with an alert emphasis and the words say how far behind.
- **Done on time or early:** the actual bar stops at its end point; the words say ahead or on time.
- **Done late:** as done, with the overrun shown and worded.
- **Empty:** heading and an empty-state line; no axis.
- **Run finished:** the expected-finish marker becomes the actual finish marker; running indicators stop.

Transitions: a waiting step becomes running when all its dependencies have an end point and `clock_point` passes its start. Adding an end point to a running step ends its growth.

## Interactions
- **Pointer, hover on a row:** highlights its dependency links to earlier steps and the links from later steps that wait for it.
- **Touch:** tapping a row selects it, which shows the same links as hover.
- **Keyboard, Tab:** moves through rows in order. The focused row shows its links.
- **Keyboard, Enter:** selects the focused row (same as tap).
- **Escape and outside click:** clear the selection.

## Motion (functional)
- Running bars grow with the run; a moving highlight on the leading edge shows that work is live.
- When an earlier step overruns or finishes early, the rows after it glide to their new expected positions, so cause and effect are seen together.
- The expected-finish marker moves when its value changes; the axis widens when the run needs more room.
- Overrun portions change emphasis as they begin, so the change is noticed.
- Reduced-motion preference: positions update instantly; no moving highlights.

## Accessibility
- Rows form a list; each row is focusable and its accessible name states the step label, its state, and its variance in words (for example "two seconds behind plan").
- Emphasis is not the only signal: late and early are also stated in words and with signed figures.
- The chart area is hidden from assistive technology; the list carries all information.
- State changes are announced politely at a reasonable rate, not on every tick.

## Responsive
- Rows keep their height so nothing jumps as steps start and finish.
- At narrow widths the label column narrows, the purpose text and icons are hidden, and axis labels reduce to start and end values.

## Composition hints
- Card for the container.
- List and list item for rows.
- Bar chart primitive with an outline layer for planned and a fill layer for actual.
- Badge or state word for each row.
- Timeline axis primitive.
- Tooltip is optional; do not rely on it for information.

## Acceptance checks
1. A step whose dependency has not finished shows only a planned outline and a waiting state word.
2. A running step's actual bar length equals `clock_point` minus its start, capped at `clock_point`.
3. A done step whose actual length exceeds its plan shows its variance as late in words and as a signed figure.
4. Adding an end point to a previously running step freezes its bar at the end point.
5. Hovering a row reveals the links to the steps it depends on and to the steps that depend on it.
6. A step with no dependencies is laid out to start at zero.
7. Each row's accessible name includes its label, state and variance.
8. With reduced motion requested, a change in an earlier step updates later rows without a glide.
9. At narrow width the purpose text and icons are not visible and rows keep their height.

## Amendment: structure and demo

**Archetype:** a dependency timeline. One row per step. Each row carries a planned length as an outline and an actual length as a fill, both on one shared time axis. The summary sentence sits above the rows.

**Parts:**
- Heading, purpose line, and a summary sentence stating elapsed time, planned finish, expected finish and the difference between the last two.
- Time axis across the top with labelled ticks.
- Row per step: mark, label, planned outline, actual fill, duration figure, variance in words with a signed figure, state word.
- Planned-finish marker and expected-finish marker on the axis. They are labelled differently and shown together only when they differ.
- Dependency links, drawn only for the focused or hovered row.

**Gallery demo:** six steps, durations in minutes. Chain: "Gather sources" waits for nothing and finished early; "Draft outline" waits for it and finished late; "Write section A" waits for "Draft outline" and is running; "Write section B" waits for section A; "Review" waits for section B; "Publish" waits for "Review". Those three waiting rows are the waiting state. The expected finish sits later than the planned finish.

**Visible at rest in the gallery:** one early row, one late row, one running row, three waiting rows, the summary sentence, both finish markers. No links drawn.

**Acceptance checks (continued):**
10. The gallery at rest shows six rows: one done early, one done late, one running and three waiting. Each row's accessible name includes its label, state and variance in words.
11. The summary sentence gives elapsed time, planned finish, expected finish and the signed difference, in the same units as the axis. The two finish markers are both on the axis and labelled differently.
12. No dependency links are drawn at rest. Focusing "Review" draws links to the two steps it waits for and to "Publish", which waits for it.
