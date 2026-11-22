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

## Amendment: structure and demo

**Archetype.** A corner stack: a short column of small message cards anchored to one corner of its frame. Collapsed at rest, with older cards peeking out from behind the front card; hover or focus spreads them into a list. It is not a banner and not a single message.

**Parts that must exist:**
- Stack region with a label that names its focus key.
- Each message: status icon, title, optional detail line, optional action control, close control shown on hover and focus.
- Timer hairline on timed messages.
- Gallery trigger buttons that raise each kind of message.

**Gallery demo data, front to back:** a loading message (spinner in place of its icon, detail line naming a file); a success message with an action labelled Undo; a plain success message with a detail line.

**States visible at rest in the gallery:** collapsed stack of three; the loading message in front with no timer running; the action message behind it; the plain message at the back.

**Acceptance checks, continued:**
10. Gallery at rest: three messages are present; the front one shows a spinner in place of its icon and no running hairline; the second carries an action control; the third is a plain success.
11. Resolving the loading message in the gallery changes that same card in place to success or error; the number of cards in the stack does not change.
12. The collapsed stack shows the cards behind the front card by their edges; hover or focus spreads them so each card shows its full content.
13. Activating the Undo control closes that message and creates no new message; the other messages stay.

## Amendment 2: structure and demo

Replaces amendment 1 in three places. The gallery frame holds a small host sketch above the stack, so the frame is not empty. The gallery sets its stack edge to bottom centre rather than a corner. In the collapsed stack, the cards behind the front one show their title lines, not only their edges, so three messages are legible at rest.

**Gallery archetype:** host content first, then the component. The host sketch is a small file summary card with four action buttons; the buttons raise the three kinds of message and are gallery triggers outside the component. A one-line hint naming the focus key sits beneath the sketch. The stack is anchored to the bottom edge of the frame, centred horizontally, with the front card at the bottom.

**Component archetype:** a short column of small message cards. Collapsed, the front card is full width and the two cards behind it sit above it, offset so each shows its title line. Hover or focus spreads them into a list. It is not a banner and not a single message.

**Parts that must exist:**
- Stack region with a label that names its focus key.
- Each message: status icon, title, optional detail line, optional action control, close control shown on hover and focus.
- Timer hairline on timed messages.

**Gallery demo data (ours), front to back:**
- Loading message: title "Exporting report", detail line naming the file "report-q3.xlsx", spinner in place of its icon, no timer.
- Success message: title "Draft archived", action "Undo", timer hairline.
- Plain success message: title "Settings saved", detail line, timer hairline.

**Visible at rest:** host sketch, hint line, then the stack of three. The front card has a spinner and no hairline. The second card has the action control. The third is a plain success.

**Acceptance checks (continued):**
14. At rest the host sketch, hint line and stack appear in that order top to bottom, the stack sits at the bottom edge centred, and nothing overlaps.
15. The two cards behind the front card each show their title line, so three messages are readable at rest without hover. Replaces amendment 1 check 12 for the collapsed state.
16. The focus-key hint names the key; pressing it moves focus to the front card, the newest message, and the stack spreads (check 4).
17. Resolving the loading message changes that card in place to success or error, and the card count stays three (check 11).
18. Activating Undo closes that message and creates no new message; the other two stay (check 13).
19. The front card shows no timer hairline; the second and third cards do (check 10).
20. At phone width the stack spans the width minus a margin, and no card is clipped (check 9).
