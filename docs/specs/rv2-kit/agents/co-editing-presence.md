# Co-Editing Presence

Source item: affectUI Pro `components/agent-cursors` (functional reference only; see README).

## Purpose
Gives a person a view of one or more agents working inside a shared document: where each agent sits, what it is doing, which edits it is suggesting, and which margin notes it has left. Every suggested edit waits for the person to accept or decline it. The person may also follow any agent's viewpoint.

## Data
- `participants`: `{ id, name, palette_key?, picture?, wash_key?, spend? }[]`. Without a palette key, every participant is drawn in one neutral ink and pictures tell them apart.
- `content`: document blocks in order. Adjacent `item` blocks form one list. `block_type`: `title`, `heading`, `paragraph`, `item`, `code`, `image`, `table`.
  - `image`: `column_fraction` (0 to 1 of the column width), `aspect`, `wrap_side` (`left` | `right`; following text wraps beside it), `is_loading`, `source_url`, `description`. The text is the caption.
  - `table`: `cell_grid`, rows of cell blocks whose top row names the columns. A cell may carry a `badge_key`, rendered as a status pill.
- `positions`: a map from participant id to `{ block_id, offset?, in_proposal?, selection?, handle?, activity?, is_typing? }`, or `null` (equivalent to absent). `offset` is a character index into the block's current text. `in_proposal` names a proposal id when the position falls inside a suggested insertion. `selection` marks a passage. `handle` marks a corner or edge midpoint of an image, used for resizing. `activity` is a short status line. `is_typing` is a boolean.
- `pins`: margin notes `{ id, participant_id, block_id, offset?, horizontal_fraction?, vertical_offset?, body, is_open? }[]`. A pin is anchored to a character, or to a block's top edge at a horizontal fraction (0 to 1) across the page.
- `proposals`: `{ id, participant_id, block_id, offset, removal?, insertion?, reason? }[]`. Removed text is struck through and inserted text is underlined, both in the participant's palette key.
- `views`: `{ id, caption }[]`, `active_view`, `on_view_change(id)`.
- `followed_id` / `on_follow_change(participant_id | null)`: controlled or uncontrolled.
- `heading`: the document name.
- `page_height?`: a fixed page height. When set, content fades out toward the bottom.
- Callbacks: `on_apply(proposal)`, `on_dismiss(proposal)`, `on_apply_all()`, `on_pin_close(pin)`, `on_pin_reply(pin, text)`.
- Helper `apply_proposal(blocks, proposals, id)` returns `{ blocks, proposals }`: the insertion is written into the text, and other proposals in the same block have their offsets shifted.
- `format_spend(amount) => string`.
- `extra_class?`

## Anatomy
From top to bottom:
1. Header: the document heading; a view switcher (when `views` is supplied); a row of participant faces, one per participant, each showing its state; a count of pending proposals; an "Apply all" control when more than one proposal is pending.
2. Page: the document blocks in reading order, heading first.
3. Overlays on the page:
   - Pointers: one per participant that has a position, with a flag showing its name and `activity`, and a spend tag.
   - Selections: passages a participant has selected, drawn in its palette key.
   - Proposal marks: struck-through removals and underlined insertions, each focusable.
   - Pin markers: round markers bearing the participant's glyph. They expand into a window on hover, or when `is_open` is set.
4. Proposal window: opens for a focused or clicked proposal. It shows the participant, the reason, and Apply and Dismiss buttons.
5. Pin window: shows the body, a Close control and, when `on_pin_reply` is supplied, a reply field (Enter sends).

## States
- Participant present or absent. Absent means its entry in `positions` is null or missing.
- Participant typing (the caret holds still, the pointer follows each letter) or idle.
- Participant followed or not. A followed participant's face shows a filled state; other pointers and selections fade back.
- Proposal pending, applied, or dismissed. Applying writes the change into the text and clears the mark; dismissing clears the mark only.
- Pin closed, open, or replying.
- Image loading (placeholder with a light sweep) or loaded.
- Switching view fades in the new view.
- Empty document: not shown in the public demo.

