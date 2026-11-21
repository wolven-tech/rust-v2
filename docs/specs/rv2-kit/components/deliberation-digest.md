# Deliberation Digest

Source item: affectUI Pro `components/reasoning-summary` (functional reference only; see README).

## Purpose
Condenses, into one compact card, how an agent spent its effort on a single answer: which kinds of work it carried out and for how long, which models took part and how hard each one reasoned, which sources it consulted, and the ordered list of steps. A person can open any model, the sources, or the steps for detail.

## Data
| Field | Type | Meaning |
|---|---|---|
| `step_list` | `Step[]` | Ordered. Each: `step_id`, `kind_key` (key into the kind table), `step_label` (past tense when finished, present tense when running), `elapsed_time?` (omitted while running), `is_running?`, `model_id?`. |
| `model_list` | `Model[]` | One per participating model. Each: `model_id`, `display_name`, `logo?`, `role_label`, `effort_level` (`"low" \| "medium" \| "high" \| "max"`), `time_taken`, `provider_name`, `token_counts?` (`{ input, output }`), `spend?`, `selection_note?`. |
| `source_list` | `Source[]` | Each: `source_name`, `source_detail?`, `source_href?`, `logo?`. Only a few need to be supplied. |
| `total_source_count` | number, optional | Total when more sources exist than are supplied. |
| `is_reasoning` | boolean | Still reasoning. The header then reads "Reasoning". |
| `expected_time` | duration, optional | Expected total. Shows the remaining part of the bar as a faded track. |
| `kind_table` | map key -> `{ kind_label, kind_emphasis, kind_mark? }` | Names, emphasis levels and marks for kinds. Some built-in kinds are provided; callers may add or override them. |
| `agent_identity` | `{ agent_id, agent_emphasis? }`, optional | Selects a small identity mark and an emphasis level. |
| `start_open` | boolean | Whether the steps list starts open. Defaults to closed. |

Malformed values are tolerated: a non-numeric time counts as zero; an unknown kind gets a neutral square and its key as its name; a model or source with no logo shows its first letter; a step with no label shows its kind name.

## Anatomy
In reading order:
1. Header: "Reasoned for <total>, <n> steps" when finished, or "Reasoning" with a pulsing mark while running.
2. Stacked bar: one segment per step, widths proportional to time, coloured by kind; a faded remainder when `expected_time` exceeds the elapsed time.
3. Kinds grid: one cell per kind present, showing its mark, name and total time; two columns.
4. Models section: one row per model with logo, name, an effort meter (four small bars), its role and its time. Clicking a row opens its detail panel beneath.
5. Model detail panel: provider, tokens in and out, spend, and the note on why it was chosen.
6. Sources toggle and list: stacked logos and a count; the list gives name, detail and link.
7. Steps toggle and list: each step with its kind mark, label and time.

## States
- **Running:** header reads "Reasoning"; the running step's segment grows with a moving highlight; the header label shimmers; the mark animates.
- **Finished:** header shows total time and step count; the bar is complete.
- **Model detail open:** only one model's detail is open at a time (or none).
- **Sources and steps lists:** each toggles independently.
- **Hover or focus on a kind, model or segment:** items that do not belong with it fade back; the related items stay clear.
- **Empty steps:** the bar is empty and the header shows the kinds list only.

Transitions: a new step appends a segment and grows in from the start side; a running step becomes finished when it receives a duration.

## Interactions
- **Pointer, kind cell:** highlights that kind's segments, the models that did the work, and the steps that belong to it.
- **Pointer, model row:** highlights its segments and steps, and opens its detail on click.
- **Pointer, segment:** shows its step label and time as a tooltip; highlights its model.
- **Pointer, sources or steps toggle:** expands or collapses the list.
- **Keyboard, Tab:** four stops in order: kinds, models, sources, steps.
- **Keyboard, arrow keys within kinds or models:** move focus; the focused item highlights its related items.
- **Keyboard, Home and End:** first and last item in the current group.
- **Keyboard, Enter or Space on a model, sources or steps toggle:** opens or closes it.
- **Keyboard, Escape:** closes whatever is open.
- **Touch:** tapping mirrors pointer behaviour; rows are larger.

## Motion (functional)
- The running segment grows continuously, with a light sheen moving along it.
- New segments enter from the left, in order.
- The mark animates while reasoning; the header label has a soft shimmer.
- When rows of kinds appear or lists open, the card grows smoothly to its new height.
- Hovering or focusing a related item fades unrelated items back.
- Reduced motion: all of this is off; items simply appear.

## Accessibility
- The stacked bar is an image whose text names the time per kind; each segment exposes its step and time on focus or hover.
- The model, source and step toggles are buttons that report expanded state; each model's name includes its role, duration and effort level.
- A polite live region announces the summary once reasoning finishes, for example "Reasoned for 11 seconds, 6 steps".
- Focus rings show for keyboard users only.
- Emphasis is never the only signal: each kind has its name and mark; effort is stated in words in the label.

## Responsive
- Fills its container's width. Kinds stay in two columns; long names truncate with an ellipsis before times do.
- Long model names, roles and step labels truncate, with the full text available on hover or focus; the role is the first thing to drop.
- At very narrow widths the model role is hidden so names keep their room.
- Numbers format with grouping separators and compact units for large values.

## Composition hints
- Card for the container.
- Stacked bar chart primitive.
- Legend grid with mark and label.
- Avatar or logo tile for models.
- Meter (four bars) for effort.
- Disclosure (expand and collapse) for detail, sources and steps.
- Tooltip for segments.
- Polite live region.

## Acceptance checks
1. A finished digest with three steps of different kinds shows a bar whose segment widths are proportional to each step's time.
2. While `is_reasoning` is true the header reads "Reasoning", and the running step's segment grows over time without a page reload.
3. Clicking a model opens exactly one detail panel showing provider, tokens and spend; clicking another replaces it.
4. Focusing a kind cell highlights only the segments, models and steps that have that kind.
5. A step with a missing or non-numeric time is counted as zero and does not break the total.
6. A source with no logo shows its first letter.
7. The sources and steps toggles can each be opened and closed with Enter and Space without affecting each other.
8. When reasoning finishes, the live region announces the total time and step count once.
9. At very narrow widths, model roles are hidden and model names remain readable.
</content>
