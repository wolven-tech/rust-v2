# Change Log Timeline

Source item: affectUI Pro `components/commit-trail` (functional reference only; see README).

## Purpose
Presents the recent changes on one branch as a vertical timeline: one row per change, showing its message, a kind tag, and how long ago it happened. Opening a change reveals who made it (a person, an agent, or both), its identifier, and the size of the change. Merges fold away the changes they brought in. A change still being written shows as in progress until it is saved.

## Data
| Field | Type | Meaning |
|---|---|---|
| `change_list` | Change[] | Newest first. `{ change_id, change_message, change_author, co_author?, changed_at?, change_kind?, file_count?, lines_added?, lines_removed?, change_body?, merged_changes?: Change[], in_progress? }` |
| `change_author` / `co_author` | `{ writer_name, is_agent?, lane_color?, writer_mark? }` | Shown when the change is open. Agent authors receive a badge in their assigned colour. |
| `changed_at` | number \| Date \| string | Shown as a relative label; the full timestamp appears when the change is open. |
| `change_message` | string | One line. Backticked names render as inline chips: file-like names get a file icon, others get a branch icon. |
| `name_tooltips` | Record<string, string> | Tooltip text for names in the message, shown after a brief rest on the name. |
| `change_kind` | string | Tag: `new_capability`, `correction`, `restructure`, `speed`, `documentation`, `verification`, `housekeeping`, `release`, `merge`, or a custom value. |
| `kind_overrides` | Record<string, { label?, color? }> | Renames kinds or adds new ones. |
| `release_tag` | string | Release or tag label, shown as a small chip. |
| `in_progress` | boolean | The change is still being written; shows an in-progress indicator. Turning it false shows a saved check. |
| `branch_names` / `current_branch` / `on_branch_switch(b)` | string[] / string / callback | Branch switcher in the header; two or more branches open a menu. |
| `open_change_id` / `on_open_change(id \| null)` | controlled | Only one change is open at a time. |
| `expanded_merge_ids` / `on_merge_expand_change(ids)` | controlled | Which merges show their folded changes. Defaults to all folded. |
| `has_older` | boolean | Older changes exist; the timeline fades out at its end. |
| `scroll_height` | number | The list scrolls once it exceeds this height. |
| `date_locale` | string | Locale used for date formatting. |
| `text_overrides` | partial object | Every visible word: title, empty state, in-progress text, untitled, copy, copied, with, now, switch-branch, file, branch, counts, saved text. |
| `root_class` | string | Extra class applied to the root. |

## Anatomy
Reading order:
1. Header: branch name (or switcher) and change count.
2. Timeline: a vertical line with one marker per change. Each row shows the marker, the message with its chips, the kind tag, and the relative time.
3. Merge rows: a chevron folds or unfolds the changes merged in; when unfolded, a branch line bends off the main line.
4. Open change details (beneath its row): author and co-author, full time, identifier, files, added and removed counts, body text, and a copy-identifier control.
5. Footer fade when more history exists.

## States
- **Idle:** rows at rest; one row per change; merges folded.
- **Change open:** details shown beneath it; the others step back.
- **In-progress change:** the marker shows an arc animation; the text shows an in-progress label; the row is not a button.
- **Saved:** a check appears and the line to the next change fills in.
- **Merge expanded:** its folded changes appear on a side branch.
- **No changes:** the header remains and a quiet empty line replaces the list.
- **Branch switch:** the list is replaced with the other branch's changes.

## Interactions
- Up/Down: move between changes, including into an open merge's changes.
- Home/End: first or last change.
- Enter or Space on a change: opens it. On a merge: expands or collapses its changes.
- Escape: closes the open change, or folds the merge that has focus.
- Tab inside an open change: reaches the copy-identifier control.
- Pointer hover on a name in a message: after a brief rest, shows its tooltip. Not required on touch.
- Branch switcher: Enter or Down opens the menu; arrows move; Enter picks; Escape returns to the switcher.

## Motion (functional)
- New changes push the list downward; on first display, rows rise in one after another.
- A change's check draws in when it is saved, and the line to the next change fills.
- In progress: a light travels up the last stretch of the line into the in-progress marker, repeating; the marker rings once when saved.
- Expanding a merge: the trunk line shifts sideways, a side line peels away past the merged changes, then rejoins.
- Reduced motion: changes are immediate and the travelling light stays off.

## Accessibility
- The timeline is an ordered list whose name carries the branch identity.
- Every change works as a button exposing expansion; its label joins message, writers and timestamp.
- An in-progress change is not a button and reads as in progress along with its author.
- A polite live region announces when a change is saved.
- Branch switcher: a menu trigger whose items are radio options.
- Focus indicators show for keyboard use only.

## Responsive
- One line per change at a fixed row height; touch devices get taller rows.
- Long messages truncate inside the chip rather than dropping it; tags and branch names end in an ellipsis, with the full text on hover.
- Times sit in a fixed-width column so tags line up.
- Right-to-left: the timeline anchors to the opposite edge and side lines peel off mirrored.
- No horizontal scrolling at phone width.

## Composition hints
List (ordered), button (change row), badge (kind tag), avatar or agent badge (author), chip (file and branch names in messages), disclosure (merge fold), menu button with radio items (branch switcher), tooltip (name tooltips), live region helper, timestamp text.

## Acceptance checks
1. Changes render newest first; a change with `in_progress: true` shows an in-progress label and is not a button.
2. Setting `in_progress` to false on an in-progress change draws its saved mark and fires exactly one live-region announcement containing its message.
3. Opening a change shows its author, co-author (when present), full time, identifier, file count, and added and removed counts.
4. A merge row shows the count of changes it brought in; expanding it reveals those changes in a nested list.
5. Backticked names in a message render as chips; names with an extension get the file icon.
6. Switching branches with `on_branch_switch` replaces the list and resets the open change.
7. Escape closes an open change and returns focus to its row.
8. A kind without a configured colour renders as a neutral tag.
9. Relative times show the appropriate unit for recent and older times.
10. At phone width, rows stay one line tall and no horizontal scrolling occurs.
</content>
