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
