# Assistant Persona Step

Source item: affectUI Pro `screens/onboarding-tone` (functional reference only; see README).

## Purpose
The third step of assistant setup: the person names the assistant, picks how it sounds, how long its replies run, and whether it hands over to a human when unsure, and if so to which address. A test chat on the dashboard beside the form answers under those choices, so the effect is visible before continuing.

## Data
| Field | Type | Meaning |
|---|---|---|
| website_domain | string | Website from step one. Used to propose the handover address (support at the domain). |
| brand_name | string | Name from step two. Derived from the domain when omitted. |
| short_forms | string[] | Short forms from step two; shown as tags on the assistant card. |
| default_name | string | Assistant name. Defaults to "<brand> Assistant". |
| default_tone | "friendly" \| "neutral" \| "formal" | Tone selected at first. Defaults to "neutral". |
| default_length | "short" \| "balanced" \| "detailed" | Reply length. Defaults to "balanced". The test chat's reply follows it. |
| default_handover | boolean | Hand over to a person when unsure. Defaults to true. |
| default_email | string | Handover address. Defaults to support at the domain. |
| on_continue | (values: { assistant_name; voice_tone; reply_length; handover; handover_email }) => void | Called on Continue. Blocked unless a name exists and, while handover is enabled, the address is valid. |
| on_back | () => void | Back control. |
| on_logout | () => void | Account menu action. |
| account_email, product_name, brand_mark, frame_style, auto_focus, step_index/step_count | as in Knowledge Source Step | Shared framing. Defaults to 3 of 4. |

## Anatomy
Reading order:
1. Top bar: back control, account label, log out.
2. Form, left column, left-aligned: heading, a one-line prompt, then four fields:
   - Name: a text field.
   - Tone: a three-option radio group (friendly, neutral, formal).
   - Reply length: a three-option radio group (short, balanced, detailed).
   - Handover: a labelled switch row; enabling it reveals a mandatory address input.
   Then the Continue button.
3. Dashboard preview, right: the assistant card with name, status, greeting, and tags; a test chat with one sample question and an answer that uses the chosen tone and length; a handover line naming the address when the handover switch is on.
4. Footer.

## States
- **Defaults.** Name prefilled from the brand; tone neutral; length balanced; handover on.
- **Name missing.** Continue off.
- **Handover on, address invalid.** The address field is marked invalid; Continue off.
- **Handover off.** The address field is disabled and removed from the tab order; the handover line in the preview says nobody takes over and the assistant answers itself.
- **Tone or length changed.** The test chat reply re-renders in the new style immediately.
- **Name changed.** The assistant card and the test chat greeting update.

## Interactions
- Name field: typing updates the card and greeting; Enter continues when the form is valid.
- Tone and length: arrow keys move within each group; focus or a click selects an option.
- Handover switch: click or Space toggles it; turning it off disables the address field.
- Address field: Enter continues when valid; an invalid address shows an error and keeps Continue off.
- Continue calls `on_continue` with the values.
- Back calls `on_back`.

## Motion (functional)
- As the step opens, the dashboard glides down to the test chat, so the person sees a reply.
- Changing tone or length re-types the sample reply in the new style.
- The handover line rises in when the switch turns on and folds away when it turns off.
- Under reduced motion, the reply changes without typing, and the lines appear at once.

## Accessibility
- Every field has its label above it.
- Tone and reply length are radio groups, each with one tab stop; arrow keys move the choice.
- The handover control is a switch named by its row; switching it off greys out the address input.
- An invalid address is marked invalid and Continue stays unavailable; the reason is given in a polite hint.
- The dashboard preview is hidden from assistive technology.

## Responsive
- Same frame as the other steps; the title sits at the same height in every step.
- Below the narrow breakpoint, the preview sits behind the form.
- The screen fills its container.

## Composition hints
Top bar, step panel. Text field, radio group (segmented or stacked), switch row with a conditional field. Primary button. Assistant card, test chat bubble, and handover line in the read-only preview.

## Acceptance checks
1. With defaults, Continue is enabled and calls `on_continue` with tone "neutral", length "balanced", and handover true.
2. Clearing the name disables Continue.
3. Turning handover off disables the address field, and Continue stays enabled.
4. An address whose domain has no dot marks the field invalid and blocks Continue while handover is on.
5. Choosing "formal" with the arrow keys changes the test chat reply style and calls no callback until Continue.
6. Enter in the name field continues when the form is valid.
7. The assistant card shows the chosen name, with each short form as a tag.
8. With reduced motion, the test chat reply appears without a typing animation.
9. At phone width the preview produces no horizontal scroll.
10. Continue calls `on_continue` exactly once per press.
