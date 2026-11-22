# Editable Step Plan

Source item: affectUI Pro `components/agent-plan` (functional reference only; see README).

## Purpose
Lets a person review an agent's proposed ordered steps before anything runs: rename, remove, reorder or add steps. Starting then runs the final list and shows each step's progress. A failed step can be retried from that point.

## Data
- `plan_heading`: string. Heading, and the label of the step list.
- `plan_steps`: list of `{ step_key: string (unique, stable), step_title: string, step_detail?: string, tool_label?: string (shown as a chip), step_state: "pending" | "in_progress" | "succeeded" | "errored", measured_ms?: number, error_text?: string }`. When `measured_ms` is absent, duration is measured from the moment the step becomes `in_progress`.
- `initial_steps?`: same shape. Used when the host does not control the list; edits are kept inside the component.
- `plan_phase`: `drafting` | `executing` | `completed`. Editing is possible only while `drafting`.
- `estimate_text?`: string shown in the review footer.
- `on_edit(steps)`: fires on every review change (rename, remove, reorder, add).
- `on_start(steps)`: fires with the steps in final order. The host moves the phase to `executing`.
- `on_cancel()`: optional; shows a cancel control beside start.
- `on_retry_from(step_key)`: optional; shows a retry control under each errored step.
- `on_repeat()`: optional; shows a repeat control after completion.
- `class_name?`.

## Anatomy
1. Heading: plan heading and, while executing, the progress count.
2. Ordered list: index marker, step title, detail text, tool chip, drag grip and remove control. A text field replaces the title while renaming.
3. Add-step field below the list, shown only while `drafting`.
4. Footer. Drafting: estimate text, cancel, start. Executing: progress line with "k of n" and elapsed time. Completed: total time and repeat.

## States
- `drafting`: every step editable. Index markers show numbers.
- `executing`: add-step field hidden. Index marker becomes a spinner on the in-progress step, a check on succeeded steps, an error mark on errored steps. The in-progress step carries current-step state. Progress line fills per finished step.
- `completed`: every marker shows a check. Footer shows total time and repeat.
- `errored` step: retry control under it. Retrying resets that step and every later step to `pending` and keeps the plan executing.
- Empty plan: text saying there are no steps, with the add field focused.

## Interactions
- Tab: moves between steps, their remove controls, the add field and footer actions.
- Up or Down: shifts focus one step up or down the list.
- Alt with Up or Down: relocates the focused step one place.
- Enter on a focused step: starts renaming. Enter again saves. Escape cancels a rename without calling `on_edit`.
- Submitting the add field appends the typed text as a new last step and leaves the cursor in the field.
- Delete: removes the focused step.
- Pointer drag on the grip: reorders steps.
- Start calls `on_start` with the steps in their current order.

## Motion (functional)
- While a step is dragged it tracks the pointer and its neighbours yield space. Keyboard moves and externally supplied order changes use the same easing path.
- Removed steps collapse and close the gap. Added steps fade in.
- Index markers change from number to spinner to check. The progress line fills per step.
- The add field hides when the run starts; the footer swaps to run status.
- Reduced motion: every change is instant.

## Accessibility
- The step list is an ordered list whose name is the plan heading.
- Each step is focusable while drafting and its description lists its shortcuts.
- Every reorder, rename, removal and addition is spoken through a polite live region that reports the step's new place in the list.
- During execution, every row's state is readable as text, the active row is exposed as the current item, and a row finishing or failing is announced.
- The grip responds only to pointer drag; Alt with the arrow keys is the keyboard path to the same move.
- Icon buttons carry labels. Focus rings appear on keyboard focus only.

## Responsive
- Takes the width its container offers, up to a readable maximum. Long titles and details wrap onto new lines; tool chips never break.
- When the container is narrow, the footer hint is hidden and gutters tighten. Usable at the narrowest phone width.
- Touch input: grip and remove control always visible, at a comfortable touch target size.

## Composition hints
Card, ordered list, text input, icon buttons, chip (badge), progress line, live region, footer button group.

## Acceptance checks
1. Removing a step in review calls `on_edit` with the remaining steps in order.
2. Alt with Down moves the focused step and announces its new position.
3. Enter starts a rename; Enter saves; Escape cancels without calling `on_edit`.
4. In the add field, Enter appends a step at the end and the field keeps focus.
5. `on_start` receives the steps in their final order, not the original proposal order.
6. While `executing`, the add field and rename and remove controls are not available.
7. A retry resets the failed step and later steps to `pending` and keeps the plan executing.
8. Works at the narrowest phone width with no horizontal page scroll.
9. Unverified in demo: retry and completion states. Headless removal of a step updated the list, and start moved the plan into the executing state with a count and elapsed time. Completion view not inspected.
