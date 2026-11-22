# Quota Usage Panel

Source item: affectUI Pro `components/usage-meter` (functional reference only; see README).

## Purpose
A usage and billing panel for a product that meters credits, tokens, minutes or messages within a billing cycle. It shows consumption against the plan, what that consumption was spent on, limits such as seats and storage, and a daily trend. As usage approaches the cap, it offers a top-up or upgrade action.

## Data
| Field | Type | Meaning |
|---|---|---|
| `plan_total` | number | Credits included in the plan. Sets the full width of the main meter |
| `spend_breakdown` | `{ spend_label, spend_value, spend_emphasis? }[]` | Where credits went during the selected period. Parts without emphasis levels receive graded shades of the primary emphasis, with a neutral shade last |
| `cap_list` | `{ cap_label, cap_used, cap_total?: number or null, unit?, decimals? }[]` | Slim meters beneath the main one, such as seats and storage. A null total marks an open-ended allowance |
| `period_choices` | `{ period_id, period_label }[]` | Choices for the period switch. Leave it out and the switch disappears |
| `shown_period` | string | The period currently displayed, owned by the host. Supply the matching usage, caps and daily series alongside it |
| `first_period` | string | Period shown first when the host does not supply `shown_period`. Falls back to the first choice |
| `on_period_pick` | (period_id) => void | Fired when a period is chosen |
| `resets_at` | Date, string or number | When credits refresh. Rendered as a countdown, with the calendar date revealed on hover |
| `daily_trend` | `{ day_label, day_value: number or null }[]` | Usage per day. A null stands for a day not yet reached and draws a flat stub |
| `on_action` | (state) => void | Invoked by the action button with `"ok"`, `"warn"` or `"limit"` |
| `panel_heading` | string | Card title, for example the workspace or product name |
| `plan_label` | string | Plan name shown ahead of the refresh date |
| `meter_unit` | string | Unit name on the main meter. Defaults to "credits" |
| `alert_ratio` | number 0..1 | Ratio at which the meter and caps begin to warn. Defaults to 0.85 |
| `headroom_action` | string | Button text while room remains |
| `ceiling_action` | string | Button text close to and at the cap |

## Anatomy
Reading order:
1. Header: title, plan name and refresh countdown.
2. Period switch, when provided.
3. Main meter: the consumed part of the plan total, a percentage tag, and the consumed figure stated alongside its total.
4. Spend breakdown: segments of the main meter, each with a name, value and share, plus a legend row per category.
5. Cap meters: one row per cap with its consumed figure and total, or an open-ended marker.
6. Daily sparkline, with each day's value revealed on focus.
7. Action button: a standard action while there is headroom, a prominent top-up action near or at the cap.

## States
- **Under the alert ratio:** neutral meter; standard action.
- **Approaching the cap (warning):** the percentage tag changes emphasis, a small indicator appears, the action becomes prominent, and the text names the condition.
- **At the cap:** the percentage tag takes the strongest emphasis; the text says the cap is reached; the action turns into a top-up action.
- **Category gone:** its segment shrinks to nothing and its legend row folds away.
- **Category hovered:** other categories dim and its legend row lights up.
- **Period switched:** values roll to their new numbers and segments glide to new widths.
- **No data:** the meter reads zero; the sparkline shows flat stubs.

Transitions: the first view fills the meter from zero while the numbers count up. A period switch refreshes everything in one step.

## Interactions
- **Pointer, category segment or legend row:** reveals its value and highlights it. On touch, a tap reveals the value.
- **Pointer, period choice:** switches the period.
- **Pointer, action button:** invokes `on_action` with the current state.
- **Keyboard, Tab:** visits the period switch, the main meter, the sparkline and the action button.
- **Keyboard, Left and Right:** on the period switch, selects the previous or next period; on the meter, steps through categories; on the sparkline, steps through days.
- **Keyboard, Escape:** hides the category or day readout.

