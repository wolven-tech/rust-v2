use dioxus::prelude::*;

use super::{Item, NOW_MS, Section, gallery_now};

use super::screens_agents_b::{
    AutonomousRunWorkspaceDemo, MultiAgentRunCanvasDemo, OrchestratorThreadDemo,
};
use rv2_kit::screens::agent_action_log::{
    Actor, AgentActionLog, AppRecord, DetailRow, StepRecord as LogStep, UndoWindow,
};
use rv2_kit::screens::agent_memory_manager::{
    AgentMemoryManager, MemoryRecord, SourceInfo, SourceKind, UsageEntry,
};
use rv2_kit::screens::agent_roster::{
    AgentRoster, AppRef, NewAgent, Provider, RecentTask, RosterEntry, RunState,
};
use rv2_kit::screens::agent_run_timeline::{
    AgentRunTimeline, RunRecord, StepRecord as RunStepRec, StepType,
};
use rv2_kit::screens::agent_schedule_board::{
    AgentScheduleBoard, HistoryEntry, MarkState, Outcome, RunTime, ScheduleRecord,
};
use rv2_kit::screens::agent_skill_library::{
    AgentSkillLibrary, NewSkill, Requirement, SkillRecord, SkillSource, SkillUsage,
};
use rv2_kit::screens::agent_spend_overview::{
    AgentSpendOverview, AttentionItem, AttentionKind, Breakdown, EntryKind, TopTask, UsageStats,
};
use rv2_kit::screens::agent_task_board::{
    AgentRecord, AgentTaskBoard, NewTask, StepRecord, TaskRecord, TaskState,
};
use rv2_kit::screens::agent_task_list::{
    AgentTaskList, Ask, AskKind, DecisionEntry, NewTaskLine, Progress, TaskEvent,
    TaskRecord as ListTask, TaskStep,
};
use rv2_kit::screens::agent_tool_explorer::{
    Access, AgentInfo, AgentToolExplorer, CallRecord, CallState, ChartLayout, Param, TimeRange,
    ToolInfo,
};

#[component]
pub fn ScreensAgents() -> Element {
    rsx! {
        Section { id: "screens_agents", title: "Screens: agents",
            AgentActionLogDemo {}
            AgentMemoryManagerDemo {}
            AgentRosterDemo {}
            AgentRunTimelineDemo {}
            AgentScheduleBoardDemo {}
            AgentSkillLibraryDemo {}
            AgentSpendOverviewDemo {}
            AgentTaskBoardDemo {}
            AgentTaskListDemo {}
            AgentToolExplorerDemo {}
            MultiAgentRunCanvasDemo {}
            AutonomousRunWorkspaceDemo {}
            OrchestratorThreadDemo {}
        }
    }
}

const DAY: u64 = 86_400_000;

#[component]
fn AgentActionLogDemo() -> Element {
    // Rest state: 16:42 today, so the Today rows (16:30 and earlier) are in the past.
    let now = gallery_now() + (16 * 60 + 42) * 60_000;
    let today = now - now % DAY;
    let at = |day_offset: u64, hour: u64, minute: u64| {
        today - day_offset * DAY + (hour * 60 + minute) * 60_000
    };
    let step = |id: &str,
                occurred_at: u64,
                app_id: &str,
                sentence: &str,
                task: &str,
                actor: Actor,
                approved_at: Option<u64>|
     -> LogStep {
        LogStep {
            id: id.into(),
            occurred_at,
            app_id: app_id.into(),
            sentence: sentence.into(),
            task_name: Some(task.into()),
            actor,
            rule_label: None,
            approved_at,
            detail_rows: vec![],
            preview_lines: vec![],
            undo_window: None,
            final_reason: None,
            is_undone: false,
            undone_at: None,
            cost_label: None,
        }
    };
    let mut steps = use_signal(|| {
        let mut mail = step(
            "s1",
            at(0, 16, 30),
            "mail",
            "Sent the status email to Dana Ortiz",
            "Weekly update",
            Actor::Approved,
            Some(now - 14 * 60_000),
        );
        mail.rule_label = Some("Client email".into());
        mail.detail_rows = vec![
            DetailRow {
                entry_label: "To".into(),
                entry_value: "Dana Ortiz <dana@acme.example>".into(),
            },
            DetailRow {
                entry_label: "Subject".into(),
                entry_value: "Week 46 status".into(),
            },
        ];
        mail.preview_lines = (1..=7).map(|n| format!("Line {n}: on schedule.")).collect();
        mail.undo_window = Some(UndoWindow {
            available_until: now + 18 * 60_000,
            entry_description: "Recall the email if it is still unread".into(),
        });
        mail.cost_label = Some("0.04 EUR".into());
        let mut refund = step(
            "s2",
            at(0, 16, 4),
            "billing",
            "Issued a refund of 38 to Mara Lind",
            "Billing",
            Actor::Automatic,
            None,
        );
        refund.final_reason = Some("Refunds cannot be reversed once issued.".into());
        let mut staging = step(
            "s3",
            at(0, 15, 38),
            "chat",
            "Posted the staging link in the team channel",
            "Release notes",
            Actor::Automatic,
            None,
        );
        staging.undo_window = Some(UndoWindow {
            available_until: now + 40 * 60_000,
            entry_description: "Delete the message from the channel".into(),
        });
        let mut pull = step(
            "s4",
            at(0, 15, 7),
            "code",
            "Opened pull request 482 to retry failed webhooks",
            "Webhook retries",
            Actor::Automatic,
            None,
        );
        pull.undo_window = Some(UndoWindow {
            available_until: now + 45 * 60_000,
            entry_description: "Close the pull request".into(),
        });
        let mut call = step(
            "s5",
            at(0, 13, 42),
            "calendar",
            "Booked a 30-minute call with Ben Ortega",
            "Onboarding",
            Actor::Approved,
            Some(at(0, 13, 40)),
        );
        call.undo_window = Some(UndoWindow {
            available_until: now + 90 * 60_000,
            entry_description: "Cancel the invitation".into(),
        });
        let mut folder = step(
            "s6",
            at(0, 12, 42),
            "docs",
            "Created the folder for Acme Q4 in Clients",
            "Ingest pipeline",
            Actor::Automatic,
            None,
        );
        folder.undo_window = Some(UndoWindow {
            available_until: now + 120 * 60_000,
            entry_description: "Move the folder to the trash".into(),
        });
        let mut merge = step(
            "s7",
            at(1, 14, 42),
            "code",
            "Merged pull request 479 into main",
            "API migration",
            Actor::Approved,
            Some(at(1, 14, 40)),
        );
        merge.final_reason = Some("Merged commits are kept in the history.".into());
        let mut reminder = step(
            "s8",
            at(1, 11, 42),
            "mail",
            "Sent a payment reminder to Lumen Studio",
            "Billing",
            Actor::Approved,
            Some(at(1, 11, 40)),
        );
        reminder.is_undone = true;
        reminder.undone_at = Some(at(1, 11, 50));
        let mut keys = step(
            "s9",
            at(1, 9, 42),
            "chat",
            "Asked Ben Ortega for the staging keys",
            "Billing",
            Actor::Approved,
            Some(at(1, 9, 40)),
        );
        keys.final_reason = Some("Sent messages cannot be recalled.".into());
        vec![
            mail, refund, staging, pull, call, folder, merge, reminder, keys,
        ]
    });
    let apps = vec![
        AppRecord {
            id: "mail".into(),
            entry_name: "Mail".into(),
            account: Some("dana@acme.example".into()),
            mark: None,
        },
        AppRecord {
            id: "billing".into(),
            entry_name: "Billing".into(),
            account: Some("acme finance".into()),
            mark: None,
        },
        AppRecord {
            id: "chat".into(),
            entry_name: "Chat".into(),
            account: None,
            mark: None,
        },
        AppRecord {
            id: "code".into(),
            entry_name: "Code".into(),
            account: None,
            mark: None,
        },
        AppRecord {
            id: "calendar".into(),
            entry_name: "Calendar".into(),
            account: None,
            mark: None,
        },
        AppRecord {
            id: "docs".into(),
            entry_name: "Docs".into(),
            account: None,
            mark: None,
        },
    ];
    let mut last = use_signal(|| "no callback yet".to_string());
    let empty_apps = apps.clone();
    rsx! {
        Item { slug: "agent-action-log", name: "AgentActionLog",
            div { style: "height: 800px; overflow: auto;",
                AgentActionLog {
                    app_list: apps,
                    step_list: steps(),
                    selected_step_id: Some("s1".to_string()),
                    clock_ms: Some(now),
                    on_undo: move |step: LogStep| {
                        last.set(format!("undo {}", step.id));
                        steps.with_mut(|list| {
                            if let Some(s) = list.iter_mut().find(|s| s.id == step.id) {
                                s.is_undone = true;
                            }
                        });
                    },
                    on_select: move |id: String| last.set(format!("select {id}")),
                    on_open_task: move |name: String| last.set(format!("open task {name}")),
                    on_navigate: move |target: String| last.set(format!("navigate {target}")),
                }
            }
            div { class: "gl-out", "data-out": "agent-action-log", "{last()}" }
        }
        Item { slug: "agent-action-log-empty", name: "AgentActionLog (empty)",
            div { style: "height: 640px; overflow: auto;",
                AgentActionLog {
                    app_list: empty_apps,
                    is_empty: true,
                    clock_ms: Some(now),
                    on_navigate: move |target: String| last.set(format!("navigate {target}")),
                }
            }
        }
        Item { slug: "agent-action-log-loading", name: "AgentActionLog (loading)",
            div { style: "height: 640px; overflow: auto;",
                AgentActionLog { is_loading: true, clock_ms: Some(now) }
            }
        }
    }
}

