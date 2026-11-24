# Expandable Panel Strip

Source item: affectUI Pro `blocks/agent-gallery` (functional reference only; see README).

## Purpose
Shows a row of feature panels, one per item, where picking one opens it to reveal a larger preview of a product screen. Lets a visitor scan several capabilities without leaving the section.

## Data
- `panels`: list of { id, title, caption, preview: node }. `preview` is shown when the panel opens.
- `initial_open_id`: id of the panel open at first render; default is the first.
- `open_id`, `on_open_change(id | null)`: controlled selection. Passing `null` closes all.
- `preview_label`: accessible name for the open preview.
- `extra_classes`.

## Anatomy
In reading order:
1. Title and short intro line for the strip (optional, supplied by the parent section).
2. Panel row: every panel is a slim vertical tile showing its title (rotated when collapsed, horizontal when open) and its artwork region.
3. Open preview: the selected panel expands into the middle of the row. Its preview node sits in a single framed card with its caption under it.
4. Pager controls at the bottom: previous and next, shown when the row is narrow.

## States
- Default: first panel open.
- Collapsed panel: a slim tile with a de-emphasised artwork region and the title.
- Open panel: widened; preview shown; title shown in full.
- Hover on a collapsed panel: its artwork returns to full strength; the preview is not opened by hover alone.
- Focused panel: focus ring visible; the panel is not yet open.
- Narrow container: the open panel fills the width and one neighbour peeks on each side.
- Disabled: panels cannot be picked; the open preview stays.

## Interactions
- Click or tap on a collapsed panel: opens it and collapses the previous one.
- Click on the open panel: no change; a panel is closed with Escape.
- Hover: strengthens the artwork and reveals the caption under the tile.
- Keyboard: Tab moves onto the panel row, one stop per panel; Enter or Space opens the focused panel; Left and Right move focus between panels without opening them; Home and End jump to the ends; Escape closes the open panel and returns focus to its tile.
- Pager: Previous and Next move the open selection one step; they wrap at the ends.
- Touch: swipe left or right moves the open selection.

## Motion (functional)
- Opening a panel widens it into the middle while the previous panel narrows to a slim tile: one width transition covers both panels.
- The preview content fades in after its panel has finished opening.
- Collapsed panels show their artwork de-emphasised; hover restores full emphasis.
- With reduced motion preferred, a panel change is instant and the preview appears without a fade.

## Accessibility
- The row is a tab list; each panel is a tab with the panel title as its name and a selected state.
- The open preview is the tab panel, labelled by the selected tab.
- Collapsed tiles keep their title in the accessible name even when the visual title is rotated.
- Focus ring is visible on every tile and on pager buttons.

## Responsive
- Wide containers: open panel takes the centre; collapsed tiles sit on either side.
- Narrow containers: the open panel takes the full width and one neighbour tile is visible on each side; swipe or pager moves the selection.
- Phone width: the pager is the primary control; tiles stay tappable with enlarged hit areas.

## Composition hints
- Tabs (the tile row as a tablist), card (the preview frame), button (pager), a motion wrapper for width and fade.

## Acceptance checks
1. Clicking a collapsed panel opens it and collapses the previously open one in the same transition.
2. Keyboard focus moves across tiles with Left and Right without changing the open panel.
3. Enter on a focused tile opens it; Escape returns focus to that tile.
4. The open tile's accessible name is its full title, even when its visual title is shown rotated.
5. Hovering a collapsed tile strengthens its artwork and does not change the open panel.
6. At narrow width, exactly one neighbour tile is visible per side and nothing scrolls horizontally.
7. Pager Next on the last panel wraps to the first.
8. With reduced motion preferred, opening a panel does not animate its width.

## Amendment 5: structure and demo

Archetype: centred heading over a strip of seven panel tiles, one open as a preview card, with a caption row and pager beneath.

Visible at rest:
- Section heading, centred, with one short line beneath it.
- Seven tiles in one row: three collapsed tiles, the open tile, then three collapsed tiles. The open tile is the fourth.
- Collapsed tile: artwork region de-emphasised; its title set vertically.
- Open preview: one framed card with a header bar (agent swatch, a name selector showing the agent name, a "New agent" button, owner initials); a title and meta line; a trigger line; four figures with small trend lines (Runs, Success, Hours saved, Escalated); four tabs with Context selected; an "Agent context" section with one help line, a search field with a filter button, two context items (initials and text), and a composer row with an "Add context" button.
- Caption row beneath the frame: agent name in bold, one sentence, and previous and next controls at the right, inside the frame.

Gallery demo:
- Heading "Specialist agents, one at a time". Line: "Each agent has its own steps, tools and limits."
- Tiles in order: Ticket triage, Lead finder, Call notes, Renewal watch (open), Onboarding guide, Copy check, Changelog writer.
- Open card: title "Renewal watch", meta "Updated 3 min ago", trigger "Starts when a plan has 30 days left · Owner Sam Ortega", figures Runs 126, Success 94%, Hours saved 2.8, Escalated 9.
- Context items: "Renewals above the plan threshold go to the account owner first." (SO); "Sam added the regional price list on 2 Nov." (PN).
- Caption: "Renewal watch. Flags quiet accounts before their contract ends and drafts the first call."

Motion, settled frame: open preview with content; neighbours collapsed.
Moves: opening widens the new panel and narrows the previous one in one transition. The preview content fades in after its panel has opened. Hovering a collapsed tile strengthens its artwork without opening it.
Reduced motion: panel change is instant; the preview appears without a fade.

Acceptance checks (continued):
9. Seven tiles at rest: three collapsed tiles on each side of the open one.
10. Every part of the open preview has content: header, title, four figures, four tabs, context items and composer row. No part is an empty region.
11. Previous and next controls sit inside the frame at every width; Next on the last tile wraps to the first.
