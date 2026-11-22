# Connector Picker

Source item: affectUI Pro `components/integration-picker` (functional reference only; see README).

## Purpose
The services step of a command menu. A person searches a list of services, sees which are connected and which are available, picks one, and connects or disconnects it. The same component attaches tools to an agent when the wording is changed.

## Data
- `connector_list` (Connector[]): each entry is `{ connector_id, connector_name, connector_domain, connector_category, connector_summary, connector_features (text[]), connector_mark (node), is_connected (boolean) }`. Defaults to a sample set of ten services.
- `product_name` (text): shown in the title, for example "<product> + <service>".
- `product_mark` (node): the product's mark on the leading tile.
- `initial_selection_id` (text, default: first item).
- `on_connection_change(connector_id, is_connected)`.
- `on_explore(connector_id)`: called by the explore control.
- `on_back()`: shows a back control and runs on Left arrow when the search is empty.
- `on_close()`: runs on Escape when the search is empty.
- `label_text` (partial): title, search placeholder, explore, connect, disconnect. Lets the component serve other lists (for example "Tools").
- `action_mode` (`"explore" | "connect"`, default `"explore"`):
  - explore: a toggle near the title connects; the action button calls `on_explore`.
  - connect: no switch; the button connects the selected item (primary style) or disconnects it.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Header: back control (when given), title "<product> + <picked service>" or the labelled title, and a search field with a shortcut hint.
2. Two groups in the list: Connected, then Available. Each row shows a mark, name and category.
3. Detail pane (beside the list on wide screens, below it on narrow ones): the picked service's mark, name, domain, connection status line, summary, feature bullets, and the action.
4. Footer: a count reading "N of M connected" and a key-hint row (arrows to move, Enter to switch, Escape to close).

## States
- Browsing: a row is picked (highlighted); the detail shows it.
- Searching: list filtered; groups without matches hide; an empty result shows a "no matches" line. (Spec choice.)
- Connected row: a mark on the row; switch on (explore mode).
- Available row: switch off.
- Connected detail: connected status; the connect toggle reads disconnect.
- Empty search with back: back control active.

Transitions: a switch changes the connection, updates the count, moves the row between groups, and calls `on_connection_change`.

## Interactions
Pointer:
- Click a row: picks it and shows its details.
- Click the switch (explore mode): toggles the connection.
- Click the action (explore mode): calls `on_explore`.
- Click the action (connect mode): connects or disconnects the picked item.
- Type in the search: filters the list.

Keyboard:
- Up / Down: move through the list, across both groups without a pause.
- Enter: switches the picked connector on or off.
- Escape: clears the search; if the search is already empty, closes.
- Left arrow: back, when the search is empty and `on_back` is given.
- Cmd/Ctrl + / (or "/"): jumps into the search from anywhere.

## Motion (functional)
- The highlight steps from row to row, including across groups, with no long glide.
- The service mark on the detail swaps in with a brief fade and lift.
- A connected service shows a soft ring that pulses outward; it pauses when scrolled off screen.
- Reduced motion: all changes instant.

## Accessibility
- The search is a combobox controlling the list; the picked row is the active option.
- The connection control is a real switch whose label says what it will do.
- Detail updates are announced politely when the picked row changes.

## Responsive
- Sizes to its container. Narrow: the list stacks above the detail pane and the key-hint row disappears. Very narrow: feature bullets disappear too.

## Composition hints
Command-menu panel, search input (combobox), grouped list with options, row with mark and name, detail pane, switch, primary button, footer with counts and key hints, pulse ring.

## Acceptance checks
1. With the default sample set, the header reads "3 of 10 connected" (or the count for the supplied data).
2. Typing a search term hides rows that do not match and keeps the picked row if it still matches.
3. Enter on a picked available connector switches it on, moves it to the Connected group and calls `on_connection_change(id, true)`.
4. In connect mode the action label reads "connect" for an available item and "disconnect" for a connected one.
5. Left arrow with an empty search calls `on_back`; with text in the search it does not.
6. Escape with text in the search clears it; Escape with an empty search calls `on_close`.
7. Cmd/Ctrl + / from anywhere in the list focuses the search.
8. In a narrow container, the list stacks above the details and the key-hint row is omitted.
9. Reduced motion: the pulse ring is not shown.
10. Changing `product_name` changes the header title text.

## Amendment: structure and demo

Layout archetype: a two-pane panel. A header row runs across the top; beneath it a list pane on the leading side and a detail pane on the trailing side; a footer row runs across the bottom. The panel is shown with its detail pane populated at rest.

Parts that must exist:
- Header row: back control (only when `on_back` is given), the title on the leading side, and the search field on the same row at desktop width, filling the space after the title.
- List pane: two groups, "Connected" then "Available", each with a small heading. Each row: a mark tile at the leading edge, the name, and the category at the trailing edge.
- Detail pane: the picked service's mark tile and name, its domain beneath the name, a connection status line, a summary paragraph, a bullet list of features, and at the bottom the action control.
- Explore switch: in explore mode, a switch sits beside the title in the detail pane, labelled with what it does.
- Footer row: the count text "N of M connected" at the leading edge; a key-hint row at the trailing edge naming the move, switch and close keys.

Demo data for the gallery (our own names and values):
- Ten services: four connected, six available. Categories are our own words (for example storage, planning, messaging, data).
- Product name in the title from `product_name`.
- Picked row: the first connected service. Its detail shows three feature bullets.
- Explore mode (the default), so the switch beside the title is on for the picked service and the action control reads our explore wording.
- Search field empty, showing its placeholder.

States visible at rest in the gallery: both groups with their counts of rows, the picked row highlighted, the connected detail state, the explore switch on, the footer count and key hints. No search text, no no-match line.

Acceptance checks:
11. At desktop width the gallery demo shows the header row with the title and the search field on one row, the list and detail panes side by side, and the footer row beneath.
12. The list shows the Connected group with four rows and the Available group with six rows, in that order.
13. Each row shows the mark tile at its leading edge, the name, and the category at its trailing edge.
14. The picked row is the first connected service; the detail pane shows its mark, name, domain, connection status line, summary and three feature bullets.
15. In the gallery demo the explore switch beside the title is on and the action control is the explore control.
16. The footer count reads "4 of 10 connected" in the gallery demo, and the key hints sit at the trailing edge.
17. At rest the search field shows its placeholder and the no-match line is absent.
</content>
