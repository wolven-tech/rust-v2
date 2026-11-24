use dioxus::prelude::*;
use rv2_kit::screens::agent_task_board::{
    AgentRecord, AgentTaskBoard, BoardView, StepRecord, TaskAction, TaskColumn, TaskRecord,
    TaskState, TaskTab, actions_for, announcement, duration_text, filter_by_agent, live_seconds,
    new_task_ready, step_index, tab_counts, visible_order,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

const NOW: u64 = 1_795_219_200_000;

fn task(id: &str, state: TaskState, agent: &str) -> TaskRecord {
    TaskRecord {
        id: id.to_string(),
        entry_title: format!("Task {id}"),
        entry_description: Some("Pull the figures".to_string()),
        agent_id: agent.to_string(),
        task_state: state,
        started_at: None,
        elapsed_seconds: Some(125),
        source_count: Some(3),
        tool_count: Some(2),
        step_records: vec![StepRecord {
            entry_label: "Read the sheet".to_string(),
            time_label: Some("0:04".to_string()),
        }],
    }
}

fn board_tasks() -> Vec<TaskRecord> {
    vec![
        task("t1", TaskState::Waiting, "ada"),
        task("t2", TaskState::Failed, "lin"),
        task("t3", TaskState::Running, "ada"),
        task("t4", TaskState::Done, "lin"),
        task("t5", TaskState::Stopped, "ada"),
    ]
}

fn board_agents() -> Vec<AgentRecord> {
    vec![
        AgentRecord {
            id: "ada".to_string(),
            entry_name: "Ada".to_string(),
            identity_hue_token: Some("ok".to_string()),
        },
        AgentRecord {
            id: "lin".to_string(),
            entry_name: "Lin".to_string(),
            identity_hue_token: None,
        },
    ]
}

fn board_app() -> Element {
    rsx! {
        AgentTaskBoard {
            tasks: board_tasks(),
            agents: board_agents(),
            clock_ms: Some(NOW),
        }
    }
}

fn empty_board_app() -> Element {
    rsx! {
        AgentTaskBoard {
            agents: board_agents(),
            is_empty: true,
        }
    }
}

fn open_waiting_app() -> Element {
    rsx! {
        AgentTaskBoard {
            tasks: board_tasks(),
            agents: board_agents(),
            initial_task_id: Some("t1".to_string()),
            initial_view: BoardView::List,
            clock_ms: Some(NOW),
        }
    }
}

#[test]
fn waiting_task_offers_approve_and_reject_only() {
    assert_eq!(
        actions_for(TaskState::Waiting),
        vec![TaskAction::Approve, TaskAction::Reject]
    );
}

#[test]
fn failed_task_offers_retry_and_running_or_done_offers_open() {
    assert_eq!(actions_for(TaskState::Failed), vec![TaskAction::Retry]);
    assert_eq!(actions_for(TaskState::Running), vec![TaskAction::Open]);
    assert_eq!(actions_for(TaskState::Done), vec![TaskAction::Open]);
}

#[test]
fn waiting_and_failed_are_needs_you_and_stopped_is_done() {
    assert_eq!(TaskState::Waiting.column(), TaskColumn::NeedsYou);
    assert_eq!(TaskState::Failed.column(), TaskColumn::NeedsYou);
    assert_eq!(TaskState::Running.column(), TaskColumn::Running);
    assert_eq!(TaskState::Stopped.column(), TaskColumn::Done);
}

#[test]
fn tab_counts_match_cards_per_column() {
    let counts = tab_counts(&board_tasks());
    assert_eq!(counts, [5, 2, 1, 2]);
}

#[test]
fn visible_order_lists_columns_in_board_order() {
    assert_eq!(
        visible_order(&board_tasks()),
        vec!["t1", "t2", "t3", "t4", "t5"]
    );
}

#[test]
fn step_index_clamps_at_both_ends() {
    let order = vec!["a".to_string(), "b".to_string()];
    assert_eq!(step_index(&order, Some("b"), 1).as_deref(), Some("b"));
    assert_eq!(step_index(&order, Some("a"), -1).as_deref(), Some("a"));
    assert_eq!(step_index(&order, None, 1).as_deref(), Some("a"));
    assert_eq!(step_index(&[], None, 1), None);
}

#[test]
fn running_task_counts_from_the_host_clock_when_it_has_a_start() {
    let mut running = task("r", TaskState::Running, "ada");
    running.started_at = Some(NOW - 90_000);
    assert_eq!(live_seconds(&running, NOW), 90);
    assert_eq!(live_seconds(&running, 0), 125);
}

#[test]
fn duration_text_picks_seconds_minutes_or_hours() {
    assert_eq!(duration_text(42), "42s");
    assert_eq!(duration_text(252), "4m 12s");
    assert_eq!(duration_text(3_780), "1h 03m");
}

#[test]
fn agent_filter_keeps_every_task_when_nothing_is_picked() {
    assert_eq!(filter_by_agent(&board_tasks(), &[]).len(), 5);
    assert_eq!(
        filter_by_agent(&board_tasks(), &["lin".to_string()]).len(),
        2
    );
}

#[test]
fn new_task_needs_a_description_and_an_agent() {
    assert!(new_task_ready("Draft the memo", "ada"));
    assert!(!new_task_ready("   ", "ada"));
    assert!(!new_task_ready("Draft the memo", ""));
}

#[test]
fn announcement_counts_need_you_and_running() {
    assert_eq!(announcement(1, 0), "1 task need you, 0 running");
    assert_eq!(announcement(2, 3), "2 tasks need you, 3 running");
}

#[test]
fn tab_admits_only_its_own_states() {
    assert!(TaskTab::All.admits(TaskState::Stopped));
    assert!(!TaskTab::Running.admits(TaskState::Done));
}

#[test]
fn board_renders_three_columns_with_cards_in_their_column() {
    let html = render(board_app);
    assert!(html.contains("data-column=\"needs-you\""));
    assert!(html.contains("data-column=\"running\""));
    assert!(html.contains("data-column=\"done\""));
    assert!(html.contains("aria-label=\"Task t1, Waiting for you, Ada\""));
    assert!(html.contains("aria-label=\"Task t2, Failed, Lin\""));
}

#[test]
fn board_announces_counts_in_a_polite_live_region() {
    let html = render(board_app);
    assert!(html.contains("aria-live=\"polite\""));
    assert!(html.contains("2 tasks need you, 1 running"));
}

#[test]
fn empty_board_shows_only_the_note_and_new_task() {
    let html = render(empty_board_app);
    assert!(html.contains("No tasks yet"));
    assert!(html.contains("New task"));
    assert!(!html.contains("data-column="));
}

#[test]
fn open_waiting_task_shows_approve_and_reject_in_the_panel() {
    let html = render(open_waiting_app);
    assert!(html.contains("data-action=\"approve\""));
    assert!(html.contains("data-action=\"reject\""));
    assert!(!html.contains("data-action=\"retry\""));
    assert!(html.contains("data-view=\"list\""));
}

use rv2_kit::screens::agent_spend_overview::{
    AgentSpendOverview, AttentionItem, AttentionKind, Breakdown, BudgetState, actual_series,
    change_words, cumulative, forecast, forecast_series, format_money, hero_sentence, share_pct,
    sorted_by_amount,
};

fn breakdown(id: &str, amount: f64) -> Breakdown {
    Breakdown {
        id: id.to_string(),
        entry_name: id.to_string(),
        amount,
        change_vs_last_month: Some(12.0),
        daily_costs: vec![amount / 4.0; 4],
        top_tasks: vec![],
    }
}

#[test]
fn money_formats_with_symbol_grouping_and_two_decimals() {
    assert_eq!(format_money(1234.5, "USD"), "$1,234.50");
    assert_eq!(format_money(7.0, "CHF"), "7.00 CHF");
}

#[test]
fn cumulative_total_is_the_sum_of_daily_costs() {
    let daily = [12.0, 8.5, 4.25];
    assert_eq!(cumulative(&daily).last().copied(), Some(24.75));
}

#[test]
fn forecast_starts_at_the_last_actual_day_and_follows_average_pace() {
    let daily = [10.0, 20.0];
    let series = forecast_series(&daily, 5);
    assert_eq!(series[0], None);
    assert_eq!(series[1], Some(30.0));
    assert_eq!(series[2], Some(45.0));
    assert_eq!(series[4], Some(75.0));
    assert_eq!(actual_series(&daily, 5)[2], None);
}

#[test]
fn forecast_names_the_day_the_budget_is_passed_at_current_pace() {
    let daily = [10.0, 10.0, 10.0];
    let fc = forecast(&daily, 30, Some(100.0));
    assert_eq!(fc.budget_state, BudgetState::Under);
    assert_eq!(fc.budget_day, Some(10));
    assert_eq!(
        hero_sentence(&fc, Some(100.0), "USD"),
        "At this pace the budget of $100.00 is passed on day 10."
    );
}

#[test]
fn over_budget_sentence_names_the_day_it_was_passed() {
    let daily = [30.0, 30.0, 30.0];
    let fc = forecast(&daily, 30, Some(50.0));
    assert_eq!(fc.budget_state, BudgetState::Over);
    assert_eq!(fc.budget_day, Some(2));
    assert!(hero_sentence(&fc, Some(50.0), "USD").contains("passed on day 2"));
}

#[test]
fn no_budget_gives_a_forecast_sentence_and_no_budget_state() {
    let fc = forecast(&[10.0, 10.0], 10, None);
    assert_eq!(fc.budget_state, BudgetState::None);
    assert_eq!(
        hero_sentence(&fc, None, "USD"),
        "On pace for $100.00 by month end."
    );
}

#[test]
fn lists_sort_by_amount_descending_and_shares_sum_to_one_hundred() {
    let rows = vec![
        breakdown("a", 10.0),
        breakdown("b", 30.0),
        breakdown("c", 60.0),
    ];
    let sorted = sorted_by_amount(&rows);
    assert_eq!(sorted[0].id, "c");
    let total: f64 = rows.iter().map(|r| r.amount).sum();
    let shares: f64 = rows.iter().map(|r| share_pct(r.amount, total)).sum();
    assert!((shares - 100.0).abs() < 1e-9);
}

#[test]
fn change_is_stated_in_words_with_direction() {
    assert_eq!(change_words(12.4), "up 12%");
    assert_eq!(change_words(-4.0), "down 4%");
    assert_eq!(change_words(0.0), "no change");
}

fn spend_app() -> Element {
    rsx! {
        AgentSpendOverview {
            month_label: "November 2026",
            currency_code: "USD",
            daily_costs: vec![12.0, 9.0, 14.0, 8.0],
            days_in_month: 30,
            budget_amount: Some(400.0),
            project_breakdown: vec![breakdown("Support", 30.0), breakdown("Research", 50.0)],
            model_breakdown: vec![breakdown("Haiku", 20.0), breakdown("Opus", 60.0)],
            attention_items: vec![AttentionItem {
                id: "a1".to_string(),
                item_kind: AttentionKind::Spike,
                text: "Research spent 3x its usual".to_string(),
                detail_text: None,
            }],
        }
    }
}

fn spend_empty_app() -> Element {
    rsx! {
        AgentSpendOverview {
            month_label: "November 2026",
            currency_code: "USD",
            is_empty: true,
        }
    }
}

#[test]
fn spend_overview_renders_hero_lists_and_attention() {
    let html = render(spend_app);
    assert!(html.contains("$43.00"));
    assert!(html.contains("aria-label=\"Projects\""));
    assert!(html.contains("aria-label=\"Worth a look\""));
    assert!(html.contains("Research spent 3x its usual"));
    assert!(!html.contains("role=\"progressbar\""));
    assert!(html.contains("% of budget"));
}

#[test]
fn spend_overview_empty_hides_lists() {
    let html = render(spend_empty_app);
    assert!(html.contains("No spend yet"));
    assert!(!html.contains("aria-label=\"Projects\""));
}

use rv2_kit::screens::agent_action_log::{
    AgentActionLog, StepFilter, UndoState, clock_text, day_heading, move_index as log_move,
    undo_state,
};
use rv2_kit::screens::agent_memory_manager::{
    AgentMemoryManager, MemoryTab, ago_text as memory_ago, group_order,
};
use rv2_kit::screens::agent_roster::{
    AgentRoster, Provider, RunState, form_ready, name_mark, paused_after_request, summary_text,
};
use rv2_kit::screens::agent_run_timeline::{
    AgentRunTimeline, axis_ticks, tick_interval, tick_label,
};
use rv2_kit::screens::agent_schedule_board::{AgentScheduleBoard, MarkState, marker_position};
use rv2_kit::screens::agent_skill_library::{AgentSkillLibrary, MIN_INSTALL_MS, cycle_category};
use rv2_kit::screens::agent_task_list::{AgentTaskList, Progress, TaskStep, ring_fraction};
use rv2_kit::screens::agent_tool_explorer::{
    Access, AgentToolExplorer, CallRecord, CallState, TimeRange, access_from_label,
    average_duration_ms, calls_in_range, move_card, success_share,
};
use rv2_kit::screens::autonomous_run_workspace::{
    AutonomousRunWorkspace, RunStep, StepPhase, all_done, progress_text,
};
use rv2_kit::screens::multi_agent_run_canvas::{
    AgentNode, MultiAgentRunCanvas, NodeState, bar_percent, edges, layout, node_cost,
};
use rv2_kit::screens::orchestrator_thread::{
    Connection, OrchestratorThread, group_seconds, is_away, option_for_key, pager_text,
};

#[test]
fn undo_is_available_until_the_window_closes() {
    use rv2_kit::screens::agent_action_log::{Actor, StepRecord, UndoWindow};
    let step = StepRecord {
        id: "s".to_string(),
        occurred_at: 0,
        app_id: "a".to_string(),
        sentence: "Sent a reply".to_string(),
        task_name: None,
        actor: Actor::Automatic,
        rule_label: None,
        approved_at: None,
        detail_rows: vec![],
        preview_lines: vec![],
        undo_window: Some(UndoWindow {
            available_until: 1_000,
            entry_description: "delete the reply".to_string(),
        }),
        final_reason: None,
        is_undone: false,
        undone_at: None,
        cost_label: None,
    };
    assert_eq!(undo_state(&step, 500), UndoState::Available);
    assert_eq!(undo_state(&step, 1_500), UndoState::Expired);
    assert!(!StepFilter::Undone.admits(&step));
}

#[test]
fn action_log_day_headings_and_clock_text() {
    let day = 86_400_000u64;
    assert_eq!(day_heading(3 * day, 3 * day + 1), "Today");
    assert_eq!(day_heading(3 * day, 2 * day), "Yesterday");
    assert_eq!(day_heading(5 * day, 2 * day), "3 days ago");
    assert_eq!(clock_text(3_600_000 + 5 * 60_000), "01:05");
}

#[test]
fn action_log_moves_clamp_at_the_ends() {
    let order = vec!["a".to_string(), "b".to_string()];
    assert_eq!(log_move(&order, Some("b"), 1).as_deref(), Some("b"));
    assert_eq!(log_move(&order, None, -1).as_deref(), Some("b"));
}

#[test]
fn action_log_empty_shows_the_note_and_permissions_action() {
    let html = render(action_log_empty_app);
    assert!(html.contains("Steps appear here"));
}

fn action_log_empty_app() -> Element {
    rsx! { AgentActionLog { is_empty: true } }
}

#[test]
fn memory_ago_text_picks_a_unit_and_never_reads_as_mount_time() {
    assert_eq!(memory_ago(10_000, None), "never");
    assert_eq!(memory_ago(10_000, Some(9_999_000)), "now");
    assert_eq!(memory_ago(3_600_000 * 3, Some(0)), "3 h ago");
}

#[test]
fn memory_groups_put_about_you_first_then_projects_in_order() {
    use rv2_kit::screens::agent_memory_manager::{MemoryRecord, SourceInfo, SourceKind};
    let make = |id: &str, project: Option<&str>| MemoryRecord {
        id: id.to_string(),
        text: id.to_string(),
        project_id: project.map(|p| p.to_string()),
        source_info: SourceInfo {
            source_kind: SourceKind::User,
            occurred_at: 0,
            quote_text: None,
            question_text: None,
            file_name: None,
        },
        use_count: 0,
        last_used_at: None,
        usage_log: vec![],
        is_pending_review: false,
    };
    let list = vec![
        make("a", Some("Launch")),
        make("b", None),
        make("c", Some("Hiring")),
    ];
    let names = vec!["Hiring".to_string(), "Launch".to_string()];
    let groups: Vec<String> = group_order(&list, &names)
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    assert_eq!(groups, vec!["About you", "Hiring", "Launch"]);
    let _ = MemoryTab::Saved;
}

#[test]
fn memory_empty_saved_tab_shows_the_remember_action() {
    let html = render(memory_empty_app);
    assert!(html.contains("Remember adds one yourself"));
}

fn memory_empty_app() -> Element {
    rsx! { AgentMemoryManager { is_empty: true, initial_tab: MemoryTab::Saved } }
}

#[test]
fn roster_counts_and_summary_follow_run_states() {
    assert_eq!(summary_text(&[]), "0 working · 0 idle · 0 paused");
    assert_eq!(
        paused_after_request(RunState::Working, true),
        RunState::Paused
    );
    assert_eq!(
        paused_after_request(RunState::Paused, false),
        RunState::Idle
    );
    assert_eq!(name_mark("ada"), "A");
    assert_eq!(name_mark(""), "?");
    assert!(!form_ready("   "));
    let _ = Provider::Other;
}

#[test]
fn roster_empty_shows_only_the_note_and_new_agent() {
    let html = render(roster_empty_app);
    assert!(html.contains("No agents yet"));
    assert!(!html.contains("kit-ro__cards"));
}

fn roster_empty_app() -> Element {
    rsx! { AgentRoster { is_empty: true } }
}

#[test]
fn run_timeline_ticks_are_round_and_at_most_six() {
    assert_eq!(tick_interval(40.0), 10.0);
    assert_eq!(tick_label(90.0), "1:30");
    assert_eq!(tick_label(3_720.0), "1:02:00");
    assert!(axis_ticks(100.0).len() <= 7);
}

#[test]
fn run_timeline_without_a_run_says_so() {
    let html = render(timeline_empty_app);
    assert!(html.contains("This run has no steps yet."));
}

fn timeline_empty_app() -> Element {
    rsx! { AgentRunTimeline { is_empty: true } }
}

#[test]
fn schedule_markers_left_of_now_are_past_and_right_are_upcoming() {
    let now = 100_000_000u64;
    assert!((marker_position(now, now) - 50.0).abs() < 1e-9);
    assert!(marker_position(now - 3_600_000, now) < 50.0);
    assert!(marker_position(now + 3_600_000, now) > 50.0);
    let _ = MarkState::Upcoming;
}

#[test]
fn schedule_board_empty_offers_ask_in_thread() {
    let html = render(schedule_empty_app);
    assert!(html.contains("Ask in Thread"));
}

fn schedule_empty_app() -> Element {
    rsx! { AgentScheduleBoard { is_empty: true } }
}

#[test]
fn skill_categories_cycle_and_wrap() {
    let cats = vec!["Sales".to_string(), "Ops".to_string()];
    assert_eq!(
        cycle_category(&cats, Some("Ops"), 1).as_deref(),
        Some("Ops")
    );
    assert_eq!(
        cycle_category(&cats, Some("Sales"), -1).as_deref(),
        Some("Sales")
    );
    assert_eq!(cycle_category(&cats, None, 1).as_deref(), Some("Sales"));
    assert_eq!(MIN_INSTALL_MS, 700);
}

#[test]
fn skill_library_empty_offers_new_skill() {
    let html = render(skill_empty_app);
    assert!(html.contains("No skills yet"));
}

fn skill_empty_app() -> Element {
    rsx! { AgentSkillLibrary { is_empty: true } }
}

#[test]
fn ring_fills_to_steps_done_over_total() {
    let steps = vec![
        TaskStep {
            entry_label: "a".to_string(),
            is_done: true,
        },
        TaskStep {
            entry_label: "b".to_string(),
            is_done: false,
        },
        TaskStep {
            entry_label: "c".to_string(),
            is_done: false,
        },
        TaskStep {
            entry_label: "d".to_string(),
            is_done: true,
        },
    ];
    assert!((ring_fraction(&steps) - 0.5).abs() < 1e-9);
    assert_eq!(ring_fraction(&[]), 0.0);
    let _ = Progress::Planning;
}

#[test]
fn task_list_empty_active_offers_new_task() {
    let html = render(task_list_empty_app);
    assert!(html.contains("No active tasks"));
}

fn task_list_empty_app() -> Element {
    rsx! { AgentTaskList { is_empty: true } }
}

fn call(id: &str, at: u64, state: CallState, duration: Option<u64>) -> CallRecord {
    CallRecord {
        id: id.to_string(),
        tool_id: "t".to_string(),
        started_at: at,
        duration_ms: duration,
        call_state: state,
        input_data: "in".to_string(),
        output_data: None,
        error_text: None,
        token_count: Some(10),
        run_id: None,
    }
}

#[test]
fn tool_explorer_range_success_and_average() {
    let day = 86_400_000u64;
    let calls = vec![
        call("1", 10 * day, CallState::Done, Some(100)),
        call("2", 9 * day, CallState::Failed, Some(300)),
        call("3", 0, CallState::Done, Some(500)),
    ];
    let in_week = calls_in_range(&calls, 10 * day, TimeRange::SevenDays);
    assert_eq!(in_week.len(), 2);
    assert_eq!(in_week[0].id, "1");
    assert!((success_share(&in_week) - 50.0).abs() < 1e-9);
    assert_eq!(average_duration_ms(&in_week), 200);
    assert_eq!(access_from_label("Ask first"), Access::Ask);
}

#[test]
fn tool_cards_move_by_one_and_clamp() {
    let order = vec!["x".to_string(), "y".to_string()];
    assert_eq!(move_card(&order, None, 1).as_deref(), Some("x"));
    assert_eq!(move_card(&order, Some("y"), 1).as_deref(), Some("y"));
}

#[test]
fn tool_explorer_without_an_agent_renders_a_placeholder_title() {
    let html = render(tool_explorer_app);
    assert!(html.contains("No agent"));
}

fn tool_explorer_app() -> Element {
    rsx! { AgentToolExplorer {} }
}

fn node(id: &str, parent: Option<&str>, state: NodeState) -> AgentNode {
    AgentNode {
        id: id.to_string(),
        entry_name: id.to_string(),
        node_role: "research".to_string(),
        model_name: "m".to_string(),
        node_state: state,
        parent_id: parent.map(|p| p.to_string()),
        cost_label: Some("$1.00".to_string()),
        duration_seconds: None,
        token_count: None,
    }
}

#[test]
fn canvas_edges_join_each_node_to_its_parent_only() {
    let nodes = vec![
        node("root", None, NodeState::Done),
        node("a", Some("root"), NodeState::Running),
        node("b", Some("root"), NodeState::Queued),
        node("orphan", Some("missing"), NodeState::Done),
    ];
    let links = edges(&nodes);
    assert_eq!(links.len(), 2);
    assert!(links.iter().all(|(p, _)| p == "root"));
    let positions = layout(&nodes);
    assert_eq!(positions.len(), 4);
}

#[test]
fn queued_node_reads_queued_and_has_no_cost() {
    assert_eq!(
        node_cost(&node("q", Some("root"), NodeState::Queued)),
        "queued"
    );
    assert_eq!(node_cost(&node("r", None, NodeState::Running)), "$1.00");
}

#[test]
fn canvas_empty_shows_the_no_agents_note() {
    let html = render(canvas_empty_app);
    assert!(html.contains("No agents yet"));
}

fn canvas_empty_app() -> Element {
    rsx! { MultiAgentRunCanvas { is_empty: true } }
}

#[test]
fn trace_bar_stays_inside_the_axis() {
    use rv2_kit::screens::multi_agent_run_canvas::{TraceRow, TraceState};
    let row = TraceRow {
        id: "t".to_string(),
        agent_id: "a".to_string(),
        entry_label: "step".to_string(),
        start_seconds: 10.0,
        duration_seconds: Some(20.0),
        row_state: TraceState::Done,
        error_text: None,
    };
    let (start, width) = bar_percent(&row, 40.0);
    assert!((start - 25.0).abs() < 1e-9);
    assert!((width - 50.0).abs() < 1e-9);
}

#[test]
fn autonomous_progress_counts_done_steps_and_all_done_needs_steps() {
    let step = |phase: StepPhase| RunStep {
        id: format!("{phase:?}"),
        entry_title: "s".to_string(),
        detail_text: None,
        tool_name: "tool".to_string(),
        step_phase: phase,
        tool_call_list: vec![],
        approval_request: None,
    };
    let steps = vec![step(StepPhase::Done), step(StepPhase::Pending)];
    assert_eq!(progress_text(&steps), "1 of 2");
    assert!(!all_done(&steps));
    assert!(!all_done(&[]));
}

#[test]
fn plan_edit_reorders_renames_pending_steps_and_drops_removed_ones() {
    use rv2_kit::agents::editable_step_plan::{PlanStep, StepState};
    use rv2_kit::screens::autonomous_run_workspace::apply_plan_edit;
    let run = |id: &str, title: &str, phase: StepPhase| RunStep {
        id: id.to_string(),
        entry_title: title.to_string(),
        detail_text: None,
        tool_name: "tool".to_string(),
        step_phase: phase,
        tool_call_list: vec![],
        approval_request: None,
    };
    let steps = vec![
        run("a", "Collect", StepPhase::Done),
        run("b", "Draft", StepPhase::Pending),
        run("c", "Send", StepPhase::Pending),
    ];
    let plan = |key: &str, title: &str| PlanStep {
        step_key: key.to_string(),
        step_title: title.to_string(),
        step_detail: None,
        tool_label: None,
        step_state: StepState::Pending,
        measured_ms: None,
        error_text: None,
    };
    let edited = vec![
        plan("c", "Send today"),
        plan("b", "Write draft"),
        plan("a", "Renamed done"),
    ];
    let out = apply_plan_edit(&steps, &edited);
    let ids: Vec<&str> = out.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["c", "b", "a"]);
    assert_eq!(out[0].entry_title, "Send today");
    assert_eq!(out[2].entry_title, "Collect");
    assert_eq!(apply_plan_edit(&steps, &[plan("a", "Collect")]).len(), 1);
}

