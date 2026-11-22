# Variant Test Card

Source item: affectUI Pro `components/experiment-card` (functional reference only; see README).

## Purpose
A compact card that summarises an A/B or multi-arm test. Folded, it shows the test name, its state and the current leader. Unfolded, it sets every variant against a baseline for a chosen metric, and lets a person mark one as the winner and act on that choice. Used for experiment lists, prompt and model comparisons, and pricing trials.

## Data
- `test_name` (text).
- `arms` (Variant[]): each entry is `{ arm_id, arm_letter (single character), arm_name, readings: { [metric_id]: number }, is_control?, sample_size?, confidence?: { [metric_id]: number }, badge_tone? }`. The control is the arm flagged `is_control: true`; without one, the first arm acts as control.
- `metric_options` (Metric[]): each entry is `{ metric_id, metric_label, format?, lower_wins? }`. With a single metric there is no switch.
- `active_metric` / `starting_metric` and `on_metric_change(id)`.
- `chosen_arm` / `starting_choice` (arm id or null) and `on_choose(id | null)`. Picking the current winner again clears it.
- `status_chip` (`{ chip_text, chip_tone? }`): chip shown beneath the name. Tones: error, caution, success, info, secondary, muted.
- `status_aside` (text, optional): a brief note beside the chip, such as a day count.
- `result_action` (`{ action_text: text | (arm) => text, on_run(arm) }`): displayed on the last row once a winner is chosen.
- `empty_prompt` (node, default "Pick a winner"): text on the last row while no winner is chosen.
- `reveal` (`"hover" | "tap"`, default `"hover"`): hovering reveals the detail in hover mode; taps and keys toggle it in either mode.
- `is_open` / `starting_open` (boolean) and `on_toggle(open)`.
- `extra_class` (text, optional).

Rules:
- Hand over unformatted numbers and let each metric's `format` do the display work, which keeps animated counters fluid.
- The card does not compute statistical confidence; the caller provides it.
- With `lower_wins` set on a metric, a decrease is shown with its sign in the success tone and counts as leading.

## Anatomy
Reading order:
1. Head, a single button that expands the card: test name, status chip, status aside, a small decorative chart of the arms, and a line naming the leader.
2. Detail, revealed beneath the head:
   - Metric switch (when more than one metric exists), a segmented control.
   - One row per arm: letter badge, name, the value for the active metric, the change against control (the control row reads "Base"), and a bar sized to the value.
   - Last row: the empty prompt, or the action button once a winner is chosen.

## States
- Folded: head only. Nothing inside can take focus.
- Unfolded, no winner: rows shown; prompt on the last row.
- Arm hovered or focused: its column is emphasised in the head; the others fade.
- Metric switched: every number updates together; bars change length.
- Winner chosen: the arm's badge becomes a check; the action button takes the place of the prompt.
- Winner cleared: back to the prompt; the action is removed.

## Interactions
Pointer:
- Hover (with `reveal: "hover"`): unfolds; moving away folds. Tap or click: toggles.
- Touch tap outside: folds.
- Click a metric segment: switches metric.
- Click an arm row: chooses it as winner; click again to clear.
- Click the action: calls `on_run(winner_arm)`.

Keyboard:
- Enter or Space on the card: unfolds or folds it.
- Left and Right on the metric switch: previous or next metric, wrapping at the ends.
- Tab: metric switch, then arms, then the action.
- Space on an arm: chooses it, or clears the choice.
- Escape: folds the card and returns focus to it.

## Motion (functional)
- The head stays fixed; the detail unfolds beneath it while fading in.
- Switching metric: every number rolls together in one shared transition; bars resize smoothly.
- Pointing at an arm enlarges its slice of the head and dims the other slices.
- Choosing a winner swaps its badge for a check and reveals the action button.
- With reduced motion requested, nothing moves; states change at once.

## Accessibility
- The head is a button with expanded state; its name includes the status and the current leader.
- The metric switch and the arm list are radio groups. Each arm's accessible name carries its value and its delta from control.
- The head chart is decorative; all of its information lives in the rows.
- While folded, the content is not focusable.
- The prompt or action line is announced politely when it changes.

## Responsive
- Fills its container up to a maximum width.
- On narrow layouts the status aside is removed and the chips tighten.
- On touch, tapping opens the card, tapping outside closes it, and rows and the metric switch have larger targets.

## Composition hints
Panel, expand button, tag (status), mini chart (decorative), segmented control (metric), radio list of rows with badge, bar, delta text, primary button, animated number (rolling), live region.

## Acceptance checks
1. With a control reading 40.0 and a second arm at 41.8 on a higher-is-better metric, the second arm's row shows "+4.5%".
2. With `lower_wins` true, a 7.5 percent drop is shown with its sign and in the success tone.
3. Switching the metric updates every row's value together and moves the bars to their new lengths.
4. Space on an arm calls `on_choose(id)`; a second Space calls `on_choose(null)`.
5. With a winner set, the action button appears, and its label function receives the winning arm.
6. With `reveal: "tap"`, hovering does not open the card; a click does.
7. Escape while unfolded folds the card and returns focus to the head button.
8. With one metric, no metric switch is rendered.
9. On narrow layouts the status aside is not rendered.
10. With reduced motion requested, values update without rolling and the detail appears without fading.

## Amendment: structure and demo

Layout archetype: one card with two stacked zones. The head is a full-width row of text and a small chart; the detail is a table-like block beneath it with a segmented metric switch above the rows. The gallery renders the card unfolded so the rows and the prompt are visible at rest.

Parts that must exist:
- Head row: test name at the leading edge; then the status chip, then a note line (for example a day count) in muted text. At the trailing edge: a mini bar chart with one bar per arm, and a leader line naming the arm ahead on the active metric.
- Metric switch: a segmented control with one segment per metric, the active one marked. Omitted when there is one metric.
- Arm rows: one per arm, in arm order. Each row: letter badge at the leading edge, arm name, the value for the active metric, the change against control at the trailing side (the control row shows the word "Base" instead), and a horizontal bar whose length is proportional to the value.
- Prompt row: the last row. Shows the empty prompt while no winner is chosen; once a winner is chosen it shows the action button in place of the prompt.

Demo data for the gallery (our own names and values):
- Test name "Pricing page copy".
- Status chip "Significant" in the success tone, with a confidence note.
- Note line: "Day 9 of 14" equivalent, written in our wording.
- Three arms: A (control), B, C. Their values on the active metric make B the leader.
- Three metrics: Conversion (active), Revenue, and Refund rate (flagged lower-wins).
- No winner chosen.

States visible at rest in the gallery: unfolded card, head with chip, note and mini chart, metric switch with the first metric active, three arm rows with deltas and bars, the prompt on the last row. No action button and no hover highlight.

Acceptance checks:
11. The gallery demo shows the card unfolded at rest: head row with detail beneath it.
12. The head row shows, at the leading edge, the title, status chip and note in that order, and at the trailing edge, the mini chart followed by the leader line.
13. The metric switch shows three segments with the first one active, and includes the lower-wins metric.
14. The gallery demo shows three arm rows: A with the word "Base" as its change, B and C with signed changes, each with a bar.
15. The longest bar in the gallery demo is the bar of the highest value on the active metric.
16. At rest the last row reads the empty prompt and no action button is present.
17. The mini chart is hidden from assistive technology, and the head's accessible name includes the status and the leader (SSR test).
