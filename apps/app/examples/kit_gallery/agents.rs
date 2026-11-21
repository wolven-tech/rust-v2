use dioxus::prelude::*;

use super::{Item, NOW_MS, Section, gallery_now};

#[component]
pub fn Agents() -> Element {
    rsx! {
        Section { id: "agents", title: "Agents",
            ParallelSpanTimelineDemo {}
            ExchangeDiagramDemo {}
            DelegationCardTreeDemo {}
            BackgroundTaskInboxDemo {}
            OvernightRunDigestDemo {}
            MidRunNotePanelDemo {}
            StepProgressListDemo {}
            SubRunTreeDemo {}
            ReversibleChangeLogDemo {}
            SingleAgentStageDemo {}
            RequestRoutingMapDemo {}
            WorkerFleetPanelDemo {}
            ToolCallNetworkDemo {}
            WorkQueueFlowDemo {}
            BatonPassLogDemo {}
            SingleRunStatusCardDemo {}
            ParallelProbeFeedDemo {}
            CoEditingPresenceDemo {}
            RunTimelineRowsDemo {}
            RunCheckpointRailDemo {}
            EditableStepPlanDemo {}
        }
    }
}

#[component]
fn ParallelSpanTimelineDemo() -> Element {
    use rv2_kit::agents::parallel_span_timeline::{
        ParallelSpanTimeline, Unit, UnitPhase, WorkerRow,
    };
    let mut last = use_signal(String::new);
    let unit =
        |key: &str, worker: &str, label: &str, phase: UnitPhase, begin: u64, end: Option<u64>| {
            Unit {
                unit_key: key.into(),
                worker_key: worker.into(),
                unit_label: label.into(),
                phase,
                begins_at_ms: begin,
                ends_at_ms: end,
                hands_off_to_key: None,
                decision_prompt: None,
                highlights: vec![],
            }
        };
    let mut held = unit(
        "u4",
        "w2",
        "Draft migration",
        UnitPhase::Held,
        4_000,
        Some(9_000),
    );
    held.decision_prompt = Some("Apply schema change?".into());
    let units = vec![
        unit(
            "u1",
            "w1",
            "Read ledger",
            UnitPhase::Settled,
            0,
            Some(3_000),
        ),
        unit("u2", "w1", "Summarise", UnitPhase::Active, 3_000, None),
        unit(
            "u3",
            "w2",
            "Scan repo",
            UnitPhase::Settled,
            1_000,
            Some(6_000),
        ),
        held,
        unit(
            "u5",
            "w3",
            "Run tests",
            UnitPhase::Broken,
            2_000,
            Some(7_500),
        ),
    ];
    let workers = vec![
        WorkerRow {
            worker_key: "w1".into(),
            worker_label: "Reader".into(),
            model_label: Some("model A".into()),
        },
        WorkerRow {
            worker_key: "w2".into(),
            worker_label: "Scout".into(),
            model_label: None,
        },
        WorkerRow {
            worker_key: "w3".into(),
            worker_label: "Tester".into(),
            model_label: Some("model B".into()),
        },
    ];
    rsx! {
        Item { slug: "parallel-span-timeline", name: "ParallelSpanTimeline",
            ParallelSpanTimeline {
                worker_rows: workers,
                units,
                window_span: 20_000,
                clock_ms: Some(gallery_now() - NOW_MS + 10_000),
                on_confirm: move |u: Unit| last.set(format!("confirm {}", u.unit_key)),
                on_decline: move |u: Unit| last.set(format!("decline {}", u.unit_key)),
                on_details: move |u: Unit| last.set(format!("details {}", u.unit_key)),
            }
            div { class: "gl-out", "data-out": "parallel-span-timeline", "{last()}" }
        }
    }
}

#[component]
fn ExchangeDiagramDemo() -> Element {
    use rv2_kit::agents::exchange_diagram::{
        Actor, ActorRole, Exchange, ExchangeDiagram, ExchangeKind, ExchangeState,
    };
    let mut last = use_signal(String::new);
    let actors = vec![
        Actor {
            actor_key: "p".into(),
            role: ActorRole::Person,
            caption: "You".into(),
            mark_source: None,
            tint_key: None,
            model_tag: None,
            summary: None,
        },
        Actor {
            actor_key: "a".into(),
            role: ActorRole::Agent,
            caption: "Planner".into(),
            mark_source: None,
            tint_key: Some("blue".into()),
            model_tag: Some("model A".into()),
            summary: Some("Breaks work down".into()),
        },
        Actor {
            actor_key: "t".into(),
            role: ActorRole::Tool,
            caption: "Search".into(),
            mark_source: None,
            tint_key: None,
            model_tag: None,
            summary: None,
        },
    ];
    let ex = |key: &str,
              from: &str,
              to: &str,
              kind: ExchangeKind,
              state: ExchangeState,
              headline: &str| Exchange {
        exchange_key: key.into(),
        from_key: from.into(),
        to_key: to.into(),
        kind,
        answers_key: None,
        state,
        sent_at_ms: key[1..].parse::<i64>().ok().map(|n| n * 20_000),
        group_tag: None,
        headline: headline.into(),
        detail_text: None,
    };
    let exchanges = vec![
        ex(
            "e1",
            "p",
            "a",
            ExchangeKind::Request,
            ExchangeState::Ok,
            "Plan the release notes",
        ),
        ex(
            "e2",
            "a",
            "t",
            ExchangeKind::Request,
            ExchangeState::Failed,
            "Query changelog",
        ),
        ex(
            "e3",
            "t",
            "a",
            ExchangeKind::Answer,
            ExchangeState::Ok,
            "12 commits found",
        ),
        ex(
            "e4",
            "a",
            "p",
            ExchangeKind::Request,
            ExchangeState::AwaitingPerson,
            "Approve the outline?",
        ),
    ];
    rsx! {
        Item { slug: "exchange-diagram", name: "ExchangeDiagram",
            ExchangeDiagram {
                actors,
                exchanges,
                run_heading: "Release run",
                on_pick: move |e: Option<Exchange>| last.set(format!("pick {}", e.map(|x| x.exchange_key).unwrap_or("none".into()))),
            }
            div { class: "gl-out", "data-out": "exchange-diagram", "{last()}" }
        }
    }
}

fn agent(
    id: &str,
    name: &str,
    state: rv2_kit::agents::delegation_card_tree::AgentState,
) -> rv2_kit::agents::delegation_card_tree::Agent {
    use rv2_kit::agents::delegation_card_tree::Agent;
    Agent {
        id: id.into(),
        name: name.into(),
        state,
        doing_text: None,
        load_ratio: None,
        load_capacity_tokens: None,
        load_used_tokens: None,
        model_label: None,
        model_mark: None,
        tool_names: vec![],
        last_transfer: None,
        assignment: None,
        started_at: None,
        ended_at: None,
        spend: None,
        tokens: None,
        identity_mark: None,
        sub_agents: Vec::<Agent>::new(),
    }
}