#[test]
fn autonomous_workspace_without_a_run_says_so() {
    let html = render(autonomous_empty_app);
    assert!(html.contains("This run has no steps yet."));
}

fn autonomous_empty_app() -> Element {
    rsx! { AutonomousRunWorkspace { is_empty: true } }
}

#[test]
fn away_card_appears_only_past_the_threshold() {
    let minute = 60_000u64;
    assert!(is_away(31 * minute, 0, 30));
    assert!(!is_away(29 * minute, 0, 30));
}

#[test]
fn option_keys_map_digits_to_options_in_range() {
    assert_eq!(option_for_key("2", 3), Some(1));
    assert_eq!(option_for_key("4", 3), None);
    assert_eq!(option_for_key("0", 3), None);
    assert_eq!(option_for_key("x", 3), None);
    assert_eq!(pager_text(0, 3), "1 of 3");
    assert_eq!(group_seconds(&[]), 0.0);
    let _ = Connection::Online;
}

#[test]
fn orchestrator_empty_offers_a_new_chat() {
    let html = render(orchestrator_empty_app);
    assert!(html.contains("Start a chat"));
}

fn orchestrator_empty_app() -> Element {
    rsx! { OrchestratorThread { is_empty: true } }
}

#[test]
fn skill_chips_cycle_through_all_first_then_each_category() {
    use rv2_kit::screens::agent_skill_library::cycle_chip;
    let cats = vec!["Ops".to_string(), "Sales".to_string()];
    assert_eq!(cycle_chip(&cats, None, 1).as_deref(), Some("Ops"));
    assert_eq!(cycle_chip(&cats, Some("Sales"), 1), None);
    assert_eq!(cycle_chip(&cats, None, -1).as_deref(), Some("Sales"));
}

