# Plan Comparison Section

Source item: affectUI Pro `blocks/pricing-plans` (functional reference only; see README).

## Purpose
Lets a visitor compare the available plans side by side, switch between paying monthly and yearly, and open each plan's details to see what it includes.

## Data
- `heading`, `intro`.
- `billing_options`: list of { id: "monthly" | "yearly", label }; `selected_billing`, `default_billing`, `on_billing_change(id)`.
- `yearly_saving_label`: text shown beside the yearly option (e.g. a percentage saving).
- `plans`: list of { id, name, price_monthly, price_yearly, billing_note, cta_label, cta_href, highlighted: boolean, badge?: string, limits: { label, value }[], integrations: { label, sync_label, max_selectable, logos: node[] }, features: { label, included: boolean }[] }.
- `initial_details_open`: boolean; `on_details_toggle(plan_id, open)`.
- `extra_classes`.

## Anatomy
In reading order:
1. Heading and intro.
2. Billing switch: monthly and yearly, with the saving label on yearly.
3. Plan cards in a row: name, optional badge, price with its period, billing note, primary call to action, a short limits list, then a collapsible details area.
4. Details area per card: integrations with their sync note and a selection count, then the feature list with included and not-included rows.

## States
- Default: monthly selected; details collapsed on every card.
- Yearly: prices change to the yearly figure; billing note changes to the yearly-billed wording.
- Highlighted plan: its card shows the badge and the primary call to action in its emphasised form.
- Details open: the card grows; the toggle label reads hide details.
- Not-included feature: its row is shown with a de-emphasised marker and text, plus the words not included.
- Loading prices: price slots show placeholder blocks.

## Interactions
- Click the billing option: switches monthly and yearly; the price figures change in place.
- Click a plan's details toggle: opens or closes that card's details only.
- Click a call to action: follows its link.
- Keyboard: Tab order is the billing options, then each card's call to action and details toggle, then the details contents when open. Left and Right move between billing options. Enter or Space toggles details.
- Escape: closes any open details area and returns focus to its toggle.

## Motion (functional)
- Switching billing animates each price from its old figure to its new one.
- Opening details expands the card's details area downward; the rest of the row keeps its height unless the tallest card changes.
- The billing indicator glides between the two options.
- With reduced motion preferred, prices and details change instantly.

## Accessibility
- The period selector is a radio set whose label reads "Billing period".
- Each plan is a labelled group; price text includes its period ("per month" or "per year") in the text, not only in layout.
- Details toggles are disclosure buttons with an expanded state; the details region is labelled by its toggle.
- Not-included features carry the words "not included".

## Responsive
- Wide: three plan cards in a row.
- Medium: plan cards wrap into rows of two then one.
- Phone width: cards stack in a single column in plan order; the billing switch stays at the top of the section.

## Composition hints
- Radio group or segmented switch (billing), card (plan), badge (highlight and savings), button (call to action), disclosure (details), list with check and de-emphasised markers (features), logo row (integrations).

## Acceptance checks
1. Switching to yearly updates every plan's price to its yearly figure and shows the yearly billing note.
2. Opening one card's details does not open or change the others.
3. The details toggle label and its expanded state agree.
4. Left and Right on the billing switch change the selected period.
5. A not-included feature shows the words "not included".
6. The highlighted plan shows its badge and emphasised call to action.
7. At phone width the plan cards stack in order and nothing scrolls sideways.

## Amendment 5: structure and demo

Archetype: heading and billing switch above three plan cards in one row, each card with its details open.

Replaces the Default state line ("details collapsed on every card"): at rest every card's details are open.

Visible at rest:
- Heading "Pick the plan that fits your team" and one intro line.
- Billing switch: Monthly and Yearly, Monthly selected. A saving label beside Yearly: "Save 15%".
- Three plan cards in one row. The middle card is highlighted: a "Most chosen" badge and a filled call to action. The others have outlined calls to action.
- Each card: name; price with its period ("per month"); billing note; call to action "Start free trial"; limits list (three rows); integrations block (sync cadence, selection count, a row of caller-supplied marks); feature list (seven rows, each marked included or "not included").
- Toggle label on each card reads "Hide details".

Gallery demo:
- Solo $19 per month, $15 billed yearly. Team $59, $47. Studio $149, $119.
- Limits: monthly readers 5k, 50k, 500k. Seats 2, 10, Unlimited. Workspaces 1, 3, 10.
- Sync cadence: Daily, Hourly, Live. Selection: Choose 2 of 8, Choose 4 of 8, Choose 8 of 8.
- Features (Solo, Team, Studio): Unlimited boards (not included, included, included); Custom domain (not included, included, included); API access (not included, included, included); Scheduled exports (included on all three); Single sign-on (not included, not included, included); Dedicated onboarding (not included, not included, included); Email support (included on all three).

Motion, settled frame: Monthly selected; details open; Team highlighted.
Moves: the billing switch rolls each price from its old figure to the new one. Details open and close downwards; the billing indicator glides between the options.
Reduced motion: prices and details change instantly.

Acceptance checks (continued):
8. At rest every card has its details open, its toggle reads "Hide details", and its feature list shows included rows and "not included" rows.
9. Every price figure is fully legible at rest: no digit overlaps another, and the billing roll has finished before the frame is judged.
10. Each integrations block shows its sync cadence, its selection count and a row of marks.
11. The "Most chosen" badge appears on the highlighted card only.
