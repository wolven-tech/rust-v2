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
- Expands to its container; the page holds a comfortable reading column.
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

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** settings page: a level selector and summary bar on top, an apps list and a fixed-rules block below, and a right panel listing one app's actions by action class.

**At rest (gallery demo):**
- Level selector: three radio cards in one row (careful, balanced, hands-off), each with one sentence. Balanced is checked.
- Summary bar: segments for rules that run on their own, rules that ask, and rules that never run, with counts in words above it.
- Apps list: seven rows. Each row has a mark, an account line, a counts sentence and a small bar.
- Fixed rules block: four rules, each with a setting of Ask first or Never. The block states that the level does not change these rules.
- App panel open for the first app. Its actions sit under three headings in this order: Read only, Edits, Sensitive. Each action row shows a setting menu, weekly counts, and recent events. One action has been changed by hand, marked as changed, with a Reset control.

**States at rest:** Balanced selected; one action changed by hand; app panel open; fixed rules visible.

**Arrangement:** level cards and summary bar across the top; apps list and fixed rules in the main column; app panel on the right.

**Motion, settled frame:** the summary bar and app bars sit at their current widths. Nothing animates at rest.

**Checks (continued):**
9. At rest the apps list shows seven rows, each with its counts sentence, and the summary counts equal the sums across all rules.
10. The fixed rules block shows four rules, each with a setting, and states that the level does not change them.
11. The app panel groups its actions under Read only, Edits and Sensitive, in that order, with a count for each action.
12. The action changed by hand shows a changed marker and a Reset control.
13. At 1280 wide the apps list and the app panel are visible together, and the three level cards sit in one row.

## Amendment 6: structure and demo

Replaces, from Amendment 5 (this amendment wins where they differ):
- "Fixed rules block: four rules". The block holds seven fixed rules at rest: six set to Never, one set to Ask first.
- "App panel open for the first app ... One action has been changed by hand". The panel opens on the fifth app row (the payments app). No action is marked changed at rest.
- "Summary counts equal the sums across all rules" (check 9). The summary counts app action rules only. Fixed rules are counted separately.
- The panel group headings (check 11). A heading appears only when the group has rows.
- The changed-by-hand marker and Reset control (check 12). At rest the payments panel shows an amount limit instead.
- Checks 9, 10, 11 and 12 are superseded by 14, 15, 16 and 17. Check 13 stands, and check 18 is new.

**Archetype:** settings page. Level cards and the summary bar sit across the top; the apps list and fixed rules form the main column; a right panel lists the open app's actions by class.

**Parts visible at rest (gallery demo):**
- Level cards in one row: Careful, Balanced, Hands-off, each with one sentence. Balanced is checked.
- Summary: words above a stacked bar reading 14 on its own, 6 ask first, 6 never.
- Apps list: seven rows in this order: mail, team chat, code host, shared drive, payments, calendar, web browser. Each row has a mark, an account line, a counts sentence and a small bar. Counts (on its own, ask first, never): mail 3, 1, 1; team chat 2, 1, 1; code host 2, 1, 1; shared drive 2, 1, 1; payments 2, 0, 1; calendar 2, 1, 0; web browser 1, 1, 1.
- Fixed rules block below the apps list: seven rules, each with a setting of Never or Ask first, and a sentence stating that the level does not change them. Six read Never and one reads Ask first.
- App panel open on the payments row. Groups: Read only (one action: read charges, on its own) and Sensitive (two actions: issue credit notes, on its own up to an amount with a sentence stating it asks above that amount; change plan prices, never). No Edits group, because no action of that class is on this app.
- A Lately block at the foot of the panel lists the app's three most recent events, each with an outcome word.

**States at rest:** Balanced selected; no row marked changed; one row with an amount limit; panel open; fixed rules visible.

**Arrangement:** level cards across the top, summary bar beneath them. The apps list sits left of the panel. The fixed rules block sits beneath the apps list. The panel sits at the right.

**Motion, settled frame:** the summary bar and app bars sit at their current widths. Nothing animates at rest.

**Checks (continued):**
14. At rest the summary reads 14 on its own, 6 ask first and 6 never. Those totals equal the sums across the seven app rows. The seven fixed rules are not in those totals.
15. The fixed rules block lists seven rules, each with a setting, and states that the level does not change them.
16. The open panel lists Read only (one row) and then Sensitive (two rows). A group heading appears only when its group has rows.
17. No row is marked changed at rest. The issue credit notes row shows its amount limit and the sentence that it asks above that amount.
18. At 1280 x 720 without page scroll, the level cards in one row, the summary bar, the apps list and the open panel are visible. The fixed rules block may sit below them. It is reached by scrolling the content area and is never clipped.
