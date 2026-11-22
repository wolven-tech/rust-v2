//! Segmented bar explorer amendment: the two gallery instances, checked as data and at rest.

use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;
use rv2_kit::components::segmented_bar_explorer::{
    GroupingAxis, MetricDef, SegmentedBarExplorer, TallyRow, bar_breakdown, stack_tone,
};

const REGIONS: [&str; 4] = ["North", "South", "East", "West"];
const STATUSES: [&str; 3] = ["Open", "In review", "Closed"];
const PRIORITIES: [&str; 3] = ["High", "Medium", "Low"];

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn rows() -> Vec<TallyRow> {
    (0..118usize)
        .map(|i| TallyRow {
            axis: BTreeMap::from([
                (
                    "region".to_string(),
                    REGIONS[(i * 3 + i / 7) % 4].to_string(),
                ),
                ("status".to_string(), STATUSES[(i + i / 5) % 3].to_string()),
                (
                    "priority".to_string(),
                    PRIORITIES[(i * 2 + i / 9) % 3].to_string(),
                ),
            ]),
            metrics: BTreeMap::from([("hours".to_string(), (1 + (i * 37) % 23) as f64)]),
        })
        .collect()
}

fn axes() -> Vec<GroupingAxis> {
    vec![
        GroupingAxis {
            axis_key: "region".into(),
            axis_label: "Region".into(),
            axis_values: REGIONS.map(String::from).to_vec(),
        },
        GroupingAxis {
            axis_key: "status".into(),
            axis_label: "Status".into(),
            axis_values: STATUSES.map(String::from).to_vec(),
        },
        GroupingAxis {
            axis_key: "priority".into(),
            axis_label: "Priority".into(),
            axis_values: PRIORITIES.map(String::from).to_vec(),
        },
    ]
}

fn metrics() -> Vec<MetricDef> {
    vec![
        MetricDef {
            metric_key: "tickets".into(),
            metric_label: "Tickets".into(),
            unit: "tickets".into(),
            source_field: None,
        },
        MetricDef {
            metric_key: "hours".into(),
            metric_label: "Hours".into(),
            unit: "h".into(),
            source_field: Some("hours".into()),
        },
    ]
}

fn tickets_app() -> Element {
    rsx! {
        SegmentedBarExplorer {
            tally_rows: rows(),
            grouping_axes: axes(),
            metric_defs: metrics(),
            heading_text: "Tickets by region",
            start_metric: "tickets",
            start_axis: "region",
            start_stack: "status",
        }
    }
}

fn focus_app() -> Element {
    rsx! {
        SegmentedBarExplorer {
            tally_rows: rows(),
            grouping_axes: axes(),
            metric_defs: metrics(),
            heading_text: "Hours by region",
            start_metric: "hours",
            start_axis: "region",
            start_stack: "priority",
            start_focus: "East",
            start_hidden: vec!["Low".to_string()],
        }
    }
}

#[test]
fn stack_tone_cycles_through_the_six_series_tokens() {
    assert_eq!(stack_tone(0), "series-1");
    assert_eq!(stack_tone(5), "series-6");
    assert_eq!(stack_tone(6), "series-1");
}

#[test]
fn gallery_tickets_bars_total_all_118_rows_across_four_regions() {
    let (rows, axes, metrics) = (rows(), axes(), metrics());
    let bars = bar_breakdown(
        &rows,
        &axes[0],
        Some(&axes[1]),
        &metrics[0],
        &BTreeSet::new(),
    );
    assert_eq!(bars.len(), 4);
    let sum: f64 = bars.iter().map(|b| b.total).sum();
    assert_eq!(sum, 118.0);
}

#[test]
fn hidden_low_priority_segment_is_removed_and_the_rest_keep_axis_order() {
    let (rows, axes, metrics) = (rows(), axes(), metrics());
    let hidden = BTreeSet::from(["Low".to_string()]);
    let bars = bar_breakdown(&rows, &axes[0], Some(&axes[2]), &metrics[1], &hidden);
    for bar in &bars {
        let names: Vec<&str> = bar.segments.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["High", "Medium"]);
    }
}

#[test]
fn gallery_tickets_instance_renders_four_bars_three_pressed_toggles_and_a_totals_row() {
    let html = render(tickets_app);
    assert_eq!(count(&html, "class=\"kit-seg__bar\""), 4);
    assert_eq!(count(&html, "aria-pressed=\"true\""), 3);
    assert_eq!(count(&html, "aria-pressed=\"false\""), 0);
    assert_eq!(count(&html, "class=\"kit-seg__totals\""), 1);
    assert!(html.contains("118 tickets"));
}

#[test]
fn focused_east_bar_dims_the_others_and_highlights_its_table_row() {
    let html = render(focus_app);
    assert_eq!(count(&html, "data-dim=\"true\""), 3);
    assert_eq!(count(&html, "data-active=\"true\""), 2);
    assert_eq!(count(&html, "kit-seg__detail-row"), 2);
}

#[test]
fn unpressed_low_priority_toggle_is_the_only_one_off_in_the_hours_instance() {
    let html = render(focus_app);
    assert_eq!(count(&html, "aria-pressed=\"false\""), 1);
    assert_eq!(count(&html, "aria-pressed=\"true\""), 2);
}
