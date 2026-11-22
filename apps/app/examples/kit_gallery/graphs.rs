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
    let memory = |id: &str, text: &str, outdated: bool, source: (&str, &str)| Memory {
        memory_id: id.into(),
        memory_text: text.into(),
        memory_kind: Some("Preference".into()),
        memory_source_line: Some("Learned from you".into()),
        source_artifact: SourceArtifact {
            artifact_name: source.0.into(),
            artifact_kind: Some(source.1.into()),
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
                    memory("m1", "Prefers short sentences", false, ("Chat, Mar 3", "Chat")),
                    memory("m2", "Old pricing tier", true, ("Pricing sheet", "Sheet")),
                    {
                        let mut replacing = memory("m3", "Pricing tier: Team", false, ("Pricing sheet", "Sheet"));
                        replacing.replaces_memory_id = Some("m2".into());
                        replacing
                    },
                    memory("m4", "Signs off as Maya", false, ("Style guide", "Doc")),
                ],
                tool_items: vec![
                    tool("t1", "Docs search"),
                    tool("t2", "Calendar"),
                    tool("t3", "Mail"),
                    tool("t4", "CRM"),
                    tool("t5", "Sheets"),
                ],
                active_tool_ids: vec!["t1".to_string()],
                recalled_memory_ids: vec!["m1".to_string()],
                available_tools: vec![tool("t6", "Drive")],
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
    let mut selected = use_signal(|| Some("a3".to_string()));
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
        MapAgent {
            agent_id: "a3".into(),
            agent_name: "Reviewer".into(),
            model_name: Some("model B".into()),
            agent_mark: None,
            agent_state: AgentState::Waiting,
            agent_task: Some("Check citations".into()),
            granted_tools: vec![
                Grant {
                    tool_id: "search".into(),
                    call_list: vec![call("c3", CallState::Done), call("c4", CallState::Failed)],
                },
                Grant {
                    tool_id: "mail".into(),
                    call_list: vec![call("c6", CallState::Done)],
                },
                Grant {
                    tool_id: "docs".into(),
                    call_list: vec![call("c5", CallState::Running)],
                },
                Grant {
                    tool_id: "crm".into(),
                    call_list: vec![call("c7", CallState::Done), call("c8", CallState::Done)],
                },
                Grant {
                    tool_id: "calendar".into(),
                    call_list: vec![call("c9", CallState::Done)],
                },
                Grant {
                    tool_id: "sheets".into(),
                    call_list: vec![],
                },
            ],
        },
    ];
    let tool = |id: &str, name: &str, summary: Option<&str>| AccessTool {
        tool_id: id.into(),
        tool_name: name.into(),
        tool_mark: None,
        tool_summary: summary.map(str::to_string),
    };
    let catalog = vec![
        tool("search", "Web search", Some("Public web")),
        tool("mail", "Mail", None),
        tool("docs", "Docs", Some("Team drive")),
        tool("crm", "CRM", None),
        tool("calendar", "Calendar", None),
        tool("sheets", "Sheets", None),
        tool("drive", "Drive", Some("Team drive")),
        tool("tickets", "Tickets", None),
        tool("billing", "Billing", None),
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
        step(
            "s5",
            StepKind::Wait,
            "Wait for customer reply",
            StepIcon::Wait,
        ),
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
            }
            div { class: "gl-out", "data-out": "automation-flow-editor", "{last()}" }
        }
    }
}

