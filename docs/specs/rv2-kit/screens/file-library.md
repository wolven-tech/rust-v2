# File Library Screen

Source item: affectUI Pro `screens/files` (functional reference only; see README).

## Purpose
The top level of a shared file area. The person sees their folders and the file templates available to start from, and can search, sort, switch between grid and list views, and create a folder or a template.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_name | string | Sidebar label. |
| account | { name; avatar_url? } | Account block. |
| folders | { id; name: string; scope: "shared" \| "private"; file_count: number }[] | Folder list in the left column. |
| templates | { id; name: string; kind: "doc" \| "sheet" \| "pdf" \| "other"; owner: string; scope: "mine" \| "shared" \| "private" }[] | Starting files. |
| folder_filter | "all" \| "shared" \| "private" | Folder tab. |
| template_filter | "all" \| "mine" \| "shared" | Template tab. |
| query | string | Search text across folders and templates. |
| sort_key | "name" \| "updated" \| "kind" | Sort order. |
| view | "grid" \| "list" | Layout toggle. |
| is_loading | boolean | Placeholders in both sections. |
| is_empty | boolean | No folders and no templates: each section shows its own empty state with an action. |
| on_create_folder | () => void | New folder action. |
| on_add_template | () => void | Add template action. |
| on_open_folder | (id) => void | A folder card was opened. |
| on_use_template | (id) => void | Template used to start a new file. |
| on_navigate | (label: string) => void | Sidebar item or project activated. |

## Anatomy
Reading order:
1. Sidebar, left.
2. Page title row: "Files", with a one-line explanation and a learn-more link.
3. Folders section: a section title with its count, filter tabs (All, Shared, Private), a "Create folder" button, and then the folder cards in a grid.
4. File templates section: a section title with its count, filter tabs (All, Mine, Shared with me), a type filter, an "Add template" button, and then the template rows or cards.
5. Shared toolbar across both sections: search field, sort menu, and the grid/list toggle.

Folder cards show a folder glyph, the folder name, a scope label, and the file count. Template items show a kind glyph with its kind code, the name, the owner, and a scope label.

## States
- **Loaded.** The default.
- **Loading.** Both sections show placeholders shaped like their cards.
- **Empty.** Folders: "No folders yet", followed by a New folder button. Templates: "No templates yet", followed by an Add template button. Tabs, search, and actions stay.
- **Filtered, no match.** Sections show "Nothing matches" and a clear-filter action.
- **Searching.** Results across both sections; each section count updates.
- **Grid or list.** The same items in two layouts; the choice is remembered per user.

## Interactions
- Folder card: click or Enter opens the folder.
- Template: click or Enter uses it to start a new file; its menu offers rename, duplicate, and remove.
- Filter tabs: arrow keys move; Enter selects.
- Search: typing filters both sections live; Escape clears the field.
- Sort menu: opens with Enter; arrows choose; Enter confirms.
- View toggle: a two-state control; Enter or Space switches it.
- Shortcut: "/" focuses search (a deliberate choice of this spec).
- Sidebar and project items: click or Enter call `on_navigate`.

## Motion (functional)
- Filtering reflows the cards in place; items that leave the filter fade out, and the rest close the gap.
- Switching between grid and list keeps each item's identity, so focus stays on the same item.

## Accessibility
- Each section is a labelled region; filter tabs form tab lists with counts.
- Folder cards and template items are buttons whose names include their scope and count.
- The search field has a visible label; the results count is announced politely.
- Sort and view controls state their current value.

## Responsive
- At phone width, folder cards become a single column; the filter tabs scroll inside their own strip; list view keeps only the name and scope.
- The toolbar wraps onto its own line.
- The page never scrolls sideways.

## Composition hints
- Workspace frame.
- Tab strip with counts for the filters.
- Folder card and template card or row with glyph tiles.
- Search field, sort menu, segmented grid/list toggle.
- Empty state with an action; skeleton placeholders.
- Buttons for create and add actions; a badge for scope.

## Acceptance checks
1. With `is_loading` set, both sections show placeholder cards, and their headings stay.
2. With `is_empty` set, each section shows its own empty state, and the matching button calls its callback.
3. Typing in search hides folders and templates whose names do not match, and updates each section's count.
4. Choosing "Shared" in the folder filter shows only shared folders.
5. Switching from grid to list keeps the focused item focused.
6. Enter on a folder card calls `on_open_folder` with its id.
7. Enter on a template calls `on_use_template` with its id.
8. Escape in search clears the query and restores all items.
9. Sorting by name orders folders and templates alphabetically within their sections.
10. At phone width there is no horizontal page scroll; the filter strip scrolls inside itself.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** two stacked sections, Folders and File templates, under one shared toolbar, each a grid of cards.

**At rest (gallery demo):**
- Title row: "Files", a one-line explanation and a learn-more link.
- Shared toolbar on one line: search field, sort control, grid/list toggle with grid selected.
- Folders section: heading with count 4; filter tabs (All, Shared, Private); Create folder button; four folder cards in one row, each with a folder glyph, name, scope label and file count.
- File templates section: heading with count 4; filter tabs (All, Mine, Shared with me); type filter; Add template button; four template cards in a grid, each with a kind glyph and its code, name, owner and scope label.
- Template names at rest: Brand guidelines, Campaign budget, Client onboarding kit, Weekly status report.

**States at rest:** loaded; All selected in both filters; search empty; grid view.

**Arrangement:** toolbar on one line across the top; folders section above templates section; both sections span the full page width.

**Motion, settled frame:** nothing moves at rest.

**Checks (continued):**
11. At 1280 wide every template name shows in full. A name that does not fit on one line wraps to a second line before it truncates. No demo name is truncated at rest.
12. Each folder card shows its file count and scope label. Each template card shows its owner and scope label.
13. At rest the page shows four folders and four templates, and each section heading's count matches its cards.
14. At 1280 wide the toolbar sits on one line above both sections. It wraps only below phone width.