fn demo_memory(
    id: &str,
    text: &str,
    project: Option<&str>,
    pending: bool,
    now: u64,
) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        text: text.into(),
        project_id: project.map(str::to_string),
        source_info: SourceInfo {
            source_kind: SourceKind::Conversation,
            occurred_at: now - DAY,
            quote_text: None,
            question_text: None,
            file_name: None,
        },
        use_count: 3,
        last_used_at: Some(now - 2 * DAY),
        usage_log: vec![],
        is_pending_review: pending,
    }
}

#[component]
fn AgentMemoryManagerDemo() -> Element {
    let now = gallery_now();
    let mut memories = vec![
        MemoryRecord {
            id: "m1".into(),
            text: "Prefers short weekly summaries".into(),
            project_id: None,
            source_info: SourceInfo {
                source_kind: SourceKind::Conversation,
                occurred_at: now - DAY,
                quote_text: Some("Keep it short, weekly.".into()),
                question_text: None,
                file_name: None,
            },
            use_count: 5,
            last_used_at: Some(now - 3_600_000),
            usage_log: vec![UsageEntry {
                task_name: "Weekly digest".into(),
                how: "summary length".into(),
                occurred_at: now - 3_600_000,
            }],
            is_pending_review: false,
        },
        MemoryRecord {
            id: "m2".into(),
            text: "Launch date is 14 March".into(),
            project_id: Some("Launch".into()),
            source_info: SourceInfo {
                source_kind: SourceKind::File,
                occurred_at: now - 2 * DAY,
                quote_text: None,
                question_text: None,
                file_name: Some("plan.md".into()),
            },
            use_count: 2,
            last_used_at: Some(now - 2 * DAY),
            usage_log: vec![],
            is_pending_review: false,
        },
        MemoryRecord {
            id: "m3".into(),
            text: "Hiring freeze until Q2".into(),
            project_id: Some("Hiring".into()),
            source_info: SourceInfo {
                source_kind: SourceKind::Decision,
                occurred_at: now - DAY,
                quote_text: None,
                question_text: Some("Pause hiring?".into()),
                file_name: None,
            },
            use_count: 0,
            last_used_at: None,
            usage_log: vec![],
            is_pending_review: true,
        },
    ];
    memories.extend([
        demo_memory("m4", "Works in Lisbon time", None, false, now),
        demo_memory("m5", "Signs off as Dee", None, false, now),
        demo_memory("m6", "Dislikes passive voice", None, false, now),
        demo_memory("m7", "Reviews drafts on Mondays", None, false, now),
        demo_memory(
            "m8",
            "Pricing freezes before launch",
            Some("Launch"),
            false,
            now,
        ),
        demo_memory("m9", "Beta list has 40 teams", Some("Launch"), false, now),
        demo_memory(
            "m10",
            "Two open engineering roles",
            Some("Hiring"),
            false,
            now,
        ),
        demo_memory(
            "m11",
            "Competitor X raised prices in May",
            Some("Research"),
            false,
            now,
        ),
        demo_memory(
            "m12",
            "Use the 2025 survey results",
            Some("Research"),
            true,
            now,
        ),
        demo_memory(
            "m13",
            "Panel interviews run 45 minutes",
            Some("Hiring"),
            false,
            now,
        ),
    ]);
    let mut memory_list = use_signal(move || memories.clone());
    let mut last = use_signal(|| "no callback yet".to_string());
    rsx! {
        Item { slug: "agent-memory-manager", name: "AgentMemoryManager",
            button {
                r#type: "button",
                class: "gl-btn",
                "data-out": "agent-memory-manager-recall",
                onclick: move |_| {
                    memory_list.with_mut(|list| {
                        if let Some(m) = list.iter_mut().find(|m| m.id == "m1") {
                            m.use_count += 1;
                            m.last_used_at = Some(now);
                        }
                    });
                },
                "Simulate a recall of m1"
            }
            div { style: "height: 720px; overflow: auto;",
                AgentMemoryManager {
                    memory_list: memory_list(),
                    project_names: vec!["Launch".to_string(), "Hiring".to_string(), "Research".to_string()],
                    initial_memory_id: Some("m1".to_string()),
                    clock_ms: Some(now),
                    on_select: move |id: String| last.set(format!("select {id}")),
                    on_create: move |created: MemoryRecord| {
                        last.set(format!("create {}", created.text));
                        memory_list.with_mut(|list| list.insert(0, created));
                    },
                    on_change: move |changed: MemoryRecord| last.set(format!("change {} {:?}", changed.id, changed.project_id)),
                    on_forget: move |id: String| last.set(format!("forget {id}")),
                    on_restore: move |(memory, index): (MemoryRecord, usize)| last.set(format!("restore {} at {index}", memory.id)),
                    on_keep: move |id: String| last.set(format!("keep {id}")),
                    on_open_source: move |memory: MemoryRecord| last.set(format!("open source {}", memory.id)),
                }
            }
            div { class: "gl-out", "data-out": "agent-memory-manager", "{last()}" }
        }
        Item { slug: "agent-memory-manager-empty", name: "AgentMemoryManager (empty)",
            div { style: "height: 640px; overflow: auto;",
                AgentMemoryManager { is_empty: true, clock_ms: Some(now) }
            }
        }
        Item { slug: "agent-memory-manager-loading", name: "AgentMemoryManager (loading)",
            div { style: "height: 640px; overflow: auto;",
                AgentMemoryManager { is_loading: true, clock_ms: Some(now) }
            }
        }
    }
}

