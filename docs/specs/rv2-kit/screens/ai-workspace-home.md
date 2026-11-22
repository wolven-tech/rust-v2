# AI Workspace Home

Source item: affectUI Pro `screens/assistant-overview` (functional reference only; see README).

## Purpose
The landing page for someone who returns to the AI workspace every day. A composer at the top starts new work. Below it sit the person's own figures: activity over time, recent chats, plan usage, files the assistant produced, and this month compared with last month.

## Data
- `activity_days`: list of `{ date, chat_count }` for the displayed period (for example, the last nine months).
- `recent_chat_list`: list of `{ id, entry_title, updated_at }`, plus `hidden_chat_count`.
- `usage_meter`: `{ used_units, unit_limit, resets_on }`.
- `file_list`: list of `{ entry_name, extension, created_at }`, plus `hidden_file_count`.
- `month_stats`: `{ chat_stat: { entry_value, change_vs_last }, files_made: { entry_value, change_vs_last } }`.
- `workspace_name`, `account`, `is_loading`, `is_empty`.
- Callbacks: `on_send(text)`, `on_navigate(entry_label)` (for the "more" links, to Chat or Folders), `on_open_chat(id)`, `on_open_file(entry_name)`.

## Anatomy
1. Sidebar (shared app frame).
2. Composer at the top of the page: a text field, an attach control, and quick shortcuts ("Work in a folder", "Connect apps").
3. Board beneath the composer, four cards in a grid:
   - Activity: a grid of small squares, one per day, arranged in weeks. The total and date range appear above it.
   - Recent chats: up to three rows, each with its time, followed by an "N more" link.
   - Usage: a meter showing used and total units, plus the reset date.
   - Files made: rows with name, extension, and time, followed by "N more".
   - This month: two figures, chats and files made, each with its change against last month stated in words.

## States
- **populated**: every card is filled.
- **loading** (`is_loading`): placeholder shapes laid out like each card's content.
- **empty** (`is_empty`, new workspace): the board is replaced by a single empty state reading "Nothing here yet", a line describing what will appear, and a Start a chat button that focuses the composer.
- **over usage limit**: the meter is full, with words saying the limit has been reached.
- **positive or negative change**: the change text carries its sign and its comparison ("up 12 on last month").

## Interactions
- Tab order runs from the composer to the activity grid, then to each card's "N more" link and its rows.
- Arrow keys in the activity grid: left and right move a week back or forward; up and down move a day; Home and End go to the first and last day.
- Focusing or hovering a day square shows a tip with its date and count. Escape hides the tip.
- Clicking a row opens the chat or file. "N more" opens the full list.

## Motion (functional)
- Cards fade in when their data arrives.
- Reduced motion: content appears directly.

## Accessibility
- Cards expose themselves as sections whose names come from their headings.
- The activity grid is a single graphic labelled with its total; keyboard focus on a day announces its date and count.
- The usage meter exposes its value as text as well as a visual.
- Change figures state what they compare against, in words.
- A colour or mark is never the only signal for a change or a limit.

## Responsive
- Four columns on wide screens; two on narrower ones; one on phones.
- The activity grid always fills the width of its card.
- Touch targets are enlarged for touch input.

## Composition hints
Prompt composer; activity grid with tip; list rows; meter; stat tiles with change text; file row with type label; empty state; skeleton.

## Acceptance checks
1. The activity grid's squares sum to the displayed total.
2. Arrow keys move focus day by day and show the tip with its date and count.
3. A usage value above the limit is stated in words.
4. The empty state's button focuses the composer.
5. Recent chats show at most three rows, followed by the "N more" link.
6. Long file names truncate in the middle while their extension stays visible.
7. Each change figure names its comparison period in words.
