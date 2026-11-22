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
