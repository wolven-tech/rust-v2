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
</content>
