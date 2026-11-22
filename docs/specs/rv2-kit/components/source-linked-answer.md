# Source-Linked Answer

Source item: affectUI Pro `components/cited-answer` (functional reference only; see README).

## Purpose
Presents a finished answer whose sentences are each tied to the source passages they were drawn from. A reader can point at a citation to see which sentence and which source it links, and open a source to see the quoted passage with its matching words marked. Used to check an answer, not to watch it stream.

## Data
| Field | Type | Meaning |
|---|---|---|
| `answer_segments` | Segment[] | The answer in order: `{ text, cited_ids?: ref_id[], starts_paragraph? }`. Leading and trailing spaces stay outside the highlight |
| `reference_list` | Reference[] | `{ ref_id, headline, origin_site, link_target?, published_on?, passage?, exact_phrase?, leading_mark? }`. Numbered in list order, one chip each |
| `pages_consulted` | number | Shown in the header as a count of sources and pages read |
| `start_open_ref` | string | `ref_id` whose preview is open first |
| `on_open_request(reference)` | callback | "Open reference" pressed. When provided, the link does not navigate by itself |
| `on_preview_change(ref_id \| null)` | callback | Preview opened, switched or closed |
| `trailing_action` | `{ action_text, on_activate }` | Optional button in the last row |
| `root_class` | string | Extra class on root |

## Anatomy
Reading order:
1. Header: source count and pages-read count.
2. Answer body: paragraphs of segments. Each cited segment ends with small numbered markers.
3. Reference chip row: one chip per reference with its number and site; overflow collapses to a "+N" chip.
4. Preview region (below the chip row when open): reference headline, site, date, passage with matching words marked, and actions (Open reference, Copy passage).
5. Footer row: a status line that describes the last action, and the optional trailing action.

## States
- **Closed**: no preview; all markers and chips at rest.
- **Hovering or focusing a marker**: its segment gets a soft tint; its chip is highlighted.
- **Hovering or focusing a chip**: every segment citing it is tinted; the rest of the answer fades back.
- **Preview open**: preview shows the chosen reference; switching to another reference cross-fades its content in place.
- **Passage copied**: footer status line reads that the passage was copied; reverts after a moment.
- **Reference without passage**: preview shows headline and link only.
- **Citation to an unknown ref id**: marker is not rendered.

## Interactions
- Pointer: hover marker or chip as above; click marker or chip opens its preview; click again closes.
- Tab: markers in reading order, then chips and "+N", then preview buttons, then trailing action.
- Enter or Space on a marker or chip: opens or closes its preview.
- Left/Right arrows or Home/End: move between chips and "+N".
- Escape: closes the preview and returns focus to its chip.
- Touch: markers and chips have enlarged hit areas.

## Motion (functional)
- Hovering a marker tints its sentence and lights its chip.
- Preview unfolds below the chip row and folds away again.
- Switching references cross-fades content in place.
- Footer status line cross-fades between messages.
- Reduced motion: immediate.

## Accessibility
- Each marker and chip is a button named "Reference n: <site> – <headline>", with expanded state and a control relation to the preview.
- Focus on a marker or chip highlights the same links that hover does.
- Preview is a named region, inert while closed.
- Footer status line is polite live.

## Responsive
- Fills its container up to a readable width and works down to a narrow width.
- Chips stay on one line; overflow goes into "+N".
- Below a narrow width, the preview drops secondary metadata and the copy action shrinks to an icon.
- Touch: larger targets for markers, chips and preview buttons.
- No horizontal page scroll at phone width.

## Composition hints
Card, badge or chip (reference chips), button (markers, actions), disclosure or region (preview), tooltip (site or headline), live region helper, text highlighter (matching words in passage).

## Acceptance checks
1. A segment with `cited_ids: ["b"]` shows one marker whose accessible name contains reference b's headline; clicking it opens b's preview.
2. Hovering a chip tints every segment that cites that reference and dims segments that do not.
3. A `cited_ids` entry that is not in `reference_list` renders no marker and no error.
4. Opening a reference with a `passage` marks the words shared with the citing sentence; with an explicit `exact_phrase`, only that exact phrase is marked.
5. Switching from one reference preview to another calls `on_preview_change` once with the new id.
6. Escape closes the preview and focus lands on the chip that opened it.
7. With `on_open_request` given, the Open reference action calls it and does not navigate; without it, the link opens in a new tab.
8. The last word of a cited sentence does not wrap alone away from its markers.
9. At the narrowest phone width the copy action keeps an accessible name while showing only its icon.
10. At phone width no horizontal scroll occurs.

## Amendment: structure and demo

**Archetype.** A prose answer card. The answer text is the main content; citations are small markers inside the text; a row of source chips sits under it. It is not a bibliography list and not a chat bubble.

**Parts that must exist, top to bottom:**
- Header line: a count of sources and a count of pages read, both as numerals.
- Answer body: two paragraphs, six sentences in total.
- Inline markers: a sentence citing one source shows one marker; a sentence citing two shows two markers, in the order given.
- Chip row: one chip per source with its number, its site and a shortened headline. Chips past the row width fold into one "+N" chip.
- Preview region directly beneath the chip row.
- Footer row: a status line on the left, one trailing button on the right.

**Gallery demo data:** four sources; three of them carry a passage; one source is cited from two separate sentences; the first source's passage contains words shared with the citing sentence, which are marked.

**States visible at rest in the gallery:**
- Preview open on the first reference: headline, site, date, passage with matched words marked, and two action buttons.
- Markers and chips at rest, none highlighted.
- Status line present in the DOM, empty or showing an idle message.
- Trailing button present.

**Acceptance checks, continued:**
11. Gallery at rest: the preview is open on the first reference and shows its headline, site, date, a passage with matched words marked, and both action buttons.
12. A segment citing two ids renders both markers, in the order given, after its last word.
13. The trailing button is the last focusable element in the card, and activating it does not change the open preview.
14. The preview region is named and leaves the accessibility tree whenever no preview is open; the status line exists in the DOM at rest.
