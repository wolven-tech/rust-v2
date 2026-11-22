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
    let entry =
        |id: &str, name: &str, note: &str, state: EntryState, button: EntryButton| DigestEntry {
            entry_id: id.into(),
            entry_name: name.into(),
            entry_state: state,
            entry_note: Some(note.into()),
            button_text: button,
        };
    let entries = vec![
        entry(
            "e1",
            "Pricing copy needs a tier",
            "Writer asks whether to name the Team plan",
            EntryState::AwaitingInput,
            EntryButton::Label("Answer".into()),
        ),
        entry(
            "e2",
            "Invoice run waits on approval",
            "Finance approval for 14 invoices",
            EntryState::AwaitingInput,
            EntryButton::Default,
        ),
        entry(
            "e3",
            "Drive sync failed",
            "Access token expired 3 hours ago",
            EntryState::Failed,
            EntryButton::Label("Retry".into()),
        ),
        entry(
            "e4",
            "S3 export failed",
            "Bucket quota reached at 04:12",
            EntryState::Failed,
            EntryButton::Default,
        ),
        entry(
            "e5",
            "Weekly recap sent",
            "Delivered to 12 recipients",
            EntryState::Completed,
            EntryButton::Default,
        ),
        entry(
            "e6",
            "Dependency bump merged",
            "Three tests added, all green",
            EntryState::Completed,
            EntryButton::Label("View".into()),
        ),
    ];
    rsx! {
        Item { slug: "absence-digest", name: "AbsenceDigest",
            AbsenceDigest {
                away_since: (NOW_MS as i64) / 1000 - 8 * 3600 - 1200,
                digest_entries: entries,
                current_time: gallery_now() as i64 / 1000,
                time_format: TimeFormat::Elapsed,
                row_limit: 5,
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
        Change, ChangeLogTimeline, ChangeWriter, KindStyle, Timestamp,
    };
    use std::collections::BTreeMap;
    let mut last = use_signal(String::new);
    let now = gallery_now() as i64 / 1000;
    let writer = |name: &str, agent: bool| ChangeWriter {
        writer_name: name.into(),
        is_agent: agent,
        lane_color: None,
        writer_mark: None,
    };
    let change = |id: &str, msg: &str, who: ChangeWriter, ago: i64, kind: &str| Change {
        change_id: id.into(),
        change_message: msg.into(),
        change_author: who,
        co_author: None,
        changed_at: Some(Timestamp::Epoch(now - ago)),
        change_kind: Some(kind.into()),
        file_count: None,
        lines_added: None,
        lines_removed: None,
        change_body: None,
        merged_changes: vec![],
        in_progress: false,
    };
    let sized = |mut c: Change, files: u32, added: u32, removed: u32| {
        c.file_count = Some(files);
        c.lines_added = Some(added);
        c.lines_removed = Some(removed);
        c
    };
    let mut writing = change(
        "c1",
        "Wire `retry.rs` into the sync loop",
        writer("Drafting agent", true),
        30,
        "new_capability",
    );
    writing.in_progress = true;
    let mut schema = sized(
        change(
            "c2",
            "Add `cursor.toml` schema",
            writer("Ines", false),
            240,
            "new_capability",
        ),
        4,
        88,
        3,
    );
    schema.co_author = Some(writer("Drafting agent", true));
    schema.change_body = Some("Per-branch cursor retention and the resume position.".into());
    let mut merge = change(
        "m1",
        "`sync-cursor/retry`",
        writer("Marek", false),
        900,
        "merge",
    );
    merge.merged_changes = vec![
        sized(
            change(
                "m1a",
                "Spread retries with jitter",
                writer("Marek", false),
                1000,
                "correction",
            ),
            1,
            6,
            2,
        ),
        sized(
            change(
                "m1b",
                "Cap total retry time",
                writer("Ines", false),
                1100,
                "correction",
            ),
            1,
            9,
            1,
        ),
        sized(
            change(
                "m1c",
                "Log each attempt",
                writer("Drafting agent", true),
                1200,
                "documentation",
            ),
            2,
            40,
            0,
        ),
    ];
    let changes = vec![
        writing,
        schema,
        merge,
        sized(
            change(
                "c3",
                "Pin `build.lock` in CI",
                writer("Marek", false),
                3600,
                "housekeeping",
            ),
            1,
            5,
            1,
        ),
        sized(
            change(
                "c4",
                "Draft the cursor notes",
                writer("Ines", false),
                7200,
                "documentation",
            ),
            3,
            120,
            0,
        ),
        sized(
            change(
                "c5",
                "Fix `sync.rs` race on resume",
                writer("Marek", false),
                18_000,
                "correction",
            ),
            2,
            9,
            4,
        ),
        sized(
            change(
                "c6",
                "Start `sync-cursor` scaffold",
                writer("Ines", false),
                172_800,
                "new_capability",
            ),
            7,
            210,
            0,
        ),
    ];
    let configured = |color: &str| KindStyle {
        label: None,
        color: Some(color.into()),
    };
    let mut kinds = BTreeMap::new();
    kinds.insert("new_capability".to_string(), configured("var(--kit-id-1)"));
    kinds.insert("correction".to_string(), configured("var(--kit-series-3)"));
    kinds.insert("documentation".to_string(), configured("var(--kit-id-3)"));
    let short_history = vec![
        sized(
            change(
                "s1",
                "Pin `Cargo.lock` in CI",
                writer("Ben", false),
                3600,
                "housekeeping",
            ),
            1,
            5,
            1,
        ),
        change(
            "s2",
            "Start `ledger-sync` scaffold",
            writer("Ada", false),
            172_800,
            "new_capability",
        ),
    ];
    let mut open_id = use_signal(|| Option::<String>::None);
    let mut expanded = use_signal(|| vec!["m1".to_string()]);
    rsx! {
        Item { slug: "change-log-timeline", name: "ChangeLogTimeline",
            ChangeLogTimeline {
                change_list: changes,
                current_branch: "sync-cursor",
                now_secs: now,
                release_tag: Some("rc 3".to_string()),
                has_older: true,
                kind_overrides: kinds,
                open_change_id: open_id(),
                on_open_change: move |id: Option<String>| {
                    last.set(format!("open {id:?}"));
                    open_id.set(id);
                },
                expanded_merge_ids: Some(expanded()),
                on_merge_expand_change: move |ids: Vec<String>| expanded.set(ids),
            }
            div { class: "gl-out", "data-out": "change-log-timeline", "{last()}" }
        }
        Item { slug: "change-log-timeline-branches", name: "ChangeLogTimeline",
            ChangeLogTimeline {
                change_list: short_history,
                branch_names: vec!["ledger-sync".to_string(), "main".to_string()],
                current_branch: "main",
                now_secs: now,
            }
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
                change_events: vec![
                    ev(1.0, "onboarding.md", 12),
                    ev(2.0, "style.md", 4),
                    ev(3.0, "onboarding.md", 3),
                    ev(4.5, "tone.md", 7),
                    ev(6.0, "onboarding.md", 9),
                    ev(8.0, "faq.md", 15),
                ],
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
        model_label: None,
        prompt_tokens: None,
        completion_tokens: None,
        node_cost: None,
        input_excerpt: None,
        output_excerpt: None,
        failure_text: None,
        children,
    };
    let priced = |mut n: TraceNode, cost: f64| {
        n.node_cost = Some(cost);
        n
    };
    let metered = |mut n: TraceNode, prompt: f64, completion: f64| {
        n.model_label = Some("model A".into());
        n.prompt_tokens = Some(prompt);
        n.completion_tokens = Some(completion);
        n
    };
    let outline = metered(
        priced(
            node(
                "outline",
                "Outline sections",
                NodeType::Model,
                NodeStatus::Done,
                0.0,
                Some(1.5),
                vec![],
            ),
            0.05,
        ),
        1_400.0,
        300.0,
    );
    let lookup = priced(
        node(
            "lookup",
            "lookup_figures",
            NodeType::Tool,
            NodeStatus::Done,
            1.5,
            Some(2.5),
            vec![],
        ),
        0.02,
    );
    let verify = metered(
        priced(
            node(
                "verify",
                "Verify claims",
                NodeType::Model,
                NodeStatus::Running,
                2.5,
                None,
                vec![],
            ),
            0.06,
        ),
        800.0,
        300.0,
    );
    let fact = node(
        "fact",
        "Fact checker",
        NodeType::Subagent,
        NodeStatus::Running,
        1.5,
        None,
        vec![lookup, verify],
    );
    let fetch = priced(
        node(
            "fetch",
            "Fetch last week's tickets",
            NodeType::Tool,
            NodeStatus::Done,
            2.0,
            Some(3.0),
            vec![],
        ),
        0.01,
    );
    let write = metered(
        priced(
            node(
                "write",
                "Write the draft",
                NodeType::Model,
                NodeStatus::Done,
                3.0,
                Some(8.0),
                vec![],
            ),
            0.17,
        ),
        2_600.0,
        800.0,
    );
    let trace = node(
        "root",
        "Draft the weekly digest",
        NodeType::Agent,
        NodeStatus::Running,
        0.0,
        None,
        vec![outline, fact, fetch, write],
    );
    rsx! {
        Item { slug: "run-timing-waterfall", name: "RunTimingWaterfall",
            RunTimingWaterfall {
                trace_tree: trace,
                clock_point: Some(9.0),
                heading: "Digest run",
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
                    step("gather", "Gather sources", 4.0, vec![], Some(0.0), Some(3.0)),
                    step("draft", "Draft outline", 4.0, vec!["gather".to_string()], Some(3.0), Some(10.0)),
                    step("write_a", "Write section A", 6.0, vec!["draft".to_string()], Some(10.0), None),
                    step("write_b", "Write section B", 6.0, vec!["write_a".to_string()], None, None),
                    step("review", "Review", 2.0, vec!["write_b".to_string()], None, None),
                    step("publish", "Publish", 2.0, vec!["review".to_string()], None, None),
                ],
                clock_point: 13.0,
                run_purpose: Some("Draft report".to_string()),
                format_time: Some(Callback::new(|m: f64| format!("{m:.0} min"))),
                time_unit: "minute",
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
    let task = |id: &str, title: &str, agent: &str, share: f64, slot: Option<Slot>| ScheduledTask {
        task_id: id.into(),
        task_title: title.into(),
        usage_share: share,
        agent_name: Some(agent.into()),
        task_detail: None,
        agent_logo: None,
        pinned_slot: slot,
    };
    rsx! {
        Item { slug: "session-packing-planner", name: "SessionPackingPlanner",
            SessionPackingPlanner {
                task_queue: vec![
                    task("t1", "Model migration", "Planner", 0.9, None),
                    task("t2", "Weekly report", "Writer", 0.9, None),
                    task("t3", "Re-embed archive", "Indexer", 1.9, None),
                    task("t4", "Code review", "Reviewer", 0.9, None),
                    task("t5", "Invoice audit", "Auditor", 0.9, None),
                    task("t6", "Weekly digest", "Writer", 0.9, None),
                    task("t7", "Data sync", "Exporter", 0.9, None),
                    task("t8", "Full reindex", "Indexer", 1.8, None),
                    task("t9", "Quarterly audit", "Auditor", 1.4, None),
                    task("t10", "Archive restore", "Indexer", 1.6, None),
                    task("t11", "Data export", "Exporter", 0.9, Some(Slot { day_index: 3, session_index: 0 })),
                ],
                day_headers: vec!["Mon".to_string(), "Tue".to_string(), "Wed".to_string(), "Thu".to_string()],
                row_headers: vec!["07:00".to_string(), "12:00".to_string(), "17:00".to_string()],
                current_slot: Some(Slot { day_index: 0, session_index: 1 }),
                now_fraction: 0.35,
                smallest_fragment: 0.2,
                initial_mode: PackMode::Auto,
                auto_delay: 0,
                note: "Auto slotting has finished.",
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
            run("ledger", "Ledger sync", 540.0, 40_000.0),
            run("index", "Index rebuild", 780.0, 90_000.0),
            run("report", "Report writer", 930.0, 50_000.0),
            run("invoice", "Invoice render", 1_140.0, 30_000.0),
        ]
    });
    let measured: Vec<f64> = (0..271)
        .map(|i| 1_500.0 + 1_200.0 * ((i as f64) / 25.0).sin().abs())
        .collect();
    let mut ceiling = use_signal(|| 100_000.0_f64);
    let deadline = Deadline {
        at_minute: 1_380.0,
        marker_label: "Nightly close".into(),
    };
    rsx! {
        Item { slug: "quota-load-scheduler", name: "QuotaLoadScheduler",
            div { class: "gl-row", role: "group", "aria-label": "Ceiling",
                for (value, label) in [(100_000.0, "Ceiling 100k"), (60_000.0, "Ceiling 60k")] {
                    button {
                        r#type: "button",
                        "aria-pressed": if ceiling() == value { "true" } else { "false" },
                        onclick: move |_| ceiling.set(value),
                        "{label}"
                    }
                }
            }
            QuotaLoadScheduler {
                run_list: Some(runs()),
                provider_ceiling_per_min: ceiling(),
                safety_ratio: 0.8,
                window_start_hour: 6.0,
                deadline_marker: Some(deadline),
                now_minute: Some(630.0),
                measured_load: Some(measured),
                day_kind: DayKind::Weekdays,
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
