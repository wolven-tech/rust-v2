# Fill-in Prompt Templates

Source item: affectUI Pro `components/prompt-library` (functional reference only; see README).

## Purpose
Holds a set of saved prompt templates, each containing named blanks. A person selects a template, fills the blanks inside the sentence itself, watches the finished prompt take shape, and uses or copies it once every blank has a value.

## Data
| Field | Type | Meaning |
|---|---|---|
| `template_list` | `Template[]` | Each: `template_id`, `template_title`, `group_label?`, `group_key?`, `template_body`, `blank_specs?`. |
| `template_body` | string | Template text. A blank is written as a name inside curly braces, for example `{audience}`. Names use letters, digits, hyphens and underscores. A name that appears more than once is a single blank. |
| `blank_specs` | map name -> `{ hint_text?, fallback_value? }`, optional | Hint text shown as the blank's label; fallback values pre-fill the blank. |
| `group_label` | string, optional | Grouping label. |
| `group_key` | string, optional | Key that selects the group's marker style. |
| `selected_id` / `default_selected_id` | string | Controlled or uncontrolled selected template id. Defaults to the first template. |
| `on_select` | (template_id) => void | Fired when another template is chosen. |
| `on_submit` | (prompt_text, template, values) => void | Fired when Use is pressed with a complete prompt. Only callable once every blank is filled. |
| `on_copy` | (prompt_text) => void | Fired when the prompt is copied to the clipboard. |
| `list_title` | string | Heading. Defaults to "Prompts". |
| `submit_label` | string | Label of the Use button. Defaults to "Use prompt". |

Values typed for each template are kept per template for the life of the component, so returning to a template restores them.

## Anatomy
In reading order:
1. Heading and a search field.
2. List of templates, each row showing its title and group marker.
3. Detail area for the selected template: its title, a live preview of the finished sentence with blanks rendered as inline fields, and a preview block beneath the sentence.
4. Footer: a status line ("N of M filled" or the count of missing blanks), an estimated token count, a copy control, and the use control.

## States
- **Idle (no blanks filled):** blanks show their names or hint text as placeholders.
- **Partly filled:** the footer shows how many blanks remain; the use control is visible but not ready.
- **All filled:** the use control becomes ready; the status line states that all blanks are filled.
- **Copied:** copy confirms briefly, then returns to its normal label.
- **Used:** `on_submit` is called; the component does not clear values.
- **Switching templates:** the old content fades out while the new content fades in; the outgoing content is not interactive.
- **No templates:** an empty-state message; no detail area.
- **Search with no match:** the list shows an empty-result line; the detail area is cleared.

Transitions: choosing a template changes the detail; typing in a blank updates the preview live; filling the last blank makes the use control ready.

## Interactions
- **Pointer:** click a template to select it; click into a blank to type; click copy or use.
- **Keyboard, Down arrow from search:** moves focus into the list.
- **Keyboard, Up, Down, Home, End in the list:** move the selection; the detail updates as the selection moves.
- **Keyboard, Escape in search:** clears the search text.
- **Keyboard, Tab:** moves through the blanks in order, then to copy, then to use.
- **Keyboard, Enter in a blank:** moves focus to the next empty blank; when all are filled, activates use.
- **Touch:** same as pointer, with larger rows and fields.
- **Use while not ready:** stays focusable, announces the missing count, and does nothing.

## Motion (functional)
- The selection highlight slides between list rows while the detail content crossfades.
- A blank being typed into is briefly emphasised, then settles.
- The token estimate updates smoothly as text changes.
- When the last blank is filled, the use control gives a single brief emphasis.
- Reduced motion: all changes are immediate.

## Accessibility
- The list region is labelled with the list title.
- Templates form a listbox; the selected option carries a selected state; only the selected option is in the tab order.
- Each blank is a text field labelled with its name and hint text.
- Use remains focusable while not ready and names the missing items.
- A courteous status region reports completion of the blanks, copy actions and prompt use.
- Hidden or fading-out content is removed from the accessibility tree.

## Responsive
- Fills its container up to a readable width. When the container is narrow, the list moves above the detail; at the narrowest widths the copy control keeps only its icon.
- Blank fields grow with their content up to the width of the line.
- Touch targets are enlarged.

## Composition hints
- Card or panel for the container.
- Search input.
- Listbox with options.
- Inline text fields for blanks.
- Button pair for copy and use.
- Badge for group.
- Live region for announcements.

## Acceptance checks
1. A template with three blanks, two of which are filled, shows "2 of 3 filled", and use is not activated.
2. Filling the last blank makes use ready without calling any callback; activating use calls `on_submit` with the finished prompt and the values.
3. A blank named twice in the body is one field, filled once, and substituted in both places.
4. Switching to another template and back restores the values typed earlier.
5. Pressing Escape in the search field empties it.
6. Enter in a blank moves focus to the next empty blank.
7. Copy calls `on_copy` with the finished prompt text.
8. The token estimate is roughly one token per four characters and is labelled as an estimate.
9. Use does not call `on_submit` while any blank is empty, and it states the count of missing blanks to assistive technology.

## Amendment: structure and demo

**Archetype.** A two-column panel: a list of templates on the left with a search field above it; the selected template on the right. The right side is a sentence with inline fields, with the raw prompt text beneath. It is not a stacked form with labelled inputs.

**Parts that must exist:**
- Panel heading and search field above the list.
- List: one row per template, each with its title and a group badge; the selected row is marked.
- Detail: the template title; the sentence with its blanks as inline fields; a preview block beneath with the raw prompt, where an unfilled blank is shown by its name.
- Footer: a status text ("N of M filled"), an estimated token count, a copy control, and the use control.

**Gallery demo data:** four templates across three group badges; the selected template has four blanks, one of which is pre-filled from its fallback value.

**States visible at rest in the gallery:** partly filled. One of four blanks filled; the use control visible and not ready; the status text states the count.

**Acceptance checks, continued:**
10. Gallery at rest: the list shows four rows, each with a group badge, the first selected; the footer reads one of four filled and the use control is not ready.
11. Gallery at rest: the sentence shows four inline fields; the pre-filled one shows its value and the other three show their hint text as placeholders.
12. Gallery at rest: the preview block shows the raw prompt, with the three unfilled blanks shown by name.
</content>
