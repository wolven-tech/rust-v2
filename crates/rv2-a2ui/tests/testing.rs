use rv2_a2ui::testing::apply_as_client;
use rv2_a2ui::{CATALOG_ID, ComponentDef, ServerMessage, Surfaces};

fn create() -> ServerMessage {
    ServerMessage::create_surface("s", CATALOG_ID)
}

#[test]
fn a_stream_that_creates_a_rooted_surface_applies() {
    let mut surfaces = Surfaces::new();
    let messages = [
        create(),
        ServerMessage::update_components(
            "s",
            vec![ComponentDef::new("root", "Text").prop("text", "hi")],
        ),
    ];
    apply_as_client(&mut surfaces, &messages).unwrap();
    assert!(surfaces.get("s").unwrap().has_root());
}

#[test]
fn a_surface_left_without_a_root_is_refused() {
    let mut surfaces = Surfaces::new();
    let message = apply_as_client(&mut surfaces, &[create()]).unwrap_err();
    assert!(message.contains("no \"root\""), "{message}");
}

#[test]
fn a_component_the_catalog_refuses_names_its_surface() {
    let mut surfaces = Surfaces::new();
    let messages = [
        create(),
        ServerMessage::update_components("s", vec![ComponentDef::new("root", "Modal")]),
    ];
    let message = apply_as_client(&mut surfaces, &messages).unwrap_err();
    assert!(message.starts_with("surface \"s\""), "{message}");
    assert!(message.contains("Modal"), "{message}");
}

#[test]
fn a_deleted_surface_needs_no_root() {
    let mut surfaces = Surfaces::new();
    apply_as_client(
        &mut surfaces,
        &[create(), ServerMessage::delete_surface("s")],
    )
    .unwrap();
    assert!(surfaces.get("s").is_none());
}
