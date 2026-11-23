//! The envelopes read what the v0.9.1 specification writes. Every JSON string
//! here is copied from a2ui.org, not composed to match the parser.

use rv2_a2ui::wire::{ClientBody, ServerBody};
use rv2_a2ui::{ClientMessage, ServerMessage, VERSION, WireError, parse_jsonl, to_jsonl};
use serde_json::json;

#[test]
fn the_specifications_create_surface_example_parses() {
    let message: ServerMessage = serde_json::from_str(
        r#"{"version": "v0.9.1", "createSurface": {"surfaceId": "contact_form_1", "catalogId": "https://a2ui.org/specification/v0_9_1/catalogs/basic/catalog.json"}}"#,
    )
    .unwrap();
    let ServerBody::CreateSurface(create) = message.body else {
        panic!("expected createSurface");
    };
    assert_eq!(create.surface_id, "contact_form_1");
    assert!(!create.send_data_model);
}

#[test]
fn the_specifications_update_components_example_keeps_props_flat() {
    let message: ServerMessage = serde_json::from_str(
        r#"{"version": "v0.9.1", "updateComponents": {"surfaceId": "user_profile_card", "components": [{"id": "root", "component": "Column", "children": ["user_name", "user_title"]}, {"id": "user_name", "component": "Text", "text": "John Doe"}]}}"#,
    )
    .unwrap();
    let ServerBody::UpdateComponents(update) = message.body else {
        panic!("expected updateComponents");
    };
    assert_eq!(
        update.components[0].props["children"],
        json!(["user_name", "user_title"])
    );
    assert_eq!(update.components[1].props["text"], json!("John Doe"));
}

#[test]
fn an_omitted_value_removes_and_a_null_value_sets_null() {
    let removal: ServerMessage = serde_json::from_str(
        r#"{"version": "v0.9.1", "updateDataModel": {"surfaceId": "surface_123", "path": "/user/tempData"}}"#,
    )
    .unwrap();
    let null: ServerMessage = serde_json::from_str(
        r#"{"version": "v0.9.1", "updateDataModel": {"surfaceId": "surface_123", "path": "/user/tempData", "value": null}}"#,
    )
    .unwrap();
    let (ServerBody::UpdateDataModel(removal), ServerBody::UpdateDataModel(null)) =
        (removal.body, null.body)
    else {
        panic!("expected updateDataModel twice");
    };
    assert_eq!(removal.value, None);
    assert_eq!(null.value, Some(serde_json::Value::Null));
}

#[test]
fn the_messages_pages_action_example_parses() {
    let message: ClientMessage = serde_json::from_str(
        r#"{"version": "v0.9", "action": {"name": "select_flight", "surfaceId": "flight_search", "sourceComponentId": "btn_select_lh458", "timestamp": "2026-11-02T10:15:30.000Z", "context": {"flightId": "LH458", "price": 680}}}"#,
    )
    .unwrap();
    message.check_version().unwrap();
    let ClientBody::Action(action) = message.body else {
        panic!("expected action");
    };
    assert_eq!(action.source_component_id, "btn_select_lh458");
    assert_eq!(action.context["price"], json!(680));
}

#[test]
fn messages_written_here_round_trip_with_the_version_this_crate_writes() {
    let messages = vec![
        ServerMessage::create_surface("s", "rv2:a2ui/catalog/v1"),
        ServerMessage::update_data_model("s", "/a", json!(1)),
        ServerMessage::delete_surface("s"),
    ];
    let text = to_jsonl(&messages);
    assert_eq!(text.lines().count(), 3);
    assert!(
        text.lines()
            .all(|line| line.contains(r#""version":"v0.9.1""#))
    );
    assert_eq!(parse_jsonl(&text).unwrap(), messages);
    assert_eq!(VERSION, "v0.9.1");
}

#[test]
fn a_v1_message_is_refused_rather_than_bound_wrongly() {
    let error =
        parse_jsonl(r#"{"version": "v1.0", "deleteSurface": {"surfaceId": "s"}}"#).unwrap_err();
    assert_eq!(error, WireError::Version("v1.0".into()));
}

#[test]
fn a_malformed_line_is_named_by_its_line_number() {
    let text = "{\"version\":\"v0.9.1\",\"deleteSurface\":{\"surfaceId\":\"s\"}}\n\nnot json\n";
    match parse_jsonl(text).unwrap_err() {
        WireError::Malformed { line, .. } => assert_eq!(line, 3),
        other => panic!("expected Malformed, got {other:?}"),
    }
}