#[component]
fn DelegationCardTreeDemo() -> Element {
    use rv2_kit::agents::delegation_card_tree::{
        AgentState, CommandAction, DelegationCardTree, Transfer, Workspace,
    };
    let mut last = use_signal(String::new);
    let mut workspace = use_signal(|| Some("main".to_string()));
    let mut writer = agent("a2", "Writer", AgentState::Waiting);
    writer.doing_text = Some("Waiting for the outline".into());
    writer.last_transfer = Some(rv2_kit::agents::delegation_card_tree::LastTransfer {
        from_name: "Orchestrator".into(),
        text: "Outline v2".into(),
    });
    let mut reviewer = agent("a3", "Reviewer", AgentState::Failed);
    reviewer.doing_text = Some("Tests failed twice".into());
    reviewer.load_ratio = Some(0.97);
    let mut top = agent("a1", "Orchestrator", AgentState::Active);
    top.doing_text = Some("Coordinating the release".into());
    top.load_ratio = Some(0.62);
    top.spend = Some(1.84);
    top.sub_agents = vec![writer, reviewer];
    let transfers = vec![Transfer {
        id: "t1".into(),
        source_id: "a1".into(),
        target_id: "a2".into(),
        note: Some("Outline v2".into()),
    }];
    let workspaces = vec![
        Workspace {
            id: "main".into(),
            name: "main".into(),
            active_count: Some(3),
            attention_count: Some(1),
        },
        Workspace {
            id: "docs".into(),
            name: "docs".into(),
            active_count: Some(0),
            attention_count: None,
        },
    ];
    rsx! {
        Item { slug: "delegation-card-tree", name: "DelegationCardTree",
            DelegationCardTree {
                top_agent: top,
                transfers,
                workspaces,
                active_workspace: workspace(),
                on_workspace_change: move |w: String| {
                    last.set(format!("workspace {w}"));
                    workspace.set(Some(w));
                },
                on_open: move |a: rv2_kit::agents::delegation_card_tree::Agent| last.set(format!("open {}", a.name)),
                on_command: move |(a, c, _): (rv2_kit::agents::delegation_card_tree::Agent, CommandAction, Option<String>)| last.set(format!("{c:?} {}", a.name)),
                clock_ms: Some(gallery_now()),
            }
            div { class: "gl-out", "data-out": "delegation-card-tree", "{last()}" }
        }
    }
}

#[component]
fn BackgroundTaskInboxDemo() -> Element {
    use rv2_kit::agents::background_task_inbox::{
        BackgroundTaskInbox, InboxLayout, Job, JobStage, TimelineStep,
    };
    let mut last = use_signal(String::new);
    let job = |id: &str, heading: &str, stage: JobStage, elapsed: u64| Job {
        id: id.into(),
        heading_text: heading.into(),
        summary_text: Some("Reading 14 files for the migration plan".into()),
        owner_id: "agent-1".into(),
        stage,
        started_at: Some(NOW_MS - elapsed),
        elapsed: Some(elapsed),
        tool_count: Some(4),
        source_count: Some(9),
        timeline: vec![TimelineStep {
            label: "Started".into(),
            at: Some(NOW_MS - elapsed),
        }],
    };
    let jobs = vec![
        job("j1", "Index the docs folder", JobStage::Running, 42_000),
        job(
            "j2",
            "Confirm delete of staging rows",
            JobStage::Waiting,
            9_000,
        ),
        job("j3", "Export usage report", JobStage::Done, 120_000),
        job("j4", "Sync billing events", JobStage::Failed, 15_000),
    ];
    rsx! {
        Item { slug: "background-task-inbox", name: "BackgroundTaskInbox",
            BackgroundTaskInbox {
                jobs,
                initial_layout: InboxLayout::Board,
                clock_ms: Some(gallery_now()),
                on_confirm: move |j: Job| last.set(format!("confirm {}", j.id)),
                on_decline: move |j: Job| last.set(format!("decline {}", j.id)),
                on_restart: move |j: Job| last.set(format!("restart {}", j.id)),
                on_inspect: move |j: Job| last.set(format!("inspect {}", j.id)),
                on_layout_change: move |l: InboxLayout| last.set(format!("layout {l:?}")),
            }
            div { class: "gl-out", "data-out": "background-task-inbox", "{last()}" }
        }
    }
}

#[component]
fn OvernightRunDigestDemo() -> Element {
    use rv2_kit::agents::overnight_run_digest::{
        ActFuture, ActionWhich, CardStage, DigestEvent, DigestLane, MainAction, OvernightRunDigest,
    };
    let mut last = use_signal(String::new);
    let mut reverted = use_signal(Vec::<String>::new);
    let ev = |key: &str, hours_ago: u64, lane: DigestLane, title: &str, action: Option<&str>| {
        DigestEvent {
            event_key: key.into(),
            happened_at: Some(NOW_MS - hours_ago * 3_600_000),
            event_lane: lane,
            event_title: title.into(),
            event_detail: Some("Opened by the nightly batch".into()),
            repeat_count: None,
            main_action: action.map(|a| MainAction {
                action_label: a.into(),
            }),
            inspect_available: true,
            past_tense_label: Some("Applied".into()),
            reversible: true,
        }
    };
    let events = vec![
        ev(
            "d1",
            7,
            DigestLane::AwaitingDecision,
            "Merge 3 duplicate contacts",
            Some("Merge"),
        ),
        ev(
            "d2",
            5,
            DigestLane::Errored,
            "Export to CSV failed",
            Some("Retry"),
        ),
        ev(
            "d3",
            3,
            DigestLane::Completed,
            "Archived 18 old threads",
            None,
        ),
    ];
    let _ = CardStage::Awaiting;
    rsx! {
        Item { slug: "overnight-run-digest", name: "OvernightRunDigest",
            OvernightRunDigest {
                digest_events: events,
                away_since: NOW_MS - 8 * 3_600_000,
                now_at: Some(gallery_now()),
                reverted_keys: Some(reverted()),
                on_act: move |(e, w): (DigestEvent, ActionWhich)| {
                    last.set(format!("act {} {w:?}", e.event_key));
                    Box::pin(async { Ok(()) }) as ActFuture
                },
                on_revert: move |e: DigestEvent| {
                    last.set(format!("revert {}", e.event_key));
                    reverted.with_mut(|list| list.push(e.event_key));
                },
                on_reapply: move |e: DigestEvent| {
                    last.set(format!("reapply {}", e.event_key));
                    reverted.with_mut(|list| list.retain(|key| *key != e.event_key));
                },
            }
            div { class: "gl-out", "data-out": "overnight-run-digest", "{last()}" }
        }
    }
}