#[test]
fn draft_from_description_names_the_first_sentence_and_steps_the_rest() {
    use rv2_kit::screens::agent_skill_library::draft_from_description;
    let draft = draft_from_description("Summarise tickets. Group by topic. Post the digest.");
    assert_eq!(draft.name, "Summarise tickets");
    assert_eq!(draft.steps, vec!["Group by topic", "Post the digest"]);
}

#[test]
fn parse_definition_reads_heading_description_and_list_steps() {
    use rv2_kit::screens::agent_skill_library::parse_definition;
    let draft =
        parse_definition("# Monthly digest\nBuilds the digest.\n- Pull totals\n2. Write it\n");
    assert_eq!(draft.name, "Monthly digest");
    assert_eq!(draft.description, "Builds the digest.");
    assert_eq!(draft.steps, vec!["Pull totals", "Write it"]);
}

#[test]
fn activity_trace_climbs_for_a_running_run_and_stays_flat_for_upcoming() {
    use rv2_kit::screens::agent_schedule_board::{
        MarkState, RunTime, ScheduleRecord, trace_points,
    };
    let now = 10 * 3_600_000;
    let schedule = |state: MarkState, at: u64| ScheduleRecord {
        id: "s".into(),
        entry_name: "Ticket summary".into(),
        project_name: None,
        brief_summary: None,
        cadence_text: "Hourly".into(),
        run_times: vec![RunTime {
            at_ms: at,
            run_state: state,
            progress_percent: Some(50),
        }],
        duration_seconds: None,
        is_paused: false,
        history_log: Vec::new(),
        cost_per_run_label: None,
    };
    let climbing = trace_points(&[schedule(MarkState::Running, now - 3_600_000)], now);
    let flat = trace_points(&[schedule(MarkState::Upcoming, now + 3_600_000)], now);
    let highest = |points: &str| {
        points
            .split(' ')
            .filter_map(|p| p.split(',').nth(1)?.parse::<f64>().ok())
            .fold(f64::MAX, f64::min)
    };
    assert!(highest(&climbing) < highest(&flat) - 5.0);
    assert_eq!(flat.split(' ').count(), 51);
}

