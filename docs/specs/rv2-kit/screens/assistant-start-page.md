# Assistant Start Page

Source item: affectUI Pro `screens/assistant-home` (functional reference only; see README).

## Purpose
The first screen an assistant shows. A greeting and a prompt box lead directly into work. Shortcuts expose the assistant's tools, the sources it is connected to are listed, and suggestions offer a place to start.

## Data
- `greeting_text`: the greeting line (the host may compute it from the local time of day).
- `account`: `{ entry_name, avatar_url? }`.
- `tool_chip_list`: list of `{ id, entry_label, prompt_template }` (for example document, presentation, table, image, deep research, data visualisation, company knowledge).
- `connected_app_list`: list of `{ id, entry_name, app_mark? }`.
- `suggestion_list`: list of `{ id, entry_label, prompt_text }`.
- `suggested_artifacts`: artifacts offered inside a new work-area tab.
- `workspace_name`, `is_loading`, `is_empty`.
- Callbacks: `on_send(text, attachments)`, `on_navigate(entry_label)` (sidebar items and projects), `on_open_artifact(entry_name)`.

## Anatomy
1. Sidebar (shared app frame).
2. Central column: a greeting heading; the prompt composer (text area, attach control, and a row of tool chips inside its tray); a tray of connected apps, or a "Connect apps" action; and a grid of suggestions.
3. Work area (right; folded at the start, reopened from its icon in the top-right corner). It holds tabs. A New tab offers an address field and Tools and Suggested sections. Opening a suggestion shows the artifact in its own tab.

## States
- **start** (default): work area folded; greeting and composer visible.
- **work area open**: the work area sits beside the central column.
- **typing**: the send control is enabled when text is present.
- **sending** (text submitted while loading): sending waits until loading ends.
- **loading** (`is_loading`): placeholders for the tool chips, the connected apps, and the suggestions.
- **empty** (`is_empty`, no connected apps): the tray shows a Connect apps action in place of app marks.
- **artifact tab open**: the suggested artifact is shown in its own tab.

## Interactions
- Enter in the composer sends; Shift+Enter adds a line.
- Clicking a tool chip fills the composer with that tool's prompt template and places the caret at its end.
- Clicking a suggestion sends its prompt, or opens its artifact when it is an artifact suggestion.
- Ctrl or Cmd+K opens search from anywhere.
- Tab order: sidebar, composer, tray, suggestions, work area.
- Work area tabs: arrow keys move between tabs; Delete closes the focused tab; "+" opens a New tab; closing the last tab folds the work area and moves focus to its open control.

## Motion (functional)
- The work area slides in when opened and slides out when folded.
- Reduced motion: no sliding.

## Accessibility
- Sidebar items, projects, and the collapse control are labelled buttons.
- The work area tabs follow the tab pattern; each close control is labelled with its tab's name.
- Decorative icons are hidden from assistive technology.
- The composer has a label and receives focus on load.

## Responsive
- Fills its container; the host sets the height.
- Phones: the work area becomes a sheet from the right; the tool chips scroll horizontally.
- Touch targets are enlarged for touch input.

## Composition hints
Greeting heading; composer with a tool chip row; app chip tray; suggestion cards; work area tabs; document, sheet, and folder viewers (shared); skeleton; empty state.

## Acceptance checks
1. The greeting line is shown on load.
2. Sending text from the composer calls `on_send` with that text and clears the field.
3. Clicking a tool chip fills the composer with its prompt and does not send it.
4. Closing the last work area tab folds the area and moves focus to its open control.
5. Opening a suggested artifact creates its own tab in the work area.
6. With `is_loading` set, placeholders appear for the chips, the apps, and the suggestions, and sending waits.
7. With no connected apps, the tray shows the Connect apps action.

## Amendment 6: structure and demo

Replaces: none (no earlier amendment). Overrides Anatomy item 2 for the at-rest demo: the tool chips sit below the composer, not inside its tray, and connected apps appear as marks in the composer's footer row, not as a separate tray.

**Archetype:** a centred start column: greeting, composer, and two rows of quick chips beneath it, with the work area folded.

**Parts visible at rest (gallery demo):**
- Greeting line, worded by local time of day, followed by a short invitation question.
- Composer: text area with a placeholder. Footer row inside the composer: at the left, a working-folder control and an approval-mode control set to automatic; at the right, a Connect apps action followed by three app marks. Attach control on the composer's edge; send control disabled while the field is empty.
- Seven chips in two rows, centred under the composer: row one holds four (company knowledge, document, presentation, table); row two holds three (image, deep research, data visualisation).
- Work area folded. Its reopen control is the only element at the top right.
- No suggestion cards in the content column. Suggestions appear in the work area's New tab, under Suggested.

**States at rest:** start state; three apps connected; work area folded.

**Composition:** one width-limited column, greeting above composer above chip rows. Chips are plain labelled chips with a leading icon.

**Motion, settled frame:** nothing moves at rest.

**Defects to fix in ours:**
- Ours shows three chips and two suggestion cards in the content column. At rest the demo shows seven chips and no suggestion cards.
- The app tray is a separate block. At rest the apps are footer marks.

**Checks (continued):**
8. At 1280 x 720 without page scroll, the greeting, the composer with its footer row, both chip rows and the work area's reopen control are all visible.
9. Seven chips show at rest, in rows of four and three, centred under the composer.
10. The footer row shows the working-folder control at its left and the Connect apps action with three app marks at its right. With no connected apps, the action appears alone.
11. No suggestion cards appear in the content column at rest. Opening the work area's New tab shows Tools and Suggested sections.
12. No label is clipped at 1280 or 390 px.
