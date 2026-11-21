# Expandable Work Item Card

Source item: affectUI Pro `components/project-card` (functional reference only; see README).

## Purpose
A compact summary of a work item (a project or ticket) that reveals its subtasks, assignees and status when opened. People can tick subtasks, filter by assignee, and change status without leaving the list.

## Data
| Field | Type | Meaning |
|---|---|---|
| `item_title` | string | Name of the item. |
| `subtask_list` | `Subtask[]` | Each: `subtask_id`, `subtask_text`, `is_done?`, `assignee_name?`. |
| `on_subtasks_update` | (subtask_list) => void | Receives the complete list after each tick. If provided, the displayed ticks mirror the host's list exactly. |
| `status_id` | string | Id of the chosen status. |
| `status_options` | `StatusOption[]` | Each: `option_id`, `option_label`, `emphasis?`, `option_mark?`. Emphasis is one of a fixed set of named tones. |
| `on_status_pick` | (option_id) => void | Called with the picked status id. |
| `due_text` | string, optional | Preformatted due date, shown as "Due <value>". |
| `member_list` | `Member[]` | Each: `member_name`, `avatar?`. Shown in the header; initials appear when there is no image. |
| `primary_action` | `{ action_label, on_activate }`, optional | Button in the open card's last row. |
| `footer_hint` | ReactNode, optional | Text in the last row while nothing is focused or hovered. Defaults to a count of open subtasks, or "all done". |
| `open_on` | `"hover" \| "activate"` | Defaults to `"hover"`. Touch and keyboard always toggle regardless. |
| `is_open` / `default_open` | boolean | Controlled or uncontrolled open state. Defaults to closed. |
| `on_open_change` | (is_open) => void | Fired on every open or close transition. |

Assignee matching is by exact member name.

## Anatomy
Closed:
1. Header: title, a segmented progress ring (one segment per subtask), the due date, member avatars, and the status.
2. Summary line combining title, progress and status; it serves as the header's accessible name.

Open (the header stays where it is; content unfolds beneath it):
3. Subtask list: a checkbox row per subtask, showing its text and assignee.
4. Member filter row: a toggle per member.
5. Status options: a group of radio choices, shown when the status control is opened.
6. Footer row: the hint text, and the primary action.

## States
- **Closed:** header and summary only; nothing inside is focusable.
- **Open:** subtasks, members and footer visible.
- **Status options open:** the option group unfolds in place; the rest of the card stays visible.
- **Member filtered:** the chosen member's subtasks and ring segments stay strong; others fade.
- **Subtask done:** the ring segment becomes solid, the check fills, and the header progress updates.
- **All done:** the footer hint reads "all done" by default.
- **Read-only (host choice):** no ticks, no status change, no member filter changes.

Transitions: pointer enter or leave opens or closes the card in hover mode; tap or Enter toggles it in every mode.

## Interactions
- **Pointer, hover (hover mode):** opens the card; leaving closes it.
- **Pointer or tap, activate mode:** toggles open.
- **Pointer, subtask:** clicking ticks or unticks it; the list is re-emitted through `on_subtasks_update`.
- **Pointer, member:** clicking sets or releases the filter for that member.
- **Pointer, status:** opens the status options; clicking an option calls `on_status_pick`.
- **Keyboard, Enter or Space on the header:** opens or closes the card.
- **Keyboard, Tab:** moves through status, members, subtasks, then the primary action.
- **Keyboard, Space:** on a subtask flips its tick; on a member flips its filter; on the status control opens the options.
- **Keyboard, Escape:** closes the status options first; a second press closes the card.
- **Focus on a subtask or member:** applies the same emphasis as hovering it.
- **Outside tap:** closes the card on touch.

## Motion (functional)
- The header does not move. The list unfolds beneath it while fading in.
- Status options unfold in place.
- Hovering a subtask emphasises its ring segment and brings in the assignee.
- Ticking a subtask fills its ring segment and draws a check.
- Hovering a member fades other members' subtasks and segments.
- The footer hint slides up briefly on change; the ring's number animates when it updates.
- Reduced motion: changes happen without movement.

## Accessibility
- The header is a button with an expanded state; its name is the summary (title, progress, status, due date).
- Subtasks are checkboxes; members are toggle buttons; statuses form a radio group.
- While closed, the card's contents are inert: they cannot be focused and are not read.
- Focusing a subtask or member shows the same emphasis as hovering; the footer hint is announced politely when it changes.

## Responsive
- Fills its container up to a moderate width. When the container is narrow, the due date drops its leading word and avatars tighten; long titles truncate with an ellipsis; nothing overlaps the avatars.
- On touch screens, rows and options are taller and assignee avatars are always visible.

## Composition hints
- Card for the container.
- Segmented progress ring primitive.
- Avatar group with initials fallback.
- Checkbox list and toggle buttons.
- Radio group for status.
- Badge for status.
- Button for the primary action.

## Acceptance checks
1. A closed card exposes only its header to keyboard focus and to assistive technology.
2. Pressing Enter on the header opens the card and sets the expanded state to true; pressing it again closes it.
3. Ticking a subtask calls `on_subtasks_update` once with the full list including the change.
4. Selecting a member filter leaves only that member's subtasks at full emphasis.
5. Escape with the status options open closes only the options; a further Escape closes the card.
6. The header's accessible name contains the title, the progress figure, the status label and the due date.
7. When `on_subtasks_update` is supplied, the displayed ticks follow the host's list, not internal state.
8. A subtask whose assignee matches no member is still shown, without a member avatar.
9. When the container is narrow, the title truncates with an ellipsis and avatars do not overlap the title.
</content>
