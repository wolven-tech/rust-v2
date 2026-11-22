use dioxus::prelude::*;
use rv2_kit::blocks::agent_fleet_overview::{
    ActivityPoint, AgentFleetOverview, AgentStatus, FleetAgent, InboxAction, InboxItem, InboxKind,
    Outcome, StatusFilter, activity_curve, activity_summary, child_count, inbox_item_visible,
    status_counts, wait_label,
};
use rv2_kit::blocks::agent_team_session::{
    PlanStep, Recipients, StepStatus, TeamAgent, TeamMessage, mention_query, message_visible,
    plan_running_label, resolve_mentions, step_dimmed,
};
use rv2_kit::blocks::agent_timeline_overview::{
    TimelineSpan, axis_label, effective_end, lane_spans, since_last_seen, span_geometry,
};
use rv2_kit::blocks::chart_answer_chat::{
    BreakdownGroup, EscapeAction, breakdown_shares, escape_action, range_bounds, range_summary,
    selection_anchor,
};
use rv2_kit::blocks::client_workspace::{
    ItemKind, Reference, TimelineItem, insert_reference, mention_matches, relative_age,
    visible_items,
};
use rv2_kit::blocks::expandable_panel_strip::{edge_index, wrap_index};
use rv2_kit::blocks::integration_catalog::{
    Integration, category_total, connected_count, filter_integrations,
};
use rv2_kit::blocks::invoice_credit_pair::{
    CreditedLine, Line, LineKind, can_issue, credit_total, full_credit, is_valid_amount,
};
use rv2_kit::blocks::kanban_task_board::{
    StatusColumn, WorkItem, due_words, filter_items, move_column, move_to_status, reorder_in_column,
};
use rv2_kit::blocks::metric_overview_board::{
    OverviewSeries, good_or_bad, overview_totals, percent_change, visible_series,
};
use rv2_kit::blocks::plan_comparison_section::{
    billing_for_key, format_price, price_for, yearly_saving_percent,
};
use rv2_kit::blocks::run_tree_inspector::{
    CallKind, CallStatus, RunAgent, RunCall, RunStatus, bar_percent, duration_label, error_count,
    tree_order,
};
use rv2_kit::blocks::sales_metrics_panel::{Ranked, scaled_widths, sort_ranked};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn fleet_agent(id: &str, parent: Option<&str>, status: AgentStatus) -> FleetAgent {
    FleetAgent {
        id: id.into(),
        name: id.into(),
        model_label: "m".into(),
        parent_id: parent.map(Into::into),
        status,
        cost_usd: 0.0,
        elapsed_secs: 0,
        current_task: String::new(),
    }
}

fn fleet() -> Vec<FleetAgent> {
    vec![
        fleet_agent("lead", None, AgentStatus::Running),
        fleet_agent("api", Some("lead"), AgentStatus::Running),
        fleet_agent("ui", Some("lead"), AgentStatus::Failed),
        fleet_agent("docs", None, AgentStatus::Waiting),
    ]
}

#[test]
fn fleet_counts_and_child_count_ignore_the_filter() {
    let agents = fleet();
    assert_eq!(status_counts(&agents), [4, 2, 1, 1]);
    assert_eq!(child_count(&agents, "lead"), 2);
}

#[test]
fn fleet_activity_outside_the_filter_is_not_counted() {
    let agents = fleet();
    let points = vec![
        ActivityPoint {
            at: 10,
            agent_id: "ui".into(),
            outcome: Outcome::Failed,
        },
        ActivityPoint {
            at: 20,
            agent_id: "docs".into(),
            outcome: Outcome::NeedsYou,
        },
        ActivityPoint {
            at: 30,
            agent_id: "api".into(),
            outcome: Outcome::Done,
        },
    ];
    assert_eq!(
        activity_summary(&points, &agents, StatusFilter::Failed),
        (0, 0, 1)
    );
    assert_eq!(
        activity_summary(&points, &agents, StatusFilter::All),
        (1, 1, 1)
    );
    assert_eq!(
        activity_curve(&points, &agents, StatusFilter::Failed, 0, 40).len(),
        1
    );
}

#[test]
fn fleet_inbox_follows_its_agent_status() {
    let agents = fleet();
    let item = InboxItem {
        id: "i".into(),
        agent_id: "ui".into(),
        kind: InboxKind::Failure,
        title: "t".into(),
        detail: "d".into(),
        waiting_secs: 90,
        actions: vec![InboxAction {
            id: "retry".into(),
            label: "Retry".into(),
        }],
    };
    assert!(inbox_item_visible(&item, &agents, StatusFilter::Failed));
    assert!(!inbox_item_visible(&item, &agents, StatusFilter::Running));
    assert_eq!(wait_label(90), "1 min");
}

#[test]
fn fleet_renders_tree_roles_and_filter_empty_message() {
    fn app() -> Element {
        rsx! { AgentFleetOverview { agents: fleet(), inbox_items: vec![], activity: vec![], last_seen_at: 0, now_at: 100, status_filter: StatusFilter::Failed } }
    }
    let html = render(app);
    assert!(html.contains("role=\"tree\""));
    assert!(html.contains("Nothing needs you right now."));
    assert!(html.contains("aria-label=\"ui\""));
    assert!(!html.contains("aria-label=\"api\""));
}

fn team_msg(id: &str, author: &str, to: Recipients) -> TeamMessage {
    TeamMessage {
        id: id.into(),
        author_id: author.into(),
        to,
        reply_to_id: None,
        at_label: "09:00".into(),
        text: "x".into(),
        tool_calls: vec![],
    }
}

#[test]
fn team_focus_keeps_author_and_addressed_messages_only() {
    let to_dev = team_msg("a", "pm", Recipients::Ids(vec!["dev".into()]));
    let everyone = team_msg("b", "pm", Recipients::Everyone);
    let by_dev = team_msg("c", "dev", Recipients::Everyone);
    assert!(message_visible(&to_dev, Some("dev")));
    assert!(!message_visible(&everyone, Some("dev")));
    assert!(message_visible(&by_dev, Some("dev")));
    assert!(message_visible(&everyone, None));
}

