# Workspace Frame (Flat)

Source item: affectUI Pro `screens/app-shell-flat` (functional reference only; see README).

## Purpose
The application frame for tools and dashboards that need a quiet, full-bleed layout: a navigation sidebar, a header along the top, and the page beside them. Every screen in the product sits inside it, so the page area is the only part that changes between screens.

## Data
| Field | Type | Meaning |
|---|---|---|
| top_bar | { title: node; actions?: node } \| false | Title and action row. `false` removes the header. Default: the open page's name, a share control, and a more menu. |
| content | node | The page shown beside the sidebar. |
| current_item | string | Label of the entry that is currently open. |
| on_navigate | (label: string) => void | Receives the label of the activated menu item or project. |
| workspace_name | string | Defaults to a neutral placeholder name. Its first letter becomes the logo mark. |
| account | { name: string; avatar_url?: string; email?: string } | Account block. Without an image, initials are shown. Its menu offers locale, theme, connected services, preferences, installed apps, release notes, help, and sign-out. |
| menu_items | { item_label; item_icon }[] | Top menu items. Defaults to New chat, Folders, and Integrations. |
| project_list | string[] | Project list beneath the menu. |
| chat_list | { id; title; pinned?; unread?; project_id?; section? }[] | Chats in the sidebar. When supplied, each project becomes a collapsible folder that holds its chats; remaining chats group under pinned and recent headings. |
| on_open_chat | (chat) => void | A chat was activated. Defaults to `on_navigate` with its title. |
| menu_groups | { title: string; items: { item_label; item_icon }[] }[] | Lists shown beneath the menu in place of Projects, for example Tools or Pipelines. |
| work_area | options or instance | Optional right-hand work area with tabs (see Workspace Frame (Inset)). |
| side_panel | node | Plain content shown in the work area as its one tab. |
| class_name | string | Extra class on the root. |

## Anatomy
Reading order:
1. Sidebar, left: the workspace name and logo mark, the top menu, the project list or folder-style chat list, pinned and recent chats, and the account block at the foot.
2. Header, across the top of the page area: the sidebar toggle, the page title, and action buttons.
3. Page area: the children, filling the remaining space and scrolling on its own.
4. Work area, right (only when enabled): the tab strip and the active tab's content.

Hairlines, not cards or gaps, separate the sidebar from the page. The page area is flat: no rounded panel, no inner frame.

## States
- **Sidebar expanded.** The default. Labels and sections are visible.
- **Sidebar folded.** A rail of icons; each icon names itself in a tooltip on hover and focus. The toggle returns it to expanded.
- **Chat folder open or closed.** Toggled by its row; the chats inside show or hide.
- **Item active.** The item matching `current_item` is marked current.
- **Chat row menu open.** Rename, pin, mark unread, move to project, share, copy link, open in new window, archive. Closes on a choice, an outside click, or Escape.
- **Drawer (narrow width).** The sidebar is off-canvas and slides in over the page from the left when its toggle is pressed; the work area slides in from the right. An outside tap or Escape closes it.
- **Header omitted.** `top_bar` is `false`; the page starts at the top.

## Interactions
- Sidebar toggle: click or Enter folds or expands the sidebar.
- Menu item or project: click or Enter calls `on_navigate` with its label.
- Chat row: click opens it; hovering reveals the pin control; the more-menu opens from its own control.
- Chat drag: drag to reorder within a list or drop into a folder. The keyboard alternative is the row menu's "move to" entry.
- Edge handle of the work area (when present): arrow keys resize it; Shift with the arrows gives larger steps; Home and End jump to the minimum and maximum.
- Escape closes the open menu, then the drawer.
- Account block: click opens its menu, with the picture, name, and email at the top.

## Motion (functional)
- Folding the sidebar and the drawer slide show where the content went; they change the page's own layout only by the width the sidebar gives back.
- Folder open and close reveal their chats in place and do not move other sections sideways.
- A dragged chat follows the pointer; a line marks its drop position.

## Accessibility
- Sidebar items, projects, and the toggle are buttons with visible labels.
- Folded icons get accessible names through tooltips.
- Icons inside labelled controls are hidden from assistive technology.
- The work area is a labelled region; while folded away, it is inert and out of the tab order.
- Focus returns to the control that opened a menu or drawer when it closes.
- The header title is the page's level-one heading.

## Responsive
- When the container is narrow: the sidebar becomes a drawer from the left; the work area becomes a drawer from the right.
- Header actions collapse into the more menu when they do not fit.
- The frame never scrolls sideways; wide content scrolls inside its own region.
- The frame fills its container; the host sets the height.

## Composition hints
- Navigation list with grouped sections (menu, folders, chat list).
- Avatar with initials, and a menu.
- Context menu for chat rows.
- Drawer or sheet primitive for narrow widths.
- Tooltip for folded icons.
- Tab strip (for the work area).
- Button and icon button for header actions.
- Resizable splitter for the work area edge.

## Acceptance checks
1. With `top_bar` set to `false`, no header renders, and the page content starts at the top of the page area.
2. Clicking a menu item calls `on_navigate` with its label and marks it current.
3. Folding the sidebar shows an icon rail; hovering an icon shows its name as a tooltip.
4. Clicking a folder row toggles its chats without changing the order of other sections.
5. Opening a chat row's menu and pressing Escape closes it and returns focus to the row's control.
6. At narrow width the sidebar is off-canvas until its toggle is pressed, and closes on an outside click.
7. Dragging a chat into a folder moves it there; the same move is possible from the keyboard through the row menu.
8. The page area never produces a horizontal scroll of the whole frame.
9. The work area edge resizes with the arrow keys and jumps to min and max with Home and End.
10. A folded work area leaves no focusable element inside it.
