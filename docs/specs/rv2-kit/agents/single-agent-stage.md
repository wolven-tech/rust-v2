# Single Agent Stage

Source item: affectUI Pro `components/agent-workbench` (functional reference only; see README).

## Purpose
Lets a person follow a single assistant on a single request: who made the request, what the assistant is doing right now, which files it was handed, and which tools it may call together with the latest result from each. Makes it visible what the assistant reaches into while it works.

## Data
| Field | Type | Meaning |
|---|---|---|
| `requester` | { display_name: string; message_text: string; photo_url?: string; portrait_slot?: ReactNode } | Person who made the request. If the photo is absent or fails to load, the initials mark is shown instead. |
| `worker` | { worker_key: string; display_name: string; subtitle_text?: string; detail_rows?: DetailRow[] } | The assistant. |
| `worker_stage` | "waiting" \| "working" \| "finished" | Overall condition; selects the state word. |
| `tool_grants` | ToolGrant[] | Tools rendered as cards. Four are shown individually; anything beyond is folded into one summary card. |
| `file_items` | FileItem[] | Files the requester supplied. Four are listed; the rest are counted. |
| `reading_file_key` | string? | File the assistant is reading right now. Its row is highlighted. |
| `work_log` | WorkStep[] | Ordered oldest first. The newest three are visible in the work area. |
| `accessible_name` | string | Name announced for the whole stage. |

ToolGrant: `{ tool_key, mark_source, display_name, grant_text, previous_action?, previous_when?, live_action?, detail_rows? }`. A present `live_action` takes the place of `previous_action` and reads as an in-progress action.

FileItem: `{ file_key, display_name, mark_source?, meta_text?, detail_rows? }`.

WorkStep: `{ step_key, text, route_key, in_progress: boolean }`. `route_key` names a tool key, `"files"` or `"requester"`, and determines which connector the step travels along.

DetailRow: `{ field_label: string; field_value: string }`.

Callback: `on_open_change(target_key | null)`, optional; reports which trigger opened or closed its info card.

## Anatomy
1. Upper band, three panels left to right: the requester card (name and message, message held to four lines), the assistant card in the middle, and the file panel on the right set inside a folder-style frame.
2. Assistant card: a mark that animates while work is underway, its name, its subtitle and the state word.
3. Connectors: rounded joins linking requester to assistant, assistant to files, and assistant to every tool card. Each join ends in a hollow dot at both ends.
4. Lower band: tool cards in a row. Each card carries its mark, its name, its grant text, and either the live action or the previous action with when it happened.
5. Work area: a fixed-height band of three rows above the tool cards. It shows the newest steps, newest on top.
6. Info card: activating the assistant, a tool or a file opens a single card directly below that trigger, listing its `detail_rows`. A tool's card also lists what that tool did for this request.

## States
- Waiting: work area holds a neutral placeholder; state word "Waiting".
- Working: newest step on top, shimmering while `in_progress`; assistant mark animates; state word "Working".
- Finished: nothing animates; state word "Done".
- Tool calling: its card shows `live_action` with a shimmer.
- Tool idle: shows `previous_action` and `previous_when`; with no previous action it shows "Nothing yet".
- File being read: its row is highlighted.
- Overflow: files past four are counted in a line reading "+N more"; tools past four collapse into a summary card reading "+N more tools".
- Incomplete data: a missing photo uses the mark; a step with no text reads "Working…" while in progress and "Done" otherwise.

## Interactions
- Tab: visits the assistant, then each tool, then each file, in reading order.
- Enter or Space on the assistant, a tool or a file: opens its info card and sets `aria-expanded`.
- Enter or Space on the same trigger, a pointer press outside the card, or Esc: closes the card.
- Esc: closes the card and places focus back on the trigger that opened it.
- Only one info card may be open at any time.

