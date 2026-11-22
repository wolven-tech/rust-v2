# Request Routing Map

Source item: affectUI Pro `components/agent-router` (functional reference only; see README).

## Purpose
Shows incoming requests, the router that reads each one and picks who handles it, and every handler's score and reason. A person sees routing happen, can open any request to see why it went where it went, and can reassign it.

## Data
- `router_node`: `{ node_key: string, node_label: string, model_label?: string, reading_text?: string, model_mark?: icon element }`.
- `handler_nodes`: list of `{ handler_key: string (unique), handler_label: string, model_label?: string, handler_state?: string, activity_text?: string, load_count?: number, queued_request_keys?: string[], is_person: boolean, accent_slot: 1 to 6, handler_mark?: icon element, model_mark?: icon element }`. A handler with `is_person: true` has no card; see Anatomy.
- `request_rows`: list of `{ request_key: string (unique), request_title: string, sender_label?: string, channel_kind: "mail" | "chat" | "ticket" | "phone" | "team_chat", icon_element?: icon element, preview_text?: string (first line of the message), request_state: "incoming" | "sorting" | "assigned" | "needs_person" | "closed", assigned_to_key?: string, certainty_fraction: number (0 to 1), score_map: Record<handler_key, number (0 to 1)>, reason_text?: string, tag_list?: { label: string, accent_slot?: 1 to 6 }[] (max 3) }`.
- `layout_mode`: `stacked` (default) | `side_by_side`.
- `max_visible_requests`: number, default 4. Used by `side_by_side`.
- `heading?`: string. Header counts: sorting, waiting on a person, assigned.
- `on_reassign(request, handler_key)`: optional. When given, the open request lists every handler as a control. Without it, the dialog has no reassign controls.
- `on_open(request)`: optional. Called when a request is opened.
- `class_name?`.

## Anatomy
Stacked layout (default): request cards on top, the router card in the middle, handler cards side by side at the bottom. Each handler card has its own queue of request cards below it. A person handler has no card: a line leaves the router and a tag counts the requests waiting for that person.
Side-by-side layout: requests column, router column, handlers column, joined by connectors.
Narrow container: one column, requests indented above the router, handlers below it.
Each request card: icon tile for the channel, title, sender, state line, certainty bar, tags.

## States
- `incoming`: not yet read.
- `sorting`: the router is reading. A marker travels along the connector into the router, the router shows it is reading, and the certainty bar fills as the decision firms up.
- `assigned`: the marker continues to the chosen handler, whose card highlights. In stacked layout the request joins that handler's queue.
- `needs_person`: the marker goes to the person handler; that handler's tag count increases.
- `closed`: card dimmed in place.
- Open request: dialog with certainty, every handler's score, the reason, and reassign controls when `on_reassign` is given.
- Handler focused: its requests stay in full, the others fade back.

## Interactions
- Tab: a single stop for the entire map.
- Arrow keys: walk the cards in order: the top request, the router, each handler, then that handler's queue.
- Home and End jump to the start and the finish of the sequence.
- Enter or Space on a request opens its dialog. On a handler, the same keys isolate that handler's requests and dim everything else.
- Escape: closes the open request, or clears the handler focus.
- Reassign control in the dialog: calls `on_reassign(request, handler_key)`. The marker runs to the new handler.
- Unverified in demo: the dialog's content and reassign. Headless arrow keys, Enter and Escape ran without page errors; the dialog was not inspected.

## Motion (functional)
- A light travels the request's link into the router and then along the next link to the chosen handler, brightest mid-path and dim at each end.
- The receiving card lights up in its handler's accent, rising from the side the marker came from.
- While reading, the router's mark animates and its line shimmers. The certainty bar fills as it decides.
- On first render, cards enter with a short upward settle and their links draw in afterwards.
- Reduced motion: changes appear at once and the marker stays off.

## Accessibility
- Each card reads as one sentence, for example "<title>, from <sender>: <state>, to <handler>, <percent> sure".
- The open request is a labelled dialog. Reassign controls name the handler they send to.
- A polite live region announces where each request went.
- Connectors and markers are decorative; everything they show is in the card labels.
- Focus rings show for keyboard focus only.

## Responsive
- Wide, stacked: as in Anatomy; handler queues fade out at the bottom and share one height.
- Wide, side by side: three columns joined by curved connectors.
- Narrow container: one column, as in Anatomy.
- Long titles, senders and preview lines end in an ellipsis, with the full text on hover.

## Composition hints
Card, list or grid, buttons, dialog, badges (tags), certainty bar (progress), icon element, custom connector paths, live region.

