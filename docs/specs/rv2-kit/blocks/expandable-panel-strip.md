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
