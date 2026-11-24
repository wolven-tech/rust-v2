//! SSR checks for the AI spend chat planner at rest (amendment 5).

use dioxus::prelude::*;
use rv2_kit::components::grounded_reply::SourceEntry;
use rv2_kit::screens::ai_spend_chat_planner::{
    AiSpendChatPlanner, Answer, Chart, Metric, SplitGroup, SplitPart, Turn, group_total,
    split_bar_px, split_part_names,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn part(name: &str, dollars: f64) -> SplitPart {
    SplitPart {
        entry_name: name.into(),
        value_dollars: dollars,
    }
}

fn rest_groups() -> Vec<SplitGroup> {
    vec![
        SplitGroup {
            entry_name: "Atlas".into(),
            parts: vec![part("Large", 2400.0), part("Small", 1100.0)],
        },
        SplitGroup {
            entry_name: "Nimbus".into(),
            parts: vec![
                part("Large", 1800.0),
                part("Mid", 900.0),
                part("Small", 600.0),
            ],
        },
        SplitGroup {
            entry_name: "Quill".into(),
            parts: vec![part("Mid", 1200.0), part("Small", 500.0)],
        },
    ]
}

fn source(id: &str, chip: &str) -> SourceEntry {
    SourceEntry {
        entry_id: id.into(),
        chip_text: chip.into(),
        link_target: None,
        leading_mark: None,
        snippet: None,
    }
}

fn rest_app() -> Element {
    rsx! {
        AiSpendChatPlanner {
            turn_list: vec![
                Turn::Question { id: "q1".into(), text: "Where does the spend go this month?".into() },
                Turn::Answer(Answer {
                    id: "a1".into(),
                    text: "Atlas carries 41 percent of the 8,500 dollars spent this month. Spend is up 15 percent.".into(),
                    metric_cards: vec![
                        Metric { metric_label: "Spend".into(), value: 8500.0, change_percent: Some(15.0) },
                        Metric { metric_label: "Requests".into(), value: 152000.0, change_percent: Some(-2.0) },
                        Metric { metric_label: "Cost per 1,000".into(), value: 55.9, change_percent: Some(18.0) },
                    ],
                    chart_block: Some(Chart::Split {
                        title_text: "Spend by provider and model".into(),
                        previous_total: Some(7400.0),
                        groups: rest_groups(),
                    }),
                    caption_text: "Spend this month by provider and model. Click a bar to ask about it.".into(),
                    source_list: vec![source("s1", "Usage export"), source("s2", "Billing ledger")],
                    follow_up_list: vec![
                        "What would batch jobs save?".into(),
                        "Which requests drive the Large model?".into(),
                        "Compare with last month".into(),
                    ],
                }),
            ],
            starter_questions: Vec::new(),
            on_ask: move |_| {},
        }
    }
}

#[test]
fn rest_state_holds_one_question_and_one_answer() {
    let html = render(rest_app);
    assert_eq!(
        html.matches("class=\"kit-spend__bubble\"").count(),
        1,
        "{html}"
    );
    assert_eq!(html.matches("aria-label=\"Answer\"").count(), 1, "{html}");
    assert!(!html.contains("kit-spend__plan"), "{html}");
}

#[test]
fn answer_reads_text_then_cards_then_chart_then_sources_then_chips() {
    let html = render(rest_app);
    let order = [
        "class=\"kit-spend__text\"",
        "class=\"kit-spend__cards\"",
        "class=\"kit-spend__chart\"",
        "class=\"kit-spend__caption\"",
        "class=\"kit-spend__sourcerow\"",
        "class=\"kit-spend__followups\"",
        "class=\"kit-spend__composer\"",
    ];
    let mut last = 0;
    for marker in order {
        let at = html
            .find(marker)
            .unwrap_or_else(|| panic!("missing {marker}: {html}"));
        assert!(at >= last, "{marker} out of order: {html}");
        last = at;
    }
}

#[test]
fn split_chart_draws_three_bars_with_totals_and_segments() {
    let html = render(rest_app);
    assert_eq!(
        html.matches("class=\"kit-spend__bar\"").count(),
        3,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"kit-spend__bar-total\"").count(),
        3,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"kit-spend__segment\"").count(),
        7,
        "{html}"
    );
    assert!(html.contains("2,400"), "{html}");
    assert!(html.contains("3,300"), "{html}");
    assert!(html.contains("1,700"), "{html}");
}

#[test]
fn rest_state_has_no_data_table_in_the_thread() {
    let html = render(rest_app);
    assert!(!html.contains("<table"), "{html}");
}

#[test]
fn caption_names_the_bar_click_hint() {
    let html = render(rest_app).to_lowercase();
    assert!(html.contains("click a bar to ask about it"), "{html}");
}

#[test]
fn source_row_has_a_label_two_chips_and_copy_and_feedback_icons() {
    let html = render(rest_app);
    assert_eq!(
        html.matches("class=\"kit-spend__source\"").count(),
        2,
        "{html}"
    );
    assert!(html.contains("Sources"), "{html}");
    for label in ["Copy answer", "Helpful", "Not helpful"] {
        assert!(
            html.contains(&format!("aria-label=\"{label}\"")),
            "{label}: {html}"
        );
    }
}

#[test]
fn follow_up_chips_sit_in_one_row_of_three() {
    let html = render(rest_app);
    assert_eq!(
        html.matches("class=\"kit-spend__chip\"").count(),
        3,
        "{html}"
    );
}

#[test]
fn composer_shows_the_approval_mode_control_and_its_label() {
    let html = render(rest_app);
    assert!(html.contains("kit-pib__policy"), "{html}");
    assert!(html.contains("Confirm each"), "{html}");
}

#[test]
fn metric_cards_sit_in_one_row_of_three_labels() {
    let html = render(rest_app);
    for label in ["Spend", "Requests", "Cost per 1,000"] {
        assert!(html.contains(label), "{label}: {html}");
    }
}

#[test]
fn a_bar_total_is_the_sum_of_its_segments() {
    let groups = rest_groups();
    let totals: Vec<f64> = groups.iter().map(group_total).collect();
    assert_eq!(totals, vec![3500.0, 3300.0, 1700.0]);
    assert_eq!(totals.iter().sum::<f64>(), 8500.0);
}

#[test]
fn tallest_bar_reaches_the_track_and_others_scale_to_it() {
    assert_eq!(split_bar_px(3500.0, 3500.0), 150.0);
    assert_eq!(split_bar_px(1750.0, 3500.0), 75.0);
    assert_eq!(split_bar_px(10.0, 0.0), 0.0);
}

#[test]
fn legend_names_each_model_once_in_first_seen_order() {
    assert_eq!(
        split_part_names(&rest_groups()),
        vec!["Large".to_string(), "Small".to_string(), "Mid".to_string()]
    );
}
