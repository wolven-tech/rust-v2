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