fn roster_entry(id: &str, name: &str, state: RunState) -> RosterEntry {
    RosterEntry {
        id: id.into(),
        display_name: name.into(),
        role_label: "Research".into(),
        identity_hue_token: Some("ok".into()),
        model_name: "Claude Sonnet".into(),
        model_provider: Provider::Anthropic,
        run_state: state,
        current_activity: Some("Reading the Q3 notes".into()),
        last_run_label: Some("12 min ago".into()),
        runs_this_week: 14,
        cost_this_week: 8.4,
        daily_run_counts: vec![1, 2, 0, 3, 4, 2, 2],
        owner_name: Some("Dana".into()),
        instruction_text: Some("Summarise new notes.\nKeep claims sourced.\nFlag anything unclear.\nAsk before publishing.".into()),
        skill_names: vec!["Summarise".into(), "Cite sources".into()],
        app_refs: vec![AppRef { id: "docs".into(), entry_name: "Docs".into() }],
        recent_tasks: vec![RecentTask { entry_title: "Q3 notes digest".into(), task_state: "done".into(), occurred_at: 0 }],
    }
}

#[component]
fn AgentRosterDemo() -> Element {
    let mut last = use_signal(|| "no callback yet".to_string());
    let mut entries = vec![
        roster_entry("ada", "Ada", RunState::Working),
        roster_entry("lin", "Lin", RunState::Idle),
        roster_entry("sam", "Sam", RunState::Paused),
        roster_entry("iris", "Iris", RunState::Working),
        roster_entry("omar", "Omar", RunState::Idle),
        roster_entry("pia", "Pia", RunState::Idle),
        roster_entry("ren", "Ren", RunState::Paused),
        roster_entry("tomas", "Tomas", RunState::Working),
    ];
    let hues = [
        "ok", "teal", "plum", "slate", "warn", "fault", "accent", "harbor-7",
    ];
    for (entry, hue) in entries.iter_mut().zip(hues) {
        entry.identity_hue_token = Some(hue.to_string());
    }
    let empty: Vec<RosterEntry> = Vec::new();
    rsx! {
        Item { slug: "agent-roster", name: "AgentRoster",
            div { style: "height: 720px; overflow: auto;",
                AgentRoster {
                    roster_entries: entries,
                    initial_open_id: Some("ada".to_string()),
                    on_pause: move |(entry, paused): (RosterEntry, bool)| {
                        last.set(format!("pause {} {}", entry.id, paused));
                    },
                    on_message: move |entry: RosterEntry| last.set(format!("message {}", entry.id)),
                    on_create: move |agent: NewAgent| last.set(format!("create {}", agent.name)),
                    on_navigate: move |target: String| last.set(format!("navigate {target}")),
                }
            }
            div { class: "gl-out", "data-out": "agent-roster", "{last()}" }
        }
        Item { slug: "agent-roster-empty", name: "AgentRoster (empty)",
            div { style: "height: 640px; overflow: auto;",
                AgentRoster { roster_entries: empty, is_empty: true }
            }
        }
        Item { slug: "agent-roster-loading", name: "AgentRoster (loading)",
            div { style: "height: 640px; overflow: auto;",
                AgentRoster { roster_entries: Vec::new(), is_loading: true }
            }
        }
    }
}

fn timeline_step(
    id: &str,
    label: &str,
    step_type: StepType,
    start: f64,
    duration: Option<f64>,
    app: Option<&str>,
    sub_agent: Option<&str>,
) -> RunStepRec {
    RunStepRec {
        id: id.into(),
        entry_label: label.into(),
        step_type,
        start_seconds: start,
        duration_seconds: duration,
        nesting_depth: 0,
        app_name: app.map(str::to_string),
        sub_agent_name: sub_agent.map(str::to_string),
        has_failed: false,
        error_text: None,
        token_counts: (step_type == StepType::Model).then_some((420, 60)),
        cost_label: None,
        input_text: Some(format!("{label}: input")),
        output_text: Some(format!("{label}: output")),
    }
}

fn timeline_steps() -> Vec<RunStepRec> {
    use StepType::{Model, SubAgent, Tool};
    let nested = |mut step: RunStepRec| {
        step.nesting_depth = 1;
        step
    };
    let mut failed = timeline_step("s10", "Label tickets", Model, 170.0, Some(10.0), None, None);
    failed.has_failed = true;
    failed.error_text = Some("Rate limit reached, retried later.".into());
    failed.output_text = None;
    let mut write = timeline_step(
        "s15",
        "Write the status note",
        Tool,
        258.0,
        Some(22.0),
        Some("Docs"),
        None,
    );
    write.input_text = Some("title: Status, 14 tickets closed".into());
    write.output_text = Some("Saved status-note.md (1.2 kB)".into());
    write.token_counts = Some((520, 180));
    write.cost_label = Some("$0.06".into());
    vec![
        timeline_step("s1", "Plan the review", Model, 0.0, Some(8.0), None, None),
        timeline_step(
            "s2",
            "Read the queue",
            Tool,
            8.0,
            Some(20.0),
            Some("Helpdesk"),
            None,
        ),
        timeline_step(
            "s3",
            "Pull the metrics",
            Tool,
            28.0,
            Some(18.0),
            Some("Analytics"),
            None,
        ),
        timeline_step(
            "s4",
            "Classify tickets",
            SubAgent,
            46.0,
            Some(74.0),
            None,
            Some("Classifier"),
        ),
        nested(timeline_step(
            "s5",
            "Label batch one",
            Model,
            48.0,
            Some(22.0),
            None,
            Some("Classifier"),
        )),
        nested(timeline_step(
            "s6",
            "Label batch two",
            Model,
            70.0,
            Some(26.0),
            None,
            Some("Classifier"),
        )),
        timeline_step(
            "s7",
            "Tag the topics",
            Tool,
            120.0,
            Some(16.0),
            Some("Helpdesk"),
            None,
        ),
        timeline_step(
            "s8",
            "Score the risks",
            Model,
            136.0,
            Some(22.0),
            None,
            None,
        ),
        timeline_step(
            "s9",
            "Fetch the account",
            Tool,
            158.0,
            Some(12.0),
            Some("CRM"),
            None,
        ),
        failed,
        timeline_step(
            "s11",
            "Label tickets, retry",
            Model,
            180.0,
            Some(20.0),
            None,
            None,
        ),
        timeline_step(
            "s12",
            "Draft the reply",
            Tool,
            200.0,
            Some(12.0),
            Some("Mail"),
            None,
        ),
        timeline_step(
            "s13",
            "Check the policy",
            Tool,
            212.0,
            Some(14.0),
            Some("Docs"),
            None,
        ),
        timeline_step(
            "s14",
            "Review the draft",
            Model,
            226.0,
            Some(32.0),
            None,
            None,
        ),
        write,
        timeline_step("s16", "Write the summary", Model, 280.0, None, None, None),
    ]
}