#[component]
fn MidRunNotePanelDemo() -> Element {
    use rv2_kit::agents::mid_run_note_panel::{
        MidRunNotePanel, Note, PlanStep, RunState, StepPhase,
    };
    let mut last = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let mut queued = use_signal(Vec::<Note>::new);
    let mut state = use_signal(|| RunState::Running);
    let plan = vec![
        PlanStep {
            step_key: "s1".into(),
            label: "Collect sources".into(),
            phase: StepPhase::Done,
            began_at_ms: Some(NOW_MS - 90_000),
            took_ms: Some(42_000),
            step_notes: vec![],
        },
        PlanStep {
            step_key: "s2".into(),
            label: "Draft the summary".into(),
            phase: StepPhase::Active,
            began_at_ms: Some(NOW_MS - 20_000),
            took_ms: None,
            step_notes: vec![],
        },
        PlanStep {
            step_key: "s3".into(),
            label: "Check citations".into(),
            phase: StepPhase::Upcoming,
            began_at_ms: None,
            took_ms: None,
            step_notes: vec![],
        },
    ];
    rsx! {
        Item { slug: "mid-run-note-panel", name: "MidRunNotePanel",
            MidRunNotePanel {
                run_heading: "Summary run",
                run_agent_name: Some("Writer".to_string()),
                run_started_ms: NOW_MS - 120_000,
                run_state: state(),
                clock_ms: Some(gallery_now()),
                plan_steps: plan,
                queued_notes: queued(),
                draft_text: draft(),
                draft_hint: Some("Press Enter to queue".to_string()),
                on_draft_change: move |t: String| draft.set(t),
                on_queue_note: move |t: String| {
                    last.set(format!("queue {t}"));
                    let next = queued.read().len() + 1;
                    queued.write().push(Note { note_key: format!("n{next}"), text: t, placed: None });
                    draft.set(String::new());
                },
                on_apply_note_now: move |(t, _): (String, Option<String>)| {
                    last.set(format!("apply now {t}"));
                    draft.set(String::new());
                },
                on_drop_note: move |k: String| {
                    last.set(format!("drop {k}"));
                    queued.write().retain(|n| n.note_key != k);
                },
                on_run_state_request: move |s: RunState| {
                    last.set(format!("state {s:?}"));
                    state.set(s);
                },
            }
            div { class: "gl-out", "data-out": "mid-run-note-panel", "{last()}" }
        }
    }
}

#[component]
fn StepProgressListDemo() -> Element {
    use rv2_kit::agents::step_progress_list::{
        AgentBadge, ItemState, ProgressItem, ProgressWording, StepProgressList, SubItem, SubState,
        ToolBadge,
    };
    let mut last = use_signal(String::new);
    let items = vec![
        ProgressItem {
            item_key: "p1".into(),
            title_text: "Read the brief".into(),
            item_state: ItemState::Finished,
            took_ms: Some(1_200),
            began_at_ms: None,
            live_note: None,
            needs_approval: false,
            tool_badge: Some(ToolBadge {
                display_name: "Files".into(),
                mark_source: None,
            }),
            approval_request: None,
            sub_items: vec![SubItem {
                sub_key: "s1".into(),
                text: "Open brief.md".into(),
                sub_state: Some(SubState::Finished),
                fact_text: Some("3 sections".into()),
            }],
        },
        ProgressItem {
            item_key: "p2".into(),
            title_text: "Send the email to legal".into(),
            item_state: ItemState::AwaitingApproval,
            took_ms: None,
            began_at_ms: None,
            live_note: Some("Waiting for you".into()),
            needs_approval: true,
            tool_badge: Some(ToolBadge {
                display_name: "Mail".into(),
                mark_source: None,
            }),
            approval_request: Some("Send to legal@example.com?".into()),
            sub_items: vec![],
        },
        ProgressItem {
            item_key: "p3".into(),
            title_text: "Update the tracker".into(),
            item_state: ItemState::Queued,
            took_ms: None,
            began_at_ms: None,
            live_note: None,
            needs_approval: false,
            tool_badge: None,
            approval_request: None,
            sub_items: vec![],
        },
        ProgressItem {
            item_key: "p4".into(),
            title_text: "Draft the summary".into(),
            item_state: ItemState::InProgress,
            took_ms: None,
            began_at_ms: Some(NOW_MS - 20_000),
            live_note: None,
            needs_approval: false,
            tool_badge: Some(ToolBadge {
                display_name: "Files".into(),
                mark_source: None,
            }),
            approval_request: None,
            sub_items: vec![],
        },
        ProgressItem {
            item_key: "p5".into(),
            title_text: "Pick the release date".into(),
            item_state: ItemState::AwaitingPerson,
            took_ms: None,
            began_at_ms: None,
            live_note: Some("Which date?".into()),
            needs_approval: false,
            tool_badge: None,
            approval_request: None,
            sub_items: vec![],
        },
    ];
    rsx! {
        Item { slug: "step-progress-list", name: "StepProgressList",
            StepProgressList {
                list_heading: Some("Steps".to_string()),
                agent_badge: Some(AgentBadge { display_name: "Planner".into(), mark_source: None }),
                progress_items: items,
                clock_ms: Some(gallery_now()),
                wording: ProgressWording::default(),
                on_confirm: move |p: ProgressItem| last.set(format!("confirm {}", p.item_key)),
                on_refuse: move |p: ProgressItem| last.set(format!("refuse {}", p.item_key)),
            }
            div { class: "gl-out", "data-out": "step-progress-list", "{last()}" }
        }
    }
}

