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

## Amendment 6: structure and demo

Replaces: none (no earlier amendment). Overrides the tool count in the default demo: six tools, not two.

**Archetype:** a setup group centred in the content area. A left column holds the greeting, progress and step list; a right column holds the current step's form and its action row.

**Parts visible at rest (gallery demo, default step 2):**
- Breadcrumb in the content area naming the workspace home and the current flow.
- Left column: greeting with the first name; one line on the setup time; progress line reading "1 of 4 done"; four step buttons. Step 1 is done and its status line shows the value entered there. Step 2 is current with the status In progress. Steps 3 and 4 are upcoming, each with a short status line. Below them, a help box with a link to book a setup call.
- Right column: step 2's title and subtitle; six tool rows, each with a mark, name, one-line summary and switch. The first is on and carries a Connected badge. The other five are off. The action row sits below: Skip for now at the left; Back and Continue at the right.
- The two columns form one group, centred in the content area and narrower than the area. The group is not stretched to the edges.

**States at rest:** step 2 current; no validation message; the first tool connected.

**Arrangement:** the group is centred horizontally. The left column is about a third of the group's width. The lower part of the content area stays open. Do not add filler content to it.

**Motion, settled frame:** nothing moves at rest.

**Defects to fix in ours:**
- Ours shows two tool rows where the at-rest demo has six.
- Ours spreads the two columns to the edges of the content area, which leaves the page sparse. The group is centred and width-limited.

**Checks (continued):**
11. At rest the right column lists six tools in order. The first is on with a Connected badge, and the other five are off.
12. The step list shows step 1 done with its entered value, step 2 current, and steps 3 and 4 upcoming. The progress line reads "1 of 4 done".
13. The left column (greeting, progress, four steps, help box) and the right column (form and action row) are visible together, centred horizontally in the content area. At 1280 x 720 without page scroll, all of them are visible.
14. The help box holds a working link to book a setup call.
