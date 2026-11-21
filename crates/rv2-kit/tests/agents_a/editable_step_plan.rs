use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::editable_step_plan::{
    EditableStepPlan, PlanPhase, PlanStep, StepState, append_step, clock_text, move_step, progress,
    remove_step, rename_step, reset_from,
};

fn steps() -> Vec<PlanStep> {
    vec![
        PlanStep::new("a", "Read the brief"),
        PlanStep::new("b", "Draft the fix"),
        PlanStep::new("c", "Run the gate"),
    ]
}

fn keys(steps: &[PlanStep]) -> Vec<&str> {
    steps.iter().map(|s| s.step_key.as_str()).collect()
}

#[test]
fn removing_a_step_keeps_the_remaining_order() {
    assert_eq!(keys(&remove_step(&steps(), "b")), vec!["a", "c"]);
}

#[test]
fn renaming_changes_only_the_title() {
    let next = rename_step(&steps(), "b", "Draft the patch");
    assert_eq!(next[1].step_title, "Draft the patch");
    assert_eq!(next[1].step_key, "b");
    assert_eq!(next[0].step_title, "Read the brief");
}

#[test]
fn moving_a_step_shifts_its_neighbours() {
    assert_eq!(keys(&move_step(&steps(), 0, 2)), vec!["b", "c", "a"]);
    assert_eq!(keys(&move_step(&steps(), 2, 0)), vec!["c", "a", "b"]);
}

#[test]
fn out_of_range_move_returns_the_list_unchanged() {
    assert_eq!(move_step(&steps(), 0, 9), steps());
    assert_eq!(move_step(&steps(), 1, 1), steps());
}

#[test]
fn appended_step_gets_a_fresh_key_and_goes_last() {
    let next = append_step(&steps(), "Ship it");
    assert_eq!(next.len(), 4);
    assert_eq!(next[3].step_title, "Ship it");
    assert_eq!(next[3].step_state, StepState::Pending);
    let mut keys_seen: Vec<&str> = keys(&next);
    keys_seen.sort_unstable();
    keys_seen.dedup();
    assert_eq!(keys_seen.len(), 4);
}

#[test]
fn retry_resets_the_failed_step_and_every_later_step() {
    let mut plan = steps();
    plan[0].step_state = StepState::Succeeded;
    plan[1].step_state = StepState::Errored;
    plan[1].error_text = Some("boom".to_string());
    plan[2].step_state = StepState::InProgress;
    plan[2].measured_ms = Some(4_000);
    let next = reset_from(&plan, "b");
    assert_eq!(next[0].step_state, StepState::Succeeded);
    assert_eq!(next[1].step_state, StepState::Pending);
    assert_eq!(next[1].error_text, None);
    assert_eq!(next[2].step_state, StepState::Pending);
    assert_eq!(next[2].measured_ms, None);
}

#[test]
fn progress_counts_succeeded_steps_out_of_all() {
    let mut plan = steps();
    plan[0].step_state = StepState::Succeeded;
    assert_eq!(progress(&plan), (1, 3));
}

#[test]
fn clock_text_shows_hours_only_past_one_hour() {
    assert_eq!(clock_text(4_000), "0:04");
    assert_eq!(clock_text(3_723_000), "1:02:03");
}

fn drafting() -> Element {
    rsx! {
        EditableStepPlan {
            plan_heading: "Plan",
            initial_steps: steps(),
            on_start: move |_: Vec<PlanStep>| {},
        }
    }
}

#[test]
fn drafting_plan_is_an_ordered_list_named_by_its_heading() {
    let html = render(drafting);
    assert!(html.contains("data-phase=\"drafting\""));
    assert!(html.contains("<ol"));
    assert!(html.contains("aria-labelledby=\"kit-plan-list\""));
    assert!(html.contains("Plan</h2>"));
    assert!(html.contains("placeholder=\"Add a step\""));
    assert!(html.contains("Read the brief"));
}

#[test]
fn drafting_plan_offers_start_and_no_repeat_or_retry() {
    let html = render(drafting);
    assert!(html.contains(">Start</button>"));
    assert!(!html.contains("Retry from here"));
    assert!(!html.contains(">Repeat</button>"));
}

fn executing() -> Element {
    rsx! {
        EditableStepPlan {
            plan_heading: "Plan",
            plan_phase: PlanPhase::Executing,
            plan_steps: Some(vec![
                PlanStep { step_state: StepState::Succeeded, ..PlanStep::new("a", "Read") },
                PlanStep { step_state: StepState::InProgress, ..PlanStep::new("b", "Draft") },
            ]),
            on_start: move |_: Vec<PlanStep>| {},
        }
    }
}

#[test]
fn executing_plan_hides_the_add_field_and_marks_the_current_step() {
    let html = render(executing);
    assert!(!html.contains("placeholder=\"Add a step\""));
    assert!(html.contains("aria-current=\"step\""));
    assert!(html.contains("1 of 2"));
    assert!(!html.contains("kit-plan__btn--primary"));
}

fn empty_plan() -> Element {
    rsx! {
        EditableStepPlan {
            plan_heading: "Plan",
            on_start: move |_: Vec<PlanStep>| {},
        }
    }
}

#[test]
fn empty_plan_says_so_and_disables_start() {
    let html = render(empty_plan);
    assert!(html.contains("There are no steps yet."));
    assert!(html.contains("disabled"));
}
