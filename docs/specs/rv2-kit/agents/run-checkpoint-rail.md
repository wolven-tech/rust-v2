# Run Checkpoint Rail

Source item: affectUI Pro `components/agent-checkpoints` (functional reference only; see README).

## Purpose
Shows a long-running task as an ordered list of saved points, so a user can return to an earlier point and let the work continue from there as a new attempt. The abandoned attempt stays visible, de-emphasised, so nothing already done is lost from view.

## Data
- `entries`: list of `{ id, label, glyph?, follows_id?, run_state?, started_ms?, file_count?, spend? }`, in the order taken. Pass the whole list again whenever an entry is added or changes.
  - `follows_id`: id of the entry this one continues from. An entry whose predecessor already has a later successor starts a new attempt on its own branch line.
  - `run_state`: `"running"` | `"done"` (default) | `"failed"` | `"stopped"`.
  - `glyph`: small glyph before the label, e.g. a tool identity.
- `current_id`: id of the entry the run currently stands on. Defaults to the last entry.
- `goal_text`: string, what the run is for.
- `heading`: string, bar heading. Default "Checkpoints".
- `on_restart_from(entry)`: callback fired when a restart is requested.
- `format_elapsed(ms) => string`: formats `started_ms`. Default `m:ss`.
- `format_spend(spend) => string`: formats the total in the bar. Default dollars.
- `extra_class?`

## Anatomy
Top to bottom:
1. Bar: `heading`, `goal_text`, count of entries, number of attempts when more than one branch line exists, total spend.
2. Rail: one row per entry in order. Each row has a node on a vertical line, an optional mark, the label, and on the right the file count, the elapsed time and a restart control.
3. Branch lines: a restart bends a new line out of the chosen entry. Branch lines sit left of the main line and carry the new attempt's rows.

Each row has a fixed height. The restart control takes the place of the file count and elapsed time while the row is hovered or focused.

## States
- `done`: plain row, time and spend shown.
- `running`: label shimmers while the run works toward this entry.
- `failed`: row carries a visible text badge.
- `stopped`: an entry the user restarted away from while the run was still working toward it. Labelled "stopped".
- `current`: the entry the run stands on. The path from the first entry to `current_id` uses the emphasis tone; everything off that path is de-emphasised.
- `left behind`: an attempt the user restarted from. Its rows turn muted and fade back, but stay in the list.
- `empty`: not observed in the public demo (no zero-entry state shown). Clean room may render the bar with a count of zero and no rows.

The reference demo advances by itself: entries appear with growing elapsed time and amount, one fails, one restart starts attempt 2, which then succeeds.

## Interactions
- Pointer click on a row's restart control: fires `on_restart_from(entry)`; a new entry is appended with that row as `follows_id`; `current_id` moves to it; later rows of the previous attempt become left behind.
- Pointer hover on a row: reveals the restart control in place of file count and time.
- Keyboard: ArrowUp / ArrowDown move focus between rows; Home / End move to first / last row; Enter or Space on a focused row performs the restart.
- Focus order: one tab stop enters the list; arrows move inside it; Tab leaves it.
- Touch: the restart control is the tap target; no swipe gestures.
- Escape and outside click: no effect defined.
- Overflow: past 14 rows the list scrolls inside the container and keeps the current row in view.

## Motion (functional)
- A new entry enters at the bottom: its node appears and the connecting line draws down to it; rows below move to make room.
- A restart draws a new branch line from the chosen entry down to the new attempt; the abandoned attempt mutes and fades back.
- A new entry's node gives one short highlight on arrival.
- `running` labels shimmer; this is the only continuous motion.
- With reduced-motion preference, all changes appear immediately.

## Accessibility
- Rows form an ordered list with a single tab stop; arrows move between items.
- Each row exposes its label, attempt number, run state and whether it is current or left behind.
- The current row has `aria-current="step"`.
- The restart control is a pointer shortcut; keyboard users press Enter on the row.
- The count in the bar is announced politely when it changes (`aria-live="polite"`).
- State is never conveyed by tone alone: `failed` has a text badge, `stopped` a text label, the current entry is announced.

## Responsive
- Fixed row height: finishing or failing an entry does not shift layout.
- Narrow widths hide `goal_text` first, then the file count; label, time and restart control remain.
- Scroll container inside the component past 14 rows.
- Supports light and dark themes via an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Container card, ordered list with custom rail drawing, status badge, glyph slot, text button (restart), shimmer text, polite live region, formatter props for elapsed time and spend.

## Acceptance checks
1. Pressing the restart control on a row appends an entry whose `follows_id` is that row and moves `current_id` to it.
2. Entries after the restart point are still rendered, marked left behind, not removed.
3. A second successor of the same predecessor is drawn on a new branch line and counted as a new attempt in the bar.
4. ArrowDown / ArrowUp move focus between rows; Home and End go to the first and last row.
5. Enter or Space on a focused row triggers `on_restart_from` with that entry.
6. The list is a single tab stop; Tab from the list moves to the next focusable element after it.
7. The current row has `aria-current="step"`.
8. A `failed` entry shows text, not only a tone change.
9. Changing the entry count updates a polite live region.
10. With more than 14 rows the list scrolls within the component and the current row stays visible.

## Amendment 2: structure and demo

This spec had no first amendment, so this is its first structural note. Checks continue from 11.

**Archetype: one card, one vertical rail.** The bar runs across the top of the card. Below it, one column of rows hangs from a rail set in the left part of the card. A new attempt leaves the rail at the row it restarts from and runs to the left of the main rail, so branch lines live in a gutter and never cross a label.

**Parts that must exist**
- Bar: heading, goal text, entry count, total spend. The attempt count joins the bar only once a second attempt exists.
- Main rail: one node per entry on the main line. The node of the current entry is drawn differently from the done nodes.
- Row: optional glyph, label, then file count and elapsed time at the right edge. The restart control takes the place of those two figures on hover or focus.
- Branch line: drawn only after a restart, carrying the new attempt's rows.
- No row prints an attempt number. A row's attempt is shown by its place on a branch line.

**Gallery demo counts.** Rest shows three rows, not six, and no inline attempt labels.
- Instance one: three entries on the main line, two done and one running as the current entry.
- Instance two (after a restart): four rows. On the main line, two done entries and one failed entry, left behind. A branch line leaves the second done entry and carries the current running attempt.

**Visible at rest**
- Instance one: the three rows in order, the last one current; the bar reads three entries and a total spend; no branch line; no attempt count.
- Instance two: the failed row with its text badge and its left-behind treatment; the branch line holding the current row; the bar reads two attempts.

**Acceptance checks (continued)**
11. Instance one renders three rows on one rail, the last being the current entry, with no branch line and no attempt count.
12. The bar's entry count equals the rows on both rails together. The attempt count appears only in instance two.
13. In instance two the abandoned rows stay on the main line, muted and readable, and the new attempt sits on a branch line left of the main rail.
14. No row prints an attempt number; the only attempt indicators are the branch line and the bar count.
15. In instance one the current entry's node is visibly different from the done nodes.
