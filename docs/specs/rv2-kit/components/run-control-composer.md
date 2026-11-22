# Run Control Composer

Source item: affectUI Pro `components/run-controls` (functional reference only; see README).

## Purpose
A single input for working alongside an agent that is running or about to run. It streams the agent's answer as it arrives and exposes each control a person may need mid-run: halt, resume after a halt or failure, retry, dismiss, steer while it writes, queue the next message, switch between a read-only chat and an acting mode, choose a permission level, and see how much of the context window is used.

## Data
| Field | Type | Meaning |
|---|---|---|
| `agent_identity` | `{ agent_id, agent_name }` | The agent. Its id picks the mark shown |
| `request_text` | string, optional | The question that was sent. When absent, the run area shows a quiet placeholder |
| `answer_text` | string, optional | Answer received so far; extends as it streams |
| `run_state` | `"idle" \| "running" \| "stopped" \| "error" \| "done"` | Current state of the run. Starts at idle |
| `steer_notes` | `{ at_char, note_text }[]` | Notes sent mid-run, shown inside the answer at the character where each one arrived |
| `queued_messages` | `{ queue_id, message_text }[]` | Messages held until the run ends |
| `work_mode` / `initial_work_mode` | `"chat" \| "act"` | Chat restricts output to words; act lets the agent change things. Act is the default |
| `access_level` / `initial_access_level` | `"ask" \| "edits" \| "full"` | How much the agent may do unprompted. Ask is the default. Full receives a warning treatment |
| `context_share` | number 0..1, optional | Portion of the context window occupied. Drawn as a ring that warns past a high threshold |
| `elapsed_secs` | number, optional | How long the run took; displayed once it is done |
| `on_submit` / `on_steer` / `on_queue` | (text) => void | Submit while idle; steer while running; queue while running through the alternate key |
| `on_dequeue` | (queue_id) => void | Discards a queued message |
| `on_stop` / `on_resume` / `on_retry` / `on_dismiss` | () => void | Run controls |
| `input_hint` | string, optional | Placeholder text for the empty field |

## Anatomy
Reading order:
1. Run area: the request, then the streamed answer with steer notes embedded, a caret while running, and a result line on completion (elapsed time, or the stopped or error line).
2. Queue tray: waiting messages, each with a remove control. Shows four at most and scrolls beyond that.
3. Input field: its label changes while the agent is busy; hint text explains the keys.
4. Control row: work mode (Chat and Act as a radio group), the access menu, the context ring, and the primary action.
5. The primary action follows the state: Submit when idle; Steer and Queue while running; Stop while running; Resume, Retry and Dismiss after a halt or error; Retry after completion.

## States
- **Idle:** Submit is primary; the run area shows the placeholder or the previous run.
- **Running:** the answer streams; the caret and agent mark animate; Stop is visible; Steer and Queue are reachable from the input.
- **Stopped:** the answer ends where it stopped; the result line records the point; Resume, Retry and Dismiss are offered.
- **Error:** same as stopped, with error wording.
- **Done:** a result line with the elapsed time; Retry offered.
- **Queue has entries:** the tray lists them and the header shows the count.
- **Full access:** the access control takes the warning treatment.
- **Context high:** the context ring takes the warning treatment past the configured threshold.
- **Chat mode:** the access control is hidden or disabled, because it only governs acting.

Transitions: submitting moves idle to running; Stop moves running to stopped; completion moves running to done; a queued message is sent once the run ends.

## Interactions
- **Pointer:** choose a mode; open the access menu and pick a level; press the primary action; remove a queued message.
- **Keyboard, Enter:** submits when idle; steers the agent when running.
- **Keyboard, Alt or Option with Enter:** queues the message while the agent is running.
- **Keyboard, Shift with Enter:** inserts a line break.
- **Keyboard, Escape:** closes the access menu when open; otherwise halts the run.
- **Keyboard, Left and Right on the mode control:** flip between Chat and Act.
- **Keyboard, Up, Down, Home, End inside the access menu:** move through the levels.
- **Touch:** the Escape hint is hidden; Stop remains a button.
- **Right-to-left:** every element, from bubbles to menus to indents, is mirrored.

## Motion (functional)
- A steer note appears inside the answer at the point where it landed.
- The caret pulses while the agent writes; the mark animates.
- Queued messages and a new turn rise into place.
- Primary action buttons swap in place with a short transition.
- With reduced motion requested, none of this plays.

## Accessibility
- Status changes (running, stopped, done, queue count) are announced politely. Streamed words are not announced, which keeps screen readers from being flooded.
- The input field describes its keys through a description and changes its label while the agent is busy.
- Mode is a radio group. The access control is a radio-item menu that marks the current level; its name keeps that level even when only the icon shows.
- Word counts use locale-aware segmentation, so they stay correct in scripts without spaces.
- The conversation scrolls on its own and can be focused for keyboard scrolling.

## Responsive
- The run area keeps a fixed height and scrolls internally, so the card does not grow while the answer streams.
- The tray displays four entries and scrolls past that.
- Lengthy agent names are clipped, with the complete name revealed on hover; long links and words break across lines.
- On narrow widths the access control shows only its icon and the context control only its ring.

## Composition hints
Card for the container, chat message bubble for request and answer, streaming text container, segmented control for mode, menu with radio items for access, progress ring for context, button for the primary action, polite live region.

## Acceptance checks
1. With run state idle, Enter in the input calls `on_submit` with the typed text and clears the field.
2. With run state running, Enter calls `on_steer`; Alt or Option with Enter calls `on_queue`.
3. Escape while running calls `on_stop` when the access menu is closed.
4. With run state stopped, Resume, Retry and Dismiss are offered, and each calls its own handler.
5. A steer note appears in the answer at the character position recorded in `at_char`.
6. A queued message can be removed with its control, which calls `on_dequeue` with its id.
7. In chat mode the access control cannot be operated.
8. The access menu lists the three levels with the current one checked; arrow keys move between them.
9. A context share above the configured threshold shows the warning treatment.
10. At narrow widths the access control shows only its icon, and its accessible name still states the level.

## Amendment: structure and demo

**Archetype.** A single chat composer card in one column. Above the input sits the run area (request, then the streamed answer); between them and the input sits the queue tray; the control row closes the card. It is not a sidebar and not a split pane.

**Parts that must exist, top to bottom:**
- Card header: the agent's name on the left, a state word on the right.
- Run area: the request as a bubble, then the answer with a caret while running.
- Queue tray: one row per queued message with its remove control; a header count.
- Input: its label changes while the agent is busy; a hint line beneath names the keys for send, steer, queue and stop.
- Control row: Chat and Act as a radio group, a permission menu button showing the level name, a context meter showing its numeric share, and the primary action.

**Gallery demo data:** agent named "Itinerary agent"; a request about a three-day trip; one queued message; context share in the middle of the range; permission at its default level; mode Act.

**States visible at rest in the gallery:** running; one queued message marked as next; primary action Stop; context share shown as text.

**Acceptance checks, continued:**
11. Gallery at rest: the header shows the agent's name and the word for running; the run area shows the request and the streaming answer with a caret; the primary action is Stop.
12. Gallery at rest: the queue tray shows one entry, marked as next, with a remove control; the header count reads one.
13. The hint line names the send, steer, queue and stop keys in words and is visible at rest on wide widths.
14. Gallery at rest: the context meter shows its numeric share as text beside it.
15. Pressing Stop in the gallery moves the run to stopped and replaces Stop with Resume, Retry and Dismiss; the answer ends where it stopped; the queued message stays in the tray.
