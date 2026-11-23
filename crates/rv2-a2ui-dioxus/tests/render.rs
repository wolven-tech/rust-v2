//! A surface renders through the kit's instructions, with bound values filled
//! from the data model.

use dioxus::prelude::*;
use rv2_a2ui::{CATALOG_ID, ComponentDef, ServerMessage, Surfaces};
use rv2_a2ui_dioxus::A2uiSurface;
use serde_json::json;

fn render(messages: Vec<ServerMessage>) -> String {
    let mut dom = VirtualDom::new_with_props(Host, HostProps { messages });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[derive(Props, Clone, PartialEq)]
struct HostProps {
    messages: Vec<ServerMessage>,
}

#[allow(non_snake_case)]
fn Host(props: HostProps) -> Element {
    let surfaces = use_signal(|| {
        let mut surfaces = Surfaces::new();
        surfaces
            .apply_all(&props.messages)
            .expect("fixture applies");
        surfaces
    });
    rsx! { A2uiSurface { surfaces, surface_id: "s", on_action: move |_| {} } }
}

fn surface(components: Vec<ComponentDef>, data: serde_json::Value) -> Vec<ServerMessage> {
    vec![
        ServerMessage::create_surface("s", CATALOG_ID),
        ServerMessage::update_data_model("s", "/", data),
        ServerMessage::update_components("s", components),
    ]
}

#[test]
fn a_surface_without_root_shows_a_loading_state() {
    let html = render(surface(
        vec![ComponentDef::new("a", "Text").prop("text", "orphan")],
        json!({}),
    ));
    assert!(html.contains(r#"aria-label="Loading""#), "{html}");
    assert!(!html.contains("orphan"), "{html}");
}

#[test]
fn bound_text_comes_from_the_data_model() {
    let html = render(surface(
        vec![
            ComponentDef::new("root", "Column").children(["h", "panel"]),
            ComponentDef::new("h", "Heading")
                .prop("text", "Runway")
                .prop("level", 1),
            ComponentDef::new("panel", "ResultPanel")
                .prop("label", "Runway")
                .bind("value", "/result/runway")
                .prop("lean", "close"),
        ],
        json!({"result": {"runway": "8.0 months"}}),
    ));
    assert!(html.contains("<h1"), "{html}");
    assert!(html.contains("8.0 months"), "{html}");
    assert!(
        html.contains("text-caution-ink"),
        "lean maps to the kit's Close: {html}"
    );
}

#[test]
fn an_input_shows_its_bound_value_under_its_label() {
    let html = render(surface(
        vec![
            ComponentDef::new("root", "Card").children(["cash"]),
            ComponentDef::new("cash", "UnitField")
                .prop("label", "Cash in the bank, £")
                .prop("prefix", "£")
                .bind("value", "/inputs/cash"),
        ],
        json!({"inputs": {"cash": "120000"}}),
    ));
    assert!(html.contains(r#"for="a2ui-s-cash""#), "{html}");
    assert!(html.contains(r#"id="a2ui-s-cash""#), "{html}");
    assert!(html.contains(r#"value="120000""#), "{html}");
}

#[test]
fn a_button_without_an_action_is_disabled() {
    let html = render(surface(
        vec![
            ComponentDef::new("root", "Column").children(["go", "idle"]),
            ComponentDef::new("go", "Button")
                .prop("label", "Go")
                .prop("action", json!({"event": {"name": "go"}})),
            ComponentDef::new("idle", "Button").prop("label", "Idle"),
        ],
        json!({}),
    ));
    let idle = html.find("Idle").expect("idle button renders");
    let go = html.find(">Go<").expect("go button renders");
    // The class list names `disabled:` variants, so read the attributes only.
    let attributes = |end: usize| {
        let tag = &html[html[..end].rfind("<button").unwrap()..end];
        let class_end = tag.find("\" ").expect("class attribute closes");
        tag[class_end..].to_owned()
    };
    let (idle_attrs, go_attrs) = (attributes(idle), attributes(go));
    assert!(idle_attrs.contains("disabled"), "{idle_attrs}");
    assert!(!go_attrs.contains("disabled"), "{go_attrs}");
}

#[test]
fn a_child_that_has_not_arrived_renders_as_pending() {
    let html = render(surface(
        vec![
            ComponentDef::new("root", "Column").children(["here", "later"]),
            ComponentDef::new("here", "Stamp")
                .prop("text", "ready")
                .prop("verdict", "clear"),
        ],
        json!({}),
    ));
    assert!(html.contains("ready"), "{html}");
    assert!(html.contains(r#"aria-label="Loading""#), "{html}");
}

#[test]
fn verdicts_and_stats_reach_the_kit() {
    let html = render(surface(
        vec![
            ComponentDef::new("root", "Column").children(["strip", "call"]),
            ComponentDef::new("strip", "StatStrip").prop(
                "stats",
                json!([{"label": "Cash", "value": {"path": "/cash"}, "lean": "good"}]),
            ),
            ComponentDef::new("call", "Callout")
                .prop("verdict", "stop")
                .prop("title", "Under six months")
                .prop("text", "Cut burn."),
        ],
        json!({"cash": "£120,000"}),
    ));
    assert!(html.contains("<dt"), "{html}");
    assert!(html.contains("£120,000"), "{html}");
    assert!(
        html.contains(r#"role="alert""#),
        "a stop callout is an alert: {html}"
    );
    assert!(html.contains("Under six months"), "{html}");
}
