# Invoice and Credit Pair

Source item: affectUI Pro `blocks/invoice-credit` (functional reference only; see README).

## Purpose
Shows an invoice and the credit note that reverses part or all of it, as two sheets of one document. The person can switch between them, choose lines to credit, adjust the credited amount, and issue the credit note.

## Data
- `invoice`: { number, issued_on, due_on, period_from, period_to, currency, issuer: party, recipient: party, lines: Line[], tax_percent?, status: "open" | "paid" | "credited" }.
- `Line`: { id, name, quantity, unit_amount, tax_percent, total, kind: "fixed" | "metered" }.
- `party`: { name, address: string[], registration_id? }.
- `credit_document`: { number, reason?, credited: { line_id, amount }[], status: "draft" | "issued" }. Derived from the invoice when none exists.
- `active_side`, `on_side_change("invoice" | "credit")`.
- `on_credit_change(credited)`, `on_issue(credit_document)`, `on_download()`.
- `format_amount(amount, currency)`.

## Anatomy
In reading order:
1. Header row: a switch between the invoice and credit sides on the left; download and credit actions on the right.
2. Stacked sheets: the active sheet is in front; the other sits behind it with only its top edge visible.
3. Invoice sheet: total at top, status, parties side by side, terms block (numbers and dates), line table (name, quantity, rate, tax, price), totals block.
4. Credit sheet: same structure, with a selection column for lines and an amount field per credited line; a credit total at the bottom and a single issue button.

## States
- Invoice side, no credit: sheets as above; the credit action is enabled.
- Credit side, nothing picked: the credit total reads zero; issue disabled.
- Credit side, lines picked: picked rows are marked; the credit total sums credited amounts; issue enabled.
- Amount editing: the amount field is open; its value is held as a draft until Enter or blur confirms it; Escape discards the typed value.
- Amount over the line total: the field shows an inline error and the line is not counted.
- Issued: credit note shows "issued" status; its fields become read-only; the switch still works.
- Paid or credited invoice: a status label replaces the total's due text.

## Interactions
- Switch: Left and Right move between the two positions; Home and End jump to the first or last; Enter or Space picks the focused position.
- Click on a line's selection control (credit side): toggles the line in the credit.
- Space on a focused line: picks or drops it.
- Enter on an amount field: confirms the typed amount; Escape discards the typed value and restores the previous one.
- Credit action: switches to the credit side with all lines pre-picked at their full amount.
- Issue: confirms the credit; if any amount is invalid, issue is disabled and the first invalid field receives focus.
- Keyboard order: switch, download, credit action, then the lines in order, then issue.

## Motion (functional)
- Switching sheets slides the chosen sheet forward; the other tucks back with only its top edge visible.
- Selection marks draw in when a line is picked; the highlight chip glides between the switch positions.
- When a credit is issued the sheet settles to "issued" status with a short motion.
- With reduced motion preferred, sheets change position instantly and selection marks appear without drawing.

## Accessibility
- The toggle behaves as a tab list reachable through a single stop; every sheet is its panel, and the hidden one is removed from the accessibility tree.
- Buttons that send the person to the other sheet move focus to that sheet's tab.
- Credit lines are checkboxes with the line name as label; amount fields are labelled per line and state their maximum.
- The credit total is a live region.
- Issue button has a clear name ("Issue credit note") and is not reachable while disabled.

## Responsive
- Wide: party blocks side by side; full line table.
- Narrower: party blocks stack; rate and tax columns hide; credit list drops the total column.
- Phone width: single column; controls enlarge to touch size; amount fields keep their labels visible.

## Composition hints
- Tabs (switch), card (sheets), table (lines), checkbox, numeric input, button (primary and secondary), status badge, live region.

## Acceptance checks
1. Switching between the invoice and credit sides slides the sheets; the active sheet's heading changes accordingly.
2. Picking a line on the credit side adds its full amount to the credit total; dropping it removes it.
3. Typing an amount above the line total shows an inline error and excludes that line from the total.
4. Escape while editing an amount restores the previous amount.
5. The credit action pre-picks every line at full amount.
6. Issue is disabled when no line is picked or any amount is invalid.
7. After issuing, the credit note's fields are read-only and its status reads issued.
8. At phone width the parties stack and nothing scrolls sideways.

## Amendment 6: structure and demo

No earlier amendment exists, so no line is replaced.

Archetype: a switch and actions row above a stacked invoice sheet, with a parties row, terms, a line table and a totals block.

Visible at rest:
- Header row: at left, a two-position switch with "Invoice" selected and "Credit note". At right, a "PDF" download button and a "Credit invoice" secondary button with an icon.
- Stack: the invoice sheet in front. A second sheet sits behind it. Its upper rim alone shows above the front sheet.
- Sheet head: total amount, large, at left; due line beneath with a "Pay online" link; at right, the issuer chip (dot mark and issuer name).
- Parties: "From" and "To" blocks side by side, each with name and address lines.
- Terms: label and value rows for invoice number (a link), invoice date, due date, billing period.
- Line table: columns name, quantity, rate, tax, price. Two fixed lines, then a "Metered usage" group heading, then one metered line indented under the heading. Three lines in total.
- Totals block below and right of the table: Subtotal, Tax, a rule, then Total in bold.
- No status label at rest. The due text shows.

Gallery demo:
- Invoice "INV-2107"; issued 1 Oct 2026; due 31 Oct 2026; billing period 1 Sep to 30 Sep 2026.
- Issuer "Marlow Works Ltd"; recipient "Larch & Co Ltd".
- Lines: "Team plan", 1, 980.00, 20%, 980.00. "Extra storage", 4, 25.00, 20%, 100.00. Under "Metered usage": "Image renders", 9,640, 0.30, 20%, 2,892.00.
- Subtotal 3,972.00; Tax 794.40; Total 4,766.40.

Scale: the sheet covers about four fifths of the stage width, centred. The header row spans the same width above it. The total sits at the sheet's upper left.

Motion, settled frame: invoice side; second sheet's edge visible; no selection marks.
Moves and reduced motion: as stated in Motion (functional).

Defects, stated as checks: no subtotal or tax rows; two lines where the stack has three and a metered group (checks 9 and 10).

Acceptance checks (continued):
9. The totals block shows Subtotal, Tax and Total, and Total equals Subtotal plus Tax.
10. The line table shows two fixed lines and one metered line under a "Metered usage" group heading.
11. At rest the second sheet's top edge shows behind the invoice, and the switch shows Invoice selected.
