# Integration Arc Flow

Source item: affectUI Pro `illustrations/integration-flow` (functional reference only; see README).

## Purpose
A product scene that shows several connected tools feeding data into one central analysis hub. It tells a visitor that their tools' data gathers in one place.

## Data
- `hub`: { title, subtitle }.
- `integrations`: list of { id, label, sub_label, logo: node (supplied by caller), status: "live" | "paused" }.
- `active_id` (optional): the integration to highlight.
- `extra_classes`.

## Anatomy
In reading order (around the arc):
1. Hub card at the centre-bottom: the analysis title and subtitle.
2. Integration pills placed along a guide arc above the hub; each pill has its logo, a name and a source line.
3. Connectors: a guide path from each pill to the hub.
4. Pulse markers that travel along the connector toward the hub.
5. Background band behind the arc.

## States
- Live: pulses travel continuously from each live integration.
- Paused integration: its pill fades and its connector shows no pulses.
- Active integration: its pill is raised; its connector pulses faster in relative terms.
- Idle reduced: static arcs and pills with no pulses.

## Interactions
- Hover a pill: highlights it and its connector; other pills fade slightly.
- Click a pill: toggles its paused state, if `on_toggle` is provided by the caller; otherwise it highlights only.
- Keyboard focus: each pill is focusable in order; focus shows the same highlight as hover; Enter toggles if interactive.

## Motion (functional)
- Pills rotate along the arc slowly, so each integration passes the front position in turn.
- Pulses travel from each pill down its connector into the hub; the hub brightens briefly when a pulse arrives.
- A paused pill's motion eases to stop rather than snapping.
- With reduced motion preferred, pills stay in place and pulses are not drawn.

## Accessibility
- The scene is an image group labelled "Integrations feeding one analysis" unless a parent label is given.
- Pills in interactive mode are buttons with names "<label> integration, live" or "paused".
- Pulses are decorative.

## Responsive
- Scales with its container; on narrow widths the arc flattens and fewer pills show at once, with the rest reachable by rotation.
- Phone width: arc is replaced by a vertical stack of pills joined to the hub card.

## Composition hints
- Card (hub), pill or chip (integration), path or connector drawing, marker animation, button (interactive pills).

## Acceptance checks
1. Each live integration's connector shows pulses travelling toward the hub.
2. Hovering a pill highlights it and its connector.
3. A paused integration shows no pulses and its pill is faded.
4. With reduced motion preferred, no pulses are drawn and pills are static.
5. At phone width the arc is replaced by a vertical stack and nothing scrolls sideways.

Uses partner logos (supplied by caller) for each integration. Does not describe any brand mark.

## Amendment 5: structure and demo

Archetype: a vertical spine of integration pills beside a hub card, with one beam from the centre pill to the hub.

Replaces Anatomy item 1 (hub position) and item 2 (pills on a guide arc above the hub). The pills sit on one spine; the hub sits beside the centre pill.

Visible at rest:
- Hub card at the left, level with the centre pill: title and subtitle.
- Five integration pills on one near-vertical dashed guide to the right of the hub: two above the centre pill, the centre pill, and two below it.
- Each pill: a logo circle (caller-supplied), a kind line reading "Connector", and the integration name.
- Centre pill raised and at full strength. The other four are fainter, fading further from the centre.
- Beam from the centre pill to the hub.
- The outermost pills may run past the top and bottom edges of the stage and fade there.

Gallery demo:
- Hub: title "Analysis hub", subtitle "Every source in one view".
- Pills, top to bottom: "Helpdesk", "Warehouse", "Chat" (centre, active), "Tracker", "Payments".

Motion, settled frame: centre pill raised; beam drawn with one pulse in flight; hub lit.
Moves: pills step along the guide one slot at a time. The pill arriving at the centre becomes the active one. The centre pill charges, a beam sweeps to the hub, and the hub brightens on arrival. The background light drifts slowly.
Reduced motion: centre state held, beam drawn static, no pulses.

Acceptance checks (continued):
6. The five pills sit on one guide, two above and two below the centre pill, and the hub is level with the centre pill.
7. At rest exactly one pill is at full strength; the other four are visibly fainter.
8. The beam joins the centre pill to the hub with no gap.
9. Pill text is not clipped by the pill edge; a long name ends with an ellipsis.

## Amendment 6: structure and demo

Supersedes Amendment 5 where they differ. Replaces:
- "Visible at rest", beam bullet: no beam is drawn at rest. The beam appears only during the sweep.
- "Motion, settled frame" and "Reduced motion": no beam in either.
- Check 8: during the sweep the beam joins the centre pill to the hub with no gap.

