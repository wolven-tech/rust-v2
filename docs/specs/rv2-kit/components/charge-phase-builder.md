# Charge Phase Builder

Source item: affectUI Pro `components/billing-schedule` (functional reference only; see README).

## Purpose
Lets someone put together one billing phase of a contract: a start date, a length, the products billed during it (each with its own repeat rule and timing), an optional discount, an optional minimum commitment, and running totals.

## Data
| Field | Type | Meaning |
|---|---|---|
| `heading_text` | string | Title at the top of the card |
| `phase_label` | string | Editable name of the phase |
| `currency_symbol` | string | Prefix for prices and totals |
| `item_options` | CatalogItem[] | Products offered by "Include item": `{ option_name, recurrence, unit_amount }` |
| `starting_lines` | Line[] | Lines present on first render: products, a discount, or a minimum |
| `starting_date` | string | Start date, `yyyy-mm-dd` |
| `on_schedule_change(schedule)` | callback | Called after every change with `{ phase_label, start, duration, lines }` |
| `root_class` | string | Extra class applied to the root |

Line kinds: `product` (name, recurrence, timing, amount, quantity), `discount` (percent or fixed amount), `minimum` (amount).

Recurrence options: once, monthly, quarterly, yearly. Timing options: charged up front, charged at period end.

## Anatomy
Reading order:
1. Heading and phase label.
2. Phase row: start date control and length control.
3. Line table with column headers: Name, Recurrence, Amount. Each line has its own inputs.
4. Add controls: Include item, Include adjustment, Include floor.
5. Summary: initial charge, end date of the phase (derived from start and length), phase sum.

## States
- **No lines:** the table shows an empty hint; the summary shows zero totals.
- **Editing an amount:** the field is focused; Enter confirms; Escape reverts.
- **Menu open** (length, recurrence, timing, product picker): options listed; the highlighted option follows the keyboard.
- **Calendar open** (start date): a day grid for the month; the focused day follows the keyboard.
- **Invalid value:** the field shows an inline message; the total is computed only from valid values.
- **Discount larger than the subtotal:** the total floors at zero and the field shows a message.

## Interactions
- Pointer: every input and menu is clickable; the product picker adds a line at default values.
- Tab: phase label, start, length, each line's fields, then the add controls.
- Enter: confirms an amount, opens a menu or calendar, or picks the highlighted option or day.
- Up/Down in a menu: previous or next option; Home/End: first or last.
- Arrow keys in the calendar: day back or forward (left/right), week up or down (up/down). Home/End: first or last day of the week. Page Up/Down: previous or next month; with Shift, previous or next year.
- Escape: dismisses an open menu or calendar and hands focus back to the control that opened it.

## Motion (functional)
- A freshly added line slides in briefly; the grid resizes smoothly around it.
- The end date in the summary changes with a brief fade when the length or start changes.
- Menus scale in from their button.
- Reduced motion: all changes are immediate.

## Accessibility
- Line items are presented as a grid with labelled column heads.
- Every field and menu trigger carries a label that names its row and shows its present value.
- Dates in the calendar are announced with full day and month names, consistent across locales.
- Totals are announced politely after a change.

## Responsive
- Fills its container up to a readable width.
- When the container is narrow: fields stack vertically; the recurrence column hides and its value moves beneath the product name.
- No horizontal scrolling at phone width.

## Composition hints
Card, form fields (text, number, select), listbox (menus), date picker (calendar grid), table (lines), button, summary list, live region helper.

## Acceptance checks
1. Adding a product line from the item options fills its name, recurrence and amount from the option.
2. The summary's phase sum equals the sum of valid product lines, minus the discount, raised to the minimum if one is set.
3. Changing the length updates the end date in the summary.
4. In the calendar, the Page Down key moves the focused day forward by one month and keeps the same day of month where it exists.
5. Escape in an open menu closes it and focus returns to its trigger button.
6. Entering a non-numeric amount shows an inline message and excludes that line from the total.
7. `on_schedule_change` is called after each edit with `{ phase_label, start, duration, lines }`.
8. A discount larger than the subtotal does not produce a negative total.
9. Every line row is a table row with headers for Name and Amount.
10. At phone width no horizontal scrolling occurs and the amount is still visible.
</content>
