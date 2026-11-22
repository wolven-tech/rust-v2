//! Variant test card structure at rest: the gallery demo renders unfolded with its head,
//! metric switch, arm rows and prompt visible without any interaction.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::components::series_trend_chart::SeriesTone;
use rv2_kit::components::variant_test_card::{Arm, Metric, StatusChip, VariantTestCard};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn demo_arm(
    id: &str,
    letter: char,
    name: &str,
    control: bool,
    rate: f64,
    revenue: f64,
    refunds: f64,
) -> Arm {
    Arm {
        arm_id: id.into(),
        arm_letter: letter,
        arm_name: name.into(),
        readings: BTreeMap::from([
            ("conversion".to_string(), rate),
            ("revenue".to_string(), revenue),
            ("refunds".to_string(), refunds),
        ]),
        is_control: control,
        sample_size: Some(1_200),
        confidence: BTreeMap::from([("conversion".to_string(), 0.93)]),
        badge_tone: Some(SeriesTone::Ok),
    }
}

fn pricing_app() -> Element {
    rsx! {
        VariantTestCard {
            test_name: "Pricing page copy",
            arms: vec![
                demo_arm("a", 'A', "Current copy", true, 0.042, 4.10, 2.4),
                demo_arm("b", 'B', "Short copy", false, 0.051, 4.85, 2.1),
                demo_arm("c", 'C', "Benefit-led copy", false, 0.047, 4.40, 2.9),
            ],
            metric_options: vec![
                Metric { metric_id: "conversion".into(), metric_label: "Conversion".into(), format: Some(|v| format!("{:.1}%", v * 100.0)), lower_wins: false },
                Metric { metric_id: "revenue".into(), metric_label: "Revenue".into(), format: Some(|v| format!("${v:.2}")), lower_wins: false },
                Metric { metric_id: "refunds".into(), metric_label: "Refund rate".into(), format: Some(|v| format!("{v:.1}%")), lower_wins: true },
            ],
            starting_metric: Some("conversion".to_string()),
            status_chip: Some(StatusChip { chip_text: "Significant".into(), chip_tone: SeriesTone::Ok }),
            status_aside: Some("Day 9 of 14, 93% confidence".to_string()),
            starting_open: true,
        }
    }
}

#[test]
fn variant_card_renders_unfolded_at_rest_with_rows_and_prompt() {
    let html = render(pricing_app);
    assert!(html.contains("data-open=\"true\""));
    assert!(html.contains("+21.4%"));
    assert!(html.contains("+11.9%"));
    assert!(html.contains("Base"));
    assert!(html.contains("Pick a winner"));
    assert!(!html.contains("Ship winner"));
}

#[test]
fn variant_head_lists_title_chip_note_then_mini_chart_then_leader() {
    let html = render(pricing_app);
    let title = html.find("Pricing page copy").expect("title rendered");
    let chip = html.find("Significant").expect("chip rendered");
    let note = html.find("Day 9 of 14").expect("note rendered");
    let mini = html
        .find("kit-variant__mini\"")
        .expect("mini chart rendered");
    let leader = html.find("Leader: Short copy").expect("leader rendered");
    assert!(title < chip && chip < note && note < mini && mini < leader);
}

#[test]
fn variant_metric_switch_shows_three_segments_with_first_active() {
    let html = render(pricing_app);
    assert!(html.contains("kit-variant__metrics"));
    assert_eq!(html.matches("class=\"kit-variant__metric\"").count(), 3);
    assert!(html.contains("Refund rate"));
    let segment = |label: &str| {
        let at = html.find(label).expect("segment label rendered");
        let start = html[..at].rfind("<button").expect("segment opens");
        html[start..at].to_string()
    };
    assert!(segment(">Conversion<").contains("aria-checked=\"true\""));
    assert!(segment(">Revenue<").contains("aria-checked=\"false\""));
    assert!(segment(">Refund rate<").contains("aria-checked=\"false\""));
}

#[test]
fn variant_longest_bar_belongs_to_highest_conversion_arm() {
    let html = render(pricing_app);
    let short = html.find("Short copy").expect("B row");
    let benefit = html.find("Benefit-led copy").expect("C row");
    let full = html.find("width: 100.0%").expect("a full-width bar");
    assert!(short < full && full < benefit);
    assert_eq!(html.matches("width: 100.0%").count(), 1);
}

#[test]
fn variant_head_name_carries_status_and_leader_and_mini_chart_is_hidden() {
    let html = render(pricing_app);
    assert!(html.contains("aria-label=\"Pricing page copy, Significant, leader Short copy\""));
    let at = html
        .find("kit-variant__mini\"")
        .expect("mini chart rendered");
    let start = html[..at].rfind("<span").expect("mini chart opens");
    let end = at + html[at..].find('>').expect("mini chart tag closes");
    assert!(html[start..end].contains("aria-hidden=\"true\""));
}
