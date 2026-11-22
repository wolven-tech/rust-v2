# Agent Autonomy Settings

Source item: affectUI Pro `screens/agent-permissions` (functional reference only; see README).

## Purpose
Lets a person decide how far an agent may act without asking: one overall level, refined per app and per action, plus a fixed set of rules the agent never crosses on its own.

## Data
- `autonomy_level`: `"careful" | "balanced" | "hands-off"`. The overall choice.
- `app_list`: list of `{ id, entry_name, account?, mark? }`.
- `action_rules`: list of `{ id, app_id, action_name, detail_text?, mark?, rule_setting: "allow" | "ask" | "never", default_setting, action_class: "read" | "write" | "risky", is_fixed: boolean, amount_limit?: { amount, currency }, week_counts: { done, asked, blocked }, recent_events: [{ what, outcome: "done" | "approved" | "declined" | "blocked", occurred_at }] }`.
- `action_class` drives the presets. `is_fixed` marks a fixed rule.
- `initial_app_id`: string or null.
- `text_strings`, `account`, `workspace_name`, `is_loading`, `is_empty`.
- Callbacks: `on_change(rule)`, `on_preset(setting_level)`, `on_select(app_id)`, `on_connect()`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Introduction and level selector: a radio group of three cards, each with a plain sentence describing what the agent does unprompted at that level.
3. Summary bar: a thin stacked bar showing rules set to run on their own, rules that ask, and rules that never run, with the counts in words above it.
4. Apps list: one row per app, with its mark, account, a short count sentence, and a small bar.
5. Fixed rules list: each fixed rule has a setting of either Ask first or Never. These rows ignore the level selector.
6. App panel (right): actions grouped under three headings, Reads, Changes, and Needs care. Each action row shows its name, a setting menu, weekly counts, an optional amount limit for money-related actions, and its recent events. A Reset control returns a row to its default.

## States
- **level selected**: the radio group shows the chosen level; every non-fixed rule matches it, except rules changed by hand.
- **changed by hand**: a rule whose setting differs from what the level implies is marked as changed. Choosing the current level again clears those manual changes.
- **fixed rule** (`is_fixed`): the setting menu offers only Ask first and Never. The row explains why it is fixed.
- **limit set**: the row shows the amount; actions above it ask, and actions below it run.
- **loading** (`is_loading`): placeholder rows for apps, fixed rules, and the open app.
- **empty** (`is_empty`): no apps connected. The apps list shows a note and a Connect app button; fixed rules are hidden; the panel shows nothing.

## Preset meanings (functional)
- Careful: read actions run on their own; anything that changes data asks first.
- Balanced: actions run on their own unless they need care; those ask first.
- Hands-off: everything runs on its own except the fixed rules, which keep their fixed setting.
Each preset sets every non-fixed rule to the setting it implies, and calls `on_preset`.

## Interactions
- Arrow keys in the level radio group move the selection; the group is a single tab stop.
- Tab and Enter: every app, fixed rule, and action setting is a button. Enter on an app opens it in the panel. Enter on an action setting opens its menu.
- Menu: arrow keys move, Enter picks, Escape closes and returns focus.
- Limit field: Enter saves and calls `on_change`; Escape restores the previous amount.
- Reset on a row returns it to `default_setting`.
- Connect app calls `on_connect`, or navigates to Integrations.

## Motion (functional)
- Choosing a level moves the summary bar's segments to their new sizes, and the counts above it update.
- Each app's small bar follows the same change.
- Reduced motion: values change without transitions.

## Accessibility
- The level selector is a radio group; the chosen level is checked.
- Each app is one button named with a sentence of its counts, for example "Mailroom, account: 3 on its own, 1 asks, 1 never".
- Each action's setting control is a menu button; its accessible name carries the action and the current setting. A fixed rule's menu leaves out "On its own" and explains why.
- A polite live region announces changes, for example "Issue refunds: Ask first".
- Meaning is never carried by a mark alone; every bar segment has its words next to it.

## Responsive
- Fills its container; the page keeps a readable column width.
- Below the phone breakpoint: the three level cards stack; app rows drop the labels beside their small bars; the app panel opens as a sheet.
- Long names and accounts truncate with a tooltip; a limit field grows with its amount.
- Touch targets are enlarged for touch input.

## Composition hints
Radio card group; stacked summary bar; list rows with app mark and badge; popover menu button; number input; section headings; side panel or sheet; empty state; live region.

## Acceptance checks
1. Selecting Careful sets every read action to run on its own and every write action to ask first.
2. Changing one action by hand marks it as changed; selecting the current level again clears that change.
3. A fixed rule's menu never offers "On its own".
4. Entering a limit and pressing Enter saves it and calls `on_change`; Escape restores the previous amount.
5. The summary counts equal the number of rules at each setting.
6. With `is_empty` set, the apps list shows the connect action and the fixed rules are hidden.
7. Pressing an arrow key in the level group changes the selected level and fires `on_preset`.
8. With reduced motion requested, the summary bar changes without a transition.
