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
    AgentTaskList, Ask, AskKind, DecisionEntry, NewTaskLine, Progress, TaskRecord as ListTask,
    TaskStep,
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
    let now = gallery_now();
    let mut steps = use_signal(|| {
        vec![
            LogStep {
                id: "a1".into(),
                occurred_at: now - 600_000,
                app_id: "mail".into(),
                sentence: "Sent the renewal reminder to Harbor Labs".into(),
                task_name: Some("Renewals".into()),
                actor: Actor::Automatic,
                rule_label: Some("Renewal reminders".into()),
                approved_at: None,
                detail_rows: vec![DetailRow {
                    entry_label: "To".into(),
                    entry_value: "ops@harbor.example".into(),
                }],
                preview_lines: vec!["Hi team, your renewal is due on the 30th.".into()],
                undo_window: Some(UndoWindow {
                    available_until: now + 300_000,
                    entry_description: "recall the message".into(),
                }),
                final_reason: None,
                is_undone: false,
                undone_at: None,
                cost_label: Some("$0.01".into()),
            },
            LogStep {
                id: "a2".into(),
                occurred_at: now - 3_600_000,
                app_id: "crm".into(),
                sentence: "Moved Acme deal to Negotiation".into(),
                task_name: Some("Pipeline review".into()),
                actor: Actor::Approved,
                rule_label: None,
                approved_at: Some(now - 3_900_000),
                detail_rows: vec![],
                preview_lines: vec![],
                undo_window: Some(UndoWindow {
                    available_until: now + 60_000,
                    entry_description: "return the deal to Proposal".into(),
                }),
                final_reason: None,
                is_undone: false,
                undone_at: None,
                cost_label: None,
            },
            LogStep {
                id: "a3".into(),
                occurred_at: now - 2 * DAY,
                app_id: "pay".into(),
                sentence: "Paid invoice 1182 for hosting".into(),
                task_name: None,
                actor: Actor::Approved,
                rule_label: None,
                approved_at: Some(now - 2 * DAY),
                detail_rows: vec![],
                preview_lines: vec![],
                undo_window: None,
                final_reason: Some("Payments cannot be reversed once sent.".into()),
                is_undone: false,
                undone_at: None,
                cost_label: Some("$240.00".into()),
            },
        ]
    });
    let apps = vec![
        AppRecord {
            id: "mail".into(),
            entry_name: "Mail".into(),
            account: None,
            mark: None,
        },
        AppRecord {
            id: "crm".into(),
            entry_name: "CRM".into(),
            account: None,
            mark: None,
        },
        AppRecord {
            id: "pay".into(),
            entry_name: "Payments".into(),
            account: None,
            mark: None,
        },
    ];
    let mut last = use_signal(|| "no callback yet".to_string());
    let empty_apps = apps.clone();
    rsx! {
        Item { slug: "agent-action-log", name: "AgentActionLog",
            div { style: "height: 640px; overflow: auto;",
                AgentActionLog {
                    app_list: apps,
                    step_list: steps(),
                    selected_step_id: Some("a1".to_string()),
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

#[component]
fn AgentMemoryManagerDemo() -> Element {
    let now = gallery_now();
    let memories = vec![
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
    let mut memory_list = use_signal(move || memories.clone());
    let mut last = use_signal(|| "no callback yet".to_string());
    rsx! {
        Item { slug: "agent-memory-manager", name: "AgentMemoryManager",
            div { style: "height: 640px; overflow: auto;",
                AgentMemoryManager {
                    memory_list: memory_list(),
                    project_names: vec!["Launch".to_string(), "Hiring".to_string()],
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
            button {
                r#type: "button",
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
            div { style: "height: 640px; overflow: auto;",
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

#[component]
fn AgentRunTimelineDemo() -> Element {
    let elapsed = 95.0;
    let run = RunRecord {
        id: "r1".into(),
        entry_title: "Summarise the support queue".into(),
        agent_name: "Ada".into(),
        model_name: "Claude Sonnet".into(),
        started_at_ms: 0,
        total_seconds: None,
        cost_label: Some("$0.42".into()),
        step_records: vec![
            RunStepRec {
                id: "s1".into(),
                entry_label: "Plan the summary".into(),
                step_type: StepType::Model,
                start_seconds: 0.0,
                duration_seconds: Some(6.0),
                nesting_depth: 0,
                app_name: None,
                sub_agent_name: None,
                has_failed: false,
                error_text: None,
                token_counts: Some((820, 140)),
                cost_label: Some("$0.02".into()),
                input_text: Some("Summarise the open tickets.".into()),
                output_text: Some("1. Pull tickets 2. Group 3. Write".into()),
            },
            RunStepRec {
                id: "s2".into(),
                entry_label: "Read tickets".into(),
                step_type: StepType::Tool,
                start_seconds: 6.0,
                duration_seconds: Some(14.0),
                nesting_depth: 0,
                app_name: Some("Helpdesk".into()),
                sub_agent_name: None,
                has_failed: false,
                error_text: None,
                token_counts: None,
                cost_label: None,
                input_text: Some("query: status:open".into()),
                output_text: Some("42 tickets".into()),
            },
            RunStepRec {
                id: "s3".into(),
                entry_label: "Group by topic".into(),
                step_type: StepType::SubAgent,
                start_seconds: 20.0,
                duration_seconds: Some(40.0),
                nesting_depth: 0,
                app_name: None,
                sub_agent_name: Some("Classifier".into()),
                has_failed: false,
                error_text: None,
                token_counts: None,
                cost_label: None,
                input_text: None,
                output_text: None,
            },
            RunStepRec {
                id: "s4".into(),
                entry_label: "Label tickets".into(),
                step_type: StepType::Model,
                start_seconds: 22.0,
                duration_seconds: Some(12.0),
                nesting_depth: 1,
                app_name: None,
                sub_agent_name: Some("Classifier".into()),
                has_failed: true,
                error_text: Some("Rate limit reached, retried later.".into()),
                token_counts: Some((300, 20)),
                cost_label: None,
                input_text: Some("tickets 1-20".into()),
                output_text: None,
            },
            RunStepRec {
                id: "s5".into(),
                entry_label: "Write the summary".into(),
                step_type: StepType::Model,
                start_seconds: 60.0,
                duration_seconds: None,
                nesting_depth: 0,
                app_name: None,
                sub_agent_name: None,
                has_failed: false,
                error_text: None,
                token_counts: Some((1200, 80)),
                cost_label: None,
                input_text: None,
                output_text: None,
            },
        ],
    };
    let finished = RunRecord {
        total_seconds: Some(150.0),
        ..run.clone()
    };
    let mut last = use_signal(|| "no callback yet".to_string());
    rsx! {
        Item { slug: "agent-run-timeline", name: "AgentRunTimeline",
            AgentRunTimeline {
                run_record: Some(run),
                elapsed_seconds: Some(elapsed),
                initial_step_id: Some("s4".to_string()),
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

fn schedule(id: &str, name: &str, now: u64, offset_min: i64, paused: bool) -> ScheduleRecord {
    let at = |minutes: i64| (now as i64 + minutes * 60_000) as u64;
    ScheduleRecord {
        id: id.into(),
        entry_name: name.into(),
        project_name: Some("Support".into()),
        brief_summary: Some("Summarise the ticket queue".into()),
        cadence_text: "Every 2 hours".into(),
        run_times: vec![
            RunTime {
                at_ms: at(offset_min - 240),
                run_state: MarkState::Done,
                progress_percent: None,
            },
            RunTime {
                at_ms: at(offset_min - 120),
                run_state: MarkState::Failed,
                progress_percent: None,
            },
            RunTime {
                at_ms: at(offset_min),
                run_state: MarkState::Running,
                progress_percent: Some(38),
            },
            RunTime {
                at_ms: at(offset_min + 120),
                run_state: MarkState::Upcoming,
                progress_percent: None,
            },
            RunTime {
                at_ms: at(offset_min + 240),
                run_state: MarkState::Upcoming,
                progress_percent: None,
            },
        ],
        duration_seconds: Some(900),
        is_paused: paused,
        history_log: vec![HistoryEntry {
            at_ms: at(-240),
            outcome: Outcome::Done,
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
    let schedules = vec![
        schedule("sum", "Ticket summary", now, 0, false),
        schedule("tri", "Triage", now, 60, true),
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

#[component]
fn AgentSkillLibraryDemo() -> Element {
    let skills = vec![
        skill("w", "Weekly report", true, SkillSource::BuiltIn),
        skill("c", "Customer digest", true, SkillSource::Own),
        skill("d", "Invoice check", false, SkillSource::Community),
    ];
    let mut last = use_signal(|| "no callback yet".to_string());
    rsx! {
        Item { slug: "agent-skill-library", name: "AgentSkillLibrary",
            div { style: "height: 640px; overflow: auto;",
                AgentSkillLibrary {
                    skill_records: skills,
                    category_names: vec!["Ops".to_string(), "Sales".to_string()],
                    initial_skill_id: Some("w".to_string()),
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

fn breakdown(id: &str, amount: f64) -> Breakdown {
    Breakdown {
        id: id.into(),
        entry_name: id.into(),
        amount,
        change_vs_last_month: Some(12.0),
        daily_costs: vec![amount / 10.0; 10],
        top_tasks: vec![TopTask {
            entry_title: "Weekly digest".into(),
            run_count: 6,
            cost: amount / 2.0,
        }],
    }
}

#[component]
fn AgentSpendOverviewDemo() -> Element {
    let mut last = use_signal(|| "no callback yet".to_string());
    let mut daily = use_signal(|| vec![22.0, 18.0, 25.0, 16.0, 21.0, 20.0, 24.0, 18.0, 22.0, 18.0]);
    let mut projects =
        use_signal(|| vec![breakdown("Support", 84.0), breakdown("Research", 120.0)]);
    let mut models = use_signal(|| vec![breakdown("Haiku", 40.0), breakdown("Opus", 164.0)]);
    let add_spend = move |_| {
        daily.with_mut(|costs| {
            if let Some(day) = costs.last_mut() {
                *day += 30.0;
            }
        });
        projects.with_mut(|list| {
            if let Some(row) = list.iter_mut().find(|b| b.id == "Research") {
                row.amount += 30.0;
            }
        });
        models.with_mut(|list| {
            if let Some(row) = list.iter_mut().find(|b| b.id == "Opus") {
                row.amount += 30.0;
            }
        });
        last.set("spend +30 on Research and Opus".to_string());
    };
    rsx! {
        Item { slug: "agent-spend-overview", name: "AgentSpendOverview",
            button {
                r#type: "button",
                class: "gl-btn",
                onclick: add_spend,
                "Add $30 of spend"
            }
            div { style: "height: 640px; overflow: auto;",
                AgentSpendOverview {
                    month_label: "November 2026",
                    currency_code: "USD",
                    daily_costs: daily(),
                    days_in_month: 30,
                    budget_amount: Some(400.0),
                    project_breakdown: projects(),
                    model_breakdown: models(),
                    attention_items: vec![AttentionItem { id: "a1".into(), item_kind: AttentionKind::Spike, text: "Research spent 3x its usual on Tuesday".into(), detail_text: Some("Opus runs".into()) }],
                    usage_stats: Some(UsageStats { task_count: Some(42), run_count: Some(118), token_count: Some(2_400_000) }),
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
    }
}

#[component]
fn AgentTaskListDemo() -> Element {
    let mut tasks = use_signal(|| {
        vec![
            list_task(
                "t1",
                "Q3 summary",
                "Launch",
                Progress::Running,
                (2, 4),
                1,
                false,
            ),
            list_task(
                "t2",
                "Pricing page copy",
                "Launch",
                Progress::Planning,
                (0, 3),
                0,
                false,
            ),
            list_task(
                "t3",
                "Candidate screen",
                "Hiring",
                Progress::Paused,
                (1, 2),
                0,
                false,
            ),
            list_task(
                "t4",
                "Board memo",
                "Hiring",
                Progress::Running,
                (3, 3),
                0,
                true,
            ),
        ]
    });
    let mut last = use_signal(|| "no callback yet".to_string());
    let projects = vec!["Launch".to_string(), "Hiring".to_string()];
    rsx! {
        Item { slug: "agent-task-list", name: "AgentTaskList",
            div { style: "height: 640px; overflow: auto;",
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

fn tool(id: &str, name: &str, access: Access) -> ToolInfo {
    ToolInfo {
        id: id.into(),
        entry_name: name.into(),
        function_name: format!("{id}_call"),
        source_label: "Built-in".into(),
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
            tool("docs", "Docs", Access::Auto),
            tool("mail", "Mail", Access::Ask),
            tool("crm", "CRM", Access::Off),
        ]
    });
    let mut calls = use_signal(|| {
        vec![
            call("c1", "docs", CallState::Running, now - 4_000, None),
            call("c2", "mail", CallState::Waiting, now - 20_000, None),
            call("c3", "docs", CallState::Failed, now - 90_000, Some(1200)),
            call("c4", "docs", CallState::Done, now - 200_000, Some(480)),
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
