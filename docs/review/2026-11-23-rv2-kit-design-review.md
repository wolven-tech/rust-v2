# rv2-kit design review, 2026-11-23

Scope: the 88 components in `crates/rv2-kit`, reviewed in three themes (light, dark, test) at 1280, 390 and 320 px. Findings are numbered D1-n so fixes can cite them. Evidence paths are relative to the review scratchpad, `/private/tmp/claude-501/-Users-decebaldobrica-Projects-founder-mode-rust-v2/94abe5ef-7ab1-4f6f-9e22-964571af5935/scratchpad/`. `design-review/<theme>-<width>-NN.png` are contact sheets (six slugs each, in `design-review/slugs.txt` order). `t4c/w<width>-chunk-NN/<slug>-<theme>-<width>.png` are full-size captures.

## 1. Summary

**Verdict.** The kit is coherent and the token system holds. Dark and the test theme render every sheet checked on tokenized surfaces, and the test theme reads as a deliberately different product rather than a broken one. The weak points are narrow widths and the semantics of colour. At 390 and 320 px, components clip off the canvas, drop labels, or truncate the text a decision depends on. Orange and red also carry two meanings, as status and as chart series.

Counts: 80 live findings (D1-1 to D1-82; D1-22 and D1-27 withdrawn). High 11, Medium 37, Low 32.

**Systemic issues that matter most**

1. **No single narrow-width rule.** Some components reflow, some clip with no scroll cue, some drop columns or lanes silently, and some tables cut their last column or rows. See S1.
2. **Identity and decision text are truncated first.** Lane names, card titles, step titles, variant names, decision titles and consent sentences lose their words while pills and times remain. See S2.
3. **Orange and red mean two things.** They are status colours (needs you, failed) and also categorical series colours in charts, totals and treemaps. Series colours also change by theme. See S3.
4. **Encodings without keys.** Several colour-coded charts and grids have no legend, or a legend that does not show the colour. See S4.
5. **Primary action colour drifts.** The test theme uses teal and ink as primary, the light theme uses ink and rust, and the dark theme uses yellow for the same risky action. See S5.

## 2. Systemic findings

**S1. Narrow-width strategy is inconsistent (390 and 320).**
- Reflows well: delegation-card-tree, tool-call-network, run-process-lanes (vertical lanes at 320), sub-run-tree, overnight-run-digest (single column at 320), work-queue-flow, workspace-territory-map (becomes a list, see D1-48).
- Clips off the canvas with no scroll affordance: session-packing-planner (D1-4), parallel-edit-overlap-grid (D1-5), exchange-diagram (D1-6), background-jobs-popover (D1-3), model-selector (D1-24), connector-picker (D1-28), quick-action-palette (D1-13).
- Tables cut their last column or rows: segmented-bar-explorer (D1-56), co-editing-presence owners table at 320 (D1-52), sheet-grid (D1-29).
- Fix as one kit rule. Below 480 px, boards, timelines and tables use a horizontal scroller with a sticky first column and a visible scrollbar, or stack. Popovers clamp to the viewport with a 16 px gutter. Chip rows wrap or scroll with an edge fade. List-plus-detail components sequence the panes with a back action rather than overlapping them.

**S2. Identity and decision text are truncated or removed.**
- Lane names removed at 390: run-timeline-rows (D1-1).
- Card title removed at 390 and 320: expandable-work-item-card (D1-2).
- Titles and labels truncated at 320: changelog-timeline (D1-9), step-progress-list (D1-59), parallel-probe-feed (D1-55), variant-test-card (D1-57), request-routing-map (D1-64).
- Decision text truncated: consent-request-card at 390 and 320 hides "Expires in 40 seconds" (D1-10); absence-digest decision titles (D1-60); single-run-status-card agent name (D1-54).
- Citation chips truncated to fragments: source-linked-answer (D1-31).
- Rule: a row keeps its name and its state at every width. Text wraps before it truncates. Consent and approval sentences never truncate.

**S3. Status and category colour vocabulary collides.**
- Orange and red are status colours in most components and categorical colours in charts: segmented-bar-explorer (Closed is orange), part-of-total-card (Storage orange, Support red), provider-usage-chart (Code amber, Test red), context-treemap (tool output red).
- Series colours change by theme: segmented-bar-explorer Closed is orange in light, yellow in dark and green in test (D1-40); series-trend-chart Reopened is orange in light and green in test (D1-63).
- "Done" is drawn three ways: grey pill (delegation-card-tree, worker-fleet-panel), green pill (overnight-run-digest, run-cost-breakdown), and lowercase text (schedule-variance-timeline, candidate-race-board).
- Fix: reserve orange and red for status. Move categorical series to a separate token set that is consistent across themes. One status component (pill with dot, title case, fixed vocabulary) across the kit.

**S4. Encodings without keys.**
- parallel-span-timeline: four bar colours, no legend (D1-11).
- parallel-work-grid: filter chips are text-only, so grid colours are not keyed (D1-15).
- provider-usage-chart: two-tint stacks, no legend (D1-16).
- quota-usage-panel: near-identical greys in light and test (D1-17, D1-62).
- context-treemap and context-fill-grid: the same categories in different colours (D1-19).
- Fix: every categorical encoding ships a legend that shows the swatch. Add a shape or pattern where colour is the only channel.

**S5. Primary action colour and icon family drift.**
- Light: primary actions are ink, except access-grant-prompt, which is rust-brown.
- Dark: access-grant-prompt uses yellow for the same risky action (D1-51).
- Test: primary actions are teal (Allow, Grant, Approve, Open thread, Retry) and ink (Answer, Review, Deploy) in the same theme (D1-34).
- Icon treatments mix letter avatars, an emoji-style lightning glyph (automation-flow-editor), a sparkle (model-selector) and a warning glyph (consent-request-card, expandable-work-item-card).
- Fix: one primary-action token and one risky-action token per theme. One icon family.

