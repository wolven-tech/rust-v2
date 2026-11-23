//! The runway agent's output is applied through the same `Surfaces` the app
//! renders from, so a message the catalog would refuse fails here rather than
//! in a browser.

use api::presentation::agents::runway::{self, CALCULATE, SURFACE_ID};
use rv2_a2ui::catalog::{Lean, Verdict};
use rv2_a2ui::{ActionMessage, Node, Surfaces};
use serde_json::{Map, Value, json};

fn opened() -> Surfaces {
    let mut surfaces = Surfaces::new();
    surfaces
        .apply_all(&runway::open())
        .expect("the opening stream applies");
    surfaces
}

fn calculate(cash: Value, burn: Value) -> ActionMessage {
    let mut context = Map::new();
    context.insert("cash".into(), cash);
    context.insert("burn".into(), burn);
    ActionMessage {
        name: CALCULATE.into(),
        surface_id: SURFACE_ID.into(),
        source_component_id: "calculate".into(),
        timestamp: "2026-11-23T10:00:00.000Z".into(),
        context,
    }
}

/// Apply the answer and return the verdict callout and the panel's text.
fn answer(cash: Value, burn: Value) -> (Verdict, String, Option<(String, Lean)>) {
    let mut surfaces = opened();
    let messages = runway::act(&calculate(cash, burn)).expect("a calculate action is answered");
    surfaces.apply_all(&messages).expect("the answer applies");
    let surface = surfaces.get(SURFACE_ID).unwrap();

    let Some(Node::Callout { verdict, text, .. }) = surface.node("verdict") else {
        panic!("the answer has a verdict callout");
    };
    let shown = surface.node("result").unwrap().children();
    let panel = shown.iter().any(|id| id == "figure").then(|| {
        let Some(Node::ResultPanel { value, lean, .. }) = surface.node("figure") else {
            panic!("figure is a ResultPanel");
        };
        (value.resolve(surface.data()), *lean)
    });
    (*verdict, text.resolve(surface.data()), panel)
}

#[test]
fn the_opening_stream_renders_a_form_with_bound_inputs_and_an_action() {
    let surfaces = opened();
    let surface = surfaces.get(SURFACE_ID).unwrap();
    assert!(surface.has_root());
    assert!(
        matches!(surface.node("cash"), Some(Node::UnitField { value, .. }) if value.path() == "/inputs/cash")
    );
    let Some(Node::Button {
        action: Some(action),
        ..
    }) = surface.node("calculate")
    else {
        panic!("calculate is a Button with an action");
    };
    assert_eq!(action.name, CALCULATE);
}

#[test]
fn eight_months_of_runway_is_a_caution() {
    let (verdict, _, panel) = answer(json!("£120,000"), json!("15000"));
    assert_eq!(verdict, Verdict::Caution);
    assert_eq!(panel, Some(("8.0 months".into(), Lean::Close)));
}

#[test]
fn under_six_months_is_a_stop_and_a_year_is_clear() {
    assert_eq!(answer(json!("50000"), json!("10000")).0, Verdict::Stop);
    assert_eq!(answer(json!(240_000), json!(20_000)).0, Verdict::Clear);
}

#[test]
fn no_net_burn_has_no_limit() {
    let (verdict, _, panel) = answer(json!("1000"), json!("0"));
    assert_eq!(verdict, Verdict::Clear);
    assert_eq!(panel, Some(("No limit".into(), Lean::Good)));
}

#[test]
fn figures_are_written_as_whole_pounds() {
    let mut surfaces = opened();
    surfaces
        .apply_all(&runway::act(&calculate(json!("1234567.6"), json!("1000"))).unwrap())
        .unwrap();
    assert_eq!(
        surfaces.get(SURFACE_ID).unwrap().data()["result"]["cash"],
        json!("£1,234,568")
    );
}

#[test]
fn input_the_user_can_fix_is_answered_on_the_surface_and_hides_the_old_figure() {
    let mut surfaces = opened();
    surfaces
        .apply_all(&runway::act(&calculate(json!("120000"), json!("15000"))).unwrap())
        .unwrap();
    surfaces
        .apply_all(&runway::act(&calculate(json!(""), json!("15000"))).unwrap())
        .unwrap();
    let surface = surfaces.get(SURFACE_ID).unwrap();
    assert_eq!(surface.node("result").unwrap().children(), ["verdict"]);
    let Some(Node::Callout { verdict, text, .. }) = surface.node("verdict") else {
        panic!("verdict is a Callout");
    };
    assert_eq!(*verdict, Verdict::Stop);
    assert_eq!(text.resolve(surface.data()), "Cash in the bank is empty.");

    let (_, negative, _) = answer(json!("10"), json!("-5"));
    assert_eq!(negative, "Net burn per month cannot be negative.");
    let (_, words, _) = answer(json!("lots"), json!("5"));
    assert_eq!(words, "Cash in the bank is not a number.");
}

#[test]
fn an_action_the_surface_never_offers_is_refused() {
    let mut action = calculate(json!("1"), json!("1"));
    action.name = "delete_everything".into();
    assert_eq!(runway::act(&action).unwrap_err().field, "name");

    let mut elsewhere = calculate(json!("1"), json!("1"));
    elsewhere.surface_id = "other".into();
    assert_eq!(runway::act(&elsewhere).unwrap_err().field, "surfaceId");
}
