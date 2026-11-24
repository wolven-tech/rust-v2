# Inline Edit Review

Source item: affectUI Pro `components/inline-diff` (functional reference only; see README).

## Purpose
Presents an AI rewrite of a passage inline, marking each removed and each added word. A person accepts or rejects the changes one at a time, or all at once, by keyboard or pointer, and can undo. It suits "make it shorter", tone and grammar fixes, and review steps where a result may change only with explicit approval.

## Data
- `before_text` (text): the text as it stands now.
- `after_text` (text): the proposed text. Compared word by word against `before_text`.
- `segments` (Segment[], optional): precomputed segments, either `{ segment_kind: "same", text }` or `{ segment_kind: "change", removed_text, inserted_text }`. Used in place of `before_text` and `after_text`.
- `on_text_change(text)`: fires after each decision or undo, passing the resulting text; pending spans still hold the original words.
- `on_complete(text)`: called when the last change is decided, with the final text.
- `caption_text` (text, optional): small line above the panel (for example, the request that produced the rewrite).
- Control handle: `highlight(index | null)`, `accept(index)`, `reject(index)`, `accept_all()`, `reject_all()`, `undo()`, `reset()`.
- `extra_class` (text, optional).

## Anatomy
Reading order:
1. Caption line (optional).
2. Body: the paragraph, with unchanged text plain, removed words struck through in the error tone, and added words in the success tone. Changes sit inline, in order.
3. Toolbar: a count of pending suggestions, Reject all, Accept all, and previous / next arrows. The toolbar reads "All changes reviewed" when none remain.
4. Popover: opens above the focused change, with Accept and Reject buttons and their keys.

## States
- Pending change: shown with both removed and added words; not yet decided.
- Highlighted: the change under the arrows or focus; popover open.
- Accepted: removed words gone; added words remain as plain text.
- Rejected: added words gone; removed words remain as plain text.
- All reviewed: no pending changes; the toolbar shows completion text.
- Undo available: after any decision; undo restores the last decision.

## Interactions
Pointer:
- Click a change: highlights it and opens its popover.
- Popover Accept / Reject: decides that change.
- Reject all / Accept all: decide every pending change.
- Previous / next arrows: move the highlight; wraps around.

Keyboard:
- Tab / Shift+Tab: move between pending changes.
- Left / Right arrow on the toolbar: highlight the previous or next pending change (wrapping) and open its popover.
- Y or Enter: approves the focused change, then focuses the following one.
- N or Backspace: refuses the focused change, then focuses the following one.
- Escape: releases the change, or removes the arrow-driven highlight.

Focus rules:
- Arrow keys never seize focus. From a focused change, focus advances to the next; otherwise the target is highlighted while focus remains on the arrow control.
- Deciding the final change parks focus on Undo, preventing it from being lost.

## Motion (functional)
- Accepting: removed words shrink to nothing while their highlight fades and the added words settle into the text.
- Rejecting: added words shrink away; the strike-through and highlight fade.
- The popover fades in above a change and moves quickly between changes.
- The page scrolls to follow arrow navigation only when the target sits off-screen.
- The toolbar cross-fades to the completion text.
- Reduced motion: every change is instant.

## Accessibility
- Each pending change is a focusable button named with its position, removed text, added text, and the keys to decide.
- Deleted text uses the deletion markup and inserted text the insertion markup; the discarded half leaves the accessibility tree after a decision.
- Every decision, bulk action and undo produces a courteous announcement that includes the remaining count.
- Tone is not the only cue: strike-through and the deletion/insertion elements mark changes.
- Visible focus rings throughout.

## Responsive
- Fills its container; the paragraph reflows freely, and a change may break across lines while keeping its tint.
- At phone width the popover remains within the panel bounds.
- Narrow panels: the previous / next arrows are hidden.
- Touch: popover buttons are larger and key hints are hidden.

## Composition hints
Panel, caption, paragraph with inline diff spans, toolbar with counts and buttons, popover with actions, icon buttons, live region, word-level diff function.

## Acceptance checks
1. With `before_text` and `after_text` differing in three places, the count reads 3 and three pending changes exist.
2. Pressing Y on the first change accepts it: its removed words disappear and its added words remain; the count reads 2.
3. Pressing N on a change rejects it: its added words disappear and its removed words remain.
4. Reject all calls `on_complete` once with the original text; Accept all calls it with the rewritten text.
5. `on_text_change` fires on each decision and on undo with the text so far; undecided changes keep the original words.
6. Undo after the last decision returns focus to Undo, and the toolbar returns to the pending state.
7. Each pending change's accessible name includes its position, removed text, added text and the keys.
8. Left / Right on the toolbar wraps from the last change to the first.
9. In a narrow panel the arrows are hidden and changes are still reachable by Tab.
10. Reduced motion: decisions apply without shrink or fade.

## Amendment 4: structure and demo

**Replaces:** none. Anatomy item 3 and the arrow rules in Interactions stand. This amendment fixes their rest-state placement and the position wording. Base checks 1 to 10 stand.

**Archetype:** one panel holding an optional caption, a paragraph with its changes in place, and a toolbar along its foot. The popover is closed at rest.

**Parts at rest, top to bottom:**
- Caption line: small text above the paragraph. Optional.
- Paragraph: unchanged words plain. Each pending change shows its removed words struck through, followed by its added words marked as inserted.
- Toolbar, left to right: a count badge holding the number of pending changes, then the words "pending changes"; a previous arrow; a next arrow; a "Reject all" button; an "Accept all" button.
- Popover: closed.

**Position wording:** the count badge is the only visible indicator of position. No "n of m" text is drawn at rest or during navigation. Each pending change's accessible name carries its position in the form "Change <n> of <m>: replaces <removed> with <added>. Y accepts, N rejects." A pure deletion reads "removes <removed>" and a pure insertion reads "adds <added>".

**Arrow pointer behaviour:** a click on the next arrow highlights the following pending change and opens its popover above it. The previous arrow does the same for the preceding change. Both wrap. The popover holds Accept, keyed Y, and Reject, keyed N.

**Gallery demo (our data):** caption "Proposed edit · shorten the note". Original: "Hello team, the revised console is now available for every account. It loads a lot quicker than the old one, and reports can be exported as CSV with a single step." Suggested: "Hello team, the revised console is now live for every account. It loads faster than before, and reports can be exported as CSV in one step." Three pending changes: "available" to "live"; "a lot quicker than the old one" to "faster than before"; "with a single" to "in one". At rest the count badge reads 3, no decision is made, and no popover is open.

**Acceptance checks (continued):**
11. At rest the toolbar reads, left to right: a count badge showing 3, the words "pending changes", a previous arrow, a next arrow, "Reject all", "Accept all".
12. At rest the paragraph shows three pending changes, each with its removed and added words, and no popover is open.
13. A click on the next arrow highlights the next pending change and opens its popover above it. From the last change it wraps to the first.
14. Each pending change's accessible name follows the form in Position wording. The first gallery change reads "Change 1 of 3: replaces “available” with “live”. Y accepts, N rejects."
15. No visible text shows a position in the form "n of m" at rest or after an arrow click. The count badge is the only visible count.
</content>
