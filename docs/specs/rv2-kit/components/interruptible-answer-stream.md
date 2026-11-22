# Interruptible Answer Stream

Source item: affectUI Pro `components/response-stream` (functional reference only; see README).

## Purpose
Presents one assistant answer through every state it can pass through: writing, stopped, failed, and finished. The reader can stop the answer, continue it from the exact word where it stopped, retry after a failure without losing the text so far, copy or regenerate it when finished. Messages the reader sends while the answer is still being written appear above it, either as a steer that bends the answer from that point or as a queued message that becomes the next answer.

## Data
- `text`: string, the answer so far. The text grows while streaming. A trailing partial word is held back until it is complete. A fenced block is rendered as a code block.
- `status`: `"streaming" | "stopped" | "failed" | "done"`. The footer, caret and identity mark follow it.
- `error_message?`: one sentence, shown when `status` is `"failed"`.
- `agent`: `{ id, name, color? }`. Default name: "Assistant". The identity mark is drawn from `id`.
- `steers`: `{ id, text, kind: "steer" | "queued" }[]`. Shown as chips above the answer. The first `queued` chip reads "Next". More than three chips fold behind a "+ n more" control.
- `bends`: `{ at, text? }[]`. `at` is a character offset in `text` where a steer took effect. Each bend renders as a quiet divider in the body.
- `meta?`: `{ tokens?, duration_ms? }`, shown in the finished footer.
- `on_stop()`, `on_continue()`, `on_retry()`, `on_dismiss()`: callbacks for the stop, continue, retry and dismiss controls.
- `on_remove_steer(id)`: fired by a chip's remove control.
- `on_copy(plain_text)`: fired by the copy control. The text is also placed on the clipboard.
- `on_regenerate?()`: when supplied, the regenerate control appears in the finished footer.
- `escape_stops`: boolean, default `true`.
- `extra_class?`

Demo values in any reference are invented for this spec; no affectUI data is reused.

## Anatomy
From top to bottom:
1. Steer row: chips for each steer, with a remove control on each. Absent when there are no steers.
2. Answer block: the identity mark and agent name, then the body. The body holds the text, code blocks, and bend dividers.
3. Status line: a small label naming the state ("Writing an answer", "Stopped", "Failed", or the finished footer), set on a rule.
4. Error line: shown only when failed, on its own line beneath the text.
5. Footer:
   - Streaming: a Stop control, with a visible hint that Escape also stops.
   - Stopped: Continue and Dismiss.
   - Failed: Retry.
   - Done: the time and token figures from `meta`, Copy, and Regenerate when supplied.

## States
- Streaming: text grows; a caret sits at the growth point; the footer shows Stop. Copy and Regenerate are hidden.
- Stopped: growth ceases; the caret is removed; the status line reads Stopped; the footer offers Continue and Dismiss.
- Failed: the status line names the failure; the error sentence sits beneath the text; the footer offers Retry. The text written before the failure is kept.
- Done: the caret is removed; the footer shows `meta`, Copy, and Regenerate when supplied.
- Dismissed: after the caller removes the answer, the answer fades out and `on_dismiss` has been called once it has finished fading.
- Steer chip: `steer` or `queued`. The first queued chip is labelled "Next". Chips beyond three are folded.
- Bend divider: present at each bend's offset. It carries the steer text when `text` is given.

## Interactions
- Stop: calls `on_stop`; the state moves to Stopped.
- Escape while streaming: calls `on_stop`, unless focus is inside a text field.
- Continue: calls `on_continue`; the same answer keeps appending from where it stopped.
- Retry: calls `on_retry`; the answer continues from the text written so far.
- Dismiss: calls `on_dismiss` after the fade.
- Copy: calls `on_copy` with the plain text; the text is placed on the clipboard.
- Regenerate: calls `on_regenerate` when supplied.
- A chip's remove control: calls `on_remove_steer(id)`.
- Folded chips: the "+ n more" control expands the folded set in place.
- Keyboard: Tab reaches each footer action and each chip's remove control. Enter or Space activates the focused control.

## Motion (functional)
- Newly written words enter from a soft blur, quickly.
- The caret is a slim mark with a faint halo; it pulses while streaming and stays still under reduced motion.
- The identity mark shows a light wave while writing and is still otherwise.
- Dismissal fades the answer away.
- Reduced motion: words appear without the blur, and the caret is still.

## Accessibility
- The body is a region marked busy while streaming.
- Newly written text is spoken politely one sentence at a time, never word by word.
- Each status change is announced once: stopped, failed with its reason, finished.
- Each steer divider is a separator whose name is the steer text it records.
- Each footer control is a button with a name that states its action.
- The focus indicator shows only for keyboard focus.
- Status is conveyed by its label text, not by the identity mark or the caret.

## Responsive
- The answer grows downward. The footer stays on a single row at a fixed height and does not move sideways as its controls change.
- Below a narrow container width, the indent is removed. The footer may wrap onto a second row. The error sits on its own line.
- Long words and URLs wrap. Code blocks scroll sideways inside their own box; the page itself never scrolls sideways.
- Dark theme is supported through an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Avatar or identity mark, text body with markdown-style fenced blocks, badge or chip (steers), button and icon-button (footer), separator (bend dividers), polite live region for status and sentence announcements, busy region for the body.

## Acceptance checks
1. While `status` is `streaming` and `text` grows, the body grows downward; content below it moves by no more than the height of the added text.
2. A trailing partial word is not shown until it is complete.
3. Stop, and Escape while focus is outside a text field, each call `on_stop` once and change the state to Stopped.
4. Continue calls `on_continue` once and appends to the same text; the text written before the stop remains.
5. Failed shows the error sentence beneath the text and a Retry control that calls `on_retry` once.
6. Done shows the `meta` figures, Copy, and Regenerate only when `on_regenerate` is supplied.
7. Copy calls `on_copy` with plain text containing no markup.
8. Chips beyond three fold behind a "+ n more" control; the first queued chip reads "Next".
9. Each bend renders one divider at its offset, and the divider is a labelled separator when it has a text.
10. At phone width there is no horizontal page scrolling; only a code block scrolls, inside its own box.
11. With reduced motion requested, words appear without a blur and the caret does not breathe.

## Amendment: structure and demo

**Archetype.** One chat turn: the question sits above as a user bubble, and the assistant's answer block follows it. The gallery shows one turn, not a stack of separate answer cards.

**Parts that must exist, top to bottom:**
- User bubble holding the question.
- Steer row: absent when there are no steers; when present, it sits above the answer block.
- Answer block: identity mark and name, body, caret while streaming.
- Status line on a rule: a label naming the state.
- Footer: controls for the current state only.

**Gallery demo data:** a one-sentence question; an answer of one paragraph of about three sentences, whose final word is still held back.

**States visible at rest in the gallery:** one turn, streaming. Stopped, failed and done are reached through the gallery's own controls and replace that same turn in place. No steer chips and no error line are shown at rest.

**Acceptance checks, continued:**
12. Gallery at rest: one user bubble with the question, then one assistant turn that is streaming, with a caret at the growth point and a Stop control.
13. Gallery at rest: the footer holds only the Stop control and its Escape hint; Copy and Regenerate are not in the DOM.
14. The gallery's controls move the same turn through stopped, failed and done in place; the turn is never duplicated and the question bubble is unchanged.
15. The busy flag is set on the answer body only while streaming; the question bubble is outside it.
