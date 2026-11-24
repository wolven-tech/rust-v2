use dioxus::prelude::*;
use rv2_kit::screens::agent_spend_overview::{
    AgentSpendOverview, AttentionItem, AttentionKind, Breakdown, TopTask, UsageStats, axis_days,
    chart_top, day_x_percent,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

const DAILY: [f64; 19] = [
    13.1, 14.8, 12.6, 15.3, 14.2, 13.9, 11.4, 16.8, 14.5, 13.7, 15.0, 12.2, 14.9, 13.3, 12.8, 17.1,
    14.0, 13.6, 17.2,
];

fn row(id: &str, amount: f64, change: f64) -> Breakdown {
    Breakdown {
        id: id.to_string(),
        entry_name: id.to_string(),
        amount,
        change_vs_last_month: Some(change),
        daily_costs: DAILY.iter().map(|d| d * amount / 270.4).collect(),
        top_tasks: vec![
            TopTask {
                entry_title: "Weekly digest".to_string(),
                run_count: 12,
                cost: 41.44,
            },
            TopTask {
                entry_title: "Invoice sync".to_string(),
                run_count: 9,
                cost: 29.6,
            },
            TopTask {
                entry_title: "Lead scoring".to_string(),
                run_count: 7,
                cost: 23.68,
            },
            TopTask {
                entry_title: "Churn report".to_string(),
                run_count: 4,
                cost: 23.68,
            },
        ],
    }
}

fn rest_app() -> Element {
    rsx! {
        AgentSpendOverview {
            month_label: "November 2026",
            currency_code: "USD",
            daily_costs: DAILY.to_vec(),
            days_in_month: 30,
            budget_amount: Some(500.0),
            project_breakdown: vec![
                row("Harbor", 118.4, 12.0),
                row("Dune", 82.1, -4.0),
                row("Fern", 46.3, 6.0),
                row("Internal", 23.6, -3.0),
            ],
            model_breakdown: vec![row("Opus", 124.8, 15.0)],
            attention_items: vec![
                AttentionItem { id: "a1".into(), item_kind: AttentionKind::Spike, text: "Harbor spent 3x its usual on Tuesday".into(), detail_text: Some("Opus runs".into()) },
                AttentionItem { id: "a2".into(), item_kind: AttentionKind::Jump, text: "Fern cost up 21% this week".into(), detail_text: None },
                AttentionItem { id: "a3".into(), item_kind: AttentionKind::Budget, text: "Internal has used 5% of its share".into(), detail_text: None },
            ],
            usage_stats: Some(UsageStats { task_count: Some(186), run_count: Some(1284), token_count: Some(48_200_000) }),
        }
    }
}

#[test]
fn chart_labels_sit_on_days_one_eight_fifteen_twenty_three_and_the_last_day() {
    assert_eq!(axis_days(30), vec![1, 8, 15, 23, 30]);
    assert_eq!(day_x_percent(1, 30), 0.0);
    assert!((day_x_percent(30, 30) - 100.0).abs() < 1e-9);
}

#[test]
fn chart_has_no_y_axis_labels_and_the_budget_is_text_not_a_progress_bar() {
    let html = render(rest_app);
    assert!(!html.contains("role=\"progressbar\""));
    assert!(!html.contains("kit-aso__ylabel"));
    assert!(html.contains("of $500.00"));
    assert!(html.contains("54% of budget"));
    assert_eq!(html.matches("class=\"kit-aso__xlabel\"").count(), 5);
}

#[test]
fn budget_scale_tops_the_chart_at_the_budget() {
    assert_eq!(chart_top(&DAILY, 30, Some(500.0)), 500.0);
}

#[test]
fn hero_card_holds_total_pace_chart_and_stats_in_that_order() {
    let html = render(rest_app);
    let hero = html.find("class=\"kit-aso__hero\"").expect("hero");
    let total = html.find("class=\"kit-aso__total\"").expect("total");
    let pace = html.find("class=\"kit-aso__pace\"").expect("pace");
    let chart = html.find("class=\"kit-aso__chart\"").expect("chart");
    let stats = html.find("class=\"kit-aso__stats\"").expect("stats");
    assert!(hero < total && total < pace && pace < chart && chart < stats);
    assert!(html.contains("On pace for"));
}

#[test]
fn all_four_project_rows_are_listed_with_projects_selected_on_the_ranked_heading() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-aso__row\"").count(), 4);
    let switch = html.find("class=\"kit-aso__switch\"").expect("switch");
    let projects_pressed = html.find("aria-pressed=\"true\"").expect("pressed");
    assert!(switch < projects_pressed);
    assert!(html[projects_pressed..].starts_with("aria-pressed=\"true\""));
    assert!(html.contains("Projects"));
}

#[test]
fn worth_a_look_lists_all_three_items_with_their_open_buttons() {
    let html = render(rest_app);
    assert!(html.contains("Worth a look"));
    assert_eq!(html.matches("class=\"kit-aso__item\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-aso__open\"").count(), 3);
    let css = include_str!("../assets/agent-spend-overview.css");
    assert!(css.contains("grid-auto-rows: 30%;"));
}

#[test]
fn right_panel_at_rest_shows_name_amount_change_nineteen_bars_and_four_tasks() {
    let html = render(rest_app);
    assert!(html.contains("class=\"kit-aso__panel\""));
    assert!(html.contains("$118.40 · 44% of the month"));
    assert!(html.contains("up 12% vs last month"));
    assert_eq!(html.matches("class=\"kit-aso__bar-day\"").count(), 19);
    let tasks = html.find("class=\"kit-aso__tasks\"").expect("tasks");
    assert_eq!(html[tasks..].matches("<li").count(), 4);
}
