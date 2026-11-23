//! Applying messages: atomic, bounded, and safe to render (A2UI5).

use rv2_a2ui::{
    CATALOG_ID, ComponentDef, MAX_COMPONENTS, MAX_DEPTH, Node, ServerMessage, SurfaceError,
    Surfaces,
};
use serde_json::json;

fn opened() -> Surfaces {
    let mut surfaces = Surfaces::new();
    surfaces
        .apply(&ServerMessage::create_surface("s", CATALOG_ID))
        .unwrap();
    surfaces
}

fn text(id: &str) -> ComponentDef {
    ComponentDef::new(id, "Text").prop("text", id)
}

#[test]
fn a_surface_in_another_catalog_is_refused() {
    let mut surfaces = Surfaces::new();
    let error = surfaces
        .apply(&ServerMessage::create_surface(
            "s",
            "https://a2ui.org/specification/v0_9_1/catalogs/basic/catalog.json",
        ))
        .unwrap_err();
    assert!(matches!(error, SurfaceError::Catalog(_)));
    assert!(surfaces.get("s").is_none());
}

#[test]
fn creating_a_surface_twice_is_refused() {
    let mut surfaces = opened();
    let error = surfaces
        .apply(&ServerMessage::create_surface("s", CATALOG_ID))
        .unwrap_err();
    assert_eq!(error, SurfaceError::Exists("s".into()));
}

#[test]
fn nothing_renders_until_root_arrives_and_components_may_come_in_any_order() {
    let mut surfaces = opened();
    surfaces
        .apply(&ServerMessage::update_components("s", vec![text("a")]))
        .unwrap();
    assert!(!surfaces.get("s").unwrap().has_root());

    let root = ComponentDef::new("root", "Column").children(["a", "later"]);
    surfaces
        .apply(&ServerMessage::update_components("s", vec![root]))
        .unwrap();
    let surface = surfaces.get("s").unwrap();
    assert!(surface.has_root());
    assert!(
        surface.node("later").is_none(),
        "a pending child is allowed"
    );
}

#[test]
fn one_bad_component_leaves_the_surface_as_it_was() {
    let mut surfaces = opened();
    surfaces
        .apply(&ServerMessage::update_components(
            "s",
            vec![
                ComponentDef::new("root", "Column").children(["a"]),
                text("a"),
            ],
        ))
        .unwrap();
    let before = surfaces.clone();

    let error = surfaces
        .apply(&ServerMessage::update_components(
            "s",
            vec![
                ComponentDef::new("a", "Text").prop("text", "changed"),
                ComponentDef::new("b", "Text").prop("text", 1),
            ],
        ))
        .unwrap_err();
    match &error {
        SurfaceError::Component { index, error } => {
            assert_eq!(*index, 1);
            assert_eq!(error.prop.as_deref(), Some("text"));
        }
        other => panic!("expected Component, got {other:?}"),
    }
    assert_eq!(surfaces, before);

    let report = error.to_message("s");
    assert_eq!(report.code, "VALIDATION_FAILED");
    assert_eq!(report.path, "/updateComponents/components/1/text");
}

#[test]
fn a_component_with_two_parents_is_refused() {
    let mut surfaces = opened();
    let error = surfaces
        .apply(&ServerMessage::update_components(
            "s",
            vec![
                ComponentDef::new("root", "Column").children(["left", "right"]),
                ComponentDef::new("left", "Column").children(["shared"]),
                ComponentDef::new("right", "Column").children(["shared"]),
                text("shared"),
            ],
        ))
        .unwrap_err();
    assert_eq!(error, SurfaceError::SharedChild("shared".into()));
}

#[test]
fn a_component_that_contains_itself_or_root_is_refused() {
    let mut surfaces = opened();
    let own = surfaces
        .apply(&ServerMessage::update_components(
            "s",
            vec![ComponentDef::new("loop", "Column").children(["loop"])],
        ))
        .unwrap_err();
    assert_eq!(own, SurfaceError::Cycle("loop".into()));

    let back = surfaces
        .apply(&ServerMessage::update_components(
            "s",
            vec![
                ComponentDef::new("root", "Column").children(["inner"]),
                ComponentDef::new("inner", "Column").children(["root"]),
            ],
        ))
        .unwrap_err();
    assert_eq!(back, SurfaceError::Cycle("root".into()));
}

