# Pending Decision Stack

Source item: affectUI Pro `components/approval-gates` (functional reference only; see README).

## Purpose
Lists every point where an agent run paused for a person. The open request on top can be read in full and decided (approve, edit, reject). Decided requests move into a history list. Open requests that block later steps show what they hold.

## Data
| Field | Type | Meaning |
|---|---|---|
| `decision_items` | Item[] | Every item in the run, in arrival order. Open items form the stack; the rest form the history |
| `run_subject` | string | What the run is for; shown muted beside the heading |
| `is_running` | boolean | Run still going. With nothing open, reads as "Running"; new items slide in behind the stack |
| `on_resolve(id, outcome, revised_value?)` | callback | `outcome` is `approved`, `edited` or `rejected`. `revised_value` is the edited input (for file changes, the revised change text) |
| `section_title` | string | Default "Pending decisions" |
| `root_class` | string | Extra class on root |

Item: `{ id, requester, requester_mark?, request_kind, request_summary, request_reason, request_status, requested_at?, request_payload?, change_lines?, touched_paths?, prior_steps?, waiting_steps?, person_note? }`
- `request_kind`: `tool` | `file` | `agent` | `message`
- `request_status`: `open` | `approved` | `edited` | `rejected`
- `request_payload`: arguments, prompt or message body
- `change_lines`: per-file lists of added and removed lines
- `touched_paths`: files the request touches
- `prior_steps`: short list of completed steps before this item
- `waiting_steps`: names of steps that wait for this item
- `person_note`: what the person changed or said

## Anatomy
Reading order:
1. Heading row: section title, run subject, counters (waiting on you, decided, step progress), run state word.
2. Blocking banner (only when open items hold steps): "Paused – N steps wait on M items".
3. Stack of open items. Top card is fully expanded: requester and time, summary, reason, payload or change lines, prior steps (collapsed by default), waiting steps list, and the three decision controls plus "Revise first".
4. Cards behind the top one peek out at the edges, scaled slightly smaller.
5. History list of decided items, each a compact row that expands on activation.

## States
- **No open items, not running**: heading shows "All decided"; stack area hidden.
- **No open items, running**: "Running" state word; stack area reserves its height and shows a quiet placeholder.
- **Open items**: stack shown; the top item is the one to act on.
- **Editing top item**: payload or change lines become editable text; decision controls change to Save-and-approve, Back.
- **Decided item in history**: approved, edited or rejected label with requester and time.
- **Rejected item**: stays in history as the point where the run stopped.
- **Running, new item arrives**: slides in behind the stack; counter increments.

## Interactions
- Pointer: decision buttons on the top card; clicking a peeking card brings it to the top.
- Left and right arrow keys: move between open items; focus lands on that item's primary decision control.
- Tab: moves through the top card's controls and the history list.
- Mod+Enter while editing: approve with edits.
- Escape while editing: discard edits and return focus to the Revise first control.
- Enter or Space on a history row: expand or collapse it.
- Decide action on top card: the card leaves the stack toward the history list; the next item rises to the top and focus moves to its primary control.

## Motion (functional)
- A decided item drops out of the stack toward the history; the next one rises into place.
- A rejected item gives a single short shake before leaving.
- New arrivals enter behind the stack without moving the top card.
- Stack reserves the height of its tallest card so the history below does not move during decisions.
- Reduced motion: all changes immediate.

## Accessibility
- The stack is a group named with the open count and which item is on top.
- After each decision, a polite announcement states what was decided and how many remain.
- Diff lines are announced as added or removed in words, not colour alone.
- Status is always conveyed in text (Blocking, Approved, Rejected, Stopped the run).
- Focus moves to the next top card's primary control after a decision.

## Responsive
- Shares the width of its container, capped at a readable width.
- Narrow: counters wrap onto two rows; history rows drop the requester name.
- No horizontal scroll at phone width.

## Composition hints
Card (item cards), badge (kind, status, blocking), list (history), button group, diff view (lines with added/removed labels), definition or meta line (requester, time), live region helper, disclosure (prior steps, expanded payload).

## Acceptance checks
1. With two open items, only the top item's full content is visible; the second is present but not interactive.
2. Approving the top item calls `on_resolve(id, "approved")` once and moves that item into the history list in the same render.
3. Choosing Revise first, editing the payload and pressing Mod+Enter calls `on_resolve(id, "edited", revised_value)` with the edited text.
4. Escape while editing returns the item to its unedited state and does not call `on_resolve`.
5. Rejecting an item leaves it in history with status rejected and keeps the run subject as "Paused" or "Stopped" wording, never deletes it.
6. An item with `waiting_steps` set shows the Paused banner with the correct step and item counts.
7. Setting `is_running` true with no open items shows "Running", and an item appended to `decision_items` appears in the stack without removing the rest.
8. Arrow keys move focus between open items and land on that item's primary control.
9. Decided items are never removed from `decision_items` by the component (verified by a test that counts entries before and after decisions).
10. At phone width the heading counters wrap and no horizontal scroll occurs.
