#[allow(dead_code)]
#[path = "../src/kit_eval.rs"]
mod kit_eval;

use std::collections::BTreeMap;

use kit_eval::{count_calls, judge, parse_ratchet};

fn map(pairs: &[(&str, usize)]) -> BTreeMap<String, usize> {
    pairs
        .iter()
        .map(|(path, count)| (path.to_string(), *count))
        .collect()
}

#[test]
fn calls_are_counted_and_comments_are_not() {
    let code = "document::eval(\"a\");\n// document::eval in a comment\nlet x = document::eval(&s); document::eval(&t);\n";
    assert_eq!(count_calls(code), 3);
}

#[test]
fn a_file_off_the_list_may_not_eval() {
    let problems = judge(&map(&[("crates/rv2-kit/src/internal.rs", 1)]), &map(&[]));
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("0 allowed"));
}

#[test]
fn a_file_may_not_exceed_its_count() {
    let problems = judge(&map(&[("a.rs", 3)]), &map(&[("a.rs", 2)]));
    assert!(problems[0].contains("2 allowed"));
}

#[test]
fn a_count_that_fell_must_be_lowered_on_the_list() {
    let problems = judge(&map(&[("a.rs", 1)]), &map(&[("a.rs", 2), ("b.rs", 1)]));
    assert_eq!(problems.len(), 2);
    assert!(problems.iter().all(|p| p.contains("lower its line")));
}

#[test]
fn matching_counts_hold() {
    assert!(judge(&map(&[("a.rs", 2)]), &map(&[("a.rs", 2)])).is_empty());
}

#[test]
fn the_list_reads_paths_and_counts_and_skips_comments() {
    let ratchet = parse_ratchet("# header\ncrates/rv2-kit/src/a.rs 4  # why\n\ncrates/x.rs 1\n");
    assert_eq!(
        ratchet,
        map(&[("crates/rv2-kit/src/a.rs", 4), ("crates/x.rs", 1)])
    );
}

#[test]
fn the_shared_helpers_stay_free_of_eval() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ratchet = parse_ratchet(&std::fs::read_to_string(root.join(kit_eval::RATCHET)).unwrap());
    assert!(!ratchet.contains_key("crates/rv2-kit/src/internal.rs"));
    let counts = kit_eval::counts(&root).unwrap();
    assert!(!counts.contains_key("crates/rv2-kit/src/internal.rs"));
    assert!(
        judge(&counts, &ratchet).is_empty(),
        "{:?}",
        judge(&counts, &ratchet)
    );
}
