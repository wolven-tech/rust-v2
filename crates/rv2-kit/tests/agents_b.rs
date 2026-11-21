use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::agents::baton_pass_log::BatonPassLog;
use rv2_kit::agents::co_editing_presence::pin_window_top;
use rv2_kit::agents::overnight_run_digest::{DigestEvent, DigestLane, OvernightRunDigest};
use rv2_kit::agents::request_routing_map::{
    Channel, HandlerNode, RequestRoutingMap, RequestRow, RequestState, RouterNode,
};
use rv2_kit::agents::reversible_change_log::{
    ChangeRow, ReversibleChangeLog, RowState, cascade_keys, control_name, newest_first,
    relative_time, row_states,
};
use rv2_kit::agents::run_checkpoint_rail::{
    CheckpointEntry, RunCheckpointRail, append_restart, attempt_numbers, move_focus, on_path,
};
use rv2_kit::agents::run_timeline_rows::{
    RowKind, RowSpec, RunTimelineRows, Span, SpanMode, TickUnit, axis_length, describe_span,
    next_step, span_bounds, tick_step, tick_unit,
};
use rv2_kit::agents::single_agent_stage::{
    DetailRow, FileItem, Requester, SingleAgentStage, ToolGrant, WorkStep, Worker, WorkerStage,
    info_rows, newest_steps, split_files, split_tools, step_text, tool_status,
};
use rv2_kit::agents::single_run_status_card::{
    RunAction, RunPhase, SingleRunStatusCard, StepKind, StepRow, StepState, WorkerIdentity,
    elapsed_ms, footer_actions, format_mm_ss, format_usd, initial_of, spoken_duration,
    step_duration_ms,
};
use rv2_kit::agents::step_progress_list::{
    ItemState, ProgressItem, StepProgressList, SubItem, SubState, count_line, default_open_keys,
    fold_window, item_name,
};
use rv2_kit::agents::sub_run_tree::{
    Node, NodePhase, SubRunTree, elapsed_ms as node_elapsed, subtree_spend, summarize, visible_keys,
};
use rv2_kit::agents::tool_call_network::{
    Condition, Entity, Exchange, ExchangeResult, NodeRole, ToolCallNetwork, attention_counts,
    exchange_phrase, exchanges_last_minute, layout, link_width, neighbourhood, text_summary,
};
use rv2_kit::agents::work_queue_flow::{
    JobKind, JobRow, JobState, Priority, SlotRow, WorkQueueFlow, depth_at, line_order, next_index,
    p95_wait_ms, step_slots, throughput_per_min,
};
use rv2_kit::agents::worker_fleet_panel::{
    FleetState, LeadAgent, SubAgent, WorkerFleetPanel, done_share, fleet_totals, next_lead,
    response_actions, urgent_phrase,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn change(key: &str, at: u64, built_on: &[&str]) -> ChangeRow {
    let mut row = ChangeRow::new(key, "Edit", "Notion", at);
    row.built_on_keys = built_on.iter().map(|k| k.to_string()).collect();
    row
}

#[test]
fn cascade_starts_with_the_row_and_follows_reversible_dependents() {
    let rows = vec![
        change("a", 1, &[]),
        change("b", 2, &["a"]),
        change("c", 3, &["b"]),
    ];
    assert_eq!(cascade_keys(&rows, "a"), vec!["a", "b", "c"]);
}

#[test]
fn non_reversible_dependent_is_excluded_from_cascade_and_blocked() {
    let mut frozen = change("b", 2, &["a"]);
    frozen.reversible = false;
    let rows = vec![change("a", 1, &[]), frozen];
    assert_eq!(cascade_keys(&rows, "a"), vec!["a"]);
    let states = row_states(&rows, &[]);
    assert_eq!(states, vec![RowState::Applied, RowState::NotReversible]);
    let mut child = change("c", 3, &["b"]);
    child.reversible = true;
    let rows = vec![change("a", 1, &[]), change("b", 2, &["a"])];
    let mut blocked_rows = rows.clone();
    blocked_rows[0].reversible = false;
    let states = row_states(&blocked_rows, &["a".to_string()]);
    assert_eq!(states[1], RowState::Blocked);
}

#[test]
fn reverted_row_built_on_reverted_row_is_also_reverted() {
    let rows = vec![change("a", 1, &[]), change("b", 2, &["a"])];
    let states = row_states(&rows, &["a".to_string(), "b".to_string()]);
    assert_eq!(states, vec![RowState::Reverted, RowState::AlsoReverted]);
}

#[test]
fn control_name_states_dependent_count() {
    assert_eq!(
        control_name("Edit", "page", 2, false),
        "Revert: Edit page, 2 dependents"
    );
    assert_eq!(control_name("Edit", "", 0, true), "Reapply: Edit");
}

#[test]
fn newest_first_orders_by_time_and_relative_time_reads_plainly() {
    let rows = vec![change("old", 1_000, &[]), change("new", 9_000, &[])];
    assert_eq!(newest_first(&rows), vec![1, 0]);
    assert_eq!(relative_time(120_000, 0), "2m ago");
    assert_eq!(relative_time(5_000, 9_000), "just now");
}

#[test]
fn change_log_renders_empty_text_and_cannot_revert_flag() {
    let html = render(|| rsx! { ReversibleChangeLog { change_rows: Vec::new() } });
    assert!(html.contains("No changes yet"));
    let html = render(|| {
        let mut row = ChangeRow::new("x", "Delete", "Drive", 1);
        row.reversible = false;
        rsx! { ReversibleChangeLog { change_rows: vec![row] } }
    });
    assert!(html.contains("Cannot revert"));
    assert!(html.contains("data-state=\"not_reversible\""));
}

#[test]
fn restart_from_a_row_opens_a_second_attempt_and_keeps_the_old_one() {
    let base = vec![
        CheckpointEntry::new("1", "plan"),
        CheckpointEntry::new("2", "build"),
        CheckpointEntry::new("3", "test"),
    ];
    let next = append_restart(&base, "1", "4", "plan again");
    assert_eq!(next.len(), 4);
    assert_eq!(next[3].follows_id.as_deref(), Some("1"));
    assert_eq!(attempt_numbers(&next), vec![1, 1, 1, 2]);
    let path = on_path(&next, 3);
    assert_eq!(path, vec![true, false, false, true]);
}

#[test]
fn checkpoint_focus_moves_by_key() {
    assert_eq!(move_focus(5, 2, "ArrowDown"), Some(3));
    assert_eq!(move_focus(5, 0, "ArrowUp"), Some(0));
    assert_eq!(move_focus(5, 2, "End"), Some(4));
    assert_eq!(move_focus(5, 2, "Home"), Some(0));
    assert_eq!(move_focus(5, 2, "a"), None);
}

#[test]
fn checkpoint_rail_marks_current_and_renders_failed_text() {
    let html = render(|| {
        let mut bad = CheckpointEntry::new("b", "tests");
        bad.run_state = rv2_kit::agents::run_checkpoint_rail::RunState::Failed;
        let entries = vec![CheckpointEntry::new("a", "plan"), bad];
        rsx! { RunCheckpointRail { entries: entries, current_id: "b".to_string() } }
    });
    assert!(html.contains("aria-current=\"step\""));
    assert!(html.contains("Failed"));
    assert!(html.contains("aria-live=\"polite\""));
}

#[test]
fn checkpoint_elapsed_is_clock_minus_start() {
    let html = render(|| {
        let mut entry = CheckpointEntry::new("a", "plan");
        entry.started_ms = Some(1_000);
        rsx! { RunCheckpointRail { entries: vec![entry], clock_ms: 61_000u64 } }
    });
    assert!(html.contains("1:00"));
    assert!(!html.contains("29999990"));
}

#[test]
fn spoken_durations_use_words_and_singular_units() {
    assert_eq!(spoken_duration(125_000), "2 minutes 5 seconds");
    assert_eq!(spoken_duration(61_000), "1 minute 1 second");
    assert_eq!(spoken_duration(0), "0 minutes 0 seconds");
}

#[test]
fn exchange_counts_read_as_singular_or_plural() {
    assert_eq!(exchange_phrase(1), "1 exchange");
    assert_eq!(exchange_phrase(0), "0 exchanges");
    assert_eq!(exchange_phrase(3), "3 exchanges");
}

fn span(row: &str, mode: SpanMode, begin: f64, end: Option<f64>) -> Span {
    Span {
        span_key: None,
        row_key: row.to_string(),
        mode,
        begin_s: begin,
        end_s: end,
        tool_name: None,
        caption: None,
    }
}

#[test]
fn live_span_reaches_the_clock_and_bad_spans_are_omitted() {
    let live = span("r", SpanMode::Reasoning, 10.0, None);
    assert_eq!(span_bounds(&live, Some(42.0)), Some((10.0, 42.0)));
    assert_eq!(span_bounds(&live, None), None);
    let backwards = span("r", SpanMode::Tool, 10.0, Some(5.0));
    assert_eq!(span_bounds(&backwards, None), None);
}

#[test]
fn axis_units_switch_with_range() {
    assert_eq!(tick_unit(60.0), TickUnit::Seconds);
    assert_eq!(tick_unit(600.0), TickUnit::Minutes);
    assert_eq!(tick_unit(9000.0), TickUnit::Hours);
    assert!(tick_step(100.0) >= 10.0);
    let spans = vec![span("r", SpanMode::Reasoning, 0.0, Some(50.0))];
    assert!(axis_length(&spans, None, None) > 50.0);
    assert_eq!(axis_length(&spans, None, Some(90.0)), 90.0);
}

#[test]
fn arrows_walk_within_a_row_and_nearest_in_the_row_below() {
    let positions = vec![(0, 0.0), (0, 5.0), (1, 4.0)];
    assert_eq!(next_step(&positions, 0, "ArrowRight"), Some(1));
    assert_eq!(next_step(&positions, 1, "ArrowLeft"), Some(0));
    assert_eq!(next_step(&positions, 0, "ArrowDown"), Some(2));
    assert_eq!(next_step(&positions, 2, "ArrowUp"), Some(1));
}

#[test]
fn step_description_reads_who_what_when_and_how_long() {
    let s = span("r", SpanMode::Reasoning, 42.0, Some(45.0));
    assert_eq!(
        describe_span("Reviewer", &s, 45.0),
        "Reviewer, reasoning, from 0:42 for 3.0 s"
    );
}

#[test]
fn timeline_shows_waiting_on_you_in_words() {
    let html = render(|| {
        let rows = vec![RowSpec {
            row_key: "me".to_string(),
            display_name: "You".to_string(),
            kind: RowKind::Person,
            tint_key: None,
            mark_source: None,
        }];
        let spans = vec![span("me", SpanMode::Waiting, 1.0, None)];
        rsx! { RunTimelineRows { run_heading: "Run 1", rows: rows, spans: spans, clock_s: 5.0 } }
    });
    assert!(html.contains("Waiting on you"));
    assert!(html.contains("data-running=\"true\""));
}

#[test]
fn elapsed_excludes_paused_time_and_format_helpers_read_plainly() {
    assert_eq!(elapsed_ms(Some(1_000), None, 2_000, 10_000), 7_000);
    assert_eq!(elapsed_ms(None, None, 0, 10_000), 0);
    assert_eq!(format_mm_ss(75_000), "1:15");
    assert_eq!(format_usd(1.5), "$1.50");
    assert_eq!(initial_of("reviewer"), "R");
}

#[test]
fn footer_offers_grant_and_refuse_only_when_awaiting() {
    assert_eq!(
        footer_actions(RunPhase::AwaitingGrant),
        &[RunAction::Grant, RunAction::Refuse]
    );
    assert!(footer_actions(RunPhase::Complete).is_empty());
    let step = StepRow {
        step_key: "s".to_string(),
        step_label: "build".to_string(),
        step_kind: Some(StepKind::Execute),
        step_state: StepState::InProgress,
        started_at_ms: Some(100),
        elapsed_ms: None,
        tool_mark: None,
        tool_label: None,
        detail_text: None,
        target_text: None,
    };
    assert_eq!(step_duration_ms(&step, 400), 300);
}

#[test]
fn status_card_renders_phase_and_footer() {
    let html = render(|| {
        let identity = WorkerIdentity {
            display_label: "Builder".to_string(),
            role_label: None,
            mark: None,
        };
        rsx! { SingleRunStatusCard { run_phase: RunPhase::Held, worker_identity: identity } }
    });
    assert!(html.contains("data-phase=\"held\""));
    assert!(!html.contains("Grant"));
}

#[test]
fn count_line_and_fold_window_follow_the_spec() {
    let item = |key: &str, state: ItemState| ProgressItem {
        item_key: key.to_string(),
        title_text: key.to_string(),
        item_state: state,
        took_ms: None,
        began_at_ms: None,
        live_note: None,
        needs_approval: false,
        tool_badge: None,
        approval_request: None,
        sub_items: Vec::new(),
    };
    let items = vec![
        item("a", ItemState::Finished),
        item("b", ItemState::InProgress),
        item("c", ItemState::Queued),
    ];
    assert_eq!(count_line(&items), "1 of 3 steps finished");
    assert_eq!(default_open_keys(&items), vec!["b".to_string()]);
    assert_eq!(fold_window(4, Some(0)), (0, 4));
    assert_eq!(fold_window(10, Some(5)), (3, 8));
    assert_eq!(fold_window(10, Some(9)), (5, 10));
    assert_eq!(item_name("Deploy", "Dropped", ""), "Deploy, Dropped");
}

#[test]
fn progress_list_renders_current_step_and_awaiting_wording() {
    let html = render(|| {
        let sub = SubItem {
            sub_key: "s1".to_string(),
            text: "compile".to_string(),
            sub_state: Some(SubState::InProgress),
            fact_text: Some("3 files".to_string()),
        };
        let items = vec![ProgressItem {
            item_key: "build".to_string(),
            title_text: "Build".to_string(),
            item_state: ItemState::InProgress,
            took_ms: None,
            began_at_ms: None,
            live_note: Some("compiling".to_string()),
            needs_approval: false,
            tool_badge: None,
            approval_request: None,
            sub_items: vec![sub],
        }];
        rsx! { StepProgressList { list_heading: "Task".to_string(), progress_items: items } }
    });
    assert!(html.contains("aria-current=\"step\""));
    assert!(html.contains("0 of 1 steps finished"));
}

fn unit(key: &str, phase: NodePhase, spend: f64, branches: Vec<Node>) -> Node {
    Node {
        node_key: key.to_string(),
        caption: key.to_string(),
        brief: String::new(),
        model_tag: None,
        tool_marks: Vec::new(),
        phase,
        spend,
        began_at_ms: Some(0),
        ended_at_ms: None,
        needs_text: None,
        fault_text: None,
        branches,
    }
}

#[test]
fn folded_branch_hides_children_and_totals_match() {
    let root = unit(
        "root",
        NodePhase::Active,
        1.0,
        vec![
            unit("a", NodePhase::Complete, 2.0, vec![]),
            unit("b", NodePhase::Errored, 3.0, vec![]),
        ],
    );
    assert_eq!(subtree_spend(&root), 6.0);
    let summary = summarize(Some(&root));
    assert_eq!(summary.units, 3);
    assert_eq!(summary.errored, 1);
    assert_eq!(visible_keys(&root, &["root".to_string()]), vec!["root"]);
    assert_eq!(visible_keys(&root, &[]), vec!["root", "a", "b"]);
}

#[test]
fn pending_unit_has_no_elapsed_and_active_counts_to_clock() {
    let mut active = unit("x", NodePhase::Active, 0.0, vec![]);
    active.began_at_ms = Some(1_000);
    assert_eq!(node_elapsed(&active, 4_000), Some(3_000));
    let pending = unit("p", NodePhase::Pending, 0.0, vec![]);
    assert_eq!(node_elapsed(&pending, 4_000), None);
}

#[test]
fn sub_run_tree_renders_wait_badge_and_summary() {
    let html = render(|| {
        let root = unit(
            "root",
            NodePhase::AwaitingInput,
            1.0,
            vec![unit("child", NodePhase::Complete, 2.0, vec![])],
        );
        rsx! { SubRunTree { root_node: root, heading: "Job" } }
    });
    assert!(html.contains("Needs you"));
    assert!(html.contains("1 awaiting input"));
}

fn entity(id: &str, role: NodeRole, centre: bool, condition: Condition) -> Entity {
    Entity {
        id: id.to_string(),
        name: id.to_string(),
        node_role: role,
        compact_label: None,
        glyph: None,
        summary_line: None,
        condition,
        activity: None,
        spend: None,
        is_centre: centre,
    }
}

#[test]
fn centre_agent_sits_in_the_middle_and_tools_ring_outside() {
    let entities = vec![
        entity("planner", NodeRole::Agent, true, Condition::Working),
        entity("coder", NodeRole::Agent, false, Condition::Idle),
        entity("search", NodeRole::Tool, false, Condition::Idle),
    ];
    let placed = layout(&entities, &[("planner".to_string(), "search".to_string())]);
    let centre = placed.iter().find(|(id, _, _)| id == "planner").unwrap();
    assert_eq!((centre.1, centre.2), (500.0, 500.0));
    let tool = placed.iter().find(|(id, _, _)| id == "search").unwrap();
    let dist = ((tool.1 - 500.0).powi(2) + (tool.2 - 500.0).powi(2)).sqrt();
    assert!((dist - 430.0).abs() < 1e-6);
}

#[test]
fn exchange_counts_neighbourhood_and_text_summary() {
    let exchanges = vec![Exchange {
        id: "e1".to_string(),
        from_id: "planner".to_string(),
        to_id: "search".to_string(),
        sent_at_ms: 90_000,
        result: ExchangeResult::Ok,
        caption: None,
        latency_ms: Some(1200),
    }];
    assert_eq!(exchanges_last_minute(&exchanges, "search", 120_000), 1);
    assert_eq!(exchanges_last_minute(&exchanges, "search", 400_000), 0);
    assert_eq!(
        neighbourhood(&[], &exchanges, "planner"),
        vec!["search".to_string()]
    );
    assert_eq!(link_width(0), 1.0);
    let entities = vec![entity("search", NodeRole::Tool, false, Condition::Waiting)];
    assert_eq!(
        text_summary(&entities, &exchanges, 120_000)[0],
        "search: Needs you, 1 exchange in the last minute"
    );
    assert_eq!(attention_counts(&entities), (1, 0));
}

#[test]
fn network_renders_needs_you_text_and_a_hidden_list() {
    let html = render(|| {
        let entities = vec![
            entity("planner", NodeRole::Agent, true, Condition::Working),
            entity("person", NodeRole::User, false, Condition::Waiting),
        ];
        rsx! { ToolCallNetwork { entities: entities } }
    });
    assert!(html.contains("Needs you"));
    assert!(html.contains("person: Needs you, 0 exchanges in the last minute"));
}

fn job(
    key: &str,
    state: JobState,
    enqueued: u64,
    started: Option<u64>,
    priority: Priority,
) -> JobRow {
    JobRow {
        job_key: key.to_string(),
        job_label: key.to_string(),
        job_kind: JobKind::ModelCall,
        job_state: state,
        enqueued_at_ms: enqueued,
        started_at_ms: started,
        ended_at_ms: None,
        slot_key: None,
        lane_priority: priority,
        progress_fraction: None,
        expected_run_ms: None,
        attempt_number: 1,
        max_attempts: None,
        detail_text: None,
        job_mark: None,
        error_text: None,
    }
}

#[test]
fn urgent_job_moves_to_the_front_of_the_line() {
    let jobs = vec![
        job("early", JobState::Enqueued, 1, None, Priority::Standard),
        job("urgent", JobState::Enqueued, 5, None, Priority::Urgent),
    ];
    assert_eq!(line_order(&jobs), vec![1, 0]);
}

#[test]
fn queue_metrics_depth_and_slot_steps() {
    let jobs = vec![
        job("a", JobState::Enqueued, 0, Some(10), Priority::Standard),
        job("b", JobState::Enqueued, 0, Some(40), Priority::Standard),
    ];
    assert_eq!(depth_at(&jobs, 5), 2);
    assert_eq!(depth_at(&jobs, 20), 1);
    assert_eq!(p95_wait_ms(&jobs), Some(40));
    assert_eq!(throughput_per_min(&jobs, 60_000), 0);
    assert_eq!(step_slots(8, 1, 1, 8), 8);
    assert_eq!(step_slots(1, -1, 1, 8), 1);
    assert_eq!(next_index(3, 2, "ArrowDown"), Some(2));
}

#[test]
fn queue_renders_stats_and_chart_summary() {
    let html = render(|| {
        let slots = vec![SlotRow {
            slot_key: "s1".to_string(),
            slot_label: "Slot 1".to_string(),
            slot_detail: None,
        }];
        let jobs = vec![job("q", JobState::Enqueued, 0, None, Priority::Standard)];
        rsx! { WorkQueueFlow { job_rows: jobs, slot_rows: slots, clock_ms: 1_000u64 } }
    });
    assert!(html.contains("Backlog 1"));
    assert!(html.contains("1 slots"));
}

fn sub(id: &str, state: FleetState, spend: f64) -> SubAgent {
    SubAgent {
        id: id.to_string(),
        name: id.to_string(),
        assignment: String::new(),
        model_label: None,
        state,
        work_items: Vec::new(),
        spend: Some(spend),
        question: None,
        failure_text: None,
    }
}

fn lead(id: &str, state: FleetState, subs: Vec<SubAgent>) -> LeadAgent {
    LeadAgent {
        id: id.to_string(),
        name: id.to_string(),
        assignment: String::new(),
        model_label: None,
        picture: None,
        tool_marks: Vec::new(),
        state,
        spend: Some(1.0),
        started_at_ms: None,
        ended_at_ms: None,
        sub_agents: subs,
    }
}

#[test]
fn fleet_totals_count_each_spend_once() {
    let fleet = vec![lead(
        "l",
        FleetState::Running,
        vec![
            sub("a", FleetState::Waiting, 2.0),
            sub("b", FleetState::Failed, 3.0),
        ],
    )];
    let totals = fleet_totals(&fleet);
    assert_eq!(totals.agents, 1);
    assert_eq!(totals.sub_agents, 2);
    assert_eq!(totals.needs_you, 1);
    assert_eq!(totals.spend, 6.0);
}

#[test]
fn urgent_phrase_prefers_needs_you_then_failed() {
    let subs = vec![
        sub("a", FleetState::Failed, 0.0),
        sub("b", FleetState::Waiting, 0.0),
    ];
    assert_eq!(urgent_phrase(&subs), Some("Needs you"));
    assert_eq!(done_share(&subs), (0, 2));
    assert_eq!(response_actions(FleetState::Failed).len(), 2);
    assert!(response_actions(FleetState::Done).is_empty());
    assert_eq!(next_lead(3, 0, "End"), Some(2));
}

#[test]
fn fleet_panel_renders_lead_state_and_totals() {
    let html = render(|| {
        let fleet = vec![lead(
            "planner",
            FleetState::Waiting,
            vec![sub("coder", FleetState::Done, 0.5)],
        )];
        rsx! { WorkerFleetPanel { lead_agents: fleet } }
    });
    assert!(html.contains("planner, Needs you"));
    assert!(html.contains("aria-expanded=\"false\""));
}

#[test]
fn stage_overflow_and_status_rules() {
    let grants: Vec<ToolGrant> = (0..5)
        .map(|i| ToolGrant {
            tool_key: format!("t{i}"),
            mark_source: None,
            display_name: format!("Tool {i}"),
            grant_text: String::new(),
            previous_action: None,
            previous_when: None,
            live_action: None,
            detail_rows: Vec::new(),
        })
        .collect();
    let (shown, more) = split_tools(&grants);
    assert_eq!((shown.len(), more), (4, 1));
    assert_eq!(
        tool_status(&grants[0]),
        ("Ready", "Nothing yet".to_string())
    );
    let files: Vec<FileItem> = (0..6)
        .map(|i| FileItem {
            file_key: format!("f{i}"),
            display_name: format!("f{i}.md"),
            mark_source: None,
            meta_text: None,
            detail_rows: Vec::new(),
        })
        .collect();
    assert_eq!(split_files(&files).1, 2);
}

#[test]
fn stage_work_area_keeps_three_newest_and_words_empty_steps() {
    let log: Vec<WorkStep> = (0..4)
        .map(|i| WorkStep {
            step_key: format!("s{i}"),
            text: String::new(),
            route_key: "files".to_string(),
            in_progress: i == 3,
        })
        .collect();
    let newest = newest_steps(&log);
    assert_eq!(newest.len(), 3);
    assert_eq!(newest[0].step_key, "s3");
    assert_eq!(step_text(&newest[0]), "Working…");
    assert_eq!(step_text(&newest[1]), "Done");
}

#[test]
fn info_card_lists_tool_action_for_this_request() {
    let worker = Worker {
        worker_key: "w".to_string(),
        display_name: "Coder".to_string(),
        subtitle_text: None,
        detail_rows: vec![DetailRow {
            field_label: "Model".to_string(),
            field_value: "x".to_string(),
        }],
    };
    let grant = ToolGrant {
        tool_key: "search".to_string(),
        mark_source: None,
        display_name: "Search".to_string(),
        grant_text: String::new(),
        previous_action: Some("query docs".to_string()),
        previous_when: None,
        live_action: None,
        detail_rows: Vec::new(),
    };
    let rows = info_rows("tool:search", &worker, &[grant], &[]).unwrap();
    assert_eq!(rows.last().unwrap().field_value, "query docs");
    assert!(info_rows("file:none", &worker, &[], &[]).is_none());
}

#[test]
fn stage_renders_waiting_state_and_requester_initials() {
    let html = render(|| {
        let requester = Requester {
            display_name: "Ada".to_string(),
            message_text: "Summarise".to_string(),
            photo_url: None,
        };
        let worker = Worker {
            worker_key: "w".to_string(),
            display_name: "Coder".to_string(),
            subtitle_text: None,
            detail_rows: Vec::new(),
        };
        rsx! { SingleAgentStage { requester: requester, worker: worker, worker_stage: WorkerStage::Waiting } }
    });
    assert!(html.contains("Waiting"));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("Ada"));
}

