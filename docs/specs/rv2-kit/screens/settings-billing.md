# Settings: Billing

Source item: affectUI Pro `screens/settings-subscription` (functional reference only; see README).

## Purpose
A workspace administrator sees the plan the workspace is on, the next bill, and how much of each allowance is used. They compare plans and add-ons, edit the billing address and card, read past invoices, and cancel when needed.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_name | string | Shell label. |
| account | { name; avatar_url? } | Account block. |
| current_plan | { name: string; price_per_seat: string; interval: "month" \| "year"; seats_used: number; seats_total: number; renews_on: string; upcoming_amount: string; breakdown: { label; amount }[] } | The plan currently in force. |
| allowances | { label; used: number; total: number; unit: string }[] | Usage meters (seats, credits, storage, runs). |
| billing_cycle | "monthly" \| "yearly" | Price display switch. |
| plans | { id; name; monthly_per_seat: string; yearly_per_seat: string; features: string[]; current?: boolean; recommended?: boolean }[] | Plan cards. |
| add_ons | { id; name; description; tiers: { id; label; price: string; current?: boolean }[] }[] | Add-ons, each with its tiers. |
| billing_address | { company; line: string; email; vat_number?: string } | Billing details. |
| card | { brand: string; last4: string; expires: string; is_default: boolean } | Payment method. |
| invoices | { ref: string; date: string; amount: string; status: "paid" \| "upcoming" }[] | Newest first. |
| is_loading | boolean | Every block shown as placeholder shapes. |
| is_empty | boolean | New subscription: the history shows its empty state with a "preview first bill" action. |
| on_change_plan / on_add_add_on / on_edit_address / on_edit_card / on_cancel / on_preview_bill | callbacks | Action triggers. |
| on_navigate | (label: string) => void | Shell navigation; "Members" for "manage members". |

## Anatomy
Reading order:
1. Settings frame: left navigation grouped as Personal (Profile, Preferences, Notifications, Privacy) and Workspace (General, Members, Models, Security, Billing), with Billing marked current; a link back to the app.
2. Page header: the title "Billing", a one-line description, and a link to documentation.
3. Plan summary card: the plan name and price line; the upcoming bill with a "show details" toggle that reveals the breakdown; allowance meters (seats, credits, storage, runs) in a two-by-two grid.
4. Plans and add-ons section: a monthly/yearly switch; plan cards in a row (current plan marked, one recommended); a compare link; then add-on cards with their tiers.
5. Billing details card: address, email, company, VAT number, with edit controls.
6. Payment card: card brand, last four digits, expiry, a default mark, and an edit control.
7. History table: reference, total including tax, date, and status, newest row first.
8. Cancel subscription control at the foot, in a secondary style.

## States
- **Loaded.** The default.
- **Loading.** Plan, bill, allowances, details, and invoices shown as secondary shapes of their real sizes.
- **Empty history.** The invoices area shows a no-invoices message and a control that previews the first bill; the plan and allowances stay.
- **Details shown or hidden.** The bill toggle expands or collapses its breakdown; the button reflects the expanded state.
- **Yearly or monthly.** Plan prices switch; a saving note appears beside the yearly option.
- **Upgrade in progress.** The chosen plan's button shows a busy state until confirmed.
- **Cancel confirmation.** A small dialog asks once; confirming ends the flow with a status message.
- **Current plan.** Its card's action reads "Current plan" and is disabled.

## Interactions
- Show details: click or Enter toggles the breakdown.
- Monthly or yearly switch: arrow keys or a click choose.
- A plan card's action calls `on_change_plan` with the plan id.
- An add-on tier is a radio choice within its add-on.
- The manage-members link navigates to Members.
- Address and card edit controls each open a small form; Escape cancels.
- History rows: keyboard focus moves row by row; the invoice reference opens the invoice.
- Cancel subscription opens a dialog; Escape or an outside click keeps the plan.

## Motion (functional)
- Yearly and monthly prices change in place.
- The breakdown opens by sliding its rows in beneath the bill.
- Allowance bars fill from their previous value to the new one when the page loads.

## Accessibility
- Usage bars state their share as text.
- Tables use table roles; the status column is text.
- The details toggle and the compare control are buttons that report whether they are expanded.
- Cancel asks first in a dialog that traps focus; the outcome is announced in a status message.
- Each plan card names its plan and price in a single label.

## Responsive
- In narrow panels, the plan summary, plan cards, and details stack into one column; allowance meters go two by two.
- The history table keeps reference and total visible; date and status move beneath them.
- The screen fills its container.

## Composition hints
- Settings frame: sidebar navigation plus page area (see Settings: Profile for the shared frame).
- Stat or meter card for allowances.
- Segmented control for monthly or yearly.
- Plan card; add-on card with radio tiers.
- Definition-style detail rows; card with an edit control.
- Data table for invoices with status badges.
- Confirmation dialog for cancel; toast for outcomes.

## Acceptance checks
1. With `is_loading` set, every block is a secondary shape and no figure is readable.
2. With `is_empty` set, the invoices area shows its no-invoices message, and its preview control calls `on_preview_bill`.
3. Switching to yearly updates each plan's price and keeps the current plan marked.
4. Show details toggles the breakdown and its button's expanded state.
5. Clicking "Manage members" calls `on_navigate` with "Members".
6. Cancel subscription opens a dialog; Escape closes it without calling `on_cancel`.
7. Confirming cancel calls `on_cancel` once.
8. The current plan's action is disabled and reads "Current plan".
9. Each allowance meter shows used of total as text.
10. At phone width there is no horizontal page scroll, and the history table keeps its reference and total visible.
