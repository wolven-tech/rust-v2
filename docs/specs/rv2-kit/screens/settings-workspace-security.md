# Settings: Workspace Security

Source item: affectUI Pro `screens/settings-security` (functional reference only; see README).

## Purpose
A workspace administrator decides who may sign in. They choose which email domains the organisation owns, whether people join by domain, whether single sign-on and two-step verification are required, how long sessions last, the identity-provider values to copy into the provider, which network ranges may connect, and an emergency action that signs everyone out.

## Data
| Field | Type | Meaning |
|---|---|---|
| email_domains | { name: string; verified: boolean; txt_record: string }[] | Email domains. Unverified ones show the TXT record to add. |
| join_by_domain | boolean | Members with verified addresses join without an invite. |
| require_sso | boolean | Members at verified domains sign in through the provider. |
| two_step_required | boolean | Members who do not sign in with SSO must supply a one-time code from an authenticator app. |
| session_length | "1d" \| "7d" \| "30d" \| "never" | Idle session lifetime. |
| idp | { audience_uri; acs_url; entity_id; metadata_url: string } | Values to copy; the metadata URL is typed in by the admin. |
| ip_ranges | { id; range: string; label?: string }[] | Allowed networks. With none, all networks may connect. |
| is_loading | boolean | Secondary shapes in place of each list and switch. |
| is_empty | boolean | No domains and no ranges: their empty states appear, and both switches stay off. |
| on_verify_domain | (domain) => Promise<boolean> | Checks the TXT record; resolves true once it is found. |
| on_save | (security) => void | Called on any change; the metadata URL is saved on blur. |
| on_sign_out_all | () => void | Ends every session in the workspace after confirmation. |
| on_navigate | (label: string) => void | Settings navigation. |

## Anatomy
Reading order:
1. Settings frame: Security marked current.
2. Page header: the title "Security", a one-line description.
3. Email domains card: an Add domain control; rows showing the domain name, a status badge (Verified or Pending) and, while pending, a Verify control; beneath a pending row, the TXT record line to copy.
4. Sign-in card: rows for join by domain, require single sign-on, and two-step verification, each with a note. The SSO switch is disabled, with its reason, while no domain is verified. Also the session length choice.
5. Identity provider card: four copy rows (audience URI, SP entity ID, ACS URL, and a metadata URL field). Each copy control briefly turns into a check.
6. IP allowlist card: an explanation, an Add range control, and rows of ranges with remove controls. An empty-state line says every network can connect until a range is added.
7. Danger zone card: a sign-out-everyone row and its button.

## States
- **Loaded.** The default.
- **Loading.** Lists and switches shown as secondary shapes.
- **Domain pending.** The row shows Pending, a Verify control, and the TXT record line.
- **Verifying.** Verify shows a busy state until the check resolves; a failure keeps the domain pending and shows a toast.
- **SSO unavailable.** No verified domain: the SSO switch is disabled and says why.
- **Empty.** A no-domains message with an Add domain control; a no-ranges message with an Add range control; SSO and join by domain stay off.
- **Confirm sign-out.** A dialog asks once; confirming ends every session.

## Interactions
- Add domain and Add range: a small form; Enter adds, Escape cancels; invalid input shows a toast.
- Verify calls `on_verify_domain`.
- Switches toggle on click or Space; disabled ones announce why.
- Session length: a menu of choices.
- A copy control places its value on the clipboard and briefly shows a confirmation mark. Read-only fields select all their text when focused.
- Remove range and remove domain: their buttons appear on hover and on keyboard focus, and are always visible on touch screens.
- The metadata URL saves on blur.
- Sign out everyone opens a dialog; Escape cancels.

## Motion (functional)
- A verified status changes in place.
- A copy control's check appears and then returns to the copy icon.
- Removed rows close the gap beneath them.

## Accessibility
- Switches have the switch role with aria-checked; disabled switches say why.
- Copy buttons confirm with text as well as an icon.
- Remove buttons are reachable by keyboard focus, not only by pointer.
- The sign-out dialog traps focus and returns it to the trigger.

## Responsive
- Below a narrow width, each control moves beneath its label; switches and domain rows stay on one line.
- The screen fills its container.

## Composition hints
- Settings frame.
- Settings rows with switches; list of domains with status badges and inline verify.
- Read-only copy fields with copy buttons.
- Chip-like rows for IP ranges with remove.
- Danger zone card with confirmation dialog.

## Acceptance checks
1. Joining by domain cannot be turned on while no domain is verified; the switch is disabled and states the reason.
2. Verify on a pending domain calls `on_verify_domain`; a true result changes its badge to Verified.
3. A false result keeps the domain pending and shows a toast.
4. Copying the ACS URL puts it on the clipboard and shows a check briefly.
5. Adding an IP range with Enter appends it to the list and calls `on_save`.
6. With no IP ranges, the card states that every network can connect.
7. Sign out everyone opens a dialog; Escape keeps all sessions and calls nothing.
8. Confirming sign-out calls `on_sign_out_all` once.
9. Changing session length to 7 days calls `on_save` with that value.
10. Remove controls are reachable by keyboard focus, not only by hover.
