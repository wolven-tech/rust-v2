# Step Progress List

Source item: affectUI Pro `components/agent-trail` (functional reference only; see README).

## Purpose
Shows the ordered steps an agent works through on one task: what is finished, what is in progress and what is next, with sub-steps for the step in progress, and an approval gate where the person must decide before the agent proceeds.

## Data
| Field | Type | Meaning |
|---|---|---|
| `list_heading` | string? | Task name shown in a header. Omit for a headerless list. |
| `agent_badge` | { display_name: string; mark_source?: ReactNode } | Who works on the task, shown in the header. |
| `progress_items` | ProgressItem[] | Ordered items. |
| `open_keys` | string[]? | Items whose sub-items are shown. Controlled. Default: the in-progress item open, others closed until activated. |
| `on_open_change` | (keys: string[]) => void | An item was opened or closed. |
| `on_confirm` | (item: ProgressItem) => void | Person confirmed an item that awaits approval. |
| `on_refuse` | (item: ProgressItem) => void | Person refused it. The host sets its state to `dropped`. |
| `wording` | Partial<ProgressWording> | Every word the component shows. Defaults apply per key. |

ProgressItem: `{ item_key, title_text, item_state: "finished" | "in_progress" | "queued" | "awaiting_approval" | "awaiting_person" | "errored" | "dropped", took_ms?, began_at_ms?, live_note?, needs_approval: boolean, tool_badge?: { display_name, mark_source }, approval_request?: string, sub_items?: SubItem[] }`.

SubItem: `{ sub_key, text, sub_state?: "finished" | "in_progress" | "queued", fact_text? }`. `fact_text` is short and sits at the right; long values keep their end visible.

ProgressWording keys: list_heading_fallback, in_progress, approval, awaiting, errored, dropped, finished, queued, empty, untitled_item, earlier_count, later_count, show_fewer, confirm, refuse. Defaults are the component's own English strings.

## Anatomy
1. Header (if `list_heading`): heading, a count such as "n of m steps finished", and the agent badge.
2. Ordered list of items. Each row: state mark, optional tool badge, title, and on the right either the duration (finished) or the live note with a live timer (in progress).
3. Sub-items: nested under their item, joined by a branch line. The in-progress sub-item is the one a light travels to.
4. Approval: an item with `needs_approval` shows a gate marker until reached. Once `awaiting_person`, its row offers Confirm and Refuse beneath the request text.

## States
- `finished`: check mark, duration on the right, title unchanged.
- `in_progress`: spinning mark, `live_note` shimmers, timer counts up from `began_at_ms`. Only one item should be in progress.
- `queued`: muted row, no duration.
- `awaiting_approval`: gate marker with the word "Approval" until reached.
- `awaiting_person`: word "Awaiting you"; the row becomes activatable to show Confirm and Refuse.
- `errored`: failure mark and word "Failed".
- `dropped`: muted, title struck through, with the word "Dropped".
- No items: header stays and wording `empty` takes the list's place.
- Missing title: wording `untitled_item`.
- Sub-item lists longer than six: show the in-progress sub-item and its neighbours with `earlier_count` and `later_count` controls; a control expands the full list and `show_fewer` folds it back.

## Interactions
- Tab: moves to each item that has sub-items, and to the "earlier" or "later" controls.
- Enter or Space on an item with sub-items: opens or closes its sub-items.
- Enter on an item in `awaiting_person`: opens the Confirm and Refuse panel with focus on Confirm.
- Esc in the panel: closes it and returns focus to the item.
- Tab or Enter on an earlier/later control: shows every sub-item; focus stays on the row.

## Motion (functional)
- A short highlight runs from the in-progress item down its branch into the sub-item in progress; it rings once on arrival.
- The in-progress mark turns and its status shimmers; the timer counts up.
- A finished item's check appears with a draw-in; the line to the next item fills.
- Opening or closing an item's sub-items animates the branch line along with the rows it connects.
- Reduced motion: state changes apply at once; the highlight is hidden.

## Accessibility
- Ordered list named after the heading (or the fallback). The in-progress item and sub-item carry `aria-current="step"`.
- The count reads in words, for example "3 of 5 steps finished".
- Each item reads its title, state and time in one name.
- An item that owns sub-items is exposed as a toggle that reports whether it is expanded.
- Closed sub-items are removed from the accessibility tree.
- A polite live region announces when an item finishes, fails, or awaits the person.
- Focus ring visible for keyboard focus only.

## Responsive
- Row heights are fixed for item and sub-item rows, larger on touch screens, so a changing status or timer never moves another row.
- Long sub-item lists fold to five rows around the in-progress one.
- No steps: header stays; wording `empty` shown.
- Long titles end in an ellipsis with full text on hover; the right-hand side never takes more than 40% of a row.
- Right-to-left: the list mirrors to the right edge and the branch line bends the other way.
- Theme-aware: follows the host's light or dark theme.

## Composition hints
Card header with count and badge, ordered list, status mark, button with disclosure state, collapsible branch, inline approval panel, live region.

## Acceptance checks
1. Items render in array order; exactly one item marked `in_progress` carries `aria-current="step"`.
2. The header count updates from "n of m steps finished" when an item moves to `finished`.
3. An item in `awaiting_person` shows the wording `awaiting` and, once activated, Confirm and Refuse.
4. Confirm calls `on_confirm` with that item; Refuse calls `on_refuse` with that item; neither fires on render.
5. Esc closes the approval panel and returns focus to the item that opened it.
6. With more than six sub-items, only five plus the two fold controls are visible until expanded.
7. A missing `title_text` renders the `untitled_item` wording.
8. A `dropped` item's title carries the struck-through style and its accessible name includes the word "Dropped".
9. Row heights stay constant when a live note changes from empty to a long value.
10. At phone width the list has no horizontal page overflow.
