# Assistant Chat With Sidebars

Source item: affectUI Pro `screens/agent-chat-rail` (functional reference only; see README).

## Purpose
A plain assistant chat with web search, for products that have several sections. Chats are grouped by project and can be narrowed to one project. Results the assistant produces open in a work area on the right.

## Data
- `workspace_name`, `account`.
- `section_list`: list of `{ id, entry_label, icon_key }` for the icon column.
- `project_list`: list of `{ id, entry_name }`. `selected_project_id`: a string, or null for all projects.
- `chat_list`: list of `{ id, entry_title, project_id?, is_pinned?, updated_at }`.
- `active_chat`: messages. Assistant messages carry tool activity (for example, web searches) and output files.
- `output_list`: list of `{ entry_name, output_kind: "document" | "sheet" | "folder" | "code", content }`.
- `approval_mode`: `"auto" | "approve"`.
- `is_loading`, `is_empty`.
- Callbacks: `on_navigate(entry_label)` (icons, sidebar items, projects), `on_send`, `on_select_chat`, `on_select_project`, `on_open_output`.

## Anatomy
1. Icon column: the workspace's initial at the top, one icon per section, and the account avatar at the foot. The open section is marked as current.
2. Chat list column: a project switcher at the top (all projects, or one), then pinned chats, then chats grouped under project headings. Chats from one project appear only when that project is selected.
3. Conversation: messages; consecutive web search steps fold into a single line; the answer types in; a stop control ends generation; output files appear as chips. The composer sits at the foot, with the approval-mode control.
4. Work area (right): resizable by dragging its edge. Tabs open outputs. Closing the last tab collapses the area.

## States
- **all projects**: every chat is listed, grouped.
- **one project**: only that project's chats are listed.
- **empty** (`is_empty`): the chat column says there are no chats; the conversation is empty; the work area holds no files. The composer stays, but sending waits.
- **loading** (`is_loading`): placeholder messages and a placeholder work area, as in the Orchestrator Thread's loading state; sending waits.
- **searching**: a tool step is shown folded; expanding it lists the searches.
- **narrow**: the icon column and chat list slide in together as one drawer.

## Interactions
- Tab moves through the icon column, the chat list, the conversation, and then the work area.
- Ctrl or Cmd+K opens search from anywhere.
- Up and down arrows move through the project switcher and menus.
- Escape closes a menu, or the drawer on narrow screens.
- Work area edge: drag to resize; when the edge has focus, arrow keys also move it.
- Enter in the composer sends.

## Motion (functional)
- The drawer slides in on narrow screens.
- Tool steps fold and unfold smoothly.
- Reduced motion: the drawer and folds change without motion.

## Accessibility
- Navigation: the icon column is a named landmark; each icon announces its name on focus and on hover.
- The active section carries the current-page marker.
- The project menu shows the selected project with a check.
- The conversation follows the Orchestrator Thread's accessibility rules (log role, polite updates, busy while streaming).

## Responsive
- The icon column stays on the page; the chat list forms the left part of the main area, separated from the conversation by a draggable edge.
- Narrow screens: the icon column and chat list become a drawer, and the work area becomes a sheet.
- Touch targets are enlarged for touch input.

## Composition hints
Icon rail; project switcher menu; chat list with headings; conversation (plain-chat variant of the thread); tool step group; output chip; resizable work area with tabs; composer with approval-mode control; drawer; skeleton; empty state.

## Acceptance checks
1. Choosing a project shows only that project's chats in the chat list.
2. Chats appear under their project headings in all-projects mode.
3. A group of web searches folds into one line and expands when activated.
4. On a narrow screen, the icon column and chat list appear as one drawer, and Escape closes it.
5. Clicking an output chip opens it in a work area tab.
6. Dragging the work area edge resizes it; the minimum width keeps the tabs usable.
7. With `is_empty` set, the empty note appears and sending stays available.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** icon rail, chat list column, conversation, and a right work area that holds file previews as tabs.

**At rest (gallery demo):**
- Icon rail: workspace initial, five section icons, account avatar at the foot. One section is current.
- Chat list: project switcher set to all projects. One pinned chat. Three project groups holding three, two and two chats (seven in total). Eight chats listed in all.
- Conversation: four messages (two user, two assistant). The second assistant message holds a folded web-search group of three searches, an answer paragraph, and two output chips.
- Work area open with two tabs: a sheet tab (active) and a document tab. The sheet has a header row, four data rows and four columns. A formula bar above it shows the active cell.
- Composer at the foot with the approval-mode control.

**States at rest:** all projects; search group folded; work area open with the sheet tab active.

**Arrangement:** icon rail, chat list, conversation and work area in one row. The conversation keeps its readable column width; the work area takes the remaining right-hand width.

**Motion, settled frame:** nothing moves. The folded group expands only when activated.

**Checks (continued):**
8. At rest the work area shows the sheet tab active beside the document tab. The sheet has a header row, four data rows, four columns, and a formula bar showing the active cell.
9. Activating an output chip whose tab is already open makes that tab current and adds no tab.
10. At rest the chat list shows one pinned chat above three project headings holding seven chats.
11. At 1280 wide the icon rail, chat list, conversation and work area are visible together, and the conversation holds its readable width.
