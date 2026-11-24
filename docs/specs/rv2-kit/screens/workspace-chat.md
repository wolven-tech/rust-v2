# Workspace Chat

Source item: affectUI Pro `screens/agent-chat` (functional reference only; see README).

## Purpose
A chat with an orchestrating agent that ends in a concrete task. Beside the conversation, the agents currently at work are shown, and a brief for the current goal is filled in alongside.

## Data
- `workspace_name`, `account`.
- `chat_list`: list of `{ id, entry_title, is_pinned?, project_id?, updated_at }`, plus `project_list`.
- `active_chat`: `{ id, message_list, agent_status_list: [{ id, entry_name, node_role, entry_state, current_step? }], orchestrator_status }`.
- `brief_record`: `{ entry_title, field_list: [{ id, entry_label, entry_value, field_type: "text" | "choice" | "multi-choice" | "number" }], file_list: [{ entry_name, file_kind: "document" | "sheet" | "folder" }], activity_log: [{ at, text }] }`.
- `approval_mode`: `"auto" | "approve"` (how the agent asks before acting).
- `is_loading`, `is_empty`.
- Callbacks: `on_send(text, attachments)`, `on_select_chat(id)`, `on_field_change(id, entry_value)`, `on_generate()`, `on_open_file(entry_name)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar: navigation entries (new chat, folders, integrations), pinned chats, projects with their chats, and the account at the foot.
2. Conversation column: messages; agent messages show a strip of the agents at work with their current step; the composer sits at the foot with an approval-mode control (automatic, or approve each action).
3. Work area (right): tabs. The first tab is the Brief: labelled fields the person fills in, a files list, and an activity list. A Generate button sits below the fields, with a short line stating what it will produce and roughly how long it takes.
4. Files open in their own tabs, using the shared viewers (document, sheet with formulas, folder).

## States
- **Empty** (`is_empty`): no chats; the conversation shows a note and a Start a chat action (navigates to New chat); each of the brief's three sections says what will appear there.
- **Chat open**: the messages and brief for that chat.
- **Agents working**: the strip shows each agent's state and current step.
- **File tabs open**: extra tabs after the Brief tab.
- **Work area closed**: when the last tab closes, the area folds; the Brief returns when the area is opened again.
- **Loading** (`is_loading`): placeholders for the chat name, messages, the agent strip, and the brief sections; sending waits until loading ends.

## Interactions
- Tab moves through the sidebar, then the conversation, then the work area tabs.
- Left and right arrows move focus between tabs. Delete closes the focused tab; when the last tab closes, the work area folds and focus goes to its open control.
- Enter in the composer sends; Shift+Enter adds a line.
- Ctrl or Cmd+K opens global search from anywhere.
- Changing a field calls `on_field_change`.

## Motion (functional)
- Agent state indicators show live work with a subtle pulse.
- Reduced motion: static indicators.

## Accessibility
- Each sidebar entry, tab, and toolbar command is a named control.
- Tabs follow the tab pattern; each close control is labelled with its tab's name.
- Icons carry no meaning on their own and are skipped by assistive technology.
- Agent state is stated in words.

## Responsive
- Fills its container; the host sets the height.
- Narrow: the sidebar collapses; the work area becomes a sheet or drawer.
- On touch input, controls grow to a comfortable touch target.

## Composition hints
- Sidebar navigation; chat list; conversation message; agent status strip; composer with approval-mode control; tabs; form fields; file tiles; activity list; button; skeleton; empty state.

## Acceptance checks
1. Selecting another chat replaces the conversation and the brief.
2. Left and right arrows move focus between work area tabs.
3. Closing the last tab folds the work area; reopening it shows the Brief tab.
4. With `is_empty` set, the note and the Start a chat action appear, and each brief section names what will appear there.
5. Enter in the composer calls `on_send` with the typed text and clears the field.
6. Each agent in the strip shows its state in words.
7. At phone width the work area is a sheet, and the page does not scroll horizontally.

## Amendment 5: structure and demo

Replaces: none (no earlier amendment in this spec).

**Archetype:** conversation column with an agent status strip, beside a tabbed work area whose first tab is a brief form.

**At rest (gallery demo):**
- One active chat with four messages: a user request; an agent reply with a work-time line and a folded step group; an agent question that waits for an answer; the user's answer.
- Agent strip above the composer: five agents (one orchestrator, four workers). Each shows a state word and its current step.
- Composer at the foot with the approval-mode control set to automatic.
- Work area open with the Brief tab active. Brief fields in order: goal (text), selling angle (choice, four options), audience (multi-choice, three options), channels (multi-choice shown as three switches), palette (choice, four named options), formats (choice, three options), launch date, budget (number).
- Below the fields: a Generate button with one line stating what it produces.
- Below that: a files list with two rows, and an activity list with three entries, newest first.

**States at rest:** chat open; three agents working, one done, one waiting; Brief tab active; no file tab open.

**Arrangement:** conversation column takes the middle width; work area on the right; agent strip sits between the thread and the composer.

**Motion, settled frame:** agent indicators show their state as text and a static mark. Pulsing applies only to working agents, and not under reduced motion.

**Checks (continued):**
8. At rest the open chat shows four messages, and the last agent message ends with a question the composer can answer.
9. The agent strip lists five agents, each with a state word and a current step.
10. The Brief tab lists its eight fields in the order above. Channels render as three switches; each choice field lists its options.
11. Generate sits below the last brief field and has one line naming what it will produce.
12. The Brief tab lists two files and at least three activity entries, newest first.

## Amendment 6: structure and demo

Replaces, from Amendment 5 (this amendment wins where they differ):
- The at-rest agent strip and its Arrangement line. The five worker agents sit in the conversation, directly under the last agent message. The orchestrator is one line above the composer and is not one of the five.
- "Four messages" and the folded step group. The thread has no folded step group at rest and no user answer after the question.
- The Brief field list and the activity list under it. Activity is its own tab, not part of the Brief.
- Checks 8, 9, 10 and 12 are superseded by 13, 14, 15 and 17. Check 11 stands.

**Archetype:** conversation column with the worker agents under the thread, beside a tabbed work area whose Brief tab holds a product card, fields and a files list.

**Parts visible at rest (gallery demo):**
- Thread, in order: one user message; an agent block with a one-line acknowledgement; a work-time line; an agent block with a short reply that ends in a question the composer can answer. The thread ends on that question.
- Under the last agent block: five worker chips, each with its name and a state word. Three are working, one is done, one waits for the person.
- Above the composer: one line naming the orchestrator and its job.
- Composer at the foot, approval-mode control set to automatic.
- Work area open. Tabs in order: Brief (active), Assets, Activity, then the add control. No file tab is open.
- Brief tab, top to bottom: a product card (name, price, one line of sales figures, a top-seller badge); selling angle (choice, three options); audience (multi-choice, four options as chips); channels (three switches); palette (choice, four named swatches); formats (choice, three options); details (launch date, markets as multi-choice with three options, budget as a number); a Generate button with one line stating what it produces; a files list with three rows, each with name and kind.

**States at rest:** chat open; three workers working, one done, one waiting; Brief tab active; no file tab open.

**Arrangement:** the conversation column takes the middle width and holds the worker chips under the thread. The work area is on the right, and its Brief content fits the stage height.

**Motion, settled frame:** as Amendment 5. The waiting chip is static.

**Defects to fix in ours:**
- The worker strip sits only above the composer. At rest it sits under the thread.
- The Brief runs past the bottom of the stage and the files list is cut.

**Checks (continued):**
13. At rest the thread shows one user message, then an agent acknowledgement, a work-time line, and an agent reply that ends in a question. No folded step group appears.
14. Five worker chips sit under the last agent message, each with a state word. The orchestrator line sits above the composer and is not counted among them.
15. The Brief tab lists, in order: product card, selling angle (three options), audience (four options), channels (three switches), palette (four swatches), formats (three options), launch date, markets (three options), budget (number). No goal text field is shown at rest.
16. Generate sits below the last Brief field and has one line naming what it produces.
17. The files list has three rows, each with name and kind. The Activity tab is not active at rest and holds at least three entries, newest first.
18. At 1280 x 720 without page scroll, the thread, worker chips, composer, work area tabs, every Brief section through Generate, and the files list are all visible.
