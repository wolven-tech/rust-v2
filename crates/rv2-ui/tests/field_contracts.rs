#![allow(unused_braces)]

use dioxus::prelude::*;
use rv2_ui::{Button, FieldShell, TextArea, TextField, field_description};

fn render(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn attributes(html: &str, name: &str, value: &str) -> usize {
    html.matches(&format!("{name}=\"{value}\"")).count()
        + html.matches(&format!("{name}={value}")).count()
}

fn control<'a>(html: &'a str, tag: &str, id: &str) -> &'a str {
    html.split(&format!("<{tag}"))
        .skip(1)
        .map(|part| part.split('>').next().unwrap())
        .find(|part| part.contains(&format!("id=\"{id}\"")))
        .unwrap_or_else(|| panic!("Missing {tag} with ID {id}"))
}

fn natively_disabled(tag: &str) -> bool {
    tag.split_ascii_whitespace()
        .any(|attribute| attribute == "disabled" || attribute.starts_with("disabled="))
}

#[component]
fn Fields() -> Element {
    rsx! {
        TextField { id: "price", label: "Price", value: "invalid", help: "Enter pounds", error: "Enter a number", oninput: |_| {} }
        TextArea { id: "notes", label: "Notes", value: "", help: "Public evidence only", error: "Source needed", disabled: true, oninput: |_| {} }
        TextField { label: "First generated field", value: "", oninput: |_| {} }
        TextField { label: "Second generated field", value: "", oninput: |_| {} }
    }
}

#[component]
fn Busy() -> Element {
    rsx! {
        Button { busy: true, "Saving comparison" }
        Button { disabled: true, "Unavailable action" }
        Button { "Available action" }
    }
}

#[component]
fn Plain() -> Element {
    rsx! { TextField { id: "plain", label: "Plain", value: "", oninput: |_| {} } }
}

#[component]
fn Custom() -> Element {
    rsx! { FieldShell { id: "custom", label: "Custom amount", help: "Blank means unknown",
        input { id: "custom", "aria-describedby": "custom-help" }
    } }
}

#[component]
fn BlankMessages() -> Element {
    rsx! {
        TextField { id: "blank-field", label: "Blank field", value: "", help: " ", error: "\n", disabled: true, oninput: |_| {} }
        TextArea { id: "blank-area", label: "Blank area", value: "", help: "\t", error: "", oninput: |_| {} }
    }
}

#[test]
fn help_and_error_references_only_include_nonempty_rendered_messages() {
    assert_eq!(field_description("price", None, None), None);
    assert_eq!(field_description("price", Some(" "), Some("")), None);
    assert_eq!(
        field_description("price", Some("Pounds"), None).as_deref(),
        Some("price-help")
    );
    assert_eq!(
        field_description("price", None, Some("Invalid")).as_deref(),
        Some("price-error")
    );
    assert_eq!(
        field_description("price", Some("Pounds"), Some("Invalid")).as_deref(),
        Some("price-help price-error")
    );
}

#[test]
fn rendered_fields_bind_labels_help_and_errors_to_the_input() {
    let html = render(Fields);
    for id in ["price", "notes"] {
        assert!(html.contains(&format!("for=\"{id}\"")));
        assert!(html.contains(&format!("id=\"{id}-help\"")));
        assert!(html.contains(&format!("id=\"{id}-error\"")));
        assert!(html.contains(&format!("aria-describedby=\"{id}-help {id}-error\"")));
    }
    assert_eq!(attributes(&html, "aria-invalid", "true"), 2);
    assert_eq!(html.matches("role=\"alert\"").count(), 2);
    assert!(natively_disabled(control(&html, "textarea", "notes")));
    let generated: Vec<_> = html
        .split("id=\"rv2-field-")
        .skip(1)
        .map(|s| s.split('"').next().unwrap())
        .collect();
    assert_eq!(generated.len(), 2);
    assert_ne!(generated[0], generated[1]);
}

#[test]
fn valid_fields_do_not_reference_missing_error_nodes() {
    let html = render(Plain);
    assert!(!html.contains("aria-describedby"));
    assert!(!html.contains("plain-error"));
    assert_eq!(attributes(&html, "aria-invalid", "false"), 1);
}

#[test]
fn busy_buttons_are_natively_disabled_and_expose_busy_state() {
    let html = render(Busy);
    let disabled_count = html
        .split("<button")
        .skip(1)
        .map(|part| part.split('>').next().unwrap())
        .filter(|tag| natively_disabled(tag))
        .count();
    assert_eq!(disabled_count, 2);
    assert_eq!(attributes(&html, "aria-busy", "true"), 1);
    assert_eq!(attributes(&html, "aria-busy", "false"), 2);
    assert!(html.contains("Saving comparison"));
}

#[test]
fn whitespace_messages_do_not_create_invalid_states_or_dangling_references() {
    let html = render(BlankMessages);
    assert!(!html.contains("aria-describedby"));
    assert!(!html.contains("role=\"alert\""));
    assert!(!html.contains("-help\""));
    assert!(!html.contains("-error\""));
    assert_eq!(attributes(&html, "aria-invalid", "false"), 2);
    assert!(natively_disabled(control(&html, "input", "blank-field")));
    assert!(!natively_disabled(control(&html, "textarea", "blank-area")));
}

#[test]
fn custom_typed_fields_use_the_same_label_and_help_contract() {
    let html = render(Custom);
    assert!(html.contains("for=\"custom\""));
    assert!(html.contains("id=\"custom-help\""));
    assert!(html.contains("aria-describedby=\"custom-help\""));
}
