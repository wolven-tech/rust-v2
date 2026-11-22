//! rv2-kit stylesheets take every colour from a `--kit-*` token.
//!
//! `kit.css` is the one place a literal colour may appear: it holds the
//! fallbacks behind each token. A literal anywhere else is correct in at most
//! one theme, and an app overriding the registers cannot reach it.
//!
//! The scan landed before the stylesheets were moved onto tokens, so it carries
//! a ratchet: [`RATCHET`] lists the files that still offend. The list may only
//! shrink. A file that offends and is not listed fails; a listed file that has
//! become clean also fails, so the list cannot keep an entry nobody needs.
//!
//! Standalone on purpose (std only, no `crate::` paths): `tests/kit_colours.rs`
//! compiles this file directly, because xtask is a binary with no library.

use std::path::Path;

/// The marker a line may carry to opt out, shared with the class-name scan.
pub const OPT_OUT: &str = "hue-scan-ok";

/// Relative to the workspace root.
pub const ASSETS: &str = "crates/rv2-kit/assets";
pub const RATCHET: &str = "tooling/xtask/kit-colour-ratchet.txt";

/// Every line of `css` that names a colour literally, as (1-based line, text).
/// Comments are skipped, including ones spanning lines.
pub fn offending_lines(css: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    let mut in_comment = false;
    for (index, raw) in css.lines().enumerate() {
        let code = strip_comments(raw, &mut in_comment);
        if raw.contains(OPT_OUT) {
            continue;
        }
        if has_literal_colour(&code) {
            found.push((index + 1, raw.trim().to_string()));
        }
    }
    found
}

fn strip_comments(line: &str, in_comment: &mut bool) -> String {
    let mut out = String::new();
    let mut rest = line;
    loop {
        if *in_comment {
            match rest.find("*/") {
                Some(end) => {
                    rest = &rest[end + 2..];
                    *in_comment = false;
                }
                None => return out,
            }
        } else {
            match rest.find("/*") {
                Some(start) => {
                    out.push_str(&rest[..start]);
                    rest = &rest[start + 2..];
                    *in_comment = true;
                }
                None => {
                    out.push_str(rest);
                    return out;
                }
            }
        }
    }
}

fn has_literal_colour(code: &str) -> bool {
    let lower = code.to_ascii_lowercase();
    for function in ["rgb(", "rgba(", "hsl(", "hsla(", "hwb(", "lab(", "lch("] {
        if contains_function(&lower, function) {
            return true;
        }
    }
    has_hex(&lower) || has_literal_oklch(&lower)
}

/// `lch(` must not match inside `oklch(`, nor `lab(` inside `oklab(`.
fn contains_function(code: &str, function: &str) -> bool {
    code.match_indices(function).any(|(at, _)| {
        at == 0
            || (!code.as_bytes()[at - 1].is_ascii_alphanumeric() && code.as_bytes()[at - 1] != b'-')
    })
}

fn has_hex(code: &str) -> bool {
    let bytes = code.as_bytes();
    for (at, _) in code.match_indices('#') {
        let digits = bytes[at + 1..]
            .iter()
            .take_while(|b| b.is_ascii_hexdigit())
            .count();
        let after = bytes.get(at + 1 + digits).copied();
        let ends_word =
            after.is_none_or(|b| !(b.is_ascii_alphanumeric() || b == b'-' || b == b'_'));
        if matches!(digits, 3 | 4 | 6 | 8) && ends_word {
            return true;
        }
    }
    false
}

/// `oklch(L C H)` is a literal unless both lightness and chroma come from
/// `var()`; the hue alone may stay a component's own variable or number.
fn has_literal_oklch(code: &str) -> bool {
    for (at, _) in code.match_indices("oklch(") {
        let args = balanced_args(&code[at + "oklch(".len()..]);
        let parts = split_top_level(args);
        let lightness = parts.first().map(String::as_str).unwrap_or("");
        let chroma = parts.get(1).map(String::as_str).unwrap_or("");
        if !lightness.starts_with("var(") || !chroma.starts_with("var(") {
            return true;
        }
    }
    false
}

fn balanced_args(after_open: &str) -> &str {
    let mut depth = 1;
    for (index, ch) in after_open.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &after_open[..index];
                }
            }
            _ => {}
        }
    }
    after_open
}

fn split_top_level(args: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    for ch in args.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            c if c.is_whitespace() && depth == 0 => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

/// Compare the offending files against the ratchet list. Returns the problems,
/// one line each; empty means the gate passes.
pub fn judge(offending: &[String], ratchet: &[String]) -> Vec<String> {
    let mut problems = Vec::new();
    for file in offending {
        if !ratchet.contains(file) {
            problems.push(format!("{file}: literal colour; use a --kit-* token"));
        }
    }
    for file in ratchet {
        if !offending.contains(file) {
            problems.push(format!(
                "{file}: clean now, remove it from {RATCHET} so it stays clean"
            ));
        }
    }
    problems
}

/// Parse the ratchet file: one file name per line, `#` starts a comment.
pub fn parse_ratchet(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// The gate: scan every stylesheet but `kit.css`, then hold the result to the
/// ratchet. Prints offending lines for files not on the list.
pub fn check(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut offending = Vec::new();
    let mut detail = Vec::new();
    let mut names: Vec<_> = std::fs::read_dir(root.join(ASSETS))?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".css") && name != "kit.css")
        .collect();
    names.sort();
    for name in &names {
        let css = std::fs::read_to_string(root.join(ASSETS).join(name))?;
        let lines = offending_lines(&css);
        if !lines.is_empty() {
            offending.push(name.clone());
            for (line, text) in lines {
                detail.push((name.clone(), format!("{ASSETS}/{name}:{line}: {text}")));
            }
        }
    }

    let ratchet = parse_ratchet(&std::fs::read_to_string(root.join(RATCHET)).unwrap_or_default());
    let problems = judge(&offending, &ratchet);
    if problems.is_empty() {
        println!(
            "clean ({} stylesheets, {} still on the ratchet)",
            names.len(),
            ratchet.len()
        );
        return Ok(());
    }
    for (file, line) in &detail {
        if !ratchet.contains(file) {
            eprintln!("{line}");
        }
    }
    Err(format!(
        "rv2-kit stylesheet colours\n         {}\n         tokens: docs/design-system/rv2-kit-tokens.md",
        problems.join("\n         ")
    )
    .into())
}
