# Connected Apps Catalogue Screen

Source item: affectUI Pro `screens/integrations` (functional reference only; see README).

## Purpose
A directory in which a person browses the external tools the assistant can connect to, finds one by name or category, and sees which ones are already connected. Requests for a tool that is not listed go through a form.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_name | string | Sidebar label. |
| account | { name; avatar_url? } | Account block. |
| apps | { id: string; name: string; summary: string; category: string; connected: boolean; popular?: boolean; logo_url?: string; bundle?: string[] }[] | The directory. `bundle` lists the names of several services grouped under one entry. |
| categories | string[] | Section headings, in order. |
| filter | "all" \| "connected" \| "available" \| "popular" | Top tab. |
| query | string | Search text. |
| is_loading | boolean | Two sections of secondary rows shaped like the list rows. |
| is_empty | boolean | No apps: title, tabs, search, and the request control stay; the list shows an empty state with a request button. |
| on_connect | (id) => void | Connect action on an available app. |
| on_disconnect | (id) => void | Disconnect action on a connected app. |
| on_request | (name: string, note: string) => void | Request form submitted. |

## Anatomy
Reading order:
1. Sidebar, left.
2. Sticky header: the page title, the request control on the right, and beneath them the top tabs (All, Connected, Available, Request, Popular) with a search field.
3. Category sections in the given order; each has a heading and a list of app rows.
4. Each app row: a logo or initial tile, the name, a one-line summary, and a connection control (Connect, or Connected with a menu).
5. Bundle entries show the grouped service names after their summary.

## States
- **Loaded.** The default.
- **Loading.** Two sections of secondary rows; the header and tabs stay.
- **Empty.** The list shows "No integrations yet", one line of explanation, then a button labelled Request integration.
- **Connected row.** Shows a connected mark and a menu containing Disconnect.
- **Available row.** Shows Connect.
- **Connecting.** The Connect control shows progress, then Connected.
- **Request form open.** A small form with name and note fields; Submit is enabled once the name is filled.
- **No match.** Lists show "No apps match" with a clear-search action.

## Interactions
- Top tabs: arrow keys move; Enter selects; choosing Request opens the request form.
- Search: typing filters rows across all categories; Escape clears it.
- Connect: click or Enter on an available row's control connects it.
- Connected menu: Enter opens it; Disconnect removes the connection after a confirmation.
- Row: Enter on the name opens the app's detail view (provided by the host).
- Request form: Enter in the name field moves to the note; Enter in the note submits; Escape closes it.
- The sticky header keeps the request control and tabs reachable while scrolling.

## Motion (functional)
- A row that changes state moves into its place under the selected tab without a full reload.
- Connecting shows progress in place and resolves on the same row.

## Accessibility
- Top tabs form a tab list; counts appear in their labels.
- Each row is a labelled group with its name, summary, and connection state.
- The connection control names its app: "Connect Slack", or "Connected: Slack, open menu".
- Request form fields are labelled; errors are announced.
- Logos are decorative when the name is present.

## Responsive
- At phone width, rows keep the logo, name, and control; summaries wrap under the name; the tab strip scrolls inside itself.
- The sticky header stays at the top of the page area.
- The page never scrolls sideways.

## Composition hints
- Workspace frame.
- Tab strip with counts; search field.
- Section headings with list rows; app row with logo tile and status badge.
- Button for Connect; menu button for the connected state.
- Dialog or inline form for requests.
- Empty state and skeleton rows.

## Acceptance checks
1. With `is_loading` set, two sections of secondary rows show, and the tabs stay clickable.
2. With `is_empty` set, the list shows its empty state, and the request button opens the request form.
3. Choosing "Connected" shows only rows whose `connected` is true.
4. Clicking Connect on an available row calls `on_connect` with its id, and the row changes to connected.
5. Typing the start of a name in search narrows the list to matching rows across all categories.
6. Submitting the request form with a name calls `on_request` with the name and the note.
7. Submit stays disabled while the name is empty.
8. Escape in the request form closes it without calling `on_request`.
9. The header with its tabs stays visible while the list scrolls.
10. At phone width the page has no horizontal scroll, and the tab strip scrolls inside itself.
