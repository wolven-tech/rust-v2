# Absence Digest

Source item: affectUI Pro `components/away-summary` (functional reference only; see README).

## Purpose
Placed inside a conversation when someone comes back after agents continued working without them. It states how long the person was gone, then groups what waits on them, what broke, and what completed, with one action per row. The whole block can be collapsed into a single divider line.

## Data
| Field | Type | Meaning |
|---|---|---|
| `away_since` | timestamp value (date, number or string) | The moment the person stepped away |
| `digest_entries` | Entry[] | `{ entry_id, entry_name, entry_state, entry_note?, button_text? }`. Arrives in any order; the component applies its own ordering |
| `entry_state` | `"awaiting_input" \| "failed" \| "completed"` | Per entry. Determines the marker, the ordering and the default button wording (Review, Retry, Open) |
| `button_text` | string \| false | Per-entry wording override. `false` produces a row without a button |
| `on_entry_action(entry_id, entry)` | callback, may return a promise | Fired when a row button is activated. A returned promise holds the button in a busy state until it settles |
| `on_acknowledge()` | callback | Fired when the person acknowledges. The caller records that the digest was seen |
| `initially_folded` / `folded` | boolean | Whether the block starts as the divider |
| `on_fold_change(folded)` | callback | Fired when the block collapses or reopens, from the card or the divider |
| `row_limit` | number | How many rows appear before the "Show more" control. Default 5 |
| `time_format` | `"elapsed" \| "clock"` | Shows the time since leaving as a duration, or as the wall-clock time of leaving |
| `current_time` | timestamp value | Reference moment. Defaults to the live clock, refreshed on a slow schedule |
| `text_overrides` | partial object | Every visible string: heading, counts, default button wording, show-more, acknowledge, divider text |
| `root_class` | string | Extra class applied to the root |

## Anatomy
Reading order:
1. Heading reading "Since your last visit", followed by the elapsed time.
2. Count line giving the number awaiting input, failed, and completed.
3. Rows ordered awaiting input first, failed next, completed last. Each row has a state marker, the entry name, a note line, and a button.
4. A "Show N more" control, present only when rows outnumber `row_limit`.
5. An acknowledge control.
6. When collapsed: one divider line showing the elapsed time and an expand control.

## States
- **Expanded, unresolved**: every row visible, buttons active.
- **Row busy**: the row's button displays progress. The button stays in place and keeps focus.
- **Row resolved by a change**: an awaiting-input row whose `entry_state` becomes `completed` stays at its index and swaps its marker and note in place.
- **Collapsed**: only the divider shows; the body takes no input.
- **Empty**: no entries. The card shows a brief "Nothing new" line and the acknowledge control only.
- **Show more expanded**: the hidden rows are visible; Show less brings the list back to the limit.

## Interactions
- Pointer: row button, Show more, Acknowledge, and the divider to reopen.
- Tab: row buttons in list order, then Show more, then Acknowledge.
- Enter or Space on a row button: runs that row's action.
- Enter or Space on Acknowledge: collapses the card; focus moves to the divider.
- Enter or Space on the divider: reopens the card; focus goes back to the button the person last used, or to the first button.
- Escape inside the expanded card: collapses it, with the same focus handling as Acknowledge.
- Enter on Show more: the rows expand and focus moves to the first newly revealed row's button.

## Motion (functional)
- The awaiting-input marker pulses softly. It is the only continuous motion.
- A row that changes swaps in place: the new marker appears, a completion tick draws itself, and the counts update.
- Collapsing and reopening animate the height between card and divider.
- Reduced motion: every change applies immediately.

## Accessibility
- The region is labelled "Since your last visit", and its heading includes the elapsed time.
- Rows form a list. Each button names its entry, for example "Review: <entry name>".
- States are conveyed in words, never by colour alone.
- Acknowledge and the divider are buttons that expose their expanded state. Hidden content is removed from the focus order.
- A live region announces each row change along with the updated counts.
- Focus never drops to the page body when a button disappears or the card collapses.

## Responsive
- Rows hold their height across states, so nothing beneath shifts when an entry resolves.
- When there is little horizontal room, the counts move beneath the heading. Long names and notes truncate, and the full text appears on hover or long press.
- Long custom button labels have a width cap and truncate.
- Right-to-left layouts mirror alignment and keep each text run in its own direction.
- No horizontal scrolling at phone width.

## Composition hints
Card, list, badge or marker (state), button, count text, disclosure (fold), divider line, live region helper, tooltip (truncated text).

## Acceptance checks
1. With three entries in mixed states, rows render as awaiting input, then failed, then completed, regardless of input order.
2. Activating a row button calls `on_entry_action(entry_id, entry)` once; a returned promise keeps the button busy and in place until it settles.
3. Acknowledge collapses the card to a divider and calls `on_acknowledge` once; activating the divider reopens the card.
4. Escape inside the card collapses it and moves focus to the divider.
5. Changing an awaiting-input entry's `entry_state` to `completed` leaves its row at the same index.
6. With more entries than `row_limit`, only that many are visible until Show more is activated.
7. An entry with `button_text: false` renders no button.
8. A live region announces a row change together with the new counts.
9. With `time_format: "clock"`, the heading shows the wall-clock time of `away_since`.
10. At phone width there is no horizontal scrolling.
</content>