#[component]
fn AgentRunTimelineDemo() -> Element {
    let elapsed = 296.0;
    let run = RunRecord {
        id: "r1".into(),
        entry_title: "Summarise the support queue".into(),
        agent_name: "Ada".into(),
        model_name: "Claude Sonnet".into(),
        started_at_ms: 1_795_219_200_000,
        total_seconds: None,
        cost_label: Some("$4.82".into()),
        step_records: timeline_steps(),
    };
    let mut finished_steps = run.step_records.clone();
    if let Some(last) = finished_steps.last_mut() {
        last.duration_seconds = Some(16.0);
    }
    let finished = RunRecord {
        total_seconds: Some(296.0),
        step_records: finished_steps,
        ..run.clone()
    };
    let mut last = use_signal(|| "no callback yet".to_string());
    rsx! {
        Item { slug: "agent-run-timeline", name: "AgentRunTimeline",
            AgentRunTimeline {
                run_record: Some(run),
                elapsed_seconds: Some(elapsed),
                initial_step_id: Some("s15".to_string()),
                on_stop: move |_: ()| last.set("stop".to_string()),
                on_rerun: move |_: ()| last.set("rerun".to_string()),
                on_navigate: move |target: String| last.set(format!("navigate {target}")),
            }
            div { class: "gl-out", "data-out": "agent-run-timeline", "{last()}" }
        }
        Item { slug: "agent-run-timeline-finished", name: "AgentRunTimeline (finished)",
            AgentRunTimeline {
                run_record: Some(finished),
                initial_step_id: Some("s2".to_string()),
                on_rerun: move |_: ()| last.set("rerun".to_string()),
            }
        }
        Item { slug: "agent-run-timeline-empty", name: "AgentRunTimeline (empty)",
            AgentRunTimeline { is_empty: true }
        }
        Item { slug: "agent-run-timeline-loading", name: "AgentRunTimeline (loading)",
            AgentRunTimeline { is_loading: true }
        }
    }
}

/// `past` and `upcoming` are minutes from now; each past run carries its own state.
fn sched(
    id: &str,
    name: &str,
    now: u64,
    past: &[(i64, MarkState)],
    upcoming: &[i64],
    paused: bool,
    last: Outcome,
) -> ScheduleRecord {
    let at = |minutes: i64| (now as i64 + minutes * 60_000) as u64;
    let mut run_times: Vec<RunTime> = past
        .iter()
        .map(|(minutes, state)| RunTime {
            at_ms: at(*minutes),
            run_state: *state,
            progress_percent: None,
        })
        .collect();
    run_times.extend(upcoming.iter().map(|minutes| RunTime {
        at_ms: at(*minutes),
        run_state: MarkState::Upcoming,
        progress_percent: None,
    }));
    ScheduleRecord {
        id: id.into(),
        entry_name: name.into(),
        project_name: Some("Support".into()),
        brief_summary: Some("Summarise the ticket queue".into()),
        cadence_text: "Every 2 hours".into(),
        run_times,
        duration_seconds: Some(900),
        is_paused: paused,
        history_log: vec![HistoryEntry {
            at_ms: at(-60),
            outcome: last,
            summary: "Summary sent".into(),
            cost_label: Some("$0.31".into()),
        }],
        cost_per_run_label: Some("$0.31".into()),
    }
}

#[component]
fn AgentScheduleBoardDemo() -> Element {
    // A live clock would re-key every Skip button each tick: run times are
    // absolute, so they must not be rebuilt from the moving gallery clock.
    let now = NOW_MS;
    let mut digest = sched(
        "digest",
        "Daily digest",
        now,
        &[(-120, MarkState::Failed)],
        &[420],
        false,
        Outcome::Failed,
    );
    digest.cadence_text = "Daily at 09:00".into();
    let mut churn = sched(
        "churn",
        "Churn watch",
        now,
        &[(-60, MarkState::Done)],
        &[360],
        false,
        Outcome::Done,
    );
    churn.cadence_text = "Every 6 hours".into();
    let mut billing = sched(
        "billing",
        "Billing check",
        now,
        &[],
        &[480, 600],
        false,
        Outcome::Done,
    );
    billing.cadence_text = "Every hour".into();
    let mut qa = sched("qa", "QA sweep", now, &[], &[540], false, Outcome::Done);
    qa.cadence_text = "Weekdays at 08:30".into();
    let mut review = sched(
        "review",
        "Weekly review",
        now,
        &[],
        &[60, 180],
        true,
        Outcome::Done,
    );
    review.cadence_text = "Mondays at 10:00".into();
    let mut triage = sched(
        "tri",
        "Triage",
        now,
        &[(-180, MarkState::Done)],
        &[150],
        false,
        Outcome::Done,
    );
    triage.cadence_text = "Every 3 hours".into();
    let schedules = vec![
        sched(
            "sum",
            "Ticket summary",
            now,
            &[(-240, MarkState::Done)],
            &[45, 105, 165, 225, 285],
            false,
            Outcome::Done,
        ),
        triage,
        digest,
        churn,
        billing,
        qa,
        review,
    ];
    let mut last = use_signal(|| "no callback yet".to_string());
    rsx! {
        Item { slug: "agent-schedule-board", name: "AgentScheduleBoard",
            div { style: "height: 640px; overflow: auto;",
                AgentScheduleBoard {
                    schedule_list: schedules,
                    now_ms: now,
                    selected_schedule_id: Some("sum".to_string()),
                    on_pause: move |(id, paused): (String, bool)| last.set(format!("pause {id} {paused}")),
                    on_run_now: move |id: String| last.set(format!("run now {id}")),
                    on_skip: move |(id, at, skipped): (String, u64, bool)| last.set(format!("skip {id} {at} {skipped}")),
                    on_select: move |id: String| last.set(format!("select {id}")),
                    on_navigate: move |target: String| last.set(format!("navigate {target}")),
                }
            }
            div { class: "gl-out", "data-out": "agent-schedule-board", "{last()}" }
        }
        Item { slug: "agent-schedule-board-empty", name: "AgentScheduleBoard (empty)",
            div { style: "height: 640px; overflow: auto;",
                AgentScheduleBoard { is_empty: true, now_ms: now }
            }
        }
        Item { slug: "agent-schedule-board-loading", name: "AgentScheduleBoard (loading)",
            div { style: "height: 640px; overflow: auto;",
                AgentScheduleBoard { is_loading: true, now_ms: now }
            }
        }
    }
}

