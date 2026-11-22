# Mid-Run Note Panel

Source item: affectUI Pro `components/agent-steering` (functional reference only; see README).

## Purpose
Lets a person add a note to an agent that is already working: either queue it for the agent's next step, or apply it to the active step at once. The panel shows which step is active and how each note landed.

## Data
| Field | Type | Meaning |
|---|---|---|
| `run_heading` | string | What the agent is working on. |
| `run_agent_name` | string? | Agent or model name shown under the heading. |
| `run_started_ms` | number | Start of the run; the header clock counts from here. |
| `run_state` | "running" \| "paused" \| "finished" | The run. Clocks tick only while `running`; when `finished` the composer is removed. |
| `plan_steps` | PlanStep[] | Ordered steps. |
| `queued_notes` | Note[] | Notes waiting for the next step. |
| `draft_text` | string | Composer text. Controlled. |
| `draft_hint` | string? | Placeholder text shown in an empty composer. |
| `on_queue_note` | (text: string) => void | Composer submitted to the queue. |
| `on_apply_note_now` | (text: string, note_key?: string) => void | Composer submitted to apply now, or a queued note's "apply now" action. |
| `on_drop_note` | (note_key: string) => void | A queued note was removed. |
| `on_run_state_request` | (next: "running" \| "paused") => void | Pause or resume pressed. |
| `on_draft_change` | (text: string) => void | Composer text changed. |

PlanStep: `{ step_key, label, phase: "upcoming" | "active" | "done", began_at_ms?, took_ms?, step_notes: Note[] }`.
Note: `{ note_key, text, placed?: "next" | "now" }`. A note the agent has taken in is moved from `queued_notes` into a step's `step_notes` with the same `note_key` and `placed` set.

## Anatomy
1. Header: heading, agent name, step counter ("step n of m"), run clock.
2. Plan: ordered list of steps. Each row shows a phase mark, its label, and on the right its duration (done) or live elapsed time (active). Notes a step has taken in sit beneath it.
3. Queue: lists `queued_notes` with a count. Each queued note has an "apply now" action and a remove action. The queue opens and closes without moving the rows around it.
4. Composer: multi-line text field with a visible label, a hint line showing the two submit shortcuts, and a send control.
5. Run control: a pause or resume button in the header area.

## States
- Run `running`: clocks tick, active step shows its spinning mark, composer enabled.
- Run `paused`: clocks frozen; control reads Resume; composer still accepts notes.
- Run `finished`: clocks stop, composer and queue removed, final durations shown.
- Step `upcoming`, `active`, `done`: mark and motion differ; active step is the only one with a live clock.
- Queue empty: queue region hidden.
- Composer empty: send control disabled; hint shown.
- Note placed `next` vs `now`: shown on the note's card in words.

## Interactions
- Enter in the composer: queues the note (`on_queue_note`), clears the field.
- Ctrl+Enter, or Cmd+Enter on Apple platforms: applies the note to the active step at once (`on_apply_note_now`).
- Shift+Enter: inserts a line break.
- Esc in the composer: clears the field.
- Tab order: pause or resume control, then each queued note's "apply now" and remove actions, then the send control.
- Remove action on a queued note: calls `on_drop_note`.

## Motion (functional)
- A note the agent takes in travels from the queue into its target step, then brightens briefly once it lands. The queue's entry is removed as the note arrives.
- The composer's card height follows its content, so opening or closing the queue never causes a visible jump.
- The active step's mark spins and its label shimmers; a finished step's check appears with a small pop.
- Reduced-motion preference: all changes apply instantly and the shimmer stops.

## Accessibility
- Plan is an ordered list named "Plan". The active step carries `aria-current="step"`.
- Queuing, applying now, and each note the agent takes in are announced in a polite live region.
- Composer has a visible label; the shortcut hint is linked to it as its description.
- Icon-only buttons have names: pause, resume, remove note (with the note text), queue note, apply now.
- Run state and placement are stated in words, not colour alone.

## Responsive
- Fills its container up to a maximum width.
- Below a narrow width, padding tightens.
- Touch screens get larger hit areas on every button.
- Theme-aware: follows the host's light or dark theme.

## Composition hints
Card, header with clock, ordered list with status marks, collapsible region, multi-line text field, icon button, live region.

## Acceptance checks
1. Pressing Enter in the composer calls `on_queue_note` once with the trimmed text and clears the field.
2. Ctrl+Enter (Cmd+Enter on Apple platforms) calls `on_apply_note_now` and does not add the note to the queue.
3. Shift+Enter inserts a line break and does not submit.
4. Esc clears the composer text and calls `on_draft_change` with an empty string.
5. Setting `run_state` to `paused` stops the active step's clock; setting it back to `running` resumes it from the same value.
6. Moving a note from `queued_notes` into a step's `step_notes` with the same key shows it inside that step and removes it from the queue.
7. Activating remove on a queued note calls `on_drop_note` with its key only.
8. The active step has `aria-current="step"`; no other step does.
9. A new note the agent takes in is announced through the polite live region.
10. At phone width the panel fits without horizontal page overflow.
