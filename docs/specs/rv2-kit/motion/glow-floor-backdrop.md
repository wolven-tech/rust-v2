# Glow Floor Backdrop

Source item: affectUI Pro `motion/gradient-floor` (functional reference only; see README).

## Purpose
A soft, moving light that rises from the bottom edge of a section, used as a backdrop behind a hero or a feature area. It adds life without competing with the content above it.

## Data
- `children`: content rendered above the backdrop.
- `intensity`: "subtle" | "standard" | "strong".
- `anchor`: "bottom" (default) | "top".
- `palette_token`: name of a palette entry from the design system (the caller chooses it; the component does not carry values).
- `paused` (boolean).
- `extra_classes`.

## Anatomy
In reading order (back to front):
1. Base: the section's own surface.
2. Two glow layers: each a blurred band with an uneven shape, placed near the anchor edge.
3. Fade mask: the glow fades out before reaching the content's centre.
4. Content layer: the caller's children, above all glow.

## States
- Playing: both layers drift and change shape slowly.
- Paused: layers hold their current shape and stop moving.
- Reduced motion: layers hold a single shape; no drift.

## Interactions
- None; the backdrop does not receive pointer events; content above is fully interactive.

## Motion (functional)
- The two layers drift in different directions at different speeds so the light never repeats in a visible pattern.
- Each layer slowly morphs its shape; the change is gradual and never a sudden shift.
- Intensity changes ease over time when the caller changes `intensity`.
- With reduced motion preferred, the layers are static.

## Accessibility
- Decorative; `aria-hidden`.
- Must not reduce contrast of content above; the caller checks contrast of text against the glow's brightest point.

## Responsive
- Scales with the section; on narrow widths the glow height is reduced so it does not cover the content.
- Phone width: anchored to the bottom only; the fade mask is shorter.

## Composition hints
- Layered blurred shapes, mask, content wrapper, `pointer-events` pass-through.

## Acceptance checks
1. Content above the backdrop receives clicks and focus normally.
2. The two glow layers move at different speeds and do not share a visible loop.
3. Pausing holds both layers in place.
4. With reduced motion preferred, nothing moves.
5. Changing `intensity` eases the glow to its new strength.
6. At phone width the glow stays within the section and nothing scrolls sideways.
