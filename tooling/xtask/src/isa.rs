//! `apps speak the kit` — UI7 in `docs/architecture/003-ui-semantic-isa.md`.
//!
//! An app states intent through `rv2-ui` and never writes a raw control,
//! heading or link with a hand-made class string. Before this check the kit
//! had the components and the apps routinely skipped them: the same outline
//! button typed out seven times, two inputs that had lost their focus ring, ten
//! raw anchors each styled where it stood. Each copy was reasonable on its own
//! day, and the sum was a kit that changed only half the page when it changed.
//!
//! A raw element is still allowed where it is the subject rather than the
//! chrome — a skip link, markup that is itself being demonstrated — and the
//! marker says so beside it, on the element's line or in the comment directly
//! above. A visible exception is a design note; an invisible one is drift.
//!
//! A scan over source text rather than a lint, for the reason the hue check
//! gives: it needs no app to compile, and it reaches every app at once.

use std::path::{Path, PathBuf};

/// The marker that admits one raw element, with its reason.
pub const OPT_OUT: &str = "isa-raw-ok";

/// RSX elements an app must not write itself. Each has an instruction:
/// `Button`/`LinkButton`/`TextLink`, `Input`/`TextField`/`TextArea`/`Select`,
/// `Heading`.
const ELEMENTS: &[&str] = &[
    "a", "button", "input", "select", "textarea", "h1", "h2", "h3", "h4",
];

/// The raw elements opened on one line of RSX.
///
/// An element opens where its name is the first thing on the line or follows
/// an opening brace, and is followed by whitespace and a brace of its own —
/// `button {`, `li { a {`. That shape is what RSX looks like and what Rust
/// expressions do not: `match a {` and `if a {` have a keyword before the name.
fn raw_elements(line: &str) -> Vec<&'static str> {
    let code = line.trim_start();
    if code.starts_with("//") {
        return Vec::new();
    }
    let mut found = Vec::new();
    for &name in ELEMENTS {
        let mut from = 0;
        while let Some(offset) = code[from..].find(name) {
            let start = from + offset;
            let end = start + name.len();
            from = end;

            let before = code[..start].trim_end();
            let opens_here = before.is_empty() || before.ends_with('{');
            let bounded = code[..start]
                .chars()
                .next_back()
                .is_none_or(|c| c.is_whitespace() || c == '{');
            let rest = &code[end..];
            let then_brace =
                rest.starts_with(char::is_whitespace) && rest.trim_start().starts_with('{');

            if opens_here && bounded && then_brace {
                found.push(name);
            }
        }
    }
    found
}

/// Every unmarked raw element in one file, as `path:line: text`.
fn scan(path: &str, source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut offenders = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if raw_elements(line).is_empty() || line.contains(OPT_OUT) {
            continue;
        }
        let excused = lines[..index]
            .iter()
            .rev()
            .take_while(|l| l.trim_start().starts_with("//"))
            .any(|l| l.contains(OPT_OUT));
        if !excused {
            offenders.push(format!("{path}:{}: {}", index + 1, line.trim()));
        }
    }
    offenders
}

fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            rust_files(&path, into)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
    Ok(())
}

/// Scan `apps/*/src`.
///
/// Sources only: `examples/` are previews of the kit's own markup, and `tests/`
/// render pages rather than define them.
pub fn check(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for app in std::fs::read_dir(root.join("apps"))? {
        let src = app?.path().join("src");
        if src.is_dir() {
            rust_files(&src, &mut files)?;
        }
    }
    files.sort();

    let mut offenders = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)?;
        let shown = file
            .strip_prefix(root)
            .unwrap_or(file)
            .display()
            .to_string();
        offenders.extend(scan(&shown, &source));
    }

    if !offenders.is_empty() {
        eprintln!("{}", offenders.join("\n"));
        return Err(format!(
            "an app wrote a raw control, heading or link instead of using rv2-ui\n         \
             the instructions are listed in docs/architecture/003-ui-semantic-isa.md\n         \
             if the element is genuinely the subject of the page, mark it `{OPT_OUT}: <why>`"
        )
        .into());
    }
    println!("clean ({} files)", files.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_planted_raw_button_fails_the_scan() {
        let source = "rsx! {\n    button { class: \"rounded-md\", \"Go\" }\n}\n";
        assert_eq!(
            scan("x.rs", source),
            ["x.rs:2: button { class: \"rounded-md\", \"Go\" }"]
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
}
