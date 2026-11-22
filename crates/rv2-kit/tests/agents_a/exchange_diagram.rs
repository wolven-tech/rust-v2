use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::exchange_diagram::{
    Actor, ActorRole, Exchange, ExchangeDiagram, ExchangeKind, ExchangeState, NavKey,
    announce_exchange, clock_text, exchange_name, group_label_starts, next_index,
    open_request_keys, resolve_columns, run_span_ms, tint_for,
};

fn actor(key: &str, role: ActorRole, caption: &str) -> Actor {
    Actor {
        actor_key: key.to_string(),
        role,
        caption: caption.to_string(),
        mark_source: None,
        tint_key: None,
        model_tag: None,
        summary: None,
    }
}

fn exchange(key: &str, from: &str, to: &str, kind: ExchangeKind, headline: &str) -> Exchange {
    Exchange {
        exchange_key: key.to_string(),
        from_key: from.to_string(),
        to_key: to.to_string(),
        kind,
        answers_key: None,
        state: ExchangeState::Ok,
        sent_at_ms: None,
        group_tag: None,
        headline: headline.to_string(),
        detail_text: None,
    }
}

#[test]
fn negative_times_read_as_zero_and_hours_show_past_one_hour() {
    assert_eq!(clock_text(-5_000), "0:00");
    assert_eq!(clock_text(75_000), "1:15");
    assert_eq!(clock_text(3_725_000), "1:02:05");
}

#[test]
fn exchange_naming_an_unknown_actor_has_no_columns() {
    let actors = vec![actor("lead", ActorRole::Agent, "Lead")];
    let known = exchange("a", "lead", "lead", ExchangeKind::Request, "Self");
    let lost = exchange("b", "lead", "ghost", ExchangeKind::Request, "Lost");
    assert_eq!(resolve_columns(&known, &actors), Some((0, 0)));
    assert_eq!(resolve_columns(&lost, &actors), None);
}

#[test]
fn tints_go_person_then_primary_agent_then_slots_then_tool() {
    let actors = vec![
        actor("you", ActorRole::Person, "You"),
        actor("lead", ActorRole::Agent, "Lead"),
        actor("coder", ActorRole::Agent, "Coder"),
        actor("grep", ActorRole::Tool, "Search"),
    ];
    assert_eq!(
        tint_for(&actors),
        vec!["person", "primary", "slot-1", "tool"]
    );
}

#[test]
fn a_request_with_no_answer_is_still_open() {
    let mut answered = exchange("r1", "you", "lead", ExchangeKind::Request, "Plan");
    answered.state = ExchangeState::Ok;
    let mut answer = exchange("a1", "lead", "you", ExchangeKind::Answer, "Planned");
    answer.answers_key = Some("r1".to_string());
    let open = exchange("r2", "lead", "coder", ExchangeKind::Request, "Build");
    let all = vec![answered, answer, open];
    assert_eq!(open_request_keys(&all), vec!["r2".to_string()]);
}

#[test]
fn group_frames_start_at_the_first_row_of_each_tagged_run() {
    let tags = vec![
        Some("retry".to_string()),
        Some("retry".to_string()),
        None,
        Some("parallel".to_string()),
    ];
    assert_eq!(group_label_starts(&tags), vec![true, false, false, true]);
}

#[test]
fn navigation_keys_move_within_the_rows() {
    assert_eq!(next_index(None, 3, NavKey::Down), Some(0));
    assert_eq!(next_index(None, 3, NavKey::Up), Some(2));
    assert_eq!(next_index(Some(0), 3, NavKey::Down), Some(1));
    assert_eq!(next_index(Some(2), 3, NavKey::Down), Some(2));
    assert_eq!(next_index(Some(0), 3, NavKey::Up), Some(0));
    assert_eq!(next_index(Some(1), 3, NavKey::Home), Some(0));
    assert_eq!(next_index(Some(1), 3, NavKey::End), Some(2));
    assert_eq!(next_index(None, 0, NavKey::Down), None);
}

#[test]
fn row_name_says_route_headline_state_and_time() {
    assert_eq!(
        exchange_name(
            "Coder",
            "Lead",
            "Crash",
            ExchangeState::Failed,
            Some("0:01")
        ),
        "Coder to Lead, Crash, Failed, 0:01. Opens a card."
    );
    assert_eq!(
        exchange_name("Lead", "You", "Plan", ExchangeState::AwaitingPerson, None),
        "Lead to You, Plan, Needs your reply. Opens a card."
    );
}

