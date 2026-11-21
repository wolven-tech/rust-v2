# Knowledge Source Step

Source item: affectUI Pro `screens/onboarding-website` (functional reference only; see README).

## Purpose
The first step of assistant setup: the person lists the websites, documentation sites, and help centres the assistant should learn from. Each entry visibly lands in a knowledge table on the dashboard beside the step, so the person watches the setup take shape.

## Data
| Field | Type | Meaning |
|---|---|---|
| default_domains | string[] | Domains already known, for example from the sign-up address. The first is the company's own site. |
| on_continue | (domains: string[]) => void | Called with cleaned domains (no scheme, no leading "www", no path) when Continue is pressed. |
| on_back | () => void | Supplies a back control in the top bar; omitted on the first step. |
| on_logout | () => void | Account menu action. |
| account_email | string | Signed-in account, shown top right. |
| product_name | string | Product name for the logo label and footer. |
| brand_mark | node | Optional logo at top left; a neutral mark when omitted. |
| frame_style | "inset" \| "flat" | Dashboard picture frame. When omitted, it follows the host shell's setting. |
| auto_focus | boolean | Places the caret in the entry field as soon as the step appears. |
| step_index / step_count | number | Position in the flow. Defaults to 1 of 4. |
| demo_entries | string[] | Optional text typed once into the field on load, for recordings; stops at the first keypress. |

Dashboard preview (decorative, right side): an assistant card with its name, a status (Draft), a greeting line, conversation and knowledge counts, and a knowledge table with one row per domain. Each row's status moves through "Queued", then "Learning", then "Ready".

## Anatomy
Reading order:
1. Top bar: back control (when present), account label, and log out.
2. Step panel, left of centre: step markers (one per step, current one marked), a title, a one-line prompt, the chip field, its live hint, and the action row with Continue.
3. Chip field: entries shown as removable chips above or inside the input.
4. Dashboard preview, right: on wide layouts it begins near the centre and is clipped by the viewport's right side. Hidden from assistive technology.
5. Footer: product copyright line and terms and privacy links.

## States
- **Empty field.** Continue is available only when at least one domain exists.
- **Entry pending.** Typed text that is not yet a chip; Enter adds it.
- **Invalid entry.** Marked invalid and explained in a polite hint under the field; nothing is added.
- **Entries present.** Each chip holds a domain; the preview has a row for each, its status advancing from "Queued" to "Learning" to "Ready".
- **Continue ready.** One or more valid domains are present.
- **Preview on first entry.** The dashboard slides to the knowledge table when the first chip appears.

## Interactions
- Typing followed by Enter, comma, or space adds the typed text as a domain chip, after cleaning it.
- Enter on the empty field: Continue.
- Backspace on the empty field removes the last chip.
- A chip's remove button removes that chip.
- Continue calls `on_continue` with the cleaned list.
- Back calls `on_back`.
- The account menu's Log out calls `on_logout`.
- Pasting a list of addresses splits it on line breaks and commas into chips.

## Motion (functional)
- The dashboard preview follows the newest entry: its knowledge table scrolls to the row being learned.
- Each new domain row arrives at the bottom of the table, and its status advances through the states above.
- Chip entry is a short settle. Under reduced motion, the preview is static and shows its final state at once.

## Accessibility
- The entry field has a visible label and a hint; the invalid state uses an explicit invalid flag.
- Each chip's remove button names the domain it removes.
- The count of domains is announced in a polite live line beneath the chips.
- The dashboard preview is hidden from assistive technology.
- Animations are disabled under reduced motion.

## Responsive
- When the container is narrow, the step fills the full width and the preview fades out behind it.
- At wide widths, the step is centred in the left half.
- The screen fills its container.

## Composition hints
- Top bar with account menu and back control.
- Step indicator (dashes or progress).
- Chip input with validation and a live hint.
- Primary button for Continue.
- Read-only dashboard preview built from a stat card, an assistant card, and a table with status badges.
- Footer with links.

## Acceptance checks
1. Typing a full URL with a scheme and path, then pressing Enter, adds a chip containing only the bare domain.
2. Pressing Enter on the empty field calls `on_continue` with the current domains.
3. Entering an invalid domain marks the field invalid, shows the hint, and adds no chip.
4. Backspace on the empty field removes the last chip.
5. Pasting "a.com, b.com" creates two chips.
6. Adding the first valid domain shows one row in the preview table.
7. With `on_back` omitted, no back control renders.
8. Continue is disabled while no domain is present.
9. With reduced motion requested, the preview shows its final state without movement.
10. At phone width the dashboard preview sits behind the step and causes no horizontal scroll.
