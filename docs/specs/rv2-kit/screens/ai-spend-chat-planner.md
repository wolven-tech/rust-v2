# AI Spend Chat Planner

Source item: affectUI Pro `screens/ai-chat-spend` (functional reference only; see README).

## Purpose
A conversation about what AI usage costs. Answers arrive with charts: spend broken down by provider and model, and a weekly trend. An answer can include a plan that the person tunes: moving the levers updates the projected savings and a six-month forecast. A person can select part of a chart and ask about only that part.

## Data
- `turn_list`: as in the Revenue Chat Analyst: question turns and answer turns carrying `metric_cards`, `chart_block`, `caption_text`, `source_list`, and `follow_up_list`.
- `chart_block.chart_kind`:
  - `"split"`: `{ title_text, previous_total?, groups: [{ entry_name, logo?, tone_token?, parts: [{ entry_name, value_dollars }] }] }`, drawn as stacked bars: one bar per provider or team, one segment per model.
  - `"trend"` and `"forecast"`: as in the Revenue Chat Analyst, with values in thousands of dollars.
  - `"plan"`: `{ current_monthly, months, growth_rate, lever_list }`, drawn as the totals, the levers, and a forecast line.
- `lever_list`: list of `{ id, entry_label, detail_text, lever_kind: "range" | "switch", monthly_base_dollars, max_saving_share, max_percent?, entry_value }`.
  - `monthly_base_dollars`: the monthly spend the lever affects.
  - `max_saving_share`: the fraction of that spend saved when the lever is fully engaged.
  - `entry_value`: for a range, a percentage from 0 up to `max_percent`; for a switch, 1 (on) or 0 (off).
- Savings rule (this product's design): a lever saves `monthly_base_dollars × max_saving_share × intensity`, where intensity is `entry_value / max_percent` for a range and `entry_value` for a switch. Total savings is the sum over all levers.
- Forecast rule: the baseline grows by `growth_rate` per month from `current_monthly`. The planned line is the baseline multiplied by `(1 − total_savings / current_monthly)` for each month.
- `focus_range`: passed to `on_ask` when a question concerns part of a chart (as in the Revenue Chat Analyst).
- `chat_list`, `is_loading`, `is_empty`.
- Callbacks: `on_ask(question, focus_range?, plan_snapshot?)` (returns an answer or a promise), `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame), folded to icons by default.
2. Thread column with a fixed maximum width. Answers include metric cards, a split chart, and a trend or forecast chart.
3. Plan card (inside an answer): the levers as a list. Each range lever is a slider with its label, its current value, and the monthly saving it yields. Each switch lever is a toggle. Beneath the levers: total monthly savings, new monthly spend, and the six-month forecast line against the baseline.
4. A "Turn into rules" button under the plan card. It asks a follow-up question with the plan quoted above it.
5. Composer at the foot, with suggestion chips.

## States
- **default levers**: each lever at its supplied value.
- **lever changed**: totals and the forecast update; the forecast line moves to its new path.
- **all levers off**: savings show zero; the forecast equals the baseline.
- **empty** (`is_empty`): a short note and starter questions.
- **working**: a status line with the assistant's mark and the words "Reading your usage…"; content not yet shown.
- **loading** (`is_loading`): placeholder lines, cards, and chart shapes.

## Interactions
- Tab moves through levers, charts, and buttons.
- Range slider: left and right arrows change the value in steps of 5 percent of the maximum; Home and End set the minimum and maximum.
- Switch: Space toggles it.
- Split chart bar: Enter selects it; Enter again asks about it.
- Line chart: as in the Revenue Chat Analyst (arrow keys step, Enter selects, Enter asks).
- Escape closes a pick field and returns focus to the chart.
- "Turn into rules" calls `on_ask` with the plan snapshot.

## Motion (functional)
- Changing a lever rolls the totals to their new values, and the forecast line moves to its new path.
- While working, the assistant's mark shows a subtle live indicator.
- Reduced motion: values and lines are set directly.

## Accessibility
- Every slider is labelled with its lever and reads out the saving in plain words, for example "Batch jobs: 120 dollars a month saved".
- Each switch is a switch control named with its lever.
- A chart takes focus; its caption serves as its name, and each point speaks its value when reached.
- The answer is marked busy while its text arrives.
- A polite live region announces the new monthly saving after a lever change, not on every intermediate step.

## Responsive
- The sidebar starts folded; the thread keeps a fixed maximum width.
- Phones: metric cards go two to a row; sliders take the full width and have touch-sized handles; charts keep their height.
- Touch targets are enlarged for touch input.

## Composition hints
Metric cards; split stacked bar chart; trend or forecast chart; plan card with range sliders, switches, and summary figures; button; composer; chat list; skeleton; empty state; live region.

## Acceptance checks
1. Moving a range lever changes total savings by `monthly_base_dollars × max_saving_share × (entry_value / max_percent)` for that lever, and the forecast line recalculates.
2. Turning a switch off sets its value to 0 and removes its saving.
3. Left and right arrows on a range slider change the value by 5 percent of its maximum.
4. "Turn into rules" calls `on_ask` with a question that includes the plan snapshot.
5. Each split bar's segments add up to its total.
6. Selecting a split bar and asking calls `on_ask` with a focus range covering only that group.
7. With all levers off, the planned forecast equals the baseline forecast.
8. With reduced motion requested, no roll or path animation runs.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** single-column conversation with one answered exchange: answer text, metric cards, split bar chart, source row and follow-up chips, above a composer.

**At rest (gallery demo):**
- Question bubble on the right.
- Answer: a two-sentence text paragraph, then three metric cards in one row.
- Split chart: three provider bars, each with two or three model segments and a total label above it. Caption row beneath with the hint "click a bar to ask about it".
- Source row: a label and two source chips, with copy and feedback icons at the end.
- Follow-up chips: three in one row, below the source row.
- Composer at the foot with the approval-mode control and the mode label.
- Plan card: not shown at rest. It appears only when an answer carries a plan.
- No data table in the thread at rest. Segment values are reachable in each bar's tooltip and in its focus announcement.

**States at rest:** one answer; no pick; no plan card.

**Arrangement:** one column at a fixed maximum width. Chart spans the column. Chips sit in one row under the source row.

**Motion, settled frame:** values and bars at their final positions. Nothing rolls at rest.

**Checks (continued):**
9. Answer text and metric cards sit above the chart. The follow-up chips sit below the chart and above the composer.
10. Any table an answer includes shows every row. The thread scrolls, and no row is cut at the bottom edge of the panel.
11. The composer stays at the foot, and its approval-mode control is visible at 1280 wide and at phone width.
12. Chip text wraps inside its chip. No chip label is clipped.
