# Keyboard Prompt Input

Source item: affectUI Pro `illustrations/glow-keyboard` (functional reference only; see README).

## Purpose
A decorative scene for an AI chat input: a prompt field that types itself, above a stylised keyboard. Every key press sends a wave through a glowing floor under the keyboard, so the scene reads as a live chat.

## Data
- `sample_prompts`: list of strings typed in turn when no `value` is supplied.
- `value` (optional, controlled text); `on_submit(text)` when the scene's field is submitted.
- `typing_speed_mode`: "steady" | "bursty".
- `paused` (boolean).
- `extra_classes`.

## Anatomy
In reading order:
1. Prompt field: a pill-shaped input with a small status mark at its left and its text at the centre.
2. Keyboard: a row arrangement of keys (function row, number row, letter rows, modifier row, space row), drawn as a flat set.
3. Glow floor: a soft band under the keyboard whose light moves outward from each pressed key.
4. Glow ring around the prompt field, brightest while typing.

## States
- Idle: field shows the placeholder; keys rest; floor breathes slowly.
- Typing: field text grows one character at a time; the key for each character lifts briefly; a wave leaves that key's position.
- Pause between prompts: field holds the full text for a moment, then clears.
- Paused: typing stops; field holds its text; glow fades to rest.
- Submitted (when interactive): field shows a short sending state and the text moves up as a message.

## Interactions
- Decorative mode: no interaction; pointer and focus do not change it.
- Interactive mode (when `on_submit` is given): the field accepts typed input; Enter submits; Shift + Enter inserts a line break.
- Pressing a key on the physical keyboard while the field is focused lifts the matching drawn key and sends a wave.

## Motion (functional)
- Each character appears as if typed; keys lift in the order typed; waves travel outward from the key and fade with distance.
- The floor's light follows the most recent wave, so the strongest light is always near the latest key.
- Between prompts the field clears with a short fade so the next line starts clean.
- With reduced motion preferred: characters appear at once, keys do not lift, waves are replaced by a single steady glow.

## Accessibility
- Decorative when not interactive: `aria-hidden`.
- Interactive: the field is a labelled text input; its placeholder is not the only label.
- Typing animation does not move focus and is not announced; submitted text is announced by the parent.

## Responsive
- Scales with its container; the keyboard may drop its function row below narrow widths.
- Phone width: keyboard hides and only the field and glow remain.

## Composition hints
- Text input (interactive mode), decorative keyboard drawn from key shapes, layered glow.

## Acceptance checks
1. In decorative mode, nothing responds to pointer or focus.
2. In interactive mode, Enter submits the typed text once and Shift + Enter adds a line break.
3. Each typed character lifts its matching key in order.
4. The glow's strongest point follows the most recent key.
5. With reduced motion preferred, the field shows full text without typed-in frames.
6. At phone width the keyboard is hidden and the field is still fully visible.

Uses no partner logos.