#[test]
fn nesting_past_the_limit_is_refused_and_at_the_limit_is_not() {
    let chain = |depth: usize| {
        let mut defs = Vec::new();
        let mut parent = "root".to_owned();
        for level in 1..=depth {
            let child = format!("c{level}");
            defs.push(ComponentDef::new(parent.clone(), "Column").children([child.clone()]));
            parent = child;
        }
        defs.push(text(&parent));
        defs
    };

    let mut surfaces = opened();
    surfaces
        .apply(&ServerMessage::update_components("s", chain(MAX_DEPTH)))
        .unwrap();

    let mut surfaces = opened();
    let error = surfaces
        .apply(&ServerMessage::update_components("s", chain(MAX_DEPTH + 1)))
        .unwrap_err();
    assert!(matches!(error, SurfaceError::TooDeep(_)));
}

#[test]
fn a_surface_over_the_component_limit_is_refused() {
    let mut surfaces = opened();
    let defs = (0..=MAX_COMPONENTS)
        .map(|n| text(&format!("t{n}")))
        .collect();
    let error = surfaces
        .apply(&ServerMessage::update_components("s", defs))
        .unwrap_err();
    assert_eq!(error, SurfaceError::TooMany(MAX_COMPONENTS + 1));
}

#[test]
fn a_repeated_id_in_one_update_is_refused() {
    let mut surfaces = opened();
    let error = surfaces
        .apply(&ServerMessage::update_components(
            "s",
            vec![text("a"), text("a")],
        ))
        .unwrap_err();
    assert_eq!(error, SurfaceError::DuplicateId("a".into()));
}

#[test]
fn a_later_update_replaces_a_component_by_id() {
    let mut surfaces = opened();
    surfaces
        .apply(&ServerMessage::update_components("s", vec![text("a")]))
        .unwrap();
    surfaces
        .apply(&ServerMessage::update_components(
            "s",
            vec![ComponentDef::new("a", "Stamp").prop("text", "ready")],
        ))
        .unwrap();
    assert!(matches!(
        surfaces.get("s").unwrap().node("a"),
        Some(Node::Stamp { .. })
    ));
}

#[test]
fn data_updates_upsert_remove_and_drive_bound_props() {
    let mut surfaces = opened();
    surfaces
        .apply(&ServerMessage::update_data_model(
            "s",
            "/result/runway",
            json!("8.0 months"),
        ))
        .unwrap();
    surfaces
        .apply(&ServerMessage::update_data_model("s", "/scratch", json!(1)))
        .unwrap();
    let removal: ServerMessage = serde_json::from_str(
        r#"{"version":"v0.9.1","updateDataModel":{"surfaceId":"s","path":"/scratch"}}"#,
    )
    .unwrap();
    surfaces.apply(&removal).unwrap();

    let surface = surfaces.get("s").unwrap();
    assert_eq!(surface.data(), &json!({"result": {"runway": "8.0 months"}}));

    let def = ComponentDef::new("panel", "ResultPanel")
        .prop("label", "Runway")
        .bind("value", "/result/runway");
    let Node::ResultPanel { value, .. } = Node::parse(&def).unwrap() else {
        panic!("expected ResultPanel");
    };
    assert_eq!(value.resolve(surface.data()), "8.0 months");
}

#[test]
fn an_action_resolves_its_bindings_from_what_the_user_typed() {
    let mut surfaces = opened();
    let button = ComponentDef::new("go", "Button").prop("label", "Go").prop(
        "action",
        json!({"event": {"name": "calculate", "context": {"cash": {"path": "/in/cash"}, "unset": {"path": "/in/none"}, "unit": "GBP"}}}),
    );
    surfaces
        .apply(&ServerMessage::update_components("s", vec![button]))
        .unwrap();
    surfaces
        .get_mut("s")
        .unwrap()
        .set_value("/in/cash", json!("120000"))
        .unwrap();

    let surface = surfaces.get("s").unwrap();
    let Some(Node::Button {
        action: Some(action),
        ..
    }) = surface.node("go")
    else {
        panic!("expected a Button with an action");
    };
    let message = action
        .request("s", "go", surface.data())
        .stamp("2026-11-23T10:00:00.000Z");
    assert_eq!(message.name, "calculate");
    assert_eq!(message.source_component_id, "go");
    assert_eq!(message.context["cash"], json!("120000"));
    assert_eq!(message.context["unset"], json!(null));
    assert_eq!(message.context["unit"], json!("GBP"));
}

#[test]
fn deleting_a_surface_forgets_it_and_updates_to_it_then_fail() {
    let mut surfaces = opened();
    surfaces.apply(&ServerMessage::delete_surface("s")).unwrap();
    let error = surfaces
        .apply(&ServerMessage::update_components("s", vec![text("a")]))
        .unwrap_err();
    assert_eq!(error, SurfaceError::Missing("s".into()));
}
