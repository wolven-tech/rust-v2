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

## Amendment 5: structure and demo

Archetype: a framed stage with a bottom-anchored glow floor behind a heading and one button.

Visible at rest:
- Stage: rounded panel. Heading and one button in its upper half, above the glow.
- Two glow layers: a layer that runs through three gradient stops across the width, and a lighter layer over it.
- The glow rises from the bottom edge. Its tallest peak reaches about half the stage height. No light shows above the upper third.
- Peaks and valleys uneven across the width.

Gallery demo:
- Heading "Launch checklist", button "Start the checklist".
- Demo controls above the stage: "Pause", "Subtle", "Standard", "Strong".

Motion, settled frame: the two layers at different peak positions.
Moves: the two layers move sideways in step, and the lighter one moves less. Shapes morph gradually. Intensity changes ease to the new strength.
Reduced motion: layers static.

Acceptance checks (continued):
7. At rest the glow reaches at least half-way up the stage at its tallest peak, and no light shows above the upper third.
8. The two layers differ in peak position and shape at every moment.
9. The heading and button meet the text contrast requirement at the glow's brightest point.

## Amendment 6: structure and demo

Supersedes Amendment 5 where they differ. Replaces:
- "Visible at rest", stage bullet: the stage holds no content at rest. No heading and no button.
- "Gallery demo": the heading, the button and the demo controls are removed from the stage. If a demo control row is shown, it sits outside the stage.
- Check 9: applies to any content a caller places above the glow. The gallery has none.

Archetype: a rounded stage filled by a bottom-anchored glow floor, with nothing inside it at rest.

Visible at rest:
- Rounded stage, empty.
- Two glow layers. The first runs through three gradient stops across the width. The second is lighter and sits over it.
- The glow rises from the bottom edge. Its tallest peak reaches about 55% of the stage height.
- No light above the upper third at any point, including the sides.
- Peaks and valleys uneven across the width; the two layers peak in different places.
- The glow spans the full stage width.

Motion, settled frame: the two layers at different peak positions, at the heights above.
Moves: the two layers move sideways in step, the lighter one moves less. Shapes morph gradually. Intensity changes ease to the new strength.
Reduced motion: layers static.

Defects, stated as checks: a content card over a faint grey glow confined to the bottom third (checks 10 and 11).

Acceptance checks (continued):
10. At rest the tallest peak reaches at least half the stage height. The glow shows a visible colour at its peak, not a grey wash.
11. The gallery stage holds no card, heading or button at rest, and the glow spans the stage width.
12. Content a caller places above the glow meets the text contrast requirement at the glow's brightest point.

## Amendment 7: structure and demo

Replaces: Amendment 6 "Visible at rest" bullets "Two glow layers", "The glow rises from the bottom edge" and "Peaks and valleys uneven across the width" (the layers are now built from blobs, placed below). Amendment 6 "Motion, settled frame", "Moves" and "Reduced motion" stand. Amendment 6 checks 10 to 12 stand; checks 13 to 17 are new.

Archetype: a rounded stage filled by several soft glow blobs rising from its bottom edge. Nothing inside it.

Stage geometry: the card (neutral surface, hairline border) holds the stage. The stage is 90% of card width and 70% of card height, centred, aspect about 2.2 to 1, rounded corners, hairline edge, neutral surface base.

Glow: two layers, built from six soft blobs. Each blob is a blurred ellipse with no hard edge. Positions: x is across the stage (0 at left, 1 at right); height is above the stage bottom as a fraction of stage height.
- Layer one, blob A: centre x 0.03, peak height 0.58. Warm palette token. The tallest shape.
- Layer one, blob B: centre x 0.10, peak height 0.14. Light, near the bottom-left corner.
- Layer one, blob C: centre x 0.30, peak height 0.18. Cool palette token.
- Layer one, blob D: spans x 0.45 to 1.00. Top edge around height 0.40 across x 0.60 to 0.80. Second accent token. The broad band.
- Layer two, blob E: centre x 0.52, peak height 0.10. Very light, near the bottom.
- Layer two, blob F: centre x 0.86, peak height 0.22. Cool palette token, lighter than blob D.

Coverage: coloured light reaches the upper half of the stage only in the left fifth, where it peaks at 55% to 60% of stage height. Across the middle the coloured area stops at about 40% of stage height. At the right edge it stops at about 30%. Roughly the lower 40% of the stage holds colour; the upper third is empty at every x.

