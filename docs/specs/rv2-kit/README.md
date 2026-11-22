# rv2-kit functional specs

Functional specifications for `crates/rv2-kit`, written for a clean-room
rebuild of the functionality of affectUI's 160 Pro items
(decision AF7 in `docs/architecture/005-rv2-affect.md`).

## How these were made

1. **Spec room.** Agents read only affectUI's public component pages and
   observed the live demos as a user would. They did not fetch the paid
   registry, any JS or CSS bundle, or page source. Each spec describes
   purpose, data, anatomy, states, interactions, accessibility and
   responsive behaviour in our own words and with our own names and demo
   data. Specs carry no colours, sizes, timings, CSS, class names, art or
   copied prose. Screenshots taken for reference stayed outside the
   repository.
2. **Clean room.** Separate agents implement `crates/rv2-kit` from these specs
   alone, on rv2's registers and the `rv2-affect` primitives, without access
   to affectUI's site, screenshots or downloaded files.

Each spec names its source item so the mapping is auditable. That line is a
functional reference, not a claim of derivation from affectUI's code.

## Layout

| Folder | What |
|---|---|
| `agents/` | Agent-observability components |
| `components/` | General and data components |
| `screens/` | Full pages composed from components |
| `blocks/` | Page sections |
| `illustrations/` | Animated illustrations, original art |
| `motion/` | Text and surface effects |

## Index

