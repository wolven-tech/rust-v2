# Agent Spend Overview

Source item: affectUI Pro `screens/agent-usage` (functional reference only; see README).

## Purpose
Shows what agents cost in the current month: the running total against a budget, where it is heading at the current pace, where the money went by project and by model, and which items deserve a look.

## Data
- `currency_code`: ISO 4217 code.
- `daily_costs`: list of numbers, one per elapsed day of the month; the last entry is today.
- `days_in_month`: number.
- `budget_amount`: number or null. Null means no budget line and no budget sentence.
- `project_breakdown` and `model_breakdown`: lists of `{ id, entry_name, amount, change_vs_last_month?, tone_token?, daily_costs?, top_tasks?: [{ entry_title, run_count, cost }] }`.
- `attention_items`: list of `{ id, item_kind: "spike" | "jump" | "budget", text, detail_text?, badge? }`.
- `usage_stats`: optional `{ task_count?, run_count?, token_count? }`. With `task_count`, cost per task is shown.
- `is_loading`, `is_empty`.
- Callbacks: `on_select(entry_kind, id)`, `on_open_item(item)` (default: navigate to Runs).

## Anatomy
1. Sidebar (shared app shell).
2. Header: month name.
3. Hero: total spent so far, budget fraction, and one plain sentence stating where the month will land at the current pace, and on which day the budget would be passed if it would be.
4. Cumulative chart: running total by day, a budget line when set, and a dashed forecast continuing from today.
5. Stats row: tasks, runs, tokens, and cost per task when tasks are given.
6. Two ranked lists, Projects and Models: name, amount, share of total, and change against last month with its sign. Each row has a share bar.
7. Panel (right) for the selected project or model: daily bars for the month and the top-cost tasks.
8. "Worth a look" list: attention items, each with a short text, detail and an Open button.

## States
- **under budget**: sentence says where the month lands and when, if at all, it passes the budget.
- **over budget**: sentence states the budget was passed and on which day.
- **no budget** (`budget_amount` null): no budget line; the sentence gives the forecast only.
- **no spend yet** (`is_empty`): empty note in the hero; lists hidden.
- **panel open**: a project or model is selected; its daily bars and top tasks show.
- **loading** (`is_loading`): placeholder hero, chart, rows and items.

## Interactions
- Click a project or model row, or press Enter on it: opens its panel.
- Up and down arrows move through rows in a list.
- Open on an attention item: calls `on_open_item`; default navigates to Runs.
- Hover or keyboard focus on the chart shows the day's total in a tooltip.

## Motion (functional)
- The total counts up as runs finish.
- The forecast line extends from today to month end.
- Reduced motion: values set directly.

## Accessibility
- The chart has a text summary; each day's value is available as text on focus.
- The sentence about the forecast is a normal paragraph.
- Changes carry their sign and words ("up" or "down" with the percentage), not mark alone.
- Lists are sorted by amount, descending, and announced that way.

## Responsive
- Cards stack on narrow screens; the chart keeps its height.
- Breakdown lists take the full width.
- The panel becomes a sheet on narrow screens.
- Touch targets are enlarged on touch input.

## Composition hints
- Hero figure with a progress bar; area chart with budget line and forecast; stat tiles; ranked lists with share bars; side panel or sheet; callout list with action buttons; badges; empty state; skeleton.

## Acceptance checks
1. The hero total equals the sum of `daily_costs`.
2. The forecast line starts at the last actual day and extends at the average daily pace of the days so far.
3. When the pace implies the budget is passed, the sentence names the day it is passed.
4. With `budget_amount` null, no budget line or budget sentence appears.
5. Project and model lists are sorted by amount, descending.
6. Shares in each list sum to 100 percent within rounding.
7. Amounts format with `currency_code`.
8. The Open action on an attention item calls `on_open_item` with that item.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** month overview: a hero with the pace sentence over a cumulative chart and a stats row, two ranked lists, a worth-a-look list, and a right panel that is closed at rest.

