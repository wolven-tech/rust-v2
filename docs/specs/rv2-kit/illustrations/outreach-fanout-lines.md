# Outreach Fanout Lines

Source item: affectUI Pro `illustrations/outreach-branches` (functional reference only; see README).

## Purpose
A card illustration showing one outbound source branching to three destination tools, with pulses travelling down the branches. It conveys "one message goes out to several channels".

## Data
- `source`: { label, icon: node (supplied by caller) }.
- `destinations`: list of three { id, label, icon: node (supplied by caller) }. The layout supports exactly three.
- `active_destination_id` (optional).
- `extra_classes`.

## Anatomy
In reading order (top to bottom):
1. Trunk: a line rising from the source to the branch point.
2. Branch lines: three curves from the branch point to each destination tile.
3. Destination tiles: rounded tiles holding each destination's icon.
4. Pulses: small marks that travel down each branch.
5. Source tile: at the top of the trunk.

## States
- Live: pulses travel down each branch in turn.
- Active destination: its branch is drawn fuller and its tile is raised.
- Reduced motion: static branches and tiles.

## Interactions
- Hover a destination tile: highlights its branch and tile.
- No click behaviour by default; interactive mode may make tiles buttons calling the parent.

## Motion (functional)
- Pulses leave the source one after another and fan out, each following its own branch.
- A destination tile lifts briefly when a pulse arrives at it.
- Branches draw in on first view.
- With reduced motion preferred, branches draw in as a single frame and no pulses are drawn.

## Accessibility
- Decorative in default mode; `aria-hidden`.
- In interactive mode, destination tiles are buttons with their label as name.

## Responsive
- Scales with its container; branch spread narrows at narrow widths.
- Phone width: unchanged layout at a smaller size.

## Composition hints
- Card container, icon tiles, path drawing for branches, marker animation for pulses.

## Acceptance checks
1. Pulses travel from the source down each of the three branches.
2. Hovering a destination highlights its branch and tile.
3. With reduced motion preferred, no pulses are drawn and branches are static.
4. The layout shows exactly three destinations; a fourth is not rendered.
5. At phone width the scene scales down and nothing scrolls sideways.

Uses partner logos (supplied by caller) for the source and destination icons. Does not describe any brand mark.

## Amendment 5: structure and demo

Archetype: one junction point above three destination tiles in a row, with a curve dropping from the point to each tile.

Replaces Anatomy items 1 (trunk) and 5 (source tile). The junction is a bare point at the top; the tiles sit below it.

Visible at rest:
- Bordered card holding the scene.
- Junction: a small point near the top centre, with no box and no caption.
- Three curves from the junction down to the top edge of each tile. The centre curve is straight; the two outer curves bend outward.
- Three tiles in one row, each holding only its icon. No captions.
- Centre tile raised above the other two, with a soft light beneath it.
- A pulse in flight on each curve.

Gallery demo: the icons are generic kit glyphs (an envelope, a speech bubble, a document). Callers supply their own.

Motion, settled frame: pulses at mid curve on all three; centre tile raised.
Moves: pulses leave the junction one after another, each following its own curve. A tile lifts briefly as a pulse arrives. Curves draw in on first view.
Reduced motion: curves drawn as one static frame; no pulses.

Acceptance checks (continued):
6. The junction is a bare point with no box and no caption.
7. Three curves run from the junction to the top edge of each tile; the centre curve is straight.
8. At rest each tile shows an icon only.
9. The centre tile sits raised, with a soft light beneath it.

## Amendment 6: structure and demo

No Amendment 5 line is overridden. This amendment adds scale and single-scene rules.

Archetype: one junction and three icon tiles in one row, drawn once per card.

Visible at rest:
- One scene only: one junction, three curves, three tiles. No second copy, no panel, no chip set.
- Junction: a bare point in the upper third, centred horizontally.
- Curves: the centre curve is straight; the outer curves bend outward. Each ends at the top edge of its tile.
- Tiles: one row, each holding only its icon.
- Centre tile raised by about a sixth of a tile's height, with a soft light beneath it.

Scale: the tile row spans about two fifths of the stage width, centred. Each tile is about a tenth of the stage width. The row sits in the lower half of the stage, below the junction.

Gallery demo: as Amendment 5 (envelope, speech bubble, document).

Motion, settled frame: pulses at mid curve on all three; centre tile raised.
Moves and reduced motion: as Amendment 5.

Defects, stated as checks: two small dot-tree panels with tiny icon chips, duplicated (checks 10 and 11).

Acceptance checks (continued):
10. The stage draws exactly one junction, three curves and three tiles.
11. The tile row spans at least a third of the stage width, and each tile is at least a tenth of it.
12. Each tile holds only its icon. The junction carries no label.

## Amendment 7: structure and demo

Replaces: Amendment 6 "Scale" paragraph (tile sizes and row position), and the Amendment 6 "Visible at rest" bullet "Centre tile raised by about a sixth of a tile's height". Amendment 5 and 6 otherwise stand; Amendment 6 checks 10 to 12 stand. Checks 13 to 16 are new.

Archetype: one junction point above a row of three square icon tiles, joined by three curves, drawn on a bordered card.

Stage: the bordered card is the stage. Its border is a hairline in the neutral token. No inner panel, inner border, tile backing or caption.

Junction: a bare point at 50% of W horizontally and 29% of H vertically. No more than 1% of W across. No box, no label.

Curves: three, one thin line each in the neutral token.
- Centre curve: straight and vertical, from the junction down to the top centre of the centre tile.
- Outer curves: leave the junction heading outward and slightly down, then turn to arrive vertically at the top centre of their tile.

Tiles: one row, each holding only its icon, the icon about half the tile's side.
- Side tiles: square, side 9.6% of W. Centres at 34.7% and 65.3% of W. Top edge at 52.9% of H, bottom edge at 68.7% of H.
- Centre tile: square, side 11.2% of W, centred at 50% of W. Its bottom edge shares the side tiles' bottom edge at 68.7% of H. Its top edge is at 49.8% of H, so it rises above the others by growing upward from the shared baseline.
- The row spans 40% of W, centred.
- Under the centre tile, a soft light in the accent token, reaching about 6% of H below the shared baseline.

Pulses: one on each curve, at mid-curve, at rest. Each travels from the junction to its tile. A tile lifts briefly as its pulse arrives. Curves draw in on first view. Reduced motion: curves as one static frame, no pulses.

Gallery icons: an envelope (left), a speech bubble (centre), a document (right). Callers supply their own.

Acceptance checks (continued):
13. The bordered card is the stage. No inner panel, inner border or caption sits inside it.
14. The junction is a point no wider than 1% of W, at 50% of W and 29% of H.
15. The tile row spans 40% of W (within 3 points) and is centred. Side tiles are 9.6% of W wide; the centre tile is 11.2% of W with its bottom edge on the side tiles' baseline.
16. Exactly three curves. The centre curve is straight; both outer curves arrive vertically at the top centre of their tile.
