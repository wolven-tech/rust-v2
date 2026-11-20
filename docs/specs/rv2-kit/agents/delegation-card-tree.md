# Delegation Card Tree

Source item: affectUI Pro `components/agent-crew` (functional reference only; see README).

## Purpose
Displays a lead agent and the sub-agents it hands work to as one live tree of cards. Each agent remains on screen whether it is busy or resting. A card reports what its agent is doing now, how full its context is, which tools it holds, and what it most recently received. Hand-offs between agents are animated along the connectors.

## Data
- `top_agent`: an agent node. Children nest recursively.
- Agent node: `{ id, name, state: "active" | "idle" | "paused" | "waiting" | "done" | "failed", doing_text?, load_ratio? (0 to 1), load_capacity_tokens?, load_used_tokens?, model_label?, model_mark?, tool_names?: { mark?, name }[], last_transfer?: { from_name, text }, assignment?, started_at?, ended_at?, spend?, tokens?, identity_mark?, sub_agents?: Agent[] }`.
- `transfers`: `{ id, source_id, target_id, note? }[]`. A new `id` plays one hand-off animation; ids that exist at mount time play nothing.
- `heading`: string, the header name.
- `workspaces`: `{ id, name, active_count?, attention_count? }[]`; `active_workspace`: an id; `on_workspace_change(id)`.
- `on_open(agent)`: fires when a card is activated (click or Enter), in addition to opening its details.
- `on_command(agent, action, message?)`: actions are `message`, `pause`, `stop`, `resume`, `approve`, `decline`, and `retry`. Each is offered only when it is valid for the current state.
- `format_spend(amount) => string`. Defaults to dollars; a value below one cent reads as "less than one cent".
- `extra_class?`

Demo values in any reference are invented for this spec; no affectUI data is reused.

## Anatomy
From top to bottom:
1. Header: a workspace switcher (or a static heading), with counts of agents working and agents needing the user.
2. Tree area: the lead card at the top; its direct sub-agents side by side beneath it, each with its own children hanging below as smaller cards. Connectors join each parent to its children.
3. Card (one per agent), read in this order:
   - identity mark and name
   - state word and model label
   - activity line (one line: what the agent is doing now, or what it last did while resting)
   - load bar with a percentage and a token figure, for example `44k / 200k`
   - tools as small tiles: the first few, plus `+N` for the rest
   - last transfer received: sender and text
4. Details panel (opened from a card): placed next to the card and as wide as it. Shows the assignment, the model, usage (elapsed time, spend, tokens), the tools by name, the last transfer, and command actions.
5. Message field: appears inside the details when the message action is chosen.

Layout: wide containers place sub-agents side by side; narrow containers indent each level instead.

## States
- `active`: the identity mark animates; the card's line shimmers; the activity line says what it is doing.
- `idle`: resting; the activity line reads as waiting.
- `paused`: resting, paused by the user; command actions offer Resume.
- `waiting`: blocked on the user; the card uses the attention tone; actions offer Approve and Decline.
- `done`: finished; resting.
- `failed`: the card uses the critical tone; actions offer Retry.
- Load above 80% shows a warning tone; above 95% shows a critical tone.
- Details open or closed.
- Message field open or closed.
- Workspace list open or closed.

Transitions come from the data: a changed `state` changes the card, and a new `transfers` id triggers the hand-off animation.

## Interactions
- Clicking a card opens its details and calls `on_open`.
- Clicking a command action calls `on_command` with that action. Choosing message opens the message field.
- Clicking the workspace switcher opens the list; choosing an entry sets `active_workspace`.
- Keyboard: Tab enters the tree as a single stop, then continues to the workspace switcher. Arrow keys move between cards. Home goes to the lead card; End goes to the last card.
- Enter or Space on a card toggles its details.
- Escape closes the message field first, then the details, and focus returns to the card.
- Workspace switcher: Up, Down, or Enter selects; Escape closes the list.
- A click outside closes the details (inferred from the popover pattern; the clean room should verify it).
- Touch: buttons, the close control, and list rows have larger hit areas.

## Motion (functional)
- A hand-off travels as a short highlight along the connectors from sender to receiver, passing through any intermediate agents and fading at both ends.
- The receiving card gets an emphasis that rises from the edge where the highlight arrived, then fades.
- Active cards send a wave through their identity mark and shimmer their line. Load bars fill smoothly when their values change.
- When a new workspace is shown, its cards enter and its connectors draw in.
- The details panel appears with a brief scale-in from its card.
- Reduced motion: all of the above is removed; hand-offs only update the cards.

## Accessibility
- The tree structure is exposed; each card has one label combining name, state, activity, and load, and indicates that it opens a dialog.
- Details form a dialog named after the agent. Close, command actions, and the message field are reachable with Tab.
- Workspace switcher: a button that controls a listbox, with the active workspace selected.
- A polite live region announces each hand-off and each command action, for example "Lead handed to Coder: Guard empty carts".
- The focus indicator shows only for keyboard focus.
- The message field shows focus through its border.

## Responsive
- Each card has a fixed layout: changes to activity or load do not move cards or connectors.
- Long names, activity lines, and messages are truncated with an ellipsis and show their full text on hover.
- Details float above the content and are as wide as their card; they never alter page layout.
- With more than four tools, three tiles plus `+N` are shown.
- Invalid numbers from upstream are clamped (load to the 0 to 100% range); a missing spend is omitted; a start time in the future reads as zero elapsed.
- Narrow screens: indented levels, and the workspace list uses the available width.
- Light and dark themes are supported through an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Card (per agent), tree with connector drawing, progress bar (load), badge (state), text button group (commands), dialog or popover (details), listbox (workspace switcher), polite live region, shimmer text, tile chips (tools), a formatter prop for spend.

## Acceptance checks
1. Changing an agent's `state` updates its state word and visual state without moving other cards.
2. Adding a new `transfers` id plays one hand-off animation; re-rendering with the same ids plays nothing.
3. Clicking a card opens its details and calls `on_open` with that agent.
4. The details show assignment, model, elapsed time, spend, tokens, and tools.
5. Command actions offered match the current state (Approve and Decline only when `waiting`).
6. Arrow keys move focus between cards; Home and End jump to the lead card and the last card; Tab leaves the tree.
7. Escape closes the message field first, then the details, and returns focus to the card.
8. Each hand-off produces one polite live announcement.
9. Load above 95% shows the critical tone; values above 100% are clamped to 100%.
10. With reduced motion on, no hand-off or pulse animation runs.
