# Progress Pill Scene

Source item: affectUI Pro `illustrations/progress-pill` (functional reference only; see README).

## Purpose
A compact status pill for a working agent: its mark, the title of the job, how far along it is, the step it is on, and a thin progress bar with a shimmer. It tells a person at a glance that an agent is still working on a named deliverable.

## Data
- `title`: name of the job.
- `agent_label`: role or name of the agent, with a short source count line.
- `progress`: number from 0 to 1; `step_label`.
- `state`: "working" | "paused" | "done".
- `mark`: the agent's mark (decorative pattern supplied by the design system).
- `extra_classes`.

## Anatomy
In reading order:
1. Mark: a small square pattern on the left, animated while working.
2. Text block: title on the first line with its percentage at the right; agent line with the source line beneath.
3. Step line: the current step, as a short subtitle.
4. Progress bar: a thin track with a fill to `progress`, with a shimmer moving across the fill.
5. Glow beneath the pill: a soft band whose light follows the state.

## States
- Working: mark animates; shimmer runs; glow breathes.
- Paused: mark stops; shimmer stops; glow fades.
- Done: mark rests in its complete form; bar is full; glow is a single steady band; percentage reads done.
- Reduced motion: no mark animation, no shimmer; state conveyed by text.

## Interactions
- Decorative by default.
- In interactive mode, the pill is a button that opens the job's detail; hover shows a subtle lift.

## Motion (functional)
- The fill grows to the new value when progress changes rather than jumping.
- The shimmer sweeps across the bar once per cycle while working.
- The mark pulses through its pattern while working to show life.
- The glow brightens slightly on each progress step.
- With reduced motion preferred, all of these hold still and the fill changes in one step.

## Accessibility
- The pill is a status region; progress changes are announced politely, not on every tick (only on step change or completion).
- Percentage and step are text, not only bar length.
- Interactive mode: the pill is a button named by its title and progress.

## Responsive
- Width follows the container; the text block truncates the title with an ellipsis and shows the full title on focus.
- Phone width: the pill fills the width with the same stack.

## Composition hints
- Status pill (container), progress bar, text blocks, icon mark, status region.

## Acceptance checks
1. Changing `progress` moves the fill to the new value without jumping.
2. The percentage text equals the fill value rounded to a whole number.
3. Paused state stops the mark and shimmer and reads paused in text.
4. Done state shows a full bar and the done label.
5. The step line updates when `step_label` changes and is announced.
6. With reduced motion preferred, the mark and shimmer are static.
7. At phone width the title truncates and nothing scrolls sideways.

Uses no partner logos.
