use dioxus::prelude::*;
use rv2_kit::screens::agent_run_timeline::{
    AgentRunTimeline, RunRecord, StepRecord, StepType, axis_ticks, tick_label,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn step(
    id: &str,
    label: &str,
    step_type: StepType,
    start: f64,
    duration: Option<f64>,
) -> StepRecord {
    StepRecord {
        id: id.to_string(),
        entry_label: label.to_string(),
        step_type,
        start_seconds: start,
        duration_seconds: duration,
        nesting_depth: 0,
        app_name: None,
        sub_agent_name: None,
        has_failed: false,
        error_text: None,
        token_counts: None,
        cost_label: None,
        input_text: Some(format!("{label}: input")),
        output_text: Some(format!("{label}: output")),
    }
}

fn fixture_steps() -> Vec<StepRecord> {
    use StepType::{Model, SubAgent, Tool};
    let mut steps = vec![
        step("s1", "Plan the review", Model, 0.0, Some(8.0)),
        step("s2", "Read the queue", Tool, 8.0, Some(20.0)),
        step("s3", "Pull the metrics", Tool, 28.0, Some(18.0)),
        step("s4", "Classify tickets", SubAgent, 46.0, Some(74.0)),
        step("s5", "Label batch one", Model, 48.0, Some(22.0)),
        step("s6", "Label batch two", Model, 70.0, Some(26.0)),
        step("s7", "Tag the topics", Tool, 120.0, Some(16.0)),
        step("s8", "Score the risks", Model, 136.0, Some(22.0)),
        step("s9", "Fetch the account", Tool, 158.0, Some(12.0)),
        step("s10", "Label tickets", Model, 170.0, Some(10.0)),
        step("s11", "Label tickets, retry", Model, 180.0, Some(20.0)),
        step("s12", "Draft the reply", Tool, 200.0, Some(12.0)),
        step("s13", "Check the policy", Tool, 212.0, Some(14.0)),
        step("s14", "Review the draft", Model, 226.0, Some(32.0)),
        step("s15", "Write the status note", Tool, 258.0, Some(22.0)),
        step("s16", "Write the summary", Model, 280.0, None),
    ];
    steps[4].nesting_depth = 1;
    steps[5].nesting_depth = 1;
    steps[3].sub_agent_name = Some("Classifier".to_string());
    steps[9].has_failed = true;
    steps[9].error_text = Some("Rate limit reached, retried later.".to_string());
    steps[14].token_counts = Some((520, 180));
    steps[14].cost_label = Some("$0.06".to_string());
    steps
}

fn fixture_run(steps: Vec<StepRecord>) -> RunRecord {
    RunRecord {
        id: "r1".to_string(),
        entry_title: "Summarise the support queue".to_string(),
        agent_name: "Ada".to_string(),
        model_name: "Claude Sonnet".to_string(),
        started_at_ms: 1_795_219_200_000,
        total_seconds: None,
        cost_label: Some("$4.82".to_string()),
        step_records: steps,
    }
}

fn rest_app() -> Element {
    rsx! {
        AgentRunTimeline {
            run_record: Some(fixture_run(fixture_steps())),
            elapsed_seconds: Some(296.0),
            initial_step_id: Some("s15".to_string()),
        }
    }
}

#[test]
fn timeline_at_rest_has_sixteen_step_rows_with_two_nested_under_the_sub_agent() {
    let steps = fixture_steps();
    assert_eq!(steps.len(), 16);
    let parent = steps
        .iter()
        .position(|s| s.step_type == StepType::SubAgent)
        .expect("parent");
    assert_eq!(steps[parent + 1].nesting_depth, 1);
    assert_eq!(steps[parent + 2].nesting_depth, 1);
    assert_eq!(steps[parent + 3].nesting_depth, 0);
    let html = render(rest_app);
    assert_eq!(html.matches("kit-art__row--nested").count(), 2);
    assert_eq!(html.matches("class=\"kit-art__hit\"").count(), 16);
}

#[test]
fn failed_step_retry_is_the_row_directly_beneath_with_its_own_bar() {
    let steps = fixture_steps();
    let failed = steps
        .iter()
        .position(|s| s.has_failed)
        .expect("one failed step");
    assert_eq!(steps.iter().filter(|s| s.has_failed).count(), 1);
    let retry = &steps[failed + 1];
    assert_eq!(retry.step_type, steps[failed].step_type);
    assert!(!retry.has_failed);
    assert!(retry.duration_seconds.is_some());
}

#[test]
fn last_row_at_rest_is_the_running_step() {
    let steps = fixture_steps();
    assert!(steps.last().is_some_and(|s| s.duration_seconds.is_none()));
    assert_eq!(
        steps
            .iter()
            .filter(|s| s.duration_seconds.is_none())
            .count(),
        1
    );
}

#[test]
fn steps_tile_reads_step_count_with_failure_count_together() {
    let html = render(rest_app);
    assert!(html.contains("16 steps · 1 failed"));
}

#[test]
fn detail_panel_at_rest_shows_start_tokens_cost_and_copy_for_input_and_output() {
    let html = render(rest_app);
    assert!(html.contains("Write the status note"));
    assert!(html.contains("Copy input"));
    assert!(html.contains("Copy output"));
    assert!(html.contains("Start"));
    assert!(html.contains("520 in · 180 out"));
    assert!(html.contains("$0.06"));
}

#[test]
fn axis_ticks_at_rest_are_round_and_stay_inside_the_span() {
    let ticks = axis_ticks(296.0);
    let labels: Vec<String> = ticks.iter().map(|t| tick_label(*t)).collect();
    assert_eq!(labels, vec!["0:00", "1:00", "2:00", "3:00", "4:00"]);
    assert!(ticks.iter().all(|t| *t <= 296.0));
}

#[test]
fn running_status_badge_reads_running_while_live() {
    let html = render(rest_app);
    assert!(html.contains("Running"));
    assert!(html.contains("kit-art__now"));
}
