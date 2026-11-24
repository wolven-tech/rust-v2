//! SSR checks for the revenue chat analyst at rest (amendment 5).

use dioxus::prelude::*;
use rv2_kit::screens::revenue_chat_analyst::{
    AnswerBody, ChartBlock, MetricCard, RevenueChatAnalyst, SplitGroup, SplitPart, Turn,
    short_value, split_bar_px, split_share, split_total,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

const THIS_YEAR: [f64; 13] = [
    4.6, 4.9, 4.8, 5.2, 5.1, 5.3, 5.6, 5.5, 5.9, 5.8, 6.2, 6.1, 6.4,
];
const LAST_YEAR: [f64; 13] = [
    4.2, 4.4, 4.4, 4.7, 4.5, 4.8, 4.9, 5.0, 5.1, 5.0, 5.3, 5.2, 5.4,
];

fn weeks() -> Vec<String> {
    (1..=13).map(|w| format!("Wk {w}")).collect()
}

fn region(name: &str, web: f64, partners: f64, direct: f64) -> SplitGroup {
    SplitGroup {
        entry_name: name.into(),
        parts: vec![
            SplitPart {
                entry_name: "Web".into(),
                entry_value: web,
            },
            SplitPart {
                entry_name: "Partners".into(),
                entry_value: partners,
            },
            SplitPart {
                entry_name: "Direct".into(),
                entry_value: direct,
            },
        ],
    }
}

fn at_rest() -> Element {
    let total: f64 = THIS_YEAR.iter().sum::<f64>() * 1000.0;
    rsx! {
        RevenueChatAnalyst {
            chat_list: vec!["Q4 revenue".into()],
            turn_list: vec![
                Turn::Question { id: "q1".into(), text: "How did revenue move over the last 13 weeks?".into(), quoted_focus: None },
                Turn::Answer {
                    id: "a1".into(),
                    body: AnswerBody {
                        text: "Revenue reached 71400 over 13 weeks.".into(),
                        metric_cards: vec![
                            MetricCard { entry_label: "Revenue, 13 weeks".into(), value: total, change_text: Some("+13.5% vs last year".into()) },
                            MetricCard { entry_label: "Orders".into(), value: 412.0, change_text: None },
                            MetricCard { entry_label: "Weekly average".into(), value: total / 13.0, change_text: None },
                        ],
                        chart_block: Some(ChartBlock::Trend {
                            category_labels: weeks(),
                            current_values: THIS_YEAR.to_vec(),
                            previous_values: LAST_YEAR.to_vec(),
                            marked_point: Some(8),
                        }),
                        caption_text: Some("Weekly revenue in thousands, this year against last year.".into()),
                        source_list: vec!["Ledger".into(), "Payments".into()],
                        follow_up_list: Vec::new(),
                    },
                },
                Turn::Question { id: "q2".into(), text: "Split that by region.".into(), quoted_focus: None },
                Turn::Answer {
                    id: "a2".into(),
                    body: AnswerBody {
                        text: "EMEA carries the most growth.".into(),
                        metric_cards: Vec::new(),
                        chart_block: Some(ChartBlock::Split {
                            title_text: "Revenue by region and channel, this quarter".into(),
                            previous_total: Some(58000.0),
                            groups: vec![
                                region("EMEA", 18000.0, 9000.0, 4000.0),
                                region("Americas", 12000.0, 4000.0, 3000.0),
                                region("APAC", 6000.0, 2400.0, 1600.0),
                                region("LATAM", 3000.0, 1200.0, 800.0),
                            ],
                        }),
                        caption_text: Some("Each column adds its three channels, in dollars.".into()),
                        source_list: Vec::new(),
                        follow_up_list: vec!["Show Americas only".into(), "Compare with last year".into(), "Which channel grew most?".into()],
                    },
                },
            ],
            starter_questions: vec!["How did revenue change this quarter?".into(), "Which region grew fastest?".into(), "What will next month look like?".into()],
        }
    }
}

#[test]
fn rest_state_holds_two_answers_three_metric_cards_and_one_chip_row() {
    let html = render(at_rest);
    assert_eq!(html.matches("aria-label=\"Answer\"").count(), 2, "{html}");
    assert_eq!(
        html.matches("class=\"kit-rca__metric\"").count(),
        3,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"kit-rca__followups\"").count(),
        1,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"kit-rca__source\"").count(),
        2,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"kit-rca__suggest\"").count(),
        1,
        "{html}"
    );
}

#[test]
fn metric_card_total_is_the_sum_of_the_weekly_values_in_units() {
    let html = render(at_rest);
    let sum: f64 = THIS_YEAR.iter().sum::<f64>() * 1000.0;
    assert_eq!(short_value(sum), "71.4k");
    assert!(html.contains("71.4k"), "{html}");
}

#[test]
fn area_chart_draws_three_value_labels_and_four_date_labels() {
    let html = render(at_rest);
    assert_eq!(
        html.matches("class=\"kit-trend__tick\"").count(),
        3,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"kit-trend__date\"").count()
            + html
                .matches("class=\"kit-trend__date kit-trend__date--mid\"")
                .count(),
        4,
        "{html}"
    );
    for label in ["Wk 1<", "Wk 7<", "Wk 9<", "Wk 13<"] {
        assert!(html.contains(label), "{label}: {html}");
    }
}

#[test]
fn rest_state_shows_no_no_data_label_on_a_chart_with_values() {
    let html = render(at_rest);
    assert!(!html.contains("No data"), "{html}");
}

#[test]
fn trend_chart_has_one_legend_naming_both_periods() {
    let html = render(at_rest);
    assert_eq!(
        html.matches("class=\"kit-trend__legend\"").count(),
        1,
        "{html}"
    );
    assert!(html.contains("This year"), "{html}");
    assert!(html.contains("Last year"), "{html}");
    assert_eq!(
        html.matches("class=\"kit-rca__legend\"").count(),
        1,
        "{html}"
    );
}

#[test]
fn split_chart_draws_four_region_columns_of_three_parts_each() {
    let html = render(at_rest);
    assert_eq!(html.matches("class=\"kit-rca__col\"").count(), 4, "{html}");
    assert_eq!(html.matches("class=\"kit-rca__seg\"").count(), 12, "{html}");
    for name in ["EMEA", "Americas", "APAC", "LATAM"] {
        assert!(html.contains(name), "{name}: {html}");
    }
}

#[test]
fn suggestion_chips_sit_before_the_composer_and_outside_the_thread() {
    let html = render(at_rest);
    let chips = html.find("class=\"kit-rca__suggest\"").expect("chips");
    let composer = html.find("class=\"kit-rca__composer\"").expect("composer");
    let thread = html.find("class=\"kit-rca__thread\"").expect("thread");
    let foot = html.find("class=\"kit-rca__foot\"").expect("foot");
    assert!(thread < foot, "{html}");
    assert!(chips < composer, "{html}");
    assert!(foot < chips, "{html}");
    assert_eq!(html.matches("class=\"kit-rca__chip\"").count(), 6, "{html}");
}

#[test]
fn split_share_scales_the_tallest_column_to_the_track() {
    let tallest = split_share(31_000.0, 31_000.0);
    assert_eq!(split_bar_px(tallest), 150.0);
    assert_eq!(split_share(0.0, 0.0), 0.0);
}

#[test]
fn split_parts_add_to_the_column_total() {
    let group = region("EMEA", 18000.0, 9000.0, 4000.0);
    assert_eq!(split_total(&group), 31_000.0);
}
