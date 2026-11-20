use dioxus::prelude::*;

use super::{Item, NOW_MS, Section, gallery_now};

#[component]
pub fn Graphs() -> Element {
    rsx! {
        Section { id: "graphs", title: "Graphs",
            AgentContextMapDemo {}
            AgentToolReachMapDemo {}
            AutomationFlowEditorDemo {}
            BranchChoiceMapDemo {}
            CauseAnalysisTreeDemo {}
            ModelFailoverFlowDemo {}
            ParallelEditOverlapGridDemo {}
            ParallelWorkGridDemo {}
            SheetGridDemo {}
            TaskAssignmentBoardDemo {}
            TrafficSplitRouterDemo {}
            WorkspaceTerritoryMapDemo {}
        }
    }
}

#[component]
fn AgentContextMapDemo() -> Element {
    use rv2_kit::components::agent_context_map::{
        AgentContextMap, AgentNode, ItemKind, Memory, SourceArtifact, Tool,
    };
    let mut last = use_signal(String::new);
    let memory = |id: &str, text: &str, outdated: bool| Memory {
        memory_id: id.into(),
        memory_text: text.into(),
        memory_kind: Some("Preference".into()),
        memory_source_line: Some("From chat".into()),
        source_artifact: SourceArtifact {
            artifact_name: "Chat, Mar 3".into(),
            artifact_kind: None,
            artifact_mark: None,
            artifact_link: None,
        },
        learned_via_tool_id: None,
        replaces_memory_id: None,
        is_outdated: outdated,
        related_memory_ids: vec![],
        learned_at: Some(NOW_MS as f64),
        use_count: 3,
        is_disabled: false,
    };
    let tool = |id: &str, name: &str| Tool {
        tool_id: id.into(),
        tool_mark: None,
        tool_name: name.into(),
        access_level: "Read".into(),
        scope_text: "Workspace".into(),
        last_activity_text: "2 min ago".into(),
        last_used_at: Some(NOW_MS as f64),
        call_count: 7,
        is_disabled: false,
    };
    rsx! {
        Item { slug: "agent-context-map", name: "AgentContextMap",
            AgentContextMap {
                agent_node: AgentNode { agent_id: "writer".into(), agent_name: "Writer".into(), agent_subtitle: "Drafts copy".into(), agent_mark: None },
                memory_items: vec![
                    memory("m1", "Prefers short sentences", false),
                    memory("m2", "Old pricing tier", true),
                    memory("m3", "Signs off as Maya", false),
                    memory("m4", "Uses British spelling", false),
                    memory("m5", "Avoids passive voice", false),
                    memory("m6", "Brand colour is slate", false),
                    {
                        let mut replacing = memory("m7", "Pricing tier: Team", false);
                        replacing.replaces_memory_id = Some("m2".into());
                        replacing
                    },
                ],
                tool_items: vec![tool("t1", "Docs search"), tool("t2", "Calendar")],
                active_tool_ids: vec!["t1".to_string()],
                recalled_memory_ids: vec!["m1".to_string()],
                available_tools: vec![tool("t3", "Mail draft")],
                heading_text: "Writer context",
                clock_ms: Some(gallery_now() as f64),
                on_switch_change: move |(k, id, on): (ItemKind, String, bool)| last.set(format!("switch {k:?} {id} {on}")),
                on_forget: move |m: Memory| last.set(format!("forget {}", m.memory_id)),
                on_add_tool: move |t: Tool| last.set(format!("add tool {}", t.tool_id)),
                on_select_memory: move |m: Option<Memory>| last.set(format!("select {}", m.map(|x| x.memory_id).unwrap_or("none".into()))),
            }
            div { class: "gl-out", "data-out": "agent-context-map", "{last()}" }
        }
    }
}