## Interactions
- Clicking a participant face toggles following it.
- Clicking a proposal opens its window. Apply calls `on_apply`; the host applies the change, optionally through `apply_proposal`. Dismiss calls `on_dismiss`.
- Hovering a pin marker opens its window; moving away closes it unless `is_open` is set.
- Dragging an image's corner handle resizes it, and the new `column_fraction` is reported to the host.
- Keyboard: Tab moves through the participant faces, then through the proposals; focusing a proposal opens its window. A face with focus toggles following on Enter or Space. Escape closes a proposal window.
- Pins: Enter in the reply field sends `on_pin_reply`.
- Touch: a tap opens a window; hit targets match the pointer ones.
- A click outside closes any open window (inferred; the clean room should verify it).

## Motion (functional)
- A moving pointer glides to its next position. A new pointer appears where it lands. A departing pointer fades out where it last was.
- While a participant types, its caret stays fixed and its pointer tracks each new character.
- A chosen passage brightens underneath the words, tinted with the participant's palette key.
- New proposals fill in gently.
- Hovering a pin marker widens it into a window; moving away collapses it again. The glyph remains visible on the marker.
- A ring of activity runs around a participant's face while it is present in the document.
- Images show a sweep while loading and then fade in. A resized image adjusts smoothly.
- Switching views fades; table rows added by a participant slide in.
- Reduced motion: every change applies at once.

## Accessibility
- Body copy is exposed as true headings, paragraphs, and list items. Cursor-like markers, carets, and highlighted ranges are hidden from assistive technology.
- Each proposal takes focus and is labelled with its author and the edit (for example "Analyst proposes inserting: 38%"). The window it opens is a dialog offering Apply and Dismiss.
- Participant faces are toggle buttons (`aria-pressed`), named by participant and current activity.
- New proposals are announced politely, along with the participant's reason.

## Responsive
- Positions are measured from the rendered page, so pointers stay on their characters at any width, text size, or zoom level.
- Pin windows open toward the free side of the viewport: right and down by default, left near the right edge, up near the bottom.
- Flags sit to the lower right of their pointer and may clip at the right edge rather than reposition.
- When the container is narrow: page margins tighten, and the proposal count gives way to the participant faces.
- Light and dark themes are supported through an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Document renderer (headings, paragraphs, list, table, image), overlay layer with absolutely positioned markers measured from the text, avatar and toggle buttons, dialog (proposal, pin), text input (reply), badge (spend, state), progress sweep (image load), polite live region, toolbar with view switcher.

## Acceptance checks
1. A participant entry in `positions` renders a labelled pointer at the given character offset; setting it to `null` removes the pointer with a fade.
2. A proposal shows its removal struck through and its insertion underlined, both in the participant's palette key.
3. Clicking Apply on a proposal calls `on_apply`; with `apply_proposal` the text changes and the mark disappears.
4. Clicking a participant face toggles following; while following, other pointers and selections fade.
5. Tab reaches each participant face and each proposal; focusing a proposal opens its dialog; Escape closes it.
6. Enter on a focused face toggles following.
7. A new proposal produces one polite live announcement that includes the participant's reason.
8. Setting `is_typing: true` keeps the caret fixed while the pointer offset advances with each character.
9. With reduced motion on, pointer movement and fades are immediate.
10. At a narrow width, pointers stay on their characters after the text rewraps.

## Amendment: structure and demo

**Archetype: reading column with anchored overlays.** One column of document blocks. Presence markers, proposal marks and margin pins sit over the text and are placed by their anchor: a character, or the top edge of a block at a horizontal fraction. Nothing on the page is placed freely. The header is a single row.

