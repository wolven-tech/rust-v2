use std::path::{Path, PathBuf};

use rv2_kit::index::{self, Index, Mode};
use rv2_kit::isa::REPLACEMENTS;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn kit() -> Index {
    index::build(&workspace(), Mode::Strict).expect("the kit indexes cleanly")
}

#[test]
fn the_committed_index_is_what_the_kit_generates() {
    assert_eq!(
        index::sync(&workspace(), false).unwrap(),
        Vec::<String>::new(),
        "run `cargo run -p rv2-kit -- index --write`"
    );
}

#[test]
fn every_instruction_the_scan_suggests_exists() {
    let kit = kit();
    for (element, with) in REPLACEMENTS {
        for name in *with {
            assert!(
                kit.component(name).is_some_and(|c| c.in_isa),
                "the scan suggests `{name}` for `{element}`, which is not an instruction"
            );
        }
    }
}

#[test]
fn every_public_instruction_has_an_intent_and_motion_has_none() {
    let kit = kit();
    assert!(kit.problems().is_empty(), "{:?}", kit.problems());
    for component in &kit.components {
        assert_eq!(
            component.in_isa,
            component.module != "motion",
            "{}",
            component.name
        );
    }
}

#[test]
fn a_search_by_intent_finds_the_instruction_first() {
    let kit = kit();
    let first = |query: &str| kit.search(query).first().map(|c| c.name.clone());
    assert_eq!(
        first("delete a row without a modal").as_deref(),
        Some("ArmedDeleteButton")
    );
    assert_eq!(first("show a trend").as_deref(), Some("Sparkline"));
    assert_eq!(first("check readiness").as_deref(), Some("GateList"));
    assert!(kit.search("zzzz qqqq").is_empty());
}

#[test]
fn show_lists_the_parts_a_component_takes_and_what_builds_them() {
    let kit = kit();
    let gates = kit.component("GateList").unwrap();
    let used: Vec<&str> = kit
        .types_used_by(gates)
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert!(used.contains(&"Gate"), "{used:?}");
    assert!(
        used.contains(&"Verdict"),
        "Gate::new takes a Verdict: {used:?}"
    );
    let gate = kit.type_doc("Gate").unwrap();
    assert!(
        gate.signatures.iter().any(|s| s.starts_with("fn new(")),
        "{:?}",
        gate.signatures
    );
}

#[test]
fn props_say_which_are_required_and_what_the_defaults_are() {
    let kit = kit();
    let dialog = kit.component("ConfirmDialog").unwrap();
    let prop = |name: &str| dialog.props.iter().find(|p| p.name == name).unwrap();
    assert!(prop("title").required);
    assert!(!prop("children").required);
    assert!(!prop("error").required, "an Option is optional");
    assert_eq!(
        prop("cancel_label").default.as_deref(),
        Some("\"Keep it\".to_string()")
    );
}

#[test]
fn an_example_skips_definitions_and_braces_inside_strings() {
    let source = "pub struct Card { x: u8 }\n\
                  fn page() -> Element {\n\
                  \x20   rsx! {\n\
                  \x20       Card { class: \"}{\",\n\
                  \x20           Text { \"a } b\" }\n\
                  \x20       }\n\
                  \x20   }\n\
                  }\n";
    let example = index::example("x.rs", source, "Card").expect("an example");
    assert_eq!(example.line, 4);
    assert_eq!(
        example.code,
        "Card { class: \"}{\",\n    Text { \"a } b\" }\n}"
    );
    assert!(index::example("x.rs", source, "CardList").is_none());
}

#[test]
fn intents_come_from_the_instruction_table_only() {
    let doc = "## Operands\n\n| A | B |\n|---|---|\n| `Variant` | x |\n\n\
               ## Instructions\n\n| Intent | Instruction | Emits | Replaces |\n|---|---|---|---|\n\
               | Do something | `Button`¹, `use_toast` | `button` | — |\n\n## Next\n";
    let rows = index::intents(doc);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].intent, "Do something");
    assert_eq!(rows[0].instructions, ["Button", "use_toast"]);
    assert_eq!(rows[0].emits, "`button`");
}

#[test]
fn a_fork_without_the_docs_still_indexes_its_own_kit() {
    let root = std::env::temp_dir().join(format!("rv2-kit-fork-{}", std::process::id()));
    let src = root.join(index::KIT_SRC);
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("lib.rs"), "pub mod widgets;\n").unwrap();
    std::fs::write(
        src.join("widgets.rs"),
        "/// A local button.\n#[component]\npub fn Knob(label: String) -> Element { todo!() }\n",
    )
    .unwrap();

    let lenient = index::build(&root, Mode::Lenient).unwrap();
    assert_eq!(lenient.components.len(), 1);
    assert_eq!(lenient.components[0].docs, "A local button.");
    assert!(
        index::build(&root, Mode::Strict).is_err(),
        "strict needs the ISA doc"
    );
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_embedded_json_round_trips() {
    let kit = kit();
    assert_eq!(Index::from_json(&kit.to_json()).unwrap(), kit);
}