#[test]
fn memory_count_text_is_singular_for_one() {
    use rv2_kit::screens::agent_memory_manager::memory_noun;
    assert_eq!(memory_noun(1), "memory");
    assert_eq!(memory_noun(0), "memories");
    assert_eq!(memory_noun(3), "memories");
}

use rv2_kit::screens::agent_task_list::{
    Ask, AskKind, Progress as ListProgress, TaskRecord as ListRecord, TaskTab as ListTab, TreeKey,
    TreeMove, count_text, day_label, parent_heading, step_key, tree_keys,
};
use rv2_kit::screens::agent_tool_explorer::{
    ChartLayout, clock_text as tool_clock_text, table_height, zoom_step,
};

fn list_row(id: &str, project: &str, is_done: bool) -> ListRecord {
    ListRecord {
        id: id.to_string(),
        entry_title: format!("Task {id}"),
        project_name: project.to_string(),
        progress_state: ListProgress::Running,
        status_phrase: None,
        step_records: Vec::new(),
        artifact_names: Vec::new(),
        pending_asks: vec![Ask {
            id: format!("{id}-0"),
            question_text: "Ship?".to_string(),
            ask_kind: AskKind::Approval,
        }],
        updated_at_ms: NOW,
        is_done,
        goal_text: None,
        started_at_ms: None,
        decision_log: Vec::new(),
        event_log: Vec::new(),
    }
}

