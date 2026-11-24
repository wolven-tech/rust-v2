# Integration Catalog

Source item: affectUI Pro `blocks/app-directory` (functional reference only; see README).

## Purpose
Lets a person find a third-party tool, read what it does, and connect it to their workspace. Connected tools are visible at a glance so the person knows what is already live.

## Data
- `integrations`: list of { id, name, vendor, description, category_ids: string[], connected: boolean, logo: node (supplied by caller), badges?: string[] }.
- `category_options`: list of { id, label, count }. A built-in "featured" and "all" category are provided by the component.
- `featured_ids`: ids shown under the featured category.
- `on_connect(id)`, `on_disconnect(id)`: the connect flow is owned by the caller; the component reports intent and shows the pending state while the caller resolves it.
- `pending_id`: the integration currently connecting, or null.
- `filter_groups`: list of { id, label, options }, with `active_filter_ids`, `on_filter_change`.
- `search_text`, `on_search_change`.
- The connected count is derived from `integrations`.

## Anatomy
In reading order:
1. Header: title, short intro, and a count of connected tools at the top right.
2. Search field spanning the width above the content, with a filter control at its right.
3. Left column: category list with counts; on narrow containers this becomes a row of chips.
4. Main grid: a section heading (featured or the chosen category) with its count, then integration cards.
5. Integration card: logo, name, vendor line, description, and a connect control. A connected card shows a "Connected" status instead, and offers disconnect on hover or focus.
6. Trailing "Discover more" card when the category has more items than shown, with a link to view all.

## States
- Default: featured category, no search text.
- Searching: the grid shows matching cards only; the heading shows the match count; no match shows an empty message with a clear-search action.
- Filtered: active filter chips appear above the grid.
- Connecting: the card's connect control shows a spinner and its label changes to a connecting word; the control keeps its width so the card does not jump.
- Connected: the card shows a positive status badge; hover reveals disconnect.
- Error: a card shows an inline message under its description and the connect control returns to its default label.
- Disabled: connect controls are inert; search and filters still work.

## Interactions
- Click connect on a card: calls `on_connect`, enters the connecting state.
- Click disconnect (on a connected card): asks for confirmation within the card, then calls `on_disconnect`.
- Click a category: switches the grid; search text is kept.
- Typing in search: filters live; Escape clears the field when it has text.
- Keyboard: Tab order is search, filter control, categories, cards (each card's connect control), "Discover more". Enter activates the focused control. Arrow keys move between categories when the category list has focus.
- Outside click closes open filter menus.

## Motion (functional)
- Cards fade in when a category or search result set changes; new cards arrive, others do not move.
- While connecting, the connect control changes width to fit its new label rather than snapping.
- The filter menu drops from its trigger.
- With reduced motion preferred, all of these are instant.

## Accessibility
- The category list is a single-select list; the selected category is announced.
- Each card is a labelled group; the connect control's name includes the integration name.
- Connected state is a text badge, not a visual cue alone.
- The result count is a polite live region after a search change.
- Filter menu is a listbox with checkable options.

## Responsive
- Wide: sidebar categories and a three-column grid.
- Medium: categories become a row of chips above the grid; grid two columns.
- Phone width: one column; the search field moves to full width and the filter control sits below it.
- Touch: hover-only disconnect becomes a visible control.

## Composition hints
- Card (integration card), button (connect), badge (connected status), input with search affordance, list (categories), chip group, popover menu for filters, empty-state block.

## Acceptance checks
1. Typing in search narrows cards and the count matches the number of visible cards.
2. Clicking connect shows a pending state on that card only, and the control keeps its width.
3. Calling `on_connect` resolves to a connected badge when `connected` flips to true.
4. Disconnect asks for confirmation within the card before calling `on_disconnect`.
5. The connected count in the header updates when a card's `connected` value changes.
6. Escape in a non-empty search field clears it and keeps focus in the field.
7. At phone width the category list becomes a horizontal chip row and nothing scrolls sideways.
8. With no matches, the empty message and clear-search action appear.

## Amendment 5: structure and demo

Archetype: catalogue page: header with a connected count, full-width search, a category rail beside a three-column card grid.

Visible at rest:
- Header: title "Integrations", one intro line with a "Read more" link, and the connected count at the top right ("1 connected").
- Search field across the width, with the filter control at its right.
- Category rail with 13 entries and counts, in this order: Essentials (8, selected), All apps (22), Analytics (3), Automation (1), Billing (2), Communication (5), CRM (3), Design (2), Development (4), Docs (1), Product (2), Scheduling (3), Storage (2).
- Grid heading "Essentials 8", then three columns and three rows: eight integration cards and one final cell.
- Integration card: logo, name with a verified mark, "By vendor" line, two-line description, connect control. A connected card shows a status chip; its disconnect control appears on hover or focus only.
- Final cell: "Discover more", "+14 apps", and a "View all" button.

Gallery demo, eight Essentials cards (name, vendor, description):
- Chatroom, By Ardent: "Get alerts in your channels and reply without leaving the thread." Connected.
- Docs hub, By Quillo: "Turn records into pages and keep project notes in step."
- Tracker, By Brightline: "Turn customer feedback into work items and follow them to release."
- Design board, By Kestrel: "Preview mockups inline and pin each frame to its ticket."
- Code host, By Halden: "Link changes and show build status beside each task."
- Payments, By Oriel: "Sync customers, invoices and renewals as they change."
- Automation hub, By Veyra: "Connect thousands of apps with no-code triggers."
- Contacts, By Lumen: "Keep people and companies matched across both tools."

Motion, settled frame: all cards shown; none pending.
Moves: cards fade in when the category or search changes. A connect control changes width to fit its new label instead of snapping.
Reduced motion: instant.

Acceptance checks (continued):
9. The grid has three columns. Cards in the third column end inside the grid with a clear margin, and none is cut at the grid's right edge at any supported width.
10. The category rail lists 13 entries at rest, each with its count, beginning with Essentials and All apps.
11. The connected count in the header equals the number of connected cards at rest.
