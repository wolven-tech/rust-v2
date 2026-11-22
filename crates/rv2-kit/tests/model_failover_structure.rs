use dioxus::prelude::*;
use rv2_kit::components::model_failover_flow::{Attempt, AttemptState, End, ModelFailoverFlow};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn attempt(
    id: &str,
    model: &str,
    state: AttemptState,
    error: Option<&str>,
    cost: Option<f64>,
) -> Attempt {
    Attempt {
        attempt_id: id.into(),
        model_name: model.into(),
        provider_name: "Provider".into(),
        attempt_state: state,
        duration: Some(1_300.0),
        attempt_cost: cost,
        error_text: error.map(str::to_string),
    }
}

fn demo_attempts() -> Vec<Attempt> {
    vec![
        attempt(
            "a1",
            "Model A",
            AttemptState::Failed,
            Some("429 rate limited"),
            Some(0.02),
        ),
        attempt("a2", "Model B", AttemptState::InProgress, None, Some(0.02)),
        attempt("a3", "Model C", AttemptState::Queued, None, Some(0.02)),
        attempt("a4", "Model D", AttemptState::Queued, None, None),
    ]
}

fn flow(attempts: Vec<Attempt>) -> Element {
    rsx! {
        ModelFailoverFlow {
            attempt_list: attempts,
            heading_text: "Reply request",
            routing_label: Some("Primary then fallback".to_string()),
            origin_node: Some(End { node_name: "Request".into(), node_mark: None, node_text: Some("User asked".into()), node_details: vec![] }),
            result_node: Some(End { node_name: "Reply".into(), node_mark: None, node_text: Some("Delivered".into()), node_details: vec!["Model B".into()] }),
            elapsed_ms: Some(2_600.0),
        }
    }
}

fn demo() -> Element {
    flow(demo_attempts())
}

fn answered() -> Element {
    let mut list = demo_attempts();
    list[1].attempt_state = AttemptState::Answered;
    list[2].attempt_state = AttemptState::Skipped;
    list[3].attempt_state = AttemptState::Skipped;
    flow(list)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

#[test]
fn flow_lays_out_request_then_attempts_then_result_in_dom_order() {
    let html = render(demo);
    let origin = html.find("kit-mfl__origin").expect("request node rendered");
    let attempts = html
        .find("kit-mfl__attempts")
        .expect("attempt stack rendered");
    let result = html.find("kit-mfl__result").expect("result node rendered");
    assert!(origin < attempts && attempts < result);
}

#[test]
fn demo_shows_four_attempts_with_named_states_at_rest() {
    let html = render(demo);
    assert_eq!(count(&html, "class=\"kit-mfl__attempt\""), 4);
    assert!(html.contains("429 rate limited"));
    assert_eq!(count(&html, "kit-mfl__state\">Running<"), 1);
    assert_eq!(count(&html, "kit-mfl__state\">Up next<"), 2);
}

#[test]
fn result_node_shows_waiting_placeholder_and_no_preview_at_rest() {
    let html = render(demo);
    assert!(html.contains("Waiting for an answer"));
    assert_eq!(count(&html, "kit-mfl__preview"), 0);
}

#[test]
fn attempt_cards_carry_provider_state_time_and_cost_except_the_costless_one() {
    let html = render(demo);
    assert_eq!(count(&html, "class=\"kit-mfl__provider\""), 4);
    assert_eq!(count(&html, "class=\"kit-mfl__state\""), 4);
    assert_eq!(count(&html, "kit-mfl__meta\">$0.0200<"), 3);
    assert!(!html.contains("$0.0000"));
}

#[test]
fn connector_to_result_appears_only_after_an_answer() {
    let running = render(demo);
    assert_eq!(count(&running, "class=\"kit-mfl__wire\""), 4);
    assert_eq!(count(&running, "data-leg=\"out\""), 0);
    let done = render(answered);
    assert_eq!(count(&done, "data-leg=\"out\""), 1);
    assert_eq!(count(&done, "kit-mfl__preview"), 1);
}

#[test]
fn summary_orders_position_elapsed_time_and_total_cost() {
    let html = render(demo);
    let position = html.find("Attempt 2 of 4").expect("position rendered");
    let elapsed = html.find("2.6 s").expect("elapsed rendered");
    let cost = html.find("kit-mfl__cost").expect("cost rendered");
    assert!(position < elapsed && elapsed < cost);
}

#[test]
fn demo_at_rest_shows_no_retry_control() {
    let html = render(demo);
    assert!(!html.contains("kit-mfl__retry"));
}
