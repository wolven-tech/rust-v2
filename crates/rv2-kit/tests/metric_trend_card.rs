//! Metric trend card: the structure amendment's acceptance checks, as SSR renders and pure logic.

use dioxus::prelude::*;
use rv2_kit::components::metric_trend_card::{
    MetricSeries, MetricTrendCard, change_chip, ranked_indices,
};
use rv2_kit::components::series_trend_chart::{SeriesTone, compact_number};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The gallery's formatter: whole figures compact, fractions to one decimal.
fn figure(n: f64) -> String {
    if n.fract() == 0.0 {
        compact_number(n)
    } else {
        format!("{n:.1}")
    }
}

fn days() -> Vec<String> {
    ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .map(str::to_string)
        .to_vec()
}

fn three_series() -> Vec<MetricSeries> {
    vec![
        MetricSeries {
            series_name: "Trials".into(),
            series_tone: SeriesTone::Ink,
            series_values: vec![120.0, 150.0, 140.0, 180.0, 170.0, 200.0, 210.0],
        },
        MetricSeries {
            series_name: "Upgrades".into(),
            series_tone: SeriesTone::Ok,
            series_values: vec![40.0, 52.0, 44.0, 60.0, 58.0, 66.0, 72.0],
        },
        MetricSeries {
            series_name: "Renewals".into(),
            series_tone: SeriesTone::Warn,
            series_values: vec![20.0, 24.0, 30.0, 28.0, 33.0, 35.0, 41.0],
        },
    ]
}

fn tagged(n: f64) -> String {
    format!("q{n}")
}

fn instance_a() -> Element {
    rsx! {
        MetricTrendCard {
            metric_name: "New accounts",
            headline_value: "1,284",
            change_value: 8.1,
            change_suffix: "%",
            axis_labels: days(),
            data_series: three_series(),
            format_value: figure as fn(f64) -> String,
            footer_text: Some("Refreshed from the nightly load.".to_string()),
        }
    }
}

fn instance_b() -> Element {
    rsx! {
        MetricTrendCard {
            metric_name: "New accounts",
            headline_value: "1,284",
            change_value: -3.2,
            change_suffix: "%",
            axis_labels: days(),
            data_series: three_series(),
            format_value: figure as fn(f64) -> String,
        }
    }
}

fn instance_c() -> Element {
    rsx! {
        MetricTrendCard {
            metric_name: "New accounts",
            headline_value: "0",
            change_value: 0.0,
            change_suffix: "%",
            axis_labels: days(),
            data_series: Vec::<MetricSeries>::new(),
        }
    }
}

fn tagged_app() -> Element {
    rsx! {
        MetricTrendCard {
            metric_name: "Traffic",
            headline_value: "1.2k",
            change_value: 4.0,
            axis_labels: days(),
            data_series: three_series(),
            format_value: tagged as fn(f64) -> String,
        }
    }
}