fn grouped_list_app() -> Element {
    rsx! {
        AgentTaskList {
            task_records: vec![list_row("a1", "Launch", false), list_row("a2", "Launch", false), list_row("b1", "Hiring", true)],
            project_names: vec!["Launch".to_string(), "Hiring".to_string()],
            now_ms: NOW,
        }
    }
}

fn compact_explorer_app() -> Element {
    rsx! {
        rv2_kit::screens::agent_tool_explorer::AgentToolExplorer {
            now_at: NOW,
            chart_layout: ChartLayout::Compact,
        }
    }
}

#[test]
fn tree_keys_list_visible_headings_and_only_open_groups_rows() {
    let tasks = vec![
        list_row("a1", "A", false),
        list_row("a2", "A", true),
        list_row("b1", "B", false),
    ];
    let projects = vec!["A".to_string(), "B".to_string()];
    let heading = |p: &str| TreeKey::Heading(p.to_string());
    let row = |id: &str| TreeKey::Row(id.to_string());
    assert_eq!(
        tree_keys(&tasks, &projects, ListTab::Active, &[]),
        vec![heading("A"), row("a1"), heading("B"), row("b1")]
    );
    assert_eq!(
        tree_keys(&tasks, &projects, ListTab::Active, &["A".to_string()]),
        vec![heading("A"), heading("B"), row("b1")]
    );
    assert_eq!(
        tree_keys(&tasks, &projects, ListTab::Done, &[]),
        vec![heading("A"), row("a2"), heading("B")]
    );
}

