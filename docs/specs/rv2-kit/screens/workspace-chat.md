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
- Left and right arrows on tabs move between them. Delete closes the focused tab; closing the last one folds the work area and moves focus to its open control.
- Enter in the composer sends; Shift+Enter adds a line.
- Ctrl or Cmd+K opens global search from anywhere.
- Changing a field calls `on_field_change`.

## Motion (functional)
- Agent state indicators show live work with a subtle pulse.
- Reduced motion: static indicators.

## Accessibility
- Each sidebar entry, tab, and toolbar command is a named control.
- Tabs follow the tab pattern; each close control is labelled with its tab's name.
- Icons are decorative and hidden from screen readers.
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
