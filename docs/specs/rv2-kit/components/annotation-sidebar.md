# Annotation Sidebar

Source item: affectUI Pro `components/comments-panel` (functional reference only; see README).

## Purpose
A column that runs alongside a document, design or record. It lists the notes people have left, filters them, lets a person add a note, and runs any note that opens with a slash command as a patch applied to the document.

## Data
| Field | Type | Meaning |
|---|---|---|
| `notes` | Note[] | `{ note_id, writer: { writer_name, writer_mark? }, posted_at, note_text, note_state? }`. A note whose text begins with `/patch` is runnable. |
| `acting_writer` | Writer | Writer recorded on newly added notes. |
| `on_run_patch(note)` | callback, may return a promise | Runs the patch. The control shows a working state until the returned value settles. |
| `on_note_added(note)` | callback | Called for each new note. |
| `on_close_request()` | callback | Optional. Adds a close control. |
| `section_title` | string | Defaults to "Notes". |
| `root_class` | string | Extra class applied to the root. |

## Anatomy
Reading order:
1. Header: section title and, optionally, a close control.
2. Filter tabs: All, Queries, Patches, each with a count.
3. Sort control: a menu of radio items, newest first or oldest first.
4. Note list: each item shows the writer's name, a relative time and the text. Patch notes also show an Apply patch control.
5. Composer at the foot: a text field and a send control.

## States
- **Empty:** a "No notes yet" line appears in the list area.
- **Filter active:** the list shows only matching notes; the chosen tab shows its selected state.
- **Patch running:** the apply control shows a working label and a spinner, and is disabled until it settles.
- **Patch failed:** the apply control returns to idle, and an error line appears beneath the note.
- **Resolved note:** shown muted with its resolved toggle on. It stays listed under All and can be removed from view with a Hide resolved control.

## Interactions
- Tab: filter tabs, sort, note controls, composer.
- Enter in the composer: sends the note.
- Enter or Space on an Apply patch control: runs the patch.
- Enter or Space on the resolve toggle: flips the resolved state.
- Escape within the sort menu: closes it and returns focus to the sort control.

## Motion (functional)
- New notes fade in at the top of the list, or at the bottom when sorted oldest first.
- A patch note shows a spinner while its patch runs.
- The sort menu drops down from its control.
- Reduced motion: no transitions.

## Accessibility
- Filters behave as a tab set; the sort control is a radio menu; the resolve control is a toggle reporting pressed or not.
- Timestamps are rendered as machine-readable time elements, with full text available.
- The list announces new notes politely.
- The working state of Apply patch is announced.

## Responsive
- Fills a side column. When the container is narrow, it becomes a full-height sheet.
- No horizontal scrolling at phone width.

## Composition hints
Card (panel), tabs (filters), menu with radio items (sort), list (notes), avatar, button (apply patch, send), toggle button (resolve), text field (composer), live region helper, spinner.

## Acceptance checks
1. A note whose text starts with `/patch` shows an Apply patch control; other notes do not.
2. Pressing Apply patch calls `on_run_patch(note)` once and keeps the control disabled until the returned promise settles.
3. Choosing the Patches filter shows only notes whose text starts with `/patch`.
4. Typing text and pressing Enter in the composer calls `on_note_added` with `writer` set to `acting_writer`, then clears the field.
5. Escape within the sort menu closes it and returns focus to the sort control.
6. Resolving a note flips its pressed state and updates its appearance without removing it from the list.
7. With no notes, the empty line is shown and no list items are rendered.
8. A rejected `on_run_patch` promise returns the control to idle and shows an error line under that note.
9. Timestamps are rendered in a time element carrying a machine-readable value.
10. At phone width the panel becomes a full-height sheet with no horizontal scrolling.
</content>
