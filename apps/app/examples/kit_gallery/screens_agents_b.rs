use dioxus::html::FileData;
use dioxus::prelude::*;

use super::{Item, NOW_MS, gallery_now};

use rv2_kit::screens::autonomous_run_workspace::{
    ActivityEntry, Approval, AutonomousRunWorkspace, CallState as RunCallState, DraftRecord,
    FactRow, FileEntry, FileKind, Risk, RunRecord as AutoRun, RunState as AutoState, RunStep,
    StepPhase, ToolCall,
};
use rv2_kit::screens::multi_agent_run_canvas::{
    AgentNode, CanvasRun, CanvasRunState, MultiAgentRunCanvas, NodeState, TraceRow, TraceState,
};
use rv2_kit::screens::orchestrator_thread::{
    Attachment, Connection, Decision, DecisionOption, Delivery, OrchestratorThread, Part, RowState,
    ThreadMessage, ToolRow, UploadState,
};

const MINUTE: u64 = 60_000;

fn run_node(id: &str, name: &str, role: &str, parent: Option<&str>, state: NodeState) -> AgentNode {
    AgentNode {
        id: id.into(),
        entry_name: name.into(),
        node_role: role.into(),
        model_name: "Claude Sonnet".into(),
        node_state: state,
        parent_id: parent.map(|p| p.to_string()),
        cost_label: (state != NodeState::Queued).then(|| "$0.42".to_string()),
        duration_seconds: Some(64.0),
        token_count: Some(9_000),
    }
}

fn trace(
    id: &str,
    agent: &str,
    label: &str,
    start: f64,
    duration: Option<f64>,
    state: TraceState,
) -> TraceRow {
    TraceRow {
        id: id.into(),
        agent_id: agent.into(),
        entry_label: label.into(),
        start_seconds: start,
        duration_seconds: duration,
        row_state: state,
        error_text: None,
    }
}