#[test]
fn team_plan_steps_dim_outside_the_focused_owner() {
    let step = PlanStep {
        id: "s".into(),
        title: "t".into(),
        owner_id: Some("qa".into()),
        status: StepStatus::Running,
        started_at_ms: Some(0),
        finished_at_ms: None,
        blocked_reason: None,
    };
    assert!(step_dimmed(&step, Some("dev")));
    assert!(!step_dimmed(&step, Some("qa")));
    assert!(!step_dimmed(&step, None));
}

#[test]
fn team_running_time_stops_when_every_started_step_finishes() {
    let done = PlanStep {
        id: "a".into(),
        title: "a".into(),
        owner_id: None,
        status: StepStatus::Done,
        started_at_ms: Some(0),
        finished_at_ms: Some(65_000),
        blocked_reason: None,
    };
    assert_eq!(
        plan_running_label(std::slice::from_ref(&done), 9_000_000),
        "1:05"
    );
    let running = PlanStep {
        id: "b".into(),
        title: "b".into(),
        owner_id: None,
        status: StepStatus::Running,
        started_at_ms: Some(0),
        finished_at_ms: None,
        blocked_reason: None,
    };
    assert_eq!(plan_running_label(&[done, running], 3_725_000), "1:02:05");
}

#[test]
fn team_mentions_resolve_and_query() {
    let agents = vec![TeamAgent {
        id: "dev".into(),
        name: "Dev".into(),
        model_label: "m".into(),
        role: "r".into(),
    }];
    assert_eq!(mention_query("ask @De"), Some("De"));
    assert_eq!(mention_query("ask @De now"), None);
    assert_eq!(
        resolve_mentions("@Dev please", &agents),
        vec!["dev".to_string()]
    );
}

#[test]
fn timeline_running_span_stops_at_the_marker() {
    let span = TimelineSpan {
        id: "s".into(),
        agent_id: "a".into(),
        label: "l".into(),
        start_at: 100,
        end_at: None,
        status: AgentStatus::Running,
        error_line: None,
    };
    let end = effective_end(&span, 600);
    assert_eq!(end, 600);
    let (left, width) = span_geometry(100, end, 0, 1000);
    assert!((left - 10.0).abs() < 1e-9 && (width - 50.0).abs() < 1e-9);
    let (_, clipped) = span_geometry(100, 5000, 0, 1000);
    assert!((clipped - 90.0).abs() < 1e-9);
}

#[test]
fn timeline_lanes_keep_time_order_and_summary_counts() {
    let mk = |id: &str, start: u64, status: AgentStatus, end: Option<u64>| TimelineSpan {
        id: id.into(),
        agent_id: "a".into(),
        label: id.into(),
        start_at: start,
        end_at: end,
        status,
        error_line: None,
    };
    let spans = vec![
        mk("late", 50, AgentStatus::Done, Some(60)),
        mk("early", 10, AgentStatus::Failed, Some(20)),
    ];
    let lane = lane_spans(&spans, "a");
    assert_eq!(lane[0].id, "early");
    assert_eq!(since_last_seen(&spans, &[], 15), (1, 0, 1));
    assert_eq!(axis_label(0, 600_000), "10 min ago");
    assert_eq!(axis_label(600_000, 600_000), "now");
}

#[test]
fn chart_range_bounds_and_summary() {
    assert_eq!(range_bounds(4, 1), (1, 4));
    let (change, sum) = range_summary(&[10.0, 12.0, 15.0], 0, 2);
    assert!((change - 50.0).abs() < 1e-9);
    assert!((sum - 37.0).abs() < 1e-9);
    assert_eq!(selection_anchor(None, 3), 3);
    assert_eq!(selection_anchor(Some(1), 3), 1);
}

#[test]
fn chart_escape_clears_selection_before_text() {
    assert_eq!(escape_action(true, true), EscapeAction::ClearSelection);
    assert_eq!(escape_action(false, true), EscapeAction::ClearText);
    assert_eq!(escape_action(false, false), EscapeAction::None);
}

#[test]
fn chart_breakdown_shares_sum_to_total_share() {
    let groups = vec![
        BreakdownGroup {
            label: "a".into(),
            value: 3.0,
        },
        BreakdownGroup {
            label: "b".into(),
            value: 1.0,
        },
    ];
    let shares = breakdown_shares(&groups);
    assert!((shares[0] - 75.0).abs() < 1e-9);
    assert!((shares.iter().sum::<f64>() - 100.0).abs() < 1e-9);
}

fn item(id: &str, kind: ItemKind, text: &str, at: u64, refs: Vec<Reference>) -> TimelineItem {
    TimelineItem {
        id: id.into(),
        kind,
        author: "Ana".into(),
        at_ms: at,
        title: None,
        text: text.into(),
        references: refs,
    }
}

#[test]
fn workspace_kind_and_search_must_both_match() {
    let items = vec![
        item("1", ItemKind::Note, "renewal call", 1, vec![]),
        item("2", ItemKind::Goal, "renewal goal", 3, vec![]),
        item(
            "3",
            ItemKind::Note,
            "budget",
            2,
            vec![Reference {
                label: "renewal deck".into(),
                target: "d".into(),
            }],
        ),
    ];
    let both = visible_items(&items, Some(ItemKind::Note), "renewal");
    assert_eq!(
        both.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec!["3", "1"]
    );
    assert_eq!(
        visible_items(&items, Some(ItemKind::Report), "renewal").len(),
        0
    );
    assert_eq!(relative_age(4 * 60_000), "4 min ago");
    assert_eq!(relative_age(0), "just now");
}

#[test]
fn workspace_mentions_only_suggest_reports() {
    let mut report = item("r", ItemKind::Report, "Q3 cash", 1, vec![]);
    report.title = Some("Q3 cash report".into());
    let items = vec![report, item("n", ItemKind::Note, "x", 2, vec![])];
    let found = mention_matches(&items, "see @Q3").unwrap_or_default();
    assert_eq!(found.len(), 1);
    assert_eq!(
        insert_reference("see @Q3", "Q3 cash report"),
        "see @Q3 cash report "
    );
}