Archetype: one hub card in the left third, beside one dashed spine of five integration pills.

Visible at rest:
- One hub card: title and subtitle, with a round mark at its right end. A subtitle that does not fit ends with an ellipsis.
- Five pills on one dashed guide that bows slightly. The guide runs down the left edge of the pills, behind their logo circles. It is not a solid connector.
- Each pill: logo circle (caller-supplied), kind line "Connector", integration name.
- Centre pill raised and at full strength; the others fainter with distance from the centre.
- Outer pills may run past the stage's top and bottom edges and fade there.
- No line between the hub and the centre pill. The hub and centre pill share a soft light.

Scale: the hub takes about a third of the stage width on the left. The pill column starts a little past the middle and takes about a third. The hub's centre is level with the centre pill's centre.

Gallery demo: as Amendment 5.

Motion, settled frame: centre pill raised; hub lit; no beam.
Moves: pills step along the guide one slot at a time. The pill arriving at the centre becomes active. The centre pill charges, a beam sweeps to the hub, and the hub brightens on arrival. The background light drifts.
Reduced motion: centre state held; no beam, no pulses.

Defects, stated as checks: two hubs and a solid vertical connector column (checks 10 and 11).

Acceptance checks (continued):
10. Exactly one hub is drawn, level with the centre pill's centre.
11. At rest no solid connector is drawn. The one dashed guide runs behind the pills' logo circles.
12. The pill column starts past the middle of the stage. The hub sits in the left third.

## Amendment 7: structure and demo

Replaces: Amendment 6 "Visible at rest" bullets 2 and 3 (the guide and the pill column), Amendment 6 "Scale" paragraph, and Amendment 6 check 12 wording "the hub sits in the left third" (now 8% to 43% of W). Amendment 6 checks 10 and 11 stand. Checks 13 to 17 are new.

Archetype: a left hub pill and a bowed column of integration pills on one dashed arc, on a full-bleed stage.

Stage: the card is the stage. No inner panel. The field fills the card edge to edge: a saturated field from a palette token, lightest at the top right, deepest at the middle left, fading to the neutral surface token across the bottom 15% of the left half.

Hub pill: the same pill shape as the integration pills, larger. Spans 8% to 43% of W. Vertical centre at 50% of H. Height 15% of H. Text is right-aligned: a title line (larger) above a subtitle line (smaller, muted). A round mark, about 75% of the pill height across, sits at the pill's right end. A subtitle that does not fit ends with an ellipsis.

Integration pills: five slots, vertical centres at 50% of H plus or minus 26.5% and plus or minus 53%. Slot 1 (top) is cut by the top edge and faint. Slot 5 (bottom) sits below the stage. Slots 2 to 4 are fully drawn.
- Each pill: 27% of W wide, 15% of H tall. Left edge at 56.7% of W for slot 3, 61% for slots 2 and 4, 71% for slot 1.
- Logo circle at the pill's left end, about 75% of pill height across, its centre 4% of W right of the pill's left edge.
- Text: first line (larger) is the kind, "Connector"; second line (smaller) is the integration name.
- Slot 3 (centre) is at full strength. Slots 2 and 4 at about half strength. Slot 1 at about a third.

Guide: one dashed line. Not solid. It passes through the logo circle centres of slots 2 to 4 and runs out of frame above slot 1. Its leftmost point is at slot 3, at 61% of W. At one slot away from slot 3 it sits about 4% of W further right; at two slots away about 11%.

Connectors: none. No line joins the hub to the centre pill. The hub and the centre pill share a soft light on the field between them, with no stroke.

Gallery demo: as Amendment 5 (hub title "Analysis hub", subtitle "Every source in one view"; pills top to bottom "Helpdesk", "Warehouse", "Chat", "Tracker", "Payments"; "Chat" is the centre pill and active).

Settled frame: centre pill raised at full strength; hub lit; no beam, no pulse. Moves and reduced motion: as Amendment 5, with the beam appearing only during the sweep.

Acceptance checks (continued):
13. The card is the stage: no inner panel, no inner border.
14. Exactly one hub, a pill with right-aligned text and a round mark at its right end, spanning 8% to 43% of W, vertical centre at 50% of H.
15. The five slot centres lie on one dashed arc: leftmost at slot 3's logo, about 4% of W right at one slot away, about 11% right at two slots away. No solid line at rest.
16. The hub's vertical centre equals the centre pill's. Only the centre pill is at full strength.
17. In each pill the kind line is larger than the name line. A line that does not fit ends with an ellipsis, never clipped at the pill edge.
