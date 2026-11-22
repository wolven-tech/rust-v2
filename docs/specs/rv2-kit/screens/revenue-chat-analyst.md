# Revenue Chat Analyst

Source item: affectUI Pro `screens/ai-chat-revenue` (functional reference only; see README).

## Purpose
A chat that answers revenue questions with something to look at: headline figures, a trend against the same period last year, a split by region, or a forecast with a likely range. A person can select part of a chart and ask about only that part.

## Data
- `turn_list`: a list of either:
  - question: `{ id, author_role: "user", text, quoted_focus? }`, or
  - answer: `{ id, author_role: "assistant", text, metric_cards?: [{ entry_label, value_text, change_text? }], chart_block?, caption_text?, source_list?, follow_up_list?: string[] }`.
- `chart_block` has a `chart_kind`:
  - `"trend"`: `{ category_labels, current_values, previous_values, marker? }`, drawn as an area with a comparison line.
  - `"forecast"`: `{ category_labels, actual_values, forecast_values, target_value? }`, drawn as a line with a dashed continuation and a target line.
  - `"split"`: `{ title_text, previous_total?, groups: [{ entry_name, parts: [{ entry_name, entry_value }] }] }`, drawn as stacked columns, one per group, each divided into its parts. Trend values are in thousands of currency units; split values are whole units.
- `focus_range`: passed to `on_ask` when the question concerns part of a chart: `{ chart_kind, from_point, to_point, label_text, detail_text }`. An example label form is "first point to last point". The question appears in the chat with this range quoted above it.
- `chat_list`: chats in the sidebar.
- `is_loading`, `is_empty`.
- Callbacks: `on_ask(question, focus_range?)` (returns an answer, or a promise of one), `on_navigate(entry_label)`.
- Without `on_ask`, the screen answers from built-in sample questions; answers about a picked selection are computed from the chart's own values.

## Anatomy
1. Sidebar (shared app frame), folded to icons by default.
2. Thread column, centred at a readable maximum width. Question bubbles sit on the right. Each answer shows a short text paragraph, then a row of metric cards, then the chart block with its caption, then sources and follow-up suggestions.
3. Composer at the foot, with suggestion chips above it.
4. Pick field: a small input that appears above a selected range on a chart, showing what is picked and its figures, along with a question field.

## States
- **empty** (`is_empty`): a short note and a set of starter questions.
- **working**: a status line with the assistant's mark and "Reading sources…"; answer content is not yet shown.
- **streaming**: answer text appears word by word; metric cards and the chart appear after the text.
- **range picked**: the stretch of a line chart, or a bar, is highlighted with a band or ring.
- **pick field open**: the field sits over the pick with the question input; the chart's own hover tips step aside.
- **answer error**: the answer block shows the error text and a retry control.
- **loading** (`is_loading`): placeholder lines, cards, and chart shape.

## Interactions
- Tab into a chart: arrow keys step through its points; Home and End jump to the first and last point.
- Line chart: Enter begins a selection at the highlighted point; the left and right arrows widen it; a second Enter opens the pick field.
- Bar: Enter picks the bar and opens the pick field.
- Enter in the composer or in the pick field sends the question (with the pick, if there is one).
- Escape in the pick field closes it and returns focus to the chart.
- Shortcuts do not fire while typing, except Enter and Escape.

## Motion (functional)
- While working, the assistant's mark shows a subtle live indicator beside its status text.
- The answer text reveals word by word with a caret; metric cards and the chart then fade up after it.
- A pick on a line chart shows a band over the picked stretch, with a marker line on each side. A picked bar gets a ring.
- Reduced motion: everything is shown at once.

## Accessibility
- Each chart can take focus and is labelled by its caption. Moving across its points reveals that point's values in a tooltip, which assistive technology reads aloud.
- The pick field is named with what is picked and its figures. The key hints appear once the chart has keyboard focus.
- While words are still arriving, the answer is flagged as busy. A polite live region announces when work begins and when it ends.
- Focus rings show for keyboard users only, never on text fields.

## Responsive
- The sidebar starts folded to icons so the thread has room; the column keeps a fixed maximum width in the middle.
- Phones: metric cards go two to a row; charts keep their height; a horizontal swipe over a line chart picks a stretch, and a tap picks a bar; the hint beneath the chart names the gesture for the device.
- Large numbers shorten with a magnitude suffix; a value too long for its card truncates, with the full value on hover.
- The pick field uses a text size that stops the phone zooming on focus.
- Touch targets are enlarged for touch input.

## Composition hints
Metric card; area chart with comparison line; stacked bar chart with legend; forecast line chart with dashed continuation; selection band overlay; popover field; composer; suggestion chips; chat list; skeleton; empty state.

## Acceptance checks
1. Metric card values agree with the totals of the chart they accompany.
2. On a line chart, arrow keys extend the picked range, and the field's label shows its first and last points.
3. Escape in the pick field returns focus to the chart with the pick cleared.
4. Choosing a bar and asking sends `on_ask` with a focus range covering only that group.
5. A forecast chart shows the dashed continuation starting after the last actual value.
6. Each stacked bar's segments add up to the bar's total.
7. Large values are shortened in metric cards, and the full value is available on hover.
8. On a phone, a horizontal swipe over a line chart creates a picked range.