fn skill(id: &str, name: &str, installed: bool, source: SkillSource) -> SkillRecord {
    SkillRecord {
        id: id.into(),
        entry_name: name.into(),
        entry_description: format!("{name} for weekly work"),
        category_name: if id == "d" { "Sales" } else { "Ops" }.into(),
        source_kind: source,
        author_name: None,
        version_label: Some("1.2".into()),
        is_installed: installed,
        is_enabled: installed,
        weekly_use_count: installed.then_some(9),
        install_count: (!installed).then_some(1200),
        trigger_phrases: vec!["weekly report".into()],
        step_texts: vec!["Collect figures".into(), "Write the summary".into()],
        requirement_list: vec![Requirement {
            entry_label: "Read the CRM".into(),
            permissions_link: true,
        }],
        usage_log: vec![SkillUsage {
            task_name: "Monday report".into(),
            occurred_at: 0,
        }],
        daily_run_counts: vec![1, 0, 2, 3, 1, 0, 2, 2, 4, 1, 0, 3, 2, 1],
    }
}

fn demo_skill(
    id: &'static str,
    name: &'static str,
    category: &'static str,
    installed: bool,
    source: SkillSource,
    author: Option<&'static str>,
) -> SkillRecord {
    let mut record = skill(id, name, installed, source);
    record.category_name = category.into();
    record.author_name = author.map(str::to_string);
    record
}

#[component]
fn AgentSkillLibraryDemo() -> Element {
    let mut skills = vec![
        demo_skill(
            "w",
            "Weekly report",
            "Ops",
            true,
            SkillSource::BuiltIn,
            None,
        ),
        demo_skill(
            "c",
            "Customer digest",
            "Support",
            true,
            SkillSource::Own,
            None,
        ),
        demo_skill(
            "d",
            "Invoice check",
            "Finance",
            true,
            SkillSource::Community,
            Some("@lin"),
        ),
        demo_skill(
            "e",
            "Lead scoring",
            "Sales",
            true,
            SkillSource::BuiltIn,
            None,
        ),
        demo_skill(
            "f",
            "Refund triage",
            "Support",
            true,
            SkillSource::Own,
            None,
        ),
        demo_skill(
            "g",
            "Board pack",
            "Finance",
            true,
            SkillSource::BuiltIn,
            None,
        ),
        demo_skill(
            "h",
            "Competitor brief",
            "Research",
            true,
            SkillSource::Community,
            Some("@research_crew"),
        ),
        demo_skill(
            "i",
            "Expense sort",
            "Finance",
            true,
            SkillSource::BuiltIn,
            None,
        ),
        demo_skill(
            "j",
            "Meeting notes",
            "Ops",
            true,
            SkillSource::BuiltIn,
            None,
        ),
        demo_skill(
            "k",
            "Churn alert",
            "Sales",
            false,
            SkillSource::Community,
            Some("@sales_ops"),
        ),
        demo_skill(
            "l",
            "Ticket macros",
            "Support",
            false,
            SkillSource::BuiltIn,
            None,
        ),
        demo_skill(
            "m",
            "Forecast sheet",
            "Finance",
            false,
            SkillSource::Community,
            Some("@cfo_kit"),
        ),
        demo_skill(
            "n",
            "Survey readout",
            "Research",
            false,
            SkillSource::BuiltIn,
            None,
        ),
        demo_skill(
            "o",
            "Onboarding email",
            "Sales",
            false,
            SkillSource::Community,
            Some("@growth"),
        ),
        demo_skill(
            "p",
            "Stand-up summary",
            "Ops",
            false,
            SkillSource::BuiltIn,
            None,
        ),
    ];
    skills[7].is_enabled = false;
    skills[1].trigger_phrases = vec![
        "customer asks for a digest".into(),
        "weekly customer summary".into(),
    ];
    skills[1].step_texts = vec![
        "Collect the week's tickets".into(),
        "Group them by account".into(),
        "Write the digest".into(),
    ];
    skills[1].requirement_list = vec![
        Requirement {
            entry_label: "Read the ticket queue".into(),
            permissions_link: true,
        },
        Requirement {
            entry_label: "Send mail to the team".into(),
            permissions_link: true,
        },
    ];
    skills[1].usage_log = vec![
        SkillUsage {
            task_name: "Monday digest".into(),
            occurred_at: 0,
        },
        SkillUsage {
            task_name: "Friday digest".into(),
            occurred_at: 0,
        },
    ];
    let mut last = use_signal(|| "no callback yet".to_string());
    rsx! {
        Item { slug: "agent-skill-library", name: "AgentSkillLibrary",
            div { style: "height: 720px; overflow: auto;",
                AgentSkillLibrary {
                    skill_records: skills,
                    initial_skill_id: Some("c".to_string()),
                    category_names: vec!["Ops".to_string(), "Sales".to_string(), "Support".to_string(), "Finance".to_string(), "Research".to_string()],
                    on_install: move |id: String| last.set(format!("install {id}")),
                    on_toggle: move |(id, on): (String, bool)| last.set(format!("toggle {id} {on}")),
                    on_remove: move |id: String| last.set(format!("remove {id}")),
                    on_create: move |skill: NewSkill| last.set(format!("create {} in {}", skill.entry_name, skill.category_name)),
                    on_select: move |id: String| last.set(format!("select {id}")),
                    on_navigate: move |target: String| last.set(format!("navigate {target}")),
                }
            }
            div { class: "gl-out", "data-out": "agent-skill-library", "{last()}" }
        }
        Item { slug: "agent-skill-library-empty", name: "AgentSkillLibrary (empty)",
            div { style: "height: 640px; overflow: auto;",
                AgentSkillLibrary { is_empty: true, category_names: vec!["Ops".to_string()] }
            }
        }
        Item { slug: "agent-skill-library-loading", name: "AgentSkillLibrary (loading)",
            div { style: "height: 640px; overflow: auto;",
                AgentSkillLibrary { is_loading: true }
            }
        }
    }
}

fn with_change(mut row: Breakdown, change: f64) -> Breakdown {
    row.change_vs_last_month = Some(change);
    row
}

