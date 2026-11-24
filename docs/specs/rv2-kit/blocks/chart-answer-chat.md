# Chart Answer Chat

Source item: affectUI Pro `blocks/revenue-chat` (functional reference only; see README).

## Purpose
A chat in which answers are built from data visuals: headline figures, a trend against a previous period, a breakdown, or a forecast, each with a short sentence. Selecting a span of the line, or a single column, lets the person put a targeted question about only that piece.

## Data
- `messages`: list of { id, role: "person" | "assistant", text?, blocks: AnswerBlock[] }.
- `AnswerBlock`: one of
  - `figures`: list of { label, value, delta_percent, delta_direction }
  - `trend`: { series_label, comparison_label, points: { x_label, value, comparison_value? }[] }
  - `breakdown`: { groups: { label, value }[] }
  - `forecast`: { points: { x_label, value, band_low?, band_high? }[], actual_until_index }
- `on_ask(text, selection?)`, where `selection` describes the chosen range or bar: { block_id, from_index, to_index }.
- `on_select_range(block_id, from, to)` (drag feedback only).
- `suggested_prompts`: list of { id, label }.
- `placeholder_text`.

## Anatomy
In reading order:
1. Conversation: person messages on one side, assistant answers on the other.
2. Answer: an optional sentence, then its blocks in order.
3. Figures block: a row of stat cards.
4. Trend block: a line chart with the current series and a comparison series; a tooltip; a range overlay while dragging.
5. Breakdown block: bars with labels and values.
6. Forecast block: a line with a band around its forecast part.
7. Composer at the bottom with suggested prompts above it when the conversation is empty.

## States
- Empty: suggested prompts shown; composer focused.
- Waiting for answer: an animated thinking line in place of the answer.
- Answer ready: blocks appear in order.
- Selection active (trend): the dragged range is highlighted; its figures are summarised in a chip above the composer.
- Selection active (breakdown): the chosen bar is marked; a chip names it.
- Selection cleared: chip removed.
- Error: the answer shows an error sentence and a retry control.

## Interactions
- Drag across a trend: highlights the range and shows the range summary in a chip. Releasing and then pressing Enter in the composer sends a question about that range.
- Click a breakdown bar: selects it; click again to clear.
- Hover on the trend: a tooltip shows the nearest point with both series values.
- Click a suggested prompt: fills and sends it.
- Composer: Enter sends; Shift + Enter inserts a line break; Escape clears the selection chip first, then clears the text.
- Keyboard on charts: Left and Right move the point cursor along the trend; Shift with Left or Right extends the range; Enter chooses the range for the next question.
- Keyboard order: conversation (each answer block), then the composer, then the selection chip's clear control.

## Motion (functional)
- Answer blocks appear in order, each after the previous one has drawn in.
- Trend lines draw from left to right; the comparison series follows the main one.
- The range overlay grows as the drag moves and eases to the clicked value when released.
- A forecast band fades in after its line has drawn.
- With reduced motion preferred, blocks appear without drawing.

## Accessibility
- Conversation is a log region; new answers are announced when they finish.
- Each chart is a labelled group; its summary (metric, range, direction of change) is read before the points.
- Figures are read as label, value and delta words.
- The selection chip is a button that clears the selection, labelled with the range it describes.
- Drag is always available by keyboard (see interactions).

## Responsive
- Wide: messages in a central column with full-width blocks.
- Phone width: stat cards wrap to a single column; charts keep full width with reduced height; the composer stays pinned at the bottom.

## Composition hints
- Card (figures and answer container), chart primitive (line with two series, bars, forecast band), chip (selection summary), composer text area, button (suggested prompts and retry), log list.

## Acceptance checks
1. Dragging across a trend highlights the range and shows a summary chip with the range's period.
2. Clicking a breakdown bar marks it and a chip names it; clicking it again clears the selection.
3. Sending a question while a selection is active includes that selection in the message's data.
4. Hovering the trend shows the nearest point with both series values.
5. Escape clears the selection chip before clearing the composer text.
6. Left and Right move the point cursor along the trend in order; Shift extends the range.
7. Answer blocks appear in the order given and the comparison series is drawn after the main series.
8. At phone width stat cards form a single column and nothing scrolls sideways.

## Amendment 5: structure and demo

Archetype: one chat panel inside a card: a question, an answer built from figures and a trend, suggestion chips, and a composer at the bottom.

Replaces the empty-only rule in Anatomy item 7 for the gallery demo: suggestion chips are shown at rest, after an answer.

Visible at rest:
- Panel title "Quarter review".
- The person's question, on the right side.
- The answer: one sentence; a figures row of three stat cards; a trend block with a legend (this period, last period), three value-axis labels, two lines, one labelled peak point (week and value), three date labels; a hint line beneath the chart.
- Suggestion chips in one row above the composer: five chips.
- Composer as the panel's last row: text field with a placeholder, a model picker at its left showing "Auto", and a send button at its right.

