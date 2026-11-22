# Grounded Reply

Source item: affectUI Pro `components/ai-answer` (functional reference only; see README).

## Purpose
Presents an assistant's answer as it arrives, with the sources it drew on listed up front and numbered markers in the text that point back to them. The reader can ask a follow-up, regenerate, stop, copy, or rate the answer.

## Data
| Field | Type | Meaning |
|---|---|---|
| `heading_text` | string | The question being answered; shown as the heading above the source list. |
| `source_entries` | Entry[] | Sources in display order. Each: `{ entry_id, chip_text, link_target?, leading_mark?, snippet? }`. `chip_text` is the short label on the marker chip. |
| `answer_text` | string | Answer text. A blank line starts a paragraph; a line beginning with `- ` is a list item; `**text**` marks emphasis; `[n]` or `[n,m]` cites entries by their 1-based position. The text may grow while streaming; a change that is not a pure extension restarts the reveal. |
| `is_still_writing` | boolean | More text is still coming. Keeps a live cursor until the reveal catches up. |
| `reveal_speed` | number | Words per second during reveal. Defaults to a pace of our choosing; tunable. |
| `next_questions` | string[] | Suggested follow-up questions, shown after the finished answer. |
| `on_next_question_chosen(text)` | callback | Fired when a suggestion is chosen. |
| `on_retry_requested()` | callback | Fired when regenerate is chosen. The component clears the text and waits for a new `answer_text`. |
| `on_halt_requested()` | callback | Fired when stop is chosen while streaming. |
| `on_vote(vote, reason?)` | callback | `vote` is `up` or `down`; `reason` is one of `vote_down_reasons` when `down`. |
| `vote_down_reasons` | string[] | Reasons offered after a thumbs-down. Default set: inaccurate, wrong citation, too long, outdated. |
| `on_copy_requested(plain_text)` | callback | Receives the answer without markup, followed by a numbered source list. |
| `reveal_animated` | boolean | When false, the full text appears at once. |
| `raised_surface` | boolean | When true, the answer sits on its own raised surface; when false, it inherits the host surface. |
| `root_class` | string | Extra class applied to the root. |

## Anatomy
Reading order, top to bottom:
1. Heading line, with a count of sources.
2. Source row: one card per source (chip text, short excerpt). Up to three cards fit in a row at normal width; the rest collapse into a "+N" tile. Two cards per row when the container is narrow.
3. Answer body. Inline citation markers are small buttons placed after the cited sentence.
4. Follow-up list (only once the answer is finished).
5. Action bar: copy, regenerate, rate up, rate down. Rate down opens a reason menu.

## States
- **Empty / no answer yet:** heading and source row may be present; the body is empty and a "reading" label appears with a pulsing indicator.
- **Reading sources:** source cards appear one after another; the label reads the count being read.
- **Streaming:** the body grows; a pulsing dot marks the growth point; actions are hidden except Stop.
- **Finished:** follow-ups and the full action bar appear; Stop is removed.
- **Rated up:** the up control shows as selected and stays that way until another rating is given.
- **Rated down, reason open:** the reason menu is visible; once a reason is picked, the menu closes and the control stays selected.
- **Copied:** the copy control shows a confirmation for a moment, then returns to normal.
- **Regenerating:** the body clears and the state returns to Reading sources or Streaming.
- **Disabled:** actions are visibly unavailable while `is_still_writing` is true, except Stop.

## Interactions
- Pointer hover on a marker: highlights the cited sentence and its source card.
- Pointer hover on a source card: highlights every marker that cites it.
- Click on a marker or card: opens a short preview of the source (chip text, excerpt, and a link if `link_target` is set).
- Touch: tapping a marker opens its preview; hover effects are not required.
- Escape while a preview is open: closes it and returns focus to the marker or card that opened it.
- Escape while the reason menu is open: closes the menu and returns focus to the rate-down control.
- Follow-up click: calls `on_next_question_chosen` with the text.
- Keyboard: Tab order runs through the source cards, the markers within the body, the follow-ups, then the action bar.

## Motion (functional)
- Source cards enter in sequence, so the reader sees sources being gathered before the text.
- Text enters word by word from a soft blur; markers appear after the sentence they belong to.
- The body container changes height smoothly as text is added, so the content below does not jump.
- Reduced motion: all of the above switch to immediate display.

## Accessibility
- Each marker and source card is a button whose accessible name is its number and chip text, for example "Source 2: <chip text>".
- Markers expose expanded and controls relationships to their preview region.
- While streaming, the body region is marked busy; completion is announced once the text is finished, not per word.
- Copy, rating and status updates are spoken through a courteous status region.
- The preview is labelled as a region; while closed it is removed from the accessibility tree.

## Responsive
- Fills its container up to a comfortable reading width.
- Source cards wrap; the overflow tile takes the place of the last visible card.
- The action bar wraps onto a second row when the container is narrow, with icon-only controls if needed.
- No horizontal scrolling at phone width; marker targets are enlarged on touch.

## Composition hints
Card (source cards), badge (count, chip text), popover or inline preview (source preview), button and icon-button (actions), menu (reason picker), list (follow-ups), live region helper, skeleton or pulsing indicator (reading state).

## Acceptance checks
1. With `is_still_writing` true and `answer_text` growing, the body region grows, and the content below it moves by no more than the height of the new text.
2. A `[2]` in the body renders a marker that is focusable, has an accessible name containing entry 2's chip text, and is the same element that opens entry 2's preview on click.
3. Hovering a source card highlights every sentence that cites it; moving the pointer away removes the highlight.
4. A marker citing an entry id that does not exist is not rendered as a broken button.
5. Thumbs-down opens a reason menu with the configured reasons; choosing one calls `on_vote("down", reason)` exactly once.
6. Copy places plain text on the clipboard containing the answer without markup, followed by a numbered source list.
7. Regenerate clears the body and calls `on_retry_requested` once; no text from the previous body remains visible.
8. Escape closes an open preview and returns focus to its opener.
9. With reduced motion requested, the full body is visible on first render with no word-by-word reveal.
10. At phone width there is no horizontal page scrolling.
</content>