**At rest (gallery demo):**
- Header: month name.
- Hero: total spent so far, a budget fraction bar, and one sentence on where the month lands at the current pace. The month is under budget at rest, so the sentence names no passed day.
- Cumulative chart: one point per elapsed day (23 days), a flat budget line, and a dashed forecast from today to month end.
- Stats row: four tiles: tasks, runs, tokens, cost per task.
- Two ranked lists side by side: Projects (4 rows) and Models (4 rows). Each row shows name, amount, share, and change against last month with its sign in words.
- Worth a look: three items (one spike, one jump, one budget), each with text, detail and an Open button.

**States at rest:** under budget; no panel open; no empty state.

**Arrangement:** hero and chart across the top; stats row; the two ranked lists; the worth-a-look list at the foot of the page area.

**Motion, settled frame:** the total is at its final value and the forecast line is drawn in full. Nothing animates at rest.

**Checks (continued):**
9. The cumulative chart draws one point per elapsed day, a flat budget line, and a dashed forecast from today to month end.
10. The stats row shows four tiles, the fourth being cost per task.
11. Projects and Models each list four rows sorted by amount, descending, and each row shows its change with a sign word.
12. The worth-a-look list shows three items at rest, each with its text, its detail and an Open button.
13. Chart axis labels are values from the data. No label repeats or reads blank, and no label reads "No data" when the chart has values.
14. The page area scrolls so the last card is fully reachable. No card is cut at the bottom edge of the page area.

## Amendment 6: structure and demo

Replaces, from Amendment 5 (this amendment wins where they differ):
- "A right panel that is closed at rest". The panel is open at rest on the first project row.
- "Two ranked lists side by side". One list shows at a time, behind a Projects / Models switch. Projects is selected at rest.
- The stats row as a separate row. The four stats sit inside the hero card, under the chart, in the order tasks, runs, cost per task, tokens.
- "One point per elapsed day (23 days)". The demo has 19 elapsed days, so the chart has 19 points and the forecast runs from day 19 to the last day of the month.
- Checks 9, 10 and 11 are superseded by 15, 16 and 18. Checks 12 to 14 stand. Checks 17, 19 and 20 are new.

**Archetype:** month overview. A hero card holds the pace sentence, the cumulative chart and four stats. Below it, a ranked card with a Projects / Models switch and a worth-a-look card. A right panel is open on the first project.

**Parts visible at rest (gallery demo):**
- Header: month name.
- Hero card: total spent so far (270.40) against a budget of 500, with a fraction bar at 54 percent, and one sentence stating that the month is on pace for 427 by its last day and is under budget.
- Cumulative chart inside the hero card: 19 points, one per elapsed day. A flat budget line. A dashed forecast from the last point to the last day of the month. Axis labels at day 1, day 8, day 15 and the last day of the month.
- Stats inside the hero card, under the chart: tasks (186), runs (1,284), cost per task (1.45), tokens (48.2M).
- Ranked card: a segmented switch, Projects and Models, above one list. Projects is selected, with four rows in this order: Harbor (118.40), Dune (82.10), Fern (46.30), Internal (23.60). Each row shows name, amount, share with a bar, and change against last month with its sign word.
- Worth-a-look card: three items, one spike, one jump and one budget item. Each has a title line, a detail line, a badge and an Open button.
- Right panel open on Harbor: its amount and share of the month, its change against last month, daily bars for the 19 days, and its top tasks (four rows with run count and cost).
- One currency code throughout the page.

**States at rest:** under budget; project panel open on the first project; Projects selected.

**Arrangement:** the hero card spans the top of the content column. The ranked card and the worth-a-look card sit below it. The panel stands beside them at the right.

**Motion, settled frame:** the total is at its final value and the forecast is drawn in full. Nothing animates at rest.

**Defects to fix in ours:**
- The frame clips below the chart and the stats tiles are cut.
- Ours shows both lists at once and the panel closed. At rest the page shows one list and an open panel.

**Checks (continued):**
15. The chart has 19 points, a flat budget line and a dashed forecast from the 19th day to the last day of the month. The 19 daily values sum to the hero total.
16. The stats row sits inside the hero card, under the chart, in the order tasks, runs, cost per task, tokens.
17. At rest the panel is open on the first project and shows its daily bars for the month and its top tasks.
18. The ranked card shows one list at a time behind a Projects / Models switch. At rest Projects is selected, with four rows sorted by amount, descending, each with a change and its sign word.
19. The worth-a-look card shows three items at rest (one spike, one jump, one budget), each with its text, detail and an Open button.
20. At 1280 x 720 without page scroll, the header, hero card, chart and stats row are visible. The ranked card and the worth-a-look card are reached by scrolling the content column, and neither is cut at the end of its scroll.

