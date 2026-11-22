# Run Timeline Rows

Source item: affectUI Pro `components/agent-timing` (functional reference only; see README).

## Purpose
Shows, on one shared time axis, when each person and agent in a run was thinking, calling a tool, or waiting on the person, and where work passed from one party to another, so a person can find where a run loses time.

## Data
| Field | Type | Meaning |
|---|---|---|
| `run_heading` | string | Name of the run. |
| `rows` | RowSpec[] | One row per party, top to bottom. |
| `spans` | Span[] | What each party did, in seconds from the run start. |
| `transfers` | Transfer[] | Hand-offs between rows. |
| `clock_s` | number? | Seconds since start for the current moment. Absent for a finished run. Spans without an end grow to this point. |
| `axis_length_s` | number? | Seconds shown on the axis. Default: run length plus a little room. |
| `describe_span` | (span: Span) => string? | Optional text for a span's card; defaults to the built-in wording. |

RowSpec: `{ row_key, display_name, kind: "person" | "agent", tint_key?, mark_source? }`.

Span: `{ span_key?, row_key, mode: "reasoning" | "tool" | "waiting", begin_s, end_s?, tool_name?, caption? }`.
- `reasoning` is a low step. `tool` is a high step and shows `tool_name` under its line. `waiting` is a step where the run waits on the person; it is marked by a distinct line style and words, not colour.
- Default `span_key` is `row_key` plus `begin_s`; set an explicit key when two spans of a row can start in the same second.

Transfer: `{ transfer_key?, from_row_key, to_row_key, at_s, caption? }`.

## Anatomy
1. Heading bar: run name and per-row counts.
2. Time axis across the top: tick marks at intervals chosen to keep roughly one tick per fixed width, labelled in seconds, minutes or hours.
3. Rows: each row has a name block on the left (name, and on the right of the name a short present-tense note of what it does now) and a plot lane on the right.
4. In each lane, a baseline sits at rest. Reasoning steps lift a little, tool steps lift higher and carry the tool name beneath, waiting steps sit at their own level with the distinct style. Adjacent steps join with a soft filled area beneath.
5. A running step extends to the clock and ends in a small pulsing dot at that point.
6. Transfers: a thin line from the source row to the destination row, drawn as the clock passes its time, ending in a small dot at the landing row.
7. Card: activating a step or a transfer opens a card with who, what, start, and duration; the card shows the caption if provided.

## States
- Live run (`clock_s` set): running steps extend to the clock and pulse; new steps appear as the clock passes them.
- Finished run (`clock_s` absent): all spans have ends; no pulse; no clock dot.
- Waiting span: waiting level with words "Waiting on you" shown in the bar.
- Row with no spans: resting baseline only.
- Span with `end_s` before `begin_s`, or non-numeric times: omitted from the plot.
- Card open: exactly one card at a time.

## Interactions
- Tab: one stop for the diagram.
- Left or Right arrow: moves to the previous or following span within the current lane, opening its card as focus lands.
- Up or Down arrow: nearest step in the adjacent row.
- Home or End: first or last step in the current row.
- Esc: hides the card.
- Pointer: hover or click a step or transfer shows the same card.

## Motion (functional)
- A newly started span lifts from the baseline to its own height; a span that finishes drops back to the baseline.
- A running step grows with the clock.
- A transfer draws itself as the clock passes its time; then a small landing dot appears.
- All positions derive from the clock, so repeated renders stay consistent.
- Reduced motion: the pulse and idle indicator stop; steps still follow the clock.

## Accessibility
- Steps and transfers form a list. Each item reads who, what, when and how long, for example "Reviewer, reasoning, from 0:42 for 3.0 s".
- Waiting-on-person is stated in words in the bar, not only by style.
- Each step names its tool and state inside its block when there is room.
- Focus ring visible for keyboard focus only.

## Responsive
- Time axis fits the available width; tick spacing adapts.
- Narrow widths: row names collapse to their markers and per-row counts move under the heading.
- Theme-aware: follows the host's light or dark theme.
- No horizontal page overflow at phone width.

## Composition hints
Heading bar, time axis with tick labels, row list with label block and plot lane, step shapes built from path or block primitives, landing dot, floating card, list semantics.

## Acceptance checks
1. A span with no `end_s` in a live run reaches exactly `clock_s` on the axis; a finished run draws no clock dot.
2. Setting `clock_s` forward extends running spans and shows a transfer whose `at_s` has passed; a transfer with a future `at_s` is not yet drawn.
3. A `tool` span shows its `tool_name` under the line; a `reasoning` span does not.
4. A `waiting` span displays "Waiting on you" in words within the bar.
5. Right arrow from a step moves focus to the next step in the same row and opens its card; Esc hides it.
6. Down arrow moves focus to the nearest step in the row below.
7. A span with `end_s` earlier than `begin_s` is not drawn and does not throw.
8. The accessible name of a step contains who, what, start and duration.
9. Tick labels switch units across ranges: seconds, then minutes, then hours.
10. At phone width the page does not overflow horizontally.

## Amendment 3: structure and demo

**Replaces:** none. This spec had no earlier amendment, so checks 1 to 10 stand. The changes below alter only the gallery's resting state.

**Archetype:** one run on a shared axis that is still early. Few steps are drawn, and lanes with nothing to show stay at their baseline.

**Parts, top to bottom:**
- Heading bar: run name, "5 rows · 3 steps", and the clock figure.
- Time axis across the top, labelled from zero to one minute.
- Five rows, top to bottom: Maintainer (person), Scout, Drafter, Auditor, Courier (agents). Each row has a name block with a present-tense note, and a plot lane.
- Steps and transfers are drawn only up to the clock.
- Card: closed at rest.

**Gallery demo (our data):** run "Refund batch", clock at 52 seconds, axis length 60 seconds.
- Scout, tool step `fetch_index`, 8 to 26 seconds, finished. Note: "reading the index".
- Drafter, reasoning step, 26 to 44 seconds, finished. Note: "writing the summary".
- Auditor, reasoning step, from 44 seconds, in flight and drawn up to the clock. Note: "comparing totals".
- Transfer from Drafter to Auditor at 44 seconds.
- Maintainer, note "reviewing later", no steps. Courier, note "holding the batch", no steps.

**Acceptance checks (continued):**
11. At rest the gallery draws three steps and one transfer. Maintainer and Courier show only their baseline and their note.
12. The heading reads "Refund batch", "5 rows · 3 steps" and the clock figure. No card is open.
13. The in-flight Auditor step ends at the clock point and carries the live marker; no step extends past the clock. The transfer is drawn because the clock has passed its time.
14. Scout's tool step shows `fetch_index` beneath its line; the two reasoning steps show no tool name.
15. Pointer or focus on the Drafter step opens its card, naming who, what, start and duration. Moving focus to the Drafter-to-Auditor transfer replaces that card with the transfer's card.