## 3. Per-component findings

Ranked High, Medium, Low. Withdrawn rows are kept so the numbering stays stable.

| # | Severity | Component | Theme/width | Finding | Concrete fix | Evidence |
|---|---|---|---|---|---|---|
| D1-1 | High | run-timeline-rows | light 390 | Lane names and roles are removed. Rows show only letter avatars (M, S, D, A, C) and two ticks (0s, 60s). | Keep a name column of about 96 px at every width. Below 480 px stack the name above its track. Keep at least three ticks. | design-review/light-390-03.png; t4c/w390-chunk-00/run-timeline-rows-light-390.png |
| D1-2 | High | expandable-work-item-card | light 390 and 320 | The work item title ("Plan the offsite") is missing from the header at both widths. Ring, count, due date, avatars and status share one row. | Give the title a full-width line (flex-basis 100%, min-width 0). Move due date, avatars and status to a second row. | design-review/light-390-13.png; design-review/light-320-13.png |
| D1-3 | Medium | background-jobs-popover | light 390 and 320; dark 1280 | At 390 the popover reaches the stage edge and its row action buttons sit on the edge. At 320 names truncate to fragments ("Index meeti…", "Summarise …"). At 1280 dark the name truncates ("Index meeting n…"). | Clamp width to `100vw - 32px` and anchor inside the viewport. Give the name column a 120 px minimum and let names wrap to two lines before they truncate. | design-review/light-390-10.png; design-review/light-320-10.png; design-review/dark-1280-10.png |
| D1-4 | High | session-packing-planner | light 390 and 320 | The board is wider than the viewport. At 390 the Mon column is cut and Tue to Thu are off-canvas, with no scroll cue. | Horizontal scroller for day columns with a visible scrollbar. Below 640 px offer a one-day view with a day switcher. | design-review/light-390-10.png; design-review/light-320-10.png |
| D1-5 | High | parallel-edit-overlap-grid | light 320 and 390 | At 320 only the Ada column is visible; Ben and Cy sit off-canvas, so the file-level decision (Merge, kept counts) reads as incomplete. At 390 Ben is clipped and Cy is not visible. | Sticky file column with horizontal scroll for agent columns. Below 480 px restack each file as agent rows. | design-review/light-320-05.png; design-review/light-390-05.png |
| D1-6 | High | exchange-diagram | light 320 | Researcher, Writer and Reviewer lanes are off-canvas. "Scope noted" and "Complete" are cut to "Sco" and "Com" at the edge. | Same pattern as D1-5: scrollable lanes with a sticky time column. Clamp annotations inside bar bounds. | design-review/light-320-02.png |
| D1-7 | High | run-process-lanes | light 1280; dark 1280 | Node labels sit on connector lines. "Brief received" and "Approve deploy" are overprinted by the connectors that meet them. | Reserve a label box above or below each node with a fixed offset. Route connectors around label boxes. | design-review/light-1280-07.png; design-review/dark-1280-07.png |
| D1-8 | High | quota-load-scheduler | light 1280 and 390 | At 1280 "Now 10:30" overprints the axis labels. At 390 vertical gridlines run through lane names ("Invoice render", "Index rebuild") and the Show, gear and × controls crowd the card edge. The known input is confirmed. | Put the grid behind a label gutter so lane names are never crossed. Move "Now" to the header rule. Stack row actions under the lane below 480 px. | design-review/light-1280-08.png; design-review/light-390-08.png |
| D1-9 | High | changelog-timeline | light 320 and 390 | At 320 change titles truncate to fragments ("Wire …", "Add …", "Spread re…", "Log e…"). At 390 "Add …" and "Fix …" lose their subject. The title is the only identifier of each row. | Below 480 px put the title on its own line under the marker and let it wrap. Never truncate a row's only identifier. | design-review/light-320-05.png; design-review/light-390-05.png |
| D1-10 | High | consent-request-card | light 390 and 320 | The request line truncates mid-sentence (Uses the Monday temp… at 390; Uses the M… at 320). "Expires in 40 seconds" is in that sentence and is hidden at both widths. | Wrap the request text. Put expiry on its own labelled line beside the timer. Consent wording never truncates. | design-review/light-390-03.png; design-review/light-320-03.png |
| D1-11 | Medium | parallel-span-timeline | light 1280 | Four bar colours (mint, white, amber, red) and a dashed "held" state with no legend. Amber "Input needed" is the only state word. | Add a legend row, or put the state word on each bar. | design-review/light-1280-01.png |
| D1-12 | Medium | metric-trend-card | light 1280 and 320 | A floating tooltip and a static box repeat the same three values (Trials 210, Upgrades 72, Renewals 41). The static box is always visible. | Keep one readout. Use the tooltip for values, or a static legend without values. | design-review/light-1280-12.png; design-review/light-320-12.png |
| D1-13 | Medium | quick-action-palette | light 320 and 390; test 1280 | The command list runs under the footer. "Runs" is cut at 320 and 390; "Billing" is half hidden at test 1280. No scroll cue. | Cap the list height and make it scroll, with a visible scrollbar and a bottom fade. | design-review/light-320-11.png; design-review/light-390-11.png; design-review/test-1280-11.png |
| D1-14 | Medium | run-timing-waterfall | light 1280 and 390 | The red cursor line runs through the duration labels ("9.0s · $0.31"). At 390 bar names truncate ("Draft the weekl…", "Write …"). | Move the cursor label to the top rule. Put duration text left of the bar or on a second line. Keep the first word of every label. | design-review/light-1280-10.png; design-review/light-390-13.png |
| D1-15 | Medium | parallel-work-grid | light 1280 | Filter chips ("done", "in progress", "retrying", "failed") are text-only. Grid colours (green, blue, orange, red) are not keyed. | Give each chip a swatch matching the grid colour, or a shape as well as a colour. | design-review/light-1280-06.png |
| D1-16 | Medium | provider-usage-chart | light 1280; test 1280 | Each provider bar is stacked in two tints with no legend. "Tasks across all providers" reuses the provider hues (Code is amber, Test is red), so one hue means a provider in one place and a task type in another. | Add a segment legend. Give task types their own labelled palette. | design-review/light-1280-13.png; design-review/test-1280-13.png |
| D1-17 | Medium | quota-usage-panel | light 1280 | Four categories use near-identical greys (black, charcoal, mid grey, slate). Legend dots cannot separate them. | Use three distinct categorical tokens, or label the segments directly. | design-review/light-1280-14.png |
| D1-18 | Medium | part-of-total-card | light 1280 | Support (5%) is red, the failure colour used across the kit. Storage is orange, the warning colour. | Use the categorical set for series. Keep red and orange for status (S3). | design-review/light-1280-11.png |
| D1-19 | Medium | context-treemap; context-fill-grid | light 1280 | Same context categories, different colours. Tool output is red in the treemap; tool results are green in the fill grid. Documents are grey in one and orange in the other. | One category-to-colour map in tokens, consumed by both. | design-review/light-1280-05.png |
| D1-20 | Medium | transient-notice-stack | light 1280 | "Upload failed" is an action button label. It reads as a status and does not say what the button does. | Label the action "Retry upload". Keep the error in the notice body. | design-review/light-1280-14.png |
| D1-21 | Medium | annotation-sidebar | light 1280 and test 1280 | Resolved notes show two treatments: grey outlined "Resolved" on some, filled green "Resolved" on others. Nothing in the view says what separates them. | One resolved treatment. If green means resolved by the current user, label it. | design-review/light-1280-05.png; design-review/test-1280-05.png |
| D1-22 | Withdrawn | parallel-probe-feed | light 1280 | The first-pass claim (green progress fill on the failed Tests row) is not reproduced. The fill is red, matching the state. | None. | design-review/light-1280-00.png |
| D1-23 | Medium | branch-choice-map | light 1280 | Unselected sub-options (Notify now, Notify after cutover, Backfill tonight) are pale grey and read as disabled without saying so. Contrast not measured. | Measure the text. Below 4.5:1, use the secondary text token and mark the state ("Not chosen"). | design-review/light-1280-06.png |
| D1-24 | Medium | model-selector | light 390 and 320 | The popover runs past the stage edge. At 390 the cost column and "$0.50 / M tokens" are clipped; at 320 the popover is cut. | Constrain popover width to the viewport with a 16 px gutter. Let the footer wrap. | design-review/light-390-08.png; design-review/light-320-08.png |
| D1-25 | Medium | period-picker | light 320 | The preset row overflows; "Last 3…" is cut with no cue to scroll. | Wrap presets into two rows, or make the row a scroller with an edge fade. | design-review/light-320-12.png |
| D1-26 | Medium | inter-agent-message-feed; assistant-memory-manager | light 320; light 390 | Filter chip rows overflow. "Writ" is cut in the feed at 320. In memory manager "Peop" is cut at 320, and "People 1" sits on the edge at 390. | Wrap chips, or scroll the row with an edge fade. Apply one rule to every chip row (S1). | design-review/light-320-04.png; design-review/light-320-07.png; design-review/light-390-07.png |
| D1-27 | Withdrawn | decision-review-queue | light 390 | The first-pass claim (detail pane overlapping the queue at 390) was not confirmed on re-render. | None. | design-review/light-320-09.png |
| D1-28 | Medium | connector-picker | light 320 | The detail panel covers the list. At 320 the Available group is hidden under the Connected group and the detail. | Sequence list and detail below 640 px, with a back action. | design-review/light-320-12.png |
| D1-29 | Medium | sheet-grid | light 390 and 320 | At 390 the Site and Renews columns disappear with no signal, while "5 rows" and "Double-click a cell to edit" remain, so the table looks complete. At 320 the Status pills ("Planned", "Retired") are clipped. | Horizontal scroll with a sticky Tool column, or a visible column picker. Do not drop columns silently. | design-review/light-390-11.png; design-review/light-320-11.png |
| D1-30 | Low | run-control-composer | light 1280 and 320 | The budget control beside "42%" is an unlabelled radio at 1280. At 320 the queued row truncates ("Then add a ferry ride on day th…"). | Label the budget control. At narrow widths wrap the queued row, with Remove on its own line. | design-review/light-1280-09.png; design-review/light-320-09.png |
| D1-31 | Medium | source-linked-answer | light 1280 and 320; dark 1280 | Citation chips truncate to fragments ("example.c… Launch s…", "exam… Prici…", "1 L…", "3 B."). The chips are the only link to sources. | Chip shows the index and a short source name; the full title sits in the detail card. Do not cut a domain mid-word. | design-review/light-1280-04.png; design-review/light-320-04.png |
| D1-32 | Medium | automation-flow-editor | light 1280 | Step icons mix letter avatars (W, C) with a lightning glyph (Summarise with agent, Close the ticket). The 12 px add-step "+" connectors crowd the cards. Known input, confirmed roughly. | One icon family and one shape per step type. Give add-step controls a 24 px hit area and a 16 px gap. | design-review/light-1280-07.png |
| D1-33 | Medium | access-grant-prompt | light 1280 | "Allow once" is rust-brown. Approve and grant actions elsewhere are ink in light and teal in test. The primary colour has no single role. | Route primary CTAs through one token. Keep rust as a named risky-action variant if that is its role. See D1-51 for dark. | design-review/light-1280-08.png |
| D1-34 | Medium | consent-request-card; single-run-status-card; pending-decision-stack; absence-digest; traffic-split-router | test 1280 | The test theme has two primary colours. Allow (consent), Grant (single run), Approve (pending decision), Retry (worker fleet) and Open thread are teal. Answer, Review and Deploy are ink. | Pick one primary token per theme (S5). Retire the second. | design-review/test-1280-00.png; design-review/test-1280-01.png; design-review/test-1280-03.png; design-review/test-1280-04.png; design-review/test-1280-08.png |
| D1-35 | Low | cause-analysis-tree | light 1280 | Status pills are lowercase ("failure", "confirmed", "ruled out", "checking", "open"), against title case "Failed" and "Needs you" elsewhere. | Use the kit status vocabulary in title case (S3). | design-review/light-1280-09.png |
| D1-36 | Low | schedule-variance-timeline | light 1280 | Row status is coloured lowercase text ("done", "running", "waiting") with no pill, unlike other components. | Use the status pill. | design-review/light-1280-08.png |
| D1-37 | Low | candidate-race-board | light 1280 | Status is an inline lowercase word ("running") beside the model name, not a pill. | Use the status pill. | design-review/light-1280-08.png |
| D1-38 | Low | overnight-run-digest | light 1280 | "Finished" and "Done" both mark completed items in the same column. | One completed label. | design-review/light-1280-02.png |
| D1-39 | Low | onboarding-checklist-panel | light 1280 | The group heading "Team" is struck through, which reads as dismissed, while the group is open with "1 of 1". | Strike only completed items. Group headings use normal weight with a count. | design-review/light-1280-13.png |
| D1-40 | Low | segmented-bar-explorer | light, dark and test 1280 | Closed is orange in light, yellow in dark and green in test. The series-to-colour mapping changes by theme, so meaning follows the theme, not the data. | Fix series-to-colour mapping in tokens (S3). | design-review/light-1280-11.png; design-review/dark-1280-11.png; design-review/test-1280-01.png |
| D1-41 | Low | metric-tile | light 1280 | The cost tile uses a bar sparkline; the other tiles use lines. | One sparkline encoding across the tile family. | design-review/light-1280-13.png |
| D1-42 | Low | daily-usage-bars | light 1280 | Today's bar is darker with no label or tooltip, so the emphasis has no stated meaning. | Label the highlighted bar or drop the emphasis. | design-review/light-1280-13.png |
| D1-43 | Low | connector-settings-card | light 1280 | The Access key field is empty with no placeholder or "not generated" state. Reveal and Generate new sit beside an empty field. | Add an empty-state hint ("No key yet"). Disable Reveal until a key exists. | design-review/light-1280-12.png |
| D1-44 | Low | charge-phase-builder | test 1280 | Serif figures in amounts and dates ("$300.00", "28 February 2027") may not align in the table and totals. Checked at contact-sheet scale only. | Use tabular figures for numeric tokens in the test theme. Confirm at full size. | design-review/test-1280-11.png |
| D1-45 | Low | agent-clash-map | light 1280 | "Faint" items use pale grey text at small size. Contrast not measured. | Measure. Raise to the secondary text token if below 4.5:1. | design-review/light-1280-01.png |
| D1-46 | Low | agent-context-map | light 1280 | Five small unlabelled chips (D, C, M, C, S) sit under the Writer node. Their meaning is not stated. | Label them, or remove them. | design-review/light-1280-07.png |
| D1-47 | Low | single-agent-stage | light 1280 | Small square connector nodes float between cards and read as render artefacts. | Remove them, or draw them as part of the connector line. | design-review/light-1280-03.png |
| D1-48 | Low | workspace-territory-map | light 1280 and 320 | The "set…" tile label is truncated at 1280. At 320 the map becomes one long list, which keeps the data and loses the spatial grouping. | Show the full name in the tile or tooltip. Document the narrow form as a list, not a map. | design-review/light-1280-09.png; design-review/light-320-09.png |
| D1-49 | Low | work-queue-flow | light 1280 | The dashed orange threshold line in the sparkline has no label. | Label the threshold with its value. | design-review/light-1280-02.png |
| D1-50 | Low | variant-test-card | light 1280 | The sample size line "n=1200" is very small and grey, with no label. | Label the sample size at body size ("Sample: 1,200 visitors"). | design-review/light-1280-12.png |
| D1-51 | Medium | access-grant-prompt | light, dark and test 1280 | The primary "Allow once" button is rust-brown in light and test, and yellow with dark text in dark. The same action has three hues. | One risky-action token per theme, used in all three. | design-review/light-1280-08.png; design-review/dark-1280-08.png; design-review/test-1280-08.png |
| D1-52 | High | co-editing-presence | light 320 | The Owners table body is clipped to its header row (Owner, Task, Status). No owner rows are visible and the card gives no scroll cue. | Wrap the table in a horizontal scroll container, or stack each row as a labelled block below 480 px. Never clip table rows silently. | design-review/light-320-00.png |
| D1-53 | Medium | agent-clash-map | light 320 | Node labels truncate ("Release…", "Docs wr…", "Billing b…"). Decide and Notice chips become large blocks that sit on the connector lines, so the topology is unreadable at this width. | Below 640 px show clashes as a list grouped by decision type, with the graph as a secondary view. | design-review/light-320-01.png |
| D1-54 | Medium | single-run-status-card | light 320 | The agent name "Refactor agent" truncates to "Re…", so the subject of a Grant decision is lost. | Let the name wrap. Move the awaiting-grant pill to its own row. | design-review/light-320-01.png |
| D1-55 | Medium | parallel-probe-feed | light 320 | Finding titles truncate ("Secret i…", "Snapsh…", "Unused…", "Option…") and check names truncate ("Docs lin…", "Bundle…"). | Let titles wrap to two lines. Keep the severity tag and the source on their own line. | design-review/light-320-00.png |
| D1-56 | Medium | segmented-bar-explorer | light 320 | The data table clips its last column. The Total header and row values are cut to a single character at the edge. | Horizontal scroll with a visible scrollbar, or move Open and In review to a detail view below 480 px. | design-review/light-320-11.png |
| D1-57 | Medium | variant-test-card | light 320 | Variant names truncate ("Current co…", "Short co…", "Benefit-l…"), so the comparison loses its labels. | Let names wrap. Keep the letter and the metric on one row. | design-review/light-320-12.png |
| D1-58 | Low | provider-usage-chart | light 320 | X-axis labels truncate ("2.2M tok…", "500k tok…"). The unit is cut from every label. | Show the provider name only, and state the unit once in the chart header. | design-review/light-320-13.png |
| D1-59 | Medium | step-progress-list | light 320 | Step titles truncate next to their status pills ("Send th…", "Pick the relea…", "Upload the pre…"). The step identity is lost while the pill remains. | Let step titles wrap. Keep the status pill on the title line or move it under the title. | design-review/light-320-03.png |
| D1-60 | Medium | absence-digest | light 320 | Decision titles truncate ("Pricing copy ne…", "Invoice run wait…"). These are the items the user is asked to answer. | Let titles wrap. Keep the state pill below the title. | design-review/light-320-04.png |
| D1-61 | Low | trust-level-selector | light 320 | "Autonomous" touches the edge of the segmented control. | Let the control scroll, or wrap labels to two lines below 360 px. | design-review/light-320-04.png |
| D1-62 | Low | quota-usage-panel | test 1280 | The four segments are brown shades that cannot be told apart. Same problem as D1-17, in the test theme. | Use distinct categorical tokens, or label the segments directly. | design-review/test-1280-14.png |
| D1-63 | Low | series-trend-chart | test 1280 | Series colours differ from light: Open items is red-orange (teal in light), Reopened is green (orange in light), Average age is purple (red in light). Meaning follows the theme. | Fix the series-to-colour mapping in tokens (S3). | design-review/test-1280-14.png; design-review/light-1280-14.png |
| D1-64 | Low | request-routing-map | light 320 | "Partnership i…" truncates on a waiting item, and that item is the one that needs the user. | Let the name wrap. | design-review/light-320-02.png |
| D1-65 | High | exchange-diagram | light 390 | Reviewer lane is cut at the canvas edge. The header reads "R Re" and the lane's track is off-canvas, so the third participant cannot be read. "Plan the relea…" truncates and "Scope noted" sits on the bar edge. Same defect as D1-6 at 390, with no scroll cue. | Same pattern as D1-6: scrollable lanes with a sticky time column. Keep at least two full lane headers visible at 390. | design-review/light-390-02.png; t4c/w390-chunk-00/exchange-diagram-light-390.png |
| D1-66 | Medium | single-run-status-card | light 390 | Agent name truncates to "Refactor ag…" beside the "Awaiting grant" pill. Same as D1-54 at 390; the subject of the Grant decision is cut. | Let the name wrap. Move the awaiting-grant pill to its own row (D1-54 fix). | design-review/light-390-01.png; t4c/w390-chunk-00/single-run-status-card-light-390.png |
| D1-67 | Medium | inter-agent-message-feed | light 390 | Filter chip row runs off the right edge. The last chip reads "Crit" and is cut. Same as D1-26 for this component, at 390. | Wrap the chips, or scroll the row with an edge fade (S1). | design-review/light-390-04.png; t4c/w390-chunk-01/inter-agent-message-feed-light-390.png |
| D1-68 | Medium | source-linked-answer | light 390 | Citation chips truncate to "Laun…", "P.", "Bet…", "Pla…". Same as D1-31 at 390. The chips are the only link to sources. | As D1-31: chip shows the index and a short source name; the full title sits in the detail card. | design-review/light-390-04.png; t4c/w390-chunk-01/source-linked-answer-light-390.png |
| D1-69 | Medium | decision-review-queue | light 390 | Detail opens as a bottom sheet over the queue. Its sticky action bar (Approve, Edit, Reject, Skip, Undo) covers the "CHANGELOG" heading mid-glyph and hides the queue row "Clean up stale branches". No scroll cue. This full-size capture re-observes the condition D1-27 withdrew (see Limits). | Scroll the detail body above the action bar, with bottom padding equal to the bar height. Show a count or handle for the queue behind the sheet. | design-review/light-390-09.png; t4c/w390-chunk-01/decision-review-queue-light-390.png |
| D1-70 | Medium | run-control-composer | light 390 | Queued row "Then add a ferry ride on day three" pushes Remove off the right edge ("Remov"). The destructive action is cut at this width. Same queue row as D1-30 at 320. | Wrap the queued row with Remove on its own line (D1-30 fix). Label the unlabelled square and radio beside Chat/Act (D1-30). | design-review/light-390-09.png; t4c/w390-chunk-02/run-control-composer-light-390.png |
| D1-71 | Medium | period-picker | light 390 | Preset row (Today, Yesterday, Last 7 days, Last 30 days, then "L…") runs off the right edge with no cue. Same as D1-25 at 390. | As D1-25: wrap presets into two rows, or make the row a scroller with an edge fade. | design-review/light-390-12.png; t4c/w390-chunk-02/period-picker-light-390.png |
| D1-72 | Medium | connector-picker | light 390 | The detail panel covers the Available group. Only a sliver of its first row shows. Same as D1-28 at 390. | As D1-28: sequence list and detail below 640 px, with a back action. | design-review/light-390-12.png; t4c/w390-chunk-02/connector-picker-light-390.png |
| D1-73 | Low | agent-clash-map | light 390 | Labels all fit at 390, but connectors run into and behind the Notice and Faint cards, so lines cross node edges. Less dense than at 320 (D1-53). | Draw connectors beneath nodes and end them at node edges. | design-review/light-390-01.png; t4c/w390-chunk-07/agent-clash-map-light-390.png |
| D1-74 | Low | baton-pass-log | light 390 | Drafter summary line truncates: "Opening paragraph, Quote from 2019 report, Pricing t…". This is the only list of what the worker holds, and it is cut. | Let the summary wrap to two lines. | design-review/light-390-01.png; t4c/w390-chunk-00/baton-pass-log-light-390.png |
| D1-75 | Low | change-replay | light 390 | Sparkline has a flat red baseline with no label or key. File timeline ticks are 3 to 6 px marks, too small to read the timing. | Label the baseline series or drop it. Give file ticks a minimum width and a value label. | design-review/light-390-06.png; t4c/w390-chunk-03/change-replay-light-390.png |
| D1-76 | Low | series-trend-chart | light 390 | "Completed" has a grey band between W3 and W5 with no line and no label. Axis labels are about 6 px. | Label the band in the legend. Set axis labels to at least 10 px. | design-review/light-390-14.png; t4c/w390-chunk-02/series-trend-chart-light-390.png |
| D1-77 | Low | transient-notice-stack | light 390 | Notice cards overlap: "Draft archived" covers the bottom edge of "Settings saved". Each progress rule runs along the title line, and the Undo action floats at mid-height. | Stack notices with a fixed gap. Put the progress rule on the card bottom edge and align Undo to the title line. | design-review/light-390-14.png; t4c/w390-chunk-01/transient-notice-stack-light-390.png |
| D1-78 | Low | work-queue-flow; file-drop-area | light 390 | Monospace state strings sit outside the cards: "slots=3" under work-queue-flow and "3 files, 1 done" under file-drop-area. They read as harness or debug text, not component copy. Not checked at other widths. | Confirm in source whether the component renders them. If it does, remove them or move them into the card header. | design-review/light-390-02.png; design-review/light-390-12.png; t4c/w390-chunk-00/work-queue-flow-light-390.png; t4c/w390-chunk-02/file-drop-area-light-390.png |
| D1-79 | Low | plan-comparison-section | test 390 and 320 | Price figures use old-style serif numerals. In "$129" the 9 drops below the baseline, so "$19", "$49" and "$129" do not share a baseline across the cards. Same class as D1-44. | Use lining tabular figures for price tokens in the test theme. | t4c/w390-chunk-03/plan-comparison-section-test-390.png; t4c/w320-chunk-03/plan-comparison-section-test-320.png |
| D1-80 | Low | plan-comparison-section | dark 390 and 320 | Primary "Start trial" is a pale grey fill with dark text. The test capture of the same card uses teal (D1-34 pattern), so the plan-card primary differs by theme (extends S5). | Route the Start trial primary through the one primary-action token (S5). | t4c/w390-chunk-03/plan-comparison-section-dark-390.png; t4c/w320-chunk-03/plan-comparison-section-dark-320.png |
| D1-81 | Low | editable-step-plan | light 390 | Remove controls are a bare "×" glyph about 10 px tall with no visible label. Hit area not measured (static capture). | Give each remove control a 24 px hit area and an accessible name ("Remove step 2"). | design-review/light-390-02.png; t4c/w390-chunk-00/editable-step-plan-light-390.png |
| D1-82 | Low | overnight-run-digest | light 390 | In the second completed row, the status word "Applied" sits as plain green text between Inspect and Revert. The first completed row has no status word. The chart plot fills about 290 of its 340 px frame, with no axis or label. | Use one action row per completed item. Label the chart axis or fill the frame. | design-review/light-390-02.png; t4c/w390-chunk-00/overnight-run-digest-light-390.png |

