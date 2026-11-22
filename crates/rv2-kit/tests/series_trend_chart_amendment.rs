//! Series trend chart amendment checks, on the gallery's seven-week data.

use dioxus::prelude::*;
use rv2_kit::components::series_trend_chart::{
    ChartVariant, SeriesTone, SeriesTrendBlock, SeriesTrendChart, SparkTile, TrendSeries,
    area_path, line_path, project_x, project_y,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn week_labels() -> Vec<String> {
    (1..=7).map(|w| format!("W{w}")).collect()
}

fn gallery_series() -> Vec<TrendSeries> {
    vec![
        TrendSeries {
            name: "Completed".into(),
            values: vec![
                Some(12.0),
                Some(18.0),
                Some(15.0),
                None,
                Some(24.0),
                Some(28.0),
                Some(31.0),
            ],
            tone: SeriesTone::Series1,
            emphasis: true,
            is_earlier: false,
        },
        TrendSeries {
            name: "Opened".into(),
            values: vec![
                Some(14.0),
                Some(11.0),
                Some(17.0),
                Some(16.0),
                Some(13.0),
                Some(15.0),
                Some(19.0),
            ],
            tone: SeriesTone::Muted,
            emphasis: false,
            is_earlier: true,
        },
    ]
}

fn line_app() -> Element {
    rsx! {
        SeriesTrendChart {
            x_labels: week_labels(),
            series: gallery_series(),
            variant: ChartVariant::Line,
            label: "Tickets per week",
            initial_active: Some(6),
        }
    }
}

fn bar_app() -> Element {
    rsx! {
        SeriesTrendChart {
            x_labels: week_labels(),
            series: gallery_series(),
            variant: ChartVariant::Bar,
            label: "Tickets per week, columns",
            initial_active: Some(6),
        }
    }
}

fn sparkline_app() -> Element {
    rsx! {
        SeriesTrendChart {
            x_labels: week_labels(),
            series: gallery_series(),
            variant: ChartVariant::Sparkline,
            label: "Completed, last seven weeks",
            initial_active: Some(6),
        }
    }
}

fn line_unanchored_app() -> Element {
    rsx! {
        SeriesTrendChart {
            x_labels: week_labels(),
            series: gallery_series(),
            variant: ChartVariant::Line,
            label: "Tickets per week, unanchored",
            from_zero: false,
        }
    }
}

fn bar_unanchored_app() -> Element {
    rsx! {
        SeriesTrendChart {
            x_labels: week_labels(),
            series: gallery_series(),
            variant: ChartVariant::Bar,
            label: "Tickets per week, columns unanchored",
            from_zero: false,
        }
    }
}

#[test]
fn completed_line_lifts_the_pen_at_the_week_four_gap() {
    let values = &gallery_series()[0].values;
    let points: Vec<Option<(f64, f64)>> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.map(|v| {
                (
                    project_x(i, values.len(), 4.0, 632.0),
                    project_y(v, 0.0, 35.0, 8.0, 200.0),
                )
            })
        })
        .collect();
    let d = line_path(&points);
    assert_eq!(d.matches('M').count(), 2, "one pen lift at W4: {d}");
    assert!(points[3].is_none());
}

#[test]
fn resting_line_highlights_the_last_week_without_a_tooltip() {
    let html = render(line_app);
    assert!(html.contains("kit-trend__guide"));
    assert!(html.contains("kit-trend__marker"));
    assert!(!html.contains("kit-trend__tooltip"));
}

#[test]
fn resting_sparkline_has_no_axis_and_no_tag() {
    let html = render(sparkline_app);
    assert!(!html.contains("kit-trend__tag"));
    assert!(!html.contains("kit-trend__tooltip"));
    assert!(!html.contains("kit-trend__tick"));
    assert!(!html.contains("kit-trend__date"));
}

