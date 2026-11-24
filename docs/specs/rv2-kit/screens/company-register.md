# Company Register Screen

Source item: affectUI Pro `screens/crm` (functional reference only; see README).

## Purpose
A sales or partner team keeps its companies in one editable table. The open view can be switched among company list, deals, contacts, and renewals, and a cell can be edited directly in the table without leaving it.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_name | string | Sidebar label. Default "Juniper Works". |
| account | { company_name; avatar_url? } | Account block at the foot of the sidebar. |
| company_records | { id: string; company_name: string; company_category: string; website_domain: string; account_owner: string; deal_value: number; pipeline_stage: string; primary_contact: string; last_contact: string; logo_url?: string }[] | The records behind all four views. `id` selects the logo; without one, the first letter is shown. |
| views | ("Company CRM" \| "Deals" \| "Contacts" \| "Renewals")[] | Tabs for open views. Each view is a column selection and filter over `company_records`. |
| sections | { label: string; items: string[] }[] | Sidebar groups: primary navigation first, then pipelines (for example "New business", "Renewals"). |
| is_loading | boolean | Table rows become secondary placeholders shaped like real rows, under the real column headings. |
| is_empty | boolean | The table for the open view is replaced by an empty message carrying a title, a line of text and an add button. |
| on_add | (view: string) => void | Fired by the empty-state action and by "Add row", passing the open view. |
| on_navigate | (label: string) => void | A sidebar item was activated. |
| on_cell_change | (id, column, deal_value) => void | An edit was committed in a cell. (Own addition; the host must persist the record.) |

## Anatomy
Reading order:
1. Sidebar, left: workspace, navigation with counts, the pipeline list, and the account.
2. Breadcrumb in the page header: the section, then the current page name.
3. View tab strip: one tab per view; the open tab is marked, the rest remain available; a "+" opens a new view.
4. Toolbar line above the table: an "Add row" button and a hint ("Double-click a cell to edit").
5. Editable table: a header row with the column names; each row shows the company name with its logo, category, website domain, account owner, deal value, stage, and last contact.
6. Footer row inside the table: add row.

The page header is a single line. The table is the main region of the page and fills the remaining height, scrolling inside itself.

## States
- **Table loaded.** The default.
- **Table loading.** Secondary rows shaped like the real ones, under the real column headings. Tabs and sidebar stay live.
- **Empty view.** Tabs and toolbar stay; the table is replaced by an empty state titled for the view (for example "No companies yet" or "No deals yet") with an add button for that view.
- **Cell viewing.** The default display of a value.
- **Cell editing.** Entered by double-click, F2, or Enter on a selected cell. An input appears in place with the current value selected.
- **Cell committed.** Enter or Tab writes the value; the change is announced.
- **Cell cancelled.** Escape restores the previous value.
- **Row selected.** The row holding keyboard focus shows a selection frame.
- **Stage cell.** Shows the stage as a labelled status; the stage choices come from a fixed menu supplied by the host.

## Interactions
- Pointer: a click selects a cell; a double-click edits it; a click outside commits the edit.
- Keyboard in the table: arrow keys move the selected cell; Tab and Shift+Tab move right and left, wrapping to the next row at the end; Enter moves down.
- Typing on a selected cell replaces its value and starts editing.
- Ctrl/Cmd+C and Ctrl/Cmd+V copy and paste a cell or range as tab-separated text; Ctrl/Cmd+Z undoes the last committed edit.
- Tab bar: arrow keys move between view tabs; Enter opens one.
- "Add row": click or Enter inserts an empty row at the end and focuses its company cell for editing.
- Sidebar item: click or Enter calls `on_navigate`.
- Escape cancels an edit first, then clears the selection.

## Motion (functional)
- A newly added row enters at the end of the table and receives focus.
- Rows stay still while the person types in another row.
- Sorting or switching views reorders rows in place rather than replacing the whole table.

## Accessibility
- The table uses table roles; column headers are linked to their cells.
- Each cell's name includes its column and row context when focused.
- Editing announces "editing", and the commit or cancel outcome, in a polite region.
- Tabs form a tab list; the open tab is selected.
- Stage labels carry their text rather than a visual cue alone.
- Logos and decorative marks are hidden from assistive technology.

## Responsive
- At phone width the sidebar becomes a drawer; the table scrolls sideways inside its own box, with the company name column kept first; the toolbar hint is hidden.
- The view tab strip scrolls inside itself when it overflows.
- The page as a whole never scrolls sideways.

## Composition hints
- Workspace frame (sidebar and page area).
- Editable data table: sticky header, keyboard cell navigation, in-place editor, copy and paste, undo.
- Tab strip with an add control.
- Company logo chip (image or initial letter).
- Stage badge with a fixed set of tones.
- Empty state, skeleton rows, a toast for the save outcome.
- Button for "Add row".

## Acceptance checks
1. With `is_loading` set, secondary rows appear under the real column headings, and the tabs remain clickable.
2. With `is_empty` set, the table is replaced by an empty state titled for the open view; its button calls `on_add` with that view.
3. Double-clicking a cell opens an editor showing the old value; Enter commits and calls `on_cell_change` with the new value.
4. Pressing Escape during an edit restores the previous value and makes no change call.
5. Arrow keys move the selected cell across rows and columns; Tab at the last column moves to the first cell of the next row.
6. Copying a selected cell and pasting it into another gives the same text value.
7. Undo after a committed edit restores the previous value.
8. Clicking "Add row" appends an empty row and focuses its company cell.
9. Switching views with the tab strip changes the visible columns and rows without a full reload.
10. At phone width the page itself does not scroll sideways; only the table scrolls inside its box.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** workspace frame with a one-line header, a view tab strip and toolbar, and a full-height editable table.

**At rest (gallery demo):**
- Single-line breadcrumb header.
- Tabs for the four views, Company CRM open, plus the add control.
- Toolbar: Add row button and the double-click hint.
- Table: seven columns (company with logo chip, category, domain, owner, value, stage, last contact) and ten company rows. Each row holds distinct values. A footer row holds Add row.
- Sidebar: primary navigation with counts (four items), and a pipelines group with four entries.
- No cell is in edit mode at rest.

**States at rest:** table loaded; Company CRM view open; no selection frame.

**Arrangement:** table fills the height under the toolbar and scrolls inside itself. Its header row stays fixed while rows scroll.

**Motion, settled frame:** nothing moves at rest.

**Checks (continued):**
11. At rest the table shows ten company rows and seven columns, headed Company, Category, Domain, Owner, Value, Stage and Last contact.
12. At 1280 wide no Category or Last contact cell is cut mid-word. A value too long for its cell ends in an ellipsis and its full text is in a tooltip.
13. Every column header reads in full at 1280 wide, including Last contact.
14. At 1280 wide all seven columns fit without a horizontal scroll of the table.
15. The sidebar shows the primary navigation with counts and a pipelines group with four entries.
16. The table fills the height left under the toolbar, and its header row stays at the top while rows scroll.