fn long_heading_app() -> Element {
    rsx! {
        MetricTrendCard {
            metric_name: "Traffic",
            headline_value: "1.2k",
            change_value: 4.0,
            axis_labels: days(),
            tooltip_labels: (1..=7).map(|d| format!("Day {d} of the week")).collect::<Vec<_>>(),
            data_series: three_series(),
        }
    }
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

#[test]
fn trend_card_axis_shows_first_middle_and_last_labels_only() {
    let html = render(instance_a);
    assert_eq!(count(&html, "class=\"kit-metric-card__date\""), 3);
    assert!(html.contains(">Mon<") && html.contains(">Thu<") && html.contains(">Sun<"));
    assert!(!html.contains(">Tue<"));
}

#[test]
fn trend_card_at_rest_draws_every_series_and_opens_the_tooltip() {
    let html = render(instance_a);
    assert_eq!(count(&html, "class=\"kit-metric-card__line\""), 3);
    assert_eq!(count(&html, "class=\"kit-metric-card__tooltip\""), 1);
    for (name, value) in [
        ("Trials", ">210<"),
        ("Upgrades", ">72<"),
        ("Renewals", ">41<"),
    ] {
        assert!(html.contains(name), "missing series name {name}");
        assert!(html.contains(value), "missing value {value}");
    }
}

#[test]
fn trend_card_footer_sits_after_the_chart_body() {
    let html = render(instance_a);
    let body = html.find("kit-metric-card__body").expect("chart body");
    let footer = html.find("kit-metric-card__footer").expect("footer");
    assert!(body < footer);
    assert!(html.contains("Refreshed from the nightly load."));
}

fn ranked_app() -> Element {
    rsx! {
        MetricTrendCard {
            metric_name: "Mix",
            headline_value: "1",
            change_value: 0.0,
            axis_labels: days(),
            data_series: vec![
                MetricSeries {
                    series_name: "Small".into(),
                    series_tone: SeriesTone::Ink,
                    series_values: vec![1.0; 7],
                },
                MetricSeries {
                    series_name: "Large".into(),
                    series_tone: SeriesTone::Ok,
                    series_values: vec![9.0; 7],
                },
            ],
        }
    }
}

#[test]
fn trend_card_tooltip_rows_rank_series_by_value_with_no_second_readout() {
    let html = render(ranked_app);
    let tooltip = html.find("kit-metric-card__tooltip").expect("tooltip");
    let large = html.find(">Large<").expect("large row");
    let small = html.find(">Small<").expect("small row");
    assert!(tooltip < large && large < small);
    assert_eq!(count(&html, "kit-metric-card__ranked"), 0);
}

#[test]
fn ranked_indices_orders_by_value_and_puts_missing_last() {
    assert_eq!(
        ranked_indices(&[Some(20.0), Some(72.0), Some(41.0)]),
        vec![1, 2, 0]
    );
    assert_eq!(
        ranked_indices(&[None, Some(2.0), Some(5.0), Some(2.0)]),
        vec![2, 1, 3, 0]
    );
}

#[test]
fn change_chip_signs_the_change_and_picks_its_tone() {
    assert_eq!(change_chip(8.1), ("+", "ok"));
    assert_eq!(change_chip(-3.2), ("-", "fault"));
    assert_eq!(change_chip(0.0), ("", "muted"));
}

#[test]
fn trend_card_negative_change_renders_minus_sign_and_error_tone() {
    let html = render(instance_b);
    assert!(html.contains("-3.2%"));
    assert!(html.contains("data-tone=\"fault\""));
    assert!(html.contains("down 3.2% vs previous period"));
}

#[test]
fn trend_card_positive_change_renders_plus_sign_and_success_tone() {
    let html = render(instance_a);
    assert!(html.contains("+8.1%"));
    assert!(html.contains("data-tone=\"ok\""));
}

#[test]
fn trend_card_headline_figure_precedes_its_change_chip() {
    let html = render(instance_a);
    let figure = html.find(">1,284<").expect("headline figure");
    let chip = html.find(">+8.1%<").expect("change chip");
    assert!(figure < chip);
}

#[test]
fn trend_card_without_values_shows_empty_message_and_no_tooltip() {
    let html = render(instance_c);
    assert!(html.contains("kit-metric-card__empty"));
    assert!(html.contains("No data yet"));
    assert_eq!(count(&html, "class=\"kit-metric-card__line\""), 0);
    assert_eq!(count(&html, "kit-metric-card__tooltip"), 0);
    assert_eq!(count(&html, "kit-metric-card__ranked"), 0);
}

#[test]
fn trend_card_format_value_formats_the_chip_and_tooltip() {
    let html = render(tagged_app);
    assert!(html.contains("+q4"));
    assert!(html.contains(">q210<"));
}

#[test]
fn trend_card_tooltip_heading_uses_tooltip_labels() {
    let html = render(long_heading_app);
    assert!(html.contains("Day 7 of the week"));
}