#[component]
fn AgentToolReachMapDemo() -> Element {
    use rv2_kit::components::agent_tool_reach_map::{
        AccessTool, AgentState, AgentToolReachMap, CallState, Grant, MapAgent, ReplayInput,
        ToolCall,
    };
    let mut last = use_signal(String::new);
    let mut selected = use_signal(|| Some("a1".to_string()));
    let call = |id: &str, state: CallState| ToolCall {
        call_id: id.into(),
        call_state: state,
        call_input: "{\"q\":\"pricing\"}".into(),
        call_output: Some("3 results".into()),
        call_error: None,
        called_at: 1.0,
        duration: Some(0.4),
        is_replay: false,
    };
    let agents = vec![
        MapAgent {
            agent_id: "a1".into(),
            agent_name: "Research".into(),
            model_name: Some("model A".into()),
            agent_mark: None,
            agent_state: AgentState::Running,
            agent_task: Some("Collect pricing".into()),
            granted_tools: vec![
                Grant {
                    tool_id: "search".into(),
                    call_list: vec![
                        call("c1", CallState::Done),
                        call("c2", CallState::Failed),
                        call("c3", CallState::Running),
                    ],
                },
                Grant {
                    tool_id: "mail".into(),
                    call_list: vec![],
                },
            ],
        },
        MapAgent {
            agent_id: "a2".into(),
            agent_name: "Writer".into(),
            model_name: None,
            agent_mark: None,
            agent_state: AgentState::Queued,
            agent_task: None,
            granted_tools: vec![],
        },
    ];
    let catalog = vec![
        AccessTool {
            tool_id: "search".into(),
            tool_name: "Web search".into(),
            tool_mark: None,
            tool_summary: Some("Public web".into()),
        },
        AccessTool {
            tool_id: "mail".into(),
            tool_name: "Mail".into(),
            tool_mark: None,
            tool_summary: None,
        },
    ];
    rsx! {
        Item { slug: "agent-tool-reach-map", name: "AgentToolReachMap",
            AgentToolReachMap {
                agent_list: agents,
                tool_catalog: catalog,
                selected_agent_id: selected(),
                on_agent_pick: move |id: String| {
                    last.set(format!("pick {id}"));
                    selected.set(Some(id));
                },
                on_replay: move |(c, _, t, a): (ToolCall, ReplayInput, AccessTool, MapAgent)| {
                    last.set(format!("replay {} {} {}", c.call_id, t.tool_id, a.agent_id));
                },
                board_title: "Tool access",
            }
            div { class: "gl-out", "data-out": "agent-tool-reach-map", "{last()}" }
        }
    }
}

#[component]
fn AutomationFlowEditorDemo() -> Element {
    use rv2_kit::components::automation_flow_editor::{
        AutomationFlowEditor, RunResult, Step, StepIcon, StepKind, Template,
    };
    let mut last = use_signal(String::new);
    let step = |id: &str, kind: StepKind, title: &str, icon: StepIcon| Step {
        step_id: id.into(),
        step_kind: kind,
        step_title: title.into(),
        step_detail: Some("Runs on every new ticket".into()),
        step_note: None,
        step_icon: Some(icon),
        branch_paths: vec![],
    };
    let steps = vec![
        step("s1", StepKind::Trigger, "New ticket", StepIcon::Webhook),
        step(
            "s2",
            StepKind::Action,
            "Summarise with agent",
            StepIcon::Flash,
        ),
        step("s3", StepKind::Action, "Post to chat", StepIcon::Chat),
        step("s4", StepKind::Action, "Close the ticket", StepIcon::Flash),
    ];
    let template = |kind: StepKind, title: &str, icon: StepIcon| Template {
        step_kind: kind,
        step_title: title.into(),
        step_detail: Some("Added from the library".into()),
        step_icon: Some(icon),
        branch_paths: vec![],
    };
    rsx! {
        Item { slug: "automation-flow-editor", name: "AutomationFlowEditor",
            AutomationFlowEditor {
                flow_name: "Ticket triage",
                default_flow_steps: steps,
                insert_templates: vec![
                    template(StepKind::Action, "Tag by priority", StepIcon::Flash),
                    template(StepKind::Action, "Notify on-call", StepIcon::Chat),
                    Template {
                        step_kind: StepKind::Branch,
                        step_title: "Split by priority".into(),
                        step_detail: Some("Urgent or routine".into()),
                        step_icon: Some(StepIcon::Flash),
                        branch_paths: vec!["Urgent".into(), "Routine".into()],
                    },
                ],
                trigger_templates: vec![template(StepKind::Trigger, "Ticket updated", StepIcon::Webhook)],
                run_results: vec![
                    ("s1".to_string(), RunResult::Done),
                    ("s2".to_string(), RunResult::Failed),
                    ("s3".to_string(), RunResult::InProgress),
                    ("s4".to_string(), RunResult::Skipped),
                ],
                on_flow_change: move |s: Vec<Step>| last.set(format!("flow {} steps", s.len())),
                on_select: move |id: Option<String>| last.set(format!("select {id:?}")),
                on_run: move |_| last.set("run".into()),
                on_stop: move |_| last.set("stop".into()),
                canvas_height: 360,
            }
            div { class: "gl-out", "data-out": "automation-flow-editor", "{last()}" }
        }
    }
}

