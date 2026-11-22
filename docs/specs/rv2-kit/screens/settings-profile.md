# Settings: Profile

Source item: affectUI Pro `screens/settings-profile` (functional reference only; see README).

## Purpose
A person maintains their own identity inside the product: picture, name, job role, account email, the time preferences used by reminders and reports, a personal context the assistant reads, and how they sign in.

## Data
| Field | Type | Meaning |
|---|---|---|
| profile_record | { first_name; last_name; account_email; job_role?: string; time_zone: string; week_start: "monday" \| "sunday"; personal_context: string; avatar_url?: string; has_password: boolean } | The person's profile. |
| role_options | string[] | Choices for the role select. |
| timezone_options | string[] | Choices for the timezone select. |
| context_limit | number | Character limit for personal context. Defaults to 500. |
| is_loading | boolean | Secondary shapes in the real layout. |
| on_save | (profile) => void | Called with the whole profile: text on blur, choices at once. |
| on_navigate | (label: string) => void | Settings navigation, or back to the app. |
| on_change_email / on_set_password | () => void | Actions behind their controls. |

## Anatomy
Reading order:
1. Settings frame: Profile marked current; a back-to-app link.
2. Page header: the title "Profile", a one-line description, and a documentation link.
3. Info line: changes apply to every workspace the person belongs to.
4. Rows card: a picture row (a preview tile showing initials when there is no image, with upload and remove), first name, last name, a role select, and an email row with a Change control.
5. Time preferences card: a timezone select and a week-start choice.
6. Personal context card: a text area with a "used / limit" count.
7. Password row: a note saying sign-in uses an emailed link or, when no password is set, an Add password control.

## States
- **Loaded.** The default.
- **Loading.** Picture, fields, choices, context, and the password note as secondary shapes.
- **Picture removed.** The preview falls back to initials.
- **Context at limit.** The count switches to the error tone; typing is blocked.
- **Email change.** Opens a small form; saving sends a confirmation message.
- **Password not set.** Shows Add password instead of the sign-in note.

## Interactions
- Picture upload and remove: upload opens a file picker; remove clears the picture at once.
- Text fields save on blur.
- Select controls open a menu: arrow keys move through choices, Enter chooses, Escape dismisses.
- Week start is a two-choice radio group; a choice saves at once.
- Personal context saves on blur.
- Email Change and Add password open their forms; Escape cancels.
- Settings navigation: click or Enter calls `on_navigate`.

## Motion (functional)
- A picture change swaps in place.
- The password note and the Add password control swap when a password is set.

## Accessibility
- Each field shows a visible label, and the picture's remove control and the selects expose names.
- Saves are announced in a polite status message.
- Select menus follow the shell's keyboard model.

## Responsive
- In narrow panels, each control moves beneath its label and takes the full width.
- The screen fills its container.

## Composition hints
- Settings frame.
- Settings row (label, control) in a card.
- Avatar with initials and an upload control.
- Text fields, select, radio group, text area with count.
- Status message for saves.

## Acceptance checks
1. Changing the first name and leaving the field calls `on_save` once with the new name.
2. Choosing a timezone from the select calls `on_save` immediately.
3. Setting week start to Sunday saves at once.
4. Removing the picture shows initials in the preview.
5. Personal context blocks input past its limit.
6. With `has_password` false, the password row shows Add password instead of the sign-in note.
7. Email Change opens a form; Escape closes it without a save.
8. Select menus respond to arrow keys, Enter, and Escape as described.
9. Save outcomes are announced in a polite status region.
10. Below a narrow width, each control stacks beneath its label at full width.
