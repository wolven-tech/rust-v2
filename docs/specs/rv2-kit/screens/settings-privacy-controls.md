# Settings: Privacy Controls

Source item: affectUI Pro `screens/settings-privacy` (functional reference only; see README).

## Purpose
A person decides what data about them and their chats is used, which cookies this browser stores, whether they receive product email, which signed-in devices remain, and which chats they have shared by link. They can also export or delete all of their chats.

## Data
| Field | Type | Meaning |
|---|---|---|
| ad_measurement | boolean | Share hashed contact and sign-up events with ad partners. |
| product_analytics | boolean | Count which features are used; never the content. |
| training_on_chats | boolean | Allows anonymised chat content to contribute to training later models. Disabled by default. |
| cookies | { essential: true; optional: string[] } | Choices for this browser. |
| product_email | boolean | Product updates subscription. |
| sessions | { id; device_label: string; place: string; last_active: string; is_current: boolean }[] | Signed-in devices. |
| shared_chats | { id; chat_title: string; visibility: "workspace" \| "public"; shared_at: string; view_count: number; link: string }[] | Shared links. |
| is_loading | boolean | Secondary shapes; counts hidden. |
| is_empty | boolean | Nothing shared: only this device is listed; the shared-chats empty state offers a button to the chat list. |
| on_save | (privacy) => void | Called after any change, covering device sign-outs and link revocations. |
| on_sign_out_session | (id) => void | Signs out a device other than the current one. |
| on_sign_out_others | () => void | Signs out every other device. |
| on_stop_share | (id) => void | Stops a shared link. |
| on_copy_link | (id) => void | Copies a shared link. |
| on_export | () => void | Exports every chat and file as an archive. |
| on_delete_chats | () => void | Deletes all chats after confirmation. |
| on_navigate | (label: string) => void | Settings navigation. |

## Anatomy
Reading order:
1. Settings frame: Privacy marked current.
2. Page header: the title "Privacy", a one-line description.
3. Data and cookies card: three switch rows (ad measurement, product analytics, training on chats), each with a short note; a cookies row with a Manage control that expands a set of choices directly beneath it; a short line stating that data settings apply to all of the person's devices.
4. Emails card: a product-updates switch row with a note.
5. Active sessions card: a list of devices, each with its place and last-active line. The current device shows a text badge rather than a sign-out control. The card head holds the bulk action that ends every other session.
6. Shared chats card: filter tabs (In the workspace, Public links) with counts; rows showing title, shared time, view count, and copy-link and stop-sharing controls.
7. Danger zone card: an export row with its button, and a delete-all-chats row with its button.

## States
- **Loaded.** The default.
- **Loading.** Switches, the email row, sessions, and shared chats shown as secondary shapes; counts hidden.
- **Cookies expanded or collapsed.** Manage toggles its choices panel.
- **Only this device.** The sessions list holds only the current device.
- **Empty shared chats.** The filtered list shows a no-shares message or a no-public-links message, with a button that opens the chat list.
- **Delete confirmation.** A dialog asks once more before deleting.
- **Export running.** The export button shows a busy state until the archive is ready.

## Interactions
- Switch rows: click or Space toggles.
- Manage cookies: click or Enter toggles the choices.
- Sessions: Sign out on each non-current device; Sign out others as a bulk action.
- Shared-chats filter: a tab list with arrow keys; Enter selects.
- Copy link copies it and confirms; Stop sharing removes the link at once.
- Delete all chats: confirmation dialog; Escape cancels.
- Export starts the export and reports completion.

## Motion (functional)
- A removed session or shared link fades out, and the list closes the gap.
- The cookie choices panel unfolds in place.

## Accessibility
- Switches have the switch role with an aria-checked state.
- Manage cookies is a disclosure with an expanded state.
- The shared-chats filter is a tab list.
- Stop-sharing buttons are named after their chat.
- The current device is marked with a text badge, not a control.
- Deleting every chat requires a second confirmation.

## Responsive
- Below a narrow width, controls move beneath their labels; session and shared-chat rows keep their action beside them.
- The screen fills its container.

## Composition hints
- Settings frame.
- Switch rows with notes; disclosure panel.
- Session list with badge; shared-link list with counts and tabs.
- Danger zone card with destructive confirmation.

## Acceptance checks
1. Toggling training on chats calls `on_save` with that flag true and leaves the other flags unchanged.
2. Manage cookies expands its choices and sets its expanded state.
3. The current device shows a "This device" badge and no sign-out control.
4. Signing out another session calls `on_sign_out_session` with its id and removes it from the list.
5. Stopping a shared link calls `on_stop_share` and removes it from the list at once.
6. The shared-chats filter shows only links in the chosen scope, with matching counts.
7. Delete all chats opens a confirmation; Escape keeps the chats and calls nothing.
8. Confirming deletion calls `on_delete_chats` once.
9. Export calls `on_export` and shows a busy state until completion.
10. Below a narrow width, session and shared rows keep their action beside the title.