#[component]
pub fn MultiAgentRunCanvasDemo() -> Element {
    let nodes = vec![
        run_node(
            "orch",
            "Orchestrator",
            "orchestrator",
            None,
            NodeState::Running,
        ),
        run_node(
            "res",
            "Research lead",
            "research",
            Some("orch"),
            NodeState::Done,
        ),
        run_node(
            "r1",
            "Filings reader",
            "research",
            Some("res"),
            NodeState::Done,
        ),
        run_node(
            "r2",
            "Results summary",
            "research",
            Some("res"),
            NodeState::Done,
        ),
        run_node(
            "r3",
            "Claims checker",
            "research",
            Some("res"),
            NodeState::Running,
        ),
        run_node(
            "r4",
            "Quote finder",
            "research",
            Some("res"),
            NodeState::Done,
        ),
        run_node(
            "ana",
            "Analysis lead",
            "analysis",
            Some("orch"),
            NodeState::Done,
        ),
        run_node("a1", "Cost model", "analysis", Some("ana"), NodeState::Done),
        run_node(
            "a2",
            "Budget check",
            "analysis",
            Some("ana"),
            NodeState::NeedsYou,
        ),
        run_node(
            "a3",
            "Option compare",
            "analysis",
            Some("ana"),
            NodeState::Done,
        ),
        run_node(
            "wri",
            "Writing lead",
            "writing",
            Some("orch"),
            NodeState::Done,
        ),
        run_node(
            "w1",
            "Brief drafter",
            "writing",
            Some("wri"),
            NodeState::Failed,
        ),
        run_node(
            "w2",
            "Style pass",
            "writing",
            Some("wri"),
            NodeState::Queued,
        ),
        run_node(
            "rev",
            "Review lead",
            "review",
            Some("orch"),
            NodeState::Done,
        ),
        run_node(
            "v1",
            "Draft reviewer",
            "review",
            Some("rev"),
            NodeState::Done,
        ),
        run_node(
            "v2",
            "Fact reviewer",
            "review",
            Some("rev"),
            NodeState::Done,
        ),
    ];
    let mut failed = trace(
        "t12",
        "w1",
        "Draft the brief",
        80.0,
        Some(12.0),
        TraceState::Failed,
    );
    failed.error_text = Some("Draft refused: broke the style guide".into());
    let trace_rows = vec![
        trace(
            "t1",
            "orch",
            "Plan the run",
            0.0,
            Some(12.0),
            TraceState::Done,
        ),
        trace(
            "t2",
            "res",
            "Search sources",
            12.0,
            Some(28.0),
            TraceState::Done,
        ),
        trace(
            "t3",
            "ana",
            "Plan the analysis",
            14.0,
            Some(10.0),
            TraceState::Done,
        ),
        trace(
            "t4",
            "a1",
            "Model the costs",
            24.0,
            Some(26.0),
            TraceState::Done,
        ),
        trace(
            "t5",
            "r1",
            "Read the filings",
            40.0,
            Some(18.0),
            TraceState::Done,
        ),
        trace(
            "t6",
            "r2",
            "Summarise results",
            40.0,
            Some(26.0),
            TraceState::Done,
        ),
        trace(
            "t7",
            "a2",
            "Ask about the budget",
            50.0,
            None,
            TraceState::Running,
        ),
        trace(
            "t8",
            "a3",
            "Compare the options",
            52.0,
            Some(18.0),
            TraceState::Done,
        ),
        trace(
            "t9",
            "r3",
            "Check the claims",
            58.0,
            None,
            TraceState::Running,
        ),
        trace(
            "t10",
            "r4",
            "Gather quotes",
            66.0,
            Some(18.0),
            TraceState::Done,
        ),
        trace(
            "t11",
            "wri",
            "Outline the brief",
            70.0,
            Some(10.0),
            TraceState::Done,
        ),
        failed,
        trace(
            "t13",
            "w1",
            "Draft the brief, retry",
            92.0,
            Some(18.0),
            TraceState::Retry,
        ),
        trace(
            "t14",
            "v1",
            "Review the draft",
            100.0,
            Some(20.0),
            TraceState::Done,
        ),
    ];
    let mut last = use_signal(|| "no callback yet".to_string());
    rsx! {
        Item { slug: "multi-agent-run-canvas", name: "MultiAgentRunCanvas",
            MultiAgentRunCanvas {
                run_record: Some(CanvasRun { id: "run-01".into(), entry_title: "Market brief".into(), run_state: CanvasRunState::Running, elapsed_seconds: 120.0, cost_label: "$4.82".into(), token_count: 184_000, agents_done: 11, agents_total: 16 }),
                agent_nodes: nodes,
                trace_rows,
                is_live: true,
                initial_open_agent_id: None,
                on_pause: move |paused: bool| last.set(format!("pause {paused}")),
                on_stop: move |_: ()| last.set("stop".to_string()),
                on_open_agent: move |id: String| last.set(format!("open {id}")),
                on_navigate: move |target: String| last.set(format!("navigate {target}")),
                on_resolve: move |(id, answer): (String, String)| last.set(format!("resolve {id} {answer}")),
            }
            div { class: "gl-out", "data-out": "multi-agent-run-canvas", "{last()}" }
        }
        Item { slug: "multi-agent-run-canvas-empty", name: "MultiAgentRunCanvas (empty)",
            MultiAgentRunCanvas { is_empty: true }
        }
        Item { slug: "multi-agent-run-canvas-loading", name: "MultiAgentRunCanvas (loading)",
            MultiAgentRunCanvas { is_loading: true }
        }
    }
}

fn run_step(id: &str, title: &str, phase: StepPhase, approval: bool) -> RunStep {
    RunStep {
        id: id.into(),
        entry_title: title.into(),
        detail_text: None,
        tool_name: "Docs".into(),
        step_phase: phase,
        tool_call_list: if phase == StepPhase::Done {
            vec![ToolCall {
                entry_name: "docs.search".into(),
                summary: "query: renewals".into(),
                args_data: "{\"q\":\"renewals\"}".into(),
                result_data: Some("3 results".into()),
                duration_ms: 480,
                call_state: RunCallState::Done,
            }]
        } else {
            Vec::new()
        },
        approval_request: approval.then(|| Approval {
            entry_title: "Send the renewal email".into(),
            entry_description: "Sends one email to a customer.".into(),
            risk_level: Risk::High,
            fact_rows: vec![FactRow {
                entry_label: "To".into(),
                entry_value: "ops@harbor.example".into(),
            }],
        }),
    }
}

