# Voice Assistant Orb

Source item: affectUI Pro `illustrations/voice-orb` (functional reference only; see README).

## Purpose
A dotted globe that shows the assistant's mode: it breathes at rest, responds to sound while listening, contracts into a spinning ring while thinking, and emits ripples while it speaks.

## Data
- `state`: "idle" | "listening" | "thinking" | "speaking".
- `level`: number from 0 to 1 for input or output loudness.
- `label` (optional, shown under the orb with a status word).
- `extra_classes`.

## Anatomy
In reading order:
1. Dot sphere: a field of small points arranged on a sphere; points are denser near the front.
2. Ring form: the same points collapse onto a circular path when thinking.
3. Ripple rings: concentric outlines expanding from the sphere when speaking.
4. Status word under the orb.
5. State switch (optional, demonstration only): four segments to set the state.

## States
- Idle: sphere breathes slowly; points drift a little.
- Listening: points respond to `level`, shifting outward with louder input.
- Thinking: points collapse into a ring that spins; the status word reads thinking.
- Speaking: ripple rings expand from the sphere in step with `level`.
- Reduced motion: a static sphere with the status word; no collapse or ripples.

## Interactions
- Decorative by default.
- Interactive mode: the orb is a button that toggles listening when pressed; it is a button with its state as text.

## Motion (functional)
- Breathing is slow, low amplitude, and never stops in idle.
- Listening responds to loudness immediately: louder input pushes points out.
- Collapsing into the ring is one continuous movement; the ring spins while thinking.
- Ripples leave the orb and fade with distance while speaking.
- With reduced motion preferred, every state is a still image with its status word.

## Accessibility
- Interactive: the orb is a button named "Voice assistant" with the state as its value.
- Decorative: `aria-hidden`; the status word is text in the region.
- State changes are announced through the status region, not on every frame.

## Responsive
- Scales with its container; the status word drops below the orb at narrow widths.
- Phone width: same stack at a smaller size; the demonstration switch is hidden.

## Composition hints
- Canvas or drawn dot field, status text, button (interactive), segmented control (demonstration).

## Acceptance checks
1. Changing `state` to thinking collapses the dots into a spinning ring.
2. Raising `level` in listening state pushes the points outward.
3. Speaking shows ripples that fade with distance from the orb.
4. Idle state keeps breathing; reduced motion makes it still.
5. Interactive orb toggles listening on press and announces the new state once.
6. At phone width the orb scales and nothing scrolls sideways.

Uses no partner logos.

## Amendment 6: structure and demo

No earlier amendment exists, so no line is replaced.

Archetype: one dotted sphere, its status word, and a four-segment demonstration switch.

Visible at rest:
- One orb in the listening state: dots on a sphere, denser at the front, pushed slightly outward at a moderate level.
- Status word beneath the orb: "Listening…".
- Four-segment switch beneath the status word: Idle, Listening, Thinking, Speaking. Listening is selected and shown as a raised segment. The switch is demonstration only.
- No ring and no ripple rings.

Scale and arrangement: the sphere is about a sixth of the stage width, centred horizontally, its centre at about 42% of the stage height. The status word sits at about 67%. The switch sits at about 78%.

Motion, settled frame: listening sphere at a moderate level, no ripple.
Moves and reduced motion: as stated in Motion (functional).

Defects, stated as checks: two Idle orbs and no listening state at rest (check 7).

Acceptance checks (continued):
7. At rest exactly one orb is drawn, in the listening state, with the status word "Listening…".
8. The four-segment switch sits under the status word with Listening selected. A consumer that renders the orb alone gets no switch.
9. The dots form one sphere at rest: no ring and no ripple rings are drawn.

## Amendment 7: structure and demo

Replaces: Amendment 6 "Scale and arrangement" paragraph, and Amendment 6 "Visible at rest" first bullet (dots "pushed slightly outward" is kept as the settled listening frame only). Amendment 6 checks 7 to 9 stand; checks 10 to 12 are new.

Archetype: one dot sphere above a status word above a four-segment switch, on an empty stage.

Stage: the card, neutral surface, edge to edge. Width to height about 1.85 to 1.

Sphere:
- Diameter 17% of stage width. Centre at 50% of W horizontally and 41% of H vertically.
- Dots form a regular lattice: about 15 latitude rings from pole to pole, each ring spaced evenly around the sphere. A visible hemisphere holds roughly 150 to 200 dots.
- All dots the same size, in the ink token. On screen the spacing shrinks toward the rim so the outline reads as a clean circle; rim dots are paler.
- No random scatter and no jitter at rest.
- Listening, settled frame: the sphere at a moderate level, dots pushed outward by about 3% of the diameter. No ring and no ripple.

Status word: centred at 67% of H. Small, muted, ink token. Reads "Listening…".

Switch: a segmented track centred at 77.5% of H, 33% of W wide, about 7% of H tall. Four equal segments: Idle, Listening, Thinking, Speaking. Listening is the raised segment, a light surface pill inside the track. The others are plain text. Demonstration only.

Acceptance checks (continued):
10. One sphere in lattice form: about 15 rings, 150 to 200 dots on the visible hemisphere, all the same size. No random scatter.
11. Sphere diameter is 16% to 18% of stage width. Its centre is at 41% of stage height, within 2 points.
12. The status word is centred at 67% of stage height. The switch is 33% of stage width (within 3 points) with its centre at 77% to 78% of stage height, and Listening is its raised segment.
