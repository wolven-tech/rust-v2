use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::parallel_span_timeline::{
    Decision, ParallelSpanTimeline, Placement, Unit, UnitPhase, WorkerRow, axis_ticks,
    connector_path, decision_outcome, effective_phase, format_ago, header_counts,
    held_announcement, placement, position_pct, range_text, row_units,
};

fn unit(key: &str, worker: &str, phase: UnitPhase, begin: u64, end: Option<u64>) -> Unit {
    Unit {
        unit_key: key.to_string(),
        worker_key: worker.to_string(),
        unit_label: format!("Unit {key}"),
        phase,
        begins_at_ms: begin,
        ends_at_ms: end,
        hands_off_to_key: None,
        decision_prompt: None,
        highlights: vec!["checked".to_string()],
    }
}

#[test]
fn unit_without_end_reaches_the_present_and_lengthens_with_it() {
    let open = unit("a", "w", UnitPhase::Active, 900_000, None);
    let early = placement(&open, 1_000_000, 600_000).expect("in window");
    let later = placement(&open, 1_100_000, 600_000).expect("in window");
    assert!((early.left - 83.33).abs() < 0.01);
    assert!((early.width - 16.67).abs() < 0.01);
    assert!(later.width > early.width);
    assert_eq!(later.width.round(), 33.0);
}

#[test]
fn unit_beginning_before_the_window_is_clipped_at_the_left_edge() {
    let old = unit("a", "w", UnitPhase::Settled, 100_000, Some(500_000));
    let placed: Placement = placement(&old, 1_000_000, 600_000).expect("overlaps window");
    assert!(placed.clipped);
    assert_eq!(placed.left, 0.0);
    assert!((placed.width - 16.67).abs() < 0.01);
}

#[test]
fn unit_wholly_outside_the_window_is_not_placed() {
    let gone = unit("g", "w", UnitPhase::Settled, 1, Some(300_000));
    assert!(placement(&gone, 1_000_000, 600_000).is_none());
    let future = unit("f", "w", UnitPhase::Active, 2_000_000, None);
    assert!(placement(&future, 1_000_000, 600_000).is_none());
}

#[test]
fn position_is_clamped_to_the_window() {
    assert_eq!(position_pct(0, 100, 100), 0.0);
    assert_eq!(position_pct(500, 100, 100), 100.0);
    assert_eq!(position_pct(150, 100, 100), 50.0);
}

#[test]
fn tab_order_within_a_row_is_by_time() {
    let units = vec![
        unit("late", "a", UnitPhase::Settled, 500, Some(600)),
        unit("early", "a", UnitPhase::Settled, 100, Some(200)),
        unit("other", "b", UnitPhase::Settled, 50, Some(60)),
    ];
    let keys: Vec<String> = row_units(&units, "a")
        .into_iter()
        .map(|u| u.unit_key)
        .collect();
    assert_eq!(keys, vec!["early".to_string(), "late".to_string()]);
}

#[test]
fn decision_without_handler_settles_or_breaks_the_unit() {
    assert_eq!(
        decision_outcome(Decision::Confirm, false),
        Some(UnitPhase::Settled)
    );
    assert_eq!(
        decision_outcome(Decision::Decline, false),
        Some(UnitPhase::Broken)
    );
    assert_eq!(decision_outcome(Decision::Confirm, true), None);
    let held = unit("h", "w", UnitPhase::Held, 0, Some(10));
    let overrides = vec![("h".to_string(), UnitPhase::Settled)];
    assert_eq!(effective_phase(&held, &overrides), UnitPhase::Settled);
    assert_eq!(effective_phase(&held, &[]), UnitPhase::Held);
}