fn workspace_steps() -> Vec<RunStep> {
    let mut collect = run_step("p1", "Collect renewals", StepPhase::Running, false);
    collect.tool_call_list = vec![
        ToolCall {
            entry_name: "crm.list".into(),
            summary: "status: due".into(),
            args_data: "{\"due\":\"march\"}".into(),
            result_data: Some("14 accounts".into()),
            duration_ms: 910,
            call_state: RunCallState::Done,
        },
        ToolCall {
            entry_name: "docs.search".into(),
            summary: "query: renewals".into(),
            args_data: "{\"q\":\"renewals\"}".into(),
            result_data: None,
            duration_ms: 0,
            call_state: RunCallState::Running,
        },
    ];
    vec![
        collect,
        run_step("p2", "Draft the reminder", StepPhase::Pending, false),
        run_step("p3", "Check the terms", StepPhase::Pending, false),
        run_step("p4", "Send the email", StepPhase::Pending, false),
        run_step("p5", "Log the send", StepPhase::Pending, false),
    ]
}

fn renewal_steps(waiting: bool) -> Vec<RunStep> {
    vec![
        run_step("p1", "Collect renewals", StepPhase::Done, false),
        run_step("p2", "Draft the reminder", StepPhase::Done, false),
        run_step(
            "p3",
            "Send the email",
            if waiting {
                StepPhase::Waiting
            } else {
                StepPhase::Pending
            },
            true,
        ),
        run_step("p4", "Log the send", StepPhase::Pending, false),
    ]
}

fn renewal_draft() -> DraftRecord {
    DraftRecord {
        original_text: "Hi Harbor, your renewal is due soon. Reply to confirm the plan. Tell us about seats. Thanks for staying. Team Ops.".into(),
        suggested_text: "Hi Harbor, your renewal is due on 30 March. Reply by Friday to confirm the plan. Tell us about new seats. Thanks for staying with us. Team Harbor Ops.".into(),
        entry_label: Some("Reminder email · 9 changes".into()),
    }
}

fn renewal_files() -> Vec<FileEntry> {
    vec![
        FileEntry {
            name: "renewals.csv".into(),
            kind: FileKind::Sheet,
            body: "name,due\nHarbor,30 Mar".into(),
        },
        FileEntry {
            name: "reminder.md".into(),
            kind: FileKind::Document,
            body: "Hi Harbor, your renewal is due on 30 March.".into(),
        },
        FileEntry {
            name: "terms.md".into(),
            kind: FileKind::Document,
            body: "Renewal terms: 12 months, same seats.".into(),
        },
    ]
}