## 4. What is good and should not change

- The token system holds. Every dark and test sheet re-rendered in this pass uses tokenized surfaces, with no unthemed panel or stray light block.
- Dark and test are the clearest themes at 1280. Primary text and status chips read clearly at that width by eye. Contrast is not measured.
- Consent and decision components keep their timers and keyboard hints (consent-request-card, decision-review-queue, run-control-composer). Keep the pattern.
- Several components reflow well and should be the model for S1: delegation-card-tree, tool-call-network, run-process-lanes (vertical lanes at 320 and 390), sub-run-tree, overnight-run-digest, metric-tile, quota-usage-panel and multi-step-wizard-form.
- State coverage is visible in the flow and run components: running, failed, awaiting input, skipped, pending and idle states are distinct. Skeleton loading state is present in metric-tile.
- No gratuitous gradients or decorative backgrounds were seen. Copy is short and specific, with real identifiers (file names, counts, timestamps).

## 5. Limits of the review

- **Process.** The first pass's image reads returned no image ("media removed: request limit"), so its rows were drafted before the sheets were in view. The second pass re-rendered each sheet cited here and checked every row against it. Two rows were withdrawn (D1-22, D1-27). Several were corrected in severity, evidence or wording (D1-3, D1-6, D1-7, D1-9, D1-30, D1-34, D1-47, D1-48, D1-50).
- **Coverage, exact.** Light 1280: 15 of 15 sheets. Dark 1280: 15 of 15. Test 1280: 15 of 15. Light 320: 15 of 15. Light 390: 15 of 15 sheets viewed. Sheets 00, 03, 05, 07, 08, 10, 11 and 13 were reviewed in the first pass. Sheets 01, 02, 04, 06, 09, 12 and 14 were added in this pass.
- **Light 390 full-size checks (24 components).** exchange-diagram, single-run-status-card, inter-agent-message-feed, source-linked-answer, work-queue-flow, baton-pass-log, agent-clash-map, request-routing-map, mid-run-note-panel, absence-digest, pending-decision-stack, trust-level-selector, overnight-run-digest, background-task-inbox, editable-step-plan, change-replay, workspace-territory-map, decision-review-queue, run-control-composer, connector-picker, period-picker, file-drop-area, series-trend-chart, transient-notice-stack. Each was opened and read at full size.
- **Light 390 contact-sheet only (16 components, no full-size check).** parallel-span-timeline, reversible-change-log, prompt-input-bar, agent-question-card, branch-choice-map, task-assignment-board, model-failover-flow, parallel-work-grid, deliberation-digest, interruptible-answer-stream, cause-analysis-tree, variant-test-card, connector-settings-card, metric-trend-card, multi-step-wizard-form, quota-usage-panel. No 390-specific defect was recorded for these. Where the sheet shows a known defect at 390, it is noted as an extension of an existing row: D1-23 (branch-choice-map pale sub-options), D1-15 (parallel-work-grid unkeyed dots), D1-17 (quota-usage-panel greys), D1-12 (metric-trend-card repeated readout), D1-43 and D1-50 (connector-settings-card, variant-test-card). Those are not re-checked at full size.
- **Light 390 sheet vs full size.** workspace-territory-map looked illegible on the contact sheet. At full size its text is legible and it shows the D1-48 list form, so no row was added.
- **plan-comparison-section.** All nine captures reviewed: light, dark and test at 1280, 390 and 320. Dark and test at 390 and 320 were added in this pass (D1-79, D1-80). The selected-pill text overlap reported by the builders was not reproduced in any of the nine captures. The component is not one of the 88 slugs.
- **D1-27 conflict.** The light 390 full-size capture of decision-review-queue (t4c/w390-chunk-01/decision-review-queue-light-390.png) shows the detail sheet over the queue, the condition D1-27 described. D1-27 stays withdrawn as written. D1-69 records the re-observation. Re-check at 390 before either row is closed.
- **Static only.** No Playwright run, so hover, focus, keyboard reachability and selected-state transitions were not checked. The quota Now and guide overlap and the run-process-lanes label overlap were confirmed visually (D1-7, D1-8). The automation add-step crowding was confirmed roughly (D1-32). The run-timeline transfer markers not being reachable by Tab, and the charge-phase edit-state date format, were not checked and are not in the table.
- **Scaled captures.** Contact sheets are downscaled, so text under about 10 px on the 320 and 390 sheets is not legible. Small-text findings should be confirmed at full size before a fix is closed.
- **Contrast not measured.** D1-23 and D1-45 need a pixel or token measurement before a fix is judged.
- **Motion not assessed.** Stills cannot show easing, motion restraint or reduced-motion behaviour.
- **No source read.** Fixes are stated at design level. Token names and CSS were not inspected.

