# Prompt Input Bar

Source item: affectUI Pro `components/chat-composer` (functional reference only; see README).

## Purpose
The input at the foot of an assistant or agent chat. Takes a message, lets the person choose a model, attach things through a plus menu, pick an approval policy, pick a working folder, link services, dictate by voice, and send.

## Data
| Field | Type | Meaning |
|---|---|---|
| `empty_hint` | string | Empty-field hint |
| `model_options` | Model[] | `{ model_id, model_name, model_mark?, model_note? }`. First is selected by default |
| `voice_sample_text` | string | Demo transcript shown while recording. Replace with real speech recognition |
| `on_submit(text, { model_id })` | callback | Send or Enter |
| `on_model_pick(model)` | callback | Model picked |
| `on_attach_pick(action)` | callback | Plus-menu entry picked. Actions: `attach_file`, `attach_connector`, `pick_folder`, `set_goal`, `toggle_plan_mode`, `attach_skill`, `attach_drawing`, `attach_document`, `attach_pdf` |
| `approval_policy` | `"confirm_each" \| "auto_safe" \| "unrestricted"` | Starting policy: confirm each action, approve safe actions automatically, or unrestricted |
| `on_policy_pick(policy)` | callback | Policy picked |
| `on_help_open()` | callback | Help link in the policy menu |
| `folder_options` | string[] | Options for the folder picker. Empty list hides the picker |
| `selected_folder` | string | Selected folder; the row shows its name |
| `on_folder_pick(folder \| null)` | callback | `null` when the selected folder is picked again |
| `service_options` | Service[] | `{ service_id, service_name, service_mark? }` for the link-services menu |
| `linked_service_ids` | string[] | Linked services at start |
| `on_links_change(ids)` | callback | Called with the ids of all linked services |
| `custom_footer` | node | Replaces the folder and service row with custom content |
| `show_context_row` | boolean | Show the muted row with folder and services. Set false once a conversation has started; services then move into the plus menu |
| `root_class` | string | Extra class on root |

## Anatomy
Reading order:
1. Writing area: multi-line text field, grows to a limit then scrolls.
2. Voice-dictation strip (only while recording): heard words in muted tone, confirmed words in normal text, a live level trace.
3. Context row (muted layer beneath the writing area, when `show_context_row`): folder picker at left, services picker at right, or custom footer.
4. Control row inside the writing area: plus button, policy control, model picker, microphone, send.

## States
- **Empty**: hint shown; send disabled.
- **Typing**: send enabled for non-empty text.
- **Recording**: dictation strip visible; text appended on confirmation.
- **Menu open**: plus, policy, model, folder or services menu; the menu sits beside its button and stays within the visible frame.
- **Policy set to unrestricted**: policy control uses a danger tint and a label.
- **Folder selected**: row shows folder name, truncated with full name in tooltip.
- **Services linked**: service marks show in the row (first three only).
- **Sending**: input is cleared after `on_submit`; no duplicate sends on repeated Enter.
- **Context row hidden**: no folder or services row; services are reached through the plus menu.

## Interactions
- Enter: send. Shift+Enter: new line.
- Enter, Space or Down on a control button: opens its menu.
- Up/Down inside an open menu: previous or next entry. Home/End: first or last.
- Enter or Space on an entry: picks it. In the services menu, picks toggle the link.
- Escape: closes a menu and returns focus to its button; while recording, discards the recording.
- Tab: closes a menu and moves on from its button.
- Pointer: clicking anywhere on the writing area focuses the text field.

## Motion (functional)
- The policy control briefly changes its icon and label when the policy changes.
- Menus open out of the button that spawned them, toward the side with more room.
- Hover highlight moves between menu rows.
- While recording, confirmed words settle from muted to normal; a trace shows input level.
- Reduced motion: no transitions.

## Accessibility
- Plus button opens a menu of items; plan mode is a checkable item.
- Policy, model and folder controls open listboxes. Services menu uses checkable items.
- Each button reports its expanded state.
- Picks (policy, model, folder, link) are announced politely.
- Long folder names expose full text via tooltip and in the picker.
- Menus use active-descendant so the highlight is announced.

## Responsive
- Fills its container up to a readable width; controls keep a fixed size.
- Narrow: the policy control shows only its icon; the folder name shortens first so the services control keeps its place.
- Touch: no hover-only functions.
- No horizontal scroll at phone width.

## Composition hints
Textarea (writing area), button and icon-button (controls), menu and listbox (plus, policy, model, folder, services), checkbox items, tag or badge (policy), avatar or logo chip (services), live region helper, waveform or level trace (dictation).

## Acceptance checks
1. Enter sends the text once through `on_submit` with the selected model's id, and clears the field.
2. Shift+Enter inserts a new line and does not send.
3. Choosing a different model calls `on_model_pick` with that model and updates the control label.
4. Toggling a service in the services menu calls `on_links_change` with the new full list of linked ids.
5. Choosing "Unrestricted" sets the control to the danger style and calls `on_policy_pick("unrestricted")`.
6. Escape in an open menu closes it and focus returns to the button that opened it.
7. With `show_context_row` false, no folder or services row is rendered and the service items appear in the plus menu.
8. Picking the already-selected folder calls `on_folder_pick(null)`.
9. Clicking blank space in the writing area focuses the text field.
10. At phone width, all controls remain reachable and no horizontal scroll occurs.
