use dioxus::prelude::*;
use rv2_ui::{
    CommandSelect,
    command_select::{command_matches, command_step},
};

#[test]
fn search_matches_all_words_without_changing_order_or_ids() {
    let options = [
        ("strand", "Strand Property Hunt"),
        ("family", "Family workbook"),
    ];
    let matching: Vec<_> = options
        .iter()
        .filter(|(_, label)| command_matches(label, "  HUNT strand  "))
        .collect();
    assert_eq!(matching, vec![&options[0]]);
    assert!(!command_matches("Strand Property Hunt", "Strand missing"));
    assert!(command_matches("Oxford", ""));
    assert!(command_matches("École family", "éCOLE"));
}

#[test]
fn arrows_wrap_and_handle_empty_or_shrinking_results() {
    assert_eq!(command_step(0, 0, false), None);
    assert_eq!(command_step(0, 0, true), None);
    assert_eq!(command_step(0, 3, true), Some(2));
    assert_eq!(command_step(2, 3, false), Some(0));
    assert_eq!(command_step(999, 1, false), Some(0));
    assert_eq!(command_step(999, 2, true), Some(0));
}

#[component]
fn Fixture() -> Element {
    rsx! {
        CommandSelect {
            id: "workspace", label: "Your workspace", value: "strand",
            name: "workspace_id", options: vec![("strand".into(), "Strand Property Hunt".into())],
            onchange: |_| {},
        }
        CommandSelect {
            id: "empty", label: "Unavailable", value: "", options: vec![],
            placeholder: "Choose workspace", disabled: true, onchange: |_| {},
        }
    }
}

#[test]
fn rendered_control_has_label_value_form_binding_and_disabled_state() {
    let mut dom = VirtualDom::new(Fixture);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("aria-labelledby=\"workspace-label workspace-value\""));
    assert!(html.contains("aria-haspopup=\"dialog\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("Strand Property Hunt"));
    assert!(html.contains("name=\"workspace_id\" value=\"strand\""));
    assert!(html.contains("Choose workspace"));
    assert!(html.contains("disabled"));
    assert!(!html.contains("<select"));
    assert!(!html.contains("role=\"listbox\""));
}
