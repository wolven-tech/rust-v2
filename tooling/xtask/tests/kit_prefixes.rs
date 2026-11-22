#[allow(dead_code)]
#[path = "../src/kit_prefixes.rs"]
mod kit_prefixes;

use kit_prefixes::{collisions, leading_blocks};

fn sheet(name: &str, css: &str) -> (String, String) {
    (name.to_string(), css.to_string())
}

#[test]
fn the_block_is_the_leftmost_kit_class_cut_at_its_element_or_modifier() {
    let css = ".kit-pp { a: b; }\n.kit-pp__grid { a: b; }\n.kit-pp--open .kit-other__x { a: b; }\n";
    assert_eq!(leading_blocks(css), ["kit-pp"]);
}

#[test]
fn rules_inside_media_and_container_queries_count() {
    let css = "@container (max-width: 420px) { .kit-rtr__tick { a: b; } }\n@media (x) { .kit-span { a: b; } }\n";
    assert_eq!(leading_blocks(css), ["kit-rtr", "kit-span"]);
}

#[test]
fn keyframe_steps_and_comments_are_not_selectors() {
    let css = "/* .kit-ghost { } */\n@keyframes kit-pulse { from { opacity: 0; } 50% { opacity: 1; } }\n.kit-real { a: b; }\n";
    assert_eq!(leading_blocks(css), ["kit-real"]);
}

#[test]
fn a_theme_or_state_prefix_before_the_block_does_not_change_the_block() {
    let css = "[data-theme=\"dark\"] .kit-acm__agent, .dark .kit-acm { a: b; }\n";
    assert_eq!(leading_blocks(css), ["kit-acm"]);
}

#[test]
fn two_stylesheets_leading_the_same_block_collide() {
    let found = collisions(&[
        sheet("agent-clash-map.css", ".kit-acm__agent { a: b; }"),
        sheet("agent-context-map.css", ".kit-acm__agent { c: d; }"),
    ]);
    assert_eq!(
        found.get("kit-acm").map(Vec::len),
        Some(2),
        "the clash-map defect must be caught"
    );
}

#[test]
fn a_variant_declaring_the_shared_block_is_not_a_collision() {
    let found = collisions(&[
        sheet("workspace-frame-inset.css", ".kit-frame { a: b; }"),
        sheet(
            "workspace-frame-flat.css",
            "/* kit-shares: kit-frame — the flat variant */\n.kit-frame[data-style=\"flat\"] { a: b; }",
        ),
    ]);
    assert!(found.is_empty());
}

#[test]
fn a_composite_adjusting_a_child_further_right_is_not_a_collision() {
    let found = collisions(&[
        sheet("metric-tile.css", ".kit-mt { a: b; }"),
        sheet("revenue-overview.css", ".kit-rev .kit-mt { a: b; }"),
    ]);
    assert!(found.is_empty());
}
