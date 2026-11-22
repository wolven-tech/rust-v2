# Workspace Setup Checklist Screen

Source item: affectUI Pro `screens/onboarding` (functional reference only; see README).

## Purpose
The first minutes in a new workspace. The person names the workspace, connects tools, invites teammates, and creates a first agent, through four steps they can move between. A checklist shows progress, and a way out lets them skip.

## Data
| Field | Type | Meaning |
|---|---|---|
| workspace_name | string | Defaults to a neutral placeholder; shown in the greeting and the first field. |
| account | { name: string; avatar_url? } | The first name appears in the greeting. |
| default_step | 1 \| 2 \| 3 \| 4 | Step shown first. Earlier steps count as done. Defaults to 2. |
| default_invites | string[] | Addresses already in the invite field, for example from sign-up. |
| connected_tools | { id; name; summary; connected: boolean }[] | Connectable tools for step 2; each is a switch. |
| templates | { id; name; description }[] | Agent templates for step 4; a radio group. |
| team_size | "1" \| "2-10" \| "11-50" \| "51+" | Chosen in step 1. |
| on_complete | (values: { workspace_name; url_slug; team_size; connected_tools; invite_emails; invite_role; agent_template }) => void | Called after the last step's Create agent, or after Skip on the last step. |
| on_navigate | (label: string) => void | Sidebar item, or a next-step shortcut (Chat, Integrations, Analytics). |

## Anatomy
Reading order:
1. Sidebar, left (see Workspace Frame).
2. Left column of the page: a greeting line with the user's first name, a short sentence on the time needed, a progress line ("1 of 4 done"), the list of four steps (each a button with its title and a one-line status), and a help box with a link to book a setup call.
3. Right column: the current step's form, its title and subtitle, the step's fields, and the action row (Skip for now, Back, Continue).

Steps:
1. Workspace: name, workspace address (derived), and team size (radio).
2. Connect your tools: a list of tools, each with a switch; a connected badge.
3. Invite your team: an email chip field that adds on Enter or comma, a role radio (Admin, Member), and a list of added chips.
4. Your first agent: a template radio group, an agent name field, and a Create agent button.

## States
- **Step active.** One step at a time; its form is shown.
- **Step done.** A check appears in the step list; the step can be reopened.
- **Step invalid.** Continue is off until the required field is valid (name present; each invite address valid).
- **Skipped.** "Skip for now" moves on without marking the step done.
- **Completed.** After Create agent, `on_complete` fires and the flow closes into the workspace home.

## Interactions
- Step list buttons: click or Enter jumps to that step; the current one is marked current.
- Enter in a step's field continues when the step is valid.
- Invite field: Enter, comma, or space adds the address as a chip; Backspace in an empty field removes the last chip; a chip's remove button removes that chip.
- Tool switches: Space or a click toggles them; the state is announced.
- Template radios: arrow keys choose; Enter continues.
- Back returns to the previous step with its entries kept.
- Continue validates and moves on.
- Skip for now moves on; the step stays unmarked.

## Motion (functional)
- A step's form slides in from the direction of travel.
- Checks draw in when a step is marked done.
- The progress line fills in increments per step.
- Under reduced motion, the step change is immediate.

## Accessibility
- Each step is a form; Enter submits the step's primary action.
- The step list marks the current step.
- Tool toggles are switches; templates are a radio group.
- Chips have accessible remove buttons; the count of invites is announced politely.
- Validation errors appear beside the field and in a polite region.

## Responsive
- On wide screens, the greeting and step list form a column beside the step; on narrow screens they stack above it.
- Action row buttons stay in order and wrap if needed.
- The page fills its container.

## Composition hints
- Workspace frame (see Workspace Frame (Inset)).
- Stepper navigation list with states (done, current, upcoming).
- Progress bar with step count.
- Form fields (text input, radio group, switch list, chip input).
- Button row (ghost skip, secondary back, primary continue).
- Callout card with a link.

## Acceptance checks
1. Opening the flow with `default_step` 3 shows steps 1 and 2 as done and step 3 as current.
2. Clicking a step in the list shows that step with its previously entered values kept.
3. Continue stays disabled while the workspace name is empty.
4. Typing an address and pressing comma adds a chip; Backspace on the empty field removes the last chip.
5. Skip for now moves to the next step without marking the skipped one done.
6. Creating the agent calls `on_complete` once with the workspace name, team size, connected tools, invite emails, role, and template.
7. Choosing a template with the arrow keys updates the selected template, and Enter continues.
8. The step list reflects "N of 4 done" after each change.
9. At phone width the greeting column sits above the form.
10. With reduced motion requested, step changes render without a slide.
