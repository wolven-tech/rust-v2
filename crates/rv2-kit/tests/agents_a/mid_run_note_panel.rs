use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::mid_run_note_panel::{
    ComposerAction, MidRunNotePanel, Note, NotePlacement, PlanStep, RunState, StepPhase, advance,
    composer_action, format_clock, fresh_notes, next_run_state, queue_visible, step_counter,
    step_time_ms, submit_text, taken_announcement, taken_notes,
};

fn note(key: &str, text: &str, placed: Option<NotePlacement>) -> Note {
    Note {
        note_key: key.to_string(),
        text: text.to_string(),
        placed,
    }
}

fn step(key: &str, phase: StepPhase, notes: Vec<Note>) -> PlanStep {
    PlanStep {
        step_key: key.to_string(),
        label: format!("Step {key}"),
        phase,
        began_at_ms: Some(10_000),
        took_ms: Some(42_000),
        step_notes: notes,
    }
}

#[test]
fn enter_queues_trimmed_text_and_blank_draft_submits_nothing() {
    assert_eq!(
        submit_text("  check the lockfile \n"),
        Some("check the lockfile".to_string())
    );
    assert_eq!(submit_text("   \n\t"), None);
}

#[test]
fn enter_applies_now_with_ctrl_or_cmd_and_never_queues() {
    assert_eq!(
        composer_action(&Key::Enter, false, false),
        ComposerAction::Queue
    );
    assert_eq!(
        composer_action(&Key::Enter, true, false),
        ComposerAction::ApplyNow
    );
}

#[test]
fn shift_enter_is_a_line_break_and_escape_clears() {
    assert_eq!(
        composer_action(&Key::Enter, false, true),
        ComposerAction::Newline
    );
    assert_eq!(
        composer_action(&Key::Escape, false, false),
        ComposerAction::Clear
    );
    assert_eq!(
        composer_action(&Key::Character("a".into()), false, false),
        ComposerAction::Ignore
    );
}

#[test]
fn clock_advances_only_while_running() {
    assert_eq!(advance(5_000, RunState::Running), 6_000);
    assert_eq!(advance(5_000, RunState::Paused), 5_000);
    assert_eq!(advance(5_000, RunState::Finished), 5_000);
}

#[test]
fn pause_control_toggles_and_finished_run_has_none() {
    assert_eq!(next_run_state(RunState::Running), Some(RunState::Paused));
    assert_eq!(next_run_state(RunState::Paused), Some(RunState::Running));
    assert_eq!(next_run_state(RunState::Finished), None);
}

#[test]
fn clock_text_shows_hours_only_past_one_hour() {
    assert_eq!(format_clock(0), "0:00");
    assert_eq!(format_clock(75_000), "1:15");
    assert_eq!(format_clock(3_725_000), "1:02:05");
}

#[test]
fn done_step_shows_duration_and_active_step_shows_elapsed() {
    let done = step("a", StepPhase::Done, Vec::new());
    assert_eq!(step_time_ms(&done, 0, 900_000), Some(42_000));
    let active = step("b", StepPhase::Active, Vec::new());
    assert_eq!(step_time_ms(&active, 0, 25_000), Some(15_000));
    let upcoming = step("c", StepPhase::Upcoming, Vec::new());
    assert_eq!(step_time_ms(&upcoming, 0, 25_000), None);
}

#[test]
fn step_counter_follows_the_active_step_or_finished_count() {
    let steps = vec![
        step("a", StepPhase::Done, Vec::new()),
        step("b", StepPhase::Active, Vec::new()),
        step("c", StepPhase::Upcoming, Vec::new()),
    ];
    assert_eq!(step_counter(&steps), (2, 3));
    let finished = vec![
        step("a", StepPhase::Done, Vec::new()),
        step("b", StepPhase::Done, Vec::new()),
    ];
    assert_eq!(step_counter(&finished), (2, 2));
}

#[test]
fn taken_note_leaves_the_queue_and_shows_in_its_step() {
    let queued = vec![
        note("n1", "use the cache", None),
        note("n2", "skip lint", None),
    ];
    let steps = vec![step(
        "a",
        StepPhase::Active,
        vec![note("n1", "use the cache", Some(NotePlacement::Next))],
    )];
    let visible = queue_visible(&queued, &steps);
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].note_key, "n2");
    assert_eq!(taken_notes(&steps)[0].placed, Some(NotePlacement::Next));
}

