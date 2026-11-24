# Assistant Naming Step

Source item: affectUI Pro `screens/onboarding-brand` (functional reference only; see README).

## Purpose
The second step of assistant setup: the person enters the business name along with any abbreviated names its customers say. The assistant introduces itself with the first name, and every other name is recognised as well.

## Data
| Field | Type | Meaning |
|---|---|---|
| website_domain | string | Website from the previous step. Proposes the first name (for example "harbor-lane.com" proposes "Harbor lane"). |
| default_names | string[] | Names already known; they replace the proposal. The leading entry is what the assistant calls itself. |
| on_continue | (names: string[]) => void | Invoked with the list when Continue is pressed; its leading entry doubles as the assistant's name. |
| on_back | () => void | Back control in the top bar. |
| on_logout | () => void | Account menu action. |
| account_email | string | Signed-in account, top right. |
| product_name | string | Logo label and footer. |
| brand_mark | node | Optional logo at top left. |
| frame_style | "inset" \| "flat" | Dashboard preview frame. |
| auto_focus | boolean | Moves focus into the text field on open. |
| step_index / step_count | number | Defaults to 2 of 4. |
| demo_entries | string[] | Optional text typed into the field once, for recordings. |

Dashboard preview (decorative): an assistant card showing its name, a status label (Draft), a greeting line, a tag for each short form, and a sidebar with Overview, Conversations, Knowledge, and Learning; a test chat panel posing a sample question.

## Anatomy
Reading order:
1. Top bar: back control, account label, and log out.
2. Step panel: step markers, title, a one-line prompt, the name chip field with its live count line, and Continue.
3. Chip field: the first chip is the primary name; following chips are short forms.
4. Dashboard preview, right: the assistant card. Its name and tags fill in as the person types; the greeting uses the first name.
5. Footer with copyright and terms and privacy links.

## States
- **Suggested.** The domain-derived first name is pre-filled as a chip.
- **Editing.** Typed text that is not yet a chip.
- **Primary name.** The first chip; the assistant card and greeting use it.
- **Short form.** Any later chip; shown as a tag on the assistant card.
- **Empty.** No chips: Continue is available only when at least one name exists.
- **Removing.** A chip is removed and the card updates.
- **Continue ready.** At least one name is present.

