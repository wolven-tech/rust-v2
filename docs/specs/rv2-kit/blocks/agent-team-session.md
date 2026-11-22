# Agent Team Session

Source item: affectUI Pro `blocks/live-session` (functional reference only; see README).

## Purpose
Lets a person follow a team of agents while they work: read the shared conversation, see the team's plan advancing, and see the shared context being consumed. Picking one agent narrows the view to that agent's messages and steps.

## Data
- `agents`: list of { id, name, model_label, role }.
- `messages`: list of { id, author_id, to_ids: string[] | "everyone", reply_to_id?, at, text, tool_calls: { name, count }[] }.
- `plan_steps`: list of { id, title, status: "pending" | "running" | "done" | "blocked", started_at?, finished_at? }.
- `context_usage`: { used_tokens, limit_tokens, segments: { label, tokens }[] }.
- `spotlight_agent_id` (controlled, nullable), `on_spotlight_change(id | null)`.
- `on_message_submit({ text, to_ids, reply_to_id })`, `on_toggle_pause(is_paused)`.
- `stream_height_cap`: height used when the layout stacks on narrow containers.
- `extra_classes`.

## Anatomy
In reading order:
1. Header: task title, live indicator, elapsed time, and the team's name line.
2. Agent tabs row: "Everyone" first, then one tab per agent with its name.
3. Message stream (left, widest): messages in time order; each shows author, recipients, time, text and a tool-call summary line.
4. Plan panel (right, top): plan title, step count, running time, pause control; steps listed in order with a status mark.
5. Context panel (right, bottom): usage figure, a cell grid showing used and free space, and a legend of segments with their token counts.
6. Composer under the stream: text area, mention trigger, send control.

## States
- Live: indicator shows live; new messages and step changes arrive as they happen.
- Paused: indicator reads paused; plan shows the pause control as resume; no new steps advance.
- Everyone selected: all messages and all steps shown.
- Agent focused: messages from that agent and messages addressed to it are shown; other messages fade; the plan keeps that agent's steps and dims the rest.
- Step blocked: step mark shows blocked with a short reason line.
- Context near limit: usage figure takes a warning word, not a visual cue alone.
- Empty stream: a one-line prompt to send the first message.

## Interactions
- Click an agent tab: focuses that agent; click it again or click Everyone to clear.
- Click a message: opens its details under the message (recipients, time, tool calls).
- Click a plan step: focuses the agent that owns it.
- Escape: clears the focused agent and returns the plan to all steps.
- Composer: Enter sends; Shift + Enter inserts a line break; typing `@` opens an agent list; arrow keys move, Enter picks.
- Pause control: toggles pause and resume.
- Keyboard: Tab order is agent tabs, messages, composer, plan controls, context. Left and Right move between agent tabs; Home and End jump to the ends. Up and Down move between messages; Enter opens a message's details.

## Motion (functional)
- Selecting an agent dims unrelated messages and plan steps together, so both panels change in a single step.
- Steps tick off as they finish; context cells fill as the conversation grows.
- New messages appear at the bottom of the stream and the stream keeps the newest message in view unless the person has scrolled up.
- With reduced motion preferred, every change applies in one step.

## Accessibility
- The whole session is a landmark named for the task heading.
- The agent tabs are a tablist; the stream is a log region with new messages announced politely.
- Plan steps form a numbered list; the in-progress step is flagged with aria-current, and its elapsed time is spoken when it updates.
- Each message includes author, recipients and time in its accessible name.
- The context usage figure is text; the cell grid is decorative with a text equivalent.

## Responsive
- Wide: stream left, plan and context stacked right; all three visible.
- Medium: plan and context move under the stream; the stream height is capped at `stream_height_cap`.
- Phone width: single column; agent tabs scroll horizontally inside their own row only.

## Composition hints
- Tabs (agent tabs), log list (messages), ordered list with status mark (plan), progress bar and cell grid (context), text area with mention, badge (status), button (pause and send).

## Acceptance checks
1. Clicking an agent tab shows only that agent's messages and messages addressed to it, and dims the other plan steps.
2. Clicking Everyone or pressing Escape restores all messages and steps.
3. A step moving to done shows its done mark and the plan's running time updates.
4. Enter sends the composed message; Shift + Enter adds a line and does not send.
5. Typing `@` in the composer opens the agent list; Enter inserts the chosen agent.
6. Pause changes the live indicator to paused and stops step advancement.
7. Up and Down move focus between messages in order; Enter opens the message details.
8. At phone width the three regions stack in order and nothing scrolls sideways outside the agent tab row.
