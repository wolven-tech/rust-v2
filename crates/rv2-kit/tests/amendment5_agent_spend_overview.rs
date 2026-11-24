use dioxus::prelude::*;
use rv2_kit::screens::agent_spend_overview::{
    AgentSpendOverview, AttentionItem, AttentionKind, Breakdown, UsageStats, change_words,
    forecast, hero_sentence, sorted_by_amount,
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
        daily_costs: vec![],
        top_tasks: vec![],
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
            model_breakdown: vec![
                row("Opus", 124.8, 15.0),
                row("Sonnet", 96.4, -2.0),
                row("Haiku", 31.2, 3.5),
                row("Embeddings", 18.0, -30.0),
            ],
            attention_items: vec![
                AttentionItem { id: "a1".to_string(), item_kind: AttentionKind::Spike, text: "Spike".to_string(), detail_text: Some("detail".to_string()) },
                AttentionItem { id: "a2".to_string(), item_kind: AttentionKind::Jump, text: "Jump".to_string(), detail_text: Some("detail".to_string()) },
                AttentionItem { id: "a3".to_string(), item_kind: AttentionKind::Budget, text: "Budget".to_string(), detail_text: Some("detail".to_string()) },
            ],
            usage_stats: Some(UsageStats { task_count: Some(186), run_count: Some(1284), token_count: Some(48_200_000) }),
        }
    }
}

#[test]
fn month_is_under_budget_at_rest_so_the_sentence_names_no_passed_day() {
    let total: f64 = DAILY.iter().sum();
    let fc = forecast(&DAILY, 30, Some(500.0));
    assert!(total < 500.0);
    let sentence = hero_sentence(&fc, Some(500.0), "USD");
    assert!(sentence.starts_with("On pace for"), "{sentence}");
    assert!(sentence.contains("under the"), "{sentence}");
    assert!(!sentence.contains("day"));
}

#[test]
fn breakdown_rows_sort_descending_and_each_change_has_a_sign_word() {
    let rows = vec![
        row("a", 10.0, 1.0),
        row("b", 30.0, -2.0),
        row("c", 20.0, 0.0),
    ];
    let sorted: Vec<String> = sorted_by_amount(&rows).into_iter().map(|r| r.id).collect();
    assert_eq!(sorted, vec!["b", "c", "a"]);
    assert!(change_words(8.0).starts_with("up"));
    assert!(change_words(-4.5).starts_with("down"));
    assert_eq!(change_words(0.0), "no change");
}

#[test]
fn worth_a_look_at_rest_shows_three_items_each_with_an_open_button() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-aso__open\"").count(), 3);
    assert_eq!(html.matches("class=\"kit-aso__item\"").count(), 3);
}

#[test]
fn chart_labels_never_read_no_data_when_the_chart_has_values() {
    let html = render(rest_app);
    assert!(!html.contains("No data"));
}
