use rv2_kit::isa::{raw_elements, scan};

#[test]
fn an_element_opening_a_line_is_found() {
    assert_eq!(raw_elements("        button {"), ["button"]);
    assert_eq!(raw_elements("    h2 { class: \"x\", \"Title\" }"), ["h2"]);
}

#[test]
fn an_element_nested_on_the_same_line_is_found() {
    assert_eq!(raw_elements("li { a { href: \"/\", \"Home\" } }"), ["a"]);
}

#[test]
fn rust_expressions_naming_the_same_word_are_not_elements() {
    assert!(raw_elements("        match a {").is_empty());
    assert!(raw_elements("        if a {").is_empty());
    assert!(raw_elements("    let a = 1;").is_empty());
    assert!(raw_elements("    data { x }").is_empty());
}

#[test]
fn kit_components_and_other_elements_pass() {
    assert!(raw_elements("        Button { variant: Variant::Outline,").is_empty());
    assert!(raw_elements("        TextLink { href: url, \"Search\" }").is_empty());
    assert!(raw_elements("        span { class: \"x\" }").is_empty());
    assert!(raw_elements("        label { class: \"x\",").is_empty());
}

#[test]
fn a_commented_out_element_is_not_an_element() {
    assert!(raw_elements("    // button { was here }").is_empty());
}

#[test]
fn a_planted_raw_button_fails_the_scan_and_names_its_replacement() {
    let source = "rsx! {\n    button { class: \"rounded-md\", \"Go\" }\n}\n";
    let findings = scan("x.rs", source);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 2);
    let shown = findings[0].to_string();
    assert!(shown.starts_with("x.rs:2: button {"), "{shown}");
    assert!(
        shown.contains("raw `button` → use Button or PressButton"),
        "{shown}"
    );
}

#[test]
fn the_marker_on_the_line_or_in_the_comment_above_excuses_it() {
    let inline = "a { class: \"sr-only\", // isa-raw-ok: skip link\n";
    assert!(scan("x.rs", inline).is_empty());

    let above = "// isa-raw-ok: a skip link must be first\n// and invisible\na {\n";
    assert!(scan("x.rs", above).is_empty());
}

#[test]
fn a_marker_separated_by_code_excuses_nothing() {
    let source = "// isa-raw-ok: for the span\nspan {}\nbutton {\n";
    assert_eq!(scan("x.rs", source).len(), 1);
}
