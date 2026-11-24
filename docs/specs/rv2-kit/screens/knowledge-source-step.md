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

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** one centred question step (a single prompt with one chip field), with a read-only dashboard preview that starts near the centre and is clipped by the right edge.

**At rest (gallery demo):**
- Top bar: account email label and log out. No back control on this first step.
- Step panel, centred in the left half: four step markers with the first marked current; a title written as a question; one prompt line; one chip field holding one chip (the company's own site); a hint line; Continue button, enabled.
- The step holds one question and one field. No other form field appears on it.
- Dashboard preview: a large assistant card in its loading state, with a knowledge table of three rows. Their statuses are Ready, Learning and Queued.
- Footer: copyright line, terms and privacy links.

**States at rest:** one entry; Continue enabled; preview table shows its three rows.

**Arrangement:** step panel in the left half. Preview begins at the middle of the viewport and runs off the right edge. The preview has no frame, border or header of its own, and no divider separates it from the step.

**Motion, settled frame:** the preview table shows its final statuses. Nothing animates under reduced motion.

**Checks (continued):**
11. At rest the step holds one question, one chip field and one Continue button. No other form field appears.
12. The preview's knowledge table lists the pages the assistant is reading from the entered source, one row per page with its status in words. At rest the one chip yields three rows: Ready, Learning and Queued.
13. At 1280 wide the step sits fully inside the left half, and the preview starts near the middle and is clipped by the right edge.
14. The preview has no frame, border or header of its own, and no divider separates it from the step.

## Amendment 7: structure and demo

Replaces: Amendment 5 "At rest (gallery demo)" step-panel bullet and dashboard-preview bullet; Amendment 5 "States at rest" line; Amendment 5 check 11 ("one Continue button") and check 12 (three preview rows). Amendment 5 checks 13 and 14 stand; checks 15 to 19 are new.

Archetype: one centred question step in the left half, with a faded dashboard preview clipped by the right edge.

Stage: the full stage, width to height about 1.6 to 1. The stage carries no host sidebar of its own; the workspace frame item decides chrome. The top bar below is this screen's.

Top bar, 3% to 7% of H: logo mark at the top-left. At the top-right: the account email, then a Log out control ending at the right edge minus a gutter. No back control on step one.

Step block: centred horizontally on 25% of W (the centre of the left half). Spans 40% to 57% of H.
- Step markers at 40% of H: four short dashes in a group, the first in the ink token, the rest muted.
- Question as the title at 44% of H, centred.
- One prompt line at 48% of H, muted, centred.
- Input field at 52% to 57% of H, 31% of W wide, centred. Inside, on the left, one domain chip (the company's own domain, from the sign-up address), then the placeholder for the next entry. At the right end: a "+" icon control and a submit arrow control in the ink token. The arrow is the primary action; assistive tech names it Continue. There is no visible Continue text button.
- No hint line and no second row at rest.

Nothing else on the step. No other field, no panel, no card.

Preview: starts at 50% of W and runs off the right edge. No frame, no border, no header, no divider. Its content is low-contrast skeleton bars, with four words readable: the assistant's name, the "Knowledge" and "Test chat" headings, and the domain row.
- Top band, about 6% to 14% of H: a row of stat tiles, skeleton bars.
- About 15% to 30% of H: "Your assistant" card with the assistant's name, a Draft tag, one greeting line.
- About 31% to 38% of H: conversations chart as skeleton bars.
- About 40% to 58% of H: "Knowledge" table. A header row, then the company's domain row with its status word, then skeleton rows.
- About 60% to 80% of H: "Test chat" with one question bubble and one answer bubble.

Settled frame: the domain row's status is Ready. Rows below it stay skeleton. Nothing animates under reduced motion.

Acceptance checks (continued):
15. The step block is centred on 25% of W and spans 40% to 57% of H.
16. The step holds one question, one prompt line, and one field with one domain chip and an arrow submit control inside it. No visible Continue text button and no hint line at rest.
17. The preview starts at 50% of W, runs off the right edge, and has no frame, border, header or divider.
18. The knowledge table has one text row (the domain and its status word) and skeleton rows below it. The status word cycles Queued, Learning, Ready.
19. Preview text is lower in contrast than the step's prompt line at rest.