## Motion (functional)
- A highlight travels along the connector currently in use, from the requester toward the file or tool being touched, leaving a fading trail.
- Incoming work steps slide in from the upper edge of the work area; the step still underway carries a moving sheen.
- The assistant's mark animates and its state word changes as soon as work begins.
- A tool that is calling shows its live action shimmering; the file being read lights up.
- Info cards grow out of the element that summoned them and retract faster than they expand.
- Users who asked for less motion get static frames: no sliding, no sheen, no travelling highlight.

## Accessibility
- Steps form a list whose newly appended entries are voiced without interrupting the reader.
- Decorative imagery is skipped by assistive technology; connector geometry carries no meaning of its own.
- Conditions are given as words (Waiting, Working, Done, Calling, Used, Ready, Nothing yet), so color is never the only signal.
- Every node in the stage is a toggle; its accessible label is its own name followed by the word details, and an expanded flag reports whether its card is showing.
- Focus indication appears only for keyboard focus.

## Responsive
- Roomy layouts place the three upper panels side by side, with the tool cards in a second band beneath.
- On phone-sized screens the panels collapse into one column, sequenced requester, assistant, files, tools, and the joins are dropped.
- Long names, grant text, file names and steps are cut with an ellipsis; full text is available on hover.
- One or two tools stay at card size and are centred rather than stretched across the row.
- The work area keeps three equal rows so the layout holds still as steps arrive.
- Theme-aware: follows the host's light or dark setting.

## Composition hints
Card, avatar or mark, status badge, connector layer, file list inside a folder-style container, list with live region, info popover, button used as trigger, shimmer text.

## Acceptance checks
1. The work area shows the three most recent steps, newest first; adding a fourth pushes the oldest out of view.
2. Only the step flagged `in_progress` shimmers; when its `route_key` names a tool, that tool's connector carries the highlight.
3. A tool with `live_action` displays that text in place of `previous_action`; a tool with neither displays "Nothing yet".
4. Activating the assistant, a tool or a file opens exactly one info card; activating a second trigger closes the first.
5. Esc closes an open info card and returns focus to the trigger that opened it.
6. Five tools render as four cards plus a summary card reading "+1 more tools".
7. A requester photo that fails to load shows the initials mark.
8. On a phone-sized screen the panels form a single column in the stated sequence, with no joins drawn.
9. With reduced motion requested, a new work step appears with no transition.
10. A newly appended step is voiced by the live region as it lands.

## Amendment: structure and demo

**Archetype: three-band stage.** An upper band holds three panels side by side: the requester on the left, the worker in the middle and the files on the right, inside a folder-style frame. Below it, a work area of three fixed rows. Tool cards form a row at the bottom. Connector lines join the requester to the worker, the worker to the files, and the worker to each tool card.

**Parts that must exist**
- Requester card: name, message cut at four lines, and a photo or initials mark.
- Worker card: mark, name, subtitle and state word.
- Files panel: up to four file rows, then a "+N more" line when there are more.
- Work area: three rows, newest on top.
- Tool cards: up to four, then one summary card for the rest.
- Connector layer and one info card at a time, opened below its trigger.

**Gallery demo counts**
- 1 requester, with a two-line message.
- 1 worker, in the working state.
- 5 files: four rows and a "+1 more" line.
- 6 tools: four cards and a summary card reading "+2 more tools".
- 4 work steps; the work area shows the newest three.

**Visible at rest**
- Worker state word "Working"; the newest step is in progress, and the connector toward the tool it names is marked as carrying traffic.
- Three tool states: one calling with its live action; one used, with its last action and when; one with "Nothing yet".
- One file row highlighted: the file being read.
- No info card open.

**Acceptance checks (continued)**
11. Five files show four rows and a "+1 more" line; six tools show four cards and a summary card reading "+2 more tools".
12. The state word reads Working while any step is in progress and Done once none is; the work area keeps three rows in both states.
13. Exactly one file row is highlighted at rest, the one named by the reading key.
14. The gallery at rest shows a calling tool with its live action, a used tool with its last action and time, and a tool showing "Nothing yet".
15. The requester's message is cut at four lines, and its full text is available on hover.