Gallery demo:
- Question: "How did signups change over the quarter?"
- Answer sentence: "Signups closed at 18.4k, 11% ahead of last year. Growth was steady, with a jump in week 36 that settled higher."
- Figures: Signups 18.4k, up 11.0%. New MRR $86K, up 9.4%. Churn 2.4%, down 0.6 pts.
- Trend: weekly signups, this period against last. Peak at week 36 with 1,640. Date labels W27, W33, W39.
- Chips: "Break it down by region", "Forecast next quarter", "Only paying accounts", "Compare with spring", "Why did week 36 jump?"
- Hint: "Drag across the chart to ask about a span of weeks."

Motion, settled frame: all blocks drawn; chips and composer idle.
Moves: blocks appear in order, each after the previous one has drawn. Trend lines draw left to right, the comparison line after the main line. The range overlay follows the drag.
Reduced motion: blocks appear without drawing.

Acceptance checks (continued):
9. The composer is the panel's last row, pinned to the bottom, with the model picker at its left and the send control at its right.
10. The suggestion chips sit above the composer at rest, and each one fills the composer when pressed.
11. The peak point carries a label with its week and value, and the hint line is visible at rest.

## Amendment 7: structure and demo

Replaces: Amendment 5 "Visible at rest" bullets (the panel title, the trend block and its peak label, the suggestion row count and the composer order); Amendment 5 "Gallery demo" chip list (three chips, not five); Amendment 5 checks 9 (model picker at the left) and 11 (peak label). Amendment 5 checks 10 stands for chip behaviour. New checks 12 to 17.

Archetype: one chat column, centred, holding a question, an answer built from three figure cards and a chart card with a tooltip at its peak, a source line, a suggestion row and a composer.

Stage: the card, neutral surface, hairline border, no title above the column. Width to height about 1.15 to 1. The column spans 9% to 91% of W.

Vertical positions, fractions of stage height (H):
- Question bubble, right-aligned to the column, 7% to 12% of H.
- Answer: a small mark at the column's left, text from 15.7% to 20.5% of H, two lines, spanning the column minus the mark.
- Figures row, 22.7% to 33.5% of H: three equal cards, each about 26% of W wide, with about 1.5% of W between them. Each card: a small label, a large value, and a delta line with a sign word and percentage, coloured by direction with the success or danger token.
- Chart card, 35.6% to 71.8% of H, full column width. Inside, as fractions of the card's height:
  - Header line at 7% from the card top: title text at the left, legend at the right with two entries: "This period" (solid) and "Last period" (dashed).
  - Y-axis labels at 14%, 46% and 78%: three values from the data.
  - Plot from 8% to 96% of card width. X labels at 85%: first, middle and last week.
  - This-period line solid with a light area fill below it; last-period line dashed.
  - Peak point at 68% of card width, on the this-period line.
  - Tooltip box left of the peak: right edge 2% of card width left of the peak, top at 12% and bottom at 40% of card height. It shows the week, then this period's value and last period's value, each with its series marker.
  - A vertical guide from the peak down to the base, and a dot on the last-period line at the same x.
  - Hint line at 90% of card height, left-aligned, muted.
- Source line, 75.5% of H: the label "From", then two source chips. At the column's right edge, four small icon buttons: copy, download, thumbs up, retry.
- Suggestion row, 79% to 82% of H: three chips in one row, left-aligned to the column, directly above the composer and fully visible.
- Composer, 81.3% to 94.4% of H, full column width, rounded box. Placeholder at 84.5% of H, left-aligned, muted. Bottom row at 91% of H: left, a "+" icon button and an approval chip with a hand glyph; right, a model picker (text and chevron), a mic icon, and a send icon that is muted while empty.

Gallery demo: question "How did signups change over the quarter?". Answer: one sentence. Figures: Signups 18.4k, up 11.0%; New MRR $86K, up 9.4%; Churn 2.4%, down 0.6 points. Chart: weekly signups, this period against last, peak at week 36 with 1,640. Source chips "CRM" and "Billing". Chips: "Break it down by region", "Forecast next quarter", "Why did week 36 jump?". Hint: "Drag across the chart to ask about a span of weeks."

Settled frame: all blocks drawn; the tooltip at the peak; chips and composer idle.

Acceptance checks (continued):
12. The column holds, top to bottom: one right-aligned question, one answer, one figures row of three equal cards, one chart card, one source line, one suggestion row, one composer. No panel title.
13. The chart card has a header line with a legend, three y-axis value labels, and three x week labels (first, middle, last). No label repeats.
14. The peak point carries a tooltip at rest showing its week and both series values; the tooltip sits left of the peak.
15. The source line shows two source chips and four icon buttons, ending at the column's right edge.
16. The composer's bottom row has "+" at the left, the approval chip, then at the right a model picker with a chevron, a mic and a send. The model picker is right of the text area, not left.
17. The suggestion row sits directly above the composer and every chip shows in full; the composer does not cover any part of it.