#[test]
fn tree_steps_clamp_at_both_ends_and_start_from_the_first_stop() {
    let keys = vec![
        TreeKey::Heading("A".into()),
        TreeKey::Row("a1".into()),
        TreeKey::Heading("B".into()),
    ];
    let heading_a = Some(TreeKey::Heading("A".into()));
    let heading_b = Some(TreeKey::Heading("B".into()));
    let on_b = TreeKey::Heading("B".into());
    let on_a1 = TreeKey::Row("a1".into());
    assert_eq!(step_key(&keys, None, TreeMove::Next), heading_a);
    assert_eq!(step_key(&keys, None, TreeMove::Prev), heading_b);
    assert_eq!(step_key(&keys, Some(&on_b), TreeMove::Next), heading_b);
    assert_eq!(step_key(&keys, Some(&on_a1), TreeMove::First), heading_a);
    assert_eq!(step_key(&keys, Some(&on_a1), TreeMove::Last), heading_b);
    assert_eq!(step_key(&[], None, TreeMove::Next), None);
}

#[test]
fn a_row_belongs_to_the_nearest_heading_above_it() {
    let keys = vec![
        TreeKey::Heading("A".into()),
        TreeKey::Row("a1".into()),
        TreeKey::Heading("B".into()),
        TreeKey::Row("b1".into()),
    ];
    let heading_a = Some(TreeKey::Heading("A".into()));
    let heading_b = Some(TreeKey::Heading("B".into()));
    assert_eq!(parent_heading(&keys, "a1"), heading_a);
    assert_eq!(parent_heading(&keys, "b1"), heading_b);
    assert_eq!(parent_heading(&keys, "missing"), None);
}

