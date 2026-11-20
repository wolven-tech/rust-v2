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
