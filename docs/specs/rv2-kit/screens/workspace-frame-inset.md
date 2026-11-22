# Workspace Frame (Inset)

Source item: affectUI Pro `screens/app-shell` (functional reference only; see README).

## Purpose
The application frame for an assistant, internal tool, or dashboard: a collapsible sidebar at the left, a rounded page panel at the centre, and a resizable side area at the right that holds tools, a terminal or browser, notes, and suggested sources. Other screens in this set are built on this frame, so they can move between frames without changes inside them.

## Data
| Field | Type | Meaning |
|---|---|---|
| content | node | The page inside the rounded panel. |
| current_item | string | Label of the open menu item or project. |
| on_navigate | (label: string) => void | A menu item or project was activated. |
| workspace_name | string | Defaults to a neutral placeholder name. Its first letter is the logo mark. |
| account | { name; avatar_url?; email? } | Account block. Its menu opens with the picture, name, and email, followed by entries for locale, theme, connected services, preferences, installed apps, release notes, help, and sign-out. |
| menu_items | { item_label; item_icon }[] | Top menu. Defaults to New chat, Folders, and Integrations. |
| project_list | string[] | Project list beneath the menu. |
| chat_list | { id; title; pinned?; unread?; project_id?; section? }[] | Chats. When supplied, each project turns into a collapsible folder that holds its chats. |
| on_open_chat | (chat) => void | A chat was activated. Defaults to `on_navigate` with its title. |
| sidebar_width | { default?; min?; max? } | Sidebar width. Dragging its edge stays within min and max; double-clicking the edge restores the default. |
| top_bar | { title: node; actions?: node } \| false | Full-width header above the sidebar and page: sidebar toggle and search, page title (as the bar title), and actions. |
| side_cards | boolean | Defaults to true. Shows the "what's new" and "getting started" cards at the foot of the sidebar. |
| work_area | options or instance | Tabs on the right, resizable and collapsible like the sidebar. Options: `tabs` (own tabs or artifact tabs), `default_tabs`, `initial_tabs`, `artifacts` (content keyed by file name), `on_open_artifact`, `on_artifact_change`, `new_tab` (start options, or `false`). |
| side_panel | node | Plain content shown in the work area as its one tab. |
| panel_title | string | Defaults to "Workspace". The name of the work area, used by its tab list and its open and close controls. |
| panel_heading | node | Tab label of a plain panel, when it differs from `panel_title`. |
| default_panel_open | boolean | Defaults to true. When false, the work area begins collapsed and its icon sits at the top-right. |
| panel_width | { default?; min?; max? } | Work area width with default, min, and max. With plenty of room, the default and max scale up together, so the work area keeps the same share of the page. |
| frame_style | "inset" \| "flat" | Defaults to "inset". "Flat" runs the page edge to edge. A host-level setting can switch every frame at once. |
| class_name | string | Extra class on the root. |

The frame also exports building blocks for its pages: an empty-state block, a spinner, a skeleton, and a loading wrapper, plus a hook that returns the work area's open, show, open-new, and artifact-change functions.

## Anatomy
Reading order:
1. A header row across the full width: sidebar toggle, search trigger, bar title (the page title), actions, and the work area title.
2. Sidebar (left, below the header): workspace switch with logo mark, search, top menu, projects or folders, chats grouped under Pinned and Chats, optional side cards at the foot, and the account block.
3. Page panel (centre): a rounded panel on the secondary surface holding `content`.
4. Work area (right): a tab strip with a "+" control, then the active tab's content.
5. Splitters: a draggable edge between the sidebar and the page, and another between the page and the work area.

## States
- **Sidebar open or folded.** Folded collapses it to a narrow icon rail; the header toggle switches between the two.
- **Sidebar being resized.** Dragging the edge keeps the width within min and max.
- **Work area open or folded.** When folded, its icon sits in the top-right corner of the header; clicking it reopens the area.
- **Work area empty.** No tabs open: the area closes after its last tab is closed, and `default_tabs` decides what opens again.
- **Tab active or inactive.** One active tab; the others keep their state.
- **Artifact tab.** Its kind decides the viewer: document, sheet, folder listing, code, or image.
- **Chat row menu open.** See Workspace Frame (Flat).
- **Narrow width.** The sidebar and work area become drawers.

## Interactions
- Header toggle: click or Enter folds or expands the sidebar.
- Sidebar edge: pointer drag resizes; double-click resets to the default width; when focused, arrow keys resize it, Shift for larger steps, and Home and End for min and max.
- Work area tabs: a single tab stop; left and right arrows move between tabs and bring the focused tab forward; Delete or Backspace closes the focused tab, and its neighbour takes focus.
- The "+" control in the tab strip: Enter or Space opens a new tab with start options (tools, sources, open action).
- Work area edge: the same keys as the sidebar edge.
- Escape closes menus first, then drawers.
- Chat row: click opens it; hovering shows a pin control; the more menu opens from its own control.
- Account block: opens its menu.

## Motion (functional)
- Folding the sidebar and folding the work area move their edges; the page panel keeps its content and only changes width.
- A new tab appears at the end of the tab strip and comes to the front.
- Folder open and close reveal their children in place.
- Dragging a chat follows the pointer and shows its drop line.

## Accessibility
- Work area tabs form a tab list; each tab controls a panel; each close control is labelled with the tab's name; the "+" is labelled "New tab".
- Splitter edges are focusable separators, with their current value announced.
- The work area is a labelled landmark; when folded, it is inert.
- Icons are hidden from assistive technology when their control has a name.
- Folded icons carry their names through tooltips.
- Drawers trap focus while open and return it to the element that opened them.

## Responsive
- When the container is narrow: the sidebar moves into a left-side drawer and the work area into a right-side drawer. Its tabs and toolbars shrink to a small minimum width.
- The tab strip narrows first (tab names fade), then the strip scrolls inside itself.
- The frame never scrolls sideways; wide tables or sheets scroll inside their own region.
- The frame fills its container.

## Composition hints
- Navigation list, grouped sections, folder rows (collapsible).
- Chat row with pin, unread dot, and context menu.
- Resizable split panes (two edges).
- Tab strip with close buttons and an add control.
- Avatar with initials and a menu.
- Tooltip for folded icons; drawer or sheet for narrow widths.
- Empty state, skeleton, and loading wrapper for pages that use the frame.
- Document, spreadsheet, folder-listing, and code viewers as artifact tab kinds.
- Card list for the "what's new" and "getting started" cards.

## Acceptance checks
1. Dragging the sidebar edge changes its width and stops at the configured min and max.
2. Double-clicking the sidebar edge returns it to its default width.
3. With `frame_style` "flat", the page panel runs edge to edge with a hairline instead of a rounded inset.
4. Folding the work area leaves its icon in the header's top-right corner, and the page widens to fill the space.
5. Pressing the "+" control opens a new tab that becomes the active tab.
6. Closing the last work area tab folds the area and moves focus to its open control.
7. Arrow keys on the focused work area tab move the active tab; Delete closes it, and focus lands on the neighbour.
8. At narrow width, the sidebar and work area are both drawers; Escape closes the open one.
9. Passing `side_cards` as false removes both cards from the sidebar foot.
10. Clicking a menu item calls `on_navigate` with its label and marks it active.