#[test]
fn strip_wraps_and_edges() {
    assert_eq!(wrap_index(3, 2, 1), 0);
    assert_eq!(wrap_index(3, 0, -1), 2);
    assert_eq!(edge_index(5, true), 4);
    assert_eq!(edge_index(5, false), 0);
}

fn integ(id: &str, name: &str, cats: &[&str], connected: bool) -> Integration {
    Integration {
        id: id.into(),
        name: name.into(),
        vendor: "v".into(),
        description: "desc".into(),
        category_ids: cats.iter().map(|c| c.to_string()).collect(),
        connected,
        logo: rsx_empty(),
        badges: vec![],
        error: None,
    }
}

fn rsx_empty() -> Element {
    rsx! { span {} }
}

#[test]
fn catalog_search_narrows_and_counts() {
    let items = vec![
        integ("slack", "Slack", &["chat"], true),
        integ("gcal", "Calendar", &["calendar"], false),
    ];
    let featured = vec!["slack".to_string()];
    assert_eq!(
        filter_integrations(&items, "all", &featured, "cal", &[]).len(),
        1
    );
    assert_eq!(
        filter_integrations(&items, "featured", &featured, "", &[]).len(),
        1
    );
    assert_eq!(category_total(&items, "chat", &featured), 1);
    assert_eq!(connected_count(&items), 1);
}

fn line(id: &str, total: f64) -> Line {
    Line {
        id: id.into(),
        name: id.into(),
        quantity: 1.0,
        unit_amount: total,
        tax_percent: 0.0,
        total,
        kind: LineKind::Fixed,
    }
}

#[test]
fn invoice_credit_total_excludes_invalid_amounts() {
    let lines = vec![line("a", 100.0), line("b", 50.0)];
    let credited = vec![
        CreditedLine {
            line_id: "a".into(),
            amount: 40.0,
        },
        CreditedLine {
            line_id: "b".into(),
            amount: 60.0,
        },
    ];
    assert!((credit_total(&credited, &lines) - 40.0).abs() < 1e-9);
    assert!(!can_issue(&credited, &lines));
    assert!(can_issue(&full_credit(&lines), &lines));
    assert!(!is_valid_amount(0.0, 10.0));
    assert!(!can_issue(&[], &lines));
}

fn work(id: &str, status: &str) -> WorkItem {
    WorkItem {
        id: id.into(),
        title: id.into(),
        status_id: status.into(),
        priority: 0,
        label_ids: vec![],
        assignee_id: None,
        due_in_days: None,
        subtasks_done: 0,
        subtasks_total: 0,
        pull_request_ref: None,
    }
}

#[test]
fn kanban_moves_between_and_within_columns() {
    let columns = vec![
        StatusColumn {
            id: "todo".into(),
            label: "To do".into(),
        },
        StatusColumn {
            id: "done".into(),
            label: "Done".into(),
        },
    ];
    let items = vec![work("a", "todo"), work("b", "todo"), work("c", "done")];
    let moved = move_to_status(&items, "a", "done").unwrap_or_default();
    assert_eq!(moved.last().map(|i| i.status_id.as_str()), Some("done"));
    assert!(move_column(&items, &columns, "a", -1).is_none());
    assert_eq!(
        move_column(&items, &columns, "a", 1)
            .unwrap_or_default()
            .last()
            .map(|i| i.id.as_str()),
        Some("a")
    );
    let swapped = reorder_in_column(&items, "b", -1).unwrap_or_default();
    assert_eq!(swapped[0].id, "b");
    assert!(reorder_in_column(&items, "a", -1).is_none());
}

#[test]
fn kanban_due_words_and_search() {
    assert_eq!(due_words(0), "due today");
    assert_eq!(due_words(-2), "2 days late");
    assert_eq!(due_words(3), "due in 3 days");
    let items = vec![work("fix-vat", "todo"), work("csv", "todo")];
    assert_eq!(filter_items(&items, "VAT", &[]).len(), 1);
}

#[test]
fn overview_change_and_segments() {
    assert!((percent_change(110.0, 100.0) - 10.0).abs() < 1e-9);
    assert_eq!(percent_change(5.0, 0.0), 0.0);
    assert_eq!(good_or_bad(-1.0, true), "good");
    assert_eq!(good_or_bad(1.0, true), "bad");
    let series = vec![
        OverviewSeries {
            label: "A".into(),
            values: vec![1.0, 2.0],
        },
        OverviewSeries {
            label: "B".into(),
            values: vec![3.0, 4.0],
        },
    ];
    assert_eq!(visible_series(&series, "A").len(), 1);
    assert_eq!(
        overview_totals(&visible_series(&series, "all")),
        vec![4.0, 6.0]
    );
}

#[test]
fn plan_price_and_saving() {
    assert_eq!(yearly_saving_percent(10.0, 96.0), 20);
    assert_eq!(format_price(49.0), "$49");
    assert_eq!(format_price(9.5), "$9.50");
    assert_eq!(billing_for_key(&Key::ArrowRight), Some("yearly"));
    assert_eq!(billing_for_key(&Key::Enter), None);
    let plan = rv2_kit::blocks::plan_comparison_section::Plan {
        id: "p".into(),
        name: "p".into(),
        price_monthly: 10.0,
        price_yearly: 100.0,
        billing_note_monthly: String::new(),
        billing_note_yearly: String::new(),
        cta_label: String::new(),
        cta_href: String::new(),
        highlighted: false,
        badge: None,
        limits: vec![],
        integrations: rv2_kit::blocks::plan_comparison_section::PlanIntegrations {
            names: vec![],
            sync_label: String::new(),
            max_selectable: 0,
        },
        features: vec![],
    };
    assert!((price_for(&plan, "yearly") - 100.0).abs() < 1e-9);
}

