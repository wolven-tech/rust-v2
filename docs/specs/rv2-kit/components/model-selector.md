# Model Selector

Source item: affectUI Pro `components/model-picker` (functional reference only; see README).

## Purpose
The model menu of an AI chat composer. A person chooses a model, sees each model's speed, quality and cost at a glance, and sets options that travel with the model: extended thinking with its effort level, and web search. The menu shows what a choice costs before the message is sent.

## Data
- `model_options` (Option[]): each `{ option_id, option_name, option_note?, option_icon?, speed_rank?, quality_rank?, cost_rank? (1 to 5, where cost 5 is most expensive), first_token_time?, price_per_million? }`. Leave all three ranks out for a router-style option; its row then reads "Varies".
- `selected_model_id` (controlled) / `initial_model_id` (defaults to the first option) and `on_model_change(id)`.
- `thinking_on` (controlled) / `initial_thinking` (default false) and `on_thinking_change(on)`.
- `web_search_on` (controlled) / `initial_web_search` (default false) and `on_web_search_change(on)`.
- `thinking_effort` (controlled) / `initial_effort` (default "medium") and `on_effort_change(effort)`. Values: `"low" | "medium" | "high"`. Shown only while thinking is on.
- `is_open` (controlled) / `initial_open` (default false) and `on_open_change(open)`.
- `menu_align` (`"start" | "end"`, default "start").
- `menu_side` (`"bottom" | "top"`, default "bottom"): choose top when the composer sits at the viewport's lower edge.
- `extra_class` (text, optional).

## Anatomy
Trigger: a button showing the current model name, with a thinking indicator when thinking is on.

Menu, opened from the trigger:
1. Model list: each row has an icon, name, a short note, three bars (speed, quality, cost) and, on the lit row, first-token time and price.
2. Divider.
3. Settings: an Extended thinking switch; when it is on, an effort choice (Low, Medium, High) unfolds beneath it. A Web search switch.
4. Footer tray: first-token time and price for the highlighted model, updating as the highlight moves.

## States
- Closed: trigger only.
- Open: list shown; the selected model is marked.
- Highlighted model: its bars fill; the footer shows its figures.
- Thinking on: effort choice visible; trigger shows the thinking indicator; the effort adjusts the first-token estimate (multiplied by a factor per level).
- Thinking off: effort choice hidden.
- Router option (no ranks): bars replaced with "Varies".

## Interactions
Pointer:
- Click the trigger: opens or closes the menu.
- Hover a model row: highlights it and fills its bars.
- Click a model row: selects it and closes the menu.
- Click the thinking switch: toggles it; effort unfolds or folds.
- Click an effort choice: sets the effort.
- Click the web search switch: toggles it.

Keyboard:
- Enter, Space or Down on the trigger: expands the menu and places focus in the list.
- Up / Down / Home / End: move between models.
- Enter or Space in the list: chooses the focused model and dismisses the menu.
- Tab leaving the list moves through thinking, effort, then web search.
- Left / Right in effort: Low, Medium, High.
- Escape: closes the menu and returns focus to the trigger.

## Motion (functional)
- Opening and closing combine a brief fade with a gentle scale anchored to the trigger's corner.
- Hovering a model fills its gauges in sequence while sibling rows recede.
- Effort unfolds beneath the thinking switch; its selection indicator slides to the chosen level.
- Latency and price figures tick to new values; the trigger label crossfades; a small marker appears beside it while thinking is active.
- Reduced motion: no movement.

## Accessibility
- The trigger announces its popup state and linked panel; its name reports the model and the thinking state.
- Models live in a listbox; focus remains there while arrow keys shift the active choice.
- Thinking and web search are on/off switches; effort is a one-stop radio set.
- While closed, the menu is inert. The footer figures are announced once, not while they change.

## Responsive
- Width is constant but capped below the viewport minus a margin. At the narrowest width, row icons and key hints are dropped and the gauges tighten.
- Touch: larger trigger, rows, switches and effort buttons; key hints hidden.

## Composition hints
Popover menu, trigger button, listbox with options, rating bars (segmented), switch, radio group, footer tray, indicator icon, live region.

## Acceptance checks
1. Opening the menu with the trigger and pressing Down moves focus to the first model; Enter selects it, calls `on_model_change`, and closes the menu.
2. Hovering a model fills its three bars in proportion to their ranks; the footer shows its first-token time and price.
3. A router option with no ranks shows "Varies" and no bars.
4. Turning thinking on reveals the effort choice with the initial effort selected; turning it off hides it.
5. Pressing Right in the effort group moves the selection from Low to Medium and calls `on_effort_change`.
6. Escape closes the menu and focus returns to the trigger.
7. With `menu_side: "top"`, the menu opens above the trigger.
8. The trigger name includes the model name and reflects thinking on or off.
9. When very narrow, the row icons and key hints are removed.
10. Reduced motion: opening and bar fill appear without animation.
</content>
