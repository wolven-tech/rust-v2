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

## Amendment 5: structure and demo

Archetype: one focused composer card, tilted back, with a faded composer above it and a selection overlay that walks its three rows.

Replaces Anatomy item 2 with the three rows below.

Visible at rest:
- Top row: add control at the left; mode selector showing `mode_label` with a chevron; notification control; round send control at the right.
- Attach row: a "Work in a folder" chip with a folder mark, and a "Connect apps" chip carrying three app marks supplied by the caller.
- Message row: the message text with a caret.
- Selection frame on the top row, with a row tag reading "Toolbar", spacing numbers and a width label.
- Faded composer above: its message text and no overlay.

Gallery demo:
- Focused message: "Draft the release notes for version 3". Mode: "Auto".
- Faded composer above: "Summarize the open tickets for the team".
- App marks: three generic glyphs from the kit, standing in for caller-supplied marks.

Motion, settled frame: tilted card; frame on the top row.
Moves: the guide walks top row, attach row, message row. The row tag changes as it passes ("Toolbar", "Attach", "Message"), spacing follows the guide, and the faded composer drifts slightly.
Reduced motion: static view, top row selected.

Acceptance checks (continued):
7. The focused composer shows three rows at rest, in order: top row, attach row, message row.
8. The attach row shows a folder chip and a connect-apps chip with exactly three app marks.
9. The faded composer above shows its message only and never takes the guide.
10. The send control sits on the top row, and the mode selector shows the value of `mode_label`.

## Amendment 6: structure and demo

Supersedes Amendment 5 where they differ. Replaces:
- "Visible at rest", faded-composer bullet: faded toolbar and attach rows stacked above and below the message row. No second composer is drawn.
- Selection frame "on the top row, tag Toolbar" (at-rest bullet, settled frame, reduced motion): the frame sits on the message row, tag "Message".
- "Gallery demo", faded-composer bullet: removed.
- Check 7: the message row is the only full-strength row.
- Check 9: the faded rows above and below never take the guide.

Archetype: one composer column, message row in front, faded toolbar and attach rows stacked above and below it, a selection frame that walks its rows.

Visible at rest, top to bottom:
1. Toolbar row, faded: add control at left; mode selector showing `mode_label` with a chevron; notification control; round send control at right.
2. Attach row, faded: "Work in a folder" chip with folder mark; "Connect apps" chip.
3. Message row, full strength: message text with a caret. Selection frame on this row with tag "Message", a width label and spacing tags.
4. Toolbar row, faded, repeating row 1's labels.
5. Attach row, faded, repeating row 2's chips without app marks.
- App marks: exactly three, on row 2 only.

Gallery demo:
- Message "Draft the release notes for version 3"; mode "Auto".
- App marks: three generic glyphs from the kit.

Scale: the column spans about three fifths of the stage width, centred horizontally. Row 3 sits at the vertical middle. Row 5 ends at least one row height above the stage's bottom edge.

Motion, settled frame: column in its tilted view; frame on row 3.
Moves: the guide walks rows 1, 2, 3 in order; the row tag changes "Toolbar", "Attach", "Message"; spacing follows the guide; the faded rows drift slightly.
Reduced motion: static view, message row selected, all spacing shown.

Defects, stated as checks: two composer copies, the second clipped at the bottom edge (check 11); no measurement frame on the message row (check 12).

Acceptance checks (continued):
11. At rest one composer column is drawn. The lowest faded row ends above the stage's bottom edge, and no row is clipped.
12. The selection frame rests on the message row with tag "Message", a width label and spacing tags. No other row carries a frame.
