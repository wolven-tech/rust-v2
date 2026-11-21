use dioxus::prelude::*;

use super::{Item, NOW_MS, Section, gallery_now};

#[component]
pub fn Timelines() -> Element {
    rsx! {
        Section { id: "timelines", title: "Timelines",
            AbsenceDigestDemo {}
            ChangeLogTimelineDemo {}
            ChangeReplayDemo {}
            RunTimingWaterfallDemo {}
            ScheduleVarianceTimelineDemo {}
            SessionPackingPlannerDemo {}
            QuotaLoadSchedulerDemo {}
        }
    }
}

#[component]
fn AbsenceDigestDemo() -> Element {
    use rv2_kit::components::absence_digest::{
        AbsenceDigest, DigestEntry, EntryButton, EntryState, TimeFormat,
    };
    let mut last = use_signal(String::new);
    let entry = |id: &str, name: &str, state: EntryState, button: EntryButton| DigestEntry {
        entry_id: id.into(),
        entry_name: name.into(),
        entry_state: state,
        entry_note: Some("While you were away".into()),
        button_text: button,
    };
    let entries = vec![
        entry(
            "e1",
            "Nightly export finished",
            EntryState::Completed,
            EntryButton::Default,
        ),
        entry(
            "e2",
            "Sync failed for Drive",
            EntryState::Failed,
            EntryButton::Label("Retry".into()),
        ),
        entry(
            "e3",
            "Agent needs an answer",
            EntryState::AwaitingInput,
            EntryButton::Label("Answer".into()),
        ),
    ];
    rsx! {
        Item { slug: "absence-digest", name: "AbsenceDigest",
            AbsenceDigest {
                away_since: (NOW_MS as i64) / 1000 - 8 * 3600,
                digest_entries: entries,
                current_time: gallery_now() as i64 / 1000,
                time_format: TimeFormat::Clock,
                row_limit: 2,
                on_entry_action: move |id: String| last.set(format!("action {id}")),
                on_acknowledge: move |_| last.set("acknowledged".into()),
                on_fold_change: move |f: bool| last.set(format!("folded {f}")),
            }
            div { class: "gl-out", "data-out": "absence-digest", "{last()}" }
        }
    }
}

#[component]
fn ChangeLogTimelineDemo() -> Element {
    use rv2_kit::components::change_log_timeline::{
        Change, ChangeLogTimeline, ChangeWriter, Timestamp,
    };
    let mut last = use_signal(String::new);
    let writer = |name: &str, agent: bool| ChangeWriter {
        writer_name: name.into(),
        is_agent: agent,
        lane_color: None,
        writer_mark: None,
    };
    let change = |id: &str, msg: &str, who: ChangeWriter, at: i64, added: u32| Change {
        change_id: id.into(),
        change_message: msg.into(),
        change_author: who,
        co_author: None,
        changed_at: Some(Timestamp::Epoch(at)),
        change_kind: Some("commit".into()),
        file_count: Some(2),
        lines_added: Some(added),
        lines_removed: Some(3),
        change_body: None,
        merged_changes: vec![],
        in_progress: false,
    };
    let mut merge = change(
        "c2",
        "Merge feature branch",
        writer("Ada", false),
        (NOW_MS as i64) / 1000 - 3600,
        40,
    );
    merge.merged_changes = vec![change(
        "c2a",
        "Add table kit",
        writer("Writer agent", true),
        (NOW_MS as i64) / 1000 - 4000,
        120,
    )];
    let changes = vec![
        change(
            "c3",
            "Draft kit notes",
            writer("Writer agent", true),
            (NOW_MS as i64) / 1000 - 60,
            88,
        ),
        change(
            "c1",
            "Fix tooltip focus",
            writer("Ben", false),
            (NOW_MS as i64) / 1000 - 600,
            12,
        ),
        merge,
    ];
    let mut open_id = use_signal(|| None::<String>);
    rsx! {
        Item { slug: "change-log-timeline", name: "ChangeLogTimeline",
            ChangeLogTimeline {
                change_list: changes,
                branch_names: vec!["main".to_string(), "feat/kit".to_string()],
                current_branch: "feat/kit",
                now_secs: gallery_now() as i64 / 1000,
                release_tag: Some("v1.4.0".to_string()),
                on_branch_switch: move |b: String| last.set(format!("branch {b}")),
                open_change_id: open_id(),
                on_open_change: move |id: Option<String>| {
                    last.set(format!("open {id:?}"));
                    open_id.set(id);
                },
            }
            div { class: "gl-out", "data-out": "change-log-timeline", "{last()}" }
        }
    }
}