#[test]
fn header_counts_active_and_held_units() {
    let units = vec![
        unit("a", "w", UnitPhase::Active, 0, None),
        unit("b", "w", UnitPhase::Held, 0, Some(5)),
        unit("c", "w", UnitPhase::Held, 0, Some(5)),
        unit("d", "w", UnitPhase::Settled, 0, Some(5)),
    ];
    assert_eq!(header_counts(&units, &[]), (1, 2));
    assert_eq!(held_announcement(2), "2 units waiting for input.");
    assert_eq!(held_announcement(0), "No units waiting for input.");
}

#[test]
fn ages_and_ranges_read_as_relative_times() {
    assert_eq!(format_ago(0), "now");
    assert_eq!(format_ago(45_000), "45s ago");
    assert_eq!(format_ago(360_000), "6m ago");
    let u = unit("r", "w", UnitPhase::Settled, 640_000, Some(900_000));
    assert_eq!(range_text(&u, 1_000_000), "from 6m ago to 1m ago");
}

#[test]
fn axis_runs_from_the_window_start_to_the_present() {
    let ticks = axis_ticks(600_000, 5);
    assert_eq!(ticks.len(), 6);
    assert_eq!(ticks[0], (0.0, "10m ago".to_string()));
    assert_eq!(ticks[4], (80.0, "2m ago".to_string()));
    assert_eq!(ticks[5], (100.0, "now".to_string()));
}

#[test]
fn hand_off_curve_runs_from_the_end_of_one_row_to_the_start_of_the_next() {
    let path = connector_path((60.0, 0), (65.0, 1));
    assert_eq!(path, "M 60.00 24.0 C 62.50 24.0 62.50 72.0 65.00 72.0");
}

fn board() -> Element {
    let mut handed = unit("u1", "a", UnitPhase::Settled, 450_000, Some(600_000));
    handed.hands_off_to_key = Some("u2".to_string());
    let mut held = unit("u2", "b", UnitPhase::Held, 620_000, Some(800_000));
    held.decision_prompt = Some("Ship the migration?".to_string());
    rsx! {
        ParallelSpanTimeline {
            worker_rows: vec![
                WorkerRow {
                    worker_key: "a".to_string(),
                    worker_label: "Builder".to_string(),
                    model_label: None,
                },
                WorkerRow {
                    worker_key: "b".to_string(),
                    worker_label: "Reviewer".to_string(),
                    model_label: None,
                },
                WorkerRow {
                    worker_key: "c".to_string(),
                    worker_label: "Scout".to_string(),
                    model_label: None,
                },
            ],
            units: vec![
                unit("u0", "a", UnitPhase::Settled, 100_000, Some(420_000)),
                handed,
                held,
                unit("u3", "a", UnitPhase::Active, 650_000, None),
            ],
            clock_ms: 1_000_000u64,
            on_confirm: move |_: Unit| {},
            on_decline: move |_: Unit| {},
        }
    }
}

#[test]
fn board_renders_one_labelled_group_per_worker() {
    let html = render(board);
    assert!(html.contains("aria-label=\"Builder\""));
    assert!(html.contains("aria-label=\"Reviewer\""));
    assert!(html.contains("data-worker=\"c\""));
    assert!(html.contains("Unassigned"));
    assert!(html.contains("Input needed"));
}

