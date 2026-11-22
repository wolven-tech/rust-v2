# Inter-Agent Message Feed

Source item: affectUI Pro `components/collaboration-stream` (functional reference only; see README).

## Purpose
A running log of the messages agents exchange during a multi-agent run: who sent each one, who it was addressed to, and what kind of message it is (a task request, an answer, a handover, a broadcast, a point needing the person, or an agent start). A person can answer a point addressed to them, name a participant to address, and open a message to see its details.

## Data
| Field | Type | Meaning |
|---|---|---|
| `participants` | Participant[] | `{ participant_id, display_name, model_name?, badge_mark?, logo? }`. Each participant receives a stable colour and badge in list order. |
| `entries` | Entry[] | In send order. `{ entry_id, sender_id, recipient_ids[], entry_kind, body_text, sent_at?, answers_id?, is_drafting?, attachment_list?, detail_card? }`. `sender_id` is a participant id or `"you"`; `recipient_ids` holds participant ids, `"you"`, or `"all"`. |
| `entry_kind` | `"task_request" \| "answer" \| "handover" \| "broadcast" \| "needs_person" \| "agent_started"` | Per entry. Handovers are tagged; `needs_person` entries are tagged "Awaiting you"; `agent_started` is a single-line note. |
| `answers_id` | string | Id of the entry this one answers. Shown as a quote unless the entry sits directly beneath it. |
| `is_drafting` | boolean | The entry is still being written; shows a drafting indicator, and the sender's badge animates. |
| `attachment_list` | Attachment[] | `{ label, kind?, preview?, meta? }`, shown as chips. The kind is inferred from the label when omitted: a trailing slash means folder; a tabular extension means table. |
| `detail_card` | `{ notes?, rows? }` | Shown in a card beneath an entry when it is selected. |
| `run_subject` | string | Topic of the run; shown as part of the stream heading. |
| `is_running` | boolean | The run is still going: shows a live indicator and keeps the view on the newest entry unless the reader has scrolled up. |
| `picked_entry_id` / `on_pick_change(id \| null)` | controlled | Selected entry; its details open and the others fade. |
| `focused_participant_id` / `on_focus_change(id \| null)` | controlled | Participant whose entries stay in view, chosen from the participant tabs. |
| `on_compose({ text, recipient_ids, answers_id? })` | callback | Composer submission. `recipient_ids` holds participants named with @, or the participant being replied to; otherwise `["all"]`. |
| `compose_hint` | string | Default: "Write to a participant – @ to pick one". |
| `inline_logos` | Record<string, node> | Names to replace with logos inside text. |
| `scroll_height` | number | Height beyond which the feed begins to scroll. |
| `feed_name` | string | Accessible label for the whole feed. |
| `root_class` | string | Extra class applied to the root. |

## Anatomy
Reading order:
1. Header: run subject, live indicator, participant tabs (one per participant plus "All").
2. Feed: entries in order. Each entry shows its sender badge, sender name, "to" target, time, kind tag if any, body text, attachment chips, reply quote, and reply action.
3. Details card (beneath a selected entry).
4. Composer: text field, @ button, send control, and a hint when a point for the person is open.

## States
- **Idle:** no selection; all participants visible.
- **Participant tab selected:** only that participant's entries are shown (entries it sent or received).
- **Entry selected:** details card open beneath it; other entries fade.
- **Point for you open:** the tagged entry stays tagged until an entry from "you" sets its `answers_id` to that point.
- **Drafting entry:** the last item shows "Drafting…" with an animated indicator; it is replaced by the full entry once `is_drafting` is false.
- **Live:** indicator visible; new entries appear at the bottom and the feed follows them unless the reader has scrolled up.
- **Preview open** (attachment chip activated): a modal preview of the file; closes on Escape.
- **@ menu open** (composer): a list of participants filtered by the typed text.

## Interactions
- Pointer: click an entry to select it; click again to deselect. Click a participant tab to focus it; click again to clear. Click a reply quote to scroll to the answered entry and briefly highlight it.
- Arrow up/down: move between entries (roving focus). Home/End: first or last.
- Arrow left/right: move between participant tabs.
- Enter or Space on an entry: selects it or clears the selection.
- Escape: clears a selected entry or participant; in the composer, drops the reply target; in the preview, closes it.
- Typing @ in the composer summons the participant list: keep typing to narrow it, arrows to traverse, Enter or Tab to choose. Clicking the @ control again dismisses it.
- Enter in the composer submits; Shift+Enter inserts a line break.

## Motion (functional)
- New entries arrive from below with a soft entrance; the sender badge animates while the entry is drafting.
- Selecting an entry fades the others and opens the details card beneath it.
- Clicking a reply quote scrolls to the original and flashes it once.
- The preview and the @ menu rise slightly as they fade in.
- Reduced motion: all changes immediate.

## Accessibility
- The feed is a log region, announced politely while live.
- Each entry is labelled with sender, recipient, time and text.
- Roving focus over entries: only one entry is in the tab order at a time.
- Participant tabs form a toolbar of toggle buttons (pressed state); arrows move between them.
- Reply and attachment controls join the tab order only for the focused entry.
- The @ menu is a listbox linked to the field. The preview is a modal dialog that traps focus and returns it to the chip.
- Kind and "Awaiting you" are conveyed in words, not colour alone.

## Responsive
- Bubbles size to their text up to a readable width; they wrap when the container is narrow.
- Participant tabs scroll sideways when there are more participants than fit.
- The preview fits the component; its content scrolls internally.
- No horizontal scrolling at phone width.

## Composition hints
List (feed, roving focus), avatar or badge (participant identity), tabs or toggle group (participant filter), chip (attachments, mentions), card (details), listbox (@ menu), dialog (preview), text field and button (composer), live region helper, typing indicator.

## Acceptance checks
1. Entries render in array order; an entry whose `entry_kind` is `needs_person` with `recipient_ids` containing `"you"` shows an "Awaiting you" tag until an entry from `"you"` with `answers_id` equal to its id is appended.
2. Typing "@" in the composer and choosing a participant calls `on_compose` with that participant's id in `recipient_ids`.
3. An entry with `is_drafting: true` shows the drafting indicator; replacing it with `is_drafting: false` removes the indicator and keeps the entry's position.
4. Selecting an entry shows its `detail_card` and sets the others to a faded state; Escape clears the selection.
5. Choosing a participant tab shows only that participant's entries; choosing it again shows all.
6. Clicking a reply quote scrolls to the original entry and flashes it.
7. Attachments with a trailing slash in their label are shown as folders; those ending in a tabular extension are shown as tables.
8. With `is_running` true the feed scrolls to the newest entry on arrival, unless the reader has scrolled up.
9. Arrow keys move focus between entries with only one entry in the tab order at a time.
10. At phone width the feed and composer fit and no horizontal scrolling occurs.
</content>