#[component]
fn SubRunTreeDemo() -> Element {
    use rv2_kit::agents::sub_run_tree::{Node, NodePhase, ResolveChoice, SubRunTree};
    let mut last = use_signal(String::new);
    let node = |key: &str, caption: &str, phase: NodePhase, branches: Vec<Node>| Node {
        node_key: key.into(),
        caption: caption.into(),
        brief: format!("{caption} brief"),
        model_tag: Some("model A".into()),
        tool_marks: vec![],
        phase,
        spend: 0.12,
        began_at_ms: Some(NOW_MS - 60_000),
        ended_at_ms: None,
        needs_text: None,
        fault_text: None,
        branches,
    };
    let mut errored = node("n3", "Lint pass", NodePhase::Errored, vec![]);
    errored.fault_text = Some("Lint exited 2".into());
    let mut waiting = node(
        "n4",
        "Pick the release date",
        NodePhase::AwaitingInput,
        vec![],
    );
    waiting.needs_text = Some("Which date?".into());
    let mut changelog = node("n2", "Changelog", NodePhase::Complete, vec![]);
    changelog.ended_at_ms = Some(NOW_MS - 30_000);
    let root = node(
        "n1",
        "Release",
        NodePhase::Active,
        vec![changelog, errored, waiting],
    );
    rsx! {
        Item { slug: "sub-run-tree", name: "SubRunTree",
            SubRunTree {
                root_node: Some(root),
                clock_ms: gallery_now(),
                on_node_pick: move |n: Node| last.set(format!("pick {}", n.node_key)),
                on_node_resolve: move |(n, c): (Node, ResolveChoice)| last.set(format!("{c:?} {}", n.node_key)),
                on_folded_change: move |k: Vec<String>| last.set(format!("folded {}", k.len())),
            }
            div { class: "gl-out", "data-out": "sub-run-tree", "{last()}" }
        }
    }
}

#[component]
fn ReversibleChangeLogDemo() -> Element {
    use rv2_kit::agents::reversible_change_log::{
        ChangeRow, ReversibleChangeLog, RowState, ValueChange,
    };
    let _ = RowState::Applied;
    let mut last = use_signal(String::new);
    let mut reverted = use_signal(Vec::<String>::new);
    let rows = vec![
        ChangeRow {
            row_key: "c1".into(),
            verb_label: "Renamed".into(),
            target_label: Some("Q3 plan".into()),
            system_label: "Docs".into(),
            system_mark: None,
            happened_at_ms: NOW_MS - 600_000,
            value_change: Some(ValueChange {
                from_text: "Q3 draft".into(),
                to_text: "Q3 plan".into(),
            }),
            summary_line: None,
            reversible: true,
            built_on_keys: vec![],
        },
        ChangeRow {
            row_key: "c2".into(),
            verb_label: "Deleted".into(),
            target_label: Some("old-export.csv".into()),
            system_label: "Drive".into(),
            system_mark: None,
            happened_at_ms: NOW_MS - 300_000,
            value_change: None,
            summary_line: Some("Moved to trash".into()),
            reversible: true,
            built_on_keys: vec!["c1".into()],
        },
        ChangeRow {
            row_key: "c3".into(),
            verb_label: "Sent".into(),
            target_label: Some("Invoice 204".into()),
            system_label: "Mail".into(),
            system_mark: None,
            happened_at_ms: NOW_MS - 60_000,
            value_change: None,
            summary_line: None,
            reversible: false,
            built_on_keys: vec![],
        },
    ];
    rsx! {
        Item { slug: "reversible-change-log", name: "ReversibleChangeLog",
            ReversibleChangeLog {
                change_rows: rows,
                actor_label: Some("Agent".to_string()),
                clock_ms: Some(gallery_now()),
                reverted_keys: Some(reverted()),
                on_revert: move |k: Vec<String>| {
                    last.set(format!("revert {}", k.join(",")));
                    reverted.with_mut(|list| list.extend(k));
                },
                on_reapply: move |k: Vec<String>| {
                    last.set(format!("reapply {}", k.join(",")));
                    reverted.with_mut(|list| list.retain(|key| !k.contains(key)));
                },
            }
            div { class: "gl-out", "data-out": "reversible-change-log", "{last()}" }
        }
    }
}

#[component]
fn SingleAgentStageDemo() -> Element {
    use rv2_kit::agents::single_agent_stage::{
        DetailRow, FileItem, Requester, SingleAgentStage, ToolGrant, WorkStep, Worker, WorkerStage,
    };
    let mut last = use_signal(String::new);
    let detail = |label: &str, value: &str| DetailRow {
        field_label: label.into(),
        field_value: value.into(),
    };
    let grant =
        |key: &str, name: &str, text: &str, previous: Option<(&str, &str)>, live: Option<&str>| {
            ToolGrant {
                tool_key: key.into(),
                mark_source: None,
                display_name: name.into(),
                grant_text: text.into(),
                previous_action: previous.map(|(action, _)| action.to_string()),
                previous_when: previous.map(|(_, when)| when.to_string()),
                live_action: live.map(|l| l.to_string()),
                detail_rows: vec![detail("Scope", "Workspace")],
            }
        };
    rsx! {
        Item { slug: "single-agent-stage", name: "SingleAgentStage",
            SingleAgentStage {
                tool_grants: vec![
                    grant("search", "Search", "Read-only web search", Some(("Searched pricing", "2 min ago")), Some("Querying competitor sites")),
                    grant("mail", "Mail", "Drafts only", None, None),
                    grant("calendar", "Calendar", "Read only", Some(("Read Friday's agenda", "1 h ago")), None),
                    grant("docs", "Docs", "Comments only", Some(("Left two comments", "12 min ago")), None),
                    grant("sheets", "Sheets", "Read only", None, None),
                    grant("code", "Code", "No commits", Some(("Listed open issues", "5 min ago")), None),
                ],
                requester: Requester { display_name: "Maya".into(), message_text: "Can you tidy the onboarding copy?".into(), photo_url: None },
                worker: Worker {
                    worker_key: "w".into(),
                    display_name: "Copy agent".into(),
                    subtitle_text: Some("Editing".into()),
                    detail_rows: vec![detail("Model", "model A"), detail("Budget", "$2.00")],
                },
                worker_stage: WorkerStage::Working,
                file_items: vec![
                    FileItem { file_key: "f1".into(), display_name: "onboarding.md".into(), mark_source: None, meta_text: Some("4 KB".into()), detail_rows: vec![detail("Lines", "120")] },
                    FileItem { file_key: "f2".into(), display_name: "style-guide.md".into(), mark_source: None, meta_text: None, detail_rows: vec![] },
                ],
                reading_file_key: Some("f1".to_string()),
                work_log: vec![
                    WorkStep { step_key: "k1".into(), text: "Read the style guide".into(), route_key: "r1".into(), in_progress: false },
                    WorkStep { step_key: "k2".into(), text: "Rewrite step two".into(), route_key: "r2".into(), in_progress: true },
                ],
                on_open_change: move |k: Option<String>| last.set(format!("open {k:?}")),
            }
            div { class: "gl-out", "data-out": "single-agent-stage", "{last()}" }
        }
    }
}

