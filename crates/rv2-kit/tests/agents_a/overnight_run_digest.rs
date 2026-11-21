use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::overnight_run_digest::{
    CardOverride, CardStage, DigestEvent, DigestLane, MainAction, OvernightRunDigest, Transition,
    band_edges, base_stage, chart_paths, chart_points, clock_utc, dom_slug, fraction, iso_utc,
    lane_cards, lane_total, main_label, stage_of, transition, visible_events,
};

fn event(key: &str, lane: DigestLane, at: Option<u64>) -> DigestEvent {
    DigestEvent {
        event_key: key.to_string(),
        happened_at: at,
        event_lane: lane,
        event_title: format!("Event {key}"),
        event_detail: None,
        repeat_count: None,
        main_action: Some(MainAction {
            action_label: "Approve".to_string(),
        }),
        inspect_available: true,
        past_tense_label: Some("Approved".to_string()),
        reversible: false,
    }
}

#[test]
fn each_lane_counts_every_event_but_shows_at_most_the_limit() {
    let events: Vec<DigestEvent> = (0..5)
        .map(|n| {
            event(
                &format!("d{n}"),
                DigestLane::AwaitingDecision,
                Some(1_000 * n),
            )
        })
        .collect();
    assert_eq!(lane_total(&events, DigestLane::AwaitingDecision), 5);
    let shown = lane_cards(&events, DigestLane::AwaitingDecision, 3);
    assert_eq!(shown.len(), 3);
    assert_eq!(shown[0].event_key, "d4");
}

#[test]
fn events_with_unreadable_times_are_omitted() {
    let events = vec![
        event("a", DigestLane::Completed, Some(5)),
        event("b", DigestLane::Completed, None),
    ];
    let visible = visible_events(&events);
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].event_key, "a");
}

#[test]
fn events_outside_the_window_sit_at_the_chart_edges() {
    assert_eq!(fraction(0, 100, 200), 0.0);
    assert_eq!(fraction(500, 100, 200), 1.0);
    assert_eq!(fraction(150, 100, 200), 0.5);
    assert_eq!(fraction(150, 200, 200), 1.0);
}

#[test]
fn merged_card_raises_its_point_on_the_chart() {
    let mut merged = event("m", DigestLane::Errored, Some(20));
    merged.repeat_count = Some(3);
    let events = vec![merged, event("s", DigestLane::Errored, Some(30))];
    let points = chart_points(&events, 0, 40, 400.0, 100.0);
    assert_eq!(points[0].event_key, "m");
    assert!((points[0].y - 25.0).abs() < 1e-9);
    assert!((points[1].y - 0.0).abs() < 1e-9);
}

#[test]
fn chart_path_closes_to_the_baseline() {
    let events = vec![event("a", DigestLane::Completed, Some(10))];
    let points = chart_points(&events, 0, 10, 100.0, 50.0);
    let (line, area) = chart_paths(&points, 50.0);
    assert_eq!(line, "M 100.0 0.0");
    assert_eq!(area, "M 100.0 0.0 L 100.0 50.0 L 100.0 50.0 Z");
}

#[test]
fn pointing_in_a_band_snaps_to_its_nearest_event() {
    let edges = band_edges(&[10.0, 50.0, 100.0], 120.0);
    assert_eq!(edges, vec![(0.0, 30.0), (30.0, 75.0), (75.0, 120.0)]);
}

#[test]
fn pending_action_leaves_card_in_prior_state_on_rejection() {
    let stage = CardStage::Awaiting;
    assert_eq!(
        stage_of(&event("x", DigestLane::AwaitingDecision, Some(1)), &[]).0,
        stage
    );
    assert_eq!(
        base_stage(&event("x", DigestLane::Errored, Some(1))),
        CardStage::Errored
    );
}

#[test]
fn successful_main_action_moves_to_acted_and_focus_goes_to_revert() {
    let next = transition(
        CardStage::Awaiting,
        CardStage::Awaiting,
        Transition::Act,
        false,
    );
    assert_eq!(next, Some((CardStage::Acted, CardStage::Awaiting)));
    let retried = transition(
        CardStage::Errored,
        CardStage::Errored,
        Transition::Act,
        false,
    );
    assert_eq!(retried, Some((CardStage::Acted, CardStage::Errored)));
}