#[component]
pub fn AutonomousRunWorkspaceDemo() -> Element {
    let mut last = use_signal(|| "no callback yet".to_string());
    let now = gallery_now();
    let run = AutoRun {
        id: "run-7".into(),
        entry_title: "Renewal reminders".into(),
        agent_name: "Ada".into(),
        started_at_ms: NOW_MS - 192_000,
        run_state: AutoState::Running,
        elapsed_seconds: 192,
        use_count: 3,
        cost_label: "$0.18".into(),
        activity_list: vec![ActivityEntry {
            at_ms: NOW_MS - 15_000,
            entry_text: "Step 1, collect renewals: 2 tool calls, 0.9 s".into(),
        }],
    };
    rsx! {
        Item { slug: "autonomous-run-workspace", name: "AutonomousRunWorkspace",
            div { class: "gl-canvas", style: "height: 720px; overflow: auto; padding: 0;",
                AutonomousRunWorkspace {
                    run_record: Some(run.clone()),
                    step_list: workspace_steps(),
                    draft_record: Some(renewal_draft()),
                    file_map: renewal_files(),
                    app_mark_list: vec!["M".to_string(), "D".to_string(), "C".to_string()],
                    clock_ms: Some(now),
                    on_approve: move |id: String| last.set(format!("approve {id}")),
                    on_deny: move |id: String| last.set(format!("deny {id}")),
                    on_accept_draft: move |_: ()| last.set("accept draft".to_string()),
                    on_reject_draft: move |_: ()| last.set("reject draft".to_string()),
                    on_stop: move |_: ()| last.set("stop".to_string()),
                    on_done: move |_: ()| last.set("done".to_string()),
                }
            }
            div { class: "gl-out", "data-out": "autonomous-run-workspace", "{last()}" }
        }
        Item { slug: "autonomous-run-workspace-simulated", name: "AutonomousRunWorkspace (simulated)",
            div {
                AutonomousRunWorkspace {
                    run_record: Some(AutoRun { run_state: AutoState::Running, ..run.clone() }),
                    step_list: renewal_steps(false),
                    draft_record: Some(renewal_draft()),
                    file_map: renewal_files(),
                    clock_ms: Some(now),
                    auto_play: true,
                    on_approve: move |id: String| last.set(format!("approve {id}")),
                    on_deny: move |id: String| last.set(format!("deny {id}")),
                    on_accept_draft: move |_: ()| last.set("accept draft".to_string()),
                    on_reject_draft: move |_: ()| last.set("reject draft".to_string()),
                    on_stop: move |_: ()| last.set("stop".to_string()),
                    on_done: move |_: ()| last.set("done".to_string()),
                }
            }
            div { class: "gl-out", "data-out": "autonomous-run-workspace-simulated", "{last()}" }
        }
        Item { slug: "autonomous-run-workspace-plan", name: "AutonomousRunWorkspace (plan)",
            div {
                AutonomousRunWorkspace {
                    run_record: Some(AutoRun { run_state: AutoState::Running, ..run.clone() }),
                    step_list: vec![
                        run_step("s0", "Fetch the list", StepPhase::Done, false),
                        run_step("s1", "Collect renewals", StepPhase::Pending, false),
                        run_step("s2", "Draft the reminder", StepPhase::Pending, false),
                    ],
                    clock_ms: Some(now),
                }
            }
        }
        Item { slug: "autonomous-run-workspace-empty", name: "AutonomousRunWorkspace (empty)",
            AutonomousRunWorkspace { is_empty: true, clock_ms: Some(now) }
        }
        Item { slug: "autonomous-run-workspace-loading", name: "AutonomousRunWorkspace (loading)",
            AutonomousRunWorkspace { is_loading: true, clock_ms: Some(now) }
        }
    }
}

const STREAM_TEXT: &str = "Pulling the renewal list now. I found fourteen accounts due this month, checked each contract end date, and drafted a reminder for the six that renew before 30 March.";

fn main_thread() -> Vec<ThreadMessage> {
    vec![
        ThreadMessage::User {
            id: "u1".into(),
            text: "Draft the Q3 summary and ask me before sending.".into(),
            delivery: Delivery::Sent,
        },
        ThreadMessage::Agent {
            id: "a1".into(),
            name: "Orchestrator".into(),
            time_label: "09:12".into(),
            parts: vec![
                Part::Text("I will start with the support tickets.".into()),
                Part::Tools(vec![
                    ToolRow {
                        id: "x1".into(),
                        entry_label: "Read tickets".into(),
                        row_state: RowState::Done,
                        duration_ms: Some(820),
                        error_text: None,
                    },
                    ToolRow {
                        id: "x2".into(),
                        entry_label: "Count topics".into(),
                        row_state: RowState::Done,
                        duration_ms: Some(430),
                        error_text: None,
                    },
                ]),
                Part::Task {
                    id: "task-1".into(),
                    entry_title: "Q3 summary".into(),
                    status: "running".into(),
                    done: 2,
                    total: 4,
                },
                Part::Error {
                    text: "The CRM did not answer.".into(),
                    retry_allowed: true,
                },
            ],
            is_streaming: false,
            is_stopped: false,
        },
        ThreadMessage::User {
            id: "u2".into(),
            text: "Use plain language.".into(),
            delivery: Delivery::Queued,
        },
        ThreadMessage::User {
            id: "u3".into(),
            text: "Send the Harbor reminder.".into(),
            delivery: Delivery::Failed,
        },
    ]
}

