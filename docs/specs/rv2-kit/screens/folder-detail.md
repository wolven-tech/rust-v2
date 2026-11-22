# Folder Detail Screen

Source item: affectUI Pro `screens/folder` (functional reference only; see README).

## Purpose
Inside a single folder of a shared file area: the folder's name and location, its files as cards with previews, and a way to add files or start a conversation about the folder.

## Data
| Field | Type | Meaning |
|---|---|---|
| folder_name | string | The open folder. The title, crumbs, and file list follow it. Default "Q3 Campaign". |
| crumbs | string[] | Path from the root to the folder. |
| workspace_name | string | Sidebar label. |
| account | { name; avatar_url? } | Account block. |
| files | { id; name: string; kind: "doc" \| "sheet" \| "pdf" \| "image" \| "other"; updated_at: string; subtitle: string; preview_url?: string }[] | Files in the folder. |
| share_state | "private" \| "shared" | Folder sharing state, shown beside the title. |
| is_loading | boolean | Placeholder cards in the grid, matching the shape of real file cards; the title, breadcrumb and actions remain visible. |
| is_empty | boolean | No files: an empty state with a hint and a button that starts a conversation about the folder. Uploads still land in the list. |
| on_upload | (files: File[]) => void | Upload control and drop target. |
| on_share | () => void | Share action. |
| on_open_file | (id) => void | A file card was opened. |
| on_start_chat | () => void | The empty state's button. |
| on_navigate | (label: string) => void | Sidebar item or breadcrumb activated. |

## Anatomy
Reading order:
1. Sidebar, left (see the workspace frame).
2. Breadcrumb trail: root, parents, current folder name.
3. Title row: folder name, a sharing-state label, and the Share and Upload actions.
4. File grid: one card per file. Each card has a preview area (thumbnail or kind glyph), the file name, a kind label, and a subtitle line with owner and time.
5. Empty state, when `is_empty` is set, in place of the grid.

## States
- **Loaded.** The default.
- **Loading.** Secondary card shapes in grid positions; the title, crumbs, and actions are unchanged.
- **Empty.** The grid is replaced by the empty state; upload stays usable.
- **File hover or focus.** The card rises slightly and shows its quick actions.
- **Upload in progress.** Each new card appears at the top with a progress line.
- **Drop target active.** While files are dragged over the page, the grid shows a drop area.
- **Breadcrumb hover.** Parent crumbs are links; the current one is not.

## Interactions
- File card: click or Enter opens the file in the work area or a viewer.
- Space on a focused card toggles its selection; Shift+click selects a range; Ctrl/Cmd+click adds to the selection.
- Arrow keys move focus between cards in the grid; Home and End jump to the first and last.
- Type-ahead: typing the start of a file name moves focus to the first matching card.
- Ctrl/Cmd+A selects all visible cards.
- Delete or Backspace on selected cards offers removal, after a confirmation.
- Breadcrumb: click or Enter navigates to that level.
- Dropping files on the page starts an upload for each file.
- Escape clears the selection.

## Motion (functional)
- New files enter at the top of the grid; the others shift down.
- A selected card lifts slightly to show it is chosen.
- Removing a file lets the cards behind it close the gap.

## Accessibility
- The grid is a listbox with multiple selection; each card is an option with its name and kind.
- The breadcrumb is a navigation landmark containing an ordered list; the current crumb carries the current-page marker.
- Upload status and completion are announced politely.
- Thumbnails are decorative when the name and kind are present.

## Responsive
- At phone width, the grid becomes a single column of wider cards; the title row stacks, and the action buttons move beneath the title.
- The breadcrumb collapses its middle levels into a menu.
- The page never scrolls sideways.

## Composition hints
- Workspace frame.
- Breadcrumb with overflow menu.
- File card with preview tile, kind label, and quick actions.
- Multi-select listbox or grid.
- Empty state, skeleton cards, a drop zone, an upload progress line.
- Buttons for Share and Upload; a badge for the sharing state.

## Acceptance checks
1. With `is_loading` set, the grid positions show placeholder cards, and the title and breadcrumb stay as they are.
2. With `is_empty` set, the grid is replaced by "No files yet", and its button calls `on_start_chat`.
3. Dropping files onto the page starts one upload per file, and each file appears in the grid while it uploads.
4. Shift+click selects a contiguous range of cards.
5. Typing the first letters of a file name moves focus to the first matching card.
6. Enter on a file card calls `on_open_file` with its id.
7. Clicking a parent breadcrumb calls `on_navigate` with that folder's path.
8. Escape clears the selection.
9. At phone width the title actions move below the title, and no horizontal page scroll occurs.
10. The grid announces changes in selection count in a polite region.
