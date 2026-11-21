# Background Task Inbox

Source item: affectUI Pro `components/agent-inbox` (functional reference only; see README).

## Purpose
Gathers every background agent job in one place, grouped by what the person has to do about it: jobs that need them, jobs still running, and jobs that have finished. Each job can be expanded to show its steps and given a next action (confirm, decline, restart, or inspect) without opening a modal per job.

## Data
- `jobs`: `{ id, heading_text, summary_text?, owner_id, stage: "running" | "waiting" | "done" | "failed", started_at?, elapsed?, tool_count?, source_count?, timeline?: { label, at? }[] }[]`.
  - `timeline` is ordered oldest first.
  - Supply `started_at` for running and waiting jobs so their clock advances; supply `elapsed` for finished jobs.
- `on_confirm(job)`: confirms a waiting job. Without it, the job moves to running inside the component.
- `on_decline(job)`: declines a waiting job. Without it, the job moves to failed.
- `on_restart(job)`: restarts a failed job. Without it, the job runs again and its clock begins from zero.
- `on_inspect(job)`: enables an inspect action on running and done jobs.
- `heading`: string, default "Agent inbox".
- `layout`: `"board" | "list"` (controlled), or `initial_layout` (uncontrolled, default `"board"`); `on_layout_change(layout)`.
- `extra_class?`

When handlers are supplied, the component never alters a stage on its own: the host updates the job and the card follows.

## Anatomy
From top to bottom:
1. Header: a breadcrumb-style label ("Agents / <heading>") and a Board / List switch (a radio group).
2. Board layout: three columns, in this order: Needs you (waiting and failed), Running, Done. Each column has a label, a count badge, and its cards in the order supplied.
3. List layout: a single list with filter tabs whose counts update as jobs change. Each row is one job.
4. Card: title; summary text (clamped to two lines); a stage line with an icon and text (for example "Waiting for you" with elapsed time, "Failed", or a running clock); source and tool counts when provided; and, when opened, a short timeline of the latest steps plus the actions valid for the stage.
5. Card actions: Confirm and Decline (waiting), Restart (failed), and Inspect (running or done, when `on_inspect` is supplied).

## States
- Job `running`: spinner indicator; live clock.
- Job `waiting`: needs-you marker; Confirm and Decline offered.
- Job `failed`: failure marker; Restart offered.
- Job `done`: completion marker; elapsed time is frozen.
- Card: closed or open (open shows steps and actions).
- Layout: board or list.
- List filter tab: selected or not, each showing a live count.
- Empty column: not specified by the reference; the clean room supplies a neutral empty line.

Column membership: the Needs-you column holds waiting and failed jobs. Within a column, cards keep the order they were supplied in.

## Interactions
- Clicking a card opens or closes it in place.
- Enter or Space on a card opens or closes it.
- The Confirm, Decline, Restart, and Inspect buttons call the matching handler, or apply the default transition described above.
- The Board / List switch changes the layout and calls `on_layout_change`.
- Escape closes the open card and returns focus to it.
- Touch: cards and buttons are taller.

## Motion (functional)
- Running jobs display a spinning indicator and a clock that ticks once per second.
- When a stage changes, the card first shows its new state where it sits, then glides to its new column.
- Cards open in place and reveal their steps as a short timeline, together with the actions.
- Reduced motion: stage changes take effect without any movement.

## Accessibility
- Each column is a labelled section.
- Each card is a button exposing `aria-expanded`; the details of a closed card are not reachable.
- The Board / List switch is a radio group.
- A polite live region states how many jobs need the person and how many are running.

## Responsive
- Three columns from a medium width upward.
- Narrower containers always show the list layout and hide the switch, down to a narrow minimum width.
- On touch screens, cards and buttons get larger targets.
- Light and dark themes are supported through an ancestor `dark` class or `data-theme="dark"`.

## Composition hints
Card (job), board layout (three columns), list with filter tabs (count badges), segmented radio control (layout switch), badge (stage), spinner, live clock text, timeline (steps), button group (actions), polite live region, position transition on column change.

## Acceptance checks
1. Jobs are grouped: waiting and failed under "Needs you", running under "Running", done under "Done"; column counts match.
2. A running job's clock advances once per second and stops when the job finishes.
3. Clicking a card toggles its open state, showing steps oldest first and the actions valid for its stage.
4. Confirm and Decline appear only on waiting jobs; Restart only on failed jobs; Inspect only on running and done jobs, and only when `on_inspect` is supplied.
5. Without `on_confirm`, confirming a waiting job moves it to running inside the component.
6. Switching to List shows one list, and its filter counts reflect the current jobs.
7. Enter or Space toggles a focused card; Escape closes it and returns focus to it.
8. Each change to the waiting or running counts produces one polite live announcement.
9. Narrow containers show only the list layout, with no layout switch.
10. With reduced motion on, a stage change appears without a glide.
