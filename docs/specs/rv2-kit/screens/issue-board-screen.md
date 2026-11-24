# Issue Board Screen

Source item: affectUI Pro `screens/kanban` (functional reference only; see README).

## Purpose
A team sorts its issues by priority into columns, drags cards between them to change
their state, and can switch to a flat list with filters and sorting. Opening an issue
edits it in place.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_name | string | Sidebar label. |
| account | { name; avatar_url? } | Account block. |
| board_columns | { id: string; column_label: string; priority_tone: "low" \| "medium" \| "high" \| "done" }[] | Left to right. `priority_tone` picks the priority mark. |
| issues | { id; column_id: string; title: string; description?: string; issue_status?: "review" \| "attention" \| "progress" \| "fixed"; comment_count?: number; attachment_count?: number; assignee?: string; labels?: string[] }[] | Each issue sits in one column. |
| view | "board" \| "list" | Layout radio. |
| filter | { status?; assignee?; label?[] } | Active filters. |
| sort | { key: "priority" \| "updated" \| "title"; dir: "asc" \| "desc" } | Sort menu. |
| is_loading | boolean | Secondary cards (board) or rows (list) under the real column heads. |
| is_empty | boolean | No issues: bar and tabs stay; board or list replaced by one empty state with a New issue button. |
| on_create | (draft: { title; description; column_id }) => void | Submitted from the New issue window. |
| on_move | (issue_id, to_column, before_id?) => void | Drag finished. |
| on_update | (issue_id, patch) => void | Edits from the open issue. |
| on_open | (issue_id) => void | Card opened. |
| on_navigate | (label: string) => void | Sidebar item. |

## Anatomy
Reading order:
1. Sidebar, left.
2. Top bar of the page: page title, a view radio (Board, List), Filter and Sort menus,
   and New issue.
3. Board: one column per entry, each with its heading, count, a quiet add control that
   shows on hover, and its cards stacked top to bottom.
4. Card: title, a short description line when present, status mark with its word,
   counts for comments and attachments.
5. List view: rows with title, issue_status, priority, assignee; same filters and sort.
6. Issue panel (opens over the page when an issue is opened): title and description
   editable in place; issue_status, priority, assignee, labels and project as menus; activity
   list below.
7. New issue window: centred dialog over a scrim with title, description, column and
   Create.

## States
- **Board loaded.** Default.
- **List view.** Same data as rows.
- **Loading.** Secondary cards in each column, or secondary rows; column headings are real.
- **Empty.** The bar and tabs stay; one empty state titled "No tasks yet" with a line on
  what shows here and a New issue button.
- **Card dragging.** The dragged card is shown in place with reduced emphasis; a line
  indicates the drop position.
- **Column hover.** Its add control appears.
- **Issue open.** Panel visible; its fields are editable; changes are logged in the
  activity.
- **New issue window open.** Dimmed page; focus in the title field.
- **Filter or sort menu open.** Menu with checked items.

## Interactions
- Pointer drag: press and move a card; release over a column sets its column; the drop
  line shows the position among cards.
- Keyboard move: on a focused card, Space picks it up; arrow keys move it between
  columns and positions; Space or Enter drops; Escape cancels.
- Tab: moves through cards; Filter, Sort and view controls are reachable in order.
- Enter on a card: opens the issue panel.
- Cmd/Ctrl+Enter: sends a comment in an open issue, or creates the issue in the New issue
  window.
- Escape: closes the New issue window, then the panel.
- Column add control: opens the New issue window with that column preselected.
- View radio: arrow keys choose Board or List.

## Motion (functional)
- Columns and cards enter one after another on first load.
- A dropped card settles into its new place; the cards around it move to make room.
- The New issue window rises over a scrim.
- Under reduced motion all movement is off, and drops take effect at once.

## Accessibility
- Each column is a named section, and every card is a focusable item carrying its label.
- The view switch is a set of radio choices; the Filter and Sort popups list their options with check marks.
- Glyphs for status and priority are ignored by assistive tech; each one's word appears next to it.
- Moves made by keyboard are announced with the new column and position.
- The issue panel traps focus while open and returns it on close.

## Responsive
- Board scrolls sideways inside its own region when columns do not fit.
- On phone width the top bar wraps; list rows keep title and status only.
- The page never scrolls sideways as a whole.

## Composition hints
- Workspace frame.
- Kanban board: columns, drag-and-drop with keyboard alternative, drop indicator.
- Issue card: title, description, badge for status, counts.
- Segmented control for view; menu with checked items for filter and sort.
- Side or modal panel for an issue with inline editable text and menus.
- Dialog for New issue; empty state; skeleton cards.

## Acceptance checks
1. Dragging a card onto another column calls `on_move` with its id and the target column.
2. Space then arrow keys then Space moves a focused card to another column and calls `on_move`.
3. Escape during a keyboard move cancels it and makes no call.
4. Pressing Enter on a card opens the issue panel with its title and description editable.
5. Cmd/Ctrl+Enter in the New issue window calls `on_create` with the title, description and column.
6. The column add control opens the New issue window with that column preselected.
7. Choosing List shows the same issues as rows and keeps the active filter.
8. With `is_empty` set, the board is replaced by one empty state whose button opens the New issue window.
9. With `is_loading` set, every column shows secondary card placeholders under its real heading.
10. At phone width the top bar wraps and list rows show only title and status.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** four-column priority board with cards, a top bar carrying the view switch, filter, sort and New issue.

**At rest (gallery demo):**
- Top bar: page title, board and list switch with Board selected, Filter, Sort by, New issue.
- Four columns, left to right: Low, Medium, High, Done. Each has a heading with its count.
- 11 cards in all: Low 3, Medium 3, High 3, Done 2. Each card shows its title, a description line, its status word, and comment and attachment counts when they are not zero.
- Each column is a full-height surface that runs to the bottom edge of the board region.

**States at rest:** board view; no card being dragged; no panel open; no menu open.

**Arrangement:** four equal columns side by side under the top bar. Cards stack from the top of each column.

**Motion, settled frame:** nothing moves at rest.

**Checks (continued):**
11. At rest the board shows four columns and 11 cards. Each column holds at least two cards, and each heading's count matches its cards.
12. Each column surface runs to the bottom edge of the board region at every width where the four columns fit, so the lower part of the board is column surface and not an empty panel.
13. Each card shows its title, its description line when present, its status word, and its comment and attachment counts when non-zero.
14. At 1280 wide all four columns are fully visible. The board scrolls sideways inside itself only when the columns do not fit.
15. Every card shows its priority mark and status word at rest.