## Acceptance checks
1. A request moves incoming, sorting, assigned in that order, and each step is visible.
2. An assigned request appears in its handler's queue in stacked layout.
3. A person handler gets no card; its tag count rises as requests reach it.
4. Arrow keys visit the top request, the router, each handler and its queue in order; Home and End jump to the ends.
5. Enter on a request opens the dialog with certainty and every score; Escape closes it.
6. With `on_reassign` given, the dialog lists every handler; choosing one calls the handler with the request and the key and re-runs the marker to the new handler.
7. The live region announces each assignment.
8. At the narrowest phone width, the narrow layout shows one column with no horizontal page scroll.

## Amendment: structure and demo

**Archetype: stacked flow.** Request cards form a top row. A router card sits centred beneath them. Handler cards form a row below the router, each with its own queue of request cards beneath it. A person is not a card: a line leaves the router and ends in a tag that counts the requests waiting for that person.

**Parts that must exist**
- Header: heading and counts of requests sorting, waiting on a person, and assigned.
- Request card: channel tile, title, sender, state line, certainty bar with its percentage, and up to three tags.
- Router card: name, model line, a reading line, and a certainty bar.
- Handler card: name, model line, state, activity line, load count, and its queue.
- Person entry: the link line and its count tag. No card.
- Request dialog: certainty, every handler's score, the reason, and reassign controls when supplied.

**Gallery demo counts**
- 5 requests: 1 incoming, 1 sorting, 2 assigned, 1 needing a person.
- 3 handlers: Billing (1 queued request), Ops (1 queued request), Support (none).
- 1 person, Sam, whose tag reads 1.

**Visible at rest**
- All four request states: incoming with an empty certainty bar; sorting with a partly filled bar and the router's reading line; assigned, in the Billing and Ops queues; needing a person, counted on Sam's tag.
- Support's queue showing its empty text.
- No request dialog open.

**Acceptance checks (continued)**
9. Five request cards render, one each in the incoming and sorting states, two in the assigned state and one needing a person; the assigned two sit in the Billing and Ops queues.
10. Support's queue shows its empty text; a handler with a queued request lists it beneath its card.
11. The person entry has no card; its tag reads 1 and counts only requests in the needs-person state.
12. The sorting request's certainty bar is partly filled; every other request's bar shows its stored value.
13. The header counts equal the number of request cards in each state, and the open request dialog lists every handler's score.

## Amendment 2: structure and demo

This amendment replaces the first amendment's stacked archetype, its five-card gallery, and checks 9, 12 and 13. Checks 14 to 19 replace them.

**Archetype: a tree from the top, one handler column per team.** One incoming request card sits at the top, with the needs-person card beside it. The router card is centred beneath the top row. Handler cards form a row beneath the router, each with its task queue stacked under it. Router-to-handler connectors form a bus. A person is not a card: a line leaves the router's side and ends in a tag counting the requests waiting for that person.

**Parts that must exist**
- Header: heading, the count of routed (assigned) requests, and the count of requests waiting on a person. The sorting count appears only while a request is sorting.
- Top row: request cards in the incoming or needs-person state. A card in the incoming state shows no certainty bar; the bar appears once sorting starts.
- Router card: name, model line, state badge, reading line, certainty bar.
- Handler card: name, model line, state, activity line, load bar with its count, and a footer naming the last request routed.
- Task queue beneath each handler card: a title, a count badge, its request cards, and the empty text when nothing is queued.
- Person entry: a line from the router's side ending in a count tag. No card.
- Request dialog, as in the first amendment.

**Gallery demo counts**
- Instance one: two request cards on top, one incoming and one needing a person. Three handlers: Billing with one assigned request in its queue, Ops with one, and Support with none. One person, Sam, whose tag reads one.
- Instance two: the incoming card becomes the sorting request, with a partly filled bar and the router's reading line. Everything else as in instance one.

**Visible at rest**
- Top row of two cards; the incoming card with no bar; the needs-person card.
- Billing and Ops queues each holding one card; Support's queue showing its empty text.
- Sam's tag reading one; no person card.
- Header: two routed and one waiting on a person.

**Acceptance checks (continued)**
14. At rest the top row holds exactly two request cards: one incoming with no certainty bar, and one needing a person.
15. The router card is centred beneath the top row. Each handler card sits beneath the router, with its queue directly under it.
16. Billing and Ops each list one assigned request in their queue. Support's queue shows its empty text.
17. The person entry is a line from the router's side ending in a tag that reads one. It has no card.
18. In instance two the sorting card's bar is partly filled and the router shows its reading line; no other card's bar changes.
19. Header counts equal the cards in each state at rest; the sorting count is absent when nothing is sorting.