## Interactions
- Enter or comma turns the typed text into a chip.
- With the field empty, Enter moves on: Continue.
- Backspace on the empty field removes the last chip.
- A chip's remove button removes that chip; if it was the primary, the next chip becomes primary.
- Dragging chips to reorder changes which name is primary (optional for this step; the keyboard alternative is the chip's "Make primary" menu item).
- Back calls `on_back`. Log out calls `on_logout`.

## Motion (functional)
- As the step opens, the dashboard glides up to the assistant card, so the eye moves from the form to what it changes.
- The assistant name and tags fill in letter by letter as typing proceeds; removing a tag collapses it in place.
- Under reduced motion, the card updates without the fill effect.

## Accessibility
- One field: Enter or comma adds a chip; Enter on an empty field continues.
- Each chip carries a removal control whose label includes the chip's text.
- A polite status line reports how many names are present after each edit.
- The dashboard preview is hidden from assistive technology.
- Animations are off under reduced motion.

## Responsive
- Same layout as the Knowledge Source Step, so moving between steps shifts nothing.
- Below the narrow breakpoint, the dashboard preview sits behind the step.
- The screen fills its container.

## Composition hints
Top bar, step panel, chip input, primary button. Assistant card with name, status badge, and tag list. Read-only dashboard preview shared with the other onboarding steps. Footer.

## Acceptance checks
1. With `website_domain` "harbor-lane.com", the first chip reads "Harbor lane" before any input.
2. With `default_names` given, the proposal is replaced by those names in order.
3. Typing a name and pressing comma adds a chip, and updates the assistant card's name when it is the primary chip.
4. Backspace on the empty field removes the last chip, and the card's tags update.
5. Continue calls `on_continue` with the names in chip order.
6. The live count line reports the correct number of names after each change.
7. Continue is disabled when no names are present.
8. With `on_back` omitted, no back control renders; with it present, clicking calls it.
9. Removing the primary chip makes the next chip primary in the card and the greeting.
10. Under reduced motion, the card updates with no letter-by-letter effect.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** one centred question step (the business name and its short forms), with a read-only assistant card preview that starts near the centre and is clipped by the right edge.

**At rest (gallery demo):**
- Top bar: back control, account label, log out.
- Step panel, centred in the left half: step markers showing step 2 of 4; a title written as a question; one prompt line; one chip field holding two chips, the first marked primary and the second a short form; a live count line; Continue button, enabled.
- The step holds one question and one field. No other form field appears on it.
- Preview: assistant card showing the primary name, status Draft, a greeting that uses the primary name, and one tag for the short form. A sidebar with four entries, and a test chat with one question and its answer.
- Footer: copyright line, terms and privacy links.

**States at rest:** two names, primary first; Continue enabled; card shows the tag.

**Arrangement:** step panel in the left half. Preview begins at the middle of the viewport and runs off the right edge, with no frame or divider of its own.

**Motion, settled frame:** the card shows its final text. Nothing types out under reduced motion.

**Checks (continued):**
11. At rest the step holds one question, one chip field and one Continue button. The chip field holds two chips, the first marked primary.
12. The assistant card shows the primary chip's name, the Draft status, one tag per short-form chip, and a greeting that uses the primary name.
13. The live count line reads the number of names as a number after each change.
14. At 1280 wide the step sits fully inside the left half, and the preview starts near the middle and is clipped by the right edge.

## Amendment 7: structure and demo

Replaces: Amendment 5 "At rest (gallery demo)" step-panel bullet (the chip field now sits in one input with the chips beneath it, and the Continue text button is removed); Amendment 5 "Arrangement" line for the step; Amendment 5 check 11 (chips in one field) and check 13's visible placement (the count line is a polite live line, visually hidden; the visible line at rest is the hint below). Amendment 5 checks 12 and 14 stand.

Archetype: one centred name step in the left half: an input, a chip row beneath it, and a hint line; a faded assistant card clipped by the right edge.

Top bar, 3% to 7% of H: a Back control (text pill with an arrow) at the top-left beside the logo mark. At the top-right: the account email and a Log out control.

Step block: centred horizontally on 25% of W. Spans 39% to 63% of H.
- Step markers at 40% of H: four short dashes; the first two in the ink token (step 2 of 4), the rest muted.
- Question title at 44% of H, centred.
- One prompt line at 48% of H, muted, centred.
- Input at 52% to 57% of H, 31% of W wide, centred. Holds the placeholder or the typed text. At its right end: a "+" control and a submit arrow control in the ink token. No visible Continue text button.
- Chip row at 59% of H, centred under the input. The first chip is the primary name and carries a small dot at its start; no text label. Each chip has a remove control. A later chip is a short form. Gallery: "Northwind" (primary), "Northwind Co.".
- Hint line at 62% of H, centred, muted. Gallery text: "Your assistant calls itself by the first name."

Nothing else on the step.

Preview: starts at 50% of W and runs off the right edge, no frame, border or divider. A faint sidebar column of short bars at 50% to 66% of W, then the content:
- Top band: stat tiles as skeleton bars.
- "Your assistant" card: the primary name, a Draft tag, one greeting line that uses the primary name, and an "Also answers to" row with one tag per short form.
- Conversations chart as skeleton bars.
- Knowledge table: the company's domain row with its status word, then skeleton rows.
- Test chat at the lower part: one question bubble, one answer bubble.

Settled frame: the chip row holds both names, the hint line is visible, the "Also answers to" row shows the short form, the domain row shows Learning. Nothing types out under reduced motion.

Acceptance checks (continued):
15. The step block is centred on 25% of W and spans 39% to 63% of H.
16. The chip row sits directly under the input, centred on the left half. The primary chip is first and carries a dot. The hint line sits under the chips.
17. The gallery has no visible Continue text button; the submit control is an arrow inside the input. The top bar holds Back at the top-left.
18. The assistant card shows the primary name, the Draft tag, a greeting naming the primary, and an "Also answers to" row with one tag per short form.
19. The preview starts at 50% of W, runs off the right edge, and has no frame or divider.