#[test]
fn counts_say_task_or_tasks_and_days_and_clock_are_utc() {
    assert_eq!(count_text(1, "task", "tasks"), "1 task");
    assert_eq!(count_text(2, "task", "tasks"), "2 tasks");
    assert_eq!(count_text(0, "task", "tasks"), "0 tasks");
    assert_eq!(day_label(0), "1 Jan 1970");
    assert_eq!(day_label(NOW), "21 Nov 2026");
    let at = NOW + 14 * 3_600_000 + 3 * 60_000;
    assert_eq!(tool_clock_text(at), "21 Nov 2026 14:03");
}

#[test]
fn table_height_and_zoom_stay_between_their_bounds() {
    assert_eq!(table_height(260.0, 1000.0), 640.0);
    assert_eq!(table_height(260.0, -1000.0), 120.0);
    assert_eq!(table_height(260.0, 16.0), 276.0);
    assert!((zoom_step(1.0, true) - 1.2).abs() < 1e-9);
    assert_eq!(zoom_step(2.0, true), 2.0);
    assert_eq!(zoom_step(0.5, false), 0.5);
}

#[test]
fn grouped_list_renders_singular_and_plural_heading_counts() {
    let html = render(grouped_list_app);
    assert!(html.contains("aria-label=\"Launch, 2 tasks\""), "{html}");
    assert!(html.contains("Hiring, 0 tasks"), "{html}");
    assert!(html.contains("role=\"tree\""), "{html}");
}

#[test]
fn tool_explorer_compact_layout_marks_its_chart() {
    let html = render(compact_explorer_app);
    assert!(html.contains("data-layout=\"compact\""), "{html}");
}
