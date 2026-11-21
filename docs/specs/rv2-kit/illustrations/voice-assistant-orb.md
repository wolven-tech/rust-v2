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
