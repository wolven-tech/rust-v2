# Change Replay

Source item: affectUI Pro `components/diff-timeline` (functional reference only; see README).

## Purpose
Shows what a coding agent changed in a pull request, one edit at a time, across the run's timeline. It can follow the agent live while it works, or replay a finished run. Reviewers use it to see when the large changes happened and which files they touched.

## Data
- `author` (`{ author_id, author_name }`): the agent. The id determines its identity mark.
- `context_text` (text, optional): quiet text after the name, such as the pull request reference.
- `change_events` (Event[]): every edit. Each Event has `at_offset` (position along the run), `file_path`, `lines_added`, `lines_removed`, and optional `line_details` (the changed lines). Scales are computed from all events, so the axis never rescales during playback.
- `shown_until_offset` (number, default: the position of the last event): later events stay hidden.
- `in_progress` (boolean, default false): the run is still writing. Shows an activity indicator and names the file being edited.
- `axis_length` (number, default: the run length rounded up to a whole minute).
- `max_visible_files` (number, default 7): file rows visible before the list scrolls. The panel keeps one height.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Header: identity mark, author name and context.
2. Status line: the current activity ("Editing <file>" while in progress; a summary once finished).
3. Totals: lines added, lines removed, net, and the number of files.
4. Chart: a time axis with two series, added and removed counts. The two strokes are drawn over each other, and the area between them reads as change.
5. File list: one row per file, with its own small bar of added and removed counts, and a time mark where its first event occurred. Rows appear in the order the agent first touched each file.
6. Event detail: when an event is opened, its changed lines and totals appear in a panel.

## States
- In progress: activity indicator on; line ends pulse; the status names the current file.
- Finished: no indicator; the status shows totals.
- Replay at a position: `shown_until_offset` set before the last event; later events hidden; totals reflect only the visible events.
- Event open: one event selected; its detail visible.
- Empty: no events; the chart shows only the axis and the file list is empty. (Spec choice.)
- Many files: the file list scrolls once it passes `max_visible_files`.

Transitions: a new event while in progress adds a segment to the chart and, if the file is new, a row to the list; the row rises in and its bar grows. Changing `shown_until_offset` moves the replay.

## Interactions
Pointer:
- Hover a file row: marks its events on the chart baseline.
- Click or hover a chart segment: shows that event's totals.
- Click an event: opens its detail. Click again or press Escape: closes it.

Keyboard:
- Tab: focuses the chart.
- Left / Right arrow: previous or next event.
- Home / End: first or last event.
- Escape: closes the open event.

## Motion (functional)
- While in progress, line ends pulse softly and the agent mark animates.
- A file row lifts into place on first contact; its bar extends from the left edge, widening with every further event.
- A faint glow tracks whichever file is being written at that moment.
- Reduced motion: all of this is turned off.

## Accessibility
- The chart is one focusable group; arrow keys step through events, and each step is read out.
- Hovering a file row is mirrored in the chart. The open event is reported in a polite live region: file, time, totals, changed lines.
- The status line is live while in progress, so the current file is announced as it changes.

## Responsive
- Fills its container; the chart and rows scale while line widths stay constant.
- When the container is narrow: totals move beneath the name, folder names hide, and counts shorten (for example 4.8k).

## Composition hints
Panel, identity mark (generated from id), status line, stat totals, two-series line chart with area fill, list rows with inline bars, detail panel, live region, pulse animation.

## Acceptance checks
1. With `in_progress` true, the status names the file of the latest event and the activity indicator is visible.
2. Setting `shown_until_offset` below the last event hides later events from the chart and the totals.
3. Right arrow on the focused chart moves the open event to the next one and reads out its file and time.
4. Hovering a file row marks that file's event times on the baseline.
5. A file's row appears only at its first event and keeps its position for later events.
6. Totals equal the sum of visible events' added and removed counts; net equals added minus removed.
7. With more files than `max_visible_files`, the list scrolls and the panel height does not change.
8. Escape closes an open event and focus returns to the chart.
9. When the container is narrow, folder names are hidden and counts are shortened.
10. With reduced motion requested, the pulse and row rise do not animate.
</content>
