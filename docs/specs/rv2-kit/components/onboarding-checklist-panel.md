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

## Amendment: structure and demo

Layout archetype: a compact floating card anchored in a dashboard corner, with three stacked bands: a header, a progress band, and a group list. Groups stack vertically inside the list, separated by dividers. The card is open with its first groups expanded in the gallery.

Parts that must exist:
- Header band: panel title at the leading edge; at the trailing edge, an "Edit" text control (when `on_edit` is supplied) followed by a close icon control (when `on_dismiss` is supplied).
- Progress band: a horizontal bar filling the width, then at the trailing edge the text "n of m done". The bar's value is the percentage.
- Group list: one section per group, in order. Each section header has the group title at the leading edge, the group's own "done of total" count at the trailing edge, and a disclosure control. Task rows sit beneath the header when the group is open.
- Task row: a toggle at the leading edge and the task text. Locked rows show a dimmed toggle and text and are disabled.
- Complete group: header struck through, rows hidden, no disclosure content.

Demo data for the gallery (our own names and values):
- Three groups with three, two and one tasks: six tasks total.
- Three tasks done in total.
- Group one: two tasks done, one pending. Group two: both tasks locked (its predecessor group is not finished). Group three: its single task done, so the group is complete and folded.
- Edit and close controls both supplied.
- Progress value: one half.

States visible at rest in the gallery: group one open with a pending task, group two open with locked tasks, group three complete and folded with its header struck through, progress bar at half with its count, both header controls present.

Acceptance checks:
9. The gallery demo shows the card anchored in a corner, with header, progress band and group list stacked in that order.
10. The header shows the title at the leading edge and the Edit control and close icon at the trailing edge.
11. The progress bar is half filled and its trailing label reads three of six done.
12. The group list shows three groups in order; each header shows its title at the leading edge and its done-of-total count at the trailing edge.
13. Group one shows two ticked rows and one pending row; group two shows two disabled locked rows; group three shows a struck header and no rows.
14. The gallery demo shows no focus outline and no open confirmation at rest.
