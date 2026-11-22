//! Metric tile group structure at rest: a labelled group holds a two-by-two grid of tiles
//! with the period control in its header; each tile stacks top, value, change, chart and
//! footer. The loading example sits in its own group below.

use dioxus::prelude::*;
use rv2_kit::components::metric_tile::{MetricTile, MetricTileGroup, MetricValue};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn chart() -> Element {
    rsx! { svg { "aria-hidden": "true" } }
}

fn populated_group() -> Element {
    rsx! {
        MetricTileGroup {
            group_label: "Summary",
            header_control: Some(rsx! {
                div { class: "kit-tile__segments", role: "group", "aria-label": "Period",
                    button { r#type: "button", class: "kit-tile__segment", "aria-pressed": "true", "7 days" }
                    button { r#type: "button", class: "kit-tile__segment", "aria-pressed": "false", "30 days" }
                }
            }),
            MetricTile {
                metric_label: "Open tasks",
                metric_value: MetricValue::Number(42.0),
                change_percent: Some(12.5),
                change_note: Some("against the prior seven days".to_string()),
                inline_chart: Some(chart()),
                footer_content: Some(rsx! { "Read 23 Nov 2026" }),
            }
            MetricTile {
                metric_label: "Median latency",
                metric_value: MetricValue::Text("1.2 s".to_string()),
                change_percent: Some(-4.0),
                change_note: Some("against the prior seven days".to_string()),
                lower_is_better: true,
                inline_chart: Some(chart()),
                footer_content: Some(rsx! { "Read 23 Nov 2026" }),
            }
            MetricTile {
                metric_label: "Cost",
                metric_value: MetricValue::Text("$18.40".to_string()),
                change_percent: Some(7.8),
                change_note: Some("against the prior seven days".to_string()),
                lower_is_better: true,
                inline_chart: Some(chart()),
                footer_content: Some(rsx! { "Read 23 Nov 2026" }),
            }
            MetricTile {
                metric_label: "Failed runs",
                metric_value: MetricValue::Number(3.0),
                change_percent: Some(-25.0),
                change_note: Some("against the prior seven days".to_string()),
                lower_is_better: true,
                inline_chart: Some(chart()),
                footer_content: Some(rsx! { "Read 23 Nov 2026" }),
            }
        }
    }
}

fn populated_with_loading_app() -> Element {
    rsx! {
        {populated_group()}
        MetricTileGroup { group_label: "Loading example",
            MetricTile {
                metric_label: "Cost",
                metric_value: MetricValue::Number(0.0),
                is_loading: true,
                inline_chart: Some(chart()),
            }
        }
    }
}

fn linked_first_tile_app() -> Element {
    rsx! {
        MetricTile {
            metric_label: "Open tasks",
            metric_value: MetricValue::Number(42.0),
            target_href: Some("/tasks".to_string()),
            corner_control: Some(rsx! {
                div { class: "kit-tile__segments", role: "group", "aria-label": "Period",
                    button { r#type: "button", class: "kit-tile__segment", "aria-pressed": "true", "7 days" }
                }
            }),
        }
    }
}

#[test]
fn populated_group_holds_four_tiles_in_one_grid() {
    let html = render(populated_group);
    assert_eq!(html.matches("class=\"kit-tile-group__grid\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-tile\"").count(), 4);
}

#[test]
fn tile_stacks_top_value_change_chart_then_footer() {
    let html = render(populated_group);
    let top = html.find("kit-tile__top").expect("top row");
    let body = html.find("kit-tile__body").expect("value line");
    let change = html.find("kit-tile__change").expect("change row");
    let chart = html.find("kit-tile__chart").expect("chart");
    let footer = html.find("kit-tile__footer").expect("footer row");
    assert!(top < body && body < change && change < chart && chart < footer);
}

#[test]
fn period_control_sits_in_the_group_header_outside_every_tile() {
    let html = render(populated_group);
    let head = html.find("kit-tile-group__head").expect("group header");
    let segments = html.find("kit-tile__segments").expect("period control");
    let grid = html.find("kit-tile-group__grid").expect("grid");
    assert!(head < segments && segments < grid);
    assert_eq!(html.matches("kit-tile__corner").count(), 0);
    assert_eq!(html.matches("aria-pressed=\"true\"").count(), 1);
}

#[test]
fn populated_charts_sit_beneath_the_change_row_never_beside_the_value() {
    let html = render(populated_group);
    assert_eq!(html.matches("data-position=\"stacked\"").count(), 4);
    assert_eq!(html.matches("data-position=\"inline\"").count(), 0);
}

#[test]
fn kpi_change_badges_follow_direction_and_favourability() {
    let html = render(populated_group);
    assert!(html.contains("↑ 12.5%"));
    assert!(html.contains("↓ 4.0%"));
    assert!(html.contains("↑ 7.8%"));
    assert!(html.contains("↓ 25.0%"));
    assert_eq!(html.matches("data-tone=\"favourable\"").count(), 3);
    assert_eq!(html.matches("data-tone=\"unfavourable\"").count(), 1);
}

#[test]
fn populated_grid_at_rest_has_no_loading_or_busy_tile() {
    let html = render(populated_group);
    assert!(!html.contains("data-loading"));
    assert!(!html.contains("aria-busy"));
    assert_eq!(html.matches("Read 23 Nov 2026").count(), 4);
}

#[test]
fn loading_example_sits_in_its_own_group_below_the_populated_grid() {
    let html = render(populated_with_loading_app);
    let grid = html.find("kit-tile-group__grid").expect("populated grid");
    let label = html.find("Loading example").expect("loading label");
    let busy = html.find("aria-busy").expect("busy loading tile");
    assert!(grid < label && label < busy);
    assert!(!html[..label].contains("data-loading"));
}

#[test]
fn linked_tile_keeps_its_corner_control_outside_the_link() {
    let html = render(linked_first_tile_app);
    let start = html.find("<a ").expect("link rendered");
    let end = start + html[start..].find("</a>").expect("link closes");
    assert!(!html[start..end].contains("kit-tile__segments"));
    assert!(html.contains("kit-tile__segments"));
}
