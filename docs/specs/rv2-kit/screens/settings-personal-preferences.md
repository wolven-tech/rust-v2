# Settings: Personal Preferences

Source item: affectUI Pro `screens/settings-preferences` (functional reference only; see README).

## Purpose
A person sets their own defaults on top of what the workspace allows: which model a new chat starts with, the language, the theme, which chat tools are on, custom instructions that accompany every message, and the memories the assistant has kept, which they can review and remove.

## Data
| Field | Type | Meaning |
|---|---|---|
| default_model | string | Model id a new chat starts with. Only models the workspace has enabled can be chosen. |
| default_image_model | string | Image model used for picture requests. |
| models | { id; name; kind: "chat" \| "image" }[] | Models the workspace allows. |
| language | string | Language of menus and answers. |
| theme | "system" \| "light" \| "dark" | Appearance. |
| app_actions | boolean | Beta switch: find app actions without explicit tags. |
| chat_tools | { id; label; description; is_enabled: boolean }[] | Tiles for web search, image generation, memory, and others. |
| instructions | { is_enabled: boolean; about_text: string; response_style: string } | Custom instructions. Each field is limited to 1,500 characters. |
| memories | { id; text: string; saved_at: string }[] | What the assistant has kept. |
| memory_on | boolean | The memory switch, mirrored from the chat tools. |
| is_loading | boolean | Secondary shapes for every block. |
| is_empty | boolean | Nothing saved: no instructions text; the memories region offers "start a chat" or turning memory on. |
| on_save | (preferences) => void | Receives the full set of values. Text fields report on blur; other controls report immediately. |
| on_navigate | (label: string) => void | Settings navigation. |

## Anatomy
Reading order:
1. Settings frame: Preferences marked current.
2. Page header: the title "Preferences", a one-line description.
3. Models card: the default chat model selector with a line of text; a "switch to Auto" suggestion beneath it; the default image model selector.
4. Language and appearance card: a language menu; appearance as a three-choice radio group (system, light, dark); the app-actions beta switch row.
5. Chat tools: a grid of toggle tiles, each with a name and a one-line description.
6. Custom instructions card: a switch to use them, and two text areas (About you, How it should answer), each with a character count. Both fade when the switch is off.
7. Memory card: a list of saved memories, each with its text, the date it was saved, and a Forget control; a Clear all control; an empty state when there are none.

## States
- **Loaded.** The default.
- **Loading.** Secondary shapes for models, language, appearance, switches, tiles, instructions, and memories.
- **Instructions paused.** The text is kept, but both text areas are disabled.
- **Memory off.** The memory list says nothing new is kept, with a link to the memory tile; saved items stay listed.
- **Empty.** No instructions text; the memories region offers "start a chat" or turns memory on.
- **Count near limit.** The count switches to the error tone at the limit.

## Interactions
- Model selectors: a menu of allowed models; Enter chooses; Escape closes.
- "Switch to Auto" sets the default to Auto.
- Language menu: arrows choose; Enter confirms.
- Appearance radio group: arrow keys choose; the change applies at once.
- Chat tile: Space or a click toggles it; its name is the checkbox label.
- Instructions switch: a click or Space pauses or resumes; resuming restores the text.
- Text areas save on blur.
- Forget removes one memory, with a confirmation or an undo.
- Clear all opens a confirmation dialog; Escape cancels.

## Motion (functional)
- Paused instructions fade back without losing their text.
- Forgetting a memory lets the rest of the list close the gap.
- Under reduced motion, state changes are immediate.

## Accessibility
- Chat tools are checkboxes with their names; appearance is a radio group.
- Paused fields are disabled, which blocks accidental input.
- Each memory's Forget button names the memory it removes.
- The character count and the limit are announced when the limit is reached.

## Responsive
- Below a narrow width, controls move beneath their labels and the chat tiles stack.
- The screen fills its container.

## Composition hints
- Settings frame.
- Model selector menu; radio group; language select.
- Toggle tile grid (checkbox semantics).
- Text area with character count and a switch-gated enabled state.
- List of saved items with remove controls; empty state.
- Confirmation dialog for clear all.

## Acceptance checks
1. A model the workspace has not allowed cannot be picked from the menu.
2. Changing appearance to dark calls `on_save` with theme "dark" at once.
3. Toggling a chat tile calls `on_save` with its enabled value changed, and the tile shows its checked state.
4. Pausing custom instructions disables both text areas and keeps their text.
5. Text areas save on blur with the full preference set.
6. Typing past 1,500 characters in a text area is blocked.
7. Forget on a memory removes it and leaves the others in order.
8. Clear all asks for confirmation; Escape keeps the memories.
9. With memory off, the memories region says nothing new is kept.
10. Below a narrow width, the chat tiles stack, and no horizontal page scroll occurs.
