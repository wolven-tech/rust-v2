# Continent Highlight Map

Source item: affectUI Pro `illustrations/world-map` (functional reference only; see README).

## Purpose
A world map with one continent picked out, to show where a business operates or where a region's data comes from. The person switches the highlighted region with a control.

## Data
- `regions`: list of { id, label, geometry_ref }. Six regions: four continents plus two groupings the caller chooses. `geometry_ref` points to the caller's own land shapes; the component does not ship them.
- `selected_region_id`, `on_region_change(id)`, `default_region_id`.
- `band_style`: "coastal" | "none" (the band follows the coastline of the highlighted region).
- `extra_classes`.

## Anatomy
In reading order:
1. Base map: all land in one shared base fill.
2. Highlight: the selected region's land in an emphasis fill, with a band following its coastline.
3. Region switch: a control at the top left naming the current region, which opens a list of the six regions.
4. Optional legend (not shown by default).

## States
- Default: first region highlighted.
- Switching: the highlight fades out of the old region and into the new one; the band follows.
- Hover on another region: a faint outline shows which land would be highlighted.
- Disabled: switch is inert; highlight stays.

## Interactions
- Click the region switch: opens the list; choosing one highlights it.
- Click land on the map: highlights that land's region if it belongs to one.
- Keyboard: the switch is a menu button; Enter or Space opens the list; Arrow keys move; Enter picks; Escape closes without change.
- Hover on land: outlines the region under the pointer.

## Motion (functional)
- Changing region fades the old highlight and draws the new highlight in, so the eye follows the change.
- The coastal band slides along the new coastline as the highlight moves.
- With reduced motion preferred, the highlight changes in one step.

## Accessibility
- The map is an image with a name that includes the selected region.
- The region switch is a menu button with the current region as its value.
- The highlighted region is named in text; a visual cue alone does not indicate it.

## Responsive
- Scales to its container; the switch stays above the map.
- Phone width: map fills width; the switch remains at top left and the list opens as a sheet.

## Composition hints
- Map image or vector drawing, menu button, listbox, overlay band.

## Acceptance checks
1. Choosing a region in the switch highlights exactly that region's land.
2. The selected region's name is shown on the switch and in the image's accessible name.
3. Escape closes the list without changing the highlight.
4. Hovering a non-selected region shows an outline without changing the selection.
5. With reduced motion preferred, changing region applies in one step.
6. At phone width the map fills the width and nothing scrolls sideways.

Uses no partner logos.
