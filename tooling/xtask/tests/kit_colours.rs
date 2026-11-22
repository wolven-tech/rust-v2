#[allow(dead_code)]
#[path = "../src/kit_colours.rs"]
mod kit_colours;

use kit_colours::{judge, offending_lines, parse_ratchet};

fn lines(css: &str) -> Vec<usize> {
    offending_lines(css)
        .into_iter()
        .map(|(line, _)| line)
        .collect()
}

#[test]
fn hex_rgb_and_hsl_values_are_literal_colours() {
    let css = ".a { color: #2563eb; }\n.b { background: rgb(15 23 42 / 0.05); }\n.c { color: hsl(200 50% 50%); }\n.d { color: #fff; }\n";
    assert_eq!(lines(css), [1, 2, 3, 4]);
}

#[test]
fn tokens_and_derived_colours_pass() {
    let css = ".a { color: var(--kit-ink); }\n.b { background: color-mix(in srgb, var(--kit-id-1) 14%, var(--kit-surface)); }\n.c { color: currentColor; border-color: transparent; }\n";
    assert!(lines(css).is_empty());
}

#[test]
fn oklch_is_literal_unless_lightness_and_chroma_are_tokens() {
    let literal = ".a { background: oklch(95% 0.035 var(--kit-x-hue)); }\n";
    assert_eq!(lines(literal), [1]);

    let half = ".a { background: oklch(var(--kit-tint-bg-l) 0.035 var(--kit-x-hue)); }\n";
    assert_eq!(lines(half), [1]);

    let tokens =
        ".a { background: oklch(var(--kit-tint-bg-l) var(--kit-tint-bg-c) var(--kit-x-hue)); }\n";
    assert!(lines(tokens).is_empty());
}

#[test]
fn a_colour_inside_a_comment_is_not_a_value() {
    let css =
        "/* was #2563eb */\n/* spans\n   rgb(0 0 0) lines */\n.a { color: var(--kit-ink); }\n";
    assert!(lines(css).is_empty());
}

#[test]
fn names_that_contain_a_function_name_are_not_that_function() {
    let css = ".a { color: oklch(var(--l) var(--c) 250); background: oklab-ish; }\n";
    assert!(lines(css).is_empty());
}

#[test]
fn hex_like_words_that_are_not_colours_pass() {
    let css = ".a { grid-area: #deadbeefcafe; }\n.b::after { content: \"#1\"; }\n";
    assert!(lines(css).is_empty());
}

#[test]
fn the_opt_out_marker_excuses_its_line() {
    let css = ".a { color: #000; } /* hue-scan-ok: print stylesheet */\n";
    assert!(lines(css).is_empty());
}

#[test]
fn an_offending_file_off_the_ratchet_fails_and_a_listed_one_passes() {
    let ratchet = vec!["old.css".to_string()];
    assert!(judge(&["old.css".to_string()], &ratchet).is_empty());
    assert_eq!(
        judge(&["new.css".to_string(), "old.css".to_string()], &ratchet).len(),
        1
    );
}

#[test]
fn a_clean_file_still_on_the_ratchet_fails_so_the_list_only_shrinks() {
    let problems = judge(&[], &["fixed.css".to_string()]);
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("remove it"));
}

#[test]
fn the_ratchet_file_ignores_blank_lines_and_comments() {
    let text = "# header\n\na.css\nb.css  # note\n";
    assert_eq!(parse_ratchet(text), ["a.css", "b.css"]);
}