fn main_decisions() -> Vec<Decision> {
    vec![
        Decision {
            id: "d1".into(),
            question_text: "Which tone should the summary use?".into(),
            task_name: "Q3 summary".into(),
            option_list: vec![
                DecisionOption {
                    id: "o1".into(),
                    entry_title: "Plain".into(),
                    summary: Some("Short sentences".into()),
                    is_recommended: true,
                },
                DecisionOption {
                    id: "o2".into(),
                    entry_title: "Formal".into(),
                    summary: None,
                    is_recommended: false,
                },
            ],
        },
        Decision {
            id: "d2".into(),
            question_text: "Include churn figures?".into(),
            task_name: "Q3 summary".into(),
            option_list: vec![
                DecisionOption {
                    id: "yes".into(),
                    entry_title: "Yes".into(),
                    summary: None,
                    is_recommended: false,
                },
                DecisionOption {
                    id: "no".into(),
                    entry_title: "No".into(),
                    summary: None,
                    is_recommended: true,
                },
            ],
        },
    ]
}

fn option_title(decisions: &[Decision], decision_id: &str, option_id: &str) -> Option<String> {
    decisions
        .iter()
        .find(|d| d.id == decision_id)?
        .option_list
        .iter()
        .find(|o| o.id == option_id)
        .map(|o| o.entry_title.clone())
}

fn seed_attachments() -> Vec<Attachment> {
    vec![
        Attachment {
            id: "f1".into(),
            name: "brief.docx".into(),
            upload_state: UploadState::Done,
        },
        Attachment {
            id: "f2".into(),
            name: "notes.pdf".into(),
            upload_state: UploadState::Failed,
        },
    ]
}

/// Walks one attachment from 0 to done, as a host upload reports progress.
fn run_upload(mut attachments: Signal<Vec<Attachment>>, id: String) {
    spawn(async move {
        for step in 1..=5u8 {
            let tick = "await new Promise((resolve) => setTimeout(resolve, 400)); return true;";
            if document::eval(tick).await.is_err() {
                return;
            }
            let state = if step == 5 {
                UploadState::Done
            } else {
                UploadState::Uploading(step * 20)
            };
            attachments.with_mut(|list| {
                if let Some(item) = list.iter_mut().find(|a| a.id == id) {
                    item.upload_state = state;
                }
            });
        }
    });
}

fn tool(id: &str, label: &str, state: RowState, ms: Option<u64>) -> ToolRow {
    ToolRow {
        id: id.into(),
        entry_label: label.into(),
        row_state: state,
        duration_ms: ms,
        error_text: None,
    }
}

fn rest_thread() -> Vec<ThreadMessage> {
    vec![
        ThreadMessage::Agent {
            id: "r0".into(),
            name: "Orchestrator".into(),
            time_label: "08:57".into(),
            parts: vec![Part::Text(
                "Pulled the Harbor and Dune contracts. Both end dates are in the calendar, so I can check the rest against them.".into(),
            )],
            is_streaming: false,
            is_stopped: false,
        },
        ThreadMessage::User {
            id: "r1".into(),
            text: "Check which renewals are due this month.".into(),
            delivery: Delivery::Sent,
        },
        ThreadMessage::Agent {
            id: "r2".into(),
            name: "Orchestrator".into(),
            time_label: "09:02".into(),
            parts: vec![
                Part::Tools(vec![
                    tool("r2a", "Read renewal calendar", RowState::Done, Some(1_200)),
                    tool("r2b", "Match contacts", RowState::Done, Some(800)),
                ]),
                Part::Text(
                    "Two renewals close this month: Harbor Legal on 18 Nov and Dune Freight on 27 Nov. I have a draft reminder for each and will not send either until you approve it.".into(),
                ),
                Part::Artifacts(vec!["renewals.csv".into(), "Harbor draft".into()]),
            ],
            is_streaming: false,
            is_stopped: false,
        },
        ThreadMessage::User {
            id: "r3".into(),
            text: "Draft both reminders and wait for my approval before sending.".into(),
            delivery: Delivery::Sent,
        },
        ThreadMessage::Agent {
            id: "r4".into(),
            name: "Orchestrator".into(),
            time_label: "09:05".into(),
            parts: vec![Part::Tools(vec![
                tool("r4a", "Open Harbor contract", RowState::Running, Some(300)),
                tool("r4b", "Open Dune contract", RowState::Running, Some(300)),
                tool("r4c", "Draft reminder email", RowState::Running, Some(300)),
            ])],
            is_streaming: false,
            is_stopped: false,
        },
    ]
}

