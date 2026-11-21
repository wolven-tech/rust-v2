# Inspected Composer Scene

Source item: affectUI Pro `illustrations/inspect-composer` (functional reference only; see README).

## Purpose
A product scene showing a chat composer as if a designer were inspecting it in a layout tool: tilted back, its parts outlined, spacing shown, and a guide walking through its rows. It explains the composer's structure to a reader.

## Data
- `composer`: { placeholder_text, mode_label, tools: string[], connected_apps: node[] (supplied by caller), send_label }.
- `inspect_steps`: ordered list of { row_index, note }.
- `auto_play` (boolean, default true); `speed`: "slow" | "normal".
- `extra_classes`.

## Anatomy
In reading order:
1. Ghost composers: faded copies above and below for depth.
2. Focused composer: a top row with mode selector and connected-apps icons; a central text row with placeholder; a bottom row with attach and send controls.
3. Selection frame and per-row outlines.
4. Spacing markers between rows and at the edges.
5. Guide line moving down the composer, with a pointer mark.
6. Row label tag.

## States
- Playing: guide moves through rows in order; spacing updates per row.
- Paused on hover or focus: guide stops on the current row.
- Reduced motion: static view with all spacing shown.

## Interactions
- Pointer over a row: pins the guide to that row.
- Keyboard focus on the scene: pauses; arrow keys move the guide by row; Escape resumes play.
- No click behaviour.

## Motion (functional)
- The composer settles into its tilted view, then the frame draws.
- The guide moves from row to row and pauses on each; the row label changes as it passes.
- Spacing markers follow the guide.
- With reduced motion preferred, nothing moves.

## Accessibility
- Decorative; `aria-hidden` on the scene; the guide is not announced.
- Scene is not focusable unless the parent requests interactive mode.

## Responsive
- Scales with its container; tilt reduces at narrow widths.
- Phone width: spacing markers hidden; guide and row labels remain.

## Composition hints
- Composer surface (drawn), outline overlay, marker labels, moving guide, tag label. Connected-app icons are supplied by the caller as partner logos.

## Acceptance checks
1. The guide visits each row in order and pauses on each.
2. Pointer over a row pins the guide there; leaving resumes play.
3. Spacing markers for the current row are visible.
4. With reduced motion preferred, the static view shows all rows and spacing.
5. Keyboard arrow keys move the guide by row; Escape resumes play.
6. At phone width spacing markers are hidden and nothing scrolls sideways.

Uses partner logos (supplied by caller) for connected-app icons. Does not describe any brand mark.