## Motion (functional)
- On first view the meter fills from zero, with the numbers counting up in step.
- Switching periods slides the light fill to its new value, rolls the numbers and glides the parts to new widths.
- A vanishing category collapses to nothing, its legend row closes up, and the card settles to its new height.
- A part under the pointer leaves its neighbours dimmed, and the readout slides along the meter.
- Near the cap, the share tag changes, an indicator appears, and the action button shifts and resizes to its new label.
- With reduced motion requested, everything switches instantly.

## Accessibility
- The panel is a labelled section. The main meter and each cap are meters whose values are spoken as text, for example "3,015 of 5,000 credits".
- The period control is a radio set that occupies a single tab stop.
- Moving between meter segments updates a readout that screen readers announce without interrupting.
- Running low and reaching the cap are announced politely; emphasis is never the sole signal, since the text repeats it.

## Responsive
- Fills its container up to a moderate width. On narrow widths the switch takes its own row and caps stack as rows.
- Works down to a small phone width. Long labels truncate with an ellipsis; numbers and units never break apart.
- On touch, tapping a part or legend row reveals its value, and targets are enlarged.

## Composition hints
- Card for the container.
- Segmented meter with legend.
- Slim meter rows for caps.
- Line or bar sparkline for the daily trend.
- Radio group for the period.
- Button for the action, in two states.
- Polite live region.

## Acceptance checks
1. With `plan_total` 5,000 and spend values summing to 3,015, the main meter reads "3,015 of 5,000" and the category shares add up to the consumed share.
2. A cap whose `cap_total` is null shows an open-ended marker and no fill percentage.
3. When the consumed share reaches `alert_ratio`, the percentage tag takes the warning treatment and the text names the condition.
4. At the full plan total the action becomes a top-up action and `on_action` receives `"limit"`.
5. Pressing Right on the meter moves focus to the next category and its readout is announced.
6. Changing the period calls `on_period_pick` and updates the displayed values without reloading the page.
7. A null entry in `daily_trend` draws a flat stub, not a zero-height bar.
8. At narrow widths the period switch takes its own row and cap rows stack.

## Amendment 3: structure and demo

**Replaces:** none. This spec had no earlier amendment, so checks 1 to 8 stand. At rest the consumed share is under the alert ratio, so the warning treatment of check 3 does not show.

**Archetype:** one usage card. A period switch with two choices sits at the top. Within the card: one main meter with a segment per category and a legend row per category; below it a daily strip; then a footer that carries the cap meters and the action row.

**Parts, top to bottom:**
- Header: panel title and plan name at the left; refresh countdown at the right.
- Period switch: two choices.
- Main meter: consumed figure with total and unit, a percentage tag, and one bar with a segment per category.
- Category rows: one per category, all visible at rest, each with its mark, name, value and share.
- Daily strip: one cell per day of the period. Days not yet reached are flat stubs.
- Cap row: slim meters for members and archive, and an open-ended marker for a cap with no total.
- Action row: the remaining figure and the action button.

**Gallery demo (our data):** title "Studio usage", plan "Studio plan", countdown "Refreshes in 8d 23h". Periods: "Cycle to date" (selected) and "Previous cycle". Unit "jobs". Plan total 500; consumed 420, shown as 84 percent.
- Categories: Batch extraction 150 (36 percent), Live answers 120 (29 percent), Index rebuilds 90 (21 percent), Exports 60 (14 percent).
- Daily strip: 14 cells; the last three are not yet reached.
- Caps: Members 7 of 12; Archive 2.4 of 8 GB; Shared spaces at 31 with no total.
- Action row: "80 jobs left" and the button "Raise limit".

**Acceptance checks (continued):**
9. At rest the main meter reads "420 of 500 jobs". Four segments and four category rows appear in the order Batch extraction, Live answers, Index rebuilds, Exports; the values sum to 420 and the shares to 100.
10. At rest the cap row holds two slim meters, Members 7 of 12 and Archive 2.4 of 8 GB, and an open-ended marker for Shared spaces with no fill and no percentage.
11. At rest the percentage tag shows the neutral treatment, no warning text appears, and "Raise limit" is the standard action.
12. At rest the daily strip has 14 cells. The last three are flat stubs, and no bar is drawn at zero height.
13. At rest the period switch shows two choices with Cycle to date selected. No category readout and no day readout is open.