fn run_call(id: &str, status: CallStatus, secs: f64) -> RunCall {
    RunCall {
        id: id.into(),
        agent_id: "a".into(),
        kind: CallKind::Tool,
        name: id.into(),
        status,
        duration_secs: secs,
        error_message: None,
    }
}

#[test]
fn run_error_count_matches_error_calls() {
    let calls = vec![
        run_call("x", CallStatus::Error, 1.0),
        run_call("y", CallStatus::Ok, 2.0),
        run_call("z", CallStatus::Error, 3.0),
    ];
    assert_eq!(error_count(&calls), 2);
    assert!((bar_percent(2.0, 4.0) - 50.0).abs() < 1e-9);
    assert_eq!(duration_label(65.0), "1 min 5 s");
}

#[test]
fn run_tree_order_is_depth_first() {
    let agent = |id: &str, parent: Option<&str>| RunAgent {
        id: id.into(),
        name: id.into(),
        model_label: "m".into(),
        status: RunStatus::Completed,
        parent_id: parent.map(Into::into),
        cost_usd: 0.0,
        elapsed_secs: 0.0,
        current_step: None,
    };
    let agents = vec![
        agent("root", None),
        agent("kid", Some("root")),
        agent("grandkid", Some("kid")),
        agent("other", None),
    ];
    assert_eq!(
        tree_order(&agents),
        vec!["root", "kid", "grandkid", "other"]
    );
}

#[test]
fn sales_ranks_largest_first_and_scales() {
    let rows = vec![
        Ranked {
            id: "a".into(),
            name: "a".into(),
            value: 2.0,
        },
        Ranked {
            id: "b".into(),
            name: "b".into(),
            value: 8.0,
        },
    ];
    let sorted = sort_ranked(&rows);
    assert_eq!(sorted[0].id, "b");
    assert_eq!(scaled_widths(&sorted), vec![100.0, 25.0]);
}

