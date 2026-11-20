# Settings: Notification Rules

Source item: affectUI Pro `screens/settings-notifications` (functional reference only; see README).

## Purpose
A person decides what they hear about and through which channel: a daily digest at a chosen time, a weekly report, a per-type grid that routes each alert to email, mobile, both channels, or the inbox alone, and quiet hours during which mobile pushes wait.

## Data
| Field | Type | Meaning |
|---|---|---|
| digest | { is_enabled: boolean; send_time: string (HH:MM); time_zone: string; includes: string[] } | Daily digest. |
| weekly_report | { is_enabled: boolean; day: string } | Weekly usage report. |
| types | { id; type_name: string; type_note: string; delivery: "both" \| "email" \| "mobile" \| "inbox" }[] | Notification types with their delivery channel. |
| quiet_hours | { is_enabled: boolean; start_time: string; end_time: string } | Window in which mobile pushes are held. |
| is_loading | boolean | Switches, times, and channels shown as secondary shapes. |
| on_save | (values) => void | Called with all settings on every change. |
| on_navigate | (label: string) => void | Settings navigation. |

## Anatomy
Reading order:
1. Settings frame: Notifications marked current.
2. Page header: the title "Notifications", a one-line description.
3. Digest card: a switch row with a label and a note; when on, a time field and a timezone line appear beneath it. Below that, the weekly usage report switch row.
4. Notification types card: a heading and a one-line instruction, then a table:
   - Column "Type": the title and short note for each type.
   - Column "Channel": a channel menu per type. The table head holds a control that sets every row at once.
5. Quiet hours card: a switch row; when on, from and to time fields appear.

## States
- **Loaded.** The default.
- **Loading.** Secondary shapes for switches, times, and channel labels.
- **Digest off.** The time and timezone rows are hidden; the switch row stays.
- **Quiet hours off.** The from and to fields are hidden.
- **Channel menu open.** Lists inbox only, email, mobile, and both, with the current choice checked.
- **Set-all used.** Every row takes the chosen channel, and each row updates.
- **Rows appearing.** Dependent rows rise in when their switch turns on.

## Interactions
- Switches: click or Space toggles; the row title is the switch's name.
- Digest time: a time field; typing or choosing a time saves on change.
- Each row's channel menu: Enter or Space opens it; arrows choose; Enter confirms; Escape closes.
- The table head control sets every row to one channel.
- Quiet hours fields: type or choose a time; the from and to values stay independent.
- Settings navigation: click or Enter calls `on_navigate`.

## Motion (functional)
- Dependent rows (digest time, quiet-hours fields) rise in when their switch turns on and leave when it turns off.
- Channel changes update in place.
- Under reduced motion, dependent rows appear and disappear without movement.

## Accessibility
- Switches have the switch role, with the row title as their accessible name.
- The channel menu on each row carries its type name, for example "Channel for task reminders".
- The table head control is labelled as setting all rows.
- Dependent rows are reachable in order directly after their switch.

## Responsive
- Below a narrow width, each select moves beneath its label; the table head keeps only the set-all control.
- The screen fills its container.

## Composition hints
- Settings frame.
- Settings rows (switch rows), time field, timezone text.
- Data table with a per-row select and a table-head bulk control.
- Card sections with headings.

## Acceptance checks
1. Turning the daily digest on reveals its time field; turning it off hides the field.
2. Changing a type's channel to "email" calls `on_save` with that type's channel set to email.
3. The table-head control sets every type to the same channel in one save.
4. Turning quiet hours on reveals the from and to fields with their current values.
5. Every channel menu's label includes its type name.
6. Switch rows report their on or off state to assistive technology.
7. Under reduced motion, the dependent rows appear with no movement.
8. Clicking "Notifications" or another settings item calls `on_navigate` with that label.
9. Below a narrow width, each select stacks beneath its label.
10. Every change calls `on_save` with the full set of values.
