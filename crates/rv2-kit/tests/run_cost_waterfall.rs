//! Run cost waterfall: bar placement on the cumulative cost axis and the table render.

use std::collections::BTreeSet;

use dioxus::prelude::*;
use rv2_kit::components::run_cost_breakdown::{
    Measure, RunCostBreakdown, Step, StepKind, StepStatus, axis_ceiling, bar_geometry, bar_reach,
    bar_spans,
};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn step(id: &str, cost: f64, children: Vec<Step>) -> Step {
    Step {
        step_id: id.into(),
        step_name: id.into(),
        step_kind: StepKind::ModelCall,
        step_status: StepStatus::Done,
        model_name: None,
        step_note: None,
        in_tokens: Some(10.0),
        out_tokens: Some(5.0),
        step_cost: Some(cost),
        step_elapsed: Some(1.0),
        prompt_text: None,
        result_text: None,
        error_text: None,
        child_steps: children,
    }
}

/// A top-level step costing 0.04, then a sub-agent whose only child costs 0.03.
fn waterfall() -> Vec<Step> {
    vec![
        step("plan", 0.04, vec![]),
        step("research", 0.0, vec![step("search", 0.03, vec![])]),
    ]
}

#[test]
fn each_top_level_bar_starts_where_the_running_total_above_it_ends() {
    let spans = bar_spans(&waterfall(), &BTreeSet::new(), Measure::Cost);
    assert_eq!(spans.len(), 3);
    assert!(close(spans[0].0, 0.0) && close(spans[0].1, 0.04));
    assert!(close(spans[1].0, 0.04) && close(spans[1].1, 0.03));
}

#[test]
fn a_child_bar_starts_at_its_parents_start_and_stays_inside_the_parent() {
    let spans = bar_spans(&waterfall(), &BTreeSet::new(), Measure::Cost);
    let (parent_start, parent_value) = spans[1];
    let (child_start, child_value) = spans[2];
    assert!(close(child_start, parent_start));
    assert!(child_start + child_value <= parent_start + parent_value + 1e-9);
}

#[test]
fn a_folded_sub_agent_drops_its_child_bars_and_leaves_the_total_alone() {
    let folded = BTreeSet::from(["research".to_string()]);
    let spans = bar_spans(&waterfall(), &folded, Measure::Cost);
    assert_eq!(spans.len(), 2);
    assert!(close(spans[1].0, 0.04));
}

#[test]
fn switching_to_tokens_rebuilds_starts_and_lengths_from_token_totals() {
    let spans = bar_spans(&waterfall(), &BTreeSet::new(), Measure::Tokens);
    assert_eq!(spans.len(), 3);
    assert!(close(spans[0].1, 15.0));
    assert!(close(spans[1].0, 15.0));
    assert!(close(spans[1].1, 15.0));
}

#[test]
fn the_axis_ceiling_is_the_round_figure_at_or_above_the_reach() {
    assert!(close(axis_ceiling(0.07), 0.1));
    assert!(close(axis_ceiling(0.1), 0.1));
    assert!(close(axis_ceiling(2_300.0), 2_500.0));
    assert!(close(axis_ceiling(0.0), 1.0));
}

#[test]
fn the_axis_moves_to_the_next_round_figure_once_the_total_passes_it() {
    let ceiling_for = |cost: f64| {
        let spans = bar_spans(
            &[step("only", cost, vec![])],
            &BTreeSet::new(),
            Measure::Cost,
        );
        axis_ceiling(bar_reach(&spans))
    };
    assert!(close(ceiling_for(0.10), 0.1));
    assert!(close(ceiling_for(0.11), 0.2));
}

#[test]
fn a_credit_bar_runs_back_from_its_start_and_stays_on_the_axis() {
    let (left, width) = bar_geometry(0.10, -0.02, 0.1);
    assert!(close(left, 80.0));
    assert!(close(width, 20.0));
    let (_, width) = bar_geometry(0.0, 0.5, 0.1);
    assert!(close(width, 100.0));
}

fn waterfall_app() -> Element {
    rsx! { RunCostBreakdown { step_list: waterfall(), run_label: "Plan run" } }
}

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn the_axis_labels_zero_and_its_ceiling_above_the_bar_zone() {
    let html = render(waterfall_app);
    assert_eq!(html.matches("kit-rcb__tick").count(), 3);
    assert!(html.contains("$0.000"));
    assert!(html.contains("$0.100"));
    assert!(html.contains("Costliest: plan"));
}

#[test]
fn the_rendered_bars_match_the_placement_of_the_first_step_and_its_child() {
    let html = render(waterfall_app);
    assert_eq!(
        html.matches("style=\"left: 0.00%; width: 40.00%\"").count(),
        1
    );
    assert_eq!(
        html.matches("style=\"left: 40.00%; width: 30.00%\"")
            .count(),
        2
    );
}
