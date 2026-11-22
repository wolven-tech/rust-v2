# Flowing Gradient Text

Source item: affectUI Pro `motion/gradient-text` (functional reference only; see README).

## Purpose
A word or phrase whose letters are filled with a gradient that keeps flowing, and that sweeps a gradient across the text the first time it appears on screen. Used for a single emphasised phrase in a headline.

## Data
- `text`: the words to render.
- `emphasis_range`: the part of `text` that carries the gradient (default: the whole text).
- `replay_enabled` (boolean): shows a replay control.
- `gradient_token`: name of a gradient in the design system (the caller chooses it).
- `play_on_view` (boolean, default true).
- `extra_classes`.

## Anatomy
In reading order:
1. Text: each letter, filled with the gradient inside the emphasised range; plain text outside it.
2. Optional replay control under the text.

## States
- Waiting: before first view, the text shows its base fill.
- Sweeping: the first time the text enters view, the gradient sweeps across it once.
- Flowing: after the sweep, the gradient keeps moving through the letters slowly.
- Replaying: after the replay control is used, the sweep runs again from the start.
- Reduced motion: text shows the gradient as a fixed fill with no sweep or flow.

## Interactions
- Replay control: restarts the sweep.
- Pointer over the text: no effect.
- Focus on the replay control: shows its focus ring.

## Motion (functional)
- The sweep runs once on first view, from the start of the text to the end.
- Flow is continuous and slow; the gradient moves through the letters as a liquid would.
- Replay restarts the sweep only; it does not change the flow.
- With reduced motion preferred, the fixed gradient is shown with no sweep or flow.

## Accessibility
- The text is the accessible name; the gradient is decorative.
- The replay control is a button named "Replay animation".
- Gradient contrast against the background must meet the text contrast requirement; the caller chooses a gradient that does.

## Responsive
- Font size and line breaks follow the container; the emphasised range keeps its gradient across line breaks.
- Phone width: unchanged behaviour; text wraps.

## Composition hints
- Text span with a gradient fill (clip), intersection observer for first view, button (replay).

## Acceptance checks
1. The sweep runs once when the text first comes into view and not again while scrolling.
2. Replay restarts the sweep from the start.
3. The emphasised range carries the gradient; text outside it is plain.
4. With reduced motion preferred, the text shows the fixed gradient and no sweep or flow.
5. The replay control is reachable by keyboard and named.
6. At phone width the text wraps and nothing scrolls sideways.