## Amendment 7: structure and demo

Replaces: Amendment 6 "Hero card" fraction bar (the budget share is text, no bar); Amendment 6 "Cumulative chart" axis labels (five labels, below); Amendment 6 check 20 (superseded by checks 21 to 25). Amendment 6 checks 15 to 19 stand.

Archetype: a month overview in two columns: a main column (hero card, then a ranked card above a worth-a-look card) and a right panel open on the first project.

Positions are fractions of the content area's width (W) and height (H), host sidebar excluded.

Main column, 5% to 63% of W:
- Header "Usage" with the month, at 2% to 7% of H.
- Hero card, 9% to 49% of H. Inside:
  - "Spent in <month>" label at 11% of H.
  - Total, large, at 15% to 17% of H, followed on the same line by "of <budget>" and the share as text, muted. No progress bar.
  - Pace sentence at 18% of H.
  - Chart at 21% to 40% of H, about 45% of the hero card's height. Y scale: zero at the plot base, the budget at the plot top. A flat budget line at the top, its label at its right end, muted. The spent line runs from day 1 to day 19 with a light area fill beneath and a dot on day 19. A dashed forecast runs from day 19 to the last day. X labels at 39% of H: day 1, day 8, day 15, day 23, last day. No y-axis labels.
  - Divider at 41% of H. Stats row at 43% to 47% of H: four stats in the order Tasks, Runs, Cost per task, Tokens, each a small label above a larger value, in four equal columns of the hero's inner width.
- Ranked card, 56% to 76% of H, 5% to 63% of W. Heading "Where it went" at 53% of H on the left. The Projects / Models switch sits at 51% to 62% of W on the heading row. Four rows at 58.6%, 63.4%, 68.2% and 73.1% of H. Per row: a marker square at 6% of W; name at 8%; share bar from 21% to 41% of W; amount right-aligned ending at 50% of W; share percentage at 51% to 54%; change at 56% to 59% with its sign word (up or down) and percentage; chevron at 61%.
- Worth-a-look card, 81% to 100% of H. Heading at 80% of H. Three items: the first at 84% of H, the second at 90%, the third cut by the bottom edge. Each item: a round icon, a title line, a detail line, a badge and an Open button.

Right panel, 68% to 99% of W, 0% to 62% of H:
- Tab row at 3.5% of H: the open project's name chip with a close control.
- Name line at 9.5% of H with its colour dot; amount and share at 14%; change with its sign word at 16%.
- "Day by day" label at 21%. Nineteen daily bars between 25% and 31% of H across the panel's inner width. Axis caption at 33%.
- "What cost the most" label at 37%. Top-tasks card, 38% to 62% of H, with four rows at 41%, 46%, 51% and 56% of H: title, run count, cost right-aligned.

Gallery values: spent 270.40 of 500 (54%); pace sentence states the month lands at 427 on its last day and is under budget; Projects rows Harbor 118.40, Dune 82.10, Fern 46.30, Internal 23.60; panel open on Harbor.

Settled frame: total at its final value, forecast drawn in full. Nothing animates at rest.

Acceptance checks (continued; these replace Amendment 6 check 20):
21. At 1280 x 720 the hero card (total, budget share, pace sentence, chart, stats row) sits fully inside the viewport without scrolling the content column.
22. All four Projects rows are visible at rest. The Projects / Models switch sits on the ranked card's heading row, with Projects selected.
23. The worth-a-look heading and its first two items are visible at rest. The third item is cut by the screen edge and reached by scrolling the content column.
24. The right panel is visible at rest beside the lists: its name, amount and share, change line, nineteen daily bars and four top-task rows.
25. The chart has x labels at days 1, 8, 15, 23 and the last day, and no y-axis labels. The hero shows the budget share as text, with no progress bar.
