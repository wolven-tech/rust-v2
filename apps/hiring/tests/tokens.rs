//! Every token a component reaches for is defined in this app's theme.
//!
//! The companion rule — that no component names a colour by its hue — used to
//! live here too, as a scan over this app's source and a few of the kit's
//! files. It now runs as `no hardcoded hues` in `cargo xtask ci`, because a
//! scan over source text needed neither this crate's test binary to compile
//! nor to be restricted to the files this app happens to own. Keeping a copy
//! here would be two definitions of one rule, free to disagree.
//!
//! This check is the other half and cannot move: it reads the theme, so it is
//! per-app by nature. A class naming a token nothing defines fails silently —
//! the browser drops the declaration and the element renders with whatever it
//! inherited.

use std::path::Path;

/// The theme this app compiles against: the kit's registers, then this app's
/// overrides.
///
/// Both files, in import order. Reading only `assets/input.css` stopped being
/// enough once the shared registers and the registry aliases moved to
/// `crates/rv2-ui/assets/theme.css` (UI2): a token defined there is defined
/// here too, and a check that looked only at the override file would fail on
/// every alias and miss a kit register that went missing at the source.
fn theme() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shared = std::fs::read_to_string(root.join("../../crates/rv2-ui/assets/theme.css"))
        .expect("the kit's shared theme must be readable");
    let own = std::fs::read_to_string(root.join("assets/input.css"))
        .expect("the theme block must be readable");
    assert!(
        own.contains("crates/rv2-ui/assets/theme.css"),
        "assets/input.css must import the kit's shared theme"
    );
    format!("{shared}\n{own}")
}

#[test]
fn every_semantic_token_a_component_uses_is_defined_in_the_theme() {
    let theme = theme();

    let tokens = [
        "ground",
        "surface",
        "raised",
        "ink",
        "ink-hover",
        "ink-muted",
        "ink-faint",
        "on-ink",
        "on-ink-muted",
        "on-ink-faint",
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
        "rust-ink",
        "index-ftse100",
        "index-ftse250",
        "index-smallcap",
        "index-aim",
        "index-targets",
        "clears-1",
        "clears-2",
        "misses-1",
        "misses-2",
    ];

    for token in tokens {
        assert!(
            theme.contains(&format!("--color-{token}:")),
            "--color-{token} is used by a component but not defined in the theme"
        );
    }
}

/// The shadcn-shaped names a copied component styles itself from.
///
/// They are aliases onto the tokens above rather than values, so a registry
/// component needs no class rewriting. `accent` is absent on purpose and must
/// stay absent: shadcn spends it on a hover surface and this theme spends it on
/// the focus ring, so defining it here would repaint every borrowed hover state
/// in the focus colour.
#[test]
fn the_registry_aliases_are_defined_and_accent_is_not_among_them() {
    let theme = theme();

    for alias in [
        "background",
        "foreground",
        "muted",
        "muted-foreground",
        "input",
        "ring",
        "primary",
        "primary-foreground",
    ] {
        assert!(
            theme.contains(&format!("--color-{alias}:")),
            "--color-{alias} is a registry alias a borrowed component needs"
        );
    }

    assert!(
        !theme.contains("--color-accent: var("),
        "--color-accent must keep meaning the focus ring, not shadcn's hover surface"
    );
}