fn breakdown(id: &str, amount: f64) -> Breakdown {
    let weights = [
        0.8, 1.1, 0.9, 1.0, 1.2, 0.7, 0.95, 1.05, 1.0, 0.9, 1.1, 1.0, 0.85, 1.15, 0.95, 1.2, 1.0,
        0.9, 1.05,
    ];
    let weight_total: f64 = weights.iter().sum();
    let tasks = [
        ("Weekly digest", 12, 0.35),
        ("Invoice sync", 9, 0.25),
        ("Lead scoring", 7, 0.2),
        ("Churn report", 4, 0.2),
    ];
    Breakdown {
        id: id.into(),
        entry_name: id.into(),
        amount,
        change_vs_last_month: Some(12.0),
        daily_costs: weights.iter().map(|w| amount * w / weight_total).collect(),
        top_tasks: tasks
            .iter()
            .map(|(title, runs, share)| TopTask {
                entry_title: (*title).into(),
                run_count: *runs,
                cost: amount * share,
            })
            .collect(),
    }
}

#[component]
fn AgentSpendOverviewDemo() -> Element {
    let mut last = use_signal(|| "no callback yet".to_string());
    let mut daily = use_signal(|| {
        vec![
            13.1, 14.8, 12.6, 15.3, 14.2, 13.9, 11.4, 16.8, 14.5, 13.7, 15.0, 12.2, 14.9, 13.3,
            12.8, 17.1, 14.0, 13.6, 17.2,
        ]
    });
    let mut projects = use_signal(|| {
        vec![
            with_change(breakdown("Harbor", 118.4), 12.0),
            with_change(breakdown("Dune", 82.1), -4.0),
            with_change(breakdown("Fern", 46.3), 6.0),
            with_change(breakdown("Internal", 23.6), -3.0),
        ]
    });
    let mut models = use_signal(|| {
        vec![
            with_change(breakdown("Opus", 124.8), 15.0),
            with_change(breakdown("Sonnet", 96.4), -2.0),
            with_change(breakdown("Haiku", 31.2), 3.5),
            with_change(breakdown("Embeddings", 18.0), -30.0),
        ]
    });
    let add_spend = move |_| {
        daily.with_mut(|costs| {
            if let Some(day) = costs.last_mut() {
                *day += 30.0;
            }
        });
        projects.with_mut(|list| {
            if let Some(row) = list.iter_mut().find(|b| b.id == "Dune") {
                row.amount += 30.0;
            }
        });
        models.with_mut(|list| {
            if let Some(row) = list.iter_mut().find(|b| b.id == "Opus") {
                row.amount += 30.0;
            }
        });
        last.set("spend +30 on Dune and Opus".to_string());
    };
    rsx! {
        Item { slug: "agent-spend-overview", name: "AgentSpendOverview",
            button {
                r#type: "button",
                class: "gl-btn",
                onclick: add_spend,
                "Add $30 of spend"
            }
            div { style: "height: 720px;",
                AgentSpendOverview {
                    month_label: "November 2026",
                    currency_code: "USD",
                    daily_costs: daily(),
                    days_in_month: 30,
                    budget_amount: Some(500.0),
                    project_breakdown: projects(),
                    model_breakdown: models(),
                    attention_items: vec![
                        AttentionItem { id: "a1".into(), item_kind: AttentionKind::Spike, text: "Harbor spent 3x its usual on Tuesday".into(), detail_text: Some("Opus runs".into()) },
                        AttentionItem { id: "a2".into(), item_kind: AttentionKind::Jump, text: "Fern cost up 21% this week".into(), detail_text: Some("Two new agents added on 9 November".into()) },
                        AttentionItem { id: "a3".into(), item_kind: AttentionKind::Budget, text: "Internal has used 5% of its share".into(), detail_text: Some("Share of the $500 budget".into()) },
                    ],
                    usage_stats: Some(UsageStats { task_count: Some(186), run_count: Some(1284), token_count: Some(48_200_000) }),
                    on_select: move |(kind, id): (EntryKind, String)| last.set(format!("select {kind:?} {id}")),
                    on_open_item: move |item: AttentionItem| last.set(format!("open {}", item.id)),
                }
            }
            div { class: "gl-out", "data-out": "agent-spend-overview", "{last()}" }
        }
        Item { slug: "agent-spend-overview-empty", name: "AgentSpendOverview (empty)",
            div { style: "height: 640px; overflow: auto;",
                AgentSpendOverview { month_label: "November 2026", currency_code: "USD", is_empty: true }
            }
        }
        Item { slug: "agent-spend-overview-loading", name: "AgentSpendOverview (loading)",
            div { style: "height: 640px; overflow: auto;",
                AgentSpendOverview { month_label: "November 2026", currency_code: "USD", is_loading: true }
            }
        }
    }
}

fn list_task(
    id: &str,
    title: &str,
    project: &str,
    state: Progress,
    (done, total): (usize, usize),
    asks: usize,
    is_done: bool,
) -> ListTask {
    ListTask {
        id: id.into(),
        entry_title: title.into(),
        project_name: project.into(),
        progress_state: state,
        status_phrase: None,
        step_records: (0..total)
            .map(|i| TaskStep {
                entry_label: format!("Step {}", i + 1),
                is_done: i < done,
            })
            .collect(),
        artifact_names: vec!["Summary.md".into()],
        pending_asks: (0..asks)
            .map(|i| Ask {
                id: format!("{id}-{i}"),
                question_text: "Send the draft to the client?".into(),
                ask_kind: AskKind::Approval,
            })
            .collect(),
        updated_at_ms: gallery_now() - if is_done { 2 * DAY } else { 600_000 },
        is_done,
        goal_text: Some("Ship the Q3 summary".into()),
        started_at_ms: None,
        decision_log: vec![DecisionEntry {
            question_text: "Tone?".into(),
            answer_text: "Plain".into(),
            at_ms: gallery_now() - DAY,
        }],
        event_log: Vec::new(),
    }
}

fn list_demo_task(
    id: &str,
    title: &str,
    project: &str,
    state: Progress,
    (done, total): (usize, usize),
    asks: usize,
) -> ListTask {
    let mut task = list_task(id, title, project, state, (done, total), asks, false);
    task.status_phrase = (state == Progress::Running).then(|| "In progress".to_string());
    task
}

fn demo_tasks() -> Vec<ListTask> {
    let mut lead = list_demo_task("t1", "Q3 summary", "Launch", Progress::Running, (2, 4), 1);
    lead.artifact_names = vec!["Q3 brief".into(), "Q3 figures.csv".into()];
    lead.event_log = vec![
        TaskEvent {
            at_ms: gallery_now() - 60_000,
            entry_text: "Asked to approve the client send".into(),
        },
        TaskEvent {
            at_ms: gallery_now() - 3_600_000,
            entry_text: "Drafted the brief from the figures".into(),
        },
        TaskEvent {
            at_ms: gallery_now() - 2 * DAY,
            entry_text: "Started the run".into(),
        },
    ];
    vec![
        lead,
        list_demo_task(
            "t2",
            "Pricing page copy",
            "Launch",
            Progress::Planning,
            (0, 3),
            0,
        ),
        list_demo_task("t3", "Launch email", "Launch", Progress::Running, (1, 3), 0),
        list_demo_task(
            "t4",
            "Onboarding checklist",
            "Launch",
            Progress::Paused,
            (2, 4),
            0,
        ),
        list_demo_task(
            "t5",
            "Candidate screen",
            "Hiring",
            Progress::Running,
            (1, 2),
            1,
        ),
        list_demo_task(
            "t6",
            "Interview plan",
            "Hiring",
            Progress::Paused,
            (0, 3),
            0,
        ),
        list_demo_task("t7", "Offer letter", "Hiring", Progress::Paused, (1, 2), 0),
        list_demo_task(
            "t8",
            "Competitor scan",
            "Research",
            Progress::Running,
            (3, 4),
            0,
        ),
        list_demo_task(
            "t9",
            "Survey readout",
            "Research",
            Progress::Paused,
            (2, 3),
            0,
        ),
    ]
}

