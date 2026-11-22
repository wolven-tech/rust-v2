# Exchange Diagram

Source item: affectUI Pro `components/agent-sequence` (functional reference only; see README).

## Purpose
Lays out the message traffic of one agent run (person, agents, tools) in chronological order. A reader can trace dependencies between requests and replies, spot concurrent branches, and locate retries or failures.

## Data
| Field | Type | Meaning |
|---|---|---|
| `actors` | Actor[] | Columns, left to right. Each has a lifeline. |
| `exchanges` | Exchange[] | Messages in the order they happened. |
| `run_heading` | string | Title of the run. |
| `run_is_live` | boolean | `true` while the run continues; `false` once it is over, which stops working indicators. |
| `show_time_gutter` | boolean? | Force the time column on or off. Default: on while any exchange carries a time. |
| `format_clock` | (ms: number) => string | Formats the gutter and detail times. Default: `m:ss`, with hours added past one hour. |
| `on_pick` | (exchange: Exchange \| null) => void | Fires when an exchange is opened, and with `null` when its card closes. |

Actor: `{ actor_key, role: "person" | "agent" | "tool", caption, mark_source?, tint_key?, model_tag?, summary? }`.
The first agent actor takes the primary tint; other agent actors take the palette in order. Tints are tokens, not literal values in the spec.

Exchange: `{ exchange_key, from_key, to_key, kind: "request" | "answer", answers_key?, state: "ok" | "failed" | "awaiting_person", sent_at_ms?, group_tag?, headline, detail_text? }`.
- `answers_key` on an answer links it to the request it closes.
- `group_tag` on consecutive exchanges shows them inside one frame with that tag, for example a parallel group, a retry, or a repeat count.

## Anatomy
1. Heading row: run title, actor count, exchange count, run span.
2. Actor header: one chip per actor. Chips are the only way to open an actor card.
3. Lifelines: one vertical line beneath each chip.
4. Work bars: while a request is open, its receiver's lifeline carries a bar that grows downward and closes when the answer arrives.
5. Exchange rows: each row is one arrow from sender to receiver. A request is a solid line; an answer is a visually distinct reply line running back. Self-addressed exchanges loop back to the same lifeline.
6. Headline tags: short text set on a solid tag in the line's tint, placed over the line. Long headlines truncate with an ellipsis; the full text appears on hover and in the card.
7. Group frames: consecutive exchanges sharing a `group_tag` sit inside one framed box with the tag at its corner.
8. Time gutter: left-hand column of times, one per exchange that has `sent_at_ms`.
9. Cards: an actor's card anchors to its chip; an exchange's card anchors to its row. Both show the full text, the model tag and summary (actors) or the detail text and timing (exchanges).

## States
- Exchange `ok`: normal line.
- Exchange `failed`: the arrow ends in a cross at the receiver; the row text says "Failed" in words.
- Exchange `awaiting_person`: marked in words as waiting for the person; its tag carries the warning tint.
- Request without answer: receiver's work bar stays open and shows the working indicator while `run_is_live`.
- Run over: work bars stop; markers stop animating.
- Empty run: heading and actor chips only; no rows.
- Actor with no exchanges: chip and lifeline shown, no bar.
- Invalid data (exchange naming an unknown actor, non-numeric time): the exchange is omitted or shown without a time; the rest of the diagram renders.
- No room to the right of an exchange's anchor: its card opens to the left; on phone width it opens below the row.

## Interactions
- Tab: moves through actor chips, then into exchange rows, as one stop per group.
- Up or Down arrow: next or previous exchange.
- Home or End: first or last exchange.
- Enter or Space on an exchange row or actor chip: opens its card.
- Esc: closes the card and returns focus to the element that opened it.
- Pointer activation of a row or chip: same as Enter.

## Motion (functional)
- A new exchange draws across from its sender toward its receiver with a highlight at the leading end and settles with a small ripple at the landing point. Its headline rises in after the line lands.
- Each line fades in from its origin to full opacity at its destination, so direction reads.
- A request opens a work bar on the receiver's lifeline that grows downward; a light moves down open bars while the run is live.
- Working indicators on active agents animate subtly.
- Cards pop in beside their anchor.
- Reduced-motion preference: exchanges appear without animation; bars and indicators stop.

