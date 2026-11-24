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

## Amendment: structure and demo

Layout archetype: a single card made of stacked bands: heading, a phase row of three inputs, a line table, an add-control row, and a summary row of three figures. The table is the only band with more than one row of repeated controls.

Parts that must exist:
- Heading band: heading text and the phase name field under a "Phase name" label.
- Phase row: start date control (with a calendar glyph), length control, and beside the length control a derived "Ends <date>" label. Name, start and length sit on one row at desktop width.
- Line table, four columns with headers: Name, Recurrence, Timing, Amount. An unlabelled trailing column holds a Remove action per line, labelled for screen readers as "Remove <line name>".
- Product line: name field, recurrence select, timing select (charged up front or at period end), amount field with the currency prefix.
- Discount line: name field, a dash in the recurrence and timing cells, amount field with a unit select beneath it (percent or fixed).
- Minimum line: name field, a dash in the recurrence and timing cells, amount field.
- Add row: one product picker listing catalogue items (placeholder "Include item"), then two buttons, "Include adjustment" and "Include floor", in one row.
- Summary row: three figures in this order: initial charge, phase end date, phase sum.

Demo data for the gallery (our own names and values):
- Phase name "Onboarding phase"; start date in the first month of the year; length of twelve months.
- Three lines: two products ("Platform seat" monthly, billed at period end; "Onboarding" once, billed up front) and one discount of ten percent.
- Summary computed from those lines, with the phase end date derived from the start and length.
- Product picker holds no selection at rest.

States visible at rest in the gallery: all three lines in the table, the derived end date beside the length, the summary figures, the add row with an empty picker. No menu, calendar, invalid message or focused field is open.

Acceptance checks:
11. At desktop width the gallery demo shows name, start and length on one row, with the "Ends <date>" label to the right of the length control.
12. The table header shows four column heads in the order Name, Recurrence, Timing, Amount, and each line row has a Remove action with a label naming its line.
13. The discount line shows a unit select beneath its amount and dashes in its recurrence and timing cells (SSR test).
14. The gallery demo table holds exactly three line rows: two products and one discount.
15. The add row holds one product picker and two buttons on one row.
16. The summary row shows initial charge, phase end date and phase sum, in that order, left to right.
17. The phase end date uses the same date format as the start date.
18. At rest the gallery shows no open menu, no open calendar and no invalid message.

## Amendment 3: structure and demo

**Replaces** checks 11, 13, 14 and 15 of amendment 1. At rest the gallery shows one line, no inputs and no add row; their content now applies to the edit state. Check 12 applies in the edit state. Checks 16 to 18 stand unchanged. This amendment supersedes the earlier amendment's gallery demo and its at-rest paragraph.

**Archetype:** a read-only contract card at rest. Editing is a state the card enters on request, and that state is the builder from the earlier amendments.

**Parts at rest, top to bottom:**
- Heading: the heading text and the phase name, both as text.
- Schedule line: start date and length as text, with "Ends <date>" beside the length.
- Lines table: a header row with four labels (Name, Recurrence, Timing, Amount) and one row per line, all as text.
- Action band: one control, "Edit phase".
- Summary band: three figures.

**Parts in the edit state:** the amendment-1 controls. The heading gains the name field, the schedule line gains its controls, each line gains its inputs and a Remove action, and the add row appears. "Stop editing" returns to the rest state with the edited values kept.

**Gallery demo (our data):** heading "Contract phase"; phase name "Rollout phase"; start 1 March 2026; length twelve months, so the end date is 28 February 2027.
- Rest state: one line, "Kickoff workshop", recurrence once, charged up front, amount 300. Summary: initial charge 300, phase end 28 February 2027, phase sum 300.
- Edit state, opened in the gallery for review: three lines, "Kickoff workshop" (once, up front, 300), "Platform seat" (monthly, charged at period end, 120), and a discount of 10 percent. Summary: initial charge 300, phase sum 378. The product picker holds no selection.