fn spark_tiles() -> Vec<SparkTile> {
    let tile = |label: &str, values: [f64; 7], tone: SeriesTone| SparkTile {
        label: label.into(),
        values: values.into_iter().map(Some).collect(),
        tone,
    };
    vec![
        tile(
            "Open items",
            [9.0, 11.0, 10.0, 13.0, 12.0, 14.0, 16.0],
            SeriesTone::Series2,
        ),
        tile(
            "Reopened",
            [2.0, 1.0, 3.0, 2.0, 4.0, 3.0, 2.0],
            SeriesTone::Series3,
        ),
        tile(
            "Average age",
            [6.0, 7.0, 5.0, 8.0, 7.0, 9.0, 8.0],
            SeriesTone::Series4,
        ),
    ]
}

fn block_app() -> Element {
    rsx! {
        SeriesTrendBlock {
            title: "Tickets per week",
            label: "Tickets per week",
            x_labels: week_labels(),
            series: gallery_series(),
            sparklines: spark_tiles(),
        }
    }
}

fn block_column_app() -> Element {
    rsx! {
        SeriesTrendBlock {
            title: "Tickets per week",
            label: "Tickets per week, columns",
            x_labels: week_labels(),
            series: gallery_series(),
            initial_variant: ChartVariant::Bar,
            sparklines: spark_tiles(),
        }
    }
}

#[test]
fn block_holds_one_headline_over_three_sparkline_tiles() {
    let html = render(block_app);
    assert_eq!(html.matches("class=\"kit-series-block\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-series-block__spark\"").count(), 3);
    let headline = html
        .find("kit-series-block__headline")
        .expect("headline row");
    let sparks = html
        .find("kit-series-block__sparks")
        .expect("sparkline row");
    assert!(headline < sparks);
}

#[test]
fn block_opens_in_area_mode_with_no_tooltip_at_rest() {
    let html = render(block_app);
    assert_eq!(html.matches("data-variant=\"area\"").count(), 1);
    assert_eq!(html.matches("data-variant=\"sparkline\"").count(), 3);
    assert!(!html.contains("kit-trend__tooltip"));
    assert!(!html.contains("kit-trend__tag"));
    let area = html.find(">Area<").expect("area mode control");
    let before = &html[..area];
    let pressed = before.rfind("aria-pressed=\"").expect("mode state");
    assert!(before[pressed..].starts_with("aria-pressed=\"true\""));
}

#[test]
fn block_headline_states_both_periods_for_the_last_week() {
    let html = render(block_app);
    assert!(html.contains("Completed at W7"));
    assert!(html.contains("Opened 19 at W7"));
    assert!(html.contains("↑ 63.2%"));
}

#[test]
fn block_column_mode_draws_columns_with_the_axis_at_zero() {
    let html = render(block_column_app);
    assert_eq!(html.matches("data-variant=\"bar\"").count(), 1);
    assert!(html.contains("kit-trend__column"));
    assert!(html.contains(">0<"));
}

#[test]
fn block_sparkline_tiles_draw_no_date_labels() {
    let html = render(block_app);
    let labels = html.matches("class=\"kit-trend__date\"").count()
        + html
            .matches("class=\"kit-trend__date kit-trend__date--mid\"")
            .count();
    assert_eq!(labels, 3);
}

#[test]
fn completed_area_lifts_the_pen_at_the_week_four_gap() {
    let values = &gallery_series()[0].values;
    let points: Vec<Option<(f64, f64)>> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            v.map(|v| {
                (
                    project_x(i, values.len(), 4.0, 632.0),
                    project_y(v, 0.0, 35.0, 8.0, 200.0),
                )
            })
        })
        .collect();
    let d = area_path(&points, 200.0);
    assert_eq!(d.matches('M').count(), 2, "one pen lift at W4: {d}");
}

#[test]
fn line_and_column_value_axes_start_at_zero_for_the_gallery_data() {
    assert!(render(line_app).contains(">0<"));
    assert!(render(bar_app).contains(">0<"));
}

#[test]
fn columns_start_at_zero_even_when_the_domain_is_not_anchored() {
    assert!(render(bar_unanchored_app).contains(">0<"));
}

#[test]
fn unanchored_line_axis_starts_above_zero() {
    assert!(!render(line_unanchored_app).contains(">0<"));
}
