# Provider Usage Chart

Source item: affectUI Pro `components/model-usage` (functional reference only; see README).

## Purpose
Shows how heavily each AI provider's models were used across a period. Each provider is a stacked column with one segment per model. The reader can switch the measure (tokens, cost or requests), aim at a column to see its models and their shares, and see what the usage was spent on (search, analysis, code, tests), across all providers or for one.

## Data
- `provider_groups` (Group[]): one column each, in order. Each entry is `{ group_name, group_tone?, group_logo?, models: Model[] }`.
  - Model: `{ model_name, usage: { tokens?, cost_usd?, request_count? }, task_usage?: { [task_kind]: { tokens?, cost_usd?, request_count? } } }`.
  - Segments run with the heaviest model at the base and most emphasis; the rest take lighter variants of the group tone.
- `measure_options` (`"tokens" | "cost" | "requests"`[]) and `starting_measure`. A single option removes the switch.
- `period_label` (text, default "This week"): name of the period shown above the total.
- `prior_totals` (`{ tokens?, cost_usd?, request_count? }`, optional) and `prior_label` (text, optional): enable a change figure against the previous period.
- `task_kinds` (map of kind to `{ kind_label, kind_tone, kind_icon? }`): labels, tones and icons for task kinds, replacing the built-in set.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Summary: period label, the total for the active measure, and the change against the prior period with a direction glyph.
2. Measure switch (tokens, cost, requests).
3. Chart: one stacked column per provider, with its logo and name below. Value axis on the left.
4. Model card: appears beside the column under the pointer and lists that provider's models with their shares.
5. Task list beneath the chart: task breakdown across every model, or limited to the provider under the pointer.

## States
- Idle: the task list covers all providers.
- Provider engaged: the model card is open and the task list narrows to that provider.
- Measure changed: columns resize; segments keep their order and shading.
- Single provider: one column.
- No usage: "No usage this week" over an empty chart.
- Bad values: anything non-numeric or below zero counts as zero. A provider with no models stands empty. A provider without a logo shows its name alone. An unknown task kind gets a neutral square and its own name.

## Interactions
Pointer:
- Click a measure option: switches the measure.
- Hover or click a column: shows its card, dims the other columns, and narrows the task list.

Keyboard:
- Tab: two stops, the measure switch and then the chart.
- Left and Right: step across columns; each one's models appear in the card and its provider's tasks appear below.
- Home and End: first or last column.
- Escape: closes the card.

## Motion (functional)
- Columns rise one after another when the chart first appears.
- Changing the measure resizes columns and segments smoothly.
- Aiming at a column dims the others and brings its card up alongside it.
- With reduced motion requested, all of this is off.

## Accessibility
- Each column is a button named with its provider and total; the card is linked to the column it describes.
- The measure switch is a group of toggle buttons with pressed state.
- The task list takes its name from the figures it presents.
- Focus rings appear for keyboard focus only.
- Each model and task announces its value; shading never stands in for a number.

## Responsive
- Fills the container width. Four or five providers read best.
- On narrow layouts the switch moves beneath the total, provider names give way to logos, the task list falls into one column, and the card stretches across the chart.
- Figures are abbreviated at every size (for example 3.10B tokens, $8.6M, 640K requests). A segment too small to read shows "<1%".
- The card sits beside its column, on whichever side has more room, so it never covers the header.

## Composition hints
Panel, summary figures, segmented control (measure), stacked column chart (SVG or elements), logo and label row, popover card, task list with inline bars, live region.

## Acceptance checks
1. With `starting_measure` set, the total equals the sum of that measure across all models of all providers.
2. The change indicator points up when the total exceeds `prior_totals` and down otherwise.
3. Switching the measure resizes every segment, and segment order is unchanged.
4. Aiming at a provider column shows its models with their shares, and the task list narrows to that provider.
5. Left and Right move focus across columns and update the card and task list.
6. A negative or non-numeric usage value counts as zero and does not break the chart.
7. With one entry in `measure_options`, no measure switch is rendered.
8. With no usage at all, "No usage this week" is shown and no columns render.
9. On narrow layouts the task list falls into one column and the card stretches across the chart.
10. With reduced motion requested, columns appear without the rise animation.

## Amendment: structure and demo

**Archetype:** one stacked column per provider, with a summary header above, a measure switch, a model card that appears beside an engaged column, and a task list beneath the chart.

**Parts:**
- Summary: period label, the total for the active measure, and the change against the prior period with a direction glyph and a sign.
- Measure switch: a group of toggle buttons for tokens, cost and requests.
- Chart: one column per provider with stacked segments per model; the provider's name, logo if any, and its total below the column; a value axis on the left.
- Model card: beside the engaged column, listing its models with values and shares.
- Task list: one row per task kind, with a proportional bar and its value as text. It covers all providers at rest and narrows to the engaged provider.

**Gallery demo:** four providers, two or three models each (ten models in total). Four task kinds: search, analyse, code, test. Measure: tokens. Prior-period totals supplied, so the change reads upward.
- Instance A (at rest): no engaged column, the task list covers all providers.
- Instance B: the second provider engaged. Its card is open beside its column, the other columns are dimmed, and the task list shows that provider's tasks only.

**Acceptance checks (continued):**
11. Instance A at rest shows four columns in fixed provider order, each with its name and total below. The summary total equals the sum of every model's value for the active measure.
12. Instance A's task list shows the four task kinds sorted by value, each with a proportional bar and its value as text. Its values add up to the summary total.
13. Instance B shows the engaged column's card with that provider's models and shares summing to 100 percent, the other columns dimmed, and the task list narrowed to that provider.
14. A model whose share of its column is under one percent reads "<1%" in the card rather than a figure.