#[test]
fn run_span_runs_from_the_earliest_to_the_latest_time() {
    let mut a = exchange("a", "x", "y", ExchangeKind::Request, "A");
    a.sent_at_ms = Some(5_000);
    let mut b = exchange("b", "x", "y", ExchangeKind::Request, "B");
    b.sent_at_ms = Some(1_000);
    let c = exchange("c", "x", "y", ExchangeKind::Request, "C");
    assert_eq!(run_span_ms(&[a, c, b]), Some((1_000, 5_000)));
}

#[test]
fn live_line_voices_the_new_exchange() {
    assert_eq!(
        announce_exchange("Lead", "Coder", "Build"),
        "Lead to Coder: Build"
    );
}

fn actors() -> Vec<Actor> {
    vec![
        actor("you", ActorRole::Person, "You"),
        actor("lead", ActorRole::Agent, "Lead"),
        actor("coder", ActorRole::Agent, "Coder"),
    ]
}

fn run() -> Element {
    let mut plan = exchange("e1", "you", "lead", ExchangeKind::Request, "Plan");
    plan.sent_at_ms = Some(0);
    let mut build = exchange("e2", "lead", "coder", ExchangeKind::Request, "Build");
    build.sent_at_ms = Some(1_000);
    build.group_tag = Some("retry".to_string());
    let mut crash = exchange("e3", "coder", "lead", ExchangeKind::Answer, "Crash");
    crash.answers_key = Some("e2".to_string());
    crash.state = ExchangeState::Failed;
    crash.sent_at_ms = Some(2_000);
    crash.group_tag = Some("retry".to_string());
    let lost = exchange("e4", "lead", "ghost", ExchangeKind::Request, "Lost");
    rsx! {
        ExchangeDiagram {
            actors: actors(),
            exchanges: vec![plan, build, crash, lost],
            run_heading: "Run 12",
        }
    }
}

#[test]
fn diagram_lists_exchanges_in_order_and_drops_unknown_actors() {
    let html = render(run);
    assert!(html.contains("1 working · 3 messages"));
    assert!(html.contains("data-kind=\"answer\""));
    assert!(html.contains("data-direction=\"backward\""));
    assert!(!html.contains("Lost"));
    assert!(html.contains("Run 12"));
}

#[test]
fn a_failed_exchange_says_failed_and_shows_a_cross() {
    let html = render(run);
    assert!(html.contains("Coder to Lead, Crash, Failed, 0:02. Opens a card."));
    assert!(html.contains("kit-agsq__cross"));
}

#[test]
fn rows_open_as_dialogs_and_the_group_tag_is_text() {
    let html = render(run);
    assert!(html.contains("aria-haspopup=\"dialog\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("kit-agsq__group"));
    assert!(html.contains(">retry<"));
    assert!(html.contains("aria-live=\"polite\""));
}

#[test]
fn actor_chips_open_cards_and_lifelines_show_without_exchanges() {
    let html = render(run);
    assert!(html.contains("kit-agsq__chip"));
    assert!(html.contains("kit-agsq__life"));
    assert!(!html.contains("kit-agsq__card--actor"));
}

fn custom() -> Element {
    let mut plan = exchange("e1", "you", "lead", ExchangeKind::Request, "Plan");
    plan.sent_at_ms = Some(1_000);
    rsx! {
        ExchangeDiagram {
            actors: actors(),
            exchanges: vec![plan],
            format_clock: Callback::new(|ms: i64| format!("T+{ms}")),
        }
    }
}

#[test]
fn a_host_clock_formatter_replaces_the_default() {
    let html = render(custom);
    assert!(html.contains("T+1000"));
}

fn empty() -> Element {
    rsx! {
        ExchangeDiagram {
            actors: vec![actor("you", ActorRole::Person, "You")],
            exchanges: Vec::new(),
        }
    }
}

#[test]
fn empty_run_shows_heading_and_chips_but_no_rows() {
    let html = render(empty);
    assert!(html.contains("No exchanges yet."));
    assert!(!html.contains("kit-agsq__row"));
}