| Source item | Spec | Name |
|---|---|---|
| `components/agent-overlap-map` | [agents/agent-clash-map.md](agents/agent-clash-map.md) | Agent Clash Map |
| `components/agent-inbox` | [agents/background-task-inbox.md](agents/background-task-inbox.md) | Background Task Inbox |
| `components/agent-handoff` | [agents/baton-pass-log.md](agents/baton-pass-log.md) | Baton Pass Log |
| `components/agent-cursors` | [agents/co-editing-presence.md](agents/co-editing-presence.md) | Co-Editing Presence |
| `components/agent-crew` | [agents/delegation-card-tree.md](agents/delegation-card-tree.md) | Delegation Card Tree |
| `components/agent-plan` | [agents/editable-step-plan.md](agents/editable-step-plan.md) | Editable Step Plan |
| `components/agent-sequence` | [agents/exchange-diagram.md](agents/exchange-diagram.md) | Exchange Diagram |
| `components/agent-steering` | [agents/mid-run-note-panel.md](agents/mid-run-note-panel.md) | Mid-Run Note Panel |
| `components/agent-recap` | [agents/overnight-run-digest.md](agents/overnight-run-digest.md) | Overnight Run Digest |
| `components/agent-findings` | [agents/parallel-probe-feed.md](agents/parallel-probe-feed.md) | Parallel Probe Feed |
| `components/agent-lanes` | [agents/parallel-span-timeline.md](agents/parallel-span-timeline.md) | Parallel Span Timeline |
| `components/agent-router` | [agents/request-routing-map.md](agents/request-routing-map.md) | Request Routing Map |
| `components/agent-ledger` | [agents/reversible-change-log.md](agents/reversible-change-log.md) | Reversible Change Log |
| `components/agent-checkpoints` | [agents/run-checkpoint-rail.md](agents/run-checkpoint-rail.md) | Run Checkpoint Rail |
| `components/handoff-swimlanes` | [agents/run-process-lanes.md](agents/run-process-lanes.md) | Run Process Lanes |
| `components/agent-timing` | [agents/run-timeline-rows.md](agents/run-timeline-rows.md) | Run Timeline Rows |
| `components/agent-workbench` | [agents/single-agent-stage.md](agents/single-agent-stage.md) | Single Agent Stage |
| `components/agent-monitor` | [agents/single-run-status-card.md](agents/single-run-status-card.md) | Single Run Status Card |
| `components/agent-trail` | [agents/step-progress-list.md](agents/step-progress-list.md) | Step Progress List |
| `components/agent-tree` | [agents/sub-run-tree.md](agents/sub-run-tree.md) | Sub-Run Tree |
| `components/agent-graph` | [agents/tool-call-network.md](agents/tool-call-network.md) | Tool Call Network |
| `components/agent-queue` | [agents/work-queue-flow.md](agents/work-queue-flow.md) | Work Queue Flow |
| `components/agent-fleet` | [agents/worker-fleet-panel.md](agents/worker-fleet-panel.md) | Worker Fleet Panel |
| `blocks/mission-control` | [blocks/agent-fleet-overview.md](blocks/agent-fleet-overview.md) | Agent Fleet Overview |
| `blocks/live-session` | [blocks/agent-team-session.md](blocks/agent-team-session.md) | Agent Team Session |
| `blocks/mission-timeline` | [blocks/agent-timeline-overview.md](blocks/agent-timeline-overview.md) | Agent Timeline Overview |
| `blocks/revenue-chat` | [blocks/chart-answer-chat.md](blocks/chart-answer-chat.md) | Chart Answer Chat |
| `blocks/client-context` | [blocks/client-workspace.md](blocks/client-workspace.md) | Client Workspace |
| `blocks/agent-gallery` | [blocks/expandable-panel-strip.md](blocks/expandable-panel-strip.md) | Expandable Panel Strip |
| `blocks/app-directory` | [blocks/integration-catalog.md](blocks/integration-catalog.md) | Integration Catalog |
| `blocks/invoice-credit` | [blocks/invoice-credit-pair.md](blocks/invoice-credit-pair.md) | Invoice and Credit Pair |
| `blocks/task-board` | [blocks/kanban-task-board.md](blocks/kanban-task-board.md) | Kanban Task Board |
| `blocks/kpi-board` | [blocks/metric-overview-board.md](blocks/metric-overview-board.md) | Metric Overview Board |
| `blocks/pricing-plans` | [blocks/plan-comparison-section.md](blocks/plan-comparison-section.md) | Plan Comparison Section |
| `blocks/run-inspector` | [blocks/run-tree-inspector.md](blocks/run-tree-inspector.md) | Run Tree Inspector |
| `blocks/metrics-dashboard` | [blocks/sales-metrics-panel.md](blocks/sales-metrics-panel.md) | Sales Metrics Panel |
| `components/away-summary` | [components/absence-digest.md](components/absence-digest.md) | Absence Digest |
| `components/permission-request` | [components/access-grant-prompt.md](components/access-grant-prompt.md) | Access Grant Prompt |
| `components/memory-graph` | [components/agent-context-map.md](components/agent-context-map.md) | Agent Context Map |
| `components/decision-card` | [components/agent-question-card.md](components/agent-question-card.md) | Agent Question Card |
| `components/tool-access-map` | [components/agent-tool-reach-map.md](components/agent-tool-reach-map.md) | Agent Tool Reach Map |
| `components/comments-panel` | [components/annotation-sidebar.md](components/annotation-sidebar.md) | Annotation Sidebar |
| `components/memory-panel` | [components/assistant-memory-manager.md](components/assistant-memory-manager.md) | Assistant Memory Manager |
| `components/flow-builder` | [components/automation-flow-editor.md](components/automation-flow-editor.md) | Automation Flow Editor |
| `components/running-now` | [components/background-jobs-popover.md](components/background-jobs-popover.md) | Background Jobs Popover |
| `components/decision-tree` | [components/branch-choice-map.md](components/branch-choice-map.md) | Branch Choice Map |
| `components/parallel-attempts` | [components/candidate-race-board.md](components/candidate-race-board.md) | Candidate Race Board |
| `components/root-cause-tree` | [components/cause-analysis-tree.md](components/cause-analysis-tree.md) | Cause Analysis Tree |
| `components/commit-trail` | [components/change-log-timeline.md](components/change-log-timeline.md) | Change Log Timeline |
| `components/diff-timeline` | [components/change-replay.md](components/change-replay.md) | Change Replay |
| `components/billing-schedule` | [components/charge-phase-builder.md](components/charge-phase-builder.md) | Charge Phase Builder |
| `components/integration-picker` | [components/connector-picker.md](components/connector-picker.md) | Connector Picker |
| `components/integration-settings` | [components/connector-settings-card.md](components/connector-settings-card.md) | Connector Settings Card |
| `components/approval-card` | [components/consent-request-card.md](components/consent-request-card.md) | Consent Request Card |
| `components/context-window` | [components/context-fill-grid.md](components/context-fill-grid.md) | Context Fill Grid |
| `components/context-map` | [components/context-treemap.md](components/context-treemap.md) | Context Treemap |
| `components/metering-chart` | [components/daily-usage-bars.md](components/daily-usage-bars.md) | Daily Usage Bars |
| `components/review-center` | [components/decision-review-queue.md](components/decision-review-queue.md) | Decision Review Queue |
| `components/reasoning-summary` | [components/deliberation-digest.md](components/deliberation-digest.md) | Deliberation Digest |
| `components/project-card` | [components/expandable-work-item-card.md](components/expandable-work-item-card.md) | Expandable Work Item Card |
| `components/dropzone` | [components/file-drop-area.md](components/file-drop-area.md) | File Drop Area |
| `components/prompt-library` | [components/fill-in-prompt-templates.md](components/fill-in-prompt-templates.md) | Fill-in Prompt Templates |
| `components/ai-answer` | [components/grounded-reply.md](components/grounded-reply.md) | Grounded Reply |
| `components/writing-assistant` | [components/inline-drafting-assistant.md](components/inline-drafting-assistant.md) | Inline Drafting Assistant |
| `components/inline-diff` | [components/inline-edit-review.md](components/inline-edit-review.md) | Inline Edit Review |
| `components/collaboration-stream` | [components/inter-agent-message-feed.md](components/inter-agent-message-feed.md) | Inter-Agent Message Feed |
| `components/response-stream` | [components/interruptible-answer-stream.md](components/interruptible-answer-stream.md) | Interruptible Answer Stream |
| `components/stat-card` | [components/metric-tile.md](components/metric-tile.md) | Metric Tile |
| `components/kpi-chart` | [components/metric-trend-card.md](components/metric-trend-card.md) | Metric Trend Card |
| `components/fallback-chain` | [components/model-failover-flow.md](components/model-failover-flow.md) | Model Failover Flow |
| `components/model-picker` | [components/model-selector.md](components/model-selector.md) | Model Selector |
| `components/combobox` | [components/multi-select-lookup.md](components/multi-select-lookup.md) | Multi-Select Lookup |
| `components/stepper-form` | [components/multi-step-wizard-form.md](components/multi-step-wizard-form.md) | Multi-step Wizard Form |
| `components/setup-guide` | [components/onboarding-checklist-panel.md](components/onboarding-checklist-panel.md) | Onboarding Checklist Panel |
| `components/conflict-radar` | [components/parallel-edit-overlap-grid.md](components/parallel-edit-overlap-grid.md) | Parallel Edit Overlap Grid |
| `components/fan-out-grid` | [components/parallel-work-grid.md](components/parallel-work-grid.md) | Parallel Work Grid |
| `components/compact-breakdown` | [components/part-of-total-card.md](components/part-of-total-card.md) | Part-of-Total Card |
| `components/approval-gates` | [components/pending-decision-stack.md](components/pending-decision-stack.md) | Pending Decision Stack |
| `components/date-range-picker` | [components/period-picker.md](components/period-picker.md) | Period Picker |
| `components/chat-composer` | [components/prompt-input-bar.md](components/prompt-input-bar.md) | Prompt Input Bar |
| `components/model-usage` | [components/provider-usage-chart.md](components/provider-usage-chart.md) | Provider Usage Chart |
| `components/command-menu` | [components/quick-action-palette.md](components/quick-action-palette.md) | Quick Action Palette |
| `components/rate-limit-planner` | [components/quota-load-scheduler.md](components/quota-load-scheduler.md) | Quota Load Scheduler |
| `components/usage-meter` | [components/quota-usage-panel.md](components/quota-usage-panel.md) | Quota Usage Panel |
| `components/judge-bracket` | [components/ranking-bracket.md](components/ranking-bracket.md) | Ranking Bracket |
| `components/run-controls` | [components/run-control-composer.md](components/run-control-composer.md) | Run Control Composer |
| `components/cost-waterfall` | [components/run-cost-breakdown.md](components/run-cost-breakdown.md) | Run Cost Breakdown |
| `components/trace-waterfall` | [components/run-timing-waterfall.md](components/run-timing-waterfall.md) | Run Timing Waterfall |
| `components/plan-vs-actual` | [components/schedule-variance-timeline.md](components/schedule-variance-timeline.md) | Schedule Variance Timeline |
| `components/breakdown-chart` | [components/segmented-bar-explorer.md](components/segmented-bar-explorer.md) | Segmented Bar Explorer |
| `components/trend-chart` | [components/series-trend-chart.md](components/series-trend-chart.md) | Series Trend Chart |
| `components/scheduled-runs` | [components/session-packing-planner.md](components/session-packing-planner.md) | Session Packing Planner |
| `components/data-table` | [components/sheet-grid.md](components/sheet-grid.md) | Sheet Grid |
| `components/cited-answer` | [components/source-linked-answer.md](components/source-linked-answer.md) | Source-Linked Answer |
| `components/delegate-board` | [components/task-assignment-board.md](components/task-assignment-board.md) | Task Assignment Board |
| `components/model-router` | [components/traffic-split-router.md](components/traffic-split-router.md) | Traffic Split Router |
| `components/toaster` | [components/transient-notice-stack.md](components/transient-notice-stack.md) | Transient Notice Stack |
| `components/autonomy-dial` | [components/trust-level-selector.md](components/trust-level-selector.md) | Trust Level Selector |
| `components/experiment-card` | [components/variant-test-card.md](components/variant-test-card.md) | Variant Test Card |
| `components/thinking-state` | [components/working-trail-panel.md](components/working-trail-panel.md) | Working Trail Panel |
| `components/repo-map` | [components/workspace-territory-map.md](components/workspace-territory-map.md) | Workspace Territory Map |
| `illustrations/world-map` | [illustrations/continent-highlight-map.md](illustrations/continent-highlight-map.md) | Continent Highlight Map |
| `illustrations/voice-capture` | [illustrations/dictation-capture-panel.md](illustrations/dictation-capture-panel.md) | Dictation Capture Panel |
| `illustrations/glass-folder` | [illustrations/frosted-folder-mark.md](illustrations/frosted-folder-mark.md) | Frosted Folder Mark |
| `illustrations/inspect-agent-card` | [illustrations/inspected-agent-card-scene.md](illustrations/inspected-agent-card-scene.md) | Inspected Agent Card Scene |
| `illustrations/inspect-composer` | [illustrations/inspected-composer-scene.md](illustrations/inspected-composer-scene.md) | Inspected Composer Scene |
| `illustrations/integration-flow` | [illustrations/integration-arc-flow.md](illustrations/integration-arc-flow.md) | Integration Arc Flow |
| `illustrations/glow-keyboard` | [illustrations/keyboard-prompt-input.md](illustrations/keyboard-prompt-input.md) | Keyboard Prompt Input |
| `illustrations/model-menu` | [illustrations/model-picker-scene.md](illustrations/model-picker-scene.md) | Model Picker Scene |
| `illustrations/outreach-branches` | [illustrations/outreach-fanout-lines.md](illustrations/outreach-fanout-lines.md) | Outreach Fanout Lines |
| `illustrations/progress-pill` | [illustrations/progress-pill-scene.md](illustrations/progress-pill-scene.md) | Progress Pill Scene |
| `illustrations/project-flow` | [illustrations/task-branch-scene.md](illustrations/task-branch-scene.md) | Task Branch Scene |
| `illustrations/voice-orb` | [illustrations/voice-assistant-orb.md](illustrations/voice-assistant-orb.md) | Voice Assistant Orb |
| `motion/blur-crossfade` | [motion/blur-swap-container.md](motion/blur-swap-container.md) | Blur Swap Container |
| `motion/gradient-text` | [motion/flowing-gradient-text.md](motion/flowing-gradient-text.md) | Flowing Gradient Text |
| `motion/gradient-floor` | [motion/glow-floor-backdrop.md](motion/glow-floor-backdrop.md) | Glow Floor Backdrop |
| `motion/tilt-card` | [motion/pointer-tilt-card.md](motion/pointer-tilt-card.md) | Pointer Tilt Card |
| `screens/agent-activity` | [screens/agent-action-log.md](screens/agent-action-log.md) | Agent Action Log |
| `screens/agent-memory` | [screens/agent-memory-manager.md](screens/agent-memory-manager.md) | Agent Memory Manager |
| `screens/agents` | [screens/agent-roster.md](screens/agent-roster.md) | Agent Roster |
| `screens/agent-run-detail` | [screens/agent-run-timeline.md](screens/agent-run-timeline.md) | Agent Run Timeline |
| `screens/agent-schedules` | [screens/agent-schedule-board.md](screens/agent-schedule-board.md) | Agent Schedule Board |
| `screens/agent-skills` | [screens/agent-skill-library.md](screens/agent-skill-library.md) | Agent Skill Library |
| `screens/agent-usage` | [screens/agent-spend-overview.md](screens/agent-spend-overview.md) | Agent Spend Overview |
| `screens/agent-inbox-board` | [screens/agent-task-board.md](screens/agent-task-board.md) | Agent Task Board |
| `screens/agent-tasks` | [screens/agent-task-list.md](screens/agent-task-list.md) | Agent Task List |
| `screens/agent-tool-calls` | [screens/agent-tool-explorer.md](screens/agent-tool-explorer.md) | Agent Tool Explorer |
| `screens/ai-chat-spend` | [screens/ai-spend-chat-planner.md](screens/ai-spend-chat-planner.md) | AI Spend Chat Planner |
| `screens/assistant-overview` | [screens/ai-workspace-home.md](screens/ai-workspace-home.md) | AI Workspace Home |
| `screens/agent-chat-progress` | [screens/assistant-chat-progress-panel.md](screens/assistant-chat-progress-panel.md) | Assistant Chat With Progress Panel |
| `screens/agent-chat-rail` | [screens/assistant-chat-sidebars.md](screens/assistant-chat-sidebars.md) | Assistant Chat With Sidebars |
| `screens/onboarding-brand` | [screens/assistant-naming-step.md](screens/assistant-naming-step.md) | Assistant Naming Step |
| `screens/onboarding-tone` | [screens/assistant-persona-step.md](screens/assistant-persona-step.md) | Assistant Persona Step |
| `screens/onboarding-ready` | [screens/assistant-ready-step.md](screens/assistant-ready-step.md) | Assistant Ready Step |
| `screens/assistant-home` | [screens/assistant-start-page.md](screens/assistant-start-page.md) | Assistant Start Page |
| `screens/workflow-builder` | [screens/automation-canvas.md](screens/automation-canvas.md) | Automation Canvas Screen |
| `screens/agent-run` | [screens/autonomous-run-workspace.md](screens/autonomous-run-workspace.md) | Autonomous Run Workspace |
| `screens/agent-permissions` | [screens/autonomy-settings.md](screens/autonomy-settings.md) | Agent Autonomy Settings |
| `screens/crm` | [screens/company-register.md](screens/company-register.md) | Company Register Screen |
| `screens/integrations` | [screens/connected-apps-catalogue.md](screens/connected-apps-catalogue.md) | Connected Apps Catalogue Screen |
| `screens/inbox` | [screens/decision-queue.md](screens/decision-queue.md) | Decision Queue |
| `screens/files` | [screens/file-library.md](screens/file-library.md) | File Library Screen |
| `screens/folder` | [screens/folder-detail.md](screens/folder-detail.md) | Folder Detail Screen |
| `screens/kanban` | [screens/issue-board-screen.md](screens/issue-board-screen.md) | Issue Board Screen |
| `screens/onboarding-website` | [screens/knowledge-source-step.md](screens/knowledge-source-step.md) | Knowledge Source Step |
| `screens/agent-orchestration` | [screens/multi-agent-run-canvas.md](screens/multi-agent-run-canvas.md) | Multi-Agent Run Canvas |
| `screens/agent-thread` | [screens/orchestrator-thread.md](screens/orchestrator-thread.md) | Orchestrator Thread |
| `screens/evals` | [screens/prompt-test-board.md](screens/prompt-test-board.md) | Prompt Test Board Screen |
| `screens/ai-chat-revenue` | [screens/revenue-chat-analyst.md](screens/revenue-chat-analyst.md) | Revenue Chat Analyst |
| `screens/analytics` | [screens/revenue-overview.md](screens/revenue-overview.md) | Revenue Overview Screen |
| `screens/settings-subscription` | [screens/settings-billing.md](screens/settings-billing.md) | Settings: Billing |
| `screens/settings-members` | [screens/settings-members-teams.md](screens/settings-members-teams.md) | Settings: Members and Teams |
| `screens/settings-models` | [screens/settings-model-access.md](screens/settings-model-access.md) | Settings: Model Access |
| `screens/settings-notifications` | [screens/settings-notification-rules.md](screens/settings-notification-rules.md) | Settings: Notification Rules |
| `screens/settings-preferences` | [screens/settings-personal-preferences.md](screens/settings-personal-preferences.md) | Settings: Personal Preferences |
| `screens/settings-privacy` | [screens/settings-privacy-controls.md](screens/settings-privacy-controls.md) | Settings: Privacy Controls |
| `screens/settings-profile` | [screens/settings-profile.md](screens/settings-profile.md) | Settings: Profile |
| `screens/settings-general` | [screens/settings-workspace-general.md](screens/settings-workspace-general.md) | Settings: Workspace General |
| `screens/settings-security` | [screens/settings-workspace-security.md](screens/settings-workspace-security.md) | Settings: Workspace Security |
| `screens/agent-chat` | [screens/workspace-chat.md](screens/workspace-chat.md) | Workspace Chat |
| `screens/app-shell-flat` | [screens/workspace-frame-flat.md](screens/workspace-frame-flat.md) | Workspace Frame (Flat) |
| `screens/app-shell` | [screens/workspace-frame-inset.md](screens/workspace-frame-inset.md) | Workspace Frame (Inset) |
| `screens/onboarding` | [screens/workspace-setup-checklist.md](screens/workspace-setup-checklist.md) | Workspace Setup Checklist Screen |
