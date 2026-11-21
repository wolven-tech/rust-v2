# Onboarding Checklist Panel

Source item: affectUI Pro `components/setup-guide` (functional reference only; see README).

## Purpose
A floating panel in a dashboard corner that guides a person through setup. Tasks are grouped; each group contains tasks that build on one another. Finished work is struck through, and the panel shows overall progress. People can fold it away and come back to it.

## Data
| Field | Type | Meaning |
|---|---|---|
| `group_list` | `Group[]` | Each: `group_id`, `group_title`, `task_list`. |
| `check_item` | `{ item_id, item_text, is_done?, is_locked? }` | `is_done` marks completion. A task stays disabled until the task before it is finished. |
| `panel_title` | string | Heading. Default "Setup guide". |
| `on_edit` | () => void, optional | Shows an edit control in the header. |
| `on_dismiss` | () => void, optional | Shows a close control in the header. |
| `on_change` | (done_ids) => void | Receives the ids of every finished task after each tick. |

## Anatomy
In reading order:
1. Header: title, optional edit and close controls.
2. Progress bar with a percentage of finished tasks, and a count of finished over total.
3. Groups, each as a collapsible section: a header with the group title, its own count, and an expand or collapse control; below it the task rows.
4. Task rows: a toggle with the task text.

## States
- **Group open / closed:** groups toggle independently; closed groups hide their rows from view and from the accessibility tree.
- **Task pending:** unchecked and enabled.
- **Task locked:** disabled until its predecessor is done.
- **Task done:** struck through; shown as pressed.
- **Group complete:** the group header is struck through, the group folds away, and the next group opens automatically.
- **All complete:** progress is full; the panel shows a completion line.
- **Panel dismissed:** the panel is hidden; reopening is provided by the host.

## Interactions
- **Pointer, group header:** expands or collapses.
- **Pointer, task:** ticks or unticks an unlocked task; calls `on_change`.
- **Pointer, edit or close:** calls the matching handler.
- **Keyboard, Tab:** moves through group headers, tasks, then header controls.
- **Keyboard, Enter or Space:** toggles a group header, or ticks or unticks a task.
- **Keyboard, locked task:** can be focused for context but cannot be toggled.
- Unticking a task that unlocked later tasks re-locks them.

## Motion (functional)
- Groups unfold and fold smoothly.
- A finished group is struck through, collapses, and hands focus to the following group.
- Progress updates smoothly.
- Reduced-motion preference: transitions are off.

## Accessibility
- Group headers are buttons with an expanded state; collapsed groups are inert.
- Locked tasks are disabled; done tasks are pressed toggle buttons.
- The progress bar is a progress indicator whose value is the percentage.
- Completion of a group is announced politely.

## Responsive
- The panel stays in its corner at desktop widths and becomes a full-width sheet at phone widths.
- Touch targets are enlarged.

## Composition hints
- Card or floating panel for the container.
- Progress bar primitive.
- Disclosure (collapsible) for groups.
- Checkbox or toggle button for tasks.
- Icon buttons for edit and close.

## Acceptance checks
1. Ticking the first task in a group unlocks the second and calls `on_change` with the new done ids.
2. Unticking a task re-locks the tasks after it that depended on it.
3. A group whose tasks are all done shows its header struck through and the next group open.
4. The progress percentage equals done tasks divided by total tasks, rounded down.
5. Locked tasks cannot be toggled by pointer or keyboard.
6. Collapsed groups do not expose their tasks to assistive technology.
7. The edit control appears only when `on_edit` is supplied.
8. With reduced motion requested, group transitions are immediate.
