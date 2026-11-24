# Orchestrator Thread

Source item: affectUI Pro `screens/agent-thread` (functional reference only; see README).

## Purpose
A single conversation with an orchestrating agent. The agent replies in text, with small cards for tasks and decisions. Background work folds into one line. Sends and uploads carry on across a lost link, a sluggish transfer, or a long period away.

## Data
- `message_list`: a list of either:
  - user: `{ id, author_role: "user", text, sent_at, attachment_list?, delivery_state: "sent" | "queued" | "sending" | "failed" }`, or
  - agent: `{ id, author_role: "agent", sent_at, part_list, is_streaming?, is_stopped? }`, where `part_list` holds any of: text; a tool group (`{ id, tool_rows: [{ id, entry_label, row_state: "running" | "done" | "failed" | "stopped", started_at?, duration?, error_text? }] }`); a task card (`{ id, entry_title, task_state, step_progress?: { done, total }, is_done? }`); a decision marker; artifact chips (a list of names); or an error (`{ text, retry_allowed? }`).
- `decision_queue`: waiting decisions in order, each `{ id, request_kind: "decision" | "pick" | "approval" | "question", question_text, task_name, task_id?, option_list?: [{ id, entry_title, summary?, is_recommended?, preview_text? }], fact_rows?: [{ entry_label, entry_value }] }`.
- `task_details`: a map from task id to its details (goal, steps, history, artifacts, decisions).
- `artifact_store`: a map from name to artifact content for the work area.
- `connection_state`: `"online" | "offline" | "reconnecting"`, optional. When absent, the screen follows the browser's connection state.
- `last_seen_at`: timestamp; `away_after_minutes`: a number of minutes.
- `running_count`, `needs_count`: counts shown in the header.
- `model_list`: optional. When absent, there is no model picker.
- `suggested_artifacts`, `text_strings`, `account`, `workspace_name`, `is_loading`, `is_empty`.
- Callbacks: `on_send(text, attachments)`, `on_stop()`, `on_retry(message_id)`, `on_reconnect()`, `on_answer(id, answer)`, `on_later(id)`, `on_open_task(task_id)`, `on_open_inbox(item_id?)`, `on_mark_read()`, `upload_file(file, report_progress, cancel_signal)`, `transcribe_clip(clip)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Head: a title, and a quiet line with the running count and the count of decisions awaiting the person. Each count is a button.
3. Message column, centred at a fixed maximum width: user messages right-aligned; agent messages left-aligned with the agent's identity mark; tool groups as one collapsible line showing the tool count and total time; task cards; artifact chips; error lines with Retry; and the away card after an absence.
4. Decision dock: directly above the composer, the first waiting decision as a card with its options (or facts, for an approval) and a pager when more are waiting. Later folds it to a single line.
5. Composer: a multi-line text field, an attach control, a voice-note control, and a send button that becomes a stop button while an answer streams.
6. Jump pill: appears when the person has scrolled up, showing how many new lines have arrived.
7. Connection banner: shown when offline or reconnecting.
8. Work area (right): artifacts and task tabs opened from chips and cards.

## States
- **idle**: messages shown; composer ready.
- **streaming** (`is_streaming`): the last agent message grows word by word, with a caret; send becomes stop.
- **tool running**: rows in the tool group show running; a finished row shows a check.
- **tool group done**: rows fold into one summary line with the total time; the group can be expanded.
- **decision waiting**: the decision dock sits above the composer.
- **decision answered**: a user line reads "Answered: <choice>"; the dock folds; the next decision docks.
- **offline**: the banner shows; new messages show "Waiting to send" and go out once back online.
- **reconnecting**: the banner shows a retry countdown (host-tunable, increasing).
- **failed message**: shows Retry.
- **upload in progress or failed**: the attachment shows progress; a failure shows Retry.
- **dictating**: a voice note is being recorded; Escape discards it.
- **away**: after the tab has been hidden for longer than `away_after_minutes`, an away card summarises what finished and what needs the person, with Open Inbox and Mark as read.
- **empty** (`is_empty`): a centred note and a Start a chat action that opens New chat.
- **loading** (`is_loading`): placeholder messages and a placeholder task tab; the composer stays, and sending waits.

## Interactions
- Enter sends. Shift+Enter adds a line. The field grows to about ten lines, then scrolls.
- Escape: while streaming, stops the answer; while dictating, discards the clip; in the dock, sets Later.
- End jumps to the latest message and resumes following it.
- "/" focuses the composer.
- Pressing 1 through 5 selects the matching option of the docked decision. A free-text row placed after the options moves focus into a typed answer.
- Within the dock, the arrow keys and Home/End move the highlight between options, and Enter answers the highlighted one. Clicking an option answers immediately.
- Tab moves through tool groups (Enter expands or folds), each card's Open or Decide control, the artifact chips, Retry, and the composer buttons.
- Shortcuts do not fire while typing, apart from Enter, Shift+Enter, and Escape in the field.

## Motion (functional)
- Parallel tools finish at their own moments, and each check appears as it does. A moment after the last one finishes, the rows fold into the summary line.
- The answer appears word by word with a caret; running tool names carry a slow highlight.
- Messages and cards rise in slightly.
- The dock folds away once answered; the next one fades in; a dock brought up from elsewhere pulses once.
- Reduced motion: no sliding, folding, or breathing; state changes apply directly.

## Accessibility
- The conversation is a log with polite live updates; it is marked busy while an answer streams. Screen readers read the finished answer, not every word.
- The banner sits in a polite status region, so queued sends, reconnection, halted generation, and failed uploads are read out as they change.
- The jump control's accessible name includes the unread count and mentions the End key.
- Each tool group is a button with an expanded state; collapsed rows are removed from the tab order.
- The decision dock is a group labelled by its question; its options form a listbox with a roving tab stop. Answering from the keyboard moves focus to the composer.
- Where a question was asked earlier in the thread, a button states that it is waiting or answered, and brings the dock up or scrolls to the answer.
- The text field never shows a focus ring; only its edge firms up while typing.

## Responsive
- Fills its container; the host sets the height.
- All blocks (agent text, cards, dock, composer) align to one column edge. The person's own messages stay on the right, capped at a set width.
- The decision card sits above the composer at the same width; the thread above shrinks to make room, so the latest message stays visible.
- At phone width: the column and composer fill the width with side gutters; cards are full width; artifacts open as a sheet.
- On touch input, controls grow to a comfortable touch target.

## Composition hints
Chat message bubble; streaming text block; tool activity group; task card; decision card (option list with pager); away summary card; composer (text area, attach, voice, send or stop); connection banner; jump pill; error line with retry; artifact chips; work area with tabs and artifact viewers; skeleton; empty state; live region.

## Acceptance checks
1. While an answer streams, its text grows in the last agent message; Escape stops it and marks the message stopped.
2. Pressing 2 with a docked option decision answers with the second option and shows an "Answered: <option>" user line.
3. A message sent while offline shows "Waiting to send" and is delivered once the connection returns.
4. Returning to the tab after more than `away_after_minutes` shows the away card.
5. A finished tool group shows one summary line with the total time, and can be expanded.
6. When scrolled up, the Jump pill appears and shows the number of new lines.
7. A failed message shows Retry, and Retry calls `on_retry` with its id.
8. With reduced motion requested, the answer appears without the word-by-word effect, and no card rises in.

## Amendment 7: structure and demo

Replaces: none; no earlier amendment exists. Base checks 1 to 8 stand; checks 9 to 13 are new.

Archetype: one agent thread column in the content area, scrolled to its end: an agent message with a running parallel tool group, a Stop control and a composer.

Stage: the content area beside the host sidebar, which the workspace frame item supplies. Its full height is the stage height. The column sits at 20% to 79% of content width (58% wide, centred).

Top row, 3% to 7% of stage height: right-aligned to the column's right edge, two count labels, each with a dot marker: "9 running" with the success token dot, "5 waiting on you" with the warning token dot. A panel toggle icon sits at the content area's far right. No page title.

Message list, scrolled to its end. Its first row is cut by the top edge. Positions are fractions of stage height:
- 28%: one user bubble, right-aligned to the column.
- 33%: agent header row (agent mark, name, time).
- 37.5%: a finished fold line, one line: check, "Ran 2 tools", total time.
- 42% to 46%: agent text, two lines, left-aligned.
- 48%: two artifact chips, left-aligned.
- 54%: one user bubble, right-aligned.
- 61%: agent header row.
- 65%: group header "Running 3 tools in parallel", with a spinner.
- 68%, 71%, 74%: three tool rows, one per line. Each: a spinner, a tool glyph, a label, and a duration right-aligned in muted text ("0.3s" on each). All three show the running state.
- 82%: Stop control, right-aligned to the column edge. A pill with a filled square and the word "Stop", shown whenever any tool row runs.

Composer, 85% to 96% of stage height, full column width, rounded box:
- Placeholder line at 88%, left-aligned, muted. Gallery text: "Message the orchestrator".
- Bottom row at 94%: left, a "+" icon button and an approval chip with a hand glyph (gallery text: "Needs approval"); right, a mic icon and a send icon. The send icon is muted while the composer is empty.
- No text buttons for attach or voice.

Not drawn at rest: a decision dock, an attachment row, a jump pill, and a title. A decision dock appears only when a decision waits; it is outside the gallery demo.

Settled frame: the group of three rows, the Stop control and the composer. Nothing animates at rest except the spinners.

Acceptance checks (continued):
9. At rest one parallel group shows three rows, each in the running state with its spinner and a duration. The rows are not folded.
10. The Stop control sits above the composer at the column's right edge while any tool row runs. It is absent when no row runs.
11. The composer has a placeholder line, a "+" control and an approval chip at left, and a mic control and a send control at right. No text buttons for attach or voice.
12. The top row holds two counts with dot markers, right-aligned to the column edge. There is no page title, no decision dock and no attachment row at rest.
13. The column is 58% of content width (within 2 points) and centred. No bubble is wider than the column.