#[component]
fn BranchChoiceMapDemo() -> Element {
    use rv2_kit::components::branch_choice_map::{
        Branch, BranchChoiceMap, Consequence, Fork, Risk,
    };
    let last = use_signal(String::new);
    let branch = |label: &str, text: &str, preferred: bool| Branch {
        branch_label: label.into(),
        branch_mark: None,
        consequences: vec![Consequence {
            text: text.into(),
            flag_explanation: None,
            reference_url: None,
        }],
        touched_apps: vec!["Mail".into()],
        agent_preferred: preferred,
        follow_up: None,
    };
    let forks = vec![Fork {
        fork_id: "f1".into(),
        question_text: "Reply to the vendor now or wait?".into(),
        context_text: Some("The quote expires Friday.".into()),
        agent_name: "Procurement agent".into(),
        model_name: Some("model A".into()),
        agent_mark: None,
        risk_level: Some(Risk::Medium),
        branches: vec![
            branch("Reply now", "Vendor confirms today", true),
            branch("Wait", "Quote may lapse", false),
        ],
    }];
    rsx! {
        Item { slug: "branch-choice-map", name: "BranchChoiceMap",
            BranchChoiceMap {
                pending_forks: forks,
                heading_text: "Decisions",
                allow_custom: true,
            }
            div { class: "gl-out", "data-out": "branch-choice-map", "{last()}" }
        }
    }
}

#[component]
fn CauseAnalysisTreeDemo() -> Element {
    use rv2_kit::components::cause_analysis_tree::{
        Cause, CauseAction, CauseAnalysisTree, CheckState, Evidence, JoinRule,
    };
    let mut last = use_signal(String::new);
    let cause = |id: &str, title: &str, state: CheckState, subs: Vec<Cause>| Cause {
        cause_id: id.into(),
        cause_title: title.into(),
        cause_mark: None,
        file_paths: vec!["src/parser.rs".into()],
        join_rule: JoinRule::All,
        check_state: state,
        verdict_text: Some("Checked against the logs".into()),
        evidence: Some(Evidence {
            evidence_kind: "log".into(),
            evidence_source: Some("ci.log".into()),
            evidence_content: "unexpected EOF at line 42".into(),
        }),
        checked_at: Some(12.0),
        sub_causes: subs,
    };
    let root = cause(
        "root",
        "Build failed",
        CheckState::Open,
        vec![
            cause("c1", "Parser rejects input", CheckState::Confirmed, vec![]),
            cause("c2", "Cache is stale", CheckState::RuledOut, vec![]),
        ],
    );
    rsx! {
        Item { slug: "cause-analysis-tree", name: "CauseAnalysisTree",
            CauseAnalysisTree {
                failure_node: root,
                failure_detail: Some("exit code 2 in CI".to_string()),
                board_title: "Root cause",
                on_cause_action: move |(c, a): (Cause, CauseAction)| last.set(format!("{a:?} {}", c.cause_id)),
            }
            div { class: "gl-out", "data-out": "cause-analysis-tree", "{last()}" }
        }
    }
}

