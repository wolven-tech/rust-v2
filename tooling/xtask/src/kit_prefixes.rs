//! Each rv2-kit stylesheet owns its block class prefix.
//!
//! The kit loads every component's stylesheet on one page, so two components
//! that both write rules for `.kit-pp…` style each other. That shipped twice:
//! one stylesheet's absolutely positioned card covered another component's
//! labels, and a progress panel's grid rules emptied a date picker.
//!
//! The block of a rule is the leftmost `kit-` class of each selector, cut at the
//! first `__` or `--`. A block may lead rules in one stylesheet only; another
//! stylesheet may still mention it further right (`.kit-screen .kit-card`), which
//! is how a composite deliberately adjusts a child.
//!
//! Standalone on purpose (std only): `tests/kit_prefixes.rs` compiles this file
//! directly, because xtask is a binary with no library.

use std::collections::BTreeMap;
use std::path::Path;

pub const ASSETS: &str = "crates/rv2-kit/assets";

/// The marker a variant stylesheet carries to extend a block another stylesheet
/// owns, e.g. `/* kit-shares: kit-frame — the flat variant of the frame */`.
pub const SHARES: &str = "kit-shares:";

/// Blocks that lead at least one rule in `css`, minus those it declares shared.
pub fn leading_blocks(css: &str) -> Vec<String> {
    let shared = declared_shares(css);
    let mut blocks = Vec::new();
    for selector_list in selectors(&strip_comments(css)) {
        for selector in selector_list.split(',') {
            if let Some(block) = first_kit_block(selector)
                && !blocks.contains(&block)
                && !shared.contains(&block)
            {
                blocks.push(block);
            }
        }
    }
    blocks
}

fn declared_shares(css: &str) -> Vec<String> {
    css.match_indices(SHARES)
        .flat_map(|(at, _)| {
            css[at + SHARES.len()..]
                .split_whitespace()
                .take_while(|word| word.starts_with("kit-"))
                .map(|word| word.trim_end_matches(',').to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// The text before each `{` that opens a style rule, skipping at-rule preludes
/// (`@media`, `@container`, `@keyframes` and its `from`/`50%` steps).
fn selectors(css: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut prelude = String::new();
    let mut keyframes_depth: Option<usize> = None;
    let mut depth = 0usize;
    for ch in css.chars() {
        match ch {
            '{' => {
                let text = prelude.trim().to_string();
                if text.starts_with("@keyframes") {
                    keyframes_depth = Some(depth);
                } else if !text.starts_with('@') && keyframes_depth.is_none() {
                    found.push(text);
                }
                depth += 1;
                prelude.clear();
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if keyframes_depth == Some(depth) {
                    keyframes_depth = None;
                }
                prelude.clear();
            }
            ';' => prelude.clear(),
            c => prelude.push(c),
        }
    }
    found
}

fn first_kit_block(selector: &str) -> Option<String> {
    let at = selector.find(".kit-")?;
    let name: String = selector[at + 1..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    let cut = [name.find("__"), name.find("--")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(name.len());
    Some(name[..cut].to_string())
}

/// Blocks led by more than one stylesheet, with the files that lead each.
pub fn collisions(sheets: &[(String, String)]) -> BTreeMap<String, Vec<String>> {
    let mut owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, css) in sheets {
        for block in leading_blocks(css) {
            owners.entry(block).or_default().push(name.clone());
        }
    }
    owners.retain(|_, files| files.len() > 1);
    owners
}

pub fn check(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut sheets = Vec::new();
    for entry in std::fs::read_dir(root.join(ASSETS))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".css") && name != "kit.css" {
            sheets.push((name, std::fs::read_to_string(entry.path())?));
        }
    }
    sheets.sort();
    let clashes = collisions(&sheets);
    if clashes.is_empty() {
        println!(
            "clean ({} stylesheets, every block owned once)",
            sheets.len()
        );
        return Ok(());
    }
    let lines: Vec<String> = clashes
        .iter()
        .map(|(block, files)| format!(".{block} leads rules in {}", files.join(", ")))
        .collect();
    Err(format!(
        "rv2-kit class blocks shared between stylesheets\n         {}\n         give each component its own block prefix",
        lines.join("\n         ")
    )
    .into())
}
