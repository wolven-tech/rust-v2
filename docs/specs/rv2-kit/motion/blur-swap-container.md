# Blur Swap Container

Source item: affectUI Pro `motion/blur-crossfade` (functional reference only; see README).

## Purpose
A container that swaps its content for a new piece of content with a soft blur transition, while the container itself resizes smoothly to fit the new content. Used for steps, wizard panels and cards that change in place.

## Data
- `children`: the current content.
- `content_key`: any value; a change of key triggers the swap.
- `on_swap_complete()` (optional).
- `reduced_motion_override` (optional, for testing only).
- `extra_classes`.

## Anatomy
In reading order:
1. Outer frame: the container whose size animates to the current content.
2. Current content layer: the visible content.
3. Outgoing layer: the previous content, present only during the swap.

## States
- Stable: one content layer, frame sized to it.
- Swapping: outgoing layer blurs and fades; incoming layer blurs in and sharpens; frame changes size to the incoming content's size.
- Swap interrupted: a new key during a swap restarts the transition from the current visual state.

## Interactions
- None of its own; content inside it keeps its own interactions.
- Focus: when the content changes, focus moves to the first focusable element of the new content only if focus was inside the outgoing content.

## Motion (functional)
- Outgoing content blurs and fades while incoming content comes from a blur into focus.
- The frame glides to the incoming content's size in the same movement, so the frame and content feel like one motion.
- Only one swap runs at a time; a new key replaces the running swap.
- With reduced motion preferred, the content changes with no blur and no size glide.

## Accessibility
- Content changes are announced only if the container is a live region supplied by the caller.
- Focus is preserved or moved as described under interactions.
- Blur is decorative; the content is fully readable when the transition ends.

## Responsive
- Frame size follows the content at every width; no fixed height.
- Phone width: the same behaviour; the frame stays within the container's width.

## Composition hints
- Layout container, two stacked layers, size measurement of incoming content.

## Acceptance checks
1. Changing `content_key` shows the outgoing content blurring out and the incoming content settling in.
2. The frame's final size matches the incoming content's size.
3. A key change during a swap restarts the transition without leaving two layers visible at the end.
4. With reduced motion preferred, the content changes with no blur and no size glide.
5. If focus was inside the outgoing content, focus lands inside the incoming content after the swap.
6. At phone width the frame stays within the container and nothing scrolls sideways.

## Amendment 5: structure and demo

Archetype: a step card with a header, a body that swaps between steps, and a footer.

Visible at rest:
- Header row: card title, a status line, and a primary button at the right.
- Body panel: a status tag and one sentence. This is the region that swaps.
- Footer row: one dot per step at the left, the current step filled; a "Next" button at the right.
- Frame sized to the card. No second layer visible.

Gallery demo (three steps of different lengths):
- Header: title "Version 3.1", status line "2 changes · draft", primary button "Publish" (inert in the demo).
- Step 1: tag "Live", sentence "Offline mode now works on phones."
- Step 2: tag "Up next", sentence "Bulk export is ready for review. Check the file format before you publish."
- Step 3: tag "Done", sentence "Every change in this version has a named owner."
- Rest state: step 1 current. Next from step 3 returns to step 1.

Motion, settled frame: step 1 shown; frame height fits step 1.
Moves: Next blurs and fades the outgoing body. The incoming body sharpens in from a slight blur. The frame glides to the incoming height in the same movement.
Reduced motion: content changes with no blur and no glide.

Acceptance checks (continued):
7. At rest the card shows header, body and footer, with three step dots and step 1 current.
8. Next swaps the body to step 2 and the frame height changes to fit step 2's content.
9. Next pressed during a swap restarts it; when the swap ends, only one body layer exists.
