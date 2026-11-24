# Assistant Chat With Progress Panel

Source item: affectUI Pro `screens/agent-chat-progress` (functional reference only; see README).

## Purpose
A plain chat in which the assistant carries out a task using tools. Alongside the conversation, a panel shows how far the work has progressed, what it has produced, and what it drew on: uploads, connected apps, web search, and skills. The person can follow the work without leaving the chat.

## Data
- `chat_list`: sidebar list; `active_chat`: messages.
- Messages: user `{ id, author_role: "user", text }`; assistant `{ id, author_role: "assistant", part_list }`, where parts are either text, or a tool call `{ id, tool_name, app_label?, entry_label, started_at, duration?, arguments_data, result_data?, call_state: "running" | "done" | "failed" }`.
- `step_list`: list of `{ id, entry_label, step_state: "done" | "running" | "pending" }`, with earlier steps folded away.
- `output_list`: list of `{ entry_name, output_kind: "document" | "sheet" | "markdown" | "folder" | "image", app_name? }`.
- `usage_record`: `{ upload_names: string[], app_names: string[], web_search?: { page_count }, skill_names: string[] }`.
- `workspace_name`, `account`, `is_loading`, `is_empty`.
- Callbacks: `on_navigate(entry_label)` (sidebar, and Folders when an output opens), `on_send(text)`, `on_open_output(entry_name)`.

## Anatomy
1. Sidebar (shared app frame).
2. Conversation column: user messages, assistant text, and tool call cards. Each card shows the tool's label and duration, and expands to show its arguments and result as formatted text. The composer sits at the foot, with an approval-mode control.
3. Panel (right, part of the page), with three sections:
   - Progress: the step list, each step marked with the words "done" or "running". Earlier steps fold into a line that can be expanded.
   - Outputs: file rows showing name, kind, and the app mark where known. Long names keep their extension visible. Rows open the file.
   - Used in this session: upload names, connected apps, web search with its page count, and skills used.
4. A bar button that opens and closes the panel and shows the number of outputs.

## States
- **running tool**: the card reads running in words, with a spinner.
- **done tool**: the card reads done, with its duration.
- **failed tool**: the card reads failed and shows the error.
- **step running**: the running step is named and marked as running in words.
- **panel open or closed**: when closed, the chat takes the full width; the bar button stays visible.
- **no outputs**: the Outputs section is hidden; the bar button shows no count.
- **no steps**: Progress states that there are no steps yet.
- **empty chat** (`is_empty`): the sidebar holds no chats. The thread shows a note with a Start a chat action (which opens New chat); Progress shows no steps; the Outputs and Used sections stay hidden; the composer remains.
- **loading** (`is_loading`): placeholders for messages, tool calls, steps, outputs, and used items.

## Interactions
- Tab moves through tool calls, the composer, the panel's bar button, and then the panel's sections, steps, and outputs.
- Enter or Space on a tool call expands or collapses its arguments and result.
- Enter or Space on a section heading folds or unfolds it.
- Enter or Space on "show earlier steps" reveals the folded steps.
- Escape (narrow screens, panel open) closes the panel and returns focus to the bar button.
- Enter on an output row opens it, navigating to Folders where that applies.

## Motion (functional)
- A new step appears at the bottom of Progress, and the running step is updated in place.
- Reduced motion: no transitions.

## Accessibility
- The side panel is a named region sitting alongside the conversation.
- The toggle announces its open or closed state together with the output count.
- Tool calls are buttons with an expanded state, and their status is given in words.
- Section headings are buttons with an expanded state; folded content is removed from the tab order.
- Progress is a list; each new step is announced, and its state appears as a word beside any icon.
- Lengthy file names are cut from the middle, preserving the suffix; screen readers still receive the full name.

## Responsive
- Wide: the panel is part of the page, separated from the chat by an edge; closing it gives the chat the full width.
- Narrow: the panel starts hidden, slides in over the conversation from the right, and is dismissed by its close control or Escape.
- Touch targets are enlarged for touch input.

## Composition hints
Chat message; tool call card (expandable); step list; output file row; used-items list; collapsible section; composer; side panel; bar button with count; skeleton; empty state.

## Acceptance checks
1. Each tool call shows its state in words: running, done, or failed.
2. Expanding a tool call shows its arguments and result.
3. Earlier steps are folded behind a line that shows their count and can be expanded.
4. The bar button's state matches the panel (open or closed), and its count matches the outputs.
5. On a narrow screen, Escape closes the panel and focus returns to the bar button.
6. With no outputs, the Outputs section is absent.
7. A long output name keeps its extension visible.

## Amendment 6: structure and demo

Replaces: none (no earlier amendment). For the at-rest gallery demo, no tool call is running or failed; those states are reached by interaction, as the States list describes.

**Archetype:** plain chat with its tool cards in the thread, beside a narrow panel that is open at rest and has three sections: progress, outputs, and what was used.

**Parts visible at rest (gallery demo):**
- Thread, in order: user message; tool card; assistant text; tool card; assistant text listing three points; user message; tool card; assistant text.
- Each tool card is collapsed and shows its tool label, a short description, the word done, and its duration. All three read done.
- Panel open, with a header that carries the title, the output count and a close control.
- Progress: a fold line reading "3 earlier steps" (expandable), then two steps, each reading done.
- Outputs: a preview card for the newest output, a document, showing its title and its first two lines. Under it, four file rows, each with name, kind word and app mark where known: a document, a sheet, a word-processor file, a markdown file.
- Used in this session: four labelled rows. Uploads (one file name and a count of extra files); Connectors (three app marks); Web search (a page count); Skills (one skill name).
- Composer at the foot, approval-mode control set to automatic.

**States at rest:** panel open; no step running; every tool call done.

**Arrangement:** the chat takes the wide area. The panel is a narrow column at the right, about one fifth of the content width, separated from the chat by an edge. The composer sits at the foot of the chat column.

**Motion, settled frame:** nothing animates.

**Defects to fix in ours:**
- Ours shows a running tool and a failed tool at rest, and no assistant text. At rest the thread is the set above.
- Ours has no preview card and one output row. At rest it has four rows.

**Checks (continued):**
8. At rest the thread holds two user messages, three assistant text blocks and three tool cards. Each tool card reads done with its duration.
9. Progress shows a fold line reading "3 earlier steps". The two visible steps each read done.
10. Outputs shows a preview card for the newest document, then four file rows, each with its kind word and app mark where known.
11. Used in this session has four rows: Uploads, Connectors, Web search, Skills. Connectors shows app marks, not names.
12. At 1280 x 720 without page scroll, the composer and all three panel sections are visible, including every output row and every used row.
13. The panel header shows the output count and a close control. The bar button appears only while the panel is closed.

Doubt: the reference shows no separate bar button at rest. Check 13 keeps the bar button from the spec for the closed state.
