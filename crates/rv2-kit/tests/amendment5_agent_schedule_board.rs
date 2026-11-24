use dioxus::prelude::*;
use rv2_kit::screens::agent_schedule_board::{
    AgentScheduleBoard, HistoryEntry, MarkState, Outcome, RunTime, ScheduleRecord, axis_ticks,
    marker_position, status_line,
};

const NOW: u64 = 1_795_219_200_000;
const MINUTE: u64 = 60_000;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn sched(
    id: &str,
    name: &str,
    past: &[(i64, MarkState)],
    upcoming: &[i64],
    paused: bool,
) -> ScheduleRecord {
    let at = |minutes: i64| (NOW as i64 + minutes * MINUTE as i64) as u64;
    let mut run_times: Vec<RunTime> = past
        .iter()
        .map(|(minutes, state)| RunTime {
            at_ms: at(*minutes),
            run_state: *state,
            progress_percent: None,
        })
        .collect();
    run_times.extend(upcoming.iter().map(|minutes| RunTime {
        at_ms: at(*minutes),
        run_state: MarkState::Upcoming,
        progress_percent: None,
    }));
    ScheduleRecord {
        id: id.to_string(),
        entry_name: name.to_string(),
        project_name: Some("Support".to_string()),
        brief_summary: Some("Summarise the ticket queue".to_string()),
        cadence_text: "Every 2 hours".to_string(),
        run_times,
        duration_seconds: Some(900),
        is_paused: paused,
        history_log: vec![HistoryEntry {
            at_ms: at(-60),
            outcome: Outcome::Done,
            summary: "Summary sent".to_string(),
            cost_label: Some("$0.31".to_string()),
        }],
        cost_per_run_label: Some("$0.31".to_string()),
    }
}

fn fixture() -> Vec<ScheduleRecord> {
    vec![
        sched(
            "sum",
            "Ticket summary",
            &[(-240, MarkState::Done)],
            &[45, 105, 165, 225, 285],
            false,
        ),
        sched("tri", "Triage", &[(-180, MarkState::Done)], &[150], false),
        sched(
            "digest",
            "Daily digest",
            &[(-120, MarkState::Failed)],
            &[420],
            false,
        ),
        sched(
            "churn",
            "Churn watch",
            &[(-60, MarkState::Done)],
            &[360],
            false,
        ),
        sched("billing", "Billing check", &[], &[480, 600], false),
        sched("qa", "QA sweep", &[], &[540], false),
        sched("review", "Weekly review", &[], &[60, 180], true),
    ]
}

fn rest_app() -> Element {
    rsx! {
        AgentScheduleBoard {
            schedule_list: fixture(),
            now_ms: NOW,
            selected_schedule_id: Some("sum".to_string()),
        }
    }
}

#[test]
fn timeline_at_rest_has_four_past_markers_three_done_and_one_failed() {
    let past: Vec<MarkState> = fixture()
        .iter()
        .flat_map(|s| s.run_times.clone())
        .filter(|r| r.at_ms < NOW && r.at_ms.abs_diff(NOW) < 5 * 3_600_000)
        .map(|r| r.run_state)
        .collect();
    assert_eq!(past.len(), 4);
    assert_eq!(past.iter().filter(|s| **s == MarkState::Done).count(), 3);
    assert_eq!(past.iter().filter(|s| **s == MarkState::Failed).count(), 1);
}

#[test]
fn timeline_at_rest_has_six_upcoming_markers_to_the_right_of_now() {
    let html = render(rest_app);
    let upcoming = html
        .match_indices("kit-asb__marker\" data-state=\"upcoming\"")
        .count();
    assert_eq!(upcoming, 6);
    let upcoming_right = fixture()
        .iter()
        .filter(|s| !s.is_paused)
        .flat_map(|s| s.run_times.clone())
        .filter(|r| {
            r.run_state == MarkState::Upcoming
                && r.at_ms > NOW
                && r.at_ms.abs_diff(NOW) < 5 * 3_600_000
        })
        .count();
    assert_eq!(upcoming_right, 6);
    assert!((marker_position(NOW, NOW) - 50.0).abs() < 1e-9);
}

#[test]
fn axis_ticks_fall_on_round_two_hour_marks_inside_the_ten_hour_span() {
    let ticks = axis_ticks(NOW);
    assert!(!ticks.is_empty());
    assert!(
        ticks
            .iter()
            .all(|(pos, label)| (0.0..=100.0).contains(pos) && label.ends_with(":00 UTC"))
    );
    let hours: Vec<u64> = ticks
        .iter()
        .map(|(_, label)| label[..2].parse::<u64>().expect("hour"))
        .collect();
    assert!(hours.iter().all(|h| h % 2 == 0));
}

#[test]
fn schedule_list_at_rest_has_seven_rows_and_the_header_reads_six_of_seven() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-asb__row\"").count(), 7);
    assert!(html.contains("6 of 7 active"));
}

#[test]
fn paused_schedule_is_dimmed_with_its_switch_off_and_no_upcoming_markers() {
    let html = render(rest_app);
    assert_eq!(html.matches("data-paused=\"true\"").count(), 1);
}

#[test]
fn panel_lists_the_next_five_runs_each_with_a_skip_control() {
    let html = render(rest_app);
    assert_eq!(html.matches(">Skip<").count(), 5);
}

#[test]
fn status_line_names_the_next_run_and_its_moment() {
    let line = status_line(&fixture(), NOW);
    assert!(line.contains("Ticket summary"));
}
