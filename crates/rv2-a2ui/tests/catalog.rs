//! The published catalog and the parser agree, and the parser refuses what
//! the catalog does not allow.

use rv2_a2ui::catalog::{COMPONENTS, PropKind};
use rv2_a2ui::{CATALOG_ID, ComponentDef, Node, catalog_document};
use serde_json::{Value, json};

const COMMITTED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/catalog/rv2-v1.json");

/// `RV2_BLESS_CATALOG=1 cargo test -p rv2-a2ui --test catalog` rewrites the
/// committed file. A diff to it in review is a change to a published contract,
/// and v1 only grows (A2UI4).
#[test]
fn the_committed_catalog_is_what_the_table_generates() {
    let generated = format!(
        "{}\n",
        serde_json::to_string_pretty(&catalog_document()).unwrap()
    );
    if std::env::var_os("RV2_BLESS_CATALOG").is_some() {
        std::fs::write(COMMITTED, &generated).unwrap();
    }
    let committed = std::fs::read_to_string(COMMITTED).unwrap_or_default();
    assert!(
        committed == generated,
        "catalog/rv2-v1.json is stale; run `RV2_BLESS_CATALOG=1 cargo test -p rv2-a2ui --test catalog` and review the diff"
    );
}

#[test]
fn the_document_names_its_catalog_twice_as_the_spec_asks() {
    let doc = catalog_document();
    assert_eq!(doc["$id"], json!(CATALOG_ID));
    assert_eq!(doc["catalogId"], json!(CATALOG_ID));
}

/// A value the table says each prop kind accepts.
fn sample(kind: PropKind) -> Value {
    match kind {
        PropKind::Text => json!({"path": "/text"}),
        PropKind::Number => json!(3.5),
        PropKind::Int { min, .. } => json!(min),
        PropKind::Share => json!(0.5),
        PropKind::Choice(values) => json!(values[values.len() - 1]),
        PropKind::Children => json!(["a", "b"]),
        PropKind::Bound => json!({"path": "/form/field"}),
        PropKind::Action => {
            json!({"event": {"name": "go", "context": {"field": {"path": "/form/field"}, "fixed": 1}}})
        }
        PropKind::Stats => {
            json!([{"label": "Cash", "value": {"path": "/cash"}, "note": "today", "lean": "good"}])
        }
    }
}

/// Every prop the table declares is one the parser reads, so the published
/// document never offers a prop the client would reject.
#[test]
fn every_component_parses_with_every_prop_it_declares() {
    for spec in COMPONENTS {
        let mut def = ComponentDef::new("x", spec.name);
        for prop in spec.props {
            def = def.prop(prop.name, sample(prop.kind));
        }
        Node::parse(&def).unwrap_or_else(|e| panic!("{}: {e}", spec.name));
    }
}

#[test]
fn every_component_parses_with_only_its_required_props() {
    for spec in COMPONENTS {
        let mut def = ComponentDef::new("x", spec.name);
        for prop in spec.props.iter().filter(|p| p.required) {
            def = def.prop(prop.name, sample(prop.kind));
        }
        Node::parse(&def).unwrap_or_else(|e| panic!("{}: {e}", spec.name));
    }
}

fn reason(def: ComponentDef) -> String {
    Node::parse(&def).unwrap_err().to_string()
}

#[test]
fn an_unknown_component_names_what_the_catalog_has() {
    let message = reason(ComponentDef::new("x", "Modal"));
    assert!(message.contains("unknown component \"Modal\""), "{message}");
    assert!(message.contains("Callout"), "{message}");
}

#[test]
fn a_class_or_any_unlisted_prop_is_refused() {
    let message = reason(
        ComponentDef::new("x", "Text")
            .prop("text", "hi")
            .prop("class", "bg-fault"),
    );
    assert!(message.contains("no prop \"class\""), "{message}");
}

#[test]
fn a_missing_required_prop_is_named() {
    let message = reason(ComponentDef::new("x", "Heading"));
    assert!(message.contains("prop \"text\": is required"), "{message}");
}

#[test]
fn a_number_where_text_belongs_is_refused_by_type() {
    let message = reason(ComponentDef::new("x", "Text").prop("text", 4));
    assert!(
        message.contains("expected a string or {\"path\"}, got a number"),
        "{message}"
    );
}

#[test]
fn an_unknown_operand_lists_the_allowed_ones() {
    let message = reason(
        ComponentDef::new("x", "Callout")
            .prop("text", "t")
            .prop("verdict", "danger"),
    );
    assert!(message.contains("note, clear, caution, stop"), "{message}");
}

#[test]
fn unsupported_value_forms_are_refused_by_name() {
    let relative = reason(ComponentDef::new("x", "Text").bind("text", "name"));
    assert!(relative.contains("relative path"), "{relative}");

    let call = reason(ComponentDef::new("x", "Text").prop(
        "text",
        json!({"call": "formatString", "args": {"value": "${/a}"}}),
    ));
    assert!(call.contains("function calls are not supported"), "{call}");

    let template = reason(
        ComponentDef::new("x", "Column")
            .prop("children", json!({"path": "/items", "componentId": "row"})),
    );
    assert!(template.contains("templates"), "{template}");

    let local = reason(ComponentDef::new("x", "Button").prop("label", "Open").prop(
        "action",
        json!({"functionCall": {"call": "openUrl", "args": {"url": "https://x"}}}),
    ));
    assert!(
        local.contains("function calls are not supported"),
        "{local}"
    );
}

#[test]
fn an_input_must_bind_its_value() {
    let message = reason(
        ComponentDef::new("x", "TextField")
            .prop("label", "Email")
            .prop("value", "a@b.c"),
    );
    assert!(message.contains("must be {\"path\""), "{message}");
}

#[test]
fn a_heading_level_outside_one_to_four_is_refused() {
    let message = reason(
        ComponentDef::new("x", "Heading")
            .prop("text", "t")
            .prop("level", 7),
    );
    assert!(message.contains("from 1 to 4"), "{message}");
}
