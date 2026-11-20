# Assistant Ready Step

Source item: affectUI Pro `screens/onboarding-ready` (functional reference only; see README).

## Purpose
The final step of assistant setup: a calm summary of what the assistant learns from, its name, its voice, and its handover, each with an Edit control that returns to the step it came from. The dashboard then comes alive: figures settle and the status changes from draft to live.

## Data
| Field | Type | Meaning |
|---|---|---|
| source_domains | string[] | Sources the assistant learns from; the summary and the knowledge count use them. |
| brand / aliases | string / string[] | From step two. When no brand is given, one is inferred from the leading domain. |
| assistant_name | string | Assistant name from step three. Defaults to "<brand> Assistant". |
| voice_tone | "friendly" \| "neutral" \| "formal" | From step three. |
| reply_length | "short" \| "balanced" \| "detailed" | From step three. |
| handover_to | string | Handover address. An empty string reads as "nobody; it answers itself". |
| on_finish | () => void | Opens the dashboard. End of the flow. |
| on_edit | (step: 1 \| 2 \| 3) => void | Called from each row's Edit control with the step that row came from. |
| on_back | () => void | Back control. |
| account_email, product_name, brand_mark, frame_style, auto_focus, step_index/step_count | as in Knowledge Source Step | Framing. Defaults to 4 of 4. |

Dashboard (preview at the end of the flow): overview navigation with Overview, Conversations, Knowledge, and Settings; an assistant card with a Live status; live counts for knowledge sources and conversations (conversations starts at zero); a status label; and a test chat that answers in the chosen tone and length and shows the handover line.

## Anatomy
Reading order:
1. Top bar: back control, account label, log out.
2. Step panel: step markers, title ("Your assistant is ready"), a one-line prompt, and a summary description list with five rows: learns from, assistant name, voice, hands over to, and (when present) also answers to. Each row has an Edit button naming what it edits. Then the Open dashboard button.
3. Dashboard preview, right: it starts at the top and comes alive: figures land, the status changes from Draft to Live, and the test chat shows its first answer.
4. Footer.

## States
- **Ready.** The default. Summary complete.
- **Long answer.** A value longer than its row ends in an ellipsis; the full value is in the label's tooltip.
- **Nobody handing over.** The handover row reads that the assistant answers itself.
- **Live.** After Open dashboard, the preview shows Live; this is the state the dashboard holds on the next visit.
- **Reduced motion.** The dashboard is already live, with no glide.

## Interactions
- An Edit button on a row calls `on_edit` with that row's step number.
- Open dashboard calls `on_finish`. Enter or Space on it does the same.
- Back calls `on_back`.
- Log out calls `on_logout`.
- Summary rows are read-only text; only the Edit buttons act.

## Motion (functional)
- As the step opens, the dashboard glides back to its top; then the figures settle, and the Draft label turns to Live.
- The test chat's first answer types in once, showing the chosen tone and length.
- Under reduced motion, the dashboard is live from the start.

## Accessibility
- The summary uses description-list semantics, so every answer is announced together with its label.
- Each Edit button names what it edits, for example "Edit voice".
- The dashboard preview is hidden from assistive technology.
- The status change to Live is announced once in a polite region.

## Responsive
- The heading stays level with those of the other steps.
- Overlong answers are cut with an ellipsis rather than wrapping onto a second line.
- The screen fills its container.

## Composition hints
Top bar, step panel, description list with edit buttons, primary button. Read-only dashboard preview built from the shared assistant card, counts, and test chat. Status badge (Draft, Live).

## Acceptance checks
1. With defaults, the summary shows the first domain under "learns from" and the name under "name".
2. Clicking "Edit voice" calls `on_edit` with 3.
3. Clicking "Edit" on the learns-from row calls `on_edit` with 2 for the source step.
4. With an empty handover address, the handover row reads "nobody; it answers itself".
5. Open dashboard calls `on_finish` once.
6. The preview status reads Live after the glide completes.
7. With reduced motion, the preview is Live at first paint.
8. A value longer than its row is truncated with an ellipsis, and its full text is available on hover.
9. Back calls `on_back`.
10. Summary rows are not focusable except for their Edit buttons and the Open dashboard button.