#[component]
fn RequestRoutingMapDemo() -> Element {
    use rv2_kit::agents::request_routing_map::{
        Channel, HandlerNode, LayoutMode, RequestRoutingMap, RequestRow, RequestState, RequestTag,
        RouterNode,
    };
    use std::collections::BTreeMap;
    let mut last = use_signal(String::new);
    let router = RouterNode {
        node_key: "router".into(),
        node_label: "Triage".into(),
        model_label: Some("model A".into()),
        reading_text: Some("Reads each request".into()),
        model_mark: None,
    };
    let handlers = vec![
        HandlerNode {
            handler_key: "billing".into(),
            handler_label: "Billing".into(),
            model_label: None,
            handler_state: Some("Idle".into()),
            activity_text: None,
            load_count: Some(1),
            queued_request_keys: vec!["r2".into()],
            is_person: false,
            accent_slot: 1,
            handler_mark: None,
            model_mark: None,
        },
        HandlerNode {
            handler_key: "sam".into(),
            handler_label: "Sam".into(),
            model_label: None,
            handler_state: None,
            activity_text: Some("In a meeting".into()),
            load_count: None,
            queued_request_keys: vec![],
            is_person: true,
            accent_slot: 2,
            handler_mark: None,
            model_mark: None,
        },
    ];
    let row =
        |key: &str, title: &str, state: RequestState, assigned: Option<&str>, certainty: f64| {
            RequestRow {
                request_key: key.into(),
                request_title: title.into(),
                sender_label: Some("customer@example.com".into()),
                channel_kind: Channel::Mail,
                preview_text: Some("Hi, my invoice shows...".into()),
                request_state: state,
                assigned_to_key: assigned.map(|a| a.to_string()),
                certainty_fraction: certainty,
                score_map: BTreeMap::from([
                    ("billing".to_string(), certainty),
                    ("sam".to_string(), 0.31),
                ]),
                reason_text: Some("Mentions invoice".into()),
                tag_list: vec![RequestTag {
                    label: "Invoice".into(),
                    accent_slot: Some(1),
                }],
            }
        };
    let rows = vec![
        row(
            "r1",
            "Duplicate charge",
            RequestState::Assigned,
            Some("billing"),
            0.94,
        ),
        row("r2", "Refund request", RequestState::Sorting, None, 0.56),
        row(
            "r3",
            "Partnership inquiry",
            RequestState::NeedsPerson,
            Some("sam"),
            0.71,
        ),
    ];
    let mut rows_state = use_signal(|| rows);
    rsx! {
        Item { slug: "request-routing-map", name: "RequestRoutingMap",
            RequestRoutingMap {
                router_node: router,
                handler_nodes: handlers,
                request_rows: rows_state(),
                layout_mode: LayoutMode::SideBySide,
                on_reassign: move |(r, to): (RequestRow, String)| {
                    last.set(format!("reassign {} to {to}", r.request_key));
                    rows_state.with_mut(|list| {
                        for row in list.iter_mut().filter(|row| row.request_key == r.request_key) {
                            row.assigned_to_key = Some(to.clone());
                            row.request_state = RequestState::Assigned;
                        }
                    });
                },
                on_open: move |r: RequestRow| last.set(format!("open {}", r.request_key)),
            }
            div { class: "gl-out", "data-out": "request-routing-map", "{last()}" }
        }
    }
}

#[component]
fn WorkerFleetPanelDemo() -> Element {
    use rv2_kit::agents::worker_fleet_panel::{
        FleetState, LeadAgent, RespondAction, SubAgent, ToolMark, WorkItem, WorkState,
        WorkerFleetPanel,
    };
    let mut last = use_signal(String::new);
    let mut expanded = use_signal(|| Some("lead-1".to_string()));
    let sub = |id: &str, name: &str, state: FleetState, question: Option<&str>| SubAgent {
        id: id.into(),
        name: name.into(),
        assignment: format!("{name} task"),
        model_label: Some("model B".into()),
        state,
        work_items: vec![WorkItem {
            label: "Search index".into(),
            state: Some(WorkState::Running),
            tool_name: Some("search".into()),
            duration: Some(4_200),
        }],
        spend: Some(0.08),
        question: question.map(|q| q.to_string()),
        failure_text: None,
    };
    let leads = vec![
        LeadAgent {
            id: "lead-1".into(),
            name: "Research lead".into(),
            assignment: "Competitor scan".into(),
            model_label: Some("model A".into()),
            picture: None,
            tool_marks: vec![ToolMark {
                mark_source: None,
                caption: "Search".into(),
            }],
            state: FleetState::Running,
            spend: Some(0.42),
            started_at_ms: Some(NOW_MS - 300_000),
            ended_at_ms: None,
            sub_agents: vec![
                sub("s1", "Pricing", FleetState::Running, None),
                sub(
                    "s2",
                    "Reviews",
                    FleetState::Waiting,
                    Some("Include app store reviews?"),
                ),
            ],
        },
        LeadAgent {
            id: "lead-2".into(),
            name: "Writer lead".into(),
            assignment: "Draft the report".into(),
            model_label: None,
            picture: None,
            tool_marks: vec![],
            state: FleetState::Queued,
            spend: None,
            started_at_ms: None,
            ended_at_ms: None,
            sub_agents: vec![],
        },
    ];
    rsx! {
        Item { slug: "worker-fleet-panel", name: "WorkerFleetPanel",
            WorkerFleetPanel {
                lead_agents: leads,
                expanded_id: expanded(),
                on_expand_change: move |id: Option<String>| {
                    last.set(format!("expand {id:?}"));
                    expanded.set(id);
                },
                on_pick: move |s: SubAgent| last.set(format!("pick {}", s.name)),
                on_respond: move |(s, a): (SubAgent, RespondAction)| last.set(format!("{a:?} {}", s.name)),
                clock_now_ms: Some(gallery_now()),
            }
            div { class: "gl-out", "data-out": "worker-fleet-panel", "{last()}" }
        }
    }
}

