# Agent Roster

Source item: affectUI Pro `screens/agents` (functional reference only; see README).

## Purpose
Gives a person a single place to see every agent they run, along with its status and access. From here they can pause or resume an agent, message it, inspect it, or add a new one to the team.

## Data
- `roster_entries`: list of `{ id, display_name, role_label, identity_hue_token, model_name, model_provider: "anthropic" | "openai" | "google" | "xai" | other, run_state: "working" | "idle" | "paused", current_activity?, last_run_label?, runs_this_week, cost_this_week, daily_run_counts?: number[], owner_name?, instruction_text?, skill_names?: string[], app_refs?: [{ id, entry_name, identity_mark? }], recent_tasks?: [{ entry_title, task_state, occurred_at }] }`.
- `identity_hue_token` is chosen from the design system's accent palette.
- `initial_view`: `"grid" | "list"`.
- `initial_open_id`: string or null.
- `is_loading`, `is_empty`: booleans.
- Callbacks: `on_pause(agent, paused)`, `on_message(agent)`, `on_create(agent)`, `on_navigate(entry_label)`.

## Anatomy
1. Sidebar (shared app frame).
2. Header: title, a summary line with a count per run state, a grid and list toggle, and a "New agent" button.
3. Main area: agent cards in a grid, or rows in a list. Each card shows the identity mark, display name, role, state badge, the app marks the agent may use, the model with its provider mark, and runs and cost for the week.
4. Detail panel (right) for the open agent: an identity header, its state with a Pause or Resume button, a Message button, and sections for Overview (a runs-per-day chart and cost), Instructions (first few lines, then "Show all"), Skills, Apps, and Recent tasks.
5. New agent form: name, purpose text, a model choice grouped by provider, and a tone choice. The identity mark updates live as the name is typed.

## States
- **working**: the badge reads Working; the identity mark shows a subtle live indicator.
- **idle**: the badge reads Idle; the card shows the last run label.
- **paused**: the badge reads Paused; the panel offers Resume.
- **loading** (`is_loading`): placeholder cards and placeholder lines in the panel.
- **empty** (`is_empty`): one short note with the New agent action; no cards.
- **panel closed**: the main area uses the full width.
- **creating**: the form is open; the identity mark previews the chosen name and tone.
- **pausing**: the button reflects the requested state immediately; the badge follows the host's update.

## Interactions
- Activating a card or row opens the agent in the panel. The open one is marked as current.
- Arrow keys move between agents in the grid or list. Enter opens the focused one.
- Pause or Resume calls `on_pause(agent, paused)`.
- Message calls `on_message`. Without it, the Thread screen opens.
- New agent opens the form. Escape closes it; Ctrl or Cmd+Enter creates the agent and calls `on_create`.
- Grid and list toggle: arrow keys move between the two options; the open agent stays open.
- The Apps and Skills entries in the panel navigate to Permissions and Skills.

## Motion (functional)
- A newly created agent fades in from a soft blur, and its card gets a brief highlight.
- A working agent's identity mark has a gentle idle animation. The status indicator in the panel pulses while the agent works.
- The runs chart fills in when the panel opens.
- Reduced motion: nothing moves; states change directly.

## Accessibility
- Each card is a button whose name includes display name, role, and state.
- The open agent is marked as current.
- A polite live region announces counts when they change, for example how many agents are working, idle, and paused.
- Grid and list controls carry names and tooltips.
- Focus rings appear for keyboard users only.

## Responsive
- Content is limited to a readable width in the middle of the page; cards sit in two columns, and in one column on phones.
- List rows drop role and model when the container is narrow, and drop the numbers when it is narrower still.
- At the narrowest width, the tool strip sits above the tabs and the New agent control shrinks to a plus icon.
- Touch targets are enlarged for touch input.
- The whole area scrolls so that card shadows and hover states are not clipped.

## Composition hints
Page header with counts; segmented control; card grid; list rows; state badge; identity mark; model tag with provider mark; app mark row; side panel (sheet on narrow screens); bar chart; tabs; button; form (text field, text area, select, tone swatches); empty state; skeleton; live region.

## Acceptance checks
1. The summary counts equal the number of agents in each run state.
2. Pausing a working agent changes its badge to Paused and calls `on_pause(agent, true)`.
3. Message without `on_message` navigates to the Thread screen.
4. Typing a name in the New agent form updates the identity mark before creation.
5. Ctrl or Cmd+Enter in the form creates the agent and calls `on_create` with the entered name, purpose, model, and tone.
6. Switching between grid and list keeps the open agent open.
7. With `is_empty` set, only the note and the New agent action appear.
8. At phone width, the grid shows one column and the page does not scroll horizontally.