**Acceptance checks (continued):**
19. At rest the gallery contains no input, select, picker or menu. Its only control is Edit phase, and every value is text.
20. At rest the lines table has four column headers in the order Name, Recurrence, Timing, Amount, and exactly one line row, "Kickoff workshop".
21. At rest the schedule line shows the start date and the length as text, with "Ends 28 February 2027" beside the length.
22. Edit phase reveals the amendment-1 controls: each line has its inputs and a Remove action labelled with its name; the discount line shows its unit select beneath its amount; the add row holds one product picker and two buttons on one row. The edit state holds three line rows: two products and one discount.
23. The summary's phase sum is 300 at rest and 378 in the edit state.
24. Stop editing returns to the rest state with the edited values kept. on_schedule_change is called once per edit, not when editing stops.

## Amendment 4: structure and demo

**Replaces:** amendment 3 in full. Its read-only rest state, its "Edit phase" control, its separate edit state and its "Stop editing" return no longer exist; the controls they hid are visible at rest. Checks 19 to 24 of amendment 3 are replaced. Of amendment 1, checks 12, 16 and 17 are replaced; checks 11, 13, 14 and 15 stay superseded as amendment 3 left them. Base check 3 is replaced. Base checks 1, 2 and 4 to 10, and amendment 1 check 18, stand.

**Archetype:** a single editable card at rest. Stacked bands: heading, a summary strip, a schedule row of two fields, a line table, an add row, and a summary band of two figures.

**Parts at rest, top to bottom:**
- Heading band: the heading text, as text.
- Summary strip: the phase name as an editable field labelled "Phase name", with no visible border at rest; then a separator, the start date as text, an arrow glyph, and the length as text. The start and length text mirror the schedule row below.
- Schedule row: two labelled fields side by side at desktop width. "Start date" is a date control with a calendar glyph. "Length" is a select-style control showing its value.
- Line table: a header with three heads, in the order Name, Recurrence, Amount. Each line is one row. Its name is text. Its recurrence is a select. Its amount cell holds the currency prefix, the amount field, and a suffix naming the recurrence; beneath the amount, right-aligned in the same cell, sits a timing select (charged up front or at period end) with no head of its own. An unlabelled trailing action removes the line, labelled "Remove <line name>"; it is hidden until the row is hovered or focused.
- Add row: a product trigger reading "Include item" at the start, which opens a catalogue listbox and is not a text field; then the buttons "Include adjustment" and "Include floor" at the end. All on one row at desktop width.
- Summary band: two figures, "First invoice" then "Phase total", left to right. First invoice is the sum of valid product lines billed up front, before any discount. Phase total follows base check 2. No phase end date is shown.

**Gallery demo (our data):** heading "Contract phase"; phase name "Rollout phase"; start 1 March 2026; length "Twelve months". One line: "Kickoff workshop", recurrence once, timing charged up front, amount 300. Summary: first invoice 300, phase total 300. The product trigger holds no selection. No discount or minimum line is present.

**At rest:** every control shows its value. No menu, calendar, invalid message or focused field is open.

**Acceptance checks (continued):**
25. Top to bottom the card shows the heading, the summary strip, the schedule row, the line table, the add row and the summary band, in that order.
26. The summary strip shows the phase name in an editable field, and the start and length as text matching the schedule row's values.
27. At desktop width the Start date and Length fields sit side by side, and the start field shows a calendar glyph.
28. The line table header shows three heads in the order Name, Recurrence, Amount. The timing select sits beneath the amount inside the Amount cell, and no column head labels it.
29. The gallery table holds exactly one line row, "Kickoff workshop", with recurrence once, timing charged up front and amount 300. Its Remove action is labelled "Remove Kickoff workshop" and is not visible until the row is hovered or focused.
30. The add row holds the product trigger "Include item" at its start and the buttons "Include adjustment" and "Include floor" at its end, on one row at desktop width. None is open at rest.
31. The summary band shows "First invoice" then "Phase total", left to right, with 300 and 300 in the gallery. No end-date figure appears at rest.
32. Changing the length updates the length text in the summary strip and the Length field together, and leaves the start date unchanged.
</content>