#[component]
fn BranchChoiceMapDemo() -> Element {
    use rv2_kit::components::branch_choice_map::{
        Branch, BranchChoiceMap, Consequence, FollowUp, Fork, Risk,
    };
    let last = use_signal(String::new);
    let plain = |text: &str| Consequence {
        text: text.into(),
        flag_explanation: None,
        reference_url: None,
    };
    let flagged = |text: &str, note: &str| Consequence {
        text: text.into(),
        flag_explanation: Some(note.into()),
        reference_url: None,
    };
    let branch = |label: &str, consequences: Vec<Consequence>, preferred: bool| Branch {
        branch_label: label.into(),
        branch_mark: None,
        consequences,
        touched_apps: vec!["Billing DB".into()],
        agent_preferred: preferred,
        follow_up: None,
    };
    let option = |label: &str| branch(label, vec![], false);
    let migration = Fork {
        fork_id: "f1".into(),
        question_text:
            "Apply the schema change to the live table now, or after the replica catches up?".into(),
        context_text: Some("The table holds 4.2M rows and checkout writes to it.".into()),
        agent_name: "Migration agent".into(),
        model_name: Some("model A".into()),
        agent_mark: None,
        risk_level: Some(Risk::High),
        branches: vec![
            Branch {
                follow_up: Some(FollowUp {
                    question_text: "Notify the owner first?".into(),
                    branches: vec![option("Notify now"), option("Notify after cutover")],
                }),
                ..branch(
                    "Apply now",
                    vec![
                        flagged(
                            "Table locks for about 40 seconds",
                            "Checkout writes queue behind the lock and retry up to three times.",
                        ),
                        plain("Replica falls behind for the rest of the window"),
                    ],
                    false,
                )
            },
            Branch {
                follow_up: Some(FollowUp {
                    question_text: "Run the backfill in the same window?".into(),
                    branches: vec![option("Backfill tonight"), option("Backfill next week")],
                }),
                ..branch(
                    "Wait for replica",
                    vec![
                        plain("Migration finishes once the replica catches up"),
                        plain("No write-path impact"),
                    ],
                    true,
                )
            },
            branch(
                "Stop and report",
                vec![
                    plain("No schema change is made"),
                    plain("Owner gets a summary of the blocker"),
                ],
                false,
            ),
        ],
    };
    let pricing = Fork {
        fork_id: "f2".into(),
        question_text: "Publish the price change to all plans?".into(),
        context_text: Some("Affects 2,300 active subscriptions.".into()),
        agent_name: "Pricing agent".into(),
        model_name: Some("model B".into()),
        agent_mark: None,
        risk_level: Some(Risk::High),
        branches: vec![
            option("Publish now"),
            option("Stage for Monday"),
            option("Hold"),
        ],
    };
    rsx! {
        Item { slug: "branch-choice-map", name: "BranchChoiceMap",
            BranchChoiceMap {
                pending_forks: vec![migration.clone(), pricing],
                heading_text: "Decisions",
                allow_custom: true,
            }
            div { class: "gl-out", "data-out": "branch-choice-map", "{last()}" }
        }
        Item { slug: "branch-choice-map-follow-up", name: "BranchChoiceMap",
            BranchChoiceMap {
                pending_forks: vec![migration],
                heading_text: "Decisions",
                allow_custom: true,
                initial_pick: Some(0),
            }
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
            cause(
                "c1",
                "Parser rejects input",
                CheckState::Confirmed,
                vec![cause("c1a", "No EOF guard", CheckState::Confirmed, vec![])],
            ),
            cause("c2", "Cache is stale", CheckState::RuledOut, vec![]),
            cause("c3", "Dependency mismatch", CheckState::Checking, vec![]),
            cause("c4", "Fixture file corrupted", CheckState::Open, vec![]),
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
                    Attempt { attempt_cost: None, ..attempt("a4", "Model D", AttemptState::Queued, None) },
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
        FileRow {
            file_path: "src/parser.rs".into(),
            edits: vec![edit("ben", (80, 120), true), edit("cy", (100, 140), true)],
            conflict: Some(vec![
                ConflictOption {
                    contributor_id: "ben".into(),
                    option_text: "Keep Ben's version".into(),
                },
                ConflictOption {
                    contributor_id: "cy".into(),
                    option_text: "Keep Cy's version".into(),
                },
            ]),
        },
        FileRow {
            file_path: "README.md".into(),
            edits: vec![edit("cy", (12, 18), false)],
            conflict: None,
        },
        FileRow {
            file_path: "tests/api.rs".into(),
            edits: vec![edit("ben", (1, 30), false)],
            conflict: None,
        },
    ];
    rsx! {
        Item { slug: "parallel-edit-overlap-grid", name: "ParallelEditOverlapGrid",
            ParallelEditOverlapGrid {
                contributors: vec![contributor("ada", "Ada"), contributor("ben", "Ben"), contributor("cy", "Cy")],
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
    use rv2_kit::components::sheet_grid::{Column, ColumnKind, GridRow, SheetGrid, TagTone, Tone};
    use std::collections::BTreeMap;
    let mut last = use_signal(String::new);
    let mut rows = use_signal(|| {
        [
            (
                "ledger",
                "Ledger Pro",
                "Core",
                "ledgerpro.example",
                "2027-03-01",
            ),
            (
                "mailroom",
                "Mailroom",
                "Planned",
                "mailroom.example",
                "2027-01-15",
            ),
            (
                "pixel",
                "Pixel Tools",
                "Trial",
                "pixeltools.example",
                "2026-12-09",
            ),
            (
                "atlas",
                "Atlas Notes",
                "Core",
                "atlas.example",
                "2027-06-30",
            ),
            (
                "forms",
                "Old Forms",
                "Retired",
                "oldforms.example",
                "2026-11-30",
            ),
        ]
        .into_iter()
        .map(|(id, title, tag, site, renews)| GridRow {
            row_id: id.into(),
            row_logo: None,
            values: BTreeMap::from([
                ("title".to_string(), title.to_string()),
                ("tag".to_string(), tag.to_string()),
                ("link".to_string(), site.to_string()),
                ("renews".to_string(), renews.to_string()),
            ]),
        })
        .collect::<Vec<GridRow>>()
    });
    let columns = vec![
        Column {
            column_width: Some(180),
            ..Column::new("title", "Tool", ColumnKind::Title)
        },
        Column {
            column_width: Some(120),
            tag_choices: ["Core", "Planned", "Trial", "Retired"]
                .map(String::from)
                .to_vec(),
            ..Column::new("tag", "Status", ColumnKind::Tag)
        },
        Column::new("link", "Site", ColumnKind::Link),
        Column::new("renews", "Renews", ColumnKind::Text),
    ];
    rsx! {
        Item { slug: "sheet-grid", name: "SheetGrid",
            SheetGrid {
                grid_heading: "Tool renewals",
                grid_columns: columns,
                grid_rows: rows(),
                tag_tones: vec![
                    TagTone { tag: "Core".into(), tone: Tone::Positive },
                    TagTone { tag: "Planned".into(), tone: Tone::Info },
                    TagTone { tag: "Trial".into(), tone: Tone::Highlight },
                    TagTone { tag: "Retired".into(), tone: Tone::Muted },
                ],
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
        Worker {
            worker_id: "w3".into(),
            worker_name: "Model C".into(),
            worker_model_name: Some("model C".into()),
            worker_avatar: None,
            busy_minutes: 0.0,
            cost_per_minute: 0.03,
        },
        Worker {
            worker_id: "w4".into(),
            worker_name: "Model D".into(),
            worker_model_name: None,
            worker_avatar: None,
            busy_minutes: 0.0,
            cost_per_minute: 0.04,
        },
    ];
    let work = vec![
        WorkItem {
            work_id: "k1".into(),
            work_title: "Summarise calls".into(),
            estimate_minutes: 10.0,
            required_tools: vec![],
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
            required_tools: vec![],
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
            required_tools: vec![],
            assigned_worker_id: None,
        },
        WorkItem {
            work_id: "k6".into(),
            work_title: "Reconcile refunds".into(),
            estimate_minutes: 18.0,
            required_tools: vec!["CRM".into(), "Sheets".into()],
            assigned_worker_id: Some("w3".into()),
        },
        WorkItem {
            work_id: "k7".into(),
            work_title: "Triage support inbox".into(),
            estimate_minutes: 9.0,
            required_tools: vec![],
            assigned_worker_id: None,
        },
        WorkItem {
            work_id: "k8".into(),
            work_title: "Update pricing page".into(),
            estimate_minutes: 14.0,
            required_tools: vec![],
            assigned_worker_id: Some("w1".into()),
        },
        WorkItem {
            work_id: "k9".into(),
            work_title: "Export ledger".into(),
            estimate_minutes: 6.0,
            required_tools: vec![],
            assigned_worker_id: Some("w3".into()),
        },
        WorkItem {
            work_id: "k10".into(),
            work_title: "Draft follow-up".into(),
            estimate_minutes: 7.0,
            required_tools: vec![],
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
        Route {
            route_id: "batch".into(),
            route_name: "Batch model".into(),
            route_provider: Some("Provider C".into()),
            route_mark: None,
            route_tone: None,
            cost_per_1k: 0.2,
            p95_seconds: 9.5,
        },
        Route {
            route_id: "edge".into(),
            route_name: "Edge model".into(),
            route_provider: Some("Provider D".into()),
            route_mark: None,
            route_tone: None,
            cost_per_1k: 0.6,
            p95_seconds: 2.1,
        },
    ];
    let strategies = vec![
        Strategy {
            strategy_id: "even".into(),
            strategy_label: "Even".into(),
            share_map: vec![
                ("fast".into(), 25.0),
                ("deep".into(), 25.0),
                ("batch".into(), 25.0),
                ("edge".into(), 25.0),
            ],
        },
        Strategy {
            strategy_id: "cost".into(),
            strategy_label: "Lowest cost".into(),
            share_map: vec![
                ("fast".into(), 20.0),
                ("deep".into(), 0.0),
                ("batch".into(), 60.0),
                ("edge".into(), 20.0),
            ],
        },
        Strategy {
            strategy_id: "latency".into(),
            strategy_label: "Lowest latency".into(),
            share_map: vec![
                ("fast".into(), 70.0),
                ("deep".into(), 0.0),
                ("batch".into(), 0.0),
                ("edge".into(), 30.0),
            ],
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
    let draft = Policy {
        active_strategy: "cost".into(),
        fallback: live.fallback.clone(),
    };
    rsx! {
        Item { slug: "traffic-split-router", name: "TrafficSplitRouter",
            TrafficSplitRouter {
                route_list: routes,
                strategy_list: strategies,
                live_policy: live.clone(),
                initial_draft: Some(draft),
                policy_revision: 7,
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
    let change = |agent: &str, added: u32, removed: u32| Change {
        agent_id: agent.into(),
        lines_added: Some(added),
        lines_removed: Some(removed),
    };
    let file = |path: &str, lines: u32, change_log: Vec<Change>| RepoFile {
        file_path: path.into(),
        line_count: lines,
        change_log,
    };
    let files = vec![
        file("api/routes.ts", 120, vec![]),
        file("api/auth.ts", 90, vec![]),
        file("api/webhooks.ts", 60, vec![]),
        file("api/session.ts", 70, vec![]),
        file("api/rates.ts", 55, vec![]),
        file("api/quotes.ts", 95, vec![]),
        file(
            "payments/rounding.ts",
            180,
            vec![change("store", 8, 2), change("payments", 10, 4)],
        ),
        file("payments/invoice.ts", 240, vec![change("payments", 12, 4)]),
        file("payments/ledger.ts", 200, vec![]),
        file("payments/refunds.ts", 130, vec![]),
        file("payments/tax.ts", 110, vec![]),
        file("store/cursor.rs", 150, vec![change("store", 30, 2)]),
        file("store/retry.rs", 110, vec![change("store", 9, 1)]),
        file("store/pool.rs", 80, vec![]),
        file("store/schema.rs", 300, vec![]),
        file(
            "checks/rounding.test.ts",
            140,
            vec![change("payments", 22, 3)],
        ),
        file("checks/invoice.test.ts", 95, vec![]),
        file("checks/ledger.test.ts", 120, vec![]),
        file("checks/fixtures.json", 40, vec![]),
        file("checks/setup.ts", 30, vec![]),
        file(
            "notes/rounding-note.md",
            90,
            vec![Change {
                agent_id: "notes".into(),
                lines_added: Some(14),
                lines_removed: None,
            }],
        ),
        file("notes/billing.md", 58, vec![]),
        file("notes/api.md", 40, vec![]),
    ];
    let agent = |id: &str, name: &str, mark: &str, file: &str| RepoAgent {
        agent_id: id.into(),
        agent_name: name.into(),
        agent_mark: Some(mark.into()),
        current_file: Some(file.into()),
        read_file: None,
        agent_action: AgentAction::Edit,
        agent_task: None,
    };
    let mut agents = vec![
        agent("payments", "Payments agent", "P", "payments/rounding.ts"),
        agent("notes", "Notes agent", "N", "payments/rounding.ts"),
        agent("store", "Store agent", "S", "store/retry.rs"),
    ];
    agents[0].read_file = Some("api/quotes.ts".into());
    rsx! {
        Item { slug: "workspace-territory-map", name: "WorkspaceTerritoryMap",
            WorkspaceTerritoryMap {
                file_list: files,
                agent_list: agents,
                repo_name: "tally-service",
                branch_name: Some("fix/cent-rounding".to_string()),
                folder_depth: 2,
            }
            div { class: "gl-out", "data-out": "workspace-territory-map", "{last()}" }
        }
    }
}
