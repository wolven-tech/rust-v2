# Agent Clash Map

Source item: affectUI Pro `components/agent-overlap-map` (functional reference only; see README).

## Purpose
Shows where the agents of several people step on each other: two agents touching the same files, changes that contradict, or the same piece of work done twice. Each agent is a chip, each open clash is a labelled badge between the agents it involves, and a curve joins the badge to each of those agents. Pointing at a clash or an agent brings its own curves forward and lets the rest recede. Opening a clash or an agent shows what it is tangled up in. People whose agents clash with nobody are listed as in sync.

## Data
- `participants`: `{ id, name, note? }[]`. `note` is one short line on what the person is working on; it shows in the agent card. A participant who appears in no clash is listed as in sync.
- `clashes`: `{ id, weight, label, summary, sides }[]`.
  - `weight`: `"decide" | "notice" | "faint"`. Decide needs a person to act now; notice is a heads-up; faint is background.
  - `label`: one or two words, shown on the badge (for example, a kind of collision such as shared files).
  - `summary`: one sentence, shown in the card.
  - `sides`: `{ participant_id, agent_name, harness_key? }[]`. Each side becomes one agent chip, so one person running two agents shows two chips.
- `harnesses`: map of `harness_key` to `{ display_name?, mark? }`. An agent without an entry shows only its name.
- `selected_clash_id?`, `selected_participant_id?`: when supplied, they control which card is open. When omitted, the map owns that state.
- `highlighted_clash_id?`: when supplied, controls the emphasised clash.
- `on_select_clash(id | null)`, `on_select_participant(id | null)`, `on_highlight_clash(id | null)`: fired on card open and close, and whenever pointer or focus moves over a clash.
- `min_height?`: the map is at least this tall and grows with the number of agents and clashes.
- `heading`: string, default "Agent clashes". Also the section's accessible name.
- `extra_class?`

Normalisation, applied before rendering:
- A side that names a participant not in `participants` is dropped.
- The same agent named twice in one clash counts once.
- A clash left with no sides is not shown.
- A participant's first name is the first word that contains a letter; leading symbols and emoji are skipped.

Demo values in any reference are invented for this spec; no affectUI data is reused.

## Anatomy
From top to bottom:
1. Header: the heading, a count of visible clashes, and a legend naming the three weights. The legend is omitted when there are no clashes.
2. Map: agent chips on the left and right edges, one clash badge per clash in the middle, and one curve for each clash-and-agent pair. The layout orders the chips so that curves cross as little as possible.
3. Agent chip: harness mark and agent name on the first line, person name beneath. An identity mark is drawn from the agent's key.
4. Clash badge: the label only. The summary is in the card.
5. In-sync line: a row listing participants with no clash, shown beneath the map.
6. Card, opened from a chip or a badge, placed next to what opened it:
   - For an agent: person name, agent name with its harness, the participant's `note`, then the list of that agent's clashes, each with its weight word, label and summary.
   - For a clash: weight word, label, summary, then each side as person name and agent name.

## States
- Idle: no pointer, focus or selection on the map; all curves at normal emphasis.
- Emphasised: a clash or an agent is pointed at or focused. Its curves are at full emphasis; every other curve fades back. No element changes size.
- Card open for an agent, or for a clash. Only one card is open at a time.
- Empty: no clashes remain after normalisation. The map shows the in-sync message in place of the map, and the legend is hidden.
- Weight, shown on the badge and in the card as a word, never as colour alone:
  - `decide`: the badge carries the strongest emphasis and breathes softly.
  - `notice`: standard emphasis.
  - `faint`: a dashed curve and a low-emphasis badge.
- Narrow layout (see Responsive).

## Interactions
- Pointer over a clash or an agent chip: emphasis as in States. Moving away restores idle.
- Click or activation of an agent chip or a clash badge: opens its card and calls the matching `on_select_*` callback.
- Click outside the card closes it.
- Escape: closes the card and returns focus to the chip or badge that opened it.
- Keyboard focus on a clash applies the same emphasis as pointing.
- Keyboard: Tab moves through the agent chips and clash badges in reading order (left edge, middle, right edge). Enter or Space on a chip or badge opens its card.
- Controlled props override internal state; a callback reports the change but does not itself change a controlled value until the parent updates it.

## Motion (functional)
- On first view the curves draw from each agent to its badge, and the badges settle in one after another.
- Emphasis changes fade the other curves; no size change.
- Agents involved in the emphasised clash show a light diagonal wave across their identity marks. Decide-weight badges breathe softly.
- A card fades in with a small downward settle.
- Reduced motion: all of the above is removed; states switch immediately.

## Accessibility
- Each agent chip and each clash badge is a button with an expanded state that reflects whether its card is open.
- An agent's accessible name combines person name, agent name, and the number of clashes it is in. A clash's accessible name combines its label and its summary.
- Curves and identity marks are decorative and hidden from assistive technology.
- Weight is conveyed by its word on the badge, in the legend and in the card.
- The focus indicator shows only for keyboard focus.
- The card is a labelled region; opening it moves focus only when it was opened from the keyboard.

