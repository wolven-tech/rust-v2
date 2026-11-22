# Access Grant Prompt

Source item: affectUI Pro `components/permission-request` (functional reference only; see README).

## Purpose
Asks a person to decide whether an agent may carry out one particular action. It shows who is asking, what will change, and how long the permission would remain in force. After a decision it folds into one line that reports the time left and offers a way to revoke.

## Data
| Field | Type | Meaning |
|---|---|---|
| `requester` | `{ name, model_name?, avatar? }` | The agent making the request. Avatar falls back to initials. |
| `requested_action` | string | What the agent wants to do, phrased as a short verb phrase. |
| `target_label` | string, optional | Where the action applies, such as a repository or account name. |
| `target_mark` | ReactNode, optional | Visual mark for the target. |
| `risk_level` | `"safe" \| "risky" \| "destructive"` | Defaults to `"risky"`. |
| `agent_rationale` | string, optional | The agent's own account of why it needs this. |
| `impact_items` | `ImpactItem[]` | Concrete things that will be affected. Each has `item_id`, `item_label`, `item_meta?`, `item_mark?`. |
| `impact_heading` | string | Heading above the impact list. Defaults to "What will happen". |
| `impact_visible_count` | number | Rows displayed before a "more" line. Defaults to 3. |
| `allowed_windows` | `("once" \| "timed" \| "always")[]` | Durations on offer. Defaults to all three. A single entry removes the choice entirely. |
| `window_choice` / `default_window` | `PermissionWindow` | Controlled or uncontrolled duration selection. Defaults to `"once"`. |
| `timed_minutes` | number | Length of a timed grant. Defaults to 90. |
| `grant_outcome` / `default_outcome` | `{ allowed, window, decided_at, valid_until? } \| null` | Controlled or uncontrolled decision. Null while awaiting an answer. |
| `on_decision_change` | (decision or null) => void | Called when the person allows or denies; called with null on revoke, change, or ask-again. |
| `current_time` | number | Current time as an epoch value, supplied by the host so remaining time can be checked. |

## Anatomy
Waiting state, reading order:
1. Title line: "<requester> wants to <action>", with a risk tag when the risk is risky or destructive.
2. Target line with its mark.
3. The agent's rationale, indented beneath the title.
4. Impact section: its heading, the impact rows with right-aligned meta, then an "N more" line when rows are hidden.
5. Duration choice, shown only when more than one window is offered.
6. Action row: Deny, followed by the allow button. The allow label names the selected duration.
7. Keyboard hints, displayed only when a hardware keyboard is in use.

Decided state, one line: outcome marker, outcome text (the granted duration or "denied"), time remaining for timed grants, and a Revoke control.

## States
- **Waiting:** full card. Duration choice defaults to once.
- **Allowed once:** collapses to the decided line; no time remaining is shown; Revoke is available.
- **Allowed for a timed window:** the collapsed line shows the time remaining and a slim bar that drains as time passes.
- **Allowed always:** collapsed line with no time remaining.
- **Denied:** collapsed line stating the denial; a "Change" control reopens the choice.
- **Expired:** once a timed grant runs out, the line reads "expired" and the host receives null.
- **Destructive risk:** the "always" option is withheld even when `allowed_windows` lists it. The allow control takes a warning appearance.
- **Reset by host:** when the host sets `grant_outcome` back to null, the card returns to waiting.

Transitions: waiting to decided on allow or deny; decided back to waiting on revoke, change, or ask-again (each calls `on_decision_change(null)`); a timed grant moves to expired when `current_time` passes `valid_until`.

## Interactions
- **Pointer:** select a duration through its option; press the allow control to grant with the chosen duration; press deny to refuse.
- **Touch:** same as pointer, with larger targets and no keyboard hints.
- **Keyboard:** Enter grants with the chosen duration while focus is inside the card. Escape refuses. Left and right arrows change the duration while the duration group holds focus.
- **Focus order:** duration group, then deny, then allow.
- **Outside click:** no effect; a decision requires an explicit choice.
- **Revoke:** calls `on_decision_change(null)` and restores the waiting layout.

## Motion (functional)
- Impact rows appear one after another so the person reads them in sequence.
- When the duration changes, the allow label swaps with a brief crossfade.
- On a decision the card folds to its one-line form, and an outcome marker appears in place.
- A timed grant shows a countdown bar that empties over time.
- Reduced motion: every change happens at once.

## Accessibility
- While waiting, the card is a dialog named by its title and described by its rationale.
- The duration choice is a radio group. The allow control's accessible name includes the duration, for example "Allow for one hour".
- Once decided, the outcome line is a status region so the result is announced.
- Focus moves into the card when it appears and remains there while it waits.
- Risk is conveyed through a word, not through emphasis alone.

## Responsive
- Fills its container up to a comfortable reading width.
- When there is little horizontal room, the rationale loses its indent, the action buttons share the full width, and keyboard hints are hidden.
- On touch-first devices, choices and buttons are larger and keyboard hints are omitted.

## Composition hints
- Card or dialog as the container.
- Avatar (initials fallback) for the requester.
- Badge for risk.
- Radio group for the duration choice.
- Button pair for deny and allow.
- List for impact rows, with a trailing meta column.
- Status line with an icon for the collapsed state.
- Progress bar for the countdown.

## Acceptance checks
1. A waiting card shows the requester, action, target, rationale and impact rows, and the allow label names the default duration.
2. With `allowed_windows` set to only "once", no duration choice appears.
3. For a destructive action, "always" is absent from the duration choice even when listed in `allowed_windows`.
4. Pressing Enter inside the waiting card calls `on_decision_change` with an allow decision carrying the chosen window.
5. Pressing Escape calls `on_decision_change` with a deny decision.
6. After a timed allow, the collapsed line shows the time left, computed from `valid_until` and `current_time`; once `current_time` passes `valid_until`, the line reads expired and `on_decision_change` is called with null.
7. Revoke calls `on_decision_change(null)` and the card returns to the waiting layout.
8. The impact list shows at most `impact_visible_count` rows and a "more" line with the hidden count.
9. When there is little horizontal room, the keyboard hint text is absent from the DOM or hidden from view.
</content>
