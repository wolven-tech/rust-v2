use dioxus::prelude::*;
use rv2_kit::screens::agent_spend_overview::{
    AgentSpendOverview, Breakdown, TopTask, UsageStats, actual_series, forecast_series,
    format_tokens,
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

fn harbor_breakdown() -> Breakdown {
    let weights = [
        0.8, 1.1, 0.9, 1.0, 1.2, 0.7, 0.95, 1.05, 1.0, 0.9, 1.1, 1.0, 0.85, 1.15, 0.95, 1.2, 1.0,
        0.9, 1.05,
    ];
    let weight_total: f64 = weights.iter().sum();
    Breakdown {
        id: "Harbor".to_string(),
        entry_name: "Harbor".to_string(),
        amount: 118.4,
        change_vs_last_month: Some(12.0),
        daily_costs: weights.iter().map(|w| 118.4 * w / weight_total).collect(),
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

fn row(id: &str, amount: f64, change: f64) -> Breakdown {
    Breakdown {
        id: id.to_string(),
        entry_name: id.to_string(),
        amount,
        change_vs_last_month: Some(change),
        daily_costs: vec![],
        top_tasks: vec![],
    }
}

fn rest_app() -> Element {
    let mut projects = vec![
        row("Dune", 82.1, -4.0),
        row("Fern", 46.3, 6.0),
        row("Internal", 23.6, -3.0),
    ];
    projects.insert(0, harbor_breakdown());
    rsx! {
        AgentSpendOverview {
            month_label: "November 2026",
            currency_code: "USD",
            daily_costs: DAILY.to_vec(),
            days_in_month: 30,
            budget_amount: Some(500.0),
            project_breakdown: projects.clone(),
            model_breakdown: vec![
                row("Opus", 124.8, 15.0),
                row("Sonnet", 96.4, -2.0),
                row("Haiku", 31.2, 3.5),
                row("Embeddings", 18.0, -30.0),
            ],
            usage_stats: Some(UsageStats { task_count: Some(186), run_count: Some(1284), token_count: Some(48_200_000) }),
        }
    }
}

#[test]
fn chart_has_nineteen_actual_points_and_a_forecast_from_the_nineteenth_day_to_month_end() {
    let actual = actual_series(&DAILY, 30);
    assert_eq!(actual.iter().filter(|p| p.is_some()).count(), 19);
    let forecast_points = forecast_series(&DAILY, 30);
    assert!(forecast_points[17].is_none());
    assert_eq!(forecast_points[18], actual[18]);
    assert!(forecast_points[29].is_some());
}

#[test]
fn the_nineteen_daily_values_sum_to_the_hero_total() {
    let total: f64 = DAILY.iter().sum();
    assert!((total - 270.4).abs() < 1e-9);
}

#[test]
fn stats_sit_inside_the_hero_in_the_order_tasks_runs_cost_per_task_tokens() {
    let html = render(rest_app);
    let hero = html.find("kit-aso__hero").expect("hero");
    let stats = html.find("class=\"kit-aso__stats\"").expect("stats");
    let tasks = html.find(">Tasks<").expect("tasks tile");
    let runs = html.find(">Runs<").expect("runs tile");
    let cost = html.find(">Cost per task<").expect("cost per task tile");
    let tokens = html.find(">Tokens<").expect("tokens tile");
    assert!(hero < stats);
    assert!(stats < tasks && tasks < runs && runs < cost && cost < tokens);
}

#[test]
fn tokens_read_in_words_of_magnitude() {
    assert_eq!(format_tokens(48_200_000), "48.2M");
    assert_eq!(format_tokens(1_500), "1.5K");
    assert_eq!(format_tokens(999), "999");
}

#[test]
fn panel_at_rest_opens_on_the_first_project_with_its_facts_and_four_top_tasks() {
    let html = render(rest_app);
    assert!(html.contains("aria-label=\"Harbor\""));
    assert!(html.contains("Harbor"));
    assert!(html.contains("$118.40 · 44% of the month"));
    assert!(html.contains("up 12% vs last month"));
    let tasks = html.find("class=\"kit-aso__tasks\"").expect("top tasks");
    assert_eq!(html[tasks..].matches("<li").count(), 4);
}

#[test]
fn ranked_card_shows_one_list_behind_a_projects_and_models_switch() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-aso__row\"").count(), 4);
    assert!(html.contains("Harbor, $118.40"));
    assert!(!html.contains("Opus, $124.80"));
    assert!(html.contains("class=\"kit-aso__switch\""));
}