#[test]
fn held_unit_exposes_expanded_state_and_its_popover_stays_closed() {
    let html = render(board);
    assert!(html.contains("aria-haspopup=\"dialog\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("kit-span__popover"));
    assert!(!html.contains("Ship the migration?"));
}

#[test]
fn clipped_unit_is_marked_and_names_its_phase_and_range() {
    let html = render(board);
    assert!(html.contains("data-clipped=\"true\""));
    assert!(html.contains("Unit u2, Held, from"));
    assert!(html.contains("1 active"));
    assert!(html.contains("1 held"));
}

#[test]
fn hand_off_is_drawn_when_both_units_are_in_view() {
    let html = render(board);
    assert_eq!(html.matches("class=\"kit-span__link\"").count(), 1);
}

#[test]
fn held_count_is_announced_politely_and_units_are_buttons() {
    let html = render(board);
    assert!(html.contains("aria-live=\"polite\""));
    assert!(html.contains("<button"));
    assert!(html.contains(">now</span>"));
}

/// The amendment's gallery timeline: five workers, seven units, one hand-off, clock at 15 minutes.
fn gallery_board() -> Element {
    let mut scan = unit("u3", "w2", UnitPhase::Settled, 60_000, Some(420_000));
    scan.highlights = vec!["412 files indexed".to_string()];
    let mut rank = unit("u6", "w2", UnitPhase::Settled, 430_000, Some(560_000));
    rank.hands_off_to_key = Some("u4".to_string());
    let mut migration = unit("u4", "w3", UnitPhase::Held, 580_000, Some(800_000));
    migration.decision_prompt = Some("Apply the ledger schema change?".to_string());
    let units = vec![
        unit("u1", "w1", UnitPhase::Settled, 320_000, Some(500_000)),
        unit("u2", "w1", UnitPhase::Active, 520_000, None),
        scan,
        rank,
        migration,
        unit("u5", "w4", UnitPhase::Broken, 360_000, Some(600_000)),
        unit("u7", "w4", UnitPhase::Active, 620_000, None),
    ];
    let worker = |key: &str, label: &str| WorkerRow {
        worker_key: key.to_string(),
        worker_label: label.to_string(),
        model_label: None,
    };
    rsx! {
        ParallelSpanTimeline {
            worker_rows: vec![
                worker("w1", "Reader"),
                worker("w2", "Scout"),
                worker("w3", "Planner"),
                worker("w4", "Tester"),
                worker("w5", "Notifier"),
            ],
            units,
            window_span: 600_000u64,
            clock_ms: 900_000u64,
            on_confirm: move |_: Unit| {},
            on_decline: move |_: Unit| {},
        }
    }
}

#[test]
fn gallery_board_renders_seven_segments_across_five_rows() {
    let html = render(gallery_board);
    assert_eq!(html.matches("class=\"kit-span__row\"").count(), 5);
    assert_eq!(html.matches("class=\"kit-span__unit\"").count(), 7);
    let notifier = html
        .split("data-worker=\"w5\"")
        .nth(1)
        .expect("notifier row rendered");
    assert!(notifier.contains("Unassigned"));
    assert!(!notifier.contains("class=\"kit-span__unit\""));
}

#[test]
fn gallery_board_clips_scouts_first_unit_and_names_its_full_range() {
    let html = render(gallery_board);
    assert_eq!(html.matches("data-clipped=\"true\"").count(), 1);
    assert!(html.contains("Scout, Unit u3, Settled, from 14m ago to 8m ago"));
}

#[test]
fn gallery_board_header_counts_two_active_and_one_held_with_an_input_tag() {
    let html = render(gallery_board);
    assert!(html.contains("2 active"));
    assert!(html.contains("1 held"));
    assert_eq!(html.matches("Input needed").count(), 1);
}

#[test]
fn gallery_board_draws_one_hand_off_from_scouts_second_unit_to_planners() {
    let html = render(gallery_board);
    assert_eq!(html.matches("class=\"kit-span__link\"").count(), 1);
}

#[test]
fn gallery_board_axis_runs_every_two_minutes_and_ends_at_now_on_the_right_edge() {
    let html = render(gallery_board);
    for label in ["10m ago", "8m ago", "6m ago", "4m ago", "2m ago"] {
        assert!(
            html.contains(&format!(">{label}</span>")),
            "missing {label}"
        );
    }
    assert!(html.contains("left: 100.00%\">now</span>"));
}

#[test]
fn gallery_board_keys_each_unit_state_in_a_legend() {
    let html = render(gallery_board);
    for key in ["settled", "active", "held", "broken"] {
        assert_eq!(html.matches(&format!("data-key=\"{key}\"")).count(), 1);
    }
}
