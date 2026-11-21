# Consent Request Card

Source item: affectUI Pro `components/approval-card` (functional reference only; see README).

## Purpose
Asks a person to allow one specific action an agent is about to take. It shows what will happen and how risky it is, and offers allow or deny. It can also expire on its own when nobody answers.

## Data
| Field | Type | Meaning |
|---|---|---|
| `request_title` | string | Imperative phrase naming the action. Also serves as the card's accessible name. |
| `context_line` | string | One line of context under the title; truncates. |
| `risk_level` | `"low" \| "medium" \| "high"` | Optional. Tints a badge in the header. Omit to hide the badge. |
| `header_mark` | node | Optional mark in the header tile. Defaults to a neutral shield glyph. |
| `detail_rows` | `{ key_label, value }[]` | Key-value rows in the detail panel. |
| `preview_content` | node | Custom content above the detail rows (recipient list, diff, and so on). Falls back to `body_content`. |
| `body_content` | node | Used as the preview when `preview_content` is not set. |
| `decision_state` | `"pending" \| "granted" \| "declined"` | Controlled state. Setting it back to `pending` starts a new request. |
| `initial_decision` | same | Uncontrolled starting state. Defaults to `pending`. |
| `on_grant({ keep_for_action })` | callback, may return a promise | Called on grant or on the keyboard shortcut. If a promise is returned, the card shows progress and locks until it settles; a rejection returns the card to pending. |
| `on_decline({ cause })` | callback | `cause` is `user` or `timeout`. |
| `on_amend()` | callback | Optional. When present, an Edit control appears before Deny. |
| `timeout_seconds` | number | Optional. Length of the countdown. |
| `on_timeout()` | callback | Fired when the countdown ends, immediately before `on_decline({ cause: "timeout" })`. |
| `show_remember_option` | boolean | Shows a "keep for this action type" checkbox. Defaults to true. |
| `text_overrides` | object | Overrides the text for grant, decline, edit, remember, and the group label. Defaults are English strings. |
| `show_key_hints` | boolean | Shows shortcut hints on buttons. Shortcuts work regardless. |
| `band_label` | node | Small label on a secondary band above the card, for example a step identifier. |
| `control_layout` | `"inline" \| "footer"` | Places the buttons in the last card row, or on a secondary band beneath the card. |
| `root_class` | string | Extra class applied to the root. |

## Anatomy
Reading order:
1. Secondary band with `band_label` (optional).
2. Header: header mark, request title, risk badge (optional).
3. Context line.
4. Preview region (custom node or body content).
5. Detail rows.
6. Footer row: remember checkbox (left), countdown text (when set), then Edit, Deny and Allow (right).
7. Resolved row, which replaces the body after a decision: outcome word, mark, and time of the decision.

## States
- **Pending:** all controls active. The countdown ticks if `timeout_seconds` is set.
- **Pending, nearing expiry:** the countdown text switches to a warning tint during a configurable final stretch of the window.
- **Pending, hovered or focused:** the countdown pauses.
- **Processing:** after grant returns a promise, the card is busy; buttons are disabled in place (not removed) so focus is kept.
- **Granted / declined:** the body collapses to a compact resolved row showing the outcome word and time.
- **Expired:** treated as declined with cause `timeout`; the same resolved row with "Expired" wording.
- **Risk levels:** low shows no tint; medium uses a warning tint; high uses a danger tint. A text label is always present.

## Interactions
- Pointer: Allow, Deny, Edit and the remember checkbox are standard buttons and a checkbox.
- Mod+Enter (Ctrl+Enter off macOS): grants while focus is anywhere inside the card.
- Escape: declines while focus is inside the card.
- Tab: remember checkbox, Edit, Deny, Allow.
- Pausing: hover or focus on the card pauses the countdown; leaving resumes from the paused value.
- Touch: shortcut hints are hidden; all controls remain tappable.

## Motion (functional)
- On a decision, the body fades out and the card height glides down to the resolved row; the outcome mark draws in.
- The countdown text updates once per second without a transition.
- Reduced motion: decision changes are immediate; only a progress indicator keeps moving.

## Accessibility
- The card is a named group. Its name is the request title; its description is the context line plus a sentence describing the countdown.
- The countdown is announced once as a full sentence, not on every tick. The visible clock is hidden from assistive technology.
- Decisions are announced in a polite live region, for example "Granted: <request title>".
- During processing, the busy state is set and controls are marked disabled.
- Once decided, focus relocates to the collapsed summary when it had been inside the card.

## Responsive
- Fills its container.
- When the card is narrow, footer buttons share a full-width row and shortcut hints are hidden.
- Long titles, context lines and detail values truncate, with the full text in a tooltip.
- Works at phone width without horizontal scrolling.

## Composition hints
Card, badge (risk), icon tile, button, checkbox, progress indicator, definition list (detail rows), live region helper, timer text.

## Acceptance checks
1. With `timeout_seconds` set and no interaction, `on_decline` is called once with `cause: "timeout"` after the countdown, and `on_timeout` fires just before it.
2. Hovering the card while the countdown runs freezes the displayed remaining time; leaving resumes from the frozen value.
3. Mod+Enter with focus on the detail rows calls `on_grant` once.
4. Escape with focus inside the card calls `on_decline` with `cause: "user"`.
5. An `on_grant` that returns a promise keeps the buttons disabled until the promise settles; a rejected promise returns the card to pending with buttons enabled.
6. Setting `decision_state` to `granted` from outside collapses the card to a resolved row without calling `on_grant`.
7. Setting `decision_state` back to `pending` after a resolution restores the full card.
8. With `show_remember_option` true and the box checked, `on_grant` receives `keep_for_action: true`.
9. After a grant, focus moves to the resolved row when focus was previously inside the card.
10. At phone width the risk badge and request title remain visible, with no horizontal scrolling.
</content>