#[component]
fn ModelFailoverFlowDemo() -> Element {
    use rv2_kit::components::model_failover_flow::{Attempt, AttemptState, End, ModelFailoverFlow};
    let mut last = use_signal(String::new);
    let attempt = |id: &str, model: &str, state: AttemptState, err: Option<&str>| Attempt {
        attempt_id: id.into(),
        model_name: model.into(),
        provider_name: "Provider".into(),
        attempt_state: state,
        duration: Some(1_300.0),
        attempt_cost: Some(0.02),
        error_text: err.map(|e| e.to_string()),
    };
    rsx! {
        Item { slug: "model-failover-flow", name: "ModelFailoverFlow",
            ModelFailoverFlow {
                attempt_list: vec![
                    attempt("a1", "Model A", AttemptState::Failed, Some("429 rate limited")),
                    attempt("a2", "Model B", AttemptState::InProgress, None),
                    attempt("a3", "Model C", AttemptState::Queued, None),
                    attempt("a4", "Model D", AttemptState::Queued, None),
                ],
                heading_text: "Reply request",
                routing_label: Some("Primary then fallback".to_string()),
                origin_node: Some(End { node_name: "Request".into(), node_mark: None, node_text: Some("User asked".into()), node_details: vec![] }),
                result_node: Some(End { node_name: "Reply".into(), node_mark: None, node_text: Some("Delivered".into()), node_details: vec!["Model B".into()] }),
                elapsed_ms: Some(2_600.0),
                on_retry: move |_| last.set("retry".into()),
            }
            div { class: "gl-out", "data-out": "model-failover-flow", "{last()}" }
        }
        Item { slug: "model-failover-flow-exhausted", name: "ModelFailoverFlow (exhausted)",
            ModelFailoverFlow {
                attempt_list: vec![
                    attempt("e1", "Model A", AttemptState::Failed, Some("429 rate limited")),
                    attempt("e2", "Model B", AttemptState::Failed, Some("503 unavailable")),
                    attempt("e3", "Model C", AttemptState::Failed, Some("Timed out")),
                ],
                heading_text: "Reply request",
                routing_label: Some("Primary then fallback".to_string()),
                origin_node: Some(End { node_name: "Request".into(), node_mark: None, node_text: Some("User asked".into()), node_details: vec![] }),
                result_node: Some(End { node_name: "Reply".into(), node_mark: None, node_text: None, node_details: vec![] }),
                elapsed_ms: Some(9_400.0),
                on_retry: move |_| last.set("retry".into()),
            }
            div { class: "gl-out", "data-out": "model-failover-flow-exhausted", "{last()}" }
        }
    }
}

