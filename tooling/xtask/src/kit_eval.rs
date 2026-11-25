//! The kit runs without eval'd JavaScript.
//!
//! `document::eval` hands the browser a string of JavaScript, which a page may
//! only run with `script-src 'unsafe-eval'`. A desktop shell or any page with a
//! strict content security policy (`'self' 'wasm-unsafe-eval'`) refuses it, and
//! every kit component that slept, read the clock or measured itself through an
//! eval died with an `EvalError`. The shared helpers in `rv2-kit/src/internal.rs`
//! now call the DOM through web-sys.
//!
//! Many components still eval their own scripts. [`RATCHET`] holds each file's
//! count of `document::eval` calls, and a count may only fall: a file not on
//! the list may have none, a file may not exceed its count, and a file that
//! drops below its count must have its line lowered (or removed at zero) so
//! the lower count is what is held.
//!
//! Standalone on purpose (std only): `tests/kit_eval.rs` compiles this file
//! directly, because xtask is a binary with no library.

use std::collections::BTreeMap;
use std::path::Path;

pub const RATCHET: &str = "tooling/xtask/kit-eval-ratchet.txt";

/// The crates the apps render from; each is scanned under `src/`.
pub const CRATES: [&str; 3] = ["crates/rv2-ui", "crates/rv2-affect", "crates/rv2-kit"];

const CALL: &str = "document::eval";

/// `document::eval` calls in `code`, outside `//` comments.
pub fn count_calls(code: &str) -> usize {
    code.lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .map(|line| line.matches(CALL).count())
        .sum()
}

/// One `path count` pair per line; `#` starts a comment.
pub fn parse_ratchet(text: &str) -> BTreeMap<String, usize> {
    text.lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let (path, count) = line.rsplit_once(char::is_whitespace)?;
            Some((path.trim().to_string(), count.parse().ok()?))
        })
        .collect()
}

/// Every way the counts disagree with the ratchet.
pub fn judge(counts: &BTreeMap<String, usize>, ratchet: &BTreeMap<String, usize>) -> Vec<String> {
    let mut problems = Vec::new();
    for (file, &count) in counts {
        let allowed = ratchet.get(file).copied().unwrap_or(0);
        if count > allowed {
            problems.push(format!(
                "{file}: {count} document::eval call(s), {allowed} allowed; call the DOM through web-sys (see crates/rv2-kit/src/internal.rs)"
            ));
        }
    }
    for (file, &allowed) in ratchet {
        let count = counts.get(file).copied().unwrap_or(0);
        if count < allowed {
            problems.push(format!(
                "{file}: {count} document::eval call(s) now, {allowed} on {RATCHET}; lower its line (remove it at 0) so it stays there"
            ));
        }
    }
    problems
}

fn scan(dir: &Path, root: &Path, counts: &mut BTreeMap<String, usize>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            scan(&path, root, counts)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let count = count_calls(&std::fs::read_to_string(&path)?);
            if count > 0 {
                let rel = path.strip_prefix(root).unwrap_or(&path);
                counts.insert(rel.to_string_lossy().replace('\\', "/"), count);
            }
        }
    }
    Ok(())
}

/// `document::eval` calls per file in the scanned crates.
pub fn counts(root: &Path) -> std::io::Result<BTreeMap<String, usize>> {
    let mut counts = BTreeMap::new();
    for krate in CRATES {
        scan(&root.join(krate).join("src"), root, &mut counts)?;
    }
    Ok(counts)
}

/// Rewrite [`RATCHET`] from today's counts. For lowering lines after removing
/// evals; never to raise one (review sees the diff).
pub fn write(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut text = String::from(
        "# document::eval calls still in each kit file. A count may only fall:\n\
         # call the DOM through web-sys instead (crates/rv2-kit/src/internal.rs),\n\
         # then lower the line, or remove it at 0. See tooling/xtask/src/kit_eval.rs.\n",
    );
    for (file, count) in counts(root)? {
        text.push_str(&format!("{file} {count}\n"));
    }
    std::fs::write(root.join(RATCHET), text)?;
    Ok(())
}

pub fn check(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let counts = counts(root)?;
    let ratchet = parse_ratchet(&std::fs::read_to_string(root.join(RATCHET)).unwrap_or_default());
    let problems = judge(&counts, &ratchet);
    if problems.is_empty() {
        let total: usize = counts.values().sum();
        println!(
            "held ({total} document::eval call(s) in {} file(s), none outside {RATCHET})",
            counts.len()
        );
        return Ok(());
    }
    Err(format!(
        "the kit's eval ratchet moved\n         {}",
        problems.join("\n         ")
    )
    .into())
}
