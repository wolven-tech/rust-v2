//! Amendment checks for the ranking bracket and the candidate race board: the states the gallery shows at rest,
//! the answers and labels a judge or a person reads, and the bracket route.

use dioxus::prelude::*;
use rv2_kit::components::candidate_race_board::{Candidate, CandidateRaceBoard, RunState};
use rv2_kit::components::ranking_bracket::{
    Entrant, RankingBracket, Verdict, champion, resolve_bracket,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn candidate(
    key: &str,
    name: &str,
    state: RunState,
    fraction: f64,
    score: Option<f64>,
    note: &str,
) -> Candidate {
    Candidate {
        candidate_key: key.into(),
        display_name: name.into(),
        method_summary: "Replays the same input".into(),
        model_id: Some("model A".into()),
        mark: None,
        run_state: state,
        fraction_done: Some(fraction),
        current_action: Some(note.into()),
        quality_score: score,
        outcome_note: (state != RunState::Running).then(|| note.to_string()),
        elapsed_secs: Some(12.0),
        run_cost: Some(0.04),
    }
}

fn at_rest_candidates() -> Vec<Candidate> {
    vec![
        candidate(
            "retry",
            "Retry queue",
            RunState::Running,
            0.2,
            None,
            "Calling the gateway",
        ),
        candidate(
            "mock",
            "Mock the gateway",
            RunState::Running,
            0.45,
            None,
            "Writing fixtures",
        ),
        candidate(
            "settle",
            "Wait for settle",
            RunState::Running,
            0.7,
            None,
            "Checking status",
        ),
    ]
}

fn mixed_candidates() -> Vec<Candidate> {
    vec![
        candidate(
            "retry",
            "Retry queue",
            RunState::Finished,
            1.0,
            Some(72.0),
            "Passes all 14 checks",
        ),
        candidate(
            "settle",
            "Wait for settle",
            RunState::Finished,
            1.0,
            Some(64.0),
            "Passes 12 of 14 checks",
        ),
        candidate(
            "mock",
            "Mock the gateway",
            RunState::Halted,
            0.35,
            None,
            "Error: fixture server refused connection",
        ),
        candidate(
            "clock",
            "Stub the clock",
            RunState::Running,
            0.6,
            Some(55.0),
            "Advancing time",
        ),
    ]
}

fn at_rest_app() -> Element {
    rsx! {
        CandidateRaceBoard {
            candidate_list: at_rest_candidates(),
            task_text: "Stop duplicate charges when a payment is retried",
            board_title: Some("Best of 3".to_string()),
        }
    }
}

fn mixed_app() -> Element {
    rsx! {
        CandidateRaceBoard {
            candidate_list: mixed_candidates(),
            task_text: "Stop duplicate charges when a payment is retried",
            board_title: Some("Best of 4".to_string()),
        }
    }
}

#[test]
fn race_at_rest_shows_three_running_rows_with_percent_text_and_no_choice() {
    let html = render(at_rest_app);
    assert_eq!(html.matches("kit-crb__row\"").count(), 3, "{html}");
    for text in [
        "20%",
        "45%",
        "70%",
        "Calling the gateway",
        "0 of 3 finished",
    ] {
        assert!(html.contains(text), "missing {text}: {html}");
    }
    assert!(!html.contains("data-best=\"true\""), "{html}");
    assert!(!html.contains("type=\"radio\""), "{html}");
}

#[test]
fn race_marks_the_highest_finished_row_best_and_in_use() {
    let html = render(mixed_app);
    assert_eq!(html.matches("data-best=\"true\"").count(), 1, "{html}");
    assert_eq!(html.matches("data-in-use=\"true\"").count(), 1, "{html}");
    let best_at = html.find("data-best=\"true\"").expect("best row");
    assert!(
        best_at < html.find("Wait for settle").expect("second row"),
        "{html}"
    );
    let name_at = html.find("Retry queue").expect("first row");
    assert!(
        best_at < name_at && name_at < html.find("Wait for settle").expect("second row"),
        "{html}"
    );
    assert!(
        html.contains(">best<") && html.contains(">in use<"),
        "{html}"
    );
}

#[test]
fn halted_row_keeps_its_percent_and_shows_its_error_note() {
    let html = render(mixed_app);
    assert!(html.contains("35%"), "{html}");
    assert!(
        html.contains("Error: fixture server refused connection"),
        "{html}"
    );
}

#[test]
fn only_finished_rows_offer_a_choice_radio_and_scores_stay_on_them() {
    let html = render(mixed_app);
    assert_eq!(html.matches("type=\"radio\"").count(), 2, "{html}");
    assert!(
        html.contains("aria-label=\"Use Retry queue, score 72\""),
        "{html}"
    );
    assert!(
        !html.contains("55"),
        "running row must not show its score: {html}"
    );
    assert!(
        html.contains("Only finished candidates can be chosen."),
        "{html}"
    );
}

fn entrant(id: &str, name: &str) -> Entrant {
    Entrant {
        entrant_id: id.into(),
        entrant_name: name.into(),
        entrant_meta: Some("3 files".into()),
        entrant_text: Some(format!("Output from {name}")),
        ..Default::default()
    }
}

fn eight_entrants() -> Vec<Entrant> {
    [
        ("a", "Dedupe table"),
        ("b", "Outbox"),
        ("c", "Queue retry"),
        ("d", "Token guard"),
        ("e", "Client retry"),
        ("f", "Manual review"),
        ("g", "Saga"),
        ("h", "Lock per order"),
    ]
    .into_iter()
    .map(|(id, name)| entrant(id, name))
    .collect()
}

fn verdict(winner: &str, upper: f64, lower: f64) -> Verdict {
    Verdict {
        winner_id: winner.into(),
        rationale: Some("Smallest change that stops the second charge.".into()),
        scores: Some((upper, lower)),
    }
}

fn quarterfinal_verdicts() -> Vec<(String, Verdict)> {
    vec![
        ("0:0".to_string(), verdict("a", 82.0, 74.0)),
        ("0:1".to_string(), verdict("d", 61.0, 77.0)),
        ("0:2".to_string(), verdict("e", 70.0, 45.0)),
        ("0:3".to_string(), verdict("h", 58.0, 80.0)),
    ]
}

fn in_progress_verdicts() -> Vec<(String, Verdict)> {
    let mut verdicts = quarterfinal_verdicts();
    verdicts.push(("1:0".to_string(), verdict("a", 85.0, 79.0)));
    verdicts
}

fn complete_verdicts() -> Vec<(String, Verdict)> {
    let mut verdicts = in_progress_verdicts();
    verdicts.push(("1:1".to_string(), verdict("h", 71.0, 68.0)));
    verdicts.push(("2:0".to_string(), verdict("a", 90.0, 76.0)));
    verdicts
}

fn in_progress_app() -> Element {
    rsx! {
        RankingBracket {
            entrants: eight_entrants(),
            verdicts: in_progress_verdicts(),
            pairing_in_flight: Some("1:1".to_string()),
            open_pairing: Some("0:1".to_string()),
            judge_label: "Judge model B".to_string(),
            brief: "Stop duplicate charges when a payment is retried.".to_string(),
        }
    }
}

fn complete_app() -> Element {
    rsx! {
        RankingBracket {
            entrants: eight_entrants(),
            verdicts: complete_verdicts(),
            judge_label: "Judge model B".to_string(),
            on_adopt: move |_: Entrant| {},
        }
    }
}

fn complete_without_adopt_app() -> Element {
    rsx! {
        RankingBracket {
            entrants: eight_entrants(),
            verdicts: complete_verdicts(),
        }
    }
}

#[test]
fn in_progress_bracket_has_no_winner_beyond_its_last_settled_pairing() {
    let rounds = resolve_bracket(&eight_entrants(), &in_progress_verdicts());
    assert_eq!(champion(&rounds), None);
    assert_eq!(rounds[1][0].winner.as_deref(), Some("a"));
    assert_eq!(rounds[1][1].winner, None);
    assert_eq!(rounds[2][0].winner, None);
}

#[test]
fn complete_bracket_shows_a_trophy_and_the_adopt_action_for_the_final_winner() {
    let html = render(complete_app);
    assert!(html.contains("kit-rb__trophy"), "{html}");
    assert!(html.contains("Winner Dedupe table"), "{html}");
    assert!(html.contains("Use Dedupe table"), "{html}");
}

#[test]
fn complete_bracket_without_an_adopt_handler_renders_no_adopt_action() {
    let html = render(complete_without_adopt_app);
    assert!(html.contains("kit-rb__trophy"), "{html}");
    assert!(!html.contains("kit-rb__adopt"), "{html}");
}

#[test]
fn open_detail_shows_both_answers_with_scores_and_rationale_in_order() {
    let html = render(in_progress_app);
    let detail_at = html.find("kit-rb__detail").expect("detail panel");
    let detail = &html[detail_at..];
    assert_eq!(detail.matches("kit-rb__answer\"").count(), 2, "{html}");
    let upper_at = detail.find("Score 61").expect("upper score");
    let lower_at = detail.find("Score 77").expect("lower score");
    assert!(upper_at < lower_at, "{html}");
    assert!(
        detail.contains("Smallest change that stops the second charge."),
        "{html}"
    );
}

#[test]
fn pairing_label_names_its_round_entrants_and_winner() {
    let html = render(complete_app);
    assert!(
        html.contains(
            "aria-label=\"Quarterfinal, Dedupe table against Outbox, winner Dedupe table\""
        ),
        "{html}"
    );
}

#[test]
fn judged_pairing_is_named_as_being_judged_and_the_status_names_it() {
    let html = render(in_progress_app);
    assert!(
        html.contains(
            "aria-label=\"Semifinal, Client retry against Lock per order, being judged\""
        ),
        "{html}"
    );
    assert!(
        html.contains("Judging Client retry against Lock per order"),
        "{html}"
    );
    assert!(!html.contains("kit-rb__adopt"), "{html}");
}