#[test]
fn revert_then_reapply_restores_the_prior_state() {
    let reverted = transition(
        CardStage::Acted,
        CardStage::Awaiting,
        Transition::Revert,
        false,
    );
    assert_eq!(reverted, Some((CardStage::Reverted, CardStage::Acted)));
    let back = transition(
        CardStage::Reverted,
        CardStage::Acted,
        Transition::Reapply,
        false,
    );
    assert_eq!(back, Some((CardStage::Acted, CardStage::Reverted)));
}

#[test]
fn completed_card_reverts_only_when_reversible() {
    assert_eq!(
        transition(
            CardStage::Completed,
            CardStage::Completed,
            Transition::Revert,
            false
        ),
        None
    );
    assert_eq!(
        transition(
            CardStage::Completed,
            CardStage::Completed,
            Transition::Revert,
            true
        ),
        Some((CardStage::Reverted, CardStage::Completed))
    );
}

#[test]
fn override_wins_over_lane_stage() {
    let event = event("o", DigestLane::Errored, Some(1));
    let overrides = vec![CardOverride {
        event_key: "o".to_string(),
        stage: CardStage::Acted,
        prior: CardStage::Errored,
    }];
    assert_eq!(
        stage_of(&event, &overrides),
        (CardStage::Acted, CardStage::Errored)
    );
}

#[test]
fn errored_main_action_defaults_to_retry() {
    let mut errored = event("e", DigestLane::Errored, Some(1));
    errored.main_action = None;
    assert_eq!(
        main_label(&errored, CardStage::Errored),
        Some("Retry".to_string())
    );
    assert_eq!(main_label(&errored, CardStage::Completed), None);
}

#[test]
fn timestamps_render_as_utc_with_machine_readable_date() {
    assert_eq!(iso_utc(0), "1970-01-01T00:00:00Z");
    assert_eq!(iso_utc(1_700_000_000_000), "2023-11-14T22:13:20Z");
    assert_eq!(clock_utc(1_700_000_000_000), "22:13 UTC");
    assert_eq!(dom_slug("run:7/a"), "run-7-a");
}

fn digest() -> Element {
    let mut merged = event("err-1", DigestLane::Errored, Some(2_000_000));
    merged.repeat_count = Some(4);
    let mut done = event("done-1", DigestLane::Completed, Some(3_000_000));
    done.reversible = true;
    rsx! {
        OvernightRunDigest {
            digest_events: vec![
                event("dec-1", DigestLane::AwaitingDecision, Some(1_000_000)),
                event("dec-2", DigestLane::AwaitingDecision, Some(1_500_000)),
                event("dec-3", DigestLane::AwaitingDecision, Some(1_600_000)),
                event("dec-4", DigestLane::AwaitingDecision, Some(1_700_000)),
                merged,
                done,
                event("bad", DigestLane::Completed, None),
            ],
            away_since: 500_000u64,
            cards_per_lane: 3usize,
        }
    }
}

#[test]
fn lanes_show_full_counts_and_cap_the_cards() {
    let html = render(digest);
    assert!(html.contains("Decisions (4)"));
    assert!(html.contains("Errors (1)"));
    assert!(html.contains("Completed (1)"));
    assert!(html.contains("Since you last checked"));
    assert_eq!(html.matches("kit-digest__card\"").count(), 3 + 1 + 1);
    assert!(!html.contains("dec-1\""));
    assert!(!html.contains("bad"));
}

#[test]
fn chart_is_hidden_from_assistive_tech_and_cards_carry_machine_dates() {
    let html = render(digest);
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("datetime=\"1970-01-01T00:33:20Z\""));
    assert!(html.contains("4 similar events merged"));
    assert!(html.contains("aria-live=\"polite\""));
}

#[test]
fn replay_switch_is_exposed_on_the_root() {
    let html = render(digest);
    assert!(html.contains("data-replay=\"true\""));
}

fn no_replay() -> Element {
    rsx! {
        OvernightRunDigest {
            digest_events: Vec::<DigestEvent>::new(),
            away_since: 0u64,
            replay_on_open: false,
        }
    }
}

#[test]
fn empty_digest_states_each_lane_and_turns_replay_off() {
    let html = render(no_replay);
    assert!(html.contains("data-replay=\"false\""));
    assert!(html.contains("Nothing is waiting on you."));
    assert!(html.contains("No errors since you left."));
    assert!(html.contains("Nothing finished since you left."));
}
