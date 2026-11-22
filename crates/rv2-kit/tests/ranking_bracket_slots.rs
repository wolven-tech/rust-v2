//! Stylesheet arithmetic for the ranking bracket: the name and meta lines of one
//! slot must fit inside the slot, or their text runs over the pairing's border.

/// The body of one CSS rule, from its selector's opening brace to the closing brace.
fn css_rule<'a>(css: &'a str, selector: &str) -> &'a str {
    let start = css
        .find(&format!("\n{selector} {{"))
        .map(|at| at + 1)
        .unwrap_or_else(|| panic!("{selector} rule exists"));
    let body = &css[start..];
    &body[..body.find('}').expect("rule closes")]
}

/// The pixel value of one declaration inside a rule body.
fn px(rule: &str, property: &str) -> f64 {
    let start = rule
        .find(&format!("{property}: "))
        .unwrap_or_else(|| panic!("{property} declared"))
        + property.len()
        + 2;
    let value = &rule[start..];
    value[..value.find("px").expect("px unit")]
        .trim()
        .parse()
        .expect("numeric px value")
}

#[test]
fn name_and_meta_lines_fit_inside_one_slot_of_a_pairing() {
    let css = include_str!("../assets/ranking-bracket.css");
    let pair_height = px(css_rule(css, ".kit-rb__pair"), "height");
    let name_line = px(css_rule(css, ".kit-rb__name"), "line-height");
    let meta_line = px(css_rule(css, ".kit-rb__slot-meta"), "line-height");
    let pair_inner = pair_height - 2.0;
    let slot_room = pair_inner / 2.0 - 1.0;
    assert!(
        name_line + meta_line <= slot_room,
        "name {name_line}px + meta {meta_line}px must fit the {slot_room}px slot"
    );
}
