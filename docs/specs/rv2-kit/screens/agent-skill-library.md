# Agent Skill Library

Source item: affectUI Pro `screens/agent-skills` (functional reference only; see README).

## Purpose
Lets a person manage the reusable skills an agent can draw on: turn skills on or off, install them from a catalogue, author their own, and review for each skill when it applies, what it does, and what it needs.

## Data
- `skill_records`: list of `{ id, entry_name, entry_description, category_name, mark?, tone_token?, source_kind: "built-in" | "own" | "community", author_name?, version_label?, is_installed: boolean, is_enabled: boolean, weekly_use_count?, install_count?, trigger_phrases: string[], step_texts: string[], requirement_list: [{ entry_label, permissions_link? }], usage_log: [{ task_name, occurred_at }], daily_run_counts: number[] }`. `daily_run_counts` covers the last fourteen days.
- `category_names`: list of category names used as filters.
- `initial_skill_id`: string or null.
- `text_strings`, `is_loading`, `is_empty`.
- Callbacks: `on_install(id)` (may return a promise), `on_toggle(id, enabled)`, `on_remove(id)`, `on_create(skill)`, `on_select(id)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Header: title and a New skill button.
3. Filter row: category chips, plus a search field.
4. Installed section: cards for installed skills, each with its name, description, source tag, weekly use count, and an enable switch.
5. Discover section: cards for catalogue skills, each with its name, description, install count, and an Install button.
6. Skill panel (right): name, source, enable switch, Remove (own and community skills only), trigger phrases (the cues that bring the skill into play), steps, requirements (each linked to Permissions), a fourteen-day run chart, and recent usage.
7. New skill dialog with four entry points: describe the intended skill and let the agent draft it; write it by hand; upload a definition file; or browse the catalogue. Created skills land in Installed under Own and open in the panel.

## States
- **installed, enabled**: switch on; the card looks normal.
- **installed, disabled**: switch off; the card is dimmed but still listed.
- **discover**: the card shows Install.
- **installing**: the Install control shows progress until the install completes. A minimum visible progress period applies even to an instant install, so the change can be seen.
- **installed (just now)**: the card moves from Discover to Installed with a short highlight and scrolls into view.
- **built-in**: can be switched off but not removed.
- **creating**: the dialog is open; the agent drafts a name, a use, and steps for review.
- **filtered**: only skills in the chosen category remain in both sections.
- **empty** (`is_empty`): no skills at all; a note with New skill.
- **loading** (`is_loading`): placeholder cards and placeholder panel lines.

## Interactions
- Space on a switch turns the skill on or off and calls `on_toggle`.
- Enter on a card opens the skill in the panel.
- J or K, or the arrow keys, move between skills, through Installed and then Discover.
- Left and right arrows on the filter chips move between categories.
- "/" focuses search. Escape clears it.
- New skill dialog: up and down arrows move between entry points; Escape closes it; Ctrl or Cmd+Enter creates or writes.
- Remove asks for confirmation, then calls `on_remove`.

## Motion (functional)
- Install: the control becomes a progress indicator that fills, then shows a check. The card moves to Installed.
- The new Installed card receives a short highlight, and the installed count updates.
- The panel's run chart grows in bar by bar.
- Reduced motion: progress is shown as complete; no highlight.

## Accessibility
- Every skill card exposes a button whose accessible name combines its description, source, weekly use and off state.
- Each switch is a switch control named with its skill.
- Installing announces its progress through a polite live region: "Installing <name>", then "<name>: installed".
- Focus rings appear for keyboard users only.

## Responsive
- Installed and Discover share one grid; the number of columns follows the width.
- Below the phone breakpoint: a single column; search collapses to an icon while idle; New skill becomes a plus button.
- Category filters scroll horizontally when they do not fit.
- Touch targets are enlarged for touch input.

## Composition hints
Filter chips; search field; card grid; card with switch and badge; install button with progress indicator; side panel; bar chart; step list; permission links; dialog with entry options; empty state; skeleton; live region.

## Acceptance checks
1. Toggling a skill's switch calls `on_toggle` with the new state, and the card reflects it.
2. A built-in skill has no Remove control.
3. Install shows progress until the promise from `on_install` settles; the card then appears under Installed.
4. A category chip narrows both sections to that category.
5. A skill created from New skill appears under Own in Installed and opens in the panel.
6. Pressing "/" focuses search; Escape clears it.
7. With `is_empty` set, the note and New skill appear.
8. At phone width the page shows one column and no horizontal scroll.