#[component]
fn ToolCallNetworkDemo() -> Element {
    use rv2_kit::agents::tool_call_network::{
        Condition, Entity, Exchange, ExchangeResult, NodeRole, ToolCallNetwork,
    };
    let mut last = use_signal(String::new);
    let ent = |id: &str, name: &str, role: NodeRole, cond: Condition, centre: bool| Entity {
        id: id.into(),
        name: name.into(),
        node_role: role,
        compact_label: None,
        glyph: None,
        summary_line: None,
        condition: cond,
        activity: None,
        spend: Some(0.05),
        is_centre: centre,
    };
    let entities = vec![
        ent("agent", "Agent", NodeRole::Agent, Condition::Working, true),
        ent("search", "Search", NodeRole::Tool, Condition::Done, false),
        ent("db", "Database", NodeRole::Tool, Condition::Error, false),
        ent("user", "You", NodeRole::User, Condition::Waiting, false),
    ];
    let exchanges = vec![
        Exchange {
            id: "x1".into(),
            from_id: "agent".into(),
            to_id: "search".into(),
            sent_at_ms: NOW_MS - 9_000,
            result: ExchangeResult::Ok,
            caption: Some("query".into()),
            latency_ms: Some(640),
        },
        Exchange {
            id: "x2".into(),
            from_id: "agent".into(),
            to_id: "db".into(),
            sent_at_ms: NOW_MS - 3_000,
            result: ExchangeResult::Error,
            caption: Some("select".into()),
            latency_ms: None,
        },
    ];
    rsx! {
        Item { slug: "tool-call-network", name: "ToolCallNetwork",
            ToolCallNetwork {
                entities,
                links: vec![("agent".to_string(), "search".to_string()), ("agent".to_string(), "db".to_string()), ("agent".to_string(), "user".to_string())],
                exchanges,
                clock_now_ms: Some(gallery_now()),
                on_focus_node: move |e: Option<Entity>| last.set(format!("focus {}", e.map(|x| x.id).unwrap_or("none".into()))),
            }
            div { class: "gl-out", "data-out": "tool-call-network", "{last()}" }
        }
    }
}

#[component]
fn WorkQueueFlowDemo() -> Element {
    use rv2_kit::agents::work_queue_flow::{
        JobKind, JobRow, JobState, Priority, SlotRow, ThrottleWindow, WorkQueueFlow,
    };
    let mut last = use_signal(String::new);
    let mut paused = use_signal(|| false);
    let mut slots = use_signal(|| 3usize);
    let now = gallery_now();
    let job = |key: &str, label: &str, state: JobState, kind: JobKind, pri: Priority| JobRow {
        job_key: key.into(),
        job_label: label.into(),
        job_kind: kind,
        started_at_ms: matches!(state, JobState::Active).then_some(NOW_MS - 60_000),
        job_state: state,
        enqueued_at_ms: NOW_MS - 120_000,
        ended_at_ms: None,
        slot_key: Some("slot-1".into()),
        lane_priority: pri,
        progress_fraction: Some(0.4),
        expected_run_ms: Some(30_000),
        attempt_number: 1,
        max_attempts: Some(3),
        detail_text: None,
        job_mark: None,
        error_text: None,
    };
    let jobs = vec![
        job(
            "j1",
            "Summarise thread",
            JobState::Active,
            JobKind::ModelCall,
            Priority::Urgent,
        ),
        job(
            "j2",
            "Fetch invoices",
            JobState::Enqueued,
            JobKind::ToolCall,
            Priority::Standard,
        ),
        job(
            "j3",
            "Review refund",
            JobState::Enqueued,
            JobKind::HumanReview,
            Priority::Deferred,
        ),
    ];
    let slot_rows: Vec<SlotRow> = (1..=slots())
        .map(|n| SlotRow {
            slot_key: format!("slot-{n}"),
            slot_label: format!("Slot {n}"),
            slot_detail: Some("model A".into()),
        })
        .collect();
    rsx! {
        Item { slug: "work-queue-flow", name: "WorkQueueFlow",
            WorkQueueFlow {
                queue_title: "Agent queue",
                queue_name: Some("default".to_string()),
                job_rows: jobs,
                slot_rows,
                clock_ms: Some(now),
                throttle_windows: vec![ThrottleWindow { from_ms: now - 30_000, until_ms: now + 90_000, reason_label: Some("Rate limit".into()) }],
                is_paused: Some(paused()),
                on_paused_change: move |p: bool| {
                    last.set(format!("paused {p}"));
                    paused.set(p);
                },
                on_slot_count_change: move |n: usize| {
                    last.set(format!("slots {n}"));
                    slots.set(n);
                },
                on_promote: move |j: JobRow| last.set(format!("promote {}", j.job_key)),
                on_withdraw: move |j: JobRow| last.set(format!("withdraw {}", j.job_key)),
                backlog_line: Some(2),
            }
            div { class: "gl-out", "data-out": "work-queue-flow", "{last()} slots={slots()}" }
        }
    }
}

#[component]
fn BatonPassLogDemo() -> Element {
    use rv2_kit::agents::baton_pass_log::{
        Assignment, BatonPassLog, BatonStage, EntryType, PayloadItem, RelayAgent, Transfer,
    };
    let mut last = use_signal(String::new);
    let agents = vec![
        RelayAgent {
            id: "a1".into(),
            name: "Scout".into(),
            model_label: Some("model A".into()),
            role_text: Some("Finds sources".into()),
        },
        RelayAgent {
            id: "a2".into(),
            name: "Writer".into(),
            model_label: None,
            role_text: None,
        },
    ];
    let assignments = vec![
        Assignment {
            id: "as1".into(),
            caption: "Gather sources".into(),
            stage: BatonStage::Done,
            holder_id: Some("a1".into()),
            memo: None,
        },
        Assignment {
            id: "as2".into(),
            caption: "Draft section".into(),
            stage: BatonStage::Working,
            holder_id: Some("a2".into()),
            memo: Some("Use the 2024 numbers".into()),
        },
        Assignment {
            id: "as3".into(),
            caption: "Check citations".into(),
            stage: BatonStage::Todo,
            holder_id: None,
            memo: None,
        },
    ];
    let transfers = vec![Transfer {
        id: "t1".into(),
        from_agent_id: "a1".into(),
        to_agent_id: "a2".into(),
        task_ref: "Gather sources".into(),
        payload: vec![
            PayloadItem {
                id: "p1".into(),
                text: "Pricing page, 2024".into(),
                entry_type: Some(EntryType::Source),
                dropped: false,
                drop_reason: None,
            },
            PayloadItem {
                id: "p2".into(),
                text: "Stale competitor note".into(),
                entry_type: Some(EntryType::Fact),
                dropped: true,
                drop_reason: Some("Older than a year".into()),
            },
        ],
    }];
    let mut assignment_state = use_signal(|| assignments);
    rsx! {
        Item { slug: "baton-pass-log", name: "BatonPassLog",
            BatonPassLog {
                relay_agents: agents,
                assignments: assignment_state(),
                transfers,
                on_hand_over: move |(a, r): (Assignment, RelayAgent)| {
                    last.set(format!("hand over {} to {}", a.id, r.name));
                    assignment_state.with_mut(|list| {
                        for item in list.iter_mut().filter(|item| item.id == a.id) {
                            item.holder_id = Some(r.id.clone());
                        }
                    });
                },
                on_reinstate: move |(t, p): (Transfer, PayloadItem)| last.set(format!("reinstate {} {}", t.id, p.id)),
            }
            div { class: "gl-out", "data-out": "baton-pass-log", "{last()}" }
        }
    }
}

