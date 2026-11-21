# Connector Settings Card

Source item: affectUI Pro `components/integration-settings` (functional reference only; see README).

## Purpose
Configuration panel for a single linked service (a shop, an advertising account, a data feed). It toggles the service, stores its access key, a checked site address and a time zone. Changes wait in a buffer until saved or discarded.

## Data
- `service_name` (text).
- `service_mark` (node): small mark in the header tile.
- `item_count` (number, optional): small number beside the name.
- `subtitle_text` (text): line under the name. When the service is switched off, it shows a paused note instead.
- `initial_values` (partial): `is_enabled`, `site_url`, `time_zone`, `api_key`.
- `site_url_hint` (text): helper text under the address field.
- `time_zone_options` (text[]).
- `on_generate_key()` (returns text or a promise): creates a key. Without it, the card generates a random demo key.
- `on_save(values)`: receives every field's value when the save control is used.
- `on_enabled_change(is_enabled)`.

## Anatomy
Reading order:
1. Header: service mark, name, count, subtitle line, and the enable switch.
2. Connection detail: the connected-since line (when connected).
3. API key field: masked value, with actions to reveal (spec choice) and to generate a new key. A new key shows a reminder to copy it.
4. Site address field: text input with a helper line; shows an invalid state with a message.
5. Time zone picker: a select listing `time_zone_options`.
6. Save row at the bottom: "Unsaved changes" text, Discard and Save. Visible only when values differ from the saved state.

## States
- Enabled: all settings visible and editable.
- Disabled (switched off): settings fold away; the subtitle shows the paused note.
- Clean: save row hidden.
- Dirty: save row visible with "Unsaved changes".
- Saving: Save shows a busy state until `on_save` resolves.
- Invalid address: the field shows its error and points to the helper text; Save is blocked.
- Key generated: a notice that the new key must be copied now.

## Interactions
Pointer:
- Click the switch: toggles enabled; calls `on_enabled_change`.
- Edit any field: marks the card dirty.
- Generate key: calls `on_generate_key`; replaces the key value.
- Discard: restores the saved values and hides the save row.
- Save: calls `on_save` with all values.
- Time zone: opens its picker; choosing one sets the value.

Keyboard:
- Space on the switch: toggles it.
- Tab: moves through the key actions, the address field and the time zone picker.
- Escape: closes the time zone picker.

## Motion (functional)
- Switching off folds the settings away.
- Once a value differs, the save bar rises from the card's lower edge.
- Reduced motion: these changes are instant.

## Accessibility
- The enable control is a real switch with a name.
- The address field reports invalid input and links the error to its helper text.
- A freshly minted key is read out along with the advice to copy it immediately.

## Responsive
- Container-based; the card fills its column. Fields stack when the container is narrow.

## Composition hints
Panel, header with tile and switch, text field with copy/reveal/generate actions, text input with validation, select (time zone), button pair (discard, save), notice, collapsible section.

## Acceptance checks
1. Switching the card off calls `on_enabled_change(false)`, folds the settings and shows the paused note in the subtitle.
2. Editing the address field shows the save row with "Unsaved changes".
3. Discard restores the last saved address and hides the save row.
4. Save calls `on_save` with an object containing `is_enabled`, `site_url`, `time_zone` and `api_key`.
5. An address without a valid scheme or host shows the error, and Save does not call `on_save`.
6. Generate key replaces the key value and shows the copy reminder.
7. Escape while the time zone picker is open closes it and keeps the selected value.
8. Space on the focused switch toggles it.
9. While `on_save` is pending, Save shows a busy state and cannot be pressed twice.
10. Reduced motion: the save row appears without sliding.
</content>