## Accessibility
- Exchanges form an ordered list. Each item is named with sender, receiver, headline, state and time, and states that it opens a card.
- Actor chips act as buttons. A card presents itself as a dialog labelled with its subject, and its dismiss control sits inside the normal focus sequence.
- New exchanges are voiced without interrupting the reader.
- No meaning by colour alone: replies are identified by text and line style; failures say "Failed" and show a cross; group frames carry their tag as text.
- Focus rings appear only for keyboard focus, in both themes.

## Responsive
- Actor columns share the available width evenly. Below a minimum column width the diagram scrolls horizontally inside its container, with an edge cue on the side that has more content.
- Phone width: narrower time gutter and smaller time text; larger touch targets around chips and close buttons.
- Rows are fixed height, so a new exchange never moves rows above it; the diagram grows downward only.
- Long headlines ellipsize; the diagram never overflows the page horizontally.
- Theme-aware: follows the host's light or dark theme.

## Composition hints
Chip button with avatar or mark, vertical list with lines, tag label, floating card layer (popover or dialog), live region, scroll container with edge cues, time formatter.

## Acceptance checks
1. Exchanges render in array order; an answer's line runs in the opposite direction to its request.
2. An exchange naming an unknown actor is not drawn; the remaining exchanges still render.
3. Pressing Down moves focus to the next exchange row; Home moves it to the first.
4. Enter on a row opens its card as a dialog; Esc closes it and returns focus to the row.
5. Adding an exchange at the end does not change the vertical position of existing rows.
6. Consecutive exchanges with the same `group_tag` sit within one frame showing the tag as text.
7. Times format with `format_clock`; a negative time shows zero; a run longer than an hour shows hours.
8. At phone width the diagram scrolls horizontally inside its container and the page itself does not overflow.
9. A failed exchange carries the word "Failed" in its accessible name.

## Amendment 2: structure and demo

This spec had no first amendment, so this is its first structural note. Checks continue from 10.

**Archetype: lifelines with arrows.** Actor chips sit in one row across the top, each with a vertical lifeline beneath it, in the order the run involves them. A time gutter runs down the left. Each exchange is a horizontal arrow between two lifelines at its own height, with its headline set as a tag on the arrow. Exchanges are not boxed; the only frames are group frames.

**Parts that must exist**
- Header line: run heading, count of working exchanges, count of messages, run span.
- Actor chips in one row, each with its mark and caption. A chip is the only way to open an actor card.
- Lifeline per actor. A work bar on the receiver's lifeline stays open while a request has no answer.
- Arrow per exchange: a request one way, its answer back the other way. A self-addressed exchange loops back on its own lifeline.
- Headline tag on the arrow. Long headlines truncate; the full text is in the card.
- State word at the receiving end of the arrow. A failed exchange ends in a cross and reads "Failed". An exchange waiting on the person reads "Needs your reply" and is marked as needing the person.
- Time gutter, showing one time beside each timed exchange.

**Gallery demo counts**
- Instance one: five actors (You, Planner, Researcher, Writer, Reviewer) and two exchanges. A request from You to Planner is unanswered, so Planner's work bar is open. A self-addressed exchange on Planner's lifeline loops back.
- Instance two: five exchanges. One answered request and its answer, one failed request, one request awaiting the person, and the self-addressed loop.

**Visible at rest**
- Instance one: five chips in one row, each with its lifeline; the header reads one working and two messages; the open work bar on Planner's lifeline; the headline tag on the request arrow; no framed exchange boxes.
- Instance two: the answer arrow running opposite to its request; the failed exchange with its cross and word; the awaiting exchange marked as needing the person.

**Acceptance checks (continued)**
10. Instance one shows five actor chips in one row, each with its lifeline beneath. The header reads one working and two messages.
11. The request in instance one has its headline tag on the arrow and an open work bar on the receiver's lifeline. The self-addressed exchange loops back on its own lifeline.
12. No exchange is drawn inside a framed box, except inside a group frame, which carries its tag as text.
13. Instance two shows the answer arrow running opposite to its request, the failed exchange with a cross at the receiver and the word "Failed", and the awaiting exchange marked as needing the person.
14. The rows of instance one keep their vertical positions in instance two, which only adds exchanges below them.
