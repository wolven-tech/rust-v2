//! Golden-corpus test for D12 (event evolution is additive-only).
//!
//! `tests/golden/` holds one JSON file per `(wire_type, schema_version)` this
//! codebase has ever released, captured at release time. Every file must keep
//! decoding into the *current* build of `rv2-events`, forever.
//!
//! ## How to use this when you change an event
//!
//! - **Adding a field?** Give it `#[serde(default)]`. The existing golden files
//!   have no such key and must still decode — this test proves it.
//! - **Anything else** (rename, retype, tighten `Option<T>` to `T`, remove a
//!   variant) will fail here. That is the point: it fails at the commit that
//!   introduces it, not six months later when an old stream is re-folded.
//! - **Releasing a schema bump?** Add a new
//!   `<wire_type>.v<n>.json` file. Never edit or delete an existing one.

use std::{collections::BTreeSet, fs, path::PathBuf};

use rv2_events::{DomainEvent, EventEnvelope, decode};

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn golden_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(golden_dir())
        .expect("tests/golden must exist")
        .map(|entry| entry.expect("readable dir entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_golden_envelope_still_decodes() {
    let files = golden_files();
    assert!(!files.is_empty(), "the golden corpus must not be empty");

    for path in files {
        let raw = fs::read_to_string(&path).expect("golden file is readable");
        let envelope: EventEnvelope = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{} is not a valid EventEnvelope: {e}", path.display()));
        let event = decode(&envelope)
            .unwrap_or_else(|e| panic!("{} no longer decodes: {e}", path.display()));

        // The file name encodes the wire type it is a golden for; check that
        // the mapping still agrees, so a renamed wire type is caught here too.
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let wire = stem
            .rsplit_once(".v")
            .map_or(stem.as_str(), |(wire, _)| wire)
            .to_string();
        assert_eq!(
            event.event_type(),
            wire,
            "{} decodes to a different wire type than its name claims",
            path.display()
        );
    }
}

/// A wire type with no golden file is a wire type whose forward compatibility
/// nobody is checking. Adding a variant therefore requires adding a fixture.
#[test]
fn every_wire_type_has_at_least_one_golden_file() {
    let covered: BTreeSet<String> = golden_files()
        .iter()
        .map(|path| {
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            stem.rsplit_once(".v")
                .map_or(stem.clone(), |(wire, _)| wire.to_string())
        })
        .collect();

    for wire in DomainEvent::ALL_WIRE_TYPES {
        assert!(
            covered.contains(*wire),
            "no golden fixture for `{wire}` — add tests/golden/{wire}.v1.json"
        );
    }
}