#[component]
fn ParallelEditOverlapGridDemo() -> Element {
    use rv2_kit::components::parallel_edit_overlap_grid::{
        ConflictOption, Contributor, Edit, FileRow, ParallelEditOverlapGrid,
    };
    let mut last = use_signal(String::new);
    let contributor = |id: &str, name: &str| Contributor {
        contributor_id: id.into(),
        contributor_name: name.into(),
        model_name: Some("model A".into()),
        branch_name: Some(format!("feat/{id}")),
        contributor_mark: None,
        is_active: true,
    };
    let edit = |id: &str, range: (u32, u32), editing: bool| Edit {
        contributor_id: id.into(),
        line_range: range,
        is_editing: editing,
        edit_note: None,
    };
    let rows = vec![
        FileRow {
            file_path: "src/api.rs".into(),
            edits: vec![edit("ada", (10, 40), true), edit("ben", (30, 60), true)],
            conflict: Some(vec![
                ConflictOption {
                    contributor_id: "ada".into(),
                    option_text: "Keep Ada's version".into(),
                },
                ConflictOption {
                    contributor_id: "ben".into(),
                    option_text: "Keep Ben's version".into(),
                },
            ]),
        },
        FileRow {
            file_path: "src/lib.rs".into(),
            edits: vec![edit("ada", (1, 5), false)],
            conflict: None,
        },
    ];
    rsx! {
        Item { slug: "parallel-edit-overlap-grid", name: "ParallelEditOverlapGrid",
            ParallelEditOverlapGrid {
                contributors: vec![contributor("ada", "Ada"), contributor("ben", "Ben")],
                file_rows: rows,
                section_title: "Parallel edits",
                on_choose_option: move |(file, who): (String, String)| last.set(format!("choose {file} {who}")),
                on_merge_request: move |_| last.set("merge".into()),
            }
            div { class: "gl-out", "data-out": "parallel-edit-overlap-grid", "{last()}" }
        }
    }
}

#[component]
fn ParallelWorkGridDemo() -> Element {
    use rv2_kit::components::parallel_work_grid::{Item as GridItem, ItemState, ParallelWorkGrid};
    let mut last = use_signal(String::new);
    let item = |id: &str, state: ItemState, start: f64, end: Option<f64>| GridItem {
        item_id: id.into(),
        item_label: format!("Chunk {id}"),
        item_state: state,
        worker_name: Some("Worker".into()),
        start_offset: Some(start),
        end_offset: end,
        item_cost: Some(0.03),
        attempt_count: 1,
        output_value: None,
        error_text: if state == ItemState::Failed {
            Some("timeout".into())
        } else {
            None
        },
    };
    let items: Vec<GridItem> = (1..=240)
        .map(|n: u32| {
            let id = n.to_string();
            let start = f64::from(n % 7);
            match n % 10 {
                0 => item(&id, ItemState::Failed, start, Some(start + 2.0)),
                1..=5 => item(
                    &id,
                    ItemState::Done,
                    start,
                    Some(start + 3.0 + f64::from(n % 4)),
                ),
                6 => item(&id, ItemState::InProgress, start, None),
                7 => item(&id, ItemState::Retrying, start, None),
                _ => item(&id, ItemState::Queued, start, None),
            }
        })
        .collect();
    rsx! {
        Item { slug: "parallel-work-grid", name: "ParallelWorkGrid",
            ParallelWorkGrid {
                job_items: items,
                job_name: "Translate 240 chunks",
                concurrency: Some(3),
                clock_offset: 6.0,
                on_retry: move |ids: Vec<String>| last.set(format!("retry {}", ids.join(","))),
                on_pick: move |i: Option<GridItem>| last.set(format!("pick {}", i.map(|x| x.item_id).unwrap_or("none".into()))),
            }
            div { class: "gl-out", "data-out": "parallel-work-grid", "{last()}" }
        }
    }
}

