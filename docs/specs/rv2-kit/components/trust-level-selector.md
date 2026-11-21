# Trust Level Selector

Source item: affectUI Pro `components/autonomy-dial` (functional reference only; see README).

## Purpose
Lets a person set how much an agent may do without asking. Three levels, plus per-capability pins that force a specific capability to always ask. The list shows, capability by capability, what the chosen level means in practice.

## Data
| Field | Type | Meaning |
|---|---|---|
| `capabilities` | Capability[] | `{ id, capability_name, capability_note?, risk_class: "routine" \| "sensitive" \| "irreversible", capability_mark? }` |
| `policy_level` | `0 \| 1 \| 2` | Controlled level. 0 = ask for everything; 1 = run routine capabilities, ask for the rest; 2 = run everything not pinned |
| `initial_policy_level` | same | Uncontrolled start. Default 1 |
| `on_policy_change(policy_level)` | callback | A level was chosen |
| `always_ask_ids` | string[] | Controlled ids of capabilities that always ask, regardless of level |
| `initial_always_ask` | string[] | Uncontrolled start. Default empty |
| `on_always_ask_change(always_ask_ids)` | callback | A pin was added or removed |
| `section_title` | string | Default: "Agent limits" |
| `section_subtitle` | string | Default: "Independent actions the agent may take" |
| `root_class` | string | Extra class on root |

Export a pure function `needs_confirmation(capability, policy_level, always_ask_ids)` with the same rule the component uses, so the agent runtime and the UI cannot disagree.

Rule: a pinned capability always asks. Level 0 asks for all. Level 1 runs only `routine` capabilities. Level 2 runs all unpinned capabilities.

## Anatomy
Reading order:
1. Title and subtitle. Subtitle is the agent's name or purpose line.
2. Level control: three options in a row, each with a short name, with a sentence below describing the chosen one.
3. Two groups: "Runs without confirmation" (capabilities that will run without asking at the current level) and "Needs confirmation" (the rest).
4. Each capability row: name, optional note line, risk tag (only for sensitive and irreversible), pin toggle.
5. Footer count line: "N run without confirmation, M need confirmation".

## States
- **Level 0**: all capabilities in "Needs confirmation".
- **Level 1**: routine capabilities in "Runs without confirmation"; sensitive and irreversible in "Needs confirmation".
- **Level 2**: all unpinned capabilities in "Runs without confirmation".
- **Pinned capability**: pin on; capability stays in "Needs confirmation" at every level.
- **Pinned and routine at level 2**: still asks; the row shows the pin as the reason.
- **No capabilities**: groups are hidden and a short empty line is shown.

## Interactions
- Pointer: click a level to select it; click a pin to toggle it.
- Left/Right arrows (also Up/Down) on the level control: previous or next level, wraps at the ends.
- Tab: from the level control to the pin toggles in list order.
- Space on a focused pin toggle: sets or clears the pin.
- Touch: pins are always visible (no hover reveal).

## Motion (functional)
- When the level or a pin changes, capabilities move between groups; moves are staggered so the reader can follow each one.
- The chosen level indicator moves smoothly between options.
- Counts update with a short roll.
- Reduced motion: moves are immediate.

## Accessibility
- Level control is a radio group labelled "Independence level", with a description of the chosen level.
- Each group is a labelled section.
- Each pin is a toggle button with a pressed state and a name that includes the capability name.
- Changes are announced politely: "Level: <name>. N run without confirmation, M need confirmation."
- Risk is conveyed by text in the tag, not colour alone.

## Responsive
- Fills its container up to a maximum width.
- Below a narrow width, note lines are hidden and level options get tighter.
- Touch targets are enlarged on touch devices.
- No horizontal scroll at phone width.

## Composition hints
Card (container), radio group (levels), list with section headings (groups), toggle button (pins), tag (risk), live region helper.

## Acceptance checks
1. At level 0, every capability appears under "Needs confirmation" and the footer reads "0 run without confirmation".
2. At level 1, exactly the `routine` capabilities appear under "Runs without confirmation".
3. At level 2, every unpinned capability appears under "Runs without confirmation", including irreversible ones.
4. Pinning a capability at level 2 moves it to "Needs confirmation" and `on_always_ask_change` is called with its id.
5. The exported `needs_confirmation` returns the same value as the on-screen group for every (capability, level, pinned) combination in a table test.
6. Arrow keys on the level control change the selected level and call `on_policy_change` once per change, wrapping at both ends.
7. Space on a focused pin toggles it and reflects the change in its pressed state.
8. Risk tags appear only on sensitive and irreversible capabilities, and carry their text.
9. At phone width there is no horizontal scroll.
10. Level change is announced through a polite live region with the counts.
