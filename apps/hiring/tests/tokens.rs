//! No component may name a colour by its hue.
//!
//! The page's palette lives in one `@theme` block. A component that writes
//! `text-slate-600` instead of `text-ink-muted` silently opts that element out
//! of every future theme, and the failure is invisible until someone switches
//! to dark and finds one paragraph still light grey on a dark ground.
//!
//! A convention nobody enforces gets re-broken, so this is a test rather than a
//! note in a doc comment. It scans source rather than rendered output because
//! the offending class may sit on a branch the tests never render.

use std::path::Path;

/// Tailwind's built-in palettes. Naming one of these ties an element to a hue
/// rather than to its job.
const PALETTES: &[&str] = &[
    "slate", "gray", "zinc", "neutral", "stone", "red", "orange", "amber", "yellow", "lime",
    "green", "emerald", "teal", "cyan", "sky", "blue", "indigo", "violet", "purple", "fuchsia",
    "pink", "rose",
];

/// Files that make up the page's visual surface.
const SOURCES: &[&str] = &[
    "src/dashboard.rs",
    "src/lib.rs",
    "src/rail.rs",
    "src/results.rs",
    "../../crates/rv2-ui/src/feedback.rs",
    "../../crates/rv2-ui/src/form.rs",
    "../../crates/rv2-ui/src/layout.rs",
    "../../crates/rv2-ui/src/motion.rs",
    "../../crates/rv2-ui/src/primitives.rs",
    "../../crates/rv2-ui/src/product_shell.rs",
    "../../crates/rv2-ui/src/site.rs",
    "../../crates/rv2-ui/src/tabs.rs",
    "../../crates/rv2-ui/src/typography.rs",
];

/// The two hues that carry no shade, and so cannot be caught by the
/// palette-and-shade pattern above.
///
/// `bg-white` is the one that actually shipped: it is correct on a light theme
/// and renders a white card on a dark page, and because it is spelled without a
/// number it slipped past this scan for as long as the scan only looked for
/// `{palette}-{shade}`.
const BARE: &[&str] = &["white", "black"];

/// The prefixes a colour can arrive under. `border` covers `border-x-white` and
/// friends only loosely, which is why the check below matches the hue rather
/// than trying to enumerate every prefix Tailwind allows.
const COLOUR_PREFIXES: &[&str] = &[
    "bg-",
    "text-",
    "border-",
    "ring-",
    "fill-",
    "stroke-",
    "from-",
    "to-",
    "via-",
    "divide-",
    "outline-",
    "shadow-",
    "decoration-",
    "accent-",
    "caret-",
    "placeholder-",
];

fn offenders(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    for palette in PALETTES {
        for shade in [
            "50", "100", "200", "300", "400", "500", "600", "700", "800", "900", "950",
        ] {
            let needle = format!("{palette}-{shade}");
            if source.contains(&needle) {
                found.push(needle);
            }
        }
    }
    for hue in BARE {
        for prefix in COLOUR_PREFIXES {
            let needle = format!("{prefix}{hue}");
            if source.contains(&needle) {
                found.push(needle);
            }
        }
    }
    found
}

#[test]
fn no_component_names_a_colour_by_its_hue() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut failures = Vec::new();

    for relative in SOURCES {
        let path = root.join(relative);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        for offender in offenders(&source) {
            failures.push(format!("{relative}: {offender}"));
        }
    }

    assert!(
        failures.is_empty(),
        "these hardcode a palette colour instead of a semantic token \
         (see apps/hiring/assets/input.css for the names): {failures:#?}"
    );
}

#[test]
fn the_scan_would_actually_catch_something() {
    assert_eq!(
        offenders("class: \"text-slate-600 bg-white\""),
        vec!["slate-600".to_string(), "bg-white".to_string()],
        "the scan must find a palette class when one is present, or the test \
         above passes by never matching anything"
    );
}

/// `bg-white` used to pass this scan, and shipped.
///
/// A shade-suffixed hue and a bare one are different spellings of the same
/// defect, and the scan only looked for the first. Each prefix is checked
/// because `bg-white` and `text-white` are both reachable and neither carries a
/// number to match on.
#[test]
fn a_hue_with_no_shade_is_caught_too() {
    for spelling in ["bg-white", "text-white", "border-black", "ring-white"] {
        assert_eq!(
            offenders(&format!("class: \"p-4 {spelling} rounded\"")),
            vec![spelling.to_string()],
            "{spelling} is a hardcoded hue and has to be reported"
        );
    }
    assert!(
        offenders("class: \"bg-surface text-ink border-rule\"").is_empty(),
        "a semantic class must not be reported, or the scan cries wolf and gets ignored"
    );
}

#[test]
fn every_semantic_token_a_component_uses_is_defined_in_the_theme() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let theme = std::fs::read_to_string(root.join("assets/input.css"))
        .expect("the theme block must be readable");

    let tokens = [
        "ground",
        "surface",
        "raised",
        "ink",
        "ink-hover",
        "ink-muted",
        "ink-faint",
        "on-ink",
        "rule",
        "rule-soft",
        "accent",
        "takeable",
        "takeable-soft",
        "takeable-wash",
        "takeable-ink",
        "caution",
        "caution-ink",
        "fault",
        "fault-ink",
        "rust-strong",
        "rust-soft",
        "rust-ink",
        "index-ftse100",
        "index-ftse250",
        "index-smallcap",
        "index-aim",
        "index-targets",
    ];

    for token in tokens {
        assert!(
            theme.contains(&format!("--color-{token}:")),
            "--color-{token} is used by a component but not defined in the theme"
        );
    }
}