## 6. Resolution

Every live finding is either fixed or accepted with a reason. Verification after the fixes: the theme proof over all 233 gallery items in light, dark and the test theme at 1280, 390 and 320 px reports 0 identical renders and 0 contrast or small-text findings, apart from the known plan-comparison-section artefact (the selected pill's background is a sibling element, so the checker reads "Monthly" against the track). The High findings were re-checked by eye at 390 and 320.

### Systemic

- **S1 narrow widths.** Boards, timelines and grids scroll under about 560 px with a sticky first column, a thin scrollbar and an edge fade, or they stack (co-editing-presence owners table, step lists). Popovers clamp to the viewport with a 16 px gutter. Chip rows wrap. List and detail panes take turns with a back control (decision-review-queue, connector-picker). The breakpoints still vary between 480 and 560 px by component; a single kit value is not enforced.
- **S2 names and decision text.** Names, titles and consent sentences wrap instead of truncating across the components listed below. Preview lines may still truncate; they are not a row's only identifier.
- **S3 status and category colour.** The identity and series palettes in `kit.css` contain no orange or red, in every theme. Status words use one title-case vocabulary shown as a pill with a dot. Components keep a few extra states the core list does not have (Queued, Idle, Complete, In flight).
- **S4 keys.** Every categorical encoding named in this review now has a legend with swatches.
- **S5 primary action.** `--kit-primary` and `--kit-risky` are defined in `kit.css`; primary buttons across the kit use the first, and only genuinely risky actions (broad access grants, destructive steps) use the second. Stop on run-control-composer is a secondary button, since stopping a run is not destructive.

### Per finding

| # | Status | Resolution |
|---|---|---|
| D1-1 | Fixed | Name column kept at every width (120 px under 480 px); names and notes wrap; ticks thin instead of vanishing; legend added. |
| D1-2 | Fixed | Title on its own row; count, due date, avatars and status wrap as one group under 480 px. "In progress" is a neutral pill. |
| D1-3 | Fixed | Panel clamped to the viewport; name column wraps; meta on its own line under 480 px; title-case pills. In the gallery's clipped demo canvas at 320 px the row actions still meet the canvas edge; in a page the clamp keeps them inside the viewport. |
| D1-4 | Accepted | Board scrolls with sticky row labels, a scrollbar, an edge fade and an agent key. No one-day view; the scroller does the job. |
| D1-5 | Accepted | Sticky file column and edge fade; Merge uses the primary token. Agent columns scroll rather than restack. |
| D1-6, D1-65 | Accepted | Sticky time gutter, scrollbar and edge fade; the Reviewer lane is reached by scrolling at 390 and 320 px. |
| D1-7 | Fixed | Titles above each node and status pills below it; labels no longer cross connectors. |
| D1-8 | Fixed | "Now" sits on the plot's top edge; lane labels sit above gridlines when narrow; the end row wraps. |
| D1-9 | Fixed | Change titles wrap and take their own line under 480 px. |
| D1-10 | Fixed | Request sentence wraps; "Expires in N seconds" is its own labelled line beside the timer. |
| D1-11 | Fixed | Legend with swatches; names and state stay visible when narrow. |
| D1-12 | Fixed | One readout: the tooltip, ordered by value. The spec was updated to match. |
| D1-13 | Fixed | Names wrap; list scrolls with a scrollbar and a bottom fade. |
| D1-14 | Fixed | Clock line in ink, captions above it, labels wrap, legend added. |
| D1-15 | Fixed | Each filter chip carries the swatch of its cell colour. |
| D1-16 | Fixed | Swatches per provider and per model; a key explains the darker segment; task types use identity colours. |
| D1-17, D1-62 | Fixed | Four distinct series colours with named legend rows. |
| D1-18 | Fixed | Series colours, no orange or red. |
| D1-19 | Accepted | Treemap and fill grid share one category-to-series map with a legend. The map is a convention documented in both components, not a token. |
| D1-20 | Fixed | The action reads "Retry upload"; the error stays in the body. |
| D1-21 | Fixed | One resolved treatment; open notes offer "Mark resolved". |
| D1-22 | Withdrawn | Not reproduced (see the table). |
| D1-23 | Fixed | Unchosen options keep full text colour and read "Not chosen". |
| D1-24 | Fixed | Menu clamped to the viewport; footer wraps; the thinking mark uses the think colour, not warn. |
| D1-25, D1-71 | Fixed | Presets wrap. |
| D1-26, D1-67 | Fixed | Chip rows wrap. |
| D1-27, D1-69 | Fixed | Under 34 rem the queue and the detail take turns with "Back to queue". |
| D1-28, D1-72 | Fixed | List and detail take turns under 640 px with a back control. |
| D1-29 | Fixed | Sticky checkbox and title columns with a visible scrollbar. |
| D1-30, D1-70 | Fixed | Context figure and mode labelled; queued row wraps. |
| D1-31, D1-68 | Fixed | Chips show the index and the full domain and wrap; the headline moves to the preview card. |
| D1-32 | Fixed | One icon family (capital letters); add-step controls 24 px with a 16 px gap. |
| D1-33, D1-51 | Fixed | Allow uses the primary token in every theme; the risky token only for risky requests. |
| D1-34 | Fixed | Allow, Grant and Review use the primary token, so each theme has one primary colour. |
| D1-35, D1-36, D1-37 | Fixed | Title-case status pills with a dot; state visible when narrow. |
| D1-38 | Fixed | Completed items read "Done". |
| D1-39 | Fixed | Group headings are not struck through. |
| D1-40, D1-63 | Fixed | A series keeps its colour across themes. |
| D1-41 | Accepted | The cost tile keeps its column chart: the spec requires that kind. Labels wrap. |
| D1-42 | Fixed | The live bar is labelled "Today". |
| D1-43 | Fixed | "No key yet" with a hint; Reveal disabled until a key exists. |
| D1-44, D1-79 | Fixed | Tabular lining figures on amounts, dates and prices. |
| D1-45 | Fixed | Faint badges no longer dimmed below secondary-text contrast. |
| D1-46 | Fixed | Tool chips show the tool name. |
| D1-47 | Fixed | Floating join squares removed. |
| D1-48 | Fixed | Tile names wrap; the narrow form is a list by design. |
| D1-49 | Fixed | Threshold labelled with its value. |
| D1-50, D1-57 | Fixed | "Sample: 1,200" at body size; variant names wrap. |
| D1-52 | Fixed | Owners table stacks labelled cells under 480 px. The document pane scrolls. |
| D1-53 | Accepted | Badges and chips widen and names wrap; the graph keeps its two-edge form because the spec requires it. |
| D1-54, D1-66 | Fixed | Agent name wraps; the pill drops to its own row when narrow. |
| D1-55 | Fixed | Titles and check names wrap. |
| D1-56 | Fixed | Table scrolls with a sticky first column. |
| D1-58 | Fixed | Labels show the figure; the unit sits on the total. |
| D1-59 | Fixed | Titles wrap; state pill in the kit vocabulary. |
| D1-60 | Fixed | Decision titles wrap; the pill moves below when narrow. |
| D1-61 | Fixed | Level columns size to their labels; "Autonomous" fits at 320 px. |
| D1-64 | Fixed | Item names wrap. |
| D1-73 | Accepted | Connectors pass beneath the chips; the remaining overlap with cards is minor. |
| D1-74 | Fixed | The preview line wraps. |
| D1-75 | Fixed | Legend for added, removed and the zero baseline; a middle tick. |
| D1-76 | Fixed | Axis text is HTML at 11 px; gaps labelled "No data". |
| D1-77 | Fixed | Collapsed notices no longer overlap; the timer sits on the bottom edge. |
| D1-78 | Fixed | Gallery event echoes are labelled "Demo event:" and set off with a rule. |
| D1-80 | Fixed | The trial button uses the primary token. |
| D1-81 | Fixed | Remove is labelled "Remove step N" with a 24 px hit area. |
| D1-82 | Accepted | The applied state is a pill; the chart frame is unchanged. |

### Inputs the review did not check

- **run-timeline-rows transfers.** Transfer markers are now tab stops; a test asserts it.
- **charge-phase-builder edit-state dates.** Accepted: the edit state uses the browser's native date input, whose display format belongs to the browser and locale. The rest state shows "28 February 2027".