#[component]
fn AgentTaskListDemo() -> Element {
    let mut tasks = use_signal(demo_tasks);
    let mut last = use_signal(|| "no callback yet".to_string());
    let projects = vec![
        "Launch".to_string(),
        "Hiring".to_string(),
        "Research".to_string(),
    ];
    rsx! {
        Item { slug: "agent-task-list", name: "AgentTaskList",
            div { style: "height: 720px; overflow: auto;",
                AgentTaskList {
                    task_records: tasks(),
                    project_names: projects.clone(),
                    initial_task_id: Some("t1".to_string()),
                    now_ms: gallery_now(),
                    on_select: move |id: String| last.set(format!("select {id}")),
                    on_create: move |new: NewTaskLine| {
                        last.set(format!("create {} in {}", new.entry_title, new.project_name));
                        let id = format!("n{}", tasks().len() + 1);
                        tasks.with_mut(|list| {
                            list.insert(0, list_task(&id, &new.entry_title, &new.project_name, Progress::Planning, (0, 0), 0, false));
                        });
                    },
                    on_open_decision: move |(task, ask): (String, Option<String>)| {
                        last.set(format!("decide {task} {}", ask.unwrap_or_default()));
                    },
                }
            }
            div { class: "gl-out", "data-out": "agent-task-list", "{last()}" }
        }
        Item { slug: "agent-task-list-empty", name: "AgentTaskList (empty)",
            div { style: "height: 640px; overflow: auto;",
                AgentTaskList {
                    project_names: projects.clone(),
                    is_empty: true,
                    now_ms: gallery_now(),
                }
            }
        }
        Item { slug: "agent-task-list-loading", name: "AgentTaskList (loading)",
            div { style: "height: 640px; overflow: auto;",
                AgentTaskList { project_names: projects, is_loading: true, now_ms: gallery_now() }
            }
        }
    }
}

fn call(id: &str, tool: &str, state: CallState, at: u64, ms: Option<u64>) -> CallRecord {
    CallRecord {
        id: id.into(),
        tool_id: tool.into(),
        started_at: at,
        duration_ms: ms,
        call_state: state,
        input_data: format!("{tool} request"),
        output_data: Some("ok".into()),
        error_text: (state == CallState::Failed).then(|| "timeout".to_string()),
        token_count: Some(420),
        run_id: Some("run-7f2".to_string()),
    }
}

fn tool_from(id: &str, name: &str, access: Access, source: &str) -> ToolInfo {
    ToolInfo {
        id: id.into(),
        entry_name: name.into(),
        function_name: format!("{id}_call"),
        source_label: source.into(),
        entry_description: format!("Lets the agent use {name}."),
        access_mode: access,
        parameter_list: vec![Param {
            entry_name: "query".into(),
            type_name: "string".into(),
            is_required: true,
        }],
    }
}

#[component]
fn AgentToolExplorerDemo() -> Element {
    let now = gallery_now();
    let mut tools = use_signal(|| {
        vec![
            tool_from("docs", "Docs", Access::Auto, "Built-in"),
            tool_from("mail", "Mail", Access::Ask, "Connected"),
            tool_from("crm", "CRM", Access::Off, "Connected"),
            tool_from("calendar", "Calendar", Access::Auto, "Built-in"),
            tool_from("sheets", "Sheets", Access::Auto, "Own"),
            tool_from("web", "Web search", Access::Ask, "Built-in"),
            tool_from("files", "Files", Access::Auto, "Built-in"),
        ]
    });
    let mut calls = use_signal(|| {
        vec![
            call("c1", "docs", CallState::Running, now - 4_000, None),
            call("c2", "mail", CallState::Waiting, now - 20_000, None),
            call("c3", "web", CallState::Failed, now - 90_000, Some(1200)),
            call("c4", "docs", CallState::Done, now - 200_000, Some(480)),
            call("c5", "crm", CallState::Rejected, now - 300_000, None),
            call("c6", "calendar", CallState::Done, now - 540_000, Some(350)),
            call("c7", "sheets", CallState::Done, now - 840_000, Some(910)),
            call("c8", "files", CallState::Done, now - 1_500_000, Some(260)),
            call("c9", "docs", CallState::Done, now - 2_400_000, Some(610)),
        ]
    });
    let mut last = use_signal(|| "no callback yet".to_string());
    let mut last_compact = use_signal(|| "no callback yet".to_string());
    let mut last_canvas = use_signal(|| "no callback yet".to_string());
    let agent = || AgentInfo {
        id: "ada".into(),
        entry_name: "Ada".into(),
        task_name: "Support summary".into(),
        model_name: "Claude Sonnet".into(),
        is_paused: false,
    };
    rsx! {
        Item { slug: "agent-tool-explorer", name: "AgentToolExplorer",
            div { style: "height: 760px; overflow: auto;",
                AgentToolExplorer {
                    agent_info: Some(agent()),
                    tool_list: tools(),
                    call_log: calls(),
                    now_at: now,
                    is_live: true,
                    time_range: TimeRange::SevenDays,
                    on_select: move |id: Option<String>| last.set(format!("select {}", id.unwrap_or_else(|| "agent".into()))),
                    on_pause: move |paused: bool| last.set(format!("pause {paused}")),
                    on_add_tool: move |_: ()| last.set("connect tool".to_string()),
                    on_resolve: move |(call, approve): (CallRecord, bool)| {
                        last.set(format!("{} {}", if approve { "approve" } else { "reject" }, call.id));
                        calls.with_mut(|list| {
                            if let Some(c) = list.iter_mut().find(|c| c.id == call.id) {
                                c.call_state = if approve { CallState::Done } else { CallState::Rejected };
                            }
                        });
                    },
                    on_access_change: move |(tool_id, access): (String, Access)| {
                        last.set(format!("access {tool_id} {}", access.key()));
                    },
                    on_disconnect: move |tool_id: String| {
                        last.set(format!("disconnect {tool_id}"));
                        tools.with_mut(|list| list.retain(|t| t.id != tool_id));
                    },
                }
            }
            div { class: "gl-out", "data-out": "agent-tool-explorer", "{last()}" }
        }
        Item { slug: "agent-tool-explorer-compact", name: "AgentToolExplorer (compact)",
            div { style: "height: 760px; overflow: auto;",
                AgentToolExplorer {
                    agent_info: Some(agent()),
                    tool_list: tools(),
                    call_log: calls(),
                    now_at: now,
                    is_live: true,
                    chart_layout: ChartLayout::Compact,
                    on_select: move |id: Option<String>| last_compact.set(format!("select {}", id.unwrap_or_else(|| "agent".into()))),
                    on_pause: move |paused: bool| last_compact.set(format!("pause {paused}")),
                    on_access_change: move |(tool_id, access): (String, Access)| {
                        last_compact.set(format!("access {tool_id} {}", access.key()));
                    },
                }
            }
            div { class: "gl-out", "data-out": "agent-tool-explorer-compact", "{last_compact()}" }
        }
        Item { slug: "agent-tool-explorer-canvas", name: "AgentToolExplorer (canvas)",
            div { style: "height: 760px; overflow: auto;",
                AgentToolExplorer {
                    agent_info: Some(agent()),
                    tool_list: tools(),
                    call_log: calls(),
                    now_at: now,
                    is_live: true,
                    chart_layout: ChartLayout::Canvas,
                    on_select: move |id: Option<String>| last_canvas.set(format!("select {}", id.unwrap_or_else(|| "agent".into()))),
                    on_pause: move |paused: bool| last_canvas.set(format!("pause {paused}")),
                    on_access_change: move |(tool_id, access): (String, Access)| {
                        last_canvas.set(format!("access {tool_id} {}", access.key()));
                    },
                }
            }
            div { class: "gl-out", "data-out": "agent-tool-explorer-canvas", "{last_canvas()}" }
        }
        Item { slug: "agent-tool-explorer-empty", name: "AgentToolExplorer (empty)",
            div { style: "height: 760px; overflow: auto;",
                AgentToolExplorer {
                    agent_info: Some(agent()),
                    now_at: now,
                    is_empty: true,
                    on_add_tool: move |_: ()| last.set("connect tool".to_string()),
                }
            }
        }
        Item { slug: "agent-tool-explorer-loading", name: "AgentToolExplorer (loading)",
            div { style: "height: 760px; overflow: auto;",
                AgentToolExplorer { is_loading: true, now_at: now }
            }
        }
    }
}