#[component]
fn ChangeReplayDemo() -> Element {
    use rv2_kit::components::change_replay::{ChangeEvent, ChangeReplay, ReplayAuthor};
    let ev = |at: f64, file: &str, added: u32| ChangeEvent {
        at_offset: at,
        file_path: file.into(),
        lines_added: added,
        lines_removed: 1,
        line_details: vec!["+ new line".into()],
    };
    rsx! {
        Item { slug: "change-replay", name: "ChangeReplay",
            ChangeReplay {
                author: ReplayAuthor { author_id: "writer".into(), author_name: "Writer agent".into() },
                context_text: Some("Rewriting the onboarding copy".to_string()),
                change_events: vec![ev(1.0, "onboarding.md", 12), ev(3.0, "style.md", 4), ev(6.0, "onboarding.md", 9)],
                axis_length: Some(10.0),
            }
        }
    }
}

#[component]
fn RunTimingWaterfallDemo() -> Element {
    use rv2_kit::components::run_timing_waterfall::{
        NodeStatus, NodeType, RunTimingWaterfall, TraceNode,
    };
    let node = |id: &str,
                label: &str,
                kind: NodeType,
                status: NodeStatus,
                begin: f64,
                end: Option<f64>,
                children: Vec<TraceNode>| TraceNode {
        node_id: id.into(),
        node_label: label.into(),
        node_type: kind,
        node_status: status,
        begin_offset: begin,
        finish_offset: end,
        node_note: None,
        node_tag: None,
        model_label: Some("model A".into()),
        prompt_tokens: Some(2_400.0),
        completion_tokens: Some(380.0),
        node_cost: Some(0.02),
        input_excerpt: Some("Summarise the thread".into()),
        output_excerpt: None,
        failure_text: None,
        children,
    };
    let mut failing = node(
        "n3",
        "Model call",
        NodeType::Model,
        NodeStatus::Failed,
        2.0,
        Some(4.5),
        vec![],
    );
    failing.failure_text = Some("Context too long".into());
    let trace = node(
        "root",
        "Summarise thread",
        NodeType::Agent,
        NodeStatus::Running,
        0.0,
        None,
        vec![
            node(
                "n1",
                "Plan",
                NodeType::Model,
                NodeStatus::Done,
                0.0,
                Some(1.5),
                vec![],
            ),
            node(
                "n2",
                "Search",
                NodeType::Tool,
                NodeStatus::Done,
                1.5,
                Some(2.0),
                vec![],
            ),
            failing,
            node(
                "n4",
                "Retry with shorter input",
                NodeType::Subagent,
                NodeStatus::Running,
                4.5,
                None,
                vec![],
            ),
        ],
    );
    rsx! {
        Item { slug: "run-timing-waterfall", name: "RunTimingWaterfall",
            RunTimingWaterfall {
                trace_tree: trace,
                clock_point: Some(6.0),
                heading: "Trace",
                flag_slowest: true,
            }
        }
    }
}