#[test]
fn ssr_team_session_has_tabs_log_and_pause() {
    use rv2_kit::blocks::agent_team_session::{AgentTeamSession, ContextSegment, ContextUsage};
    fn app() -> Element {
        rsx! {
            AgentTeamSession {
                task_title: "Billing",
                team_name: "Checkout",
                agents: vec![TeamAgent { id: "dev".into(), name: "Dev".into(), model_label: "m".into(), role: "r".into() }],
                messages: vec![team_msg("a", "dev", Recipients::Everyone)],
                plan_steps: vec![],
                context_usage: ContextUsage { used_tokens: 50.0, limit_tokens: 100.0, segments: vec![ContextSegment { label: "Brief".into(), tokens: 50.0 }] },
                clock_ms: 0,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("role=\"tablist\""));
    assert!(html.contains("role=\"log\""));
    assert!(html.contains("Pause"));
    assert!(html.contains("50% used"));
}

#[test]
fn ssr_timeline_marks_running_span_and_grid() {
    use rv2_kit::blocks::agent_timeline_overview::{
        AgentTimelineOverview, TimelineAgent, TimelineSpan,
    };
    fn app() -> Element {
        rsx! {
            AgentTimelineOverview {
                agents: vec![TimelineAgent { id: "dev".into(), name: "Dev".into(), model_label: "m".into(), status: AgentStatus::Running }],
                spans: vec![TimelineSpan { id: "s".into(), agent_id: "dev".into(), label: "Build".into(), start_at: 0, end_at: None, status: AgentStatus::Running, error_line: None }],
                inbox_items: vec![],
                window_from: 0,
                window_to: 600_000,
                now_ms: 300_000,
                last_seen_at: 0,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("role=\"grid\""));
    assert!(html.contains("data-running=\"true\""));
    assert!(html.contains("aria-label=\"Build, running\""));
}

#[test]
fn ssr_timeline_lane_carries_one_compact_bar_segment_per_span() {
    use rv2_kit::blocks::agent_timeline_overview::{
        AgentTimelineOverview, TimelineAgent, TimelineSpan,
    };
    fn app() -> Element {
        let span = |id: &str, start: u64, end: u64| TimelineSpan {
            id: id.into(),
            agent_id: "dev".into(),
            label: format!("Task {id}"),
            start_at: start,
            end_at: Some(end),
            status: AgentStatus::Done,
            error_line: None,
        };
        rsx! {
            AgentTimelineOverview {
                agents: vec![TimelineAgent { id: "dev".into(), name: "Dev".into(), model_label: "m".into(), status: AgentStatus::Done }],
                spans: vec![span("a", 0, 60_000), span("b", 120_000, 180_000)],
                inbox_items: vec![],
                window_from: 0,
                window_to: 600_000,
                now_ms: 300_000,
                last_seen_at: 0,
            }
        }
    }
    let html = render(app);
    assert_eq!(
        html.matches("class=\"kit-tl__mini-seg\"").count(),
        2,
        "{html}"
    );
    assert_eq!(html.matches("class=\"kit-tl__span\"").count(), 2, "{html}");
    assert_eq!(
        html.matches("class=\"kit-tl__filter\"").count(),
        1,
        "{html}"
    );
}

#[test]
fn ssr_invoice_back_sheet_is_out_of_the_tree() {
    use rv2_kit::blocks::invoice_credit_pair::{
        Invoice, InvoiceCreditPair, InvoiceStatus, Line, LineKind, Party, Side,
    };
    fn app() -> Element {
        let party = || Party {
            name: "P".into(),
            address: vec![],
            registration_id: None,
        };
        rsx! {
            InvoiceCreditPair {
                invoice: Invoice { number: "1".into(), issued_on: "a".into(), due_on: "b".into(), period_from: "c".into(), period_to: "d".into(), currency: "EUR".into(), issuer: party(), recipient: party(), lines: vec![Line { id: "l".into(), name: "x".into(), quantity: 1.0, unit_amount: 5.0, tax_percent: 0.0, total: 5.0, kind: LineKind::Fixed }], tax_percent: None, status: InvoiceStatus::Open },
                active_side: Side::Invoice,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("inert"));
}

#[test]
fn ssr_plan_details_closed_and_billing_radio() {
    use rv2_kit::blocks::plan_comparison_section::{
        Plan, PlanComparisonSection, PlanFeature, PlanIntegrations,
    };
    fn app() -> Element {
        rsx! {
            PlanComparisonSection {
                plans: vec![Plan {
                    id: "p".into(), name: "Team".into(), price_monthly: 10.0, price_yearly: 100.0,
                    billing_note_monthly: "m".into(), billing_note_yearly: "y".into(), cta_label: "Go".into(), cta_href: "#".into(),
                    highlighted: true, badge: Some("Popular".into()), limits: vec![],
                    integrations: PlanIntegrations { names: vec!["Slack".into()], sync_label: "s".into(), max_selectable: 2 },
                    features: vec![PlanFeature { label: "Audit".into(), included: false }],
                }],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("role=\"radiogroup\""));
    assert!(html.contains("aria-label=\"Billing period\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("not included"));
    assert!(html.contains("per month"));
}

#[test]
fn ssr_run_tree_is_a_tree_with_error_count() {
    use rv2_kit::blocks::run_tree_inspector::{RunAgent, RunCall, RunInfo, RunTreeInspector};
    fn app() -> Element {
        rsx! {
            RunTreeInspector {
                run: RunInfo { id: "r".into(), title: "Run".into(), status: RunStatus::Error, duration_secs: 1.0, cost_usd: 0.0, token_count: 0 },
                agents: vec![RunAgent { id: "a".into(), name: "Root".into(), model_label: "m".into(), status: RunStatus::Error, parent_id: None, cost_usd: 0.0, elapsed_secs: 1.0, current_step: None }],
                calls: vec![RunCall { id: "c".into(), agent_id: "a".into(), kind: CallKind::Tool, name: "t".into(), status: CallStatus::Error, duration_secs: 1.0, error_message: None }],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("role=\"tree\""));
    assert!(html.contains("role=\"treeitem\""));
    assert!(html.contains("aria-expanded=\"true\""));
}

#[test]
fn ssr_kanban_columns_and_no_drop_line_at_rest() {
    use rv2_kit::blocks::kanban_task_board::{KanbanTaskBoard, ViewMode};
    fn app() -> Element {
        rsx! {
            KanbanTaskBoard {
                breadcrumb_segments: vec!["Acme".into()],
                status_columns: vec![StatusColumn { id: "todo".into(), label: "To do".into() }],
                work_items: vec![work("a", "todo")],
                initial_view: ViewMode::Compact,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("To do"));
    assert!(html.contains("data-compact=\"true\""));
    assert!(!html.contains("kit-kb__drop"));
    assert!(html.contains("aria-pressed=\"false\""));
}

#[test]
fn ssr_catalog_connect_and_connected_badge() {
    use rv2_kit::blocks::integration_catalog::IntegrationCatalog;
    fn app() -> Element {
        rsx! {
            IntegrationCatalog {
                integrations: vec![integ("slack", "Slack", &["chat"], true), integ("gcal", "Calendar", &["calendar"], false)],
                categories: vec![],
                featured_ids: vec!["slack".into(), "gcal".into()],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("Connected"));
    assert!(html.contains(">Connect<"));
    assert!(html.contains("aria-label=\"Search integrations\""));
}

#[test]
fn ssr_expandable_strip_one_tab_selected() {
    use rv2_kit::blocks::expandable_panel_strip::{ExpandablePanelStrip, StripPanel};
    fn app() -> Element {
        rsx! {
            ExpandablePanelStrip {
                panels: vec![
                    StripPanel { id: "a".into(), title: "Alpha".into(), caption: "c".into(), preview: rsx! { span { "A" } } },
                    StripPanel { id: "b".into(), title: "Beta".into(), caption: "c".into(), preview: rsx! { span { "B" } } },
                ],
            }
        }
    }
    let html = render(app);
    assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
    assert!(html.contains("role=\"tabpanel\""));
}

#[test]
fn ssr_sales_metric_cards_expose_pressed_state() {
    use rv2_kit::blocks::sales_metrics_panel::{SalesMetric, SalesMetricsPanel, TrendPoint};
    fn app() -> Element {
        rsx! {
            SalesMetricsPanel {
                metric_cards: vec![SalesMetric { id: "rev".into(), label: "Revenue".into(), value: "1".into(), delta_percent: 1.0, series: vec![1.0, 2.0] }],
                periods: vec![],
                trend_series: vec![TrendPoint { date: "d".into(), value: 1.0 }],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("aria-label=\"Chart style\""));
}

#[test]
fn ssr_listpicker_closed_has_haspopup_and_no_options() {
    use rv2_kit::blocks::metric_overview_board::{ListPicker, PickOption};
    fn app() -> Element {
        rsx! {
            ListPicker {
                options: vec![PickOption { id: "w".into(), label: "Week".into() }],
                selected: Some("w".to_string()),
                label: "Period",
                on_change: move |_: String| {},
            }
        }
    }
    let html = render(app);
    assert!(html.contains("aria-haspopup=\"listbox\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("role=\"option\""));
}

#[test]
fn ssr_client_workspace_tabs_and_sync_status() {
    use rv2_kit::blocks::client_workspace::{ClientWorkspace, WorkspaceInfo, WorkspaceSection};
    fn app() -> Element {
        rsx! {
            ClientWorkspace {
                workspace: WorkspaceInfo { id: "a".into(), name: "Acme".into(), meta: vec![], synced_at_ms: 0 },
                workspace_options: vec![],
                stat_cards: vec![],
                sections: vec![WorkspaceSection { id: "context".into(), label: "Context".into(), count: None }],
                timeline_items: vec![],
                kind_options: vec![],
                now_ms: 120_000,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("role=\"tablist\""));
    assert!(html.contains("Synced 2 min ago"));
    assert!(html.contains("Nothing here yet"));
}

#[test]
fn ssr_metric_overview_has_pickers_and_focus_card() {
    use rv2_kit::blocks::metric_overview_board::{MetricOverviewBoard, OverviewSeries, PickOption};
    fn app() -> Element {
        rsx! {
            MetricOverviewBoard {
                stat_cards: vec![],
                overview_series: vec![OverviewSeries { label: "A".into(), values: vec![1.0] }],
                axis_labels: vec!["Mon".into()],
                period_options: vec![PickOption { id: "w".into(), label: "Week".into() }],
                segment_options: vec![PickOption { id: "all".into(), label: "All".into() }, PickOption { id: "A".into(), label: "A".into() }],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("aria-label=\"Focused day\""));
    assert!(html.contains("aria-haspopup=\"listbox\""));
}

#[test]
fn forecast_band_geometry_covers_forecast_points() {
    use rv2_kit::blocks::chart_answer_chat::{ForecastPoint, forecast_geometry};
    let points = vec![
        ForecastPoint {
            x_label: "a".into(),
            value: 1.0,
            band_low: None,
            band_high: None,
        },
        ForecastPoint {
            x_label: "b".into(),
            value: 2.0,
            band_low: Some(1.5),
            band_high: Some(2.5),
        },
        ForecastPoint {
            x_label: "c".into(),
            value: 3.0,
            band_low: Some(2.5),
            band_high: Some(3.5),
        },
    ];
    let geo = forecast_geometry(&points, 1);
    assert!(!geo.actual.is_empty());
    assert!(!geo.band.is_empty());
    assert_eq!(geo.forecast.split(' ').count(), 3);
}

#[test]
fn ssr_motion_layers_render_at_rest() {
    use rv2_kit::blocks::kanban_task_board::{KanbanTaskBoard, StatusColumn};
    use rv2_kit::blocks::metric_overview_board::{MetricOverviewBoard, OverviewSeries, PickOption};
    fn kanban() -> Element {
        rsx! {
            KanbanTaskBoard {
                breadcrumb_segments: vec![],
                status_columns: vec![StatusColumn { id: "todo".into(), label: "To do".into() }],
                work_items: vec![],
            }
        }
    }
    fn overview() -> Element {
        rsx! {
            MetricOverviewBoard {
                stat_cards: vec![],
                overview_series: vec![OverviewSeries { label: "A".into(), values: vec![1.0] }],
                axis_labels: vec!["Mon".into()],
                segment_options: vec![PickOption { id: "all".into(), label: "All".into() }, PickOption { id: "A".into(), label: "A".into() }],
            }
        }
    }
    for html in [
        render(kanban),
        render(overview),
        render(sales_breakdown_app),
    ] {
        assert!(!html.contains("data-fading"));
    }
}

#[test]
fn kanban_drop_lands_in_front_of_target_or_at_column_end() {
    use rv2_kit::blocks::kanban_task_board::move_to_position;
    let items = vec![work("a", "todo"), work("b", "doing"), work("c", "doing")];
    let order = |v: &[WorkItem]| v.iter().map(|i| i.id.clone()).collect::<Vec<_>>();
    let front = move_to_position(&items, "a", "doing", Some("c")).unwrap();
    assert_eq!(order(&front), ["b", "a", "c"]);
    assert_eq!(front[1].status_id, "doing");
    let end = move_to_position(&items, "a", "doing", None).unwrap();
    assert_eq!(order(&end), ["b", "c", "a"]);
    assert_eq!(
        move_to_position(&items, "a", "todo", Some("a")).unwrap(),
        items
    );
    assert!(move_to_position(&items, "a", "doing", Some("zz")).is_none());
}

#[test]
fn kanban_radio_arrows_clamp_at_both_ends() {
    use rv2_kit::blocks::kanban_task_board::radio_step;
    assert_eq!(radio_step(0, 3, false), 0);
    assert_eq!(radio_step(1, 3, false), 0);
    assert_eq!(radio_step(1, 3, true), 2);
    assert_eq!(radio_step(2, 3, true), 2);
}

#[test]
fn ssr_kanban_star_is_pressed_only_for_starred_cards() {
    use rv2_kit::blocks::kanban_task_board::KanbanTaskBoard;
    fn app() -> Element {
        rsx! {
            KanbanTaskBoard {
                breadcrumb_segments: vec![],
                status_columns: vec![StatusColumn { id: "todo".into(), label: "To do".into() }],
                work_items: vec![work("a", "todo"), work("b", "todo")],
                starred_ids: vec!["b".into()],
            }
        }
    }
    let html = render(app);
    let stars: Vec<&str> = html.split("kit-kb__star").skip(1).collect();
    assert_eq!(stars.len(), 2, "one star per card");
    let pressed = |chunk: &str| {
        chunk
            .split("aria-label")
            .next()
            .unwrap_or("")
            .contains("aria-pressed=\"true\"")
    };
    assert!(!pressed(stars[0]), "card a is not starred");
    assert!(pressed(stars[1]), "card b is starred");
    assert!(html.contains("aria-label=\"Star b\""));
}

#[test]
fn ssr_kanban_has_no_drop_line_without_a_drag() {
    use rv2_kit::blocks::kanban_task_board::KanbanTaskBoard;
    fn app() -> Element {
        rsx! {
            KanbanTaskBoard {
                breadcrumb_segments: vec![],
                status_columns: vec![StatusColumn { id: "todo".into(), label: "To do".into() }],
                work_items: vec![work("a", "todo")],
            }
        }
    }
    assert!(!render(app).contains("kit-kb__drop"));
}

fn chat_forecast() -> Element {
    use rv2_kit::blocks::chart_answer_chat::{
        AnswerBlock, ChartAnswerChat, ChatMessage, ForecastPoint, Role,
    };
    let point = |x: &str, v: f64, lo: f64, hi: f64| ForecastPoint {
        x_label: x.into(),
        value: v,
        band_low: Some(lo),
        band_high: Some(hi),
    };
    let messages = vec![ChatMessage {
        id: "a1".into(),
        role: Role::Assistant,
        text: Some("Revenue is forecast to reach 12 by W3.".into()),
        blocks: vec![AnswerBlock::Forecast {
            block_id: "f1".into(),
            points: vec![
                point("W1", 10.0, 8.0, 12.0),
                point("W2", 11.0, 8.5, 14.0),
                point("W3", 12.0, 9.0, 15.0),
            ],
            actual_until_index: 2,
        }],
        failed: false,
    }];
    rsx! { ChartAnswerChat { messages: messages } }
}

#[test]
fn ssr_chat_forecast_draws_band_dashed_forecast_and_stagger_index() {
    let html = render(chat_forecast);
    assert!(html.contains("kit-chat__band"), "band polygon missing");
    assert!(
        html.contains("kit-chat__forecast-line"),
        "dashed forecast missing"
    );
    assert!(html.contains("kit-chat__actual"), "actual line missing");
    assert!(
        html.contains("--kit-block: 0;"),
        "block stagger index missing"
    );
    assert!(html.contains("Forecast of 3 points"));
}

#[test]
fn chat_forecast_band_is_absent_when_no_point_has_band_values() {
    use rv2_kit::blocks::chart_answer_chat::{ForecastPoint, forecast_geometry};
    let points = vec![
        ForecastPoint {
            x_label: "W1".into(),
            value: 1.0,
            band_low: None,
            band_high: None,
        },
        ForecastPoint {
            x_label: "W2".into(),
            value: 2.0,
            band_low: None,
            band_high: None,
        },
    ];
    let geometry = forecast_geometry(&points, 1);
    assert!(geometry.band.is_empty());
    assert!(!geometry.forecast.is_empty());
}

#[test]
fn overview_roll_eases_from_old_to_new_and_lands_exactly() {
    use rv2_kit::blocks::metric_overview_board::roll_at;
    assert_eq!(roll_at(0.0, 10.0, 0, 4), 0.0);
    assert_eq!(roll_at(0.0, 10.0, 4, 4), 10.0);
    assert!(roll_at(0.0, 10.0, 1, 4) < roll_at(0.0, 10.0, 2, 4));
    assert!(
        roll_at(0.0, 10.0, 1, 4) > 5.0,
        "ease-out front-loads the change"
    );
    assert_eq!(roll_at(7.5, 7.5, 2, 4), 7.5);
}

fn sales_breakdown_app() -> Element {
    use rv2_kit::blocks::sales_metrics_panel::SalesMetricsPanel;
    rsx! {
        SalesMetricsPanel {
            metric_cards: vec![],
            periods: vec![],
            trend_series: vec![],
            top_products: vec![
                Ranked { id: "p1".into(), name: "Team plan".into(), value: 21_400.0 },
                Ranked { id: "p2".into(), name: "Starter".into(), value: 9_800.0 },
            ],
            channels: vec![Ranked { id: "c1".into(), name: "Direct".into(), value: 18_000.0 }],
        }
    }
}

#[test]
fn ssr_sales_breakdowns_are_tables_with_name_and_value_columns() {
    let html = render(sales_breakdown_app);
    assert_eq!(html.matches("<table").count(), 2, "one table per breakdown");
    assert!(html.contains("scope=\"col\""));
    assert!(html.contains("scope=\"row\""));
    assert!(html.contains("Team plan") && html.contains("Starter") && html.contains("Direct"));
    assert!(
        !html.contains("kit-part"),
        "breakdowns no longer render as part cards"
    );
    let bars = html.matches("kit-sales__bar-fill").count();
    assert_eq!(bars, 3, "one bar per row");
}

#[test]
fn sales_roll_text_keeps_host_format_and_lands_on_target() {
    use rv2_kit::blocks::sales_metrics_panel::{parse_amount, roll_text};
    let amount = parse_amount("$48.2k").expect("parses");
    assert_eq!(
        (
            amount.prefix.as_str(),
            amount.decimals,
            amount.suffix.as_str()
        ),
        ("$", 1, "k")
    );
    assert_eq!(roll_text("$48.2k", "$50.0k", 6, 12), "$49.8k");
    assert_eq!(roll_text("$48.2k", "$50.0k", 12, 12), "$50.0k");
    assert_eq!(roll_text("n/a", "612", 3, 12), "612");
    assert!(parse_amount("none").is_none());
}

#[test]
fn timeline_pan_snaps_to_minutes_and_keeps_length() {
    use rv2_kit::blocks::agent_timeline_overview::pan_window;
    assert_eq!(pan_window(600_000, 2_400_000, 90_000), (660_000, 2_460_000));
    assert_eq!(pan_window(0, 1_800_000, -60_000), (0, 1_800_000));
}

fn plan_step(title: &str, status: StepStatus, started: Option<u64>) -> PlanStep {
    PlanStep {
        id: title.into(),
        title: title.into(),
        owner_id: None,
        status,
        started_at_ms: started,
        finished_at_ms: None,
        blocked_reason: None,
    }
}

#[test]
fn team_tabs_wrap_left_right_and_jump_home_end() {
    use rv2_kit::blocks::agent_team_session::tab_target;
    assert_eq!(tab_target(0, 3, &Key::ArrowLeft), Some(2));
    assert_eq!(tab_target(2, 3, &Key::ArrowRight), Some(0));
    assert_eq!(tab_target(1, 3, &Key::Home), Some(0));
    assert_eq!(tab_target(1, 3, &Key::End), Some(2));
    assert_eq!(tab_target(1, 3, &Key::Enter), None);
    assert_eq!(tab_target(0, 0, &Key::End), None);
}

#[test]
fn team_message_up_down_stop_at_the_ends() {
    use rv2_kit::blocks::agent_team_session::message_target;
    let ids = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(message_target(&ids, "a", &Key::ArrowDown), Some("b".into()));
    assert_eq!(message_target(&ids, "c", &Key::ArrowDown), Some("c".into()));
    assert_eq!(message_target(&ids, "a", &Key::ArrowUp), Some("a".into()));
    assert_eq!(message_target(&ids, "b", &Key::ArrowUp), Some("a".into()));
    assert_eq!(message_target(&ids, "z", &Key::ArrowDown), None);
    assert_eq!(message_target(&ids, "b", &Key::Enter), None);
}

#[test]
fn team_step_live_text_changes_per_minute_not_per_tick() {
    use rv2_kit::blocks::agent_team_session::step_live_text;
    let steps = vec![
        plan_step("Plan", StepStatus::Done, Some(0)),
        plan_step("Build", StepStatus::Running, Some(10_000)),
        plan_step("Ship", StepStatus::Pending, None),
    ];
    assert_eq!(
        step_live_text(&steps, 20_000),
        "Step 2 of 3, Build: running under a minute"
    );
    assert_eq!(
        step_live_text(&steps, 125_000),
        step_live_text(&steps, 129_000)
    );
    assert_eq!(
        step_live_text(&steps, 130_000),
        "Step 2 of 3, Build: running 2 min"
    );
    let all_done = vec![
        plan_step("Plan", StepStatus::Done, Some(0)),
        plan_step("Build", StepStatus::Done, Some(10_000)),
    ];
    assert_eq!(step_live_text(&all_done, 500_000), "All 2 steps done");
    assert_eq!(step_live_text(&[], 0), "");
}

#[test]
fn ssr_team_plan_announces_running_step_in_a_status_region() {
    use rv2_kit::blocks::agent_team_session::{AgentTeamSession, ContextUsage};
    fn app() -> Element {
        rsx! {
            AgentTeamSession {
                task_title: "Billing",
                team_name: "Checkout",
                agents: vec![],
                messages: vec![],
                plan_steps: vec![plan_step("Build", StepStatus::Running, Some(0))],
                context_usage: ContextUsage { used_tokens: 0.0, limit_tokens: 100.0, segments: vec![] },
                clock_ms: 0,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("role=\"status\""));
    assert!(html.contains("Step 1 of 1, Build: running under a minute"));
}

#[test]
fn run_tree_hover_finds_the_parent_of_the_hovered_card() {
    use rv2_kit::blocks::run_tree_inspector::parent_of_hovered;
    let agent = |id: &str, parent: Option<&str>| RunAgent {
        id: id.into(),
        name: id.into(),
        model_label: "m".into(),
        status: RunStatus::Running,
        parent_id: parent.map(Into::into),
        cost_usd: 0.0,
        elapsed_secs: 1.0,
        current_step: None,
    };
    let agents = vec![agent("lead", None), agent("ui", Some("lead"))];
    assert_eq!(parent_of_hovered(&agents, Some("ui")), Some("lead".into()));
    assert_eq!(parent_of_hovered(&agents, Some("lead")), None);
    assert_eq!(parent_of_hovered(&agents, None), None);
    assert_eq!(parent_of_hovered(&agents, Some("gone")), None);
}

#[test]
fn ssr_client_workspace_loading_shows_skeleton_regions() {
    use rv2_kit::blocks::client_workspace::{ClientWorkspace, WorkspaceInfo, WorkspaceSection};
    fn app() -> Element {
        rsx! {
            ClientWorkspace {
                workspace: WorkspaceInfo { id: "a".into(), name: "Acme".into(), meta: vec![], synced_at_ms: 0 },
                workspace_options: vec![],
                stat_cards: vec![],
                sections: vec![WorkspaceSection { id: "context".into(), label: "Context".into(), count: None }],
                timeline_items: vec![],
                kind_options: vec![],
                now_ms: 0,
                is_loading: true,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("aria-label=\"Loading headline numbers\""));
    assert!(html.contains("aria-label=\"Loading timeline\""));
    assert!(html.contains("aria-busy=\"true\""));
    assert!(html.contains("Loading…"));
    assert!(!html.contains("Nothing here yet"));
}

#[test]
fn ssr_catalog_connecting_region_exists_before_and_during_connect() {
    use rv2_kit::blocks::integration_catalog::IntegrationCatalog;
    fn idle() -> Element {
        rsx! {
            IntegrationCatalog {
                integrations: vec![integ("gcal", "Calendar", &["calendar"], false)],
                categories: vec![],
                featured_ids: vec!["gcal".into()],
            }
        }
    }
    fn pending() -> Element {
        rsx! {
            IntegrationCatalog {
                integrations: vec![integ("gcal", "Calendar", &["calendar"], false)],
                categories: vec![],
                featured_ids: vec!["gcal".into()],
                pending_id: Some("gcal".to_string()),
            }
        }
    }
    let idle_html = render(idle);
    assert!(idle_html.contains("role=\"status\""));
    assert!(!idle_html.contains("Connecting Calendar"));
    let pending_html = render(pending);
    assert!(pending_html.contains("Connecting Calendar"));
    assert!(pending_html.contains("Connecting…"));
}

#[test]
fn ssr_plan_yearly_marks_the_pill_and_rolls_the_price() {
    use rv2_kit::blocks::plan_comparison_section::{
        Plan, PlanComparisonSection, PlanFeature, PlanIntegrations,
    };
    fn app(billing: &'static str) -> Element {
        rsx! {
            PlanComparisonSection {
                selected_billing: billing,
                plans: vec![Plan {
                    id: "p".into(), name: "Team".into(), price_monthly: 10.0, price_yearly: 100.0,
                    billing_note_monthly: "m".into(), billing_note_yearly: "y".into(), cta_label: "Go".into(), cta_href: "#".into(),
                    highlighted: false, badge: None, limits: vec![],
                    integrations: PlanIntegrations { names: vec![], sync_label: "s".into(), max_selectable: 0 },
                    features: vec![PlanFeature { label: "Audit".into(), included: true }],
                }],
            }
        }
    }
    let yearly = render(|| app("yearly"));
    assert!(yearly.contains("data-yearly=\"true\""));
    assert!(yearly.contains("kit-pcs__pill"));
    assert!(yearly.contains("$100"));
    let monthly = render(|| app("monthly"));
    assert!(!monthly.contains("data-yearly"));
    assert!(monthly.contains("$10"));
}

#[test]
fn credit_total_of_nothing_is_positive_zero() {
    use rv2_kit::blocks::invoice_credit_pair::credit_total;
    let total = credit_total(&[], &[]);
    assert_eq!(total.to_string(), "0");
    assert_eq!(format!("{total:.2}"), "0.00");
}
