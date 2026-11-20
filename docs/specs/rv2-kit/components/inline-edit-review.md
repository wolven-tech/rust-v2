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
</content>
