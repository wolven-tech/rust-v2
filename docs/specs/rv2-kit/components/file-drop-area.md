# File Drop Area

Source item: affectUI Pro `components/dropzone` (functional reference only; see README).

## Purpose
Lets a person drop one or more files to upload them, and shows for each file what is queued, how far along it is, how long remains, and which uploads failed and can be retried. Used for attachments, imports, asset libraries and onboarding.

## Data
- `accepted_extensions` (text[], optional): for example `[".pdf", ".png"]`. Other files produce a row with an error. Any file is accepted when omitted.
- `max_bytes` (number, default 25 MB): larger files produce a "too large" row.
- `upload_handler(file, report_progress)`: uploads one file; `report_progress` receives a value from 0 to 1. Rejecting marks the row failed and shows the message; Retry calls it again. When omitted, the component simulates an upload.
- `on_list_change(items)`: called on every change to the list. Each item: `{ item_id, file_name, byte_size, item_state, progress, error_text?, file? }`.
- `hint_text` (text, optional): subtitle beneath the heading; falls back to the permitted formats and the maximum size.
- `demo_run` (boolean, default false): runs a scripted demonstration with four sample files. Preview use only.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Drop area: a dashed bordered region with an icon cluster, a title ("Drop files here or browse"), and the hint.
2. List card below: one row per file. Each row shows a file-type badge (tinted by extension group), file name, size, a progress bar, a remaining-time or status text, and actions (Retry on failure, Remove).
3. Summary row at the bottom of the card: counts of uploading, done and failed items.

## States
Per file:
- Queued: row shown, bar empty, status "starting".
- Uploading: bar fills from 0 to 1; status shows bytes transferred and time left.
- Done: bar full; a completion check appears.
- Failed: bar in the error tone; the message shows; Retry available.
- Rejected: wrong type or too large; the row shows the reason; no upload.
- Removed: row leaves the list.

Drop area:
- Idle.
- Drag over: the area shows an active border and overlay, the icon cluster fans out, and the title shows the number of files being dragged.
- Dragging a rejected type: active styling without a count. (Spec choice.)

## Interactions
Pointer:
- Drop files on the area: each accepted file becomes a row and uploads.
- Click the area: opens the system file picker (multiple selection).
- Retry on a failed row: uploads again.
- Remove on any row: cancels if uploading, then removes the row.

Keyboard:
- Tab: visits the drop zone first, then each row's Retry and Remove controls.
- Enter or Space while the zone has focus: launches the picker.

## Motion (functional)
- Drag over: the document icons fan out; the border changes tone over a faint overlay.
- Rows enter one after another; the progress bar fills; a completion check appears when a file is done.
- The summary shimmers while any upload runs.
- A failed row jolts once and its bar switches to the warning colour family.
- Removed rows collapse away.
- Reduced motion: all of this is instant.

## Accessibility
- The drop zone acts as a button that launches the picker; its description enumerates the permitted formats and maximum size.
- Every row's progress bar reports its numeric value; Retry and Remove include the file name in their labels.
- Finished, failed and refused files are spoken through a courteous status region.

## Responsive
- Fills its container up to a maximum width; long file names truncate with an ellipsis.
- Touch: rows and buttons sized for fingers.

## Composition hints
Drop region (button with drag handlers), icon cluster, file-type badge, progress bar, status text, icon buttons (retry, remove), list panel, summary row, live region.

## Acceptance checks
1. Dropping a `.pdf` file when `accepted_extensions` is `[".pdf"]` creates one row that begins uploading.
2. Dropping a `.exe` file produces a row with an error and no call to `upload_handler`.
3. A file larger than `max_bytes` produces a "too large" row and no upload.
4. `upload_handler` reporting 0.5 shows the bar at 50 percent; resolving shows the done state.
5. A rejected upload shows its message; Retry calls `upload_handler` again for that file only.
6. Remove on an uploading row cancels it and removes the row.
7. `on_list_change` is called on each status and progress change with the full list.
8. Enter on the zone launches the picker; focus order runs zone, then Retry and Remove for each row in turn.
9. Long file names truncate with an ellipsis and do not widen the panel.
10. With `demo_run` true, four sample rows play through and the component makes no network calls.
</content>
