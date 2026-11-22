# Client Workspace

Source item: affectUI Pro `blocks/client-context` (functional reference only; see README).

## Purpose
A workspace for one client: the client's headline numbers at the top, and a shared timeline of goals, notes, files and generated reports below. People switch between clients, search and filter the timeline, and add context that everyone on the account can read.

## Data
- `workspace`: { id, name, legal_name?, meta: string[], synced_at: timestamp }.
- `workspace_options`: list of { id, name } for the switcher.
- `stat_cards`: list of { id, label, value (formatted string), delta?, series: number[] }. Maximum four.
- `sections`: list of { id, label, count? }: context, reports, financials, settings.
- `timeline_items`: list of { id, kind: "note" | "report" | "file" | "goal", author, at: timestamp, text, references?: { label, target }[] }.
- `kind_options`: list of item kinds; `selected_kind` (or "all").
- `on_switch_workspace(id)`, `on_submit_note({ text, references, attachments })`, `on_open_reference(ref)`, `on_search_text(text)`, `on_kind_change(kind | "all")`, `on_refresh()`.
- `is_syncing`: boolean.

## Anatomy
In reading order:
1. Header: workspace name with its meta line; a sync status with a refresh control; a workspace switcher.
2. Stat row: up to four cards, each with label, value, delta and a sparkline.
3. Section bar: context, reports, financials, settings.
4. Context pane: a composer at the top (text area, mention trigger, attach control, submit); under it, the timeline.
5. Timeline items: avatar, author, action sentence, time, body text with inline references.
6. Search field and kind filter sit in the section bar on wide layouts.

## States
- Loading: stat cards show placeholder blocks; the timeline shows three placeholder rows.
- Synced: sync label shows the relative time since the last sync.
- Syncing: sync label changes to a syncing word; the refresh control turns.
- Empty timeline: a one-line prompt to add the first goal or note.
- Composer empty: submit disabled. Composer with text: submit enabled.
- Submitting: composer locked; submit shows progress.
- Filtered: only items of the selected kind; the count in the section bar reflects them.
- Settings section: switches and fields replace the timeline; the section bar keeps its place.

## Interactions
- Switch workspace: opens a listbox; choosing one replaces the workspace and keeps the active section.
- Typing `@` in the composer: opens a suggestion list of reports; arrow keys move, Enter inserts a reference, Escape closes the list and keeps the text.
- Submit with Enter; Shift + Enter inserts a line break.
- Attach: opens a file picker; attached files appear as removable chips under the composer.
- Click a reference inside an item: calls `on_open_reference`.
- Search: filters items live across text and reference labels.
- Click a stat card: makes it the focused metric; the others fade back (see motion).
- Refresh control: calls `on_refresh`; while it runs, the sync label shows syncing.
- Keyboard: Tab order is workspace switcher, refresh, stat cards, sections, composer, attach, submit, search, filter, items. Escape closes any open menu.

## Motion (functional)
- On first render, stat tiles appear in sequence while their mini trend lines trace themselves.
- New items slide in at the top of the timeline; existing items move down.
- Focusing a stat card fades the other stat cards instead of redrawing their sparklines.
- Section changes move the highlight marker to the chosen section with a short spring.
- With reduced motion preferred, all of these are instant.

## Accessibility
- Sections use tablist semantics; the active panel is labelled by its tab.
- Stat cards are toggle buttons with a pressed state when focused.
- Reports in the timeline are disclosure buttons with an expanded state.
- Workspace switcher and mention suggestions are listboxes; the active option is announced.
- The sync label is a status region.
- Attachment remove buttons have names that include the file name.

## Responsive
- Wide: stat row of four; search and filter sit in the section bar.
- Medium: stat cards wrap to two rows; sections stretch across the width; search moves beneath them.
- Phone width: stat cards in a two-column grid; composer keeps full width; attach and submit stay visible.
- Touch: the `@` suggestion list is a sheet from the bottom.

## Composition hints
- Tabs (sections), card (stat and timeline items), badge (kind), avatar, text area with mention, popover list for suggestions, chip for attachments, switch and text field for settings, status indicator, skeleton.

## Acceptance checks
1. Switching workspace replaces header, stat cards and timeline, and keeps the currently active section.
2. Typing `@` opens the suggestion list; Enter inserts a reference and Escape closes the list without clearing text.
3. Submit with Enter posts the item; Shift + Enter adds a new line and does not submit.
4. The kind filter and search combine: an item must match both to appear.
5. A new item appears at the top of the timeline and existing items shift down.
6. The sync label shows a relative time when idle and a syncing word while refreshing.
7. A report item toggles its detail with its expanded state matching the visible detail.
8. At phone width the stat cards form a two-column grid and nothing scrolls sideways.
9. With no items, the empty prompt appears and the filter count reads zero.
