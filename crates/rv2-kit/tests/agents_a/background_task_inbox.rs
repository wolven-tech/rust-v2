use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::background_task_inbox::{
    BackgroundTaskInbox, InboxFilter, InboxLayout, Job, JobAction, JobColumn, JobStage,
    StageOverride, TimelineStep, actions_for, announcement, clock_text, in_filter, override_for,
    recent_steps, resolve,
};

fn job(id: &str, stage: JobStage) -> Job {
    Job {
        id: id.to_string(),
        heading_text: format!("Job {id}"),
        summary_text: Some(format!("Summary of {id}")),
        owner_id: "owner".to_string(),
        stage,
        started_at: Some(0),
        elapsed: Some(90_000),
        tool_count: Some(3),
        source_count: None,
        timeline: vec![TimelineStep {
            label: "Started".to_string(),
            at: Some(0),
        }],
    }
}

#[test]
fn waiting_and_failed_jobs_sit_in_needs_you_column() {
    assert_eq!(JobStage::Waiting.column(), JobColumn::NeedsYou);
    assert_eq!(JobStage::Failed.column(), JobColumn::NeedsYou);
    assert_eq!(JobStage::Running.column(), JobColumn::Running);
    assert_eq!(JobStage::Done.column(), JobColumn::Done);
}

#[test]
fn running_job_clock_advances_with_now_and_stops_when_done() {
    let running = job("a", JobStage::Running);
    assert_eq!(resolve(&running, &[], 5_000), (JobStage::Running, 5));
    assert_eq!(resolve(&running, &[], 6_000), (JobStage::Running, 6));

    let done = job("b", JobStage::Done);
    assert_eq!(resolve(&done, &[], 5_000), (JobStage::Done, 90));
    assert_eq!(resolve(&done, &[], 900_000), (JobStage::Done, 90));
}

#[test]
fn confirm_without_handler_moves_waiting_job_to_running_from_zero() {
    let waiting = job("w", JobStage::Waiting);
    let next = override_for(&waiting.id, 0, JobAction::Confirm, 10_000).expect("override");
    assert_eq!(next.stage, JobStage::Running);
    assert_eq!(
        resolve(&waiting, std::slice::from_ref(&next), 10_000),
        (JobStage::Running, 0)
    );
    assert_eq!(
        resolve(&waiting, std::slice::from_ref(&next), 13_000),
        (JobStage::Running, 3)
    );
}

#[test]
fn decline_without_handler_freezes_clock_at_failed() {
    let next = override_for("w", 42, JobAction::Decline, 99_000).expect("override");
    let waiting = job("w", JobStage::Waiting);
    assert_eq!(
        resolve(&waiting, std::slice::from_ref(&next), 500_000),
        (JobStage::Failed, 42)
    );
}

#[test]
fn restart_runs_failed_job_again_from_zero() {
    let failed = job("f", JobStage::Failed);
    let next: StageOverride = override_for("f", 7, JobAction::Restart, 20_000).expect("override");
    assert_eq!(resolve(&failed, &[next], 20_000), (JobStage::Running, 0));
}

#[test]
fn inspect_never_changes_stage() {
    assert!(override_for("x", 0, JobAction::Inspect, 0).is_none());
}

#[test]
fn actions_match_stage_and_inspect_needs_a_handler() {
    assert_eq!(
        actions_for(JobStage::Waiting, false),
        vec![JobAction::Confirm, JobAction::Decline]
    );
    assert_eq!(
        actions_for(JobStage::Failed, false),
        vec![JobAction::Restart]
    );
    assert!(actions_for(JobStage::Running, false).is_empty());
    assert_eq!(
        actions_for(JobStage::Running, true),
        vec![JobAction::Inspect]
    );
    assert_eq!(actions_for(JobStage::Done, true), vec![JobAction::Inspect]);
}

#[test]
fn clock_text_shows_hours_only_past_one_hour() {
    assert_eq!(clock_text(0), "0:00");
    assert_eq!(clock_text(75), "1:15");
    assert_eq!(clock_text(3_725), "1:02:05");
}

#[test]
fn live_line_counts_needs_you_and_running() {
    assert_eq!(announcement(2, 1), "2 waiting on you, 1 running.");
}

#[test]
fn list_filters_select_by_column() {
    assert!(in_filter(JobStage::Failed, InboxFilter::NeedsYou));
    assert!(!in_filter(JobStage::Done, InboxFilter::Running));
    assert!(in_filter(JobStage::Done, InboxFilter::All));
}

#[test]
fn timeline_shows_latest_steps_oldest_first() {
    let steps: Vec<TimelineStep> = (0..6)
        .map(|n| TimelineStep {
            label: format!("s{n}"),
            at: None,
        })
        .collect();
    let shown: Vec<&str> = recent_steps(&steps)
        .iter()
        .map(|s| s.label.as_str())
        .collect();
    assert_eq!(shown, vec!["s2", "s3", "s4", "s5"]);
}

fn board() -> Element {
    rsx! {
        BackgroundTaskInbox {
            jobs: vec![
                job("w", JobStage::Waiting),
                job("r", JobStage::Running),
                job("d", JobStage::Done),
            ],
        }
    }
}

#[test]
fn board_renders_three_labelled_columns_with_counts() {
    let html = render(board);
    assert!(html.contains("Needs you"));
    assert!(html.contains("data-column=\"needs-you\""));
    assert!(html.contains("data-column=\"running\""));
    assert!(html.contains("data-column=\"done\""));
    assert!(html.contains("data-layout=\"board\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("kit-inbox__body"));
}

fn list() -> Element {
    rsx! {
        BackgroundTaskInbox {
            jobs: vec![job("w", JobStage::Waiting), job("f", JobStage::Failed)],
            initial_layout: InboxLayout::List,
        }
    }
}

#[test]
fn list_layout_shows_filter_tabs_with_live_counts() {
    let html = render(list);
    assert!(html.contains("data-layout=\"list\""));
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("Needs you"));
    assert!(html.contains("kit-inbox__list"));
}

fn open_card() -> Element {
    rsx! {
        BackgroundTaskInbox {
            jobs: vec![job("w", JobStage::Waiting)],
            clock_ms: 0,
            on_confirm: move |_: Job| {},
            on_decline: move |_: Job| {},
        }
    }
}

#[test]
fn closed_card_hides_its_actions_and_steps() {
    let html = render(open_card);
    assert!(!html.contains("data-action=\"Confirm\""));
    assert!(!html.contains("kit-inbox__steps"));
    assert!(html.contains("aria-controls=\"kit-inbox-card-w-body\""));
}

#[test]
fn live_region_announces_counts_politely() {
    let html = render(board);
    assert!(html.contains("aria-live=\"polite\""));
    assert!(html.contains("1 waiting on you, 1 running."));
}
