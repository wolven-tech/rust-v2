# Dictation Capture Panel

Source item: affectUI Pro `illustrations/voice-capture` (functional reference only; see README).

## Purpose
A panel shown while a person dictates. It shows that the microphone is listening, how loud the speech is, how long they have been speaking, and offers the two ways out: finish or cancel.

## Data
- `state`: "listening" | "paused" | "idle".
- `level_samples`: a rolling list of loudness values from 0 to 1, newest last.
- `elapsed_secs` (the panel shows it as mm:ss).
- `on_finish()`, `on_cancel()`.
- `finish_label`, `cancel_label`, `shortcut_hints` (node per action, optional).
- `extra_classes`.

## Anatomy
In reading order:
1. Waveform well: a recessed band with thin bars, one per sample, rising and falling with loudness.
2. Status row: a recording dot with the elapsed time, a divider, the finish action with its shortcut hint, and the cancel action with its shortcut hint.
3. Glow beneath the panel: a soft band whose light follows loudness.

## States
- Listening: bars update with each loudness sample; time counts up; dot is live.
- Paused: bars stop moving; time stops; dot is static; finish label stays.
- Idle: panel shows zero bars and zero time with a start prompt.
- Finishing: finish action shows progress until `on_finish` resolves.

## Interactions
- Finish: calls `on_finish`.
- Cancel: calls `on_cancel` and discards the samples.
- Keyboard: Space finishes (when focus is in the panel) and Escape cancels, using the shortcut hints shown.
- Pointer hover on the waveform: no effect.

## Motion (functional)
- Bars rise and fall with each new sample; the newest bar enters at the right and older bars move left.
- The elapsed-time text updates as time passes and is the only element that counts.
- The glow brightens with louder input and eases back with quiet input.
- With reduced motion preferred, the waveform shows its latest values in a steady state without scrolling; time still updates.

## Accessibility
- The panel is a labelled region named "Dictation".
- The recording dot is decorative; the status is text ("Listening", "Paused").
- Finish and cancel are buttons with their shortcut in the accessible description.
- Level changes are not announced; the state change is.

## Responsive
- Width follows the container; the waveform keeps full width.
- Phone width: shortcut hints are hidden; buttons enlarge to touch size.

## Composition hints
- Panel (card), waveform (bar graphic), status text, button pair, glow.

## Acceptance checks
1. In listening state, waveform bars change with new samples and the time increases.
2. Pausing stops the waveform and the time and shows paused text.
3. Finish calls `on_finish` and cancel calls `on_cancel` without calling the other.
4. Escape cancels; Space finishes when focus is in the panel.
5. With reduced motion preferred, the waveform does not scroll.
6. At phone width shortcut hints are hidden and nothing scrolls sideways.

Uses no partner logos.

## Amendment 5: structure and demo

Archetype: one card with a waveform well above a single status row, with soft light beneath it.

Visible at rest:
- Waveform well across the card's full width: 36 to 48 thin bars, one per sample, centred vertically. The newest bar is at the right; older bars shrink toward the left edge.
- Status row beneath the well, one horizontal line: recording dot, elapsed time as m:ss, a vertical divider, "Done" with a Space keycap, "Cancel" with an Escape keycap.
- Nothing above the waveform: no tabs, titles or state labels.
- Soft light beneath the card's lower edge only, in two layers, one under each half of the card.

Gallery demo:
- Elapsed time "0:42". Samples: one fixed list of 40 values.
- Status text ("Listening") stays in the accessible name only.

Composition:
- Card wide and short. Time sits at the left of the status row; the two actions follow it on the same line.
- The card may run off the stage's right edge with a fade. No other edge is cropped.

Motion, settled frame: waveform frozen on the fixed samples; dot visible; time fixed; light beneath at mid strength.
Moves: while listening, bars rise and fall with each new sample and the newest enters at the right. The dot breathes, the time advances, and the light brightens with louder input.
Reduced motion: waveform holds its latest values without scrolling; time still updates.

Acceptance checks (continued):
7. The status row is one horizontal line holding the time and both actions; nothing else sits above or beside the waveform.
8. Bars are thin, one per sample, and the newest sits at the right edge of the well.
9. The light sits only beneath the card, in two layers, one under each half.
10. The only cropping at the stage edge is the right-hand fade.