#[test]
fn stage_draws_one_join_per_tool_card() {
    let html = render(|| {
        let grant = |key: &str| ToolGrant {
            tool_key: key.to_string(),
            mark_source: None,
            display_name: key.to_string(),
            grant_text: String::new(),
            previous_action: None,
            previous_when: None,
            live_action: None,
            detail_rows: Vec::new(),
        };
        let requester = Requester {
            display_name: "Ada".to_string(),
            message_text: "Summarise".to_string(),
            photo_url: None,
        };
        let worker = Worker {
            worker_key: "w".to_string(),
            display_name: "Coder".to_string(),
            subtitle_text: None,
            detail_rows: Vec::new(),
        };
        rsx! {
            SingleAgentStage {
                requester: requester,
                worker: worker,
                worker_stage: WorkerStage::Working,
                tool_grants: vec![grant("search"), grant("mail")],
            }
        }
    });
    assert_eq!(html.matches("class=\"kit-sas__join\"").count(), 2);
}

#[test]
fn pin_window_opens_below_above_or_inside_the_page() {
    assert_eq!(pin_window_top(600.0, 100.0, 120.0), 24.0);
    assert_eq!(pin_window_top(300.0, 250.0, 120.0), -144.0);
    assert_eq!(pin_window_top(168.0, 50.0, 109.0), 9.0);
}

