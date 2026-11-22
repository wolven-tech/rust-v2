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
    let mut scan = unit(
        "u3",
        "w2",
        "Scan repo",
        UnitPhase::Settled,
        60_000,
        Some(420_000),
    );
    scan.highlights = vec!["412 files indexed".into(), "no errors".into()];
    let mut migration = unit(
        "u4",
        "w3",
        "Draft migration",
        UnitPhase::Held,
        580_000,
        Some(800_000),
    );
    migration.decision_prompt = Some("Apply the ledger schema change?".into());
    let mut tests = unit(
        "u5",
        "w4",
        "Run tests",
        UnitPhase::Broken,
        360_000,
        Some(600_000),
    );
    tests.highlights = vec!["3 assertions failed".into(), "ledger_sync.rs:88".into()];
    let mut read = unit(
        "u1",
        "w1",
        "Read ledger",
        UnitPhase::Settled,
        320_000,
        Some(500_000),
    );
    read.highlights = vec!["18 tables".into()];
    let mut rank = unit(
        "u6",
        "w2",
        "Rank findings",
        UnitPhase::Settled,
        430_000,
        Some(560_000),
    );
    rank.hands_off_to_key = Some("u4".into());
    let units = vec![
        read,
        unit(
            "u2",
            "w1",
            "Summarise figures",
            UnitPhase::Active,
            520_000,
            None,
        ),
        scan,
        rank,
        migration,
        tests,
        unit("u7", "w4", "Rerun tests", UnitPhase::Active, 620_000, None),
    ];
    let workers = vec![
        WorkerRow {
            worker_key: "w1".into(),
            worker_label: "Reader".into(),
            model_label: Some("Large model".into()),
        },
        WorkerRow {
            worker_key: "w2".into(),
            worker_label: "Scout".into(),
            model_label: Some("Small model".into()),
        },
        WorkerRow {
            worker_key: "w3".into(),
            worker_label: "Planner".into(),
            model_label: Some("Large model".into()),
        },
        WorkerRow {
            worker_key: "w4".into(),
            worker_label: "Tester".into(),
            model_label: Some("Small model".into()),
        },
        WorkerRow {
            worker_key: "w5".into(),
            worker_label: "Notifier".into(),
            model_label: None,
        },
    ];
    rsx! {
        Item { slug: "parallel-span-timeline", name: "ParallelSpanTimeline",
            ParallelSpanTimeline {
                worker_rows: workers,
                units,
                window_span: 600_000,
                clock_ms: 900_000u64,
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
            tint_key: None,
            model_tag: Some("Large model".into()),
            summary: Some("Breaks the release into steps and routes each one.".into()),
        },
        Actor {
            actor_key: "t".into(),
            role: ActorRole::Agent,
            caption: "Researcher".into(),
            mark_source: None,
            tint_key: None,
            model_tag: Some("Small model".into()),
            summary: Some("Reads commits since the last tag.".into()),
        },
        Actor {
            actor_key: "w".into(),
            role: ActorRole::Agent,
            caption: "Writer".into(),
            mark_source: None,
            tint_key: None,
            model_tag: Some("Small model".into()),
            summary: Some("Drafts the highlights from the grouped commits.".into()),
        },
        Actor {
            actor_key: "r".into(),
            role: ActorRole::Agent,
            caption: "Reviewer".into(),
            mark_source: None,
            tint_key: None,
            model_tag: Some("Large model".into()),
            summary: Some("Checks the draft against the changelog.".into()),
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
    let plan = ex(
        "e1",
        "p",
        "a",
        ExchangeKind::Request,
        ExchangeState::Ok,
        "Plan the release notes",
    );
    let scope = ex(
        "e2",
        "a",
        "a",
        ExchangeKind::Answer,
        ExchangeState::Ok,
        "Scope noted",
    );
    let at_rest = vec![plan.clone(), scope.clone()];
    let mut plan_answer = ex(
        "e5",
        "a",
        "p",
        ExchangeKind::Answer,
        ExchangeState::Ok,
        "Plan ready, four steps",
    );
    plan_answer.answers_key = Some("e1".into());
    let branched = vec![
        plan,
        scope,
        ex(
            "e3",
            "a",
            "r",
            ExchangeKind::Request,
            ExchangeState::Failed,
            "Review the draft",
        ),
        ex(
            "e4",
            "a",
            "p",
            ExchangeKind::Request,
            ExchangeState::AwaitingPerson,
            "Approve the outline?",
        ),
        plan_answer,
    ];
    rsx! {
        Item { slug: "exchange-diagram", name: "ExchangeDiagram",
            ExchangeDiagram {
                actors: actors.clone(),
                exchanges: at_rest,
                run_heading: "Release run",
                on_pick: move |e: Option<Exchange>| last.set(format!("pick {}", e.map(|x| x.exchange_key).unwrap_or("none".into()))),
            }
            div { class: "gl-out", "data-out": "exchange-diagram", "{last()}" }
        }
        Item { slug: "exchange-diagram-branched", name: "ExchangeDiagram",
            ExchangeDiagram {
                actors,
                exchanges: branched,
                run_heading: "Release run, with a failed review",
            }
        }
    }
}

/// One demo card: a state, a model, an activity line, context used out of a
/// 200k window, tool marks and the last hand-off received.
#[allow(clippy::too_many_arguments)]
fn tree_card(
    id: &str,
    name: &str,
    state: rv2_kit::agents::delegation_card_tree::AgentState,
    model: &str,
    doing: &str,
    used_tokens: u64,
    tools: &[(&str, &str)],
    received: Option<(&str, &str)>,
    started_minutes_ago: u64,
    sub_agents: Vec<rv2_kit::agents::delegation_card_tree::Agent>,
) -> rv2_kit::agents::delegation_card_tree::Agent {
    use rv2_kit::agents::delegation_card_tree::{Agent, LastTransfer, ToolName};
    let now = gallery_now();
    Agent {
        id: id.into(),
        name: name.into(),
        state,
        doing_text: Some(doing.into()),
        load_ratio: None,
        load_capacity_tokens: Some(200_000),
        load_used_tokens: Some(used_tokens),
        model_label: Some(model.into()),
        model_mark: None,
        tool_names: tools
            .iter()
            .map(|(mark, tool)| ToolName {
                mark: Some((*mark).into()),
                name: (*tool).into(),
            })
            .collect(),
        last_transfer: received.map(|(from, text)| LastTransfer {
            from_name: from.into(),
            text: text.into(),
        }),
        assignment: Some(doing.into()),
        started_at: Some(now.saturating_sub(started_minutes_ago * 60_000)),
        ended_at: None,
        spend: Some(used_tokens as f64 * 0.000_008),
        tokens: Some(used_tokens),
        identity_mark: None,
        sub_agents,
    }
}

#[component]
fn DelegationCardTreeDemo() -> Element {
    use rv2_kit::agents::delegation_card_tree::{
        AgentState, CommandAction, DelegationCardTree, Transfer, Workspace,
    };
    let mut last = use_signal(String::new);
    let mut workspace = use_signal(|| Some("main".to_string()));
    let summariser = tree_card(
        "a4",
        "Summariser",
        AgentState::Paused,
        "Fast",
        "Paused until the research lands",
        24_000,
        &[("RD", "Read"), ("DF", "Diff")],
        Some(("Planner", "Changelog batch 2")),
        22,
        vec![],
    );
    let researcher = tree_card(
        "a3",
        "Researcher",
        AgentState::Done,
        "Large",
        "Read 14 changelog entries",
        56_000,
        &[("WB", "Web"), ("SE", "Search"), ("RD", "Read")],
        Some(("Planner", "Find breaking changes since 2.4")),
        19,
        vec![],
    );
    let planner = tree_card(
        "a2",
        "Planner",
        AgentState::Active,
        "Large",
        "Drafting the order of the release steps",
        82_000,
        &[("SE", "Search"), ("DC", "Docs")],
        Some(("Orchestrator", "Release brief v3")),
        17,
        vec![researcher, summariser],
    );
    let test_runner = tree_card(
        "a6",
        "Test runner",
        AgentState::Failed,
        "Fast",
        "Two empty-cart cases failed twice",
        194_000,
        &[("SH", "Shell"), ("TS", "Test")],
        Some(("Coder", "Run the cart suite")),
        12,
        vec![],
    );
    let coder = tree_card(
        "a5",
        "Coder",
        AgentState::Waiting,
        "Large",
        "Needs your approval to run the migration",
        166_000,
        &[
            ("SH", "Shell"),
            ("GT", "Git"),
            ("ED", "Edit"),
            ("TS", "Test"),
            ("DF", "Diff"),
        ],
        Some(("Orchestrator", "Guard empty carts")),
        14,
        vec![test_runner],
    );
    let reviewer = tree_card(
        "a7",
        "Reviewer",
        AgentState::Idle,
        "Fast",
        "Waiting for the diff",
        12_000,
        &[("DF", "Diff"), ("DC", "Docs")],
        Some(("Coder", "Diff ready for review")),
        9,
        vec![],
    );
    let top = tree_card(
        "a1",
        "Orchestrator",
        AgentState::Active,
        "Large",
        "Splitting checkout work across three agents",
        124_000,
        &[
            ("PL", "Plan"),
            ("RT", "Route"),
            ("NT", "Notes"),
            ("SE", "Search"),
            ("GT", "Git"),
        ],
        None,
        31,
        vec![planner, coder, reviewer],
    );
    let transfers = vec![Transfer {
        id: "t1".into(),
        source_id: "a1".into(),
        target_id: "a5".into(),
        note: Some("Guard empty carts".into()),
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
    let job = |id: &str,
               owner: &str,
               heading: &str,
               summary: &str,
               stage: JobStage,
               elapsed: u64,
               steps: &[&str]| Job {
        id: id.into(),
        heading_text: heading.into(),
        summary_text: Some(summary.into()),
        owner_id: owner.into(),
        stage,
        started_at: Some(NOW_MS - elapsed),
        elapsed: Some(elapsed),
        tool_count: Some(4),
        source_count: Some(9),
        timeline: steps
            .iter()
            .map(|label| TimelineStep {
                label: (*label).into(),
                at: Some(NOW_MS - elapsed),
            })
            .collect(),
    };
    let jobs = vec![
        job(
            "j1",
            "indexer",
            "Index the docs folder",
            "Reading 14 files for the migration plan",
            JobStage::Running,
            42_000,
            &["Listed folder", "Read 9 of 14 files"],
        ),
        job(
            "j2",
            "approvals",
            "Confirm delete of staging rows",
            "1,204 rows in staging_orders match the purge rule",
            JobStage::Waiting,
            9_000,
            &["Matched purge rule"],
        ),
        job(
            "j3",
            "exporter",
            "Export usage report",
            "Wrote usage-2026-11.csv, 38 KB",
            JobStage::Done,
            120_000,
            &["Queried events", "Wrote CSV", "Uploaded"],
        ),
        job(
            "j4",
            "billing",
            "Sync billing events",
            "Upstream returned 503 after 3 retries",
            JobStage::Failed,
            15_000,
            &["Fetched page 1", "Retry 1", "Retry 2", "Retry 3"],
        ),
        job(
            "j5",
            "support",
            "Summarise support tickets",
            "Grouping 212 tickets by product area",
            JobStage::Running,
            201_000,
            &["Loaded tickets", "Clustered 150 of 212"],
        ),
        job(
            "j6",
            "writer",
            "Draft release email",
            "Two tone options ready, waiting on your choice",
            JobStage::Waiting,
            64_000,
            &["Drafted tone A", "Drafted tone B"],
        ),
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
        ActFuture, ActionWhich, DigestEvent, DigestLane, MainAction, OvernightRunDigest,
    };
    let mut last = use_signal(String::new);
    let mut reverted = use_signal(Vec::<String>::new);
    let ev = |key: &str,
              hours_ago: u64,
              lane: DigestLane,
              title: &str,
              detail: &str,
              action: Option<&str>| {
        DigestEvent {
            event_key: key.into(),
            happened_at: Some(NOW_MS - hours_ago * 3_600_000),
            event_lane: lane,
            event_title: title.into(),
            event_detail: Some(detail.into()),
            repeat_count: None,
            main_action: action.map(|a| MainAction {
                action_label: a.into(),
            }),
            inspect_available: true,
            past_tense_label: Some("Applied".into()),
            reversible: true,
        }
    };
    let mut billing = ev(
        "e1",
        4,
        DigestLane::Errored,
        "Sync billing events",
        "Upstream returned 503 and the retries ran out",
        Some("Retry"),
    );
    billing.repeat_count = Some(2);
    let events = vec![
        ev(
            "d1",
            7,
            DigestLane::AwaitingDecision,
            "Merge 3 duplicate contacts",
            "Same email address in the CRM and billing exports",
            Some("Merge"),
        ),
        ev(
            "d2",
            5,
            DigestLane::AwaitingDecision,
            "Send renewal emails to 14 accounts",
            "Drafted by the outreach agent; needs your sign-off",
            Some("Approve"),
        ),
        ev(
            "d3",
            3,
            DigestLane::AwaitingDecision,
            "Publish the pricing page draft",
            "Copy reviewed; waiting for the go-ahead",
            Some("Publish"),
        ),
        ev(
            "d4",
            9,
            DigestLane::AwaitingDecision,
            "Rotate the staging API key",
            "Expires in 6 days; the old key still works",
            Some("Rotate"),
        ),
        billing,
        ev(
            "e2",
            2,
            DigestLane::Errored,
            "Export to CSV failed",
            "Timed out after 240 s at 38,000 rows",
            Some("Retry"),
        ),
        ev(
            "c1",
            6,
            DigestLane::Completed,
            "Applied 3 label rules",
            "Rules ran over the support inbox",
            None,
        ),
        ev(
            "c2",
            1,
            DigestLane::Completed,
            "Generated weekly metrics",
            "Revenue, usage and churn tables for the week",
            None,
        ),
    ];
    rsx! {
        Item { slug: "overnight-run-digest", name: "OvernightRunDigest",
            OvernightRunDigest {
                digest_events: events,
                away_since: NOW_MS - 8 * 3_600_000,
                now_at: Some(gallery_now()),
                replay_on_open: false,
                acted_keys: Some(vec!["c1".to_string()]),
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
        MidRunNotePanel, Note, NotePlacement, PlanStep, RunState, StepPhase,
    };
    let mut last = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let mut queued = use_signal(|| {
        vec![Note {
            note_key: "q1".into(),
            text: "Use the Q3 figures, not Q2".into(),
            placed: None,
        }]
    });
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
            step_notes: vec![Note {
                note_key: "t1".into(),
                text: "Keep it under 300 words".into(),
                placed: Some(NotePlacement::Now),
            }],
        },
        PlanStep {
            step_key: "s3".into(),
            label: "Check citations".into(),
            phase: StepPhase::Upcoming,
            began_at_ms: None,
            took_ms: None,
            step_notes: vec![Note {
                note_key: "t2".into(),
                text: "Flag any source older than two years".into(),
                placed: Some(NotePlacement::Next),
            }],
        },
        PlanStep {
            step_key: "s4".into(),
            label: "Format for the newsletter".into(),
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
            sub_items: vec![
                SubItem {
                    sub_key: "p4a".into(),
                    text: "Load the three outlines".into(),
                    sub_state: Some(SubState::Finished),
                    fact_text: Some("3 files".into()),
                },
                SubItem {
                    sub_key: "p4b".into(),
                    text: "Merge the shared sections".into(),
                    sub_state: Some(SubState::InProgress),
                    fact_text: Some("2 of 4".into()),
                },
                SubItem {
                    sub_key: "p4c".into(),
                    text: "Tighten the opening paragraph".into(),
                    sub_state: Some(SubState::Queued),
                    fact_text: None,
                },
            ],
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
        ProgressItem {
            item_key: "p6".into(),
            title_text: "Upload the press kit".into(),
            item_state: ItemState::Errored,
            took_ms: Some(8_400),
            began_at_ms: None,
            live_note: None,
            needs_approval: false,
            tool_badge: Some(ToolBadge {
                display_name: "Files".into(),
                mark_source: None,
            }),
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
    let mut collect = node("n5", "Collect commits", NodePhase::Complete, vec![]);
    collect.began_at_ms = Some(NOW_MS - 300_000);
    collect.ended_at_ms = Some(NOW_MS - 240_000);
    collect.spend = 0.04;
    let mut changelog = node("n2", "Changelog", NodePhase::Complete, vec![collect]);
    changelog.began_at_ms = Some(NOW_MS - 300_000);
    changelog.ended_at_ms = Some(NOW_MS - 150_000);
    changelog.spend = 0.09;
    let mut summarise = node("n7", "Summarise PRs", NodePhase::Active, vec![]);
    summarise.began_at_ms = Some(NOW_MS - 120_000);
    summarise.spend = 0.21;
    let mut tone = node("n8", "Tone check", NodePhase::Pending, vec![]);
    tone.spend = 0.0;
    let draft = node(
        "n6",
        "Draft notes",
        NodePhase::Active,
        vec![summarise, tone],
    );
    let mut publish = node("n9", "Publish", NodePhase::Pending, vec![]);
    publish.spend = 0.0;
    let root = node(
        "n1",
        "Release",
        NodePhase::Active,
        vec![changelog, draft, errored, waiting, publish],
    );
    rsx! {
        Item { slug: "sub-run-tree", name: "SubRunTree",
            SubRunTree {
                root_node: Some(root),
                clock_ms: gallery_now(),
                default_folded_keys: vec!["n2".to_string()],
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
    let mut reverted = use_signal(|| vec!["c1".to_string()]);
    let mut at_rest_reverted = use_signal(Vec::<String>::new);
    let price = ChangeRow {
        row_key: "c5".into(),
        verb_label: "Updated".into(),
        target_label: Some("price table".into()),
        system_label: "Sheets".into(),
        system_mark: None,
        happened_at_ms: NOW_MS - 300_000,
        value_change: Some(ValueChange {
            from_text: "$19".into(),
            to_text: "$24".into(),
        }),
        summary_line: None,
        reversible: true,
        built_on_keys: vec![],
    };
    let rename = ChangeRow {
        row_key: "c1".into(),
        verb_label: "Renamed".into(),
        target_label: Some("Q3 plan".into()),
        system_label: "Docs".into(),
        system_mark: None,
        happened_at_ms: NOW_MS - 10_800_000,
        value_change: Some(ValueChange {
            from_text: "Q3 draft".into(),
            to_text: "Q3 plan".into(),
        }),
        summary_line: None,
        reversible: true,
        built_on_keys: vec![],
    };
    let at_rest = vec![price.clone(), rename.clone()];
    let branched = vec![
        price,
        ChangeRow {
            row_key: "c4".into(),
            verb_label: "Sent".into(),
            target_label: Some("Invoice 204".into()),
            system_label: "Mail".into(),
            system_mark: None,
            happened_at_ms: NOW_MS - 1_800_000,
            value_change: None,
            summary_line: Some("To accounts@acme.test".into()),
            reversible: false,
            built_on_keys: vec![],
        },
        ChangeRow {
            row_key: "c3".into(),
            verb_label: "Deleted".into(),
            target_label: Some("old-export.csv".into()),
            system_label: "Drive".into(),
            system_mark: None,
            happened_at_ms: NOW_MS - 5_400_000,
            value_change: None,
            summary_line: Some("Moved to trash".into()),
            reversible: true,
            built_on_keys: vec![],
        },
        rename,
    ];
    rsx! {
        Item { slug: "reversible-change-log", name: "ReversibleChangeLog",
            ReversibleChangeLog {
                change_rows: at_rest,
                actor_label: Some("Agent".to_string()),
                clock_ms: Some(gallery_now()),
                reverted_keys: Some(at_rest_reverted()),
                on_revert: move |k: Vec<String>| {
                    last.set(format!("revert {}", k.join(",")));
                    at_rest_reverted.with_mut(|list| list.extend(k));
                },
                on_reapply: move |k: Vec<String>| {
                    last.set(format!("reapply {}", k.join(",")));
                    at_rest_reverted.with_mut(|list| list.retain(|key| !k.contains(key)));
                },
            }
            div { class: "gl-out", "data-out": "reversible-change-log", "{last()}" }
        }
        Item { slug: "reversible-change-log-branched", name: "ReversibleChangeLog",
            ReversibleChangeLog {
                change_rows: branched,
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
            div { class: "gl-out", "data-out": "reversible-change-log-branched", "{last()}" }
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
                requester: Requester { display_name: "Maya".into(), message_text: "Can you tidy the onboarding copy before Friday? Keep it friendly and short.".into(), photo_url: None },
                worker: Worker {
                    worker_key: "w".into(),
                    display_name: "Copy agent".into(),
                    subtitle_text: Some("Editing".into()),
                    detail_rows: vec![detail("Model", "model A"), detail("Budget", "$2.00")],
                },
                worker_stage: WorkerStage::Working,
                file_items: vec![
                    FileItem { file_key: "f1".into(), display_name: "onboarding.md".into(), mark_source: None, meta_text: Some("4 KB".into()), detail_rows: vec![detail("Lines", "120")] },
                    FileItem { file_key: "f2".into(), display_name: "style-guide.md".into(), mark_source: None, meta_text: Some("12 KB".into()), detail_rows: vec![] },
                    FileItem { file_key: "f3".into(), display_name: "tone-examples.md".into(), mark_source: None, meta_text: Some("2 KB".into()), detail_rows: vec![] },
                    FileItem { file_key: "f4".into(), display_name: "glossary.csv".into(), mark_source: None, meta_text: Some("96 lines".into()), detail_rows: vec![] },
                    FileItem { file_key: "f5".into(), display_name: "release-notes.md".into(), mark_source: None, meta_text: None, detail_rows: vec![] },
                ],
                reading_file_key: Some("f1".to_string()),
                work_log: vec![
                    WorkStep { step_key: "k0".into(), text: "Read the style guide".into(), route_key: "files".into(), in_progress: false },
                    WorkStep { step_key: "k1".into(), text: "Checked terms against the glossary".into(), route_key: "sheets".into(), in_progress: false },
                    WorkStep { step_key: "k2".into(), text: "Rewrote the welcome paragraph".into(), route_key: "requester".into(), in_progress: false },
                    WorkStep { step_key: "k3".into(), text: "Checking search results for tone".into(), route_key: "search".into(), in_progress: true },
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
        Channel, HandlerNode, RequestRoutingMap, RequestRow, RequestState, RequestTag, RouterNode,
    };
    use std::collections::BTreeMap;
    let mut last = use_signal(String::new);
    let router = RouterNode {
        node_key: "router".into(),
        node_label: "Triage".into(),
        model_label: Some("model A".into()),
        reading_text: None,
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
            queued_request_keys: vec!["r4".into()],
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
        HandlerNode {
            handler_key: "ops".into(),
            handler_label: "Ops".into(),
            model_label: Some("model B".into()),
            handler_state: Some("Working".into()),
            activity_text: Some("Tracing a carrier delay".into()),
            load_count: Some(1),
            queued_request_keys: vec!["r5".into()],
            is_person: false,
            accent_slot: 3,
            handler_mark: None,
            model_mark: None,
        },
        HandlerNode {
            handler_key: "support".into(),
            handler_label: "Support".into(),
            model_label: None,
            handler_state: Some("Idle".into()),
            activity_text: None,
            load_count: None,
            queued_request_keys: vec![],
            is_person: false,
            accent_slot: 5,
            handler_mark: None,
            model_mark: None,
        },
    ];
    let row = |key: &str,
               title: &str,
               channel: Channel,
               state: RequestState,
               assigned: Option<&str>,
               certainty: f64,
               scores: [f64; 4]| {
        RequestRow {
            request_key: key.into(),
            request_title: title.into(),
            sender_label: Some("customer@example.com".into()),
            channel_kind: channel,
            preview_text: Some("Hi, my invoice shows...".into()),
            request_state: state,
            assigned_to_key: assigned.map(|a| a.to_string()),
            certainty_fraction: certainty,
            score_map: BTreeMap::from([
                ("billing".to_string(), scores[0]),
                ("ops".to_string(), scores[1]),
                ("support".to_string(), scores[2]),
                ("sam".to_string(), scores[3]),
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
            "Login failing",
            Channel::Ticket,
            RequestState::Incoming,
            None,
            0.0,
            [0.0, 0.0, 0.0, 0.0],
        ),
        row(
            "r3",
            "Partnership inquiry",
            Channel::Phone,
            RequestState::NeedsPerson,
            Some("sam"),
            0.71,
            [0.2, 0.08, 0.12, 0.71],
        ),
        row(
            "r4",
            "Duplicate charge",
            Channel::Mail,
            RequestState::Assigned,
            Some("billing"),
            0.94,
            [0.94, 0.12, 0.2, 0.1],
        ),
        row(
            "r5",
            "Late delivery",
            Channel::TeamChat,
            RequestState::Assigned,
            Some("ops"),
            0.88,
            [0.14, 0.88, 0.18, 0.05],
        ),
    ];
    let mut sorting_rows = rows.clone();
    sorting_rows[0].request_state = RequestState::Sorting;
    sorting_rows[0].certainty_fraction = 0.56;
    let sorting_router = router.clone();
    let sorting_handlers = handlers.clone();
    let mut rows_state = use_signal(|| rows);
    rsx! {
        Item { slug: "request-routing-map", name: "RequestRoutingMap",
            RequestRoutingMap {
                router_node: router,
                handler_nodes: handlers,
                request_rows: rows_state(),
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
        Item { slug: "request-routing-map-sorting", name: "RequestRoutingMap, while sorting",
            RequestRoutingMap {
                router_node: sorting_router,
                handler_nodes: sorting_handlers,
                request_rows: sorting_rows,
            }
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
    let sub = |id: &str,
               name: &str,
               state: FleetState,
               spend: Option<f64>,
               work_items: Vec<WorkItem>,
               question: Option<&str>| SubAgent {
        id: id.into(),
        name: name.into(),
        assignment: format!("{name} task"),
        model_label: Some("model B".into()),
        state,
        work_items,
        spend,
        question: question.map(|q| q.to_string()),
        failure_text: None,
        children: Vec::new(),
    };
    let item = |label: &str, state: Option<WorkState>, tool: &str, secs: Option<u64>| WorkItem {
        label: label.into(),
        state,
        tool_name: Some(tool.into()),
        duration: secs.map(|s| s * 1_000),
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
                {
                    let mut pricing = sub(
                        "s1",
                        "Pricing",
                        FleetState::Running,
                        Some(0.08),
                        vec![
                            item("Pull price pages", None, "fetch", Some(38)),
                            item(
                                "Compare tiers",
                                Some(WorkState::Running),
                                "search",
                                Some(74),
                            ),
                            item("Write price table", Some(WorkState::Queued), "sheets", None),
                        ],
                        None,
                    );
                    let mut ad = sub(
                        "s4",
                        "Ad copy",
                        FleetState::Failed,
                        Some(0.03),
                        vec![item(
                            "Draft headlines",
                            Some(WorkState::Failed),
                            "model",
                            Some(21),
                        )],
                        None,
                    );
                    ad.failure_text = Some("Rate limit hit after 3 retries".into());
                    pricing.children = vec![ad];
                    pricing
                },
                sub(
                    "s2",
                    "Reviews",
                    FleetState::Waiting,
                    Some(0.05),
                    vec![item(
                        "Pull app store reviews",
                        Some(WorkState::Waiting),
                        "fetch",
                        Some(52),
                    )],
                    Some("Include app store reviews?"),
                ),
                sub(
                    "s3",
                    "Traffic",
                    FleetState::Done,
                    Some(0.04),
                    vec![item("Read analytics export", None, "analytics", Some(96))],
                    None,
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
            sub_agents: vec![
                sub("s6", "Outline", FleetState::Queued, None, vec![], None),
                sub("s7", "Draft", FleetState::Queued, None, vec![], None),
            ],
        },
        LeadAgent {
            id: "lead-3".into(),
            name: "Data lead".into(),
            assignment: "Clean the price history".into(),
            model_label: Some("model B".into()),
            picture: None,
            tool_marks: vec![],
            state: FleetState::Done,
            spend: Some(0.19),
            started_at_ms: Some(NOW_MS - 900_000),
            ended_at_ms: Some(NOW_MS - 120_000),
            sub_agents: vec![sub(
                "s5",
                "Dedupe",
                FleetState::Done,
                Some(0.06),
                vec![item("Merge duplicates", None, "sheets", Some(12))],
                None,
            )],
        },
        LeadAgent {
            id: "lead-4".into(),
            name: "Ops lead".into(),
            assignment: "Watch the price feeds".into(),
            model_label: Some("model A".into()),
            picture: None,
            tool_marks: vec![],
            state: FleetState::Running,
            spend: Some(0.11),
            started_at_ms: Some(NOW_MS - 240_000),
            ended_at_ms: None,
            sub_agents: vec![
                sub(
                    "s8",
                    "Monitor",
                    FleetState::Running,
                    Some(0.07),
                    vec![item(
                        "Check uptime",
                        Some(WorkState::Running),
                        "fetch",
                        Some(9),
                    )],
                    None,
                ),
                sub("s9", "Alerts", FleetState::Queued, None, vec![], None),
            ],
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
    let now = gallery_now();
    let ent = |id: &str,
               name: &str,
               role: NodeRole,
               cond: Condition,
               centre: bool,
               activity: Option<&str>| Entity {
        id: id.into(),
        name: name.into(),
        node_role: role,
        compact_label: None,
        glyph: None,
        summary_line: None,
        condition: cond,
        activity: activity.map(|a| a.into()),
        spend: Some(0.05),
        is_centre: centre,
    };
    let entities = vec![
        ent(
            "orchestrator",
            "Orchestrator",
            NodeRole::Agent,
            Condition::Working,
            true,
            Some("Planning the brief"),
        ),
        ent(
            "researcher",
            "Researcher",
            NodeRole::Agent,
            Condition::Working,
            false,
            Some("Reading competitor pages"),
        ),
        ent(
            "analyst",
            "Analyst",
            NodeRole::Agent,
            Condition::Waiting,
            false,
            Some("Needs a threshold decision"),
        ),
        ent(
            "coder",
            "Coder",
            NodeRole::Agent,
            Condition::Idle,
            false,
            None,
        ),
        ent(
            "search",
            "Web search",
            NodeRole::Tool,
            Condition::Idle,
            false,
            None,
        ),
        ent(
            "notion",
            "Notion",
            NodeRole::Tool,
            Condition::Idle,
            false,
            None,
        ),
        ent(
            "database",
            "Database",
            NodeRole::Tool,
            Condition::Error,
            false,
            None,
        ),
        ent(
            "calendar",
            "Calendar",
            NodeRole::Tool,
            Condition::Idle,
            false,
            None,
        ),
        ent(
            "github",
            "GitHub",
            NodeRole::Tool,
            Condition::Idle,
            false,
            None,
        ),
        ent(
            "slack",
            "Slack",
            NodeRole::Tool,
            Condition::Done,
            false,
            None,
        ),
        ent("you", "You", NodeRole::User, Condition::Idle, false, None),
    ];
    let exchange = |id: &str,
                    from: &str,
                    to: &str,
                    ago_ms: u64,
                    result: ExchangeResult,
                    caption: &str,
                    latency: Option<u64>| Exchange {
        id: id.into(),
        from_id: from.into(),
        to_id: to.into(),
        sent_at_ms: now - ago_ms,
        result,
        caption: Some(caption.into()),
        latency_ms: latency,
    };
    let exchanges = vec![
        exchange(
            "x1",
            "orchestrator",
            "researcher",
            40_000,
            ExchangeResult::Ok,
            "brief",
            Some(1_800),
        ),
        exchange(
            "x2",
            "researcher",
            "search",
            30_000,
            ExchangeResult::Ok,
            "query",
            Some(640),
        ),
        exchange(
            "x3",
            "analyst",
            "database",
            12_000,
            ExchangeResult::Error,
            "select",
            Some(4_200),
        ),
        exchange(
            "x4",
            "coder",
            "github",
            20_000,
            ExchangeResult::Ok,
            "open PR",
            Some(2_400),
        ),
        exchange(
            "x5",
            "orchestrator",
            "researcher",
            3_000,
            ExchangeResult::Pending,
            "follow-up",
            None,
        ),
    ];
    rsx! {
        Item { slug: "tool-call-network", name: "ToolCallNetwork",
            ToolCallNetwork {
                entities,
                links: [
                    ("orchestrator", "researcher"),
                    ("orchestrator", "analyst"),
                    ("orchestrator", "coder"),
                    ("orchestrator", "you"),
                    ("researcher", "search"),
                    ("researcher", "notion"),
                    ("analyst", "database"),
                    ("analyst", "calendar"),
                    ("coder", "github"),
                    ("coder", "slack"),
                ]
                .map(|(from, to)| (from.to_string(), to.to_string()))
                .to_vec(),
                exchanges,
                clock_now_ms: Some(now),
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
    let mut jobs = vec![
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
        job(
            "j4",
            "Draft summary",
            JobState::Succeeded,
            JobKind::ModelCall,
            Priority::Standard,
        ),
        job(
            "j5",
            "Sync CRM",
            JobState::Errored,
            JobKind::ToolCall,
            Priority::Urgent,
        ),
        job(
            "j6",
            "Tag inbox",
            JobState::Enqueued,
            JobKind::ModelCall,
            Priority::Standard,
        ),
    ];
    jobs[4].error_text = Some("Timeout after 30 s".into());
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
            name: "Intake".into(),
            model_label: Some("model A".into()),
            role_text: Some("Logs the brief".into()),
        },
        RelayAgent {
            id: "a2".into(),
            name: "Drafter".into(),
            model_label: None,
            role_text: Some("Writes the first draft".into()),
        },
        RelayAgent {
            id: "a3".into(),
            name: "Checker".into(),
            model_label: Some("model B".into()),
            role_text: Some("Verifies claims".into()),
        },
    ];
    let assignments = vec![
        Assignment {
            id: "as1".into(),
            caption: "Interview notes".into(),
            stage: BatonStage::Done,
            holder_id: None,
            memo: None,
        },
        Assignment {
            id: "as2".into(),
            caption: "Q3 draft".into(),
            stage: BatonStage::Working,
            holder_id: Some("a2".into()),
            memo: Some("Use the 2024 numbers".into()),
        },
        Assignment {
            id: "as3".into(),
            caption: "Customer quotes".into(),
            stage: BatonStage::Todo,
            holder_id: Some("a3".into()),
            memo: None,
        },
        Assignment {
            id: "as4".into(),
            caption: "Q3 summary".into(),
            stage: BatonStage::Warning,
            holder_id: Some("a3".into()),
            memo: None,
        },
        Assignment {
            id: "as5".into(),
            caption: "Launch notes".into(),
            stage: BatonStage::Todo,
            holder_id: None,
            memo: None,
        },
        Assignment {
            id: "as6".into(),
            caption: "Press copy".into(),
            stage: BatonStage::Todo,
            holder_id: None,
            memo: Some("Hold until sources are in".into()),
        },
    ];
    let item = |id: &str, text: &str, kind: EntryType, dropped: Option<&str>| PayloadItem {
        id: id.into(),
        text: text.into(),
        entry_type: Some(kind),
        dropped: dropped.is_some(),
        drop_reason: dropped.map(Into::into),
    };
    let transfers = vec![
        Transfer {
            id: "t1".into(),
            from_agent_id: "a1".into(),
            to_agent_id: "a2".into(),
            task_ref: "Interview notes".into(),
            payload: vec![
                item("p1", "Interview transcript", EntryType::Source, None),
                item("p2", "Pricing quote", EntryType::Fact, None),
            ],
        },
        Transfer {
            id: "t2".into(),
            from_agent_id: "a1".into(),
            to_agent_id: "a2".into(),
            task_ref: "Q3 draft".into(),
            payload: vec![
                item("p3", "Opening paragraph", EntryType::Fact, None),
                item("p4", "Quote from 2019 report", EntryType::Source, None),
                item("p5", "Pricing table", EntryType::Source, None),
            ],
        },
        Transfer {
            id: "t3".into(),
            from_agent_id: "a2".into(),
            to_agent_id: "a3".into(),
            task_ref: "Customer quotes".into(),
            payload: vec![
                item("p6", "Three customer quotes", EntryType::Source, None),
                item("p7", "Churn figure", EntryType::Fact, None),
                item(
                    "p8",
                    "Stale churn note",
                    EntryType::Fact,
                    Some("Superseded by 2024 data"),
                ),
            ],
        },
        Transfer {
            id: "t4".into(),
            from_agent_id: "a2".into(),
            to_agent_id: "a3".into(),
            task_ref: "Q3 summary".into(),
            payload: vec![
                item("p9", "Headline figures", EntryType::Fact, None),
                item("p10", "Chart captions", EntryType::Note, None),
            ],
        },
    ];
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
    let step = |key: &str,
                label: &str,
                kind: StepKind,
                state: StepState,
                tool: &str,
                target: &str,
                elapsed: Option<u64>| StepRow {
        step_key: key.into(),
        step_label: label.into(),
        step_kind: Some(kind),
        step_state: state,
        started_at_ms: Some(gallery_now() - 260_000),
        elapsed_ms: elapsed,
        tool_mark: None,
        tool_label: Some(tool.into()),
        detail_text: None,
        target_text: Some(target.into()),
    };
    rsx! {
        Item { slug: "single-run-status-card", name: "SingleRunStatusCard",
            SingleRunStatusCard {
                run_phase: phase(),
                worker_identity: WorkerIdentity { display_label: "Refactor agent".into(), role_label: Some("Coder".into()), mark: None },
                step_rows: vec![
                    step("s1", "Open module", StepKind::Inspect, StepState::Succeeded, "Files", "src/lib.rs", Some(4_000)),
                    step("s2", "Find call sites", StepKind::Find, StepState::Succeeded, "Search", "rg parse_config", Some(11_000)),
                    step("s3", "Run unit tests", StepKind::Execute, StepState::Errored, "Shell", "cargo test -p core", Some(46_000)),
                    step("s4", "Patch parser", StepKind::Modify, StepState::Succeeded, "Editor", "src/parse.rs", Some(128_000)),
                    step("s5", "Edit call sites", StepKind::Modify, StepState::InProgress, "Editor", "src/lib.rs", None),
                    step("s6", "Format", StepKind::Execute, StepState::Pending, "Shell", "cargo fmt", None),
                ],
                started_at_ms: Some(gallery_now() - 754_000),
                clock_ms: Some(gallery_now()),
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
    let counts = |critical: u32, warning: u32, info: u32| SeverityCounts {
        critical,
        warning,
        info,
    };
    let worker = |id: &str,
                  name: &str,
                  step: Option<&str>,
                  completed: u32,
                  planned: u32,
                  state: Option<WorkerState>,
                  c: SeverityCounts| ProbeWorker {
        id: id.into(),
        name: name.into(),
        current_step: step.map(Into::into),
        completed,
        planned,
        state,
        counts: c,
    };
    let workers = vec![
        worker(
            "w1",
            "Lint",
            Some("src/parser/"),
            4,
            8,
            Some(WorkerState::Running),
            counts(0, 2, 1),
        ),
        worker(
            "w2",
            "Security",
            None,
            8,
            8,
            Some(WorkerState::Done),
            counts(1, 0, 0),
        ),
        worker(
            "w3",
            "Types",
            Some("crates/rv2-kit"),
            9,
            12,
            Some(WorkerState::Running),
            counts(0, 1, 2),
        ),
        worker(
            "w4",
            "Tests",
            None,
            3,
            6,
            Some(WorkerState::Failed),
            counts(1, 1, 0),
        ),
        worker(
            "w5",
            "Docs links",
            Some("docs/"),
            1,
            5,
            Some(WorkerState::Running),
            counts(0, 0, 3),
        ),
        worker("w6", "Bundle size", None, 0, 4, None, counts(0, 0, 0)),
    ];
    let res =
        |id: &str, w: &str, sev: Severity, title: &str, location: Option<&str>, ago_ms: u64| {
            FeedResult {
                id: id.into(),
                worker_id: w.into(),
                severity: sev,
                title: title.into(),
                location: location.map(Into::into),
                found_ms: gallery_now() - ago_ms,
            }
        };
    let results = vec![
        res(
            "r1",
            "w2",
            Severity::Critical,
            "Secret in config",
            Some("src/api.rs:42"),
            7_000,
        ),
        res(
            "r2",
            "w4",
            Severity::Critical,
            "Snapshot mismatch in checkout",
            Some("tests/cart.rs:118"),
            41_000,
        ),
        res(
            "r3",
            "w1",
            Severity::Warning,
            "Unused import",
            Some("src/parser/mod.rs:3"),
            96_000,
        ),
        res(
            "r4",
            "w3",
            Severity::Warning,
            "Option unwrapped without check",
            Some("crates/rv2-kit/src/lib.rs:77"),
            4 * 60_000,
        ),
        res(
            "r5",
            "w5",
            Severity::Info,
            "Link to moved page",
            Some("docs/setup.md:12"),
            9 * 60_000,
        ),
        res(
            "r6",
            "w3",
            Severity::Info,
            "Long function",
            None,
            14 * 60_000,
        ),
    ];
    rsx! {
        Item { slug: "parallel-probe-feed", name: "ParallelProbeFeed",
            ParallelProbeFeed {
                workers,
                results,
                visible_count: 5,
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
        Block, BlockKind, CoEditingPresence, ImageBlock, Participant, Pin, Position, Proposal,
        TableCell, View, WrapSide,
    };
    use std::collections::BTreeMap;
    const SAMPLE_IMAGE: &str = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 9'><rect width='16' height='9' fill='%23cbd5e1'/><rect x='2' y='2' width='12' height='5' rx='1' fill='%2364748b'/></svg>";
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
        block("b5", BlockKind::Item, "Check each card in dark theme"),
        block("b6", BlockKind::Item, "Record the gaps in the review notes"),
        block("b7", BlockKind::Heading, "Owners"),
        block(
            "b8",
            BlockKind::Paragraph,
            "Ada leads the build. Ben owns the review.",
        ),
        Block {
            id: "b9".into(),
            kind: BlockKind::Table,
            text: String::new(),
            image: None,
            cell_grid: [
                vec![("Owner", None), ("Task", None), ("Status", None)],
                vec![("Ada", None), ("Build", None), ("On track", Some("ok"))],
                vec![("Ben", None), ("Review", None), ("Waiting", None)],
            ]
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|(text, badge)| TableCell {
                        text: text.into(),
                        badge_key: badge.map(Into::into),
                    })
                    .collect()
            })
            .collect(),
        },
        Block {
            id: "b10".into(),
            kind: BlockKind::Image,
            text: "Gallery at 1280px".into(),
            image: Some(ImageBlock {
                column_fraction: 0.6,
                aspect: 16.0 / 9.0,
                wrap_side: WrapSide::Right,
                is_loading: false,
                source_url: Some(SAMPLE_IMAGE.into()),
                description: Some("Kit gallery preview".into()),
            }),
            cell_grid: vec![],
        },
        block(
            "b11",
            BlockKind::Paragraph,
            "Ship the light and dark themes together. The review reads the gallery at three widths.",
        ),
    ];
    let content: Vec<Block> = if active_view().as_deref() == Some("outline") {
        content
            .into_iter()
            .filter(|b| {
                matches!(
                    b.kind,
                    BlockKind::Title | BlockKind::Heading | BlockKind::Item
                )
            })
            .collect()
    } else {
        content
    };
    let participant = |id: &str, name: &str, palette: &str, spend: f64| Participant {
        id: id.into(),
        name: name.into(),
        palette_key: Some(palette.into()),
        picture: None,
        wash_key: None,
        spend: Some(spend),
    };
    let participants = vec![
        participant("u1", "Ada", "blue", 0.02),
        participant("u2", "Ben", "amber", 0.01),
        participant("u3", "Dee", "violet", 0.03),
        participant("u4", "Cy", "emerald", 0.01),
    ];
    let at = |block: &str, offset: Option<usize>, activity: Option<&str>, typing: bool| Position {
        block_id: block.into(),
        offset,
        in_proposal: None,
        selection: None,
        activity: activity.map(Into::into),
        is_typing: typing,
    };
    let positions = BTreeMap::from([
        ("u1".to_string(), at("b3", Some(4), Some("editing"), true)),
        (
            "u2".to_string(),
            at("b4", Some(7), Some("reviewing"), false),
        ),
        (
            "u3".to_string(),
            Position {
                selection: Some(0..41),
                ..at("b8", Some(0), Some("drafting notes"), false)
            },
        ),
        (
            "u4".to_string(),
            at("b6", Some(12), Some("checking"), false),
        ),
    ]);
    let pins = vec![Pin {
        id: "pin1".into(),
        participant_id: "u1".into(),
        block_id: "b8".into(),
        offset: Some(4),
        horizontal_fraction: None,
        vertical_offset: None,
        body: "Can we name the owner?".into(),
        is_open: false,
    }];
    let proposals = vec![
        Proposal {
            id: "pr1".into(),
            participant_id: "u2".into(),
            block_id: "b3".into(),
            offset: 0,
            removal: Some("Ship".into()),
            insertion: Some("Land".into()),
            reason: Some("Tighter verb".into()),
        },
        Proposal {
            id: "pr2".into(),
            participant_id: "u3".into(),
            block_id: "b5".into(),
            offset: 29,
            removal: None,
            insertion: Some(" and light".into()),
            reason: Some("Covers both themes".into()),
        },
    ];
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
                heading: "Workspace / Launch plan",
                page_height: 520,
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
    let row = |key: &str, name: &str, kind: RowKind, note: &str| RowSpec {
        row_key: key.into(),
        display_name: name.into(),
        kind,
        tint_key: None,
        mark_source: None,
        note: Some(note.into()),
    };
    let rows = vec![
        row(
            "maintainer",
            "Maintainer",
            RowKind::Person,
            "reviewing later",
        ),
        row("scout", "Scout", RowKind::Agent, "reading the index"),
        row("drafter", "Drafter", RowKind::Agent, "writing the summary"),
        row("auditor", "Auditor", RowKind::Agent, "comparing totals"),
        row("courier", "Courier", RowKind::Agent, "holding the batch"),
    ];
    let span = |row: &str,
                mode: SpanMode,
                begin: f64,
                end: Option<f64>,
                tool: Option<&str>,
                caption: Option<&str>| Span {
        span_key: None,
        row_key: row.into(),
        mode,
        begin_s: begin,
        end_s: end,
        tool_name: tool.map(Into::into),
        caption: caption.map(Into::into),
    };
    let spans = vec![
        span(
            "scout",
            SpanMode::Tool,
            8.0,
            Some(26.0),
            Some("fetch_index"),
            None,
        ),
        span("drafter", SpanMode::Reasoning, 26.0, Some(44.0), None, None),
        span("auditor", SpanMode::Reasoning, 44.0, None, None, None),
    ];
    let transfers = vec![Transfer {
        transfer_key: None,
        from_row_key: "drafter".into(),
        to_row_key: "auditor".into(),
        at_s: 44.0,
        caption: None,
    }];
    rsx! {
        Item { slug: "run-timeline-rows", name: "RunTimelineRows",
            RunTimelineRows {
                run_heading: "Refund batch",
                rows,
                spans,
                transfers,
                clock_s: Some(52.0),
                axis_length_s: Some(60.0),
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
    let entry = |id: &str,
                 label: &str,
                 state: RunState,
                 follows: Option<&str>,
                 started: u64,
                 files: u32,
                 spend: f64| {
        CheckpointEntry {
            id: id.into(),
            label: label.into(),
            glyph: None,
            follows_id: follows.map(|f| f.to_string()),
            run_state: state,
            started_ms: Some(started),
            file_count: Some(files),
            spend: Some(spend),
        }
    };
    let ago = |minutes: u64| gallery_now() - minutes * 60_000;
    let mut entries = use_signal(|| {
        vec![
            CheckpointEntry {
                glyph: Some("P".into()),
                ..entry(
                    "c1",
                    "Plan approved",
                    RunState::Done,
                    None,
                    ago(41),
                    3,
                    0.04,
                )
            },
            entry(
                "c2",
                "Drafted section 1",
                RunState::Done,
                Some("c1"),
                ago(32),
                5,
                0.09,
            ),
            entry(
                "c3",
                "Review pass",
                RunState::Running,
                Some("c2"),
                ago(4),
                1,
                0.02,
            ),
        ]
    });
    let mut current = use_signal(|| Some("c3".to_string()));
    let mut branched = use_signal(|| {
        vec![
            CheckpointEntry {
                glyph: Some("P".into()),
                ..entry(
                    "c1",
                    "Plan approved",
                    RunState::Done,
                    None,
                    ago(41),
                    3,
                    0.04,
                )
            },
            entry(
                "c2",
                "Drafted section 1",
                RunState::Done,
                Some("c1"),
                ago(32),
                5,
                0.09,
            ),
            CheckpointEntry {
                glyph: Some("T".into()),
                ..entry(
                    "c3",
                    "Figures and tables",
                    RunState::Failed,
                    Some("c2"),
                    ago(17),
                    2,
                    0.05,
                )
            },
            entry(
                "c4",
                "Rewrite section 2",
                RunState::Running,
                Some("c2"),
                ago(12),
                3,
                0.08,
            ),
        ]
    });
    let mut branched_current = use_signal(|| Some("c4".to_string()));
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
        Item { slug: "run-checkpoint-rail-branched", name: "RunCheckpointRail",
            RunCheckpointRail {
                entries: branched(),
                current_id: branched_current(),
                goal_text: "Write the report",
                clock_ms: Some(gallery_now()),
                on_restart_from: move |e: CheckpointEntry| {
                    last.set(format!("restart {}", e.id));
                    restarts += 1;
                    let next_id = format!("r{}", restarts());
                    let started = gallery_now();
                    branched.with_mut(|list| {
                        *list = append_restart(list, &e.id, &next_id, &format!("Restarted from {}", e.label));
                        if let Some(added) = list.last_mut() {
                            added.started_ms = Some(started);
                        }
                    });
                    branched_current.set(Some(next_id));
                },
            }
            div { class: "gl-out", "data-out": "run-checkpoint-rail-branched", "{last()}" }
        }
    }
}

#[component]
fn EditableStepPlanDemo() -> Element {
    use rv2_kit::agents::editable_step_plan::{EditableStepPlan, PlanPhase, PlanStep, StepState};
    let mut last = use_signal(String::new);
    let mut phase = use_signal(|| PlanPhase::Drafting);
    let step = |key: &str, title: &str, detail: Option<&str>, tool: Option<&str>| PlanStep {
        step_key: key.into(),
        step_title: title.into(),
        step_detail: detail.map(Into::into),
        tool_label: tool.map(Into::into),
        step_state: StepState::Pending,
        measured_ms: None,
        error_text: None,
    };
    let mut steps = use_signal(|| {
        vec![
            step("s1", "Read the brief", None, Some("Files")),
            step("s2", "Draft outline", Some("Three sections"), None),
            step("s3", "Pull figures from the sheet", None, Some("Sheets")),
            step(
                "s4",
                "Check claims against sources",
                Some("Two claims flagged"),
                Some("Search"),
            ),
            step("s5", "Write the summary", Some("Under 300 words"), None),
        ]
    });
    let run = |plan: Vec<PlanStep>| -> Vec<PlanStep> {
        plan.into_iter()
            .enumerate()
            .map(|(i, mut item)| {
                let (state, measured, error) = match i {
                    0 => (StepState::Succeeded, Some(42_000), None),
                    1 => (StepState::Succeeded, Some(96_000), None),
                    2 => (
                        StepState::Errored,
                        Some(18_000),
                        Some("Sheet not shared with the agent".to_string()),
                    ),
                    3 => (StepState::InProgress, None, None),
                    _ => (StepState::Pending, None, None),
                };
                item.step_state = state;
                item.measured_ms = measured;
                item.error_text = error;
                item
            })
            .collect()
    };
    let estimate = format!("{} steps", steps.read().len());
    rsx! {
        Item { slug: "editable-step-plan", name: "EditableStepPlan",
            EditableStepPlan {
                plan_heading: "Plan",
                plan_steps: steps(),
                plan_phase: phase(),
                estimate_text: Some(estimate),
                on_start: move |s: Vec<PlanStep>| {
                    last.set(format!("start {} steps", s.len()));
                    steps.set(run(s));
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