**Parts that must exist**
- Header row: document title, a two-option view switch, one face per participant, a pending-count badge, and an "Apply all" control.
- Page column: blocks in order. The demo uses one title, two headings, two paragraphs, one list of three items, one table (a header row and two body rows, one body cell rendered as a status badge) and one image with a caption.
- Overlay layer: one pointer per present participant, each with a flag (name, activity line, cost tag); selection highlights; proposal marks; pin markers.
- Two windows, the proposal window and the pin window, each closed until opened and anchored to what opened it.

**Gallery demo counts**
- 4 participants: Ada, Ben, Dee and Cy. Cy has no position.
- 3 pointers on the page; Ada's pointer is typing.
- 1 selection, held by Dee over a paragraph.
- 2 pending proposals: one removal plus insertion in the first paragraph, one insertion only in the list.
- 2 pins: one closed (marker only, glyph visible), one open (body, Close control, reply field).
- 1 image, loaded.

**Visible at rest** (no interaction)
- Three pointers with their flags, Ada's typing pointer included.
- Cy's face in the absent state, and no pointer for Cy on the page.
- Both proposal marks, with the pending count and the "Apply all" control in the header.
- The open pin window with its reply field.
- Both proposal windows closed; no participant followed.

**Acceptance checks (continued)**
11. The gallery header shows four faces; the face with no position is marked absent and no pointer for it appears on the page.
12. The table block renders a header row and two body rows; one body cell is a status badge.
13. With one proposal pending the "Apply all" control is absent; with two it is present and the pending count reads 2.
14. A closed pin shows only its marker with its glyph; the open pin shows its body, a Close control and a reply field.
15. A press outside an open proposal or pin window closes that window and leaves the document unchanged.

## Amendment 2: structure and demo

This amendment changes three things from the first amendment: where the view switch sits, how pointers are drawn, and the gallery counts. Checks 11 and 14 are withdrawn and replaced by 16 to 20.

**Archetype: one document column, a control strip above it, and placed overlays.** The view switch sits above the card's top-left corner, outside its border. Inside the card, one header row carries the breadcrumb path, the face group, a count of present participants with their total spend, and the bulk control. Each pointer is a separate glyph set at its character, with its flag offset to the glyph's lower right. Name tags live in the flags, never in the text flow.

**Parts that must exist**
- View switch above the card.
- Header row inside the card: breadcrumb path (replacing the heading), face group, present count with total spend, bulk control. The bulk control names the pending count when more than one proposal is pending.
- Page column: blocks in order. The page has a set depth, and its lower edge dims the content that runs past it.
- Pointer: a glyph at a character position, with a flag holding the participant name, the activity line and a spend tag. A flag never hides document text: the line carrying a pointer keeps room above it for the flag. Flags may overlap one another; none moves to avoid another.
- Pin marker: a round marker on the left margin, level with its anchor, showing its glyph. Its window stays closed until opened.
- Selection, proposal marks and proposal window as in the first amendment.

**Gallery demo counts** (these replace the first amendment's counts)
- Four participants, all present: four faces and four pointers with flags. One participant is typing.
- One selection, held over a paragraph.
- Two pending proposals.
- One pin marker, closed.
- One image, loaded, with a paragraph wrapping along its side.
- No participant is absent in the gallery. The absent state remains a host behaviour.

**Visible at rest**
- Four pointers and flags, one showing the typing state.
- Four faces, none absent; the header count with total spend; the bulk control reading two.
- Both proposal marks; no proposal window open.
- One closed pin marker; no pin window open; no participant followed.
- The view switch above the card, not inside it.

**Acceptance checks (continued)**
16. Four pointers render at rest, each with a flag bearing its participant name, activity line and spend tag. No participant name appears inside the document text.
17. The header shows four present faces and a count with total spend. In the gallery no face is in the absent state.
18. The view switch sits outside the card's border. Switching view replaces the page content and leaves the header row unchanged.
19. Exactly one pin marker is visible at rest, carrying its glyph, and no pin window is open.
20. No flag covers document text: the line a pointer sits on reserves room above it, and every word of the page stays readable at rest. Flags that overlap another flag stay where they are.
