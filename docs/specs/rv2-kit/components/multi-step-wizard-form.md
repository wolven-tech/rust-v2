# Multi-step Wizard Form

Source item: affectUI Pro `components/stepper-form` (functional reference only; see README).

## Purpose
Divides a setup or onboarding form into a sequence of stages. Entries persist between stages, so navigating backward keeps earlier input. Every stage is checked before progression, and the last stage presents a summary before creation. On completion, a confirmation panel takes the place of the stages.

## Data
| Field | Type | Meaning |
|---|---|---|
| `step_list` | `Step<V>[]` | Each: `step_id`, `step_title`, `step_heading?`, `step_description?`, `render_step(step_context)`, `check_step?(form_values)`. `check_step` returns messages keyed by field name. |
| `step_context` (render argument) | object | `form_values`, `set_form_values`, `field_errors`, `go_to(index)`, `go_next()`. |
| `initial_values` | V | Form values shared by all steps. |
| `on_complete` | (form_values) => void or Promise | Invoked from the final stage. A returned promise holds the busy caption until settlement; an exception appears next to the action button. |
| `initial_index` / `current_index` / `on_index_change` | number | Starting step, or a controlled step index. |
| `final_label` | string | Label on the last step. Defaults to "Finish". |
| `busy_label` | string | Label while `on_complete` runs. Defaults to "Creating…". |
| `render_success` | (form_values, reset) => ReactNode | Replaces the steps when done. A success helper provides an animated check, a heading and a line. |

Field building blocks: a labelled field wrapper (name, label, hint, optional marker, message slot); a text input with an optional fixed prefix; a chip input for lists (emails by default), with add and remove by key; a choice group rendered as selectable cards with an optional badge; a review summary with optional edit links that jump to a step.

## Anatomy
In reading order:
1. Step indicator: each step with its number, label and a progress segment; the current step is marked; finished steps are controls that jump back.
2. Step heading and description.
3. Step content: the fields for this step.
4. Action row: Back (when not on the first step) and Continue (or the final label on the last step).

## States
- **Step active, valid:** Continue is active.
- **Step active, invalid:** Continue is de-emphasised but still responds; pressing it shows messages and focuses the first invalid field.
- **Field error:** the field shows its message and shakes once.
- **Submitting:** the action shows the busy label and is busy.
- **Submit failed:** the error is shown next to the action; the form stays on the same step with values kept.
- **Success:** steps are replaced by the success view.
- **Narrow:** the step labels collapse into a compact "Step n of N" line above the progress segments.
- **Review step:** rows summarise every value, with edit links.

Transitions: Continue moves forward when valid; Back moves back without validation; a finished step's control moves back to that step; completing the last step moves to submitting, then to success.

## Interactions
- **Pointer:** type in fields; press Continue or Back; press a finished step to return to it; press a review edit link.
- **Keyboard, Enter:** continues the step. In a chip field it adds the typed entry first; with an empty chip field it continues.
- **Keyboard, Tab:** moves through the fields, then Back, then Continue.
- **Keyboard, comma or Backspace in a chip field:** comma adds the entry; Backspace in an empty field removes the last chip.
- **Keyboard, left and right arrows in a choice group:** move and select; Enter confirms and continues.
- **Touch:** same as pointer; larger fields and buttons.

## Motion (functional)
- Each step's progress segment fills as the person reaches it.
- A finished step's number becomes a check that draws itself.
- The incoming stage glides in from the side of travel as the outgoing one exits opposite.
- Just the stage body resizes; the controls row remains on screen.
- A rejected field jolts once and reveals its explanation.
- Chips appear and disappear with small transitions.
- The selected choice card's highlight moves between cards.
- Reduced motion: all transitions are instant.

## Accessibility
- Steps form an ordered list with the current step marked; finished steps are buttons that go back.
- On narrow screens the list stays in the accessibility tree while the compact line is shown on screen.
- Focus moves to the new step's first field, or to its heading. After a failed Continue, focus moves to the first invalid field.
- Fields mark their invalid state and link their message by description.
- A polite live region announces the step change, for example "Step 2 of 4: Team", along with any errors.
- Continue stays operable while de-emphasised, so people using assistive technology hear what is missing.

## Responsive
- Fills its container up to a comfortable form width and works down to small phones.
- When the container is small, the step labels collapse, choice cards stack, and review rows wrap.
- Touch: larger fields (no zoom on iOS, so the font size meets the platform threshold) and larger buttons; the action row stays at the bottom on long pages.

## Composition hints
- Card for the container.
- Ordered list with progress segments for the stepper.
- Text input, tag input, radio cards, and summary list primitives.
- Field wrapper with message.
- Primary and secondary buttons.
- Success state with an animated check.
- Polite live region.

## Acceptance checks
1. Pressing Continue on a step whose check fails shows the messages, does not change step, and focuses the first invalid field.
2. Values entered on step 1 are still present after going forward to step 3 and back.
3. Pressing a finished step's control returns to that step with its values intact.
4. Enter inside a chip field commits the typed value; a second Enter on the empty field advances.
5. `on_complete` runs once on the last step; while its promise is pending the action shows the busy label; a rejection is shown beside the action and the step stays.
6. The success view replaces the steps after completion, and its reset returns to the first step with the initial values.
7. The step announcement reaches the live region on each step change.
8. Arrow keys in a choice group change the selected card and do not change the step.
9. At narrow width the compact "Step n of N" line is visible, and the step list is not visible but remains in the accessibility tree.

## Amendment: structure and demo

Layout archetype: one card in a single column, with a step indicator across the top, then the step body, then an action row. The step indicator spans the full card width; the body is a vertical stack of heading, description and fields; the action row sits at the bottom.

Parts that must exist:
- Step indicator: one column per step. Each column holds a thin progress segment on top and, beneath it, the step number and step label. The current step's label is marked; finished steps read as controls that jump back.
- Step heading and description: heading in the largest body style of the card; description as one line beneath it.
- Field stack: labelled fields in a single column. A prefixed input shows its fixed prefix text inside the field, before the entry. A field with a hint shows the hint line beneath the field.
- Action row: a Back control at the leading edge, hidden on the first step; the Continue control at the trailing edge. On the last step the trailing control reads the final label and Back stays.
- Review step (for the last step): summary rows with label and value, each with an edit link when a step is given.

Demo data for the gallery (our own names and values):
- Four steps: "Workspace", "Team", "Plan", "Review".
- Step 1 active. Its heading asks who the workspace is for; its description is one line of our own wording.
- Two fields: a workspace name text input, and a workspace address input with a fixed prefix of our demo host, followed by a hint line.
- Progress: the first segment filled, the rest empty.

States visible at rest in the gallery: step indicator with step 1 current and the others upcoming, heading, description, both fields, hint line under the address field, Continue active at the trailing edge, no Back control. No field error, no busy caption.

Acceptance checks:
10. The gallery demo shows the step indicator, heading and description, field stack and action row, stacked in that order inside one card.
11. The step indicator shows four steps, each with its number and label; the first label is marked current and only the first progress segment is filled.
12. Step 1 shows the name input and the prefixed address input; the prefix text appears before the entry, and the hint line appears beneath the address input.
13. At rest on step 1 no Back control is present and Continue sits at the trailing edge of the action row.
14. At rest no field error is shown and Continue is not de-emphasised.
15. At narrow width the "Step 1 of 4" line appears above the segments and the labelled list is out of view but in the accessibility tree (SSR test).
16. On the last step (SSR test with the index of the last step) the trailing control reads the final label and the review rows appear, with an edit link on each row that has a step.
</content>
