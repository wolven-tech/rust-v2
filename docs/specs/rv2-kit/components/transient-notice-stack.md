# Transient Notice Stack

Source item: affectUI Pro `components/toaster` (functional reference only; see README).

## Purpose
Brief, non-blocking messages raised from anywhere in the application: confirmations after an action, background jobs the person launched, and non-blocking failures. Each message carries a title, an optional detail line and an optional action such as Undo. A background job can open as a loading message and then flip to success or failure in place.

## Data
| Field | Type | Meaning |
|---|---|---|
| `stack_edge` | one of six corner or edge positions | Where the stack sits; messages enter from that edge |
| `inside_parent` | boolean | Anchors the stack to its nearest positioned ancestor rather than the viewport |
| `stack_key` | string, optional | Displays only the messages addressed to this key; permits a second independent stack |
| `default_hold_time` | duration, or infinite | Lifetime of a message before it dismisses itself. Infinite keeps it open |
| `visible_limit` | number | How many messages may show at once; the rest queue with their timers frozen |
| `always_expanded` | boolean | Keeps the stack fanned out permanently |
| `edge_inset` | number | Gap from the edges. Phones use a fixed small margin |
| `fan_gap` | number | Gap between messages once fanned out |
| `focus_key` | string or null | Keyboard shortcut key that moves focus to the newest message |
| `show_timer_line` | boolean | A hairline along the bottom that shrinks with the timer and freezes with it |
| `show_close_control` | boolean | Close control revealed on hover and focus; always shown on touch |
| `region_label` | string | Name announced for the region |

Imperative API:
- `notify(title, options?)`: a plain message, with variants for success, error, info and loading. Returns an id.
- `notify.promise(promise, { loading, success, error })`: a loading message that resolves into success or error. Each state may be a string, a title with detail, or a function of the result.
- `notify.dismiss(id?)`: removes one message, or every message when no id is passed.
- Options: `message_id` (reusing an existing id updates that message in place and restarts its timer), `detail_text`, `action: { action_label, on_activate }` (activation dismisses the message unless the handler vetoes it), `hold_time`, `status_mark`, `stack_key`, `on_dismiss`.

## Anatomy
Each message, in reading order: status icon, title, optional detail line, optional action control, close control.

The stack layers messages with the newest in front; older ones sit behind, a little smaller, peeking out. Hover or focus spreads them into a list.

## States
- **Entering:** rises from its edge.
- **Timed:** a hairline along the bottom empties as the countdown runs.
- **Frozen:** the countdown stops whenever the pointer or focus rests inside the stack, or the page is hidden.
- **Loading:** a spinner stands in for the status icon; no timer runs until the promise settles.
- **Resolved:** the spinner gives way to a ring holding a check or a cross; text cross-fades; the card resizes to fit the new text.
- **Replaced:** same id, new text; cross-fade; timer restarts.
- **Leaving:** fades and sinks; the messages above it rise to close the gap.
- **Flicked away:** exits in the direction of the swipe; a short swipe springs back.
- **Waiting:** messages past the visible limit hold with their timers frozen.

## Interactions
- **Pointer over the stack:** spreads it out; leaving gathers it again.
- **Pointer on an action:** runs it; the message closes unless the handler vetoes closing.
- **Pointer on close:** dismisses the message.
- **Touch:** a tap spreads the stack; a sideways or downward swipe dismisses; buttons get larger hit zones.
- **Keyboard, focus key:** focus jumps to the newest message and the stack spreads.
- **Keyboard, Tab:** cycles through each message's action and close controls.
- **Keyboard, Escape:** dismisses the focused message (or the newest) and passes focus onward; once none remain, focus returns to where it was.

## Motion (functional)
- New messages rise in from their edge; those already present step back and shrink slightly.
- Spreading and gathering glide positions and heights.
- Loading messages: the spinner folds into a result mark; text cross-fades; the card resizes.
- Dismissed messages sink and fade while the others move up; flicked messages leave in the swipe direction.
- With reduced motion requested, changes are instant, except that a loading spinner keeps turning.

## Accessibility
- The stack is a labelled region that names its focus key.
- Each message can take focus.
- Standard messages go to a polite status region; errors go to an alert region so they are heard first.
- Timers freeze during interaction, so nothing disappears before it has been read.
- A swipe is one route to dismissal among several; Escape and a named close control are always present.
- Icons are decorative; the text states the outcome, and state is never conveyed by emphasis alone.

## Responsive
- A fixed moderate width on large screens. On narrow screens the stack spans the width with a small margin and respects the bottom safe area of the device.
- Touch: tap to spread, swipe to dismiss, with larger close and action targets.

## Composition hints
- Toast or notification primitive for each message.
- Stacked layout with transform and spread.
- Timer hairline.
- Icon set for status.
- Two live regions: polite and assertive.

## Acceptance checks
1. A message left on the default hold time closes after that interval, unless the pointer or focus is inside the stack.
2. A loading message from a promise turns into a success message when it resolves and into an error message when it rejects, in the same position.
3. Calling `notify` with an existing `message_id` changes that message's text and creates no second message.
4. Pressing the focus key moves focus to the newest message.
5. Pressing Escape on a focused message closes it and moves focus to the next message.
6. An action whose handler vetoes closing keeps the message open.
7. A message beyond the visible limit is hidden, and its timer does not run until it is shown.
8. Error messages land in the assertive region and success messages in the polite one.
9. On a narrow screen the stack spans the width with a small margin and no message is clipped.
