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
