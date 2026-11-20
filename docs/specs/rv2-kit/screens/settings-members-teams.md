# Settings: Members and Teams

Source item: affectUI Pro `screens/settings-members` (functional reference only; see README).

## Purpose
A workspace administrator sees how many seats are in use, finds people, changes their roles, removes them, invites new people, exports the list, and groups members into teams.

## Data
| Field | Type | Meaning |
|---|---|---|
| member_list | { id; display_name?: string; email_address: string; member_role: "admin" \| "member" \| "viewer"; plan_name: string; usage_ratio: number (0 to 1); is_invited?: boolean }[] | Workspace members. |
| team_list | { id; display_name: string; member_ids: string[] }[] | Teams. |
| seats | { used: number; total: number } | Seat counts. |
| current_user_id | string | Marked "(You)" and never removed. |
| role_filter | "all" \| "admin" \| "member" \| "viewer" | Role filter. |
| query | string | Search across name and email. |
| default_tab | "members" \| "teams" | Tab opened first. |
| is_loading | boolean | Numbers, rows, and teams shown as secondary shapes; counts hidden. |
| is_empty | boolean | Counts read zero; a no-members message with an invite button; a no-teams message with a new-team action. |
| on_navigate | (label: string) => void | Settings navigation; "Back to app". |
| on_invite | (emails: string[], member_role) => void | Invite form submitted. |
| on_change_role | (member_id, member_role) => void | A role was chosen. |
| on_remove | (member_id) => void | Remove a member. |
| on_bulk | (member_ids, action: "role" \| "remove" \| "plan", value?) => void | Bulk changes. |
| on_export | () => void | Export the member list. |
| on_create_team / on_edit_team | callbacks | Team actions. |

## Anatomy
Reading order:
1. Settings frame: Members marked current.
2. Page header: the title "Members and teams", a one-line description with a link to documentation.
3. Summary strip: seats used of total; members count with invited count; admins count with a note on what admins can do; a usage note.
4. Tab strip: Members, Teams.
5. Members tab: a toolbar with search, a role filter, Export, and Invite members; a bulk-action bar that appears when rows are selected; a table with a checkbox, name and email, a role select, plan name, a usage bar, and a row menu.
6. Teams tab: a list of teams with member counts and a New team control; team rows expand to show their members and can be edited.

## States
- **Loaded.** The default.
- **Loading.** Numbers, table rows, and the team list shown as secondary shapes; counts hidden.
- **Rows selected.** The bulk bar shows the count and its actions; the header checkbox shows a partial state when only some rows are selected.
- **Filtered, no match.** A no-match message, with a clear control.
- **Invite form open.** An inline form with an email chip input and a role choice; Enter sends.
- **Invited row.** Shows an invited label and a resend or cancel action in place of usage.
- **Empty.** A no-members message with an invite action; a no-teams message with a new-team action.
- **Current user row.** Shows "(You)"; its role control and remove control are disabled.

## Interactions
- Search: typing filters rows; Escape clears it.
- Role filter: a menu of choices.
- Row checkbox: Space toggles it; Shift+click selects a range.
- Header checkbox: selects all visible rows.
- A row's role select opens a menu; Enter confirms and calls `on_change_role`.
- Row menu: remove (after confirmation) and other per-row actions.
- Bulk bar: change role, change plan, remove selected.
- Invite: Enter sends; Escape closes.
- Export calls `on_export`.
- Teams tab: arrow keys switch tabs; Enter expands a team row.

## Motion (functional)
- Removed rows close the gap; a changed role updates in place.
- The bulk bar slides in when the first row is selected and out when the selection empties.
- Each change is confirmed with a toast.

## Accessibility
- Tables use table roles. Each checkbox, role select, plan label, and row menu includes the person's name in its accessible label.
- Submitting the invite form works on Enter, and dismissing it works on Escape.
- Each change is reported through a status message.
- The disabled controls on the current user's row explain why in their labels.

## Responsive
- In narrow panels, the summary numbers stack; search takes its own line; plan and usage columns give way to name and role.
- The screen fills its container.

## Composition hints
- Settings frame.
- Summary stat strip.
- Tab strip (Members, Teams).
- Data table with selection, role select, usage bar, and row menu.
- Bulk action bar; chip input for invites; toolbar with search, filter, and export.
- Team list with expandable rows; empty states; toast.

## Acceptance checks
1. Selecting two rows shows a bulk bar reading "two selected"; the header checkbox shows a partial state.
2. Changing one member's role calls `on_change_role` with that member's id and the new role.
3. The current user's role select and remove control are disabled, and their labels say why.
4. Inviting two addresses with Enter calls `on_invite` once, with both addresses and the chosen role.
5. Escape in the invite form closes it without calling `on_invite`.
6. Removing a member asks for confirmation; confirming calls `on_remove` once.
7. Shift+click selects a contiguous range of rows.
8. Export calls `on_export`.
9. The Teams tab lists each team with its member count; expanding a team shows its members.
10. Below a narrow width the summary numbers stack, with no horizontal page scroll; the table keeps name and role visible.
