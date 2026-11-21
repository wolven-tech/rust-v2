# Settings: Workspace General

Source item: affectUI Pro `screens/settings-general` (functional reference only; see README).

## Purpose
An administrator sets the workspace's identity: its icon, name and web address, and a
short company context that every chat in the workspace reads. Deleting the workspace is
possible from the same page, behind a typed confirmation.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_record | { display_name: string; url_slug: string; company_context: string; icon_url?: string } | The workspace. |
| context_limit | number | Maximum characters for company context. Default 2000. |
| on_save | (workspace) => void | Called with the whole workspace on any save: text on blur, icon at once. |
| on_delete | () => void | Called after the name is typed to confirm. |
| on_navigate | (label: string) => void | Settings menu item or "back to app". |
| account | { display_name; avatar_url? } | Shell account block. |
| is_loading | boolean | Icon, display_name, address and context as secondary shapes. |

## Anatomy
Reading order:
1. Settings frame: navigation (Personal and Workspace groups), General marked current,
   back-to-app link.
2. Page header: title "General", one-line description.
3. Rows card, each row a label with help text on the left and its control on the right:
   - Workspace icon: preview tile, a "Upload" button, a remove button when an icon exists.
   - Workspace name: text field.
   - Workspace address: a fixed prefix shown as read-only text, then the slug field; a
     warning that changing it breaks saved links.
4. Company context card: label, explanation (two or three short paragraphs suit it; keep
   out slogans and history), a multi-line text area, and a character count of the form
   "used / limit".
5. Danger zone card: heading, one-line consequence, Delete workspace button.

## States
- **Loaded.** Default.
- **Loading.** Secondary shapes in each row.
- **Saved.** A toast confirms; a field that saves on blur shows no spinner.
- **Context over limit.** Count turns to the error tone; further typing is blocked.
- **Delete confirmation.** A dialog asks for the workspace name; the confirm button stays
  disabled until it matches exactly.
- **Icon removed.** Preview falls back to the workspace's initial letter.

## Interactions
- Icon upload: opens a file picker; Enter or Space on the upload button; remove button
  clears the icon at once.
- Name field and context: save on blur; Enter in the name field saves and leaves the
  field.
- Slug field: saves on blur; a change shows the warning under the field first.
- Delete workspace: opens the confirmation dialog; typing the name enables confirm;
  Escape cancels.
- Settings navigation: click or Enter calls `on_navigate`.

## Motion (functional)
- Icon change swaps the preview in place.
- Confirmation dialog rises over a scrim; the confirm button's enabled state changes in
  place.

## Accessibility
- Every field has a visible label; icon remove and size controls have names.
- Delete confirmation is a dialog; its confirm button is labelled with the workspace name.
- Saves are announced in a polite status message.
- Read-only address prefix is not focusable.

## Responsive
- Below a narrow width, each control moves under its label at full width.
- Screen fills its container.

## Composition hints
- Settings frame.
- Settings row (label, help text, control) inside a card.
- Text field, text area with count, file upload control with preview tile.
- Danger zone card with destructive button; confirmation dialog with typed match.
- Toast for save outcome.

## Acceptance checks
1. Changing the name and tabbing away calls `on_save` once with the new name.
2. Uploading an icon calls `on_save` immediately with the new icon.
3. Typing past the context limit is blocked and the count shows the error tone at the limit.
4. Editing the slug shows the link warning before save.
5. Delete workspace opens a dialog whose confirm button stays disabled until the name matches exactly.
6. Escape closes the delete dialog without calling `on_delete`.
7. Confirming delete calls `on_delete` once.
8. Removing the icon shows the initial letter in the preview.
9. Clicking "Members" in the settings navigation calls `on_navigate` with "Members".
10. Below a narrow width, each control stacks under its label at full width.
