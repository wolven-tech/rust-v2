# Frosted Folder Mark

Source item: affectUI Pro `illustrations/glass-folder` (functional reference only; see README).

## Purpose
A small decorative mark for a card that stands for "a collection of documents". On hover it suggests that the folder is open and its papers are visible.

## Data
- `label` (optional, for assistive text only).
- `open_on_hover` (boolean, default true); `open_when_focused` (boolean, default true).
- `size_token`: one of the design system's size steps.
- `extra_classes`.

## Anatomy
In reading order (front to back):
1. Back panel of the folder: a rounded shape behind the papers.
2. Papers: a stack of three sheets peeking above the front.
3. Front pocket: a frosted, translucent panel that covers the lower part of the papers.
4. Tab on the back panel.

## States
- Rest: front pocket closed over the papers; papers show only their top edges.
- Hover or focus: front pocket tips open at its top edge; papers rise slightly.
- Disabled: rest state, not interactive.

## Interactions
- Pointer hover: opens the pocket; leaving closes it.
- Keyboard focus, when the mark is inside a focusable card: opens the pocket the same way.
- No click behaviour of its own; the parent card owns clicks.

## Motion (functional)
- The pocket tips open on hover as if a person lifted its top edge; papers slide up through the gap.
- On leaving, the pocket returns with a settle, papers drop back behind it.
- The motion is one short movement, not a loop.
- With reduced motion preferred, the open and closed states switch with no movement.

## Accessibility
- Decorative by default; `aria-hidden` unless `label` is given.
- With `label`, it is an image with that name.
- It never carries information the parent card does not also state.

## Responsive
- Scales with its container using `size_token`; no separate phone layout.

## Composition hints
- Image or illustration container, layered shapes; no interactive primitives.

## Acceptance checks
1. With no pointer over it, the pocket covers the papers.
2. Hovering the mark opens the pocket and papers become visible; leaving closes it.
3. With reduced motion preferred, the open state appears without intermediate frames.
4. With `label` set, the mark exposes that name; without it, it is hidden from assistive technology.
5. Changing `size_token` changes its rendered size without clipping the papers.

Uses no partner logos.

## Amendment 5: structure and demo

Archetype: one folder mark, centred in its box.

Visible at rest:
- One folder: body at full strength of the accent token, with a tab at the top left of the back panel.
- Three sheets rising from the top of the body.
- Front pocket: translucent with a backdrop blur, covering the lower part of the sheets. Sheet edges show through it.
- Soft shadow beneath the folder, in the accent token.
- Clear margin inside the box on all sides.

Gallery demo: one mark at the default size token, with the line "Hover the folder to open it" beneath.

Motion, settled frame: pocket closed; sheet edges visible.
Moves: on hover the pocket's top edge lifts and the sheets rise. On leave the pocket settles back and the sheets drop. One movement, no loop.
Reduced motion: open and closed states switch with no movement.

Acceptance checks (continued):
6. At rest exactly one folder is drawn, with three sheets, a tab and a translucent front pocket.
7. Sheet edges are visible through the front pocket at rest.
8. At every size token the folder, tab, sheets and shadow sit inside the box with no edge cropping them.

## Amendment 6: structure and demo

Replaces Amendment 5:
- "Gallery demo", caption "Hover the folder to open it" beneath the mark: no caption is drawn inside the stage at rest. A caption, if used, sits below the stage.

Archetype: one folder mark, centred, filling about a quarter of the stage width.

Visible at rest, as Amendment 5 with these additions:
- The stage holds only the folder. Nothing else is drawn.

Scale: at the default size token the folder is about a quarter of the stage width. Its horizontal centre is the stage's centre. Its vertical centre lies between 45% and 60% of the stage height.

Motion, settled frame: pocket closed; sheet edges visible.
Moves and reduced motion: as Amendment 5.

Defects, stated as checks: a small icon in a corner that reads as a neutral tray (checks 9 and 10).

Acceptance checks (continued):
9. The folder body uses the accent token at full strength. It does not read as a neutral tray.
10. The folder is centred horizontally, about a quarter of the stage width, and its vertical centre lies between 45% and 60% of the stage height.

## Amendment 7: structure and demo

Replaces: Amendment 6 "Scale" paragraph. Amendment 5 and Amendment 6 stand otherwise; checks 9 and 10 stand; checks 11 to 14 are new.

Archetype: one folder mark, centred on an empty stage.

Stage: width to height 5 to 3. The stage fills the container width and its height follows. Surface: the neutral surface token.

Folder geometry, as fractions of stage width (W) and stage height (H):
- Folder box, tab top to body bottom: 25% of W wide and 36% of H tall. Horizontal centre at 50% of W. Top at 33% of H, bottom at 69% of H.
- Tab: at the top-left of the body, about 38% of body width and 9% of folder height. Its right end slopes down into the body's top edge.
- Body (back panel): the accent token at full strength. It is the largest saturated shape in the stage. Corner radius about 8% of body height.
- Sheets: three sheets on the light surface token, each 60% to 75% of body width, offset from one another by about 4% of body width, centred on the body. Their top edges sit at 12% of folder height below the tab top. The front pocket hides the rest of them.
- Front pocket: full body width, top edge at 21% of folder height below the tab top, running to the body bottom with the same corner radius. Translucent and frosted: lighter than the body at its top, slightly deeper at its bottom edge, with a thin highlight along its top edge.
- Shadow: soft, in the accent token, beneath the body, reaching about 6% of H below the folder bottom, centred under the body, faint.

Visible at rest: exactly one folder with a tab, a body, three sheets and one pocket. Between the sheet tops and the pocket's top edge, about 9% of folder height of sheet edges shows. Nothing else is drawn in the stage. No caption inside the stage.

Tone, relative to the stage: the stage is a light neutral. The body is the strongest saturated mass and is clearly deeper than the pocket and the sheets. The mark has no neutral dark shape.

Hover and focus: the pocket's top edge lifts by about 6% of folder height and the sheets rise by the same amount. Leaving returns both. Reduced motion: the two states switch with no intermediate frame.

Acceptance checks (continued):
11. The stage is 5 to 3. The body is 25% of stage width wide, within 3 points.
12. The folder centre is at 50% of W horizontally and between 50% and 52% of H vertically.
13. Sheet tops sit at 12% of folder height and the pocket top at 21%. At rest about 9% of folder height of sheet edges shows above the pocket.
14. The body uses the accent token at full strength and the mark has no neutral dark shape. It does not read as a neutral tray.