#[component]
fn AgentTaskBoardDemo() -> Element {
    let now = gallery_now();
    let mut tasks = use_signal(|| {
        vec![
            demo_task(
                "t1",
                "Reconcile Q3 invoices",
                TaskState::Waiting,
                "ada",
                None,
            ),
            demo_task(
                "t2",
                "Send the vendor reminder",
                TaskState::Failed,
                "lin",
                None,
            ),
            demo_task(
                "t3",
                "Summarise support tickets",
                TaskState::Running,
                "ada",
                Some(now - 252_000),
            ),
            demo_task("t4", "Draft the board memo", TaskState::Done, "sam", None),
            demo_task(
                "t5",
                "Refresh the price list",
                TaskState::Stopped,
                "lin",
                None,
            ),
        ]
    });
    let mut last = use_signal(|| "no callback yet".to_string());
    let mut last_empty = use_signal(|| "no callback yet".to_string());
    let mut set_state = move |id: String, state: TaskState| {
        tasks.with_mut(|list| {
            if let Some(task) = list.iter_mut().find(|t| t.id == id) {
                task.task_state = state;
            }
        });
    };
    rsx! {
        Item { slug: "agent-task-board", name: "AgentTaskBoard",
            div { style: "height: 640px; overflow: auto;",
                AgentTaskBoard {
                    tasks: tasks(),
                    agents: demo_agents(),
                    initial_task_id: Some("t1".to_string()),
                    clock_ms: Some(now),
                    on_approve: move |task: TaskRecord| {
                        last.set(format!("approve {}", task.id));
                        set_state(task.id, TaskState::Running);
                    },
                    on_reject: move |task: TaskRecord| {
                        last.set(format!("reject {}", task.id));
                        set_state(task.id, TaskState::Stopped);
                    },
                    on_retry: move |task: TaskRecord| {
                        last.set(format!("retry {}", task.id));
                        set_state(task.id, TaskState::Running);
                    },
                    on_open: move |task: TaskRecord| last.set(format!("open {}", task.id)),
                    on_create: move |new: NewTask| {
                        last.set(format!("create {} by {}", new.description, new.agent_id));
                        let id = format!("n{}", tasks().len() + 1);
                        tasks.with_mut(|list| {
                            list.insert(0, demo_task(&id, &new.description, TaskState::Running, &new.agent_id, Some(now)));
                        });
                    },
                }
            }
            div { class: "gl-out", "data-out": "agent-task-board", "{last()}" }
        }
        Item { slug: "agent-task-board-empty", name: "AgentTaskBoard (empty)",
            div { style: "height: 640px; overflow: auto;",
                AgentTaskBoard {
                    agents: demo_agents(),
                    is_empty: true,
                    clock_ms: Some(now),
                    on_create: move |new: NewTask| last_empty.set(format!("create {} by {}", new.description, new.agent_id)),
                }
            }
            div { class: "gl-out", "data-out": "agent-task-board-empty", "{last_empty()}" }
        }
        Item { slug: "agent-task-board-loading", name: "AgentTaskBoard (loading)",
            div { style: "height: 640px; overflow: auto;",
                AgentTaskBoard { agents: demo_agents(), is_loading: true, clock_ms: Some(now) }
            }
            div { class: "gl-out", "data-out": "agent-task-board-loading", "callbacks are off while loading" }
        }
    }
}

fn demo_task(
    id: &str,
    title: &str,
    state: TaskState,
    agent: &str,
    started: Option<u64>,
) -> TaskRecord {
    TaskRecord {
        id: id.to_string(),
        entry_title: title.to_string(),
        entry_description: Some(
            "Pull the Q3 figures and check them against the ledger.".to_string(),
        ),
        agent_id: agent.to_string(),
        task_state: state,
        started_at: started,
        elapsed_seconds: Some(252),
        source_count: Some(4),
        tool_count: Some(3),
        step_records: vec![
            StepRecord {
                entry_label: "Read the spreadsheet".to_string(),
                time_label: Some("0:04".to_string()),
            },
            StepRecord {
                entry_label: "Compare with the ledger".to_string(),
                time_label: Some("1:12".to_string()),
            },
        ],
    }
}

fn demo_agents() -> Vec<AgentRecord> {
    vec![
        AgentRecord {
            id: "ada".into(),
            entry_name: "Ada".into(),
            identity_hue_token: Some("ok".into()),
        },
        AgentRecord {
            id: "lin".into(),
            entry_name: "Lin".into(),
            identity_hue_token: Some("accent".into()),
        },
        AgentRecord {
            id: "sam".into(),
            entry_name: "Sam".into(),
            identity_hue_token: None,
        },
    ]
}
