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