Peaks: the highest peak is at x 0.03 (layer one); layer two peaks at x 0.86. The two layers never peak at the same x.

Motion, settled frame: the blobs in the positions above. Both layers drift sideways in step, layer two less. Shapes morph slowly. Intensity changes ease to the new strength. Reduced motion: layers static.

Acceptance checks (continued):
13. The stage is 90% of card width and 70% of card height, aspect about 2.2 to 1, centred on the card.
14. The glow has six blobs in two layers. The highest peak lies in the left fifth of the stage, at 55% to 60% of stage height.
15. Across the middle the coloured area stops at about 40% of stage height; at the right edge at about 30%.
16. The glow's top edge is not one band: its height varies by at least 25 points of stage height across the width.
17. No glow in the upper third of the stage at any x.

## Amendment 8: structure and demo

Replaces, from Amendment 7: the "Glow" paragraph and its six-blob list; the "Peaks" paragraph; the "Motion, settled frame" paragraph; checks 14 and 16. Amendment 7 "Stage geometry" and check 13 stand. Check 15 stands except its right-edge figure, which is replaced below. Check 17 stands.

**Archetype:** a rounded empty stage with one continuous soft wash rising from its bottom edge. The wash is a single field. No shapes are built into it.

**Stage:** as Amendment 7 "Stage geometry": about 90% of card width and 70% of card height, centred, aspect about 2.2 to 1, rounded, hairline edge, neutral base. Nothing inside the stage at rest.

**What the eye reads at rest:**
- One wash across the full stage width, from the left edge to the right edge. It reads as one soft floor of light rising from the bottom edge. No individual shape is visible in it.
- Top boundary: a soft gradient with no edge line. Measured as height above the stage's bottom edge, as a fraction of stage height: highest at the left fifth, about 0.55 to 0.60; falling through the middle to about 0.40 at the centre; about 0.40 to 0.45 at the right edge.
- Strength: most saturated across the middle half, in the lower third (up to about 0.30 of stage height). The two bottom corners carry the lightest highlights; the left corner is the brighter of the two.
- Two layers, overlaid: layer one is the saturated field above, using the warm and cool palette tokens and the second accent token. Layer two is a lighter overlay in the bottom half, brightest near the bottom centre (x about 0.50) and fading upward. Both span the full width and blend into one wash.
- Coverage: no colour above 0.67 of stage height (the upper third) at any x. Between 0.60 and 0.67 only faint haze, and only in the left fifth.

**Must not appear:**
- separate humps, arches, domes or blob silhouettes, or any shape with its own crest;
- a gap where the base surface shows between two coloured areas, at any x;
- a hard edge, outline, seam or boundary line on any part of the wash, the top boundary included;
- a flat, horizontal top line (a clipped band);
- a single centred bloom or radial spot;
- strong areas that read muted or washed out; where the wash is strong it is saturated;
- colour in the upper third of the stage;
- a visible repeat or loop within one frame.

**Motion, settled frame:** the wash in the positions above. Both layers drift sideways in step; layer two moves less. The top boundary morphs slowly, with no sudden shift. Intensity changes ease to the new strength. Reduced motion: layers static.

**Defects, stated as checks:** the current build shows separate arch-shaped humps with gaps between them, confined to the lower third. Checks 19, 20 and 22 fail on that.

**Acceptance checks (continued):**
18. Stage geometry as check 13: about 90% of card width, about 70% of card height, centred.
19. At rest, every column across the full stage width is coloured from the bottom edge up to the top boundary. No gap shows the base surface.
20. The top boundary has one highest point, in the left fifth, at 0.55 to 0.60 of stage height. No other point of the top boundary is higher than 0.45 outside the left fifth.
21. The strongest colour lies in the lower third across the middle. The two bottom corners carry the lightest highlights.
22. No hard edge, outline, seam or flat top line is visible on the wash. The only edge is the stage's rounded clip.
23. No colour above the upper third at any x. Haze above 0.60 appears only in the left fifth.
24. The top boundary varies by at least 15 points of stage height across the width. (Replaces Amendment 7 check 16, which asked for 25.)
25. The glow is one field in two overlaid layers, its highest point in the left fifth at 0.55 to 0.60. (Replaces Amendment 7 check 14, which asked for six blobs.)
26. Amendment 7 check 15, right-edge figure: 0.40 to 0.45 of stage height, not 0.30. The middle figure stands.
