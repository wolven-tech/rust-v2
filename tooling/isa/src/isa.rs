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
//! gives: it needs no app to compile, and it reaches every app at once. The
//! same reason lets it run in a repository that only depends on the kit.

use std::path::{Path, PathBuf};

/// The marker that admits one raw element, with its reason.
pub const OPT_OUT: &str = "isa-raw-ok";

/// RSX elements an app must not write itself, each with the instructions that
/// replace it. A finding prints these, so the fix is named where the mistake
/// is; `rv2-isa show <name>` gives the props and an example.
pub const REPLACEMENTS: &[(&str, &[&str])] = &[
    (
        "a",
        &[
            "TextLink",
            "LinkButton",
            "ArrowLink",
            "PressLink",
            "ProductHeader",
        ],
    ),
    (
        "button",
        &[
            "Button",
            "PressButton",
            "ArmedDeleteButton",
            "SaveToggle",
            "Chip",
            "SegmentedControl",
            "RowMenu",
        ],
    ),
    (
        "input",
        &["TextField", "Input", "UnitField", "Checkbox", "ChoiceGroup"],
    ),
    ("select", &["Select", "CommandSelect"]),
    ("textarea", &["TextArea"]),
    ("h1", &["Heading"]),
    ("h2", &["Heading"]),
    ("h3", &["Heading"]),
    ("h4", &["Heading"]),
];

/// The instructions that replace `element`.
pub fn replacements(element: &str) -> &'static [&'static str] {
    REPLACEMENTS
        .iter()
        .find(|(name, _)| *name == element)
        .map_or(&[], |(_, with)| with)
}

/// The raw elements opened on one line of RSX.
///
/// An element opens where its name is the first thing on the line or follows
/// an opening brace, and is followed by whitespace and a brace of its own —
/// `button {`, `li { a {`. That shape is what RSX looks like and what Rust
/// expressions do not: `match a {` and `if a {` have a keyword before the name.
pub fn raw_elements(line: &str) -> Vec<&'static str> {
    let code = line.trim_start();
    if code.starts_with("//") {
        return Vec::new();
    }
    let mut found = Vec::new();
    for &(name, _) in REPLACEMENTS {
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

/// One unmarked raw element.
#[derive(Debug, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub text: String,
    pub elements: Vec<&'static str>,
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.path, self.line, self.text)?;
        for element in &self.elements {
            write!(
                f,
                "\n    raw `{element}` → use {}",
                replacements(element).join(" or ")
            )?;
        }
        Ok(())
    }
}

/// Every unmarked raw element in one file.
pub fn scan(path: &str, source: &str) -> Vec<Finding> {
    let lines: Vec<&str> = source.lines().collect();
    let mut findings = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let elements = raw_elements(line);
        if elements.is_empty() || line.contains(OPT_OUT) {
            continue;
        }
        let excused = lines[..index]
            .iter()
            .rev()
            .take_while(|l| l.trim_start().starts_with("//"))
            .any(|l| l.contains(OPT_OUT));
        if !excused {
            findings.push(Finding {
                path: path.to_string(),
                line: index + 1,
                text: line.trim().to_string(),
                elements,
            });
        }
    }
    findings
}

pub(crate) fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) -> std::io::Result<()> {
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

/// `root/apps/*/src`, the directories an rv2 workspace keeps its apps in.
///
/// Sources only: `examples/` are previews of the kit's own markup, and `tests/`
/// render pages rather than define them.
pub fn app_sources(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    let apps = root.join("apps");
    if apps.is_dir() {
        for app in std::fs::read_dir(apps)? {
            let src = app?.path().join("src");
            if src.is_dir() {
                dirs.push(src);
            }
        }
    }
    dirs.sort();
    Ok(dirs)
}

/// Scan every `.rs` file under `dirs`, naming paths relative to `root`.
/// Returns the findings and the number of files read.
pub fn scan_dirs(root: &Path, dirs: &[PathBuf]) -> std::io::Result<(Vec<Finding>, usize)> {
    let mut files = Vec::new();
    for dir in dirs {
        rust_files(dir, &mut files)?;
    }
    files.sort();

    let mut findings = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)?;
        let shown = file
            .strip_prefix(root)
            .unwrap_or(file)
            .display()
            .to_string();
        findings.extend(scan(&shown, &source));
    }
    Ok((findings, files.len()))
}

/// The gate: scan `root/apps/*/src` and fail on any finding.
pub fn check(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let dirs = app_sources(root)?;
    let (findings, files) = scan_dirs(root, &dirs)?;
    report(&findings, files)
}

/// Print the findings and turn them into the gate's verdict.
pub fn report(findings: &[Finding], files: usize) -> Result<(), Box<dyn std::error::Error>> {
    if findings.is_empty() {
        println!("clean ({files} files)");
        return Ok(());
    }
    for finding in findings {
        eprintln!("{finding}");
    }
    Err(format!(
        "{} raw control(s), heading(s) or link(s) instead of rv2-ui instructions\n         \
         `rv2-isa show <Name>` prints an instruction's props and an example\n         \
         if the element is genuinely the subject of the page, mark it `{OPT_OUT}: <why>`",
        findings.len()
    )
    .into())
}