#[test]
fn host_reverted_card_offers_reapply() {
    let html = render(|| {
        let event = DigestEvent {
            event_key: "d3".to_string(),
            happened_at: Some(5_000),
            event_lane: DigestLane::Completed,
            event_title: "Archived threads".to_string(),
            event_detail: None,
            repeat_count: None,
            main_action: None,
            inspect_available: false,
            past_tense_label: None,
            reversible: true,
        };
        rsx! {
            OvernightRunDigest {
                digest_events: vec![event],
                away_since: 0u64,
                now_at: Some(10_000u64),
                reverted_keys: Some(vec!["d3".to_string()]),
            }
        }
    });
    assert!(html.contains("Reapply: Archived threads"), "{html}");
}

#[test]
fn routing_map_draws_a_link_per_request_and_card_handler() {
    let html = render(|| {
        let router = RouterNode {
            node_key: "router".to_string(),
            node_label: "Triage".to_string(),
            model_label: None,
            reading_text: None,
            model_mark: None,
        };
        let handler = HandlerNode {
            handler_key: "billing".to_string(),
            handler_label: "Billing".to_string(),
            model_label: None,
            handler_state: None,
            activity_text: None,
            load_count: None,
            queued_request_keys: Vec::new(),
            is_person: false,
            accent_slot: 1,
            handler_mark: None,
            model_mark: None,
        };
        let row = |key: &str, state: RequestState, assigned: Option<&str>| RequestRow {
            request_key: key.to_string(),
            request_title: format!("Request {key}"),
            sender_label: None,
            channel_kind: Channel::Mail,
            preview_text: None,
            request_state: state,
            assigned_to_key: assigned.map(str::to_string),
            certainty_fraction: 0.5,
            score_map: BTreeMap::new(),
            reason_text: None,
            tag_list: Vec::new(),
        };
        let rows = vec![
            row("r1", RequestState::Sorting, None),
            row("r2", RequestState::NeedsPerson, Some("billing")),
        ];
        rsx! {
            RequestRoutingMap {
                router_node: router,
                handler_nodes: vec![handler],
                request_rows: rows,
            }
        }
    });
    assert_eq!(html.matches("class=\"kit-rrm__link\"").count(), 3);
}

#[test]
fn baton_log_announces_through_a_region_present_from_mount() {
    let html = render(|| {
        rsx! {
            BatonPassLog {
                relay_agents: Vec::new(),
                assignments: Vec::new(),
                transfers: Vec::new(),
            }
        }
    });
    assert!(html.contains("kit-baton__sr"), "{html}");
    assert!(html.contains("role=\"status\""), "{html}");
}