#[component]
fn ScheduleVarianceTimelineDemo() -> Element {
    use rv2_kit::components::schedule_variance_timeline::{PlanStep, ScheduleVarianceTimeline};
    let step = |key: &str,
                label: &str,
                span: f64,
                waits: Vec<String>,
                start: Option<f64>,
                end: Option<f64>| PlanStep {
        step_key: key.into(),
        step_label: label.into(),
        planned_span: span,
        waits_for: waits,
        actual_start: start,
        actual_end: end,
        step_mark: None,
    };
    rsx! {
        Item { slug: "schedule-variance-timeline", name: "ScheduleVarianceTimeline",
            ScheduleVarianceTimeline {
                step_list: vec![
                    step("a", "Collect data", 3.0, vec![], Some(0.0), Some(4.0)),
                    step("b", "Clean data", 2.0, vec!["a".to_string()], Some(4.0), Some(5.0)),
                    step("c", "Model", 4.0, vec!["b".to_string()], Some(5.0), None),
                    step("d", "Report", 2.0, vec!["c".to_string()], None, None),
                ],
                clock_point: 6.5,
                run_purpose: Some("Monthly forecast".to_string()),
                board_title: "Plan vs actual",
            }
        }
    }
}

#[component]
fn SessionPackingPlannerDemo() -> Element {
    use rv2_kit::components::session_packing_planner::{
        PackMode, ScheduledTask, SessionPackingPlanner, Slot,
    };
    let mut last = use_signal(String::new);
    let task = |id: &str, title: &str, share: f64, slot: Option<Slot>| ScheduledTask {
        task_id: id.into(),
        task_title: title.into(),
        usage_share: share,
        agent_name: Some("Planner".into()),
        task_detail: None,
        agent_logo: None,
        pinned_slot: slot,
    };
    rsx! {
        Item { slug: "session-packing-planner", name: "SessionPackingPlanner",
            SessionPackingPlanner {
                task_queue: vec![
                    task("t1", "Weekly report", 0.4, None),
                    task("t2", "Code review", 0.3, None),
                    task("t3", "Data export", 0.5, Some(Slot { day_index: 0, session_index: 1 })),
                ],
                day_headers: vec!["Mon".to_string(), "Tue".to_string(), "Wed".to_string()],
                row_headers: vec!["Morning".to_string(), "Afternoon".to_string()],
                initial_mode: PackMode::Manual,
                title: "Session plan",
                on_plan_settled: move |p: Vec<rv2_kit::components::session_packing_planner::PlacedPart>| last.set(format!("settled {} parts", p.len())),
                on_drop: move |(id, _): (String, Vec<rv2_kit::components::session_packing_planner::PlacedPart>)| last.set(format!("drop {id}")),
            }
            div { class: "gl-out", "data-out": "session-packing-planner", "{last()}" }
        }
    }
}

#[component]
fn QuotaLoadSchedulerDemo() -> Element {
    use rv2_kit::components::quota_load_scheduler::{
        CurveEmphasis, DayKind, Deadline, PlannedRun, QuotaLoadScheduler, RunDays,
    };
    let mut last = use_signal(String::new);
    let run = |id: &str, name: &str, start: f64, peak: f64| PlannedRun {
        run_id: id.into(),
        run_name: name.into(),
        start_minute: start,
        peak_tokens_per_min: peak,
        rise_minutes: 10.0,
        tail_half_life_minutes: 30.0,
        run_days: RunDays::Weekdays,
        curve_emphasis: CurveEmphasis::Normal,
    };
    let mut runs = use_signal(|| {
        vec![
            run("r1", "Nightly sync", 540.0, 20_000.0),
            run("r2", "Reports", 780.0, 35_000.0),
        ]
    });
    rsx! {
        Item { slug: "quota-load-scheduler", name: "QuotaLoadScheduler",
            QuotaLoadScheduler {
                run_list: Some(runs()),
                deadline_marker: Some(Deadline { at_minute: 900.0, marker_label: "Quota reset".into() }),
                now_minute: Some(600.0),
                day_kind: DayKind::Weekdays,
                focused_run_id: Some("r1".to_string()),
                on_focus_change: move |id: String| last.set(format!("focus {id}")),
                on_runs_change: move |r: Vec<PlannedRun>| {
                    last.set(format!("runs {}", r.len()));
                    runs.set(r);
                },
                board_title: "Rate limit",
            }
            div { class: "gl-out", "data-out": "quota-load-scheduler", "{last()}" }
        }
    }
}