## Responsive
- Fills its container's width. Chips sit at the left and right edges and badges in the middle.
- Below a narrow container width, all agent chips move to one edge, badges line up on the opposite edge, and each chip shows the person name and harness mark. The agent name appears in the card.
- The map grows with the count of agents and clashes. Long names and labels truncate with an ellipsis; full text is in the card.
- The card opens below its anchor if it fits, otherwise above it. It never extends beyond the map's width; long text inside it truncates.
- Right-to-left and other scripts render in their own direction.
- Dark theme is supported through an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Button (agent chip, clash badge), badge (weight word, label), an SVG layer for curves, popover or anchored card, legend, list (in-sync line), polite live region for selection changes, region for the heading.

## Acceptance checks
1. Pointing at a clash makes only its curves full-emphasis; no curve or badge changes size.
2. Each clash-and-agent pair draws exactly one curve, and an agent named twice in one clash draws one.
3. A side naming an unknown participant is dropped, and a clash with no remaining sides is not rendered.
4. A participant in no clash is listed as in sync.
5. Activating an agent chip or a clash badge opens its card and calls the matching `on_select_*` callback with that id.
6. Escape closes the card and returns focus to the element that opened it.
7. Every weight is identifiable from its word alone; the legend shows when clashes exist and is absent when none do.
8. With no clashes, the in-sync message appears and the legend does not.
9. Below the narrow width, agent chips share one edge and badges the other; the agent name is absent from the chip and present in the card.
10. With reduced motion requested, no curve draws, pulses, or waves run.

## Amendment: structure and demo

**Archetype: two-edge map.** Agent chips sit in a column on the left edge and a column on the right edge. Clash badges sit in a centre column. One curve joins each badge to each agent it involves. The legend sits in the header and the in-sync line sits beneath the map.

**Parts that must exist**
- Header: heading, clash count, and a legend naming the three weights.
- Map: left chip column, centre badge column, right chip column, and a curve layer behind them.
- Agent chip: harness mark and agent name on the first line, person name beneath.
- Clash badge: its label, with its weight word.
- In-sync line beneath the map.
- Card for an agent or a clash, placed beside what opened it.

**Gallery demo counts**
- 6 people: Ana, Mira, Dev, Sol, Lee and Yuki. Ana and Lee each run two agents.
- 7 agent chips: 4 on the left edge (Ana twice, Mira, Dev) and 3 on the right (Sol, Lee twice).
- 4 clashes: one decide-weight "Shared files" (Ana's first agent with Sol); two notice-weight, "Contradiction" (Mira with Lee's first agent) and "Same files" (Ana's second agent with Lee's second); one faint-weight "Same work" (Dev with Sol).
- 8 curves: one per clash-and-agent pair.
- Yuki has no clash.

**Visible at rest**
- All four badges, each with its label and weight word.
- Legend naming Decide, Notice and Faint.
- In-sync line naming Yuki.
- No chip emphasised and no card open.

**Acceptance checks (continued)**
11. The gallery map shows seven agent chips, four on the left edge and three on the right, four badges in the centre column and eight curves.
12. Each clash-and-agent pair draws one curve; Sol's single chip draws two curves, one into each of the two clashes it is in.
13. Yuki appears only in the in-sync line.
14. Each badge carries its weight word, and the legend lists Decide, Notice and Faint.
15. Below the narrow threshold all chips sit on one edge and all badges on the other, and no chip overlaps a badge at any width.

## Amendment 2: structure and demo

This amendment replaces the first amendment's Anatomy item 4, which gave the badge its label only: the badge carries its weight word as well, as the first amendment's States already require. It also replaces the gallery counts and checks 11 and 12, which become 16 to 18.

**Archetype: two chip columns and a centre column of badges.** Agent chips form a column on the left edge and another on the right. Between them, one badge per clash sits in a centre column. Each badge is joined to each of its two agents by one curve. The legend sits in the header; the in-sync line sits beneath the map.

**Parts that must exist**
- Header: heading, clash count, legend naming the three weights.
- Left chip column, centre badge column, right chip column, and a curve layer behind them.
- Agent chip, as in the first amendment.
- Clash badge: weight word and label.
- In-sync line beneath the map.
- Card for an agent or a clash, as before.

**Gallery demo counts** (these replace the first amendment's counts)
- Eight people: Ana, Mira, Dev, Ivo, Sol, Lee, Rui and Yuki. Ana and Lee each run two agents.
- Nine agent chips: five on the left edge (Ana's two agents, Mira, Dev, Ivo) and four on the right (Sol, Lee's two agents, Rui).
- Eight clashes, one per badge: two decide, four notice, two faint.
- Sixteen curves, two per badge. Rui's chip draws three, one per clash it is in.
- Yuki has no clash.

**Visible at rest**
- Eight badges, each with its weight word and label.
- Legend naming Decide, Notice and Faint.
- In-sync line naming Yuki only.
- No chip emphasised and no card open.

**Acceptance checks (continued; 11 and 12 withdrawn)**
16. The gallery shows nine agent chips, five on the left edge and four on the right, eight badges in the centre column and sixteen curves.
17. Each badge joins exactly one left chip and one right chip with two curves. A chip that belongs to several clashes draws one curve per clash.
18. No two badges sit level with each other in the centre column.