fn streaming_thread(streaming: bool) -> Vec<ThreadMessage> {
    vec![
        ThreadMessage::User {
            id: "q1".into(),
            text: "Check which renewals are due this month.".into(),
            delivery: Delivery::Sent,
        },
        ThreadMessage::Agent {
            id: "a2".into(),
            name: "Orchestrator".into(),
            time_label: "09:14".into(),
            parts: vec![Part::Text(STREAM_TEXT.into())],
            is_streaming: streaming,
            is_stopped: !streaming,
        },
    ]
}

#[component]
pub fn OrchestratorThreadDemo() -> Element {
    let now = gallery_now();
    let mut messages = use_signal(main_thread);
    let mut decisions = use_signal(main_decisions);
    let mut attachments = use_signal(seed_attachments);
    let mut next_file = use_signal(|| 3usize);
    let mut last = use_signal(|| "no callback yet".to_string());
    let mut stream = use_signal(|| streaming_thread(true));
    let mut stream_last = use_signal(|| "no callback yet".to_string());
    let mut away_last = use_signal(|| "no callback yet".to_string());
    let mut rest = use_signal(rest_thread);
    let mut rest_last = use_signal(|| "no callback yet".to_string());
    rsx! {
        Item { slug: "orchestrator-thread", name: "OrchestratorThread",
            div { style: "height: 720px;",
                OrchestratorThread {
                    message_list: rest(),
                    running_count: 9,
                    needs_count: 5,
                    clock_ms: Some(now),
                    approval_chip: Some("Needs approval".to_string()),
                    on_send: move |(text, _ids): (String, Vec<String>)| {
                        let count = rest.read().len();
                        rest.with_mut(|list| {
                            list.push(ThreadMessage::User {
                                id: format!("s{count}"),
                                text: text.clone(),
                                delivery: Delivery::Sent,
                            })
                        });
                        rest_last.set(format!("send {text}"));
                    },
                    on_stop: move |_: ()| rest_last.set("stop".to_string()),
                    on_open_inbox: move |_: ()| rest_last.set("open inbox".to_string()),
                    on_toggle_panel: move |_: ()| rest_last.set("toggle panel".to_string()),
                }
            }
            div { class: "gl-out", "data-out": "orchestrator-thread", "{rest_last()}" }
        }
        Item { slug: "orchestrator-thread-docked", name: "OrchestratorThread (docked)",
            div { style: "height: 720px;",
                OrchestratorThread {
                    message_list: messages(),
                    decision_queue: decisions(),
                    attachment_list: attachments(),
                    running_count: 2,
                    needs_count: 1,
                    new_below_count: 3,
                    clock_ms: Some(now),
                    connection_state: None,
                    on_send: move |(text, ids): (String, Vec<String>)| {
                        let count = messages.read().len();
                        messages.with_mut(|list| {
                            list.push(ThreadMessage::User {
                                id: format!("s{count}"),
                                text: text.clone(),
                                delivery: Delivery::Sent,
                            })
                        });
                        attachments.with_mut(|list| list.retain(|a| !ids.contains(&a.id)));
                        last.set(format!("send {text} · {} files", ids.len()));
                    },
                    on_stop: move |_: ()| last.set("stop".to_string()),
                    on_retry: move |id: String| {
                        messages.with_mut(|list| {
                            for message in list.iter_mut() {
                                if let ThreadMessage::User { id: message_id, delivery, .. } = message && *message_id == id {
                                    *delivery = Delivery::Sent;
                                }
                            }
                        });
                        last.set(format!("retry {id}"));
                    },
                    on_answer: move |(decision_id, option_id): (String, String)| {
                        let title = option_title(&decisions.peek(), &decision_id, &option_id)
                            .unwrap_or_else(|| option_id.clone());
                        decisions.with_mut(|list| list.retain(|d| d.id != decision_id));
                        messages.with_mut(|list| {
                            list.push(ThreadMessage::User {
                                id: format!("answer-{decision_id}"),
                                text: format!("Answered: {title}"),
                                delivery: Delivery::Sent,
                            })
                        });
                        last.set(format!("answer {decision_id} {option_id}"));
                    },
                    on_later: move |id: String| last.set(format!("later {id}")),
                    on_open_task: move |id: String| last.set(format!("open task {id}")),
                    on_open_inbox: move |_: ()| last.set("open inbox".to_string()),
                    on_mark_read: move |_: ()| last.set("mark read".to_string()),
                    on_navigate: move |target: String| last.set(format!("navigate {target}")),
                    on_attach: move |file: FileData| {
                        let number = next_file();
                        next_file.set(number + 1);
                        let id = format!("f{number}");
                        let name = file.name();
                        attachments.with_mut(|list| {
                            list.push(Attachment {
                                id: id.clone(),
                                name: name.clone(),
                                upload_state: UploadState::Uploading(0),
                            })
                        });
                        run_upload(attachments, id);
                        last.set(format!("attach {name}"));
                    },
                    on_retry_attachment: move |id: String| {
                        attachments.with_mut(|list| {
                            if let Some(item) = list.iter_mut().find(|a| a.id == id) {
                                item.upload_state = UploadState::Uploading(0);
                            }
                        });
                        run_upload(attachments, id.clone());
                        last.set(format!("retry upload {id}"));
                    },
                    on_remove_attachment: move |id: String| {
                        attachments.with_mut(|list| list.retain(|a| a.id != id));
                        last.set(format!("remove {id}"));
                    },
                    on_voice_clip: move |seconds: u64| last.set(format!("voice {seconds} s")),
                }
            }
            div { class: "gl-out", "data-out": "orchestrator-thread-docked", "{last()}" }
        }
        Item { slug: "orchestrator-thread-streaming", name: "OrchestratorThread (streaming)",
            div { style: "height: 520px;",
                OrchestratorThread {
                    message_list: stream(),
                    running_count: 1,
                    needs_count: 0,
                    clock_ms: Some(now),
                    on_stop: move |_: ()| {
                        stream.set(streaming_thread(false));
                        stream_last.set("stop".to_string());
                    },
                }
            }
            div { class: "gl-out", "data-out": "orchestrator-thread-streaming", "{stream_last()}" }
        }
        Item { slug: "orchestrator-thread-away", name: "OrchestratorThread (away)",
            div { style: "height: 720px;",
                OrchestratorThread {
                    message_list: main_thread(),
                    decision_queue: main_decisions(),
                    running_count: 2,
                    needs_count: 1,
                    clock_ms: Some(now),
                    last_seen_at: Some(NOW_MS - 45 * MINUTE),
                    away_after_minutes: 30,
                    on_open_inbox: move |_: ()| away_last.set("open inbox".to_string()),
                    on_mark_read: move |_: ()| away_last.set("mark read".to_string()),
                }
            }
            div { class: "gl-out", "data-out": "orchestrator-thread-away", "{away_last()}" }
        }
        Item { slug: "orchestrator-thread-reconnecting", name: "OrchestratorThread (reconnecting)",
            div { style: "height: 420px;",
                OrchestratorThread {
                    message_list: main_thread(),
                    connection_state: Some(Connection::Reconnecting),
                    retry_at_ms: Some(NOW_MS + 12_000),
                    clock_ms: Some(now),
                }
            }
        }
        Item { slug: "orchestrator-thread-empty", name: "OrchestratorThread (empty)",
            div { style: "height: 420px;",
                OrchestratorThread { is_empty: true, clock_ms: Some(now), on_navigate: move |target: String| last.set(format!("navigate {target}")) }
            }
        }
        Item { slug: "orchestrator-thread-loading", name: "OrchestratorThread (loading)",
            div { style: "height: 420px;",
                OrchestratorThread { is_loading: true, clock_ms: Some(now) }
            }
        }
    }
}