#[test]
fn only_notes_not_yet_seen_are_announced() {
    let steps = vec![step(
        "a",
        StepPhase::Active,
        vec![
            note("n1", "old", Some(NotePlacement::Now)),
            note("n2", "new", Some(NotePlacement::Next)),
        ],
    )];
    let all = taken_notes(&steps);
    let fresh = fresh_notes(&all, &["n1".to_string()]);
    assert_eq!(fresh.len(), 1);
    assert_eq!(fresh[0].note_key, "n2");
    assert_eq!(taken_announcement(&fresh), "Agent took in: new");
    assert_eq!(taken_announcement(&all), "Agent took in 2 notes.");
}

fn running() -> Element {
    rsx! {
        MidRunNotePanel {
            run_heading: "Refactor the ingest path",
            run_agent_name: "Planner",
            run_started_ms: 0u64,
            clock_ms: 42_000u64,
            run_state: RunState::Running,
            plan_steps: vec![
                step("a", StepPhase::Done, Vec::new()),
                step("b", StepPhase::Active, vec![note("n1", "keep the batch size", Some(NotePlacement::Now))]),
                step("c", StepPhase::Upcoming, Vec::new()),
            ],
            queued_notes: vec![note("q1", "check the lockfile", None), note("n1", "keep the batch size", None)],
            draft_text: "",
            draft_hint: "Tell the agent something",
            on_queue_note: move |_: String| {},
            on_apply_note_now: move |_: (String, Option<String>)| {},
            on_drop_note: move |_: String| {},
            on_run_state_request: move |_: RunState| {},
            on_draft_change: move |_: String| {},
        }
    }
}

#[test]
fn running_panel_shows_plan_queue_and_composer() {
    let html = render(running);
    assert!(html.contains("data-run-state=\"running\""));
    assert!(html.contains("Step 2 of 3"));
    assert!(html.contains("aria-label=\"Plan\""));
    assert_eq!(html.matches("aria-current=\"step\"").count(), 1);
    assert!(html.contains("0:42"));
    assert!(html.contains("Queued notes (1)"));
    assert!(html.contains("check the lockfile"));
    assert!(html.contains("Applied now"));
    assert!(html.contains("Tell the agent something"));
    assert!(html.contains("aria-describedby"));
    assert!(html.contains("Pause"));
}

#[test]
fn live_region_is_polite_and_empty_draft_disables_send() {
    let html = render(running);
    assert!(html.contains("aria-live=\"polite\""));
    assert!(html.contains("kit-mrnp__send"));
    assert!(html.contains("disabled"));
    assert!(html.contains("Ctrl+Enter or Cmd+Enter applies now"));
}

fn paused() -> Element {
    rsx! {
        MidRunNotePanel {
            run_heading: "Refactor the ingest path",
            run_started_ms: 0u64,
            run_state: RunState::Paused,
            plan_steps: vec![step("a", StepPhase::Active, Vec::new())],
            queued_notes: Vec::new(),
            draft_text: "keep going",
            on_queue_note: move |_: String| {},
            on_apply_note_now: move |_: (String, Option<String>)| {},
            on_drop_note: move |_: String| {},
            on_run_state_request: move |_: RunState| {},
            on_draft_change: move |_: String| {},
        }
    }
}

#[test]
fn paused_run_offers_resume_and_still_accepts_notes() {
    let html = render(paused);
    assert!(html.contains("data-run-state=\"paused\""));
    assert!(html.contains("Resume"));
    assert!(!html.contains("Pause<"));
    assert!(html.contains("kit-mrnp__field"));
    assert!(!html.contains("Queued notes"));
}

fn finished() -> Element {
    rsx! {
        MidRunNotePanel {
            run_heading: "Refactor the ingest path",
            run_started_ms: 0u64,
            run_state: RunState::Finished,
            plan_steps: vec![step("a", StepPhase::Done, Vec::new())],
            queued_notes: vec![note("q1", "too late", None)],
            draft_text: "",
            on_queue_note: move |_: String| {},
            on_apply_note_now: move |_: (String, Option<String>)| {},
            on_drop_note: move |_: String| {},
            on_run_state_request: move |_: RunState| {},
            on_draft_change: move |_: String| {},
        }
    }
}

#[test]
fn finished_run_removes_composer_queue_and_control() {
    let html = render(finished);
    assert!(html.contains("data-run-state=\"finished\""));
    assert!(!html.contains("kit-mrnp__composer"));
    assert!(!html.contains("kit-mrnp__queue"));
    assert!(!html.contains("Resume"));
    assert!(!html.contains("Pause"));
    assert!(html.contains("0:42"));
}
