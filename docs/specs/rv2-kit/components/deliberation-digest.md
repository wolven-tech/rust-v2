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

## Amendment: structure and demo

**Archetype.** A compact summary card with three zones stacked top to bottom: a header over one stacked bar, a two-column legend of kinds, and a list of models. Below them, two disclosure rows: Sources, collapsed, and Steps, open. It is not a timeline and not a table.

**Parts that must exist:**
- Header: a mark, a label ("Reasoning" while running; total time and step count when finished), and a live indicator while running.
- Stacked bar: one segment per step, plus a remainder segment when an expected total is set beyond the elapsed time.
- Kinds legend: one entry per kind present, each with mark, name and total time.
- Model rows: name, role, an effort meter of four bars, and time.
- Model detail slot beneath the model rows, empty at rest.
- Sources disclosure: label with the total count, and a list of name, detail and link when expanded.
- Steps disclosure: each row with kind mark, label, kind name and time.

**Gallery demo data:** six steps across five kinds, one kind used twice; four models with four different effort levels; nine sources in total, of which three are supplied with logos.

**States visible at rest in the gallery:**
- Running: header reads "Reasoning" with a live indicator; the last step has no time figure yet and is followed by a remainder segment.
- Steps list open, Sources list collapsed, model detail slot empty.

**Acceptance checks, continued:**
10. Gallery at rest: the header reads "Reasoning" with a live indicator; the running step has no time figure and is followed by the remainder segment.
11. Gallery at rest: the Steps list is open with six rows, each showing its kind name and time; the Sources disclosure is collapsed and its label shows the total of nine.
12. Gallery at rest: four model rows, each with a role and an effort meter of four bars; no model detail is open.
13. The kinds legend lists five entries in two columns, in order of first appearance among the steps.
14. Tab from the header of the card visits the four stops in order: kinds, models, sources, steps; a stop with no content is skipped.

## Amendment 3: structure and demo

**Replaces** checks 10 to 13: the header label at rest, the open steps list, the four-row model list and the five-entry legend. Check 14 (tab order) stands. This amendment supersedes the gallery demo and at-rest paragraph of the earlier amendment. Opening the sources list, the steps list and the model detail remain capabilities.

**Archetype.** A compact summary at rest: one header, one stacked bar, a legend row, three model lines, and two closed disclosure rows. It is not a timeline and not a table.

**Parts, top to bottom:**
- Header: a mark, a label, and a live indicator while work continues. The label reads "Working" until the run finishes, then "Took <total> across <n> steps".
- Stacked bar: one segment per step, plus a remainder segment when an expected total exceeds the elapsed time.
- Kinds legend: one entry per kind present, each with its mark, name and total time. At desktop width all entries sit in one row; narrow widths wrap them.
- Model lines: one line per model: mark, name, role, effort meter of four bars, time. Role truncates first.
- Model detail slot: beneath the model lines, empty at rest.
- References disclosure: a closed row labelled "References" with the total count, and stacked logos for the supplied sources when expanded.
- Step log disclosure: a closed row labelled "Step log" with the step count.

**Gallery demo (our data):** six steps across three kinds, `Lookup`, `Compare` and `Verify`, in order:
1. Lookup, "Pulled the pricing table", 4 seconds.
2. Lookup, "Opened the release notes", 3 seconds.
3. Compare, "Matched tiers to usage", 2 seconds.
4. Compare, "Ranked the two options", 2 seconds.
5. Verify, "Checked totals against the invoice", 1 second.
6. Verify, running, "Checking the final figures", no time yet.

Expected total 14 seconds. Three models: Quill (role Retrieval, effort high, 4 seconds), Harbor (role Critique, effort low, 1 second), Lumen (role Drafting, effort max, 3 seconds). Nine sources in total; three supplied with logos.

**Acceptance checks (continued):**
15. At rest the header reads "Working" with a live indicator. The running step has no time figure and is followed by the remainder segment.
16. At rest the kinds legend holds three entries in one row, in order of first appearance: Lookup 7 seconds, Compare 4 seconds, Verify 1 second. The running step adds to no total.
17. At rest there are three model lines, each with a role and an effort meter, at three different effort levels. No model detail is open.
18. At rest References and Step log are both closed, labelled with 9 and 6. Opening Step log lists six rows, each with its kind name and time, the running row with none. The two disclosures toggle independently with Enter and Space.
19. When the running step receives its time, the header takes the finished form and the live indicator is removed.
</content>
