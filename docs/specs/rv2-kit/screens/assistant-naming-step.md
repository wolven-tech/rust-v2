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