#[component]
fn SingleRunStatusCardDemo() -> Element {
    use rv2_kit::agents::single_run_status_card::{
        PendingRequest, RunPhase, SingleRunStatusCard, StepKind, StepRow, StepState, WorkerIdentity,
    };
    let mut last = use_signal(String::new);
    let mut phase = use_signal(|| RunPhase::AwaitingGrant);
    let step = |key: &str, label: &str, state: StepState| StepRow {
        step_key: key.into(),
        step_label: label.into(),
        step_kind: Some(StepKind::Inspect),
        step_state: state,
        started_at_ms: Some(NOW_MS - 30_000),
        elapsed_ms: Some(2_400),
        tool_mark: None,
        tool_label: Some("Files".into()),
        detail_text: None,
        target_text: Some("src/lib.rs".into()),
    };
    rsx! {
        Item { slug: "single-run-status-card", name: "SingleRunStatusCard",
            SingleRunStatusCard {
                run_phase: phase(),
                worker_identity: WorkerIdentity { display_label: "Refactor agent".into(), role_label: Some("Coder".into()), mark: None },
                step_rows: vec![step("s1", "Open module", StepState::Succeeded), step("s2", "Edit call sites", StepState::InProgress)],
                started_at_ms: Some(NOW_MS - 120_000),
                spend_usd: Some(0.37),
                pending_request: Some(PendingRequest { request_title: "Write to src/lib.rs".into(), request_command: Some("cargo fmt".into()) }),
                focus_label: Some("Refactor".to_string()),
                on_grant: move |_| { last.set("grant".into()); phase.set(RunPhase::Active); },
                on_refuse: move |_| { last.set("refuse".into()); phase.set(RunPhase::Stopped); },
                on_hold: move |_| { last.set("hold".into()); phase.set(RunPhase::Held); },
                on_resume: move |_| { last.set("resume".into()); phase.set(RunPhase::Active); },
                on_stop: move |_| { last.set("stop".into()); phase.set(RunPhase::Stopped); },
                on_retry: move |_| last.set("retry".into()),
                on_open_log: move |_| last.set("open log".into()),
            }
            div { class: "gl-out", "data-out": "single-run-status-card", "{last()}" }
        }
    }
}

#[component]
fn ParallelProbeFeedDemo() -> Element {
    use rv2_kit::agents::parallel_probe_feed::{
        FeedResult, ParallelProbeFeed, ProbeWorker, Severity, SeverityCounts, WorkerState,
    };
    let mut last = use_signal(String::new);
    let workers = vec![
        ProbeWorker {
            id: "w1".into(),
            name: "Lint".into(),
            current_step: Some("src/".into()),
            completed: 4,
            planned: 8,
            state: Some(WorkerState::Running),
            counts: SeverityCounts {
                critical: 0,
                warning: 2,
                info: 1,
            },
        },
        ProbeWorker {
            id: "w2".into(),
            name: "Security".into(),
            current_step: None,
            completed: 8,
            planned: 8,
            state: Some(WorkerState::Done),
            counts: SeverityCounts {
                critical: 1,
                warning: 0,
                info: 0,
            },
        },
    ];
    let res = |id: &str, w: &str, sev: Severity, title: &str| FeedResult {
        id: id.into(),
        worker_id: w.into(),
        severity: sev,
        title: title.into(),
        location: Some("src/api.rs:42".into()),
        found_ms: NOW_MS - 5_000,
    };
    let results = vec![
        res("r1", "w2", Severity::Critical, "Secret in config"),
        res("r2", "w1", Severity::Warning, "Unused import"),
        res("r3", "w1", Severity::Info, "Long function"),
    ];
    rsx! {
        Item { slug: "parallel-probe-feed", name: "ParallelProbeFeed",
            ParallelProbeFeed {
                workers,
                results,
                visible_count: 2,
                clock_ms: Some(gallery_now()),
                on_select_result: move |r: FeedResult| last.set(format!("select {}", r.id)),
            }
            div { class: "gl-out", "data-out": "parallel-probe-feed", "{last()}" }
        }
    }
}

#[component]
fn CoEditingPresenceDemo() -> Element {
    use rv2_kit::agents::co_editing_presence::{
        Block, BlockKind, CoEditingPresence, Participant, Pin, Position, Proposal, View,
    };
    use std::collections::BTreeMap;
    let mut last = use_signal(String::new);
    let mut active_view = use_signal(|| Some("doc".to_string()));
    let block = |id: &str, kind: BlockKind, text: &str| Block {
        id: id.into(),
        kind,
        text: text.into(),
        image: None,
        cell_grid: vec![],
    };
    let content = vec![
        block("b1", BlockKind::Title, "Launch plan"),
        block("b2", BlockKind::Heading, "Goals"),
        block(
            "b3",
            BlockKind::Paragraph,
            "Ship the kit gallery before the review on Friday.",
        ),
        block("b4", BlockKind::Item, "Verify every component in a browser"),
    ];
    let participants = vec![
        Participant {
            id: "u1".into(),
            name: "Ada".into(),
            palette_key: Some("blue".into()),
            picture: None,
            wash_key: None,
            spend: None,
        },
        Participant {
            id: "u2".into(),
            name: "Ben".into(),
            palette_key: Some("amber".into()),
            picture: None,
            wash_key: None,
            spend: None,
        },
    ];
    let positions = BTreeMap::from([
        (
            "u1".to_string(),
            Position {
                block_id: "b3".into(),
                offset: Some(4),
                in_proposal: None,
                activity: Some("editing".into()),
                is_typing: true,
            },
        ),
        (
            "u2".to_string(),
            Position {
                block_id: "b4".into(),
                offset: None,
                in_proposal: None,
                activity: None,
                is_typing: false,
            },
        ),
    ]);
    let pins = vec![Pin {
        id: "pin1".into(),
        participant_id: "u2".into(),
        block_id: "b2".into(),
        offset: None,
        horizontal_fraction: Some(0.5),
        vertical_offset: None,
        body: "Is this still the goal?".into(),
        is_open: true,
    }];
    let proposals = vec![Proposal {
        id: "pr1".into(),
        participant_id: "u2".into(),
        block_id: "b3".into(),
        offset: 0,
        removal: Some("Ship".into()),
        insertion: Some("Land".into()),
        reason: Some("Tighter verb".into()),
    }];
    let views = vec![
        View {
            id: "doc".into(),
            caption: "Document".into(),
        },
        View {
            id: "outline".into(),
            caption: "Outline".into(),
        },
    ];
    rsx! {
        Item { slug: "co-editing-presence", name: "CoEditingPresence",
            CoEditingPresence {
                participants,
                content,
                positions,
                pins,
                proposals,
                views,
                active_view: active_view(),
                on_view_change: move |v: String| {
                    last.set(format!("view {v}"));
                    active_view.set(Some(v));
                },
                on_apply: move |p: Proposal| last.set(format!("apply {}", p.id)),
                on_dismiss: move |p: Proposal| last.set(format!("dismiss {}", p.id)),
                on_apply_all: move |_| last.set("apply all".into()),
                on_pin_close: move |p: Pin| last.set(format!("close pin {}", p.id)),
                on_pin_reply: move |(p, t): (Pin, String)| last.set(format!("reply {} {t}", p.id)),
                page_height: Some(360),
            }
            div { class: "gl-out", "data-out": "co-editing-presence", "{last()}" }
        }
    }
}