#[component]
fn SheetGridDemo() -> Element {
    use rv2_kit::components::sheet_grid::{Column, ColumnKind, GridRow, SheetGrid, TagTone};
    use std::collections::BTreeMap;
    let mut last = use_signal(String::new);
    let mut rows = use_signal(|| {
        vec![
            GridRow {
                row_id: "r1".into(),
                row_logo: None,
                values: BTreeMap::from([
                    ("name".to_string(), "Acme".to_string()),
                    ("stage".to_string(), "Lead".to_string()),
                ]),
            },
            GridRow {
                row_id: "r2".into(),
                row_logo: None,
                values: BTreeMap::from([
                    ("name".to_string(), "Globex".to_string()),
                    ("stage".to_string(), "Won".to_string()),
                ]),
            },
        ]
    });
    let columns = vec![
        Column {
            column_key: "name".into(),
            column_label: "Company".into(),
            column_width: Some(180),
            column_kind: ColumnKind::Title,
            cells_editable: true,
            tag_choices: vec![],
        },
        Column {
            column_key: "stage".into(),
            column_label: "Stage".into(),
            column_width: Some(120),
            column_kind: ColumnKind::Tag,
            cells_editable: true,
            tag_choices: vec!["Lead".into(), "Won".into()],
        },
    ];
    rsx! {
        Item { slug: "sheet-grid", name: "SheetGrid",
            SheetGrid {
                grid_heading: "Accounts",
                grid_columns: columns,
                grid_rows: rows(),
                tag_tones: vec![TagTone { tag: "Won".into(), tone: rv2_kit::components::sheet_grid::Tone::Positive }, TagTone { tag: "Lead".into(), tone: rv2_kit::components::sheet_grid::Tone::Info }],
                on_rows_change: move |next: Vec<GridRow>| {
                    last.set(format!("rows {}", next.len()));
                    rows.set(next);
                },
                on_checked_change: move |ids: Vec<String>| last.set(format!("checked {}", ids.len())),
            }
            div { class: "gl-out", "data-out": "sheet-grid", "{last()}" }
        }
    }
}

#[component]
fn TaskAssignmentBoardDemo() -> Element {
    use rv2_kit::components::task_assignment_board::{
        AssignMode, Proposal, TaskAssignmentBoard, WorkItem, Worker,
    };
    let mut last = use_signal(String::new);
    let mut mode = use_signal(|| AssignMode::Manual);
    let workers = vec![
        Worker {
            worker_id: "w1".into(),
            worker_name: "Model A".into(),
            worker_model_name: Some("model A".into()),
            worker_avatar: None,
            busy_minutes: 25.0,
            cost_per_minute: 0.02,
        },
        Worker {
            worker_id: "w2".into(),
            worker_name: "Model B".into(),
            worker_model_name: None,
            worker_avatar: None,
            busy_minutes: 5.0,
            cost_per_minute: 0.05,
        },
    ];
    let work = vec![
        WorkItem {
            work_id: "k1".into(),
            work_title: "Summarise calls".into(),
            estimate_minutes: 10.0,
            required_tools: vec!["Mail".into()],
            assigned_worker_id: None,
        },
        WorkItem {
            work_id: "k2".into(),
            work_title: "Draft reply".into(),
            estimate_minutes: 20.0,
            required_tools: vec![],
            assigned_worker_id: Some("w1".into()),
        },
        WorkItem {
            work_id: "k3".into(),
            work_title: "Tag new leads".into(),
            estimate_minutes: 15.0,
            required_tools: vec!["CRM".into()],
            assigned_worker_id: None,
        },
        WorkItem {
            work_id: "k4".into(),
            work_title: "Check invoices".into(),
            estimate_minutes: 8.0,
            required_tools: vec![],
            assigned_worker_id: Some("w2".into()),
        },
        WorkItem {
            work_id: "k5".into(),
            work_title: "Send weekly recap".into(),
            estimate_minutes: 12.0,
            required_tools: vec!["Mail".into()],
            assigned_worker_id: None,
        },
    ];
    rsx! {
        Item { slug: "task-assignment-board", name: "TaskAssignmentBoard",
            TaskAssignmentBoard {
                worker_list: workers,
                work_items: work,
                assignment_mode: Some(mode()),
                on_mode_change: move |m: AssignMode| {
                    last.set(format!("mode {m:?}"));
                    mode.set(m);
                },
                on_assign: move |(w, who): (String, Option<String>)| last.set(format!("assign {w} to {}", who.unwrap_or("nobody".into()))),
                proposed_assignment: Some(Proposal { work_id: "k1".into(), worker_id: "w2".into() }),
                clock_minutes: 30.0,
            }
            div { class: "gl-out", "data-out": "task-assignment-board", "{last()}" }
        }
    }
}

