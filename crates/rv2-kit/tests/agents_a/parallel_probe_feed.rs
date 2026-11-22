use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::parallel_probe_feed::{
    FeedResult, ParallelProbeFeed, ProbeWorker, Severity, SeverityCounts, WorkerState,
    effective_state, progress_percent, relative_text, result_announcement, severity_totals,
    summary_text, toggle_filter, totals, visible_results,
};

fn worker(id: &str, completed: u32, planned: u32) -> ProbeWorker {
    ProbeWorker {
        id: id.to_string(),
        name: format!("Probe {id}"),
        current_step: Some(format!("checking {id}")),
        completed,
        planned,
        state: None,
        counts: SeverityCounts {
            critical: 1,
            warning: 2,
            info: 0,
        },
    }
}

fn result(id: &str, worker_id: &str, found_ms: u64) -> FeedResult {
    FeedResult {
        id: id.to_string(),
        worker_id: worker_id.to_string(),
        severity: Severity::Warning,
        title: format!("Finding {id}"),
        location: Some("src/lib.rs:12".to_string()),
        found_ms,
    }
}

#[test]
fn worker_is_done_at_planned_and_running_before_unless_failed() {
    assert_eq!(effective_state(&worker("a", 3, 3)), WorkerState::Done);
    assert_eq!(effective_state(&worker("b", 2, 3)), WorkerState::Running);
    let mut failed = worker("c", 1, 3);
    failed.state = Some(WorkerState::Failed);
    assert_eq!(effective_state(&failed), WorkerState::Failed);
}

#[test]
fn header_summary_sums_completed_and_planned() {
    let workers = vec![worker("a", 3, 4), worker("b", 1, 6)];
    assert_eq!(totals(&workers), (4, 10));
    assert_eq!(summary_text(4, 10), "4 of 10 checks");
}

#[test]
fn header_severity_totals_sum_each_workers_counts() {
    let workers = vec![worker("a", 0, 1), worker("b", 0, 1)];
    assert_eq!(
        severity_totals(&workers),
        SeverityCounts {
            critical: 2,
            warning: 4,
            info: 0
        }
    );
}

#[test]
fn progress_bar_fill_is_clamped_and_full_when_nothing_is_planned() {
    assert_eq!(progress_percent(1, 4), 25.0);
    assert_eq!(progress_percent(9, 4), 100.0);
    assert_eq!(progress_percent(0, 0), 100.0);
}

#[test]
fn feed_shows_only_the_filtered_worker_and_never_more_than_the_limit() {
    let results = vec![
        result("1", "a", 50),
        result("2", "b", 40),
        result("3", "a", 30),
        result("4", "a", 20),
        result("5", "a", 10),
    ];
    let shown = visible_results(&results, Some("a"), 3);
    assert_eq!(
        shown.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(),
        vec!["1", "3", "4"]
    );
    assert_eq!(visible_results(&results, None, 2).len(), 2);
}

#[test]
fn clicking_the_active_worker_clears_the_filter() {
    assert_eq!(toggle_filter(None, "a"), Some("a".to_string()));
    assert_eq!(toggle_filter(Some("a"), "a"), None);
    assert_eq!(toggle_filter(Some("a"), "b"), Some("b".to_string()));
}

#[test]
fn severity_has_a_text_label_and_relative_times_read_in_seconds_minutes_hours() {
    assert_eq!(Severity::Critical.label(), "Critical");
    assert_eq!(relative_text(10_000, 25_000), "15s ago");
    assert_eq!(relative_text(0, 180_000), "3m ago");
    assert_eq!(relative_text(0, 7_200_000), "2h ago");
}

#[test]
fn new_result_is_announced_with_its_severity_and_worker() {
    let found = result("9", "a", 0);
    assert_eq!(
        result_announcement(&found, "Probe a"),
        "Warning: Finding 9, Probe a."
    );
}

fn swarm() -> Element {
    let mut failed = worker("c", 1, 4);
    failed.state = Some(WorkerState::Failed);
    rsx! {
        ParallelProbeFeed {
            workers: vec![worker("a", 4, 4), worker("b", 2, 6), failed],
            results: vec![
                result("r1", "a", 60_000),
                result("r2", "b", 50_000),
                result("r3", "a", 40_000),
                result("r4", "c", 30_000),
                result("r5", "b", 20_000),
                result("r6", "a", 10_000),
            ],
            on_select_result: move |_: FeedResult| {},
        }
    }
}

#[test]
fn swarm_renders_header_rows_and_feed_capped_at_four() {
    let html = render(swarm);
    assert!(html.contains("Verification"));
    assert!(html.contains("7 of 14 checks"));
    assert!(html.contains("Probe a"));
    assert!(html.contains("Probe b"));
    assert!(html.contains("Failed"));
    assert!(html.contains("Done"));
    assert!(html.contains("Latest findings"));
    assert_eq!(html.matches("kit-probe__result\"").count(), 4);
    assert!(!html.contains("Finding r6"));
    assert!(html.contains("src/lib.rs:12"));
}

#[test]
fn worker_rows_are_toggles_with_pressed_state_and_progress_values() {
    let html = render(swarm);
    assert_eq!(html.matches("aria-pressed=\"false\"").count(), 3);
    assert!(html.contains("aria-valuenow=\"2\""));
    assert!(html.contains("aria-valuemax=\"6\""));
    assert!(html.contains("2/6"));
}

#[test]
fn feed_is_a_polite_live_region_and_severity_is_labelled_in_words() {
    let html = render(swarm);
    assert!(html.contains("aria-live=\"polite\""));
    assert!(html.contains("kit-probe__sev-label"));
    assert!(html.contains(">Warning<"));
}
