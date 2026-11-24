use dioxus::prelude::*;
use rv2_kit::blocks::agent_fleet_overview::{
    ActivityPoint, AgentFleetOverview, AgentStatus, FleetAgent, Outcome, StatusFilter,
    activity_markers,
};
use rv2_kit::blocks::expandable_panel_strip::{ExpandablePanelStrip, StripPanel, wrap_index};
use rv2_kit::blocks::integration_catalog::{CategoryOption, Integration, IntegrationCatalog};
use rv2_kit::blocks::kanban_task_board::{KanbanTaskBoard, StatusColumn, WorkItem};
use rv2_kit::blocks::metric_overview_board::{
    MetricOverviewBoard, OverviewSeries, PickOption, ProductCard, StatMetric, card_axis,
};
use rv2_kit::blocks::run_tree_inspector::{
    CallKind, CallStatus, RunAgent, RunCall, RunInfo, RunStatus, RunTreeInspector, token_label,
};
use rv2_kit::blocks::sales_metrics_panel::{
    PeriodOption, Ranked, SalesMetric, SalesMetricsPanel, TrendPoint, dollars_compact,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn sales_cards() -> Vec<SalesMetric> {
    vec![
        SalesMetric {
            id: "rev".into(),
            label: "Revenue".into(),
            value: "$48.2k".into(),
            delta_percent: 12.4,
            delta_in_points: false,
            lower_is_better: false,
            series: vec![1.0, 2.0, 3.0],
        },
        SalesMetric {
            id: "orders".into(),
            label: "Orders".into(),
            value: "612".into(),
            delta_percent: -3.1,
            delta_in_points: false,
            lower_is_better: false,
            series: vec![1.0, 2.0, 3.0],
        },
        SalesMetric {
            id: "aov".into(),
            label: "Average order".into(),
            value: "$78.80".into(),
            delta_percent: 2.2,
            delta_in_points: false,
            lower_is_better: false,
            series: vec![1.0, 2.0, 3.0],
        },
        SalesMetric {
            id: "refunds".into(),
            label: "Refund rate".into(),
            value: "1.8%".into(),
            delta_percent: -0.3,
            delta_in_points: true,
            lower_is_better: true,
            series: vec![1.0, 2.0, 3.0],
        },
    ]
}

#[test]
fn sales_metric_cards_sit_in_order_revenue_orders_average_refund() {
    fn app() -> Element {
        rsx! {
            SalesMetricsPanel {
                metric_cards: sales_cards(),
                periods: vec![],
                trend_series: vec![TrendPoint { date: "Nov 1".into(), value: 1.0 }],
            }
        }
    }
    let html = render(app);
    let at = |label: &str| html.find(&format!("aria-label=\"{label}\"")).expect(label);
    assert!(at("Revenue") < at("Orders"));
    assert!(at("Orders") < at("Average order"));
    assert!(at("Average order") < at("Refund rate"));
}

#[test]
fn sales_refund_delta_reads_in_points_with_a_direction_word() {
    fn app() -> Element {
        rsx! {
            SalesMetricsPanel {
                metric_cards: sales_cards(),
                periods: vec![],
                trend_series: vec![TrendPoint { date: "Nov 1".into(), value: 1.0 }],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("0.3 pts"));
    assert!(html.contains("down vs last period"));
}

#[test]
fn sales_period_is_a_segmented_control_of_week_month_quarter() {
    fn app() -> Element {
        rsx! {
            SalesMetricsPanel {
                metric_cards: vec![],
                periods: vec![
                    PeriodOption { id: "week".into(), label: "Week".into() },
                    PeriodOption { id: "month".into(), label: "Month".into() },
                    PeriodOption { id: "quarter".into(), label: "Quarter".into() },
                ],
                selected_period: Some("month".to_string()),
                trend_series: vec![],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("aria-label=\"Period\""));
    for label in ["Week", "Month", "Quarter"] {
        assert!(html.contains(&format!(">{label}<")), "{label} segment");
    }
}

#[test]
fn sales_trend_value_axis_shows_dollars_at_or_above_the_peak() {
    fn app() -> Element {
        rsx! {
            SalesMetricsPanel {
                metric_cards: sales_cards(),
                periods: vec![],
                trend_series: [1800.0, 3620.0, 2000.0]
                    .iter()
                    .enumerate()
                    .map(|(i, v)| TrendPoint { date: format!("Nov {}", i + 1), value: *v })
                    .collect(),
            }
        }
    }
    let html = render(app);
    assert!(
        html.contains("$4k"),
        "top label 4k is at or above the 3,620 peak"
    );
    assert!(!html.contains("$5k"));
    assert!(html.contains(">Line<") && html.contains(">Bars<"));
}

#[test]
fn sales_breakdown_columns_read_revenue_and_list_five_rows() {
    fn app() -> Element {
        let rows: Vec<Ranked> = (1..=5)
            .map(|i| Ranked {
                id: format!("p{i}"),
                name: format!("Item {i}"),
                value: 1000.0 * i as f64,
            })
            .collect();
        rsx! {
            SalesMetricsPanel {
                metric_cards: vec![],
                periods: vec![],
                trend_series: vec![],
                top_products: rows.clone(),
                channels: rows,
            }
        }
    }
    let html = render(app);
    assert_eq!(
        html.matches("<th scope=\"col\" class=\"kit-sales__num\">Revenue</th>")
            .count(),
        2
    );
    assert_eq!(html.matches("kit-sales__bar-fill").count(), 10);
    assert!(
        html.find("Item 5").unwrap() < html.find("Item 1").unwrap(),
        "largest row first"
    );
}

#[test]
fn sales_dollars_compact_formats_thousands_with_a_dollar_sign() {
    assert_eq!(dollars_compact(48_200.0), "$48.2k");
    assert_eq!(dollars_compact(4_000.0), "$4k");
}

#[test]
fn overview_card_axis_keeps_only_first_and_last_date_labels() {
    let labels: Vec<String> = (1..=5).map(|d| format!("D{d}")).collect();
    assert_eq!(
        card_axis(5, &labels),
        vec!["D1", "", "", "", "D5"]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
    );
}

#[test]
fn overview_lower_is_better_footer_says_good_for_a_fall() {
    fn app() -> Element {
        rsx! {
            MetricOverviewBoard {
                stat_cards: vec![StatMetric {
                    id: "refunds".into(),
                    label: "Refunds".into(),
                    total: 107.0,
                    format_total: rv2_kit::components::series_trend_chart::group_digits,
                    current_series: vec![40.0, 67.0],
                    previous_series: vec![60.0, 84.0],
                    lower_is_better: true,
                }],
                overview_series: vec![OverviewSeries { label: "A".into(), values: vec![1.0, 2.0] }],
                axis_labels: vec!["1 Nov".into(), "Today".into()],
                segment_options: vec![PickOption { id: "all".into(), label: "All items".into() }, PickOption { id: "A".into(), label: "A".into() }],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("down, good vs Previous period"));
    assert!(html.contains("data-verdict=\"good\""));
}

#[test]
fn overview_product_card_shows_figure_comparisons_and_both_actions() {
    fn app() -> Element {
        rsx! {
            MetricOverviewBoard {
                stat_cards: vec![],
                overview_series: vec![OverviewSeries { label: "A".into(), values: vec![1.0, 2.0] }],
                axis_labels: vec!["1 Nov".into(), "Today".into()],
                product_card: Some(ProductCard {
                    name: "Desk license".into(),
                    tag: "One-time".into(),
                    figure_label: "Seats sold".into(),
                    figure_value: "438".into(),
                    delta_percent: 6.3,
                    series: vec![412.0, 438.0],
                    comparisons: vec![("Last period".into(), "412".into()), ("This period".into(), "438".into())],
                }),
                setup_action: ("Set up".to_string(), EventHandler::new(|_| {})),
                footer_action: ("See full analytics".to_string(), EventHandler::new(|_| {})),
            }
        }
    }
    let html = render(app);
    for text in [
        "Desk license",
        "One-time",
        "Seats sold",
        "up 6.3%",
        "Last period",
        "Set up",
        "See full analytics",
    ] {
        assert!(html.contains(text), "{text}");
    }
}

fn work_item(
    id: &str,
    identifier: &str,
    status: &str,
    assignee: Option<&str>,
    pr: Option<&str>,
) -> WorkItem {
    WorkItem {
        id: id.into(),
        identifier_line: identifier.into(),
        title: format!("Task {id}"),
        status_id: status.into(),
        priority: 2,
        label_ids: vec![],
        assignee_id: assignee.map(Into::into),
        due_in_days: None,
        subtasks_done: 0,
        subtasks_total: 0,
        pull_request_ref: pr.map(Into::into),
    }
}

#[test]
fn kanban_board_lists_four_columns_with_identifier_lines_and_neutral_avatars() {
    fn app() -> Element {
        let columns = ["todo", "review", "progress", "done"]
            .iter()
            .map(|id| StatusColumn {
                id: id.to_string(),
                label: id.to_string(),
            })
            .collect::<Vec<_>>();
        rsx! {
            KanbanTaskBoard {
                breadcrumb_segments: vec!["Meridian".into(), "Storefront".into(), "Sprint 9".into()],
                board_name: "Storefront",
                status_columns: columns,
                work_items: vec![
                    work_item("ST-214", "ST-214 · Size M · Created 3 Oct", "todo", None, None),
                    work_item("ST-188", "ST-188 · Created 25 Sep", "review", None, Some("#412")),
                ],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("2 tasks"));
    assert_eq!(
        html.matches("kit-kb__avatar--none").count(),
        2,
        "unassigned cards show a neutral avatar"
    );
    assert!(html.contains("ST-214 · Size M · Created 3 Oct"));
}

#[test]
fn kanban_pull_request_reference_shows_its_number_as_text() {
    fn app() -> Element {
        rsx! {
            KanbanTaskBoard {
                breadcrumb_segments: vec![],
                status_columns: vec![StatusColumn { id: "review".into(), label: "In review".into() }],
                work_items: vec![work_item("ST-188", "ST-188", "review", None, Some("#412"))],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("PR #412"));
}

fn integration(id: &str, name: &str, categories: &[&str], connected: bool) -> Integration {
    Integration {
        id: id.into(),
        name: name.into(),
        vendor: "Vendor".into(),
        description: "Description".into(),
        category_ids: categories.iter().map(|c| c.to_string()).collect(),
        connected,
        logo: rsx! { span { "L" } },
        badges: vec![],
        error: None,
    }
}

#[test]
fn integration_catalog_at_rest_counts_essentials_and_discovers_the_rest() {
    fn app() -> Element {
        let mut items: Vec<Integration> = (0..8)
            .map(|i| {
                integration(
                    &format!("e{i}"),
                    &format!("Essential {i}"),
                    &["crm"],
                    i == 0,
                )
            })
            .collect();
        items.extend((0..14).map(|i| {
            integration(
                &format!("x{i}"),
                &format!("Extra {i}"),
                &["analytics"],
                false,
            )
        }));
        rsx! {
            IntegrationCatalog {
                integrations: items,
                categories: vec![CategoryOption { id: "analytics".into(), label: "Analytics".into() }],
                featured_ids: (0..8).map(|i| format!("e{i}")).collect(),
            }
        }
    }
    let html = render(app);
    assert!(html.contains("1 connected"));
    assert!(html.contains("Essentials"));
    assert!(html.contains("All apps"));
    assert!(html.contains("Discover more"));
    assert!(html.contains("+14 apps"));
    assert_eq!(
        html.matches("kit-catalog__card\"").count(),
        8,
        "eight essential cards"
    );
    assert!(html.contains("By Vendor"));
    assert!(html.contains("aria-label=\"Verified\""));
}

#[test]
fn run_tree_header_counts_agents_and_running_ones() {
    fn app() -> Element {
        let agent = |id: &str, status: RunStatus, parent: Option<&str>| RunAgent {
            id: id.into(),
            name: id.into(),
            model_label: "m".into(),
            status,
            parent_id: parent.map(Into::into),
            cost_usd: 0.1,
            elapsed_secs: 10.0,
            current_step: None,
            faded: false,
        };
        rsx! {
            RunTreeInspector {
                run: RunInfo {
                    id: "r".into(),
                    title: "Run".into(),
                    status: RunStatus::Running,
                    duration_secs: 520.0,
                    cost_usd: 0.31,
                    token_count: 96_000,
                    model_call_count: 4,
                    tool_call_count: 6,
                },
                agents: vec![
                    agent("planner", RunStatus::Running, None),
                    agent("budget", RunStatus::Running, None),
                    agent("writer", RunStatus::Waiting, Some("planner")),
                ],
                calls: vec![RunCall {
                    id: "c".into(),
                    agent_id: "planner".into(),
                    kind: CallKind::Tool,
                    name: "Search".into(),
                    status: CallStatus::Error,
                    duration_secs: 2.0,
                    error_message: None,
                }],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("3 agents · 2 running"));
    assert!(html.contains("96k"));
    assert!(
        html.contains("Waiting"),
        "the writer card shows a waiting status"
    );
    assert!(
        html.contains("kit-rti__status-mark"),
        "call rows carry a status mark"
    );
    assert_eq!(token_label(96_000), "96k");
}

#[test]
fn expandable_strip_shows_seven_tiles_with_the_open_one_as_a_tabpanel() {
    fn app() -> Element {
        let names = ["One", "Two", "Three", "Four", "Five", "Six", "Seven"];
        let panels = names
            .iter()
            .map(|n| StripPanel {
                id: n.to_string(),
                title: n.to_string(),
                caption: format!("{n} caption"),
                preview: rsx! { span { "preview" } },
            })
            .collect();
        rsx! {
            ExpandablePanelStrip {
                panels: panels,
                heading: "Specialist agents, one at a time".to_string(),
                intro: "Each agent has its own steps, tools and limits.".to_string(),
                initial_open_id: "Four".to_string(),
            }
        }
    }
    let html = render(app);
    assert_eq!(
        html.matches("role=\"tab\"").count(),
        6,
        "six collapsed tiles"
    );
    assert_eq!(html.matches("role=\"tabpanel\"").count(), 1);
    assert!(html.contains("Specialist agents, one at a time"));
    assert!(
        html.contains("data-peek"),
        "neighbours of the open card are marked"
    );
    assert!(
        html.contains("<strong>Four</strong>"),
        "the caption row names the open agent in bold"
    );
}

#[test]
fn expandable_strip_next_on_the_last_tile_wraps_to_the_first() {
    assert_eq!(wrap_index(7, 6, 1), 0);
    assert_eq!(wrap_index(7, 0, -1), 6);
}

fn fleet_agent(id: &str, parent: Option<&str>, status: AgentStatus, cost: f64) -> FleetAgent {
    FleetAgent {
        id: id.into(),
        name: id.into(),
        model_label: "m".into(),
        parent_id: parent.map(Into::into),
        status,
        cost_usd: cost,
        elapsed_secs: 60,
        current_task: "task".into(),
    }
}

fn fleet_fixture_agents() -> Vec<FleetAgent> {
    vec![
        fleet_agent("planner", None, AgentStatus::Running, 1.42),
        fleet_agent("support", None, AgentStatus::Running, 1.02),
        fleet_agent("clerk", None, AgentStatus::Waiting, 0.05),
        fleet_agent("research", Some("planner"), AgentStatus::Done, 0.31),
        fleet_agent("drafter", Some("planner"), AgentStatus::Running, 0.22),
        fleet_agent("publisher", Some("planner"), AgentStatus::Waiting, 0.08),
        fleet_agent("reply", Some("support"), AgentStatus::Failed, 0.12),
    ]
}

#[test]
fn fleet_summary_line_and_filter_counts_match_the_agents() {
    fn app() -> Element {
        rsx! {
            AgentFleetOverview {
                agents: fleet_fixture_agents(),
                inbox_items: vec![],
                activity: vec![],
                last_seen_at: 0,
                now_at: 7_200_000,
                heading_text: "Mission control",
            }
        }
    }
    let html = render(app);
    assert!(html.contains("3 leads · 4 sub-agents · 3 running · 2 waiting on you · $3.22"));
    for label in ["All 7", "Running 3", "Needs you 2", "Failed 1"] {
        assert!(html.contains(label), "{label}");
    }
    assert!(html.contains("Since you looked"));
}

#[test]
fn fleet_activity_has_one_marker_per_event_with_its_outcome() {
    let agents = fleet_fixture_agents();
    let activity = vec![
        ActivityPoint {
            at: 100,
            agent_id: "research".into(),
            outcome: Outcome::Done,
        },
        ActivityPoint {
            at: 200,
            agent_id: "clerk".into(),
            outcome: Outcome::NeedsYou,
        },
        ActivityPoint {
            at: 300,
            agent_id: "reply".into(),
            outcome: Outcome::Failed,
        },
    ];
    let markers = activity_markers(&activity, &agents, StatusFilter::default(), 0, 1000);
    assert_eq!(markers.len(), 3);
    assert_eq!(markers[1].2.word(), "needs you");
    assert!(markers[0].0 < markers[2].0, "markers run in time order");
}