#[component]
fn TrafficSplitRouterDemo() -> Element {
    use rv2_kit::components::traffic_split_router::{
        Fallback, Policy, Route, Strategy, TrafficSplitRouter,
    };
    let mut last = use_signal(String::new);
    let routes = vec![
        Route {
            route_id: "fast".into(),
            route_name: "Fast model".into(),
            route_provider: Some("Provider A".into()),
            route_mark: None,
            route_tone: None,
            cost_per_1k: 0.4,
            p95_seconds: 1.2,
        },
        Route {
            route_id: "deep".into(),
            route_name: "Deep model".into(),
            route_provider: Some("Provider B".into()),
            route_mark: None,
            route_tone: None,
            cost_per_1k: 1.9,
            p95_seconds: 4.8,
        },
    ];
    let strategies = vec![
        Strategy {
            strategy_id: "even".into(),
            strategy_label: "Even split".into(),
            share_map: vec![("fast".into(), 0.5), ("deep".into(), 0.5)],
        },
        Strategy {
            strategy_id: "fast".into(),
            strategy_label: "Mostly fast".into(),
            share_map: vec![("fast".into(), 0.9), ("deep".into(), 0.1)],
        },
    ];
    let live = Policy {
        active_strategy: "even".into(),
        fallback: Fallback {
            enabled: true,
            from_route_id: Some("fast".into()),
            to_route_id: Some("deep".into()),
        },
    };
    rsx! {
        Item { slug: "traffic-split-router", name: "TrafficSplitRouter",
            TrafficSplitRouter {
                route_list: routes,
                strategy_list: strategies,
                live_policy: live.clone(),
                environment_label: "Production",
                on_draft_change: move |p: Policy| last.set(format!("draft {}", p.active_strategy)),
                on_deploy: move |p: Policy| last.set(format!("deploy {}", p.active_strategy)),
            }
            div { class: "gl-out", "data-out": "traffic-split-router", "{last()}" }
        }
    }
}

#[component]
fn WorkspaceTerritoryMapDemo() -> Element {
    use rv2_kit::components::workspace_territory_map::{
        AgentAction, Change, RepoAgent, RepoFile, WorkspaceTerritoryMap,
    };
    let last = use_signal(String::new);
    let files = vec![
        RepoFile {
            file_path: "src/api/routes.rs".into(),
            line_count: 320,
            change_log: vec![Change {
                agent_id: "a1".into(),
                lines_added: Some(20),
                lines_removed: Some(4),
            }],
        },
        RepoFile {
            file_path: "src/api/auth.rs".into(),
            line_count: 140,
            change_log: vec![],
        },
        RepoFile {
            file_path: "web/app.tsx".into(),
            line_count: 210,
            change_log: vec![Change {
                agent_id: "a2".into(),
                lines_added: Some(8),
                lines_removed: Some(1),
            }],
        },
    ];
    let agents = vec![
        RepoAgent {
            agent_id: "a1".into(),
            agent_name: "API agent".into(),
            agent_mark: None,
            current_file: Some("src/api/routes.rs".into()),
            agent_action: AgentAction::Edit,
            agent_task: Some("Add rate limit".into()),
        },
        RepoAgent {
            agent_id: "a2".into(),
            agent_name: "UI agent".into(),
            agent_mark: None,
            current_file: Some("web/app.tsx".into()),
            agent_action: AgentAction::Read,
            agent_task: None,
        },
    ];
    rsx! {
        Item { slug: "workspace-territory-map", name: "WorkspaceTerritoryMap",
            WorkspaceTerritoryMap {
                file_list: files,
                agent_list: agents,
                repo_name: "rv2",
                branch_name: Some("main".to_string()),
                folder_depth: 2,
            }
            div { class: "gl-out", "data-out": "workspace-territory-map", "{last()}" }
        }
    }
}
