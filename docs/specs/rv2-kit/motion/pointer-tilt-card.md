# Pointer Tilt Card

Source item: affectUI Pro `motion/tilt-card` (functional reference only; see README).

## Purpose
A card that leans toward the pointer as if it were a physical object held in the hand, with a soft reflection that follows the pointer, and that eases back to rest when the pointer leaves. Used for a feature or a membership card that invites a closer look.

## Data
- `children`: card content.
- `max_tilt_token`: "subtle" | "standard" | "strong".
- `glare` (boolean, default true).
- `touch_behaviour`: "none" | "tap_to_tilt".
- `extra_classes`.

## Anatomy
In reading order (back to front):
1. Card surface: the card's own background and border.
2. Content layer: the children.
3. Glare layer: a soft highlight whose centre follows the pointer.

## States
- Rest: card flat; glare hidden or at its edge.
- Hover: card leans toward the pointer; glare follows the pointer.
- Leaving: card springs back to rest; glare fades.
- Focus (keyboard): card shows the same lean as a centred hover, once, without pointer.
- Reduced motion: no lean; glare is a static highlight at the upper edge.

## Interactions
- Pointer move over the card: tilts toward the pointer by an amount that depends on distance from the centre.
- Pointer leave: returns to rest with a spring.
- Click: no behaviour of its own; content receives clicks.
- Keyboard focus on an interactive child: leans once as a focus cue.
- Touch: with `touch_behaviour` "none", no tilt; with "tap_to_tilt", a tap leans the card and a second tap or outside tap returns it.

## Motion (functional)
- Tilt follows the pointer with a short lag so it feels weighted.
- The glare moves opposite to the tilt, as if light reflected off a surface.
- Leaving springs back with a small overshoot-free settle.
- With reduced motion preferred, no tilt and no movement of the glare.

## Accessibility
- The card has no role of its own; interactive children keep their roles and names.
- Tilt and glare are decorative; `aria-hidden` on the glare layer.
- Nothing about the card's content depends on tilt to read.

## Responsive
- Tilt is limited to pointer devices; touch behaviour follows `touch_behaviour`.
- Phone width: card keeps its layout; no tilt by default.

## Composition hints
- Card container with a perspective wrapper, glare layer, spring-driven transform, pointer event listener.

## Acceptance checks
1. Moving the pointer to a corner tilts the card toward that corner; moving to the centre returns it to flat.
2. Leaving the card springs it back to rest.
3. The glare centre follows the pointer and sits opposite the tilt.
4. Interactive children inside the card receive clicks and keyboard focus as normal.
5. With reduced motion preferred, the card stays flat and the glare is static.
6. With `touch_behaviour` "none", a tap does not tilt the card.
7. At phone width the card keeps its layout and nothing scrolls sideways.

## Amendment 5: structure and demo

Archetype: one membership-style card, flat at rest, with a caption beneath.

Visible at rest:
- Card with its own content: a name line with a contactless mark at its top right; a chip graphic at the left under the name; a masked card number ending in four digits; two labelled fields, "Member" with the holder's name and "Since" with a month and year.
- Card background: the accent gradient token.
- Caption beneath the card in small text.

Gallery demo: name "Ledger Plus", holder "Jamie Rivera", since "03/24", number "•••• •••• •••• 2750", caption "Move the pointer over the card".

Motion, settled frame: card flat; glare hidden at its edge.
Moves: the card tilts toward the pointer; the glare follows the pointer and moves opposite the tilt; leaving springs the card back to rest.
Reduced motion: card stays flat; glare static at the upper edge.

Acceptance checks (continued):
8. At rest the card shows its name, chip, masked number and both labelled fields, with no placeholder text.
9. The caption is visible at rest.
10. While tilted, all card content stays inside the card's edges, with no text cut at a corner.