#[component]
fn RunTimelineRowsDemo() -> Element {
    use rv2_kit::agents::run_timeline_rows::{
        RowKind, RowSpec, RunTimelineRows, Span, SpanMode, Transfer,
    };
    let rows = vec![
        RowSpec {
            row_key: "you".into(),
            display_name: "You".into(),
            kind: RowKind::Person,
            tint_key: None,
            mark_source: None,
        },
        RowSpec {
            row_key: "agent".into(),
            display_name: "Agent".into(),
            kind: RowKind::Agent,
            tint_key: Some("blue".into()),
            mark_source: None,
        },
    ];
    let spans = vec![
        Span {
            span_key: Some("sp1".into()),
            row_key: "agent".into(),
            mode: SpanMode::Reasoning,
            begin_s: 0.0,
            end_s: Some(4.0),
            tool_name: None,
            caption: Some("Plan".into()),
        },
        Span {
            span_key: Some("sp2".into()),
            row_key: "agent".into(),
            mode: SpanMode::Tool,
            begin_s: 4.0,
            end_s: None,
            tool_name: Some("Search".into()),
            caption: None,
        },
        Span {
            span_key: None,
            row_key: "you".into(),
            mode: SpanMode::Waiting,
            begin_s: 2.0,
            end_s: Some(6.0),
            tool_name: None,
            caption: Some("Reviewing".into()),
        },
    ];
    let transfers = vec![Transfer {
        transfer_key: Some("tr1".into()),
        from_row_key: "agent".into(),
        to_row_key: "you".into(),
        at_s: 9.0,
        caption: Some("Asks for approval".into()),
    }];
    rsx! {
        Item { slug: "run-timeline-rows", name: "RunTimelineRows",
            RunTimelineRows {
                run_heading: "Run 42",
                rows,
                spans,
                transfers,
                clock_s: Some(7.5),
                axis_length_s: Some(12.0),
            }
        }
    }
}

#[component]
fn RunCheckpointRailDemo() -> Element {
    use rv2_kit::agents::run_checkpoint_rail::{
        CheckpointEntry, RunCheckpointRail, RunState, append_restart,
    };
    let mut last = use_signal(String::new);
    let entry = |id: &str, label: &str, state: RunState, follows: Option<&str>, started: u64| {
        CheckpointEntry {
            id: id.into(),
            label: label.into(),
            glyph: None,
            follows_id: follows.map(|f| f.to_string()),
            run_state: state,
            started_ms: Some(started),
            file_count: Some(3),
            spend: Some(0.11),
        }
    };
    let mut entries = use_signal(|| {
        vec![
            entry(
                "c1",
                "Plan approved",
                RunState::Done,
                None,
                NOW_MS - 1_500_000,
            ),
            entry(
                "c2",
                "Drafted section 1",
                RunState::Done,
                Some("c1"),
                NOW_MS - 900_000,
            ),
            entry(
                "c3",
                "Drafted section 2",
                RunState::Running,
                Some("c2"),
                NOW_MS - 240_000,
            ),
        ]
    });
    let mut current = use_signal(|| Some("c3".to_string()));
    let mut restarts = use_signal(|| 0usize);
    rsx! {
        Item { slug: "run-checkpoint-rail", name: "RunCheckpointRail",
            RunCheckpointRail {
                entries: entries(),
                current_id: current(),
                goal_text: "Write the report",
                clock_ms: Some(gallery_now()),
                on_restart_from: move |e: CheckpointEntry| {
                    last.set(format!("restart {}", e.id));
                    restarts += 1;
                    let next_id = format!("r{}", restarts());
                    let started = gallery_now();
                    entries.with_mut(|list| {
                        *list = append_restart(list, &e.id, &next_id, &format!("Restarted from {}", e.label));
                        if let Some(added) = list.last_mut() {
                            added.started_ms = Some(started);
                        }
                    });
                    current.set(Some(next_id));
                },
            }
            div { class: "gl-out", "data-out": "run-checkpoint-rail", "{last()}" }
        }
    }
}

#[component]
fn EditableStepPlanDemo() -> Element {
    use rv2_kit::agents::editable_step_plan::{EditableStepPlan, PlanPhase, PlanStep, StepState};
    let mut last = use_signal(String::new);
    let mut phase = use_signal(|| PlanPhase::Drafting);
    let mut steps = use_signal(|| {
        vec![
            PlanStep {
                step_key: "s1".into(),
                step_title: "Read the brief".into(),
                step_detail: None,
                tool_label: Some("Files".into()),
                step_state: StepState::Pending,
                measured_ms: None,
                error_text: None,
            },
            PlanStep {
                step_key: "s2".into(),
                step_title: "Draft outline".into(),
                step_detail: Some("Three sections".into()),
                tool_label: None,
                step_state: StepState::Pending,
                measured_ms: None,
                error_text: None,
            },
        ]
    });
    let estimate = format!("{} steps", steps.read().len());
    rsx! {
        Item { slug: "editable-step-plan", name: "EditableStepPlan",
            EditableStepPlan {
                plan_heading: "Plan",
                initial_steps: steps(),
                plan_phase: phase(),
                estimate_text: Some(estimate),
                on_start: move |s: Vec<PlanStep>| {
                    last.set(format!("start {} steps", s.len()));
                    phase.set(PlanPhase::Executing);
                },
                on_cancel: move |_| {
                    last.set("cancel".into());
                    phase.set(PlanPhase::Drafting);
                },
                on_edit: move |s: Vec<PlanStep>| {
                    last.set(format!("edit {}", s.len()));
                    steps.set(s);
                },
                on_retry_from: move |k: String| last.set(format!("retry {k}")),
                on_repeat: move |_| last.set("repeat".into()),
            }
            div { class: "gl-out", "data-out": "editable-step-plan", "{last()}" }
        }
    }
}
