# Reversible Change Log

Source item: affectUI Pro `components/agent-ledger` (functional reference only; see README).

## Purpose
Lists every change an agent made in external systems, newest first. A person can revert a change together with the changes built on it, where the system allows, or reapply them. The component records intent; the host performs the external revert.

## Data
- `change_rows`: list of `{ row_key: string (unique), verb_label: string, target_label?: string, system_label: string, system_mark?: icon element, happened_at_ms: number, value_change?: { from_text: string, to_text: string }, summary_line?: string, reversible: boolean (default true), built_on_keys?: string[] }`. Use `value_change` for a before and after pair, `summary_line` otherwise.
- `actor_label?`: string. Who made the changes; shown beside the heading.
- `clock_ms?`: number. Reference for relative times. Defaults to real time, refreshed.
- `reverted_keys?` and `default_reverted_keys?`: string[]. Controlled and initial sets of reverted rows.
- `on_revert(keys)`, `on_reapply(keys)`: receive the selected row key plus its reversible dependents, in cascade order.
- `spotlight_key?`: string. Lights a row and its dependents as if its revert control were pointed at. For walkthroughs.
- `heading`: default "Change log".
- `class_name?`.

## Anatomy
1. Header: heading, change count, reverted count, actor label.
2. One row per change, newest first: system mark and label, verb and target, the before and after text or the summary line, relative time, a revert or reapply control, and flags.
3. Rows keep a fixed height. Time, controls and flags share one slot.

## States
- `applied` (default): revert control shown. If the row has dependents, the control's accessible name states how many.
- `reverted`: text struck through, control becomes reapply, system mark muted.
- `not reversible`: no revert control. Flag "Cannot revert".
- `blocked`: the row depends on a reverted row that cannot be reverted. Flag "Cannot revert", control disabled.
- `also reverted`: a dependent reverted by a cascade. Flag "Also reverted".
- Empty list: text "No changes yet".

## Interactions
- Pointer hover or keyboard focus on a revert control: highlights the row and its dependents; the highlight clears on leave or blur.
- Activate revert: calls `on_revert` with the row key and its reversible dependents. Without the handler the component updates the states itself.
- Activate reapply: calls `on_reapply` with the same keys.
- Tab: moves through revert and reapply controls in row order.
- Enter or Space on a focused control: acts on that row.

## Motion (functional)
- New rows enter at the top; the list grows smoothly to make room.
- A revert runs a cascade down the affected rows, one after another, and those rows quiet down.
- Spotlight highlights the affected rows at once.
- Reduced motion: every change appears at once.

## Accessibility
- The list is ordered, labelled by the heading, newest first.
- Revert and reapply controls name the change and the dependent count, for example "Revert: <verb> <target>, 2 dependents".
- Before and after text is read as "from X to Y".
- Flags are text, not colour alone.

## Responsive
- Container-query based.
- Narrow container: numbers move to their own line, the time drops the system name, controls shrink to compact icon buttons.
- Touch input: revert and reapply controls always visible, in place of the time.

## Composition hints
List and list items, text and icon buttons, flag badges, relative-time element, icon element, container query.

## Acceptance checks
1. Reverting a row with reversible dependents calls the handler once with all keys in cascade order.
2. A row marked not reversible shows no revert control and shows "Cannot revert".
3. A row built on a reverted non-reversible row shows "Cannot revert" and has no enabled revert.
4. Reapply restores the struck text and reverts the row state.
5. A row with a new key appears at the top and existing rows keep their positions.
6. Focusing a revert control highlights its dependents; blurring clears the highlight.
7. Row height stays constant across states.
8. In a narrow container the numbers wrap to their own line and every control stays reachable.
9. Unverified in demo: the cascade animation and the dependent highlight. Headless activation of revert on two rows changed the row states as expected, and no page errors were logged.
