# Overnight Run Digest

Source item: affectUI Pro `components/agent-recap` (functional reference only; see README).

## Purpose
The first screen a person sees on returning to agents that kept working. It puts what needs a decision first, then what failed and can be retried, then what finished, with a timeline of activity since they left.

## Data
- `digest_events`: list of `{ event_key: string (unique), happened_at: timestamp, event_lane: "awaiting_decision" | "errored" | "completed", event_title: string, event_detail?: string, event_mark?: icon element, repeat_count?: number, main_action?: { action_label: string }, inspect_available?: boolean, past_tense_label?: string, reversible: boolean (default false) }`. `main_action` is required for `awaiting_decision` events. `past_tense_label` is shown once the main action succeeds. `repeat_count` merges many small events into one card and also raises its point on the chart.
- `away_since`: timestamp. When the person last left.
- `now_at?`: timestamp. Right edge of the timeline. Defaults to render time.
- `cards_per_lane`: number, default 3.
- `replay_on_open`: boolean, default true. Plays the intro replay when the section opens; false shows everything at once.
- `section_heading`: default "Since you last checked".
- `display_locale?`: string, for times and counts.
- `on_act(event, which: "main" | "inspect")`: returns a promise. The card shows progress until it settles. A rejection returns the card to its prior state.
- `on_revert(event)`, `on_reapply(event)`: for `completed` events that are reversible, and for events that were acted on.
- `class_name?`.

## Anatomy
1. Section with heading.
2. Timeline chart: a small line chart with one point per event and a filled area below. Runs from `away_since` to `now_at`.
3. Three lanes side by side: Decisions, Errors, Completed. Each lane has a heading with the count of all its events, including those beyond the card limit.
4. Cards: up to `cards_per_lane` per lane. Each shows a state tag, title, detail, time, and an action row.
5. An empty lane shows text saying so.

## States
- `awaiting_decision`: action row shows the main action and inspect.
- Working: while the main action's promise is pending, its control shows progress and the other controls are disabled.
- Acted: card shows the past-tense label and a revert control. Focus moves to revert.
- Rejected action: card returns to `awaiting_decision` and an error message is announced.
- `errored`: retry as the main action.
- `completed`: revert if reversible.
- Reverted: card shows reapply; reapply restores the prior state.

## Interactions
- Tab: card by card. Inspect, then main action (or revert or reapply).
- Enter or Space: runs the focused action. After a successful main action, focus moves to revert. After a revert, focus returns to the main action.
- Pointer or focus on a card: highlights its point on the chart.
- Pointer over the chart: draws a guide line that snaps to the nearest event, enlarges its point, highlights its card, and shows a tooltip with its title.

## Motion (functional)
- On open, a trace sweeps across the chart. Points appear as the sweep passes them, their cards fade up, and totals tick upward.
- Pointing at the chart: guide line, enlarged point, card highlight.
- Reduced motion: everything appears at once.

## Accessibility
- The region is labelled by its heading. Each lane heading carries its full count.
- Each button names its action and target, for example "<Action>: <event title>".
- Completion of an action is announced in a polite live region, and focus moves to revert.
- The chart is decorative and hidden from assistive technology. Its information is in the cards.
- Timestamps carry a machine-readable date. A lane with nothing in it states that in text.

## Responsive
- Cards in a lane are the same height; footers align across each lane.
- When the container is narrow, the lanes stack vertically.

## Composition hints
Section, card, list, buttons, state badge, line chart (custom), time element, tooltip, live region, count display.

## Acceptance checks
1. Each lane shows at most `cards_per_lane` cards and a count equal to all of its events.
2. Events with unreadable times are omitted; events outside the window sit at the chart edges.
3. While a main action's promise is pending the card shows working state; a rejection returns it to its prior state.
4. After a successful main action, focus moves to revert.
5. Pointing at the chart snaps to the nearest event and highlights its card.
6. Reduced motion: no intro animation.
7. Card footers align across a lane.
8. In a narrow container the lanes stack.
9. Unverified in demo: the intro replay and the chart-to-card highlight. Headless main-action and revert clicks ran without page errors; visual states after them were not inspected.
