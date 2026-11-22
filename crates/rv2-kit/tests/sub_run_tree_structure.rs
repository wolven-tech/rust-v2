//! Structure and demo checks for the sub-run tree amendment: the gallery tree
//! (nine units, Changelog folded), the fold label and folded summary row, and
//! the phase rules that hide elapsed time from units that have not started.

use dioxus::prelude::*;
use rv2_kit::agents::single_run_status_card::format_usd;
use rv2_kit::agents::sub_run_tree::{
    Node, NodePhase, SubRunTree, elapsed_ms, identity_slot, subtree_spend, summarize,
};

const NOW_MS: u64 = 1_000_000;

fn unit(key: &str, caption: &str, phase: NodePhase, spend: f64, branches: Vec<Node>) -> Node {
    Node {
        node_key: key.to_string(),
        caption: caption.to_string(),
        brief: format!("{caption} brief"),
        model_tag: Some("model A".to_string()),
        tool_marks: Vec::new(),
        phase,
        spend,
        began_at_ms: (phase != NodePhase::Pending).then_some(NOW_MS - 60_000),
        ended_at_ms: None,
        needs_text: None,
        fault_text: None,
        branches,
    }
}

/// The gallery's demo tree: nine units, three active, one awaiting input, one errored.
fn gallery_root() -> Node {
    let mut collect = unit(
        "n5",
        "Collect commits",
        NodePhase::Complete,
        0.04,
        Vec::new(),
    );
    collect.ended_at_ms = Some(NOW_MS - 240_000);
    let changelog = unit("n2", "Changelog", NodePhase::Complete, 0.09, vec![collect]);
    let summarise = unit("n7", "Summarise PRs", NodePhase::Active, 0.21, Vec::new());
    let tone = unit("n8", "Tone check", NodePhase::Pending, 0.0, Vec::new());
    let draft = unit(
        "n6",
        "Draft notes",
        NodePhase::Active,
        0.12,
        vec![summarise, tone],
    );
    let mut lint = unit("n3", "Lint pass", NodePhase::Errored, 0.12, Vec::new());
    lint.fault_text = Some("Lint exited 2".to_string());
    let mut waiting = unit(
        "n4",
        "Pick the release date",
        NodePhase::AwaitingInput,
        0.12,
        Vec::new(),
    );
    waiting.needs_text = Some("Which date?".to_string());
    let publish = unit("n9", "Publish", NodePhase::Pending, 0.0, Vec::new());
    unit(
        "n1",
        "Release",
        NodePhase::Active,
        0.12,
        vec![changelog, draft, lint, waiting, publish],
    )
}

fn gallery() -> Element {
    let root_node = Some(gallery_root());
    rsx! {
        SubRunTree {
            root_node,
            clock_ms: NOW_MS,
            default_folded_keys: vec!["n2".to_string()],
        }
    }
}

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn gallery_strip_reads_nine_units_three_active_one_awaiting_and_one_errored() {
    let summary = summarize(Some(&gallery_root()));
    assert_eq!(summary.units, 9);
    assert_eq!(summary.active, 3);
    assert_eq!(summary.awaiting, 1);
    assert_eq!(summary.errored, 1);
    let html = render(gallery);
    assert!(html.contains("9 units"));
    assert!(html.contains("3 active"));
    assert!(html.contains("1 awaiting input"));
    assert!(html.contains("1 errored"));
}

#[test]
fn changelog_fold_reads_show_one_unit_and_hides_collect_commits() {
    let html = render(gallery);
    assert!(html.contains("aria-label=\"Show 1 unit under Changelog\""));
    assert!(!html.contains("Collect commits"));
    assert!(html.contains("1 unit folded"));
}

#[test]
fn folded_summary_row_spend_equals_the_hidden_units_spend() {
    let root = gallery_root();
    let changelog = &root.branches[0];
    let hidden = subtree_spend(changelog) - changelog.spend;
    assert!((hidden - 0.04).abs() < 1e-9);
    let html = render(gallery);
    assert!(html.contains(&format_usd(hidden)));
}

#[test]
fn unstarted_units_show_no_elapsed_figure() {
    let root = gallery_root();
    let publish = &root.branches[4];
    let tone = &root.branches[1].branches[1];
    assert_eq!(elapsed_ms(publish, NOW_MS), None);
    assert_eq!(elapsed_ms(tone, NOW_MS), None);
}

#[test]
fn failed_unit_keeps_its_failure_word_inside_its_accessible_name() {
    let html = render(gallery);
    assert!(html.contains("aria-label=\"Lint pass, Failed\""));
    assert!(html.contains("aria-label=\"Pick the release date, Needs you\""));
}

#[test]
fn identity_slot_is_one_of_the_six_kit_identities() {
    for key in ["n1", "n2", "Lint pass", ""] {
        assert!((1..=6).contains(&identity_slot(key)));
    }
}

#[test]
fn tree_stylesheet_lays_first_tier_side_by_side_past_the_container_breakpoint() {
    let css = include_str!("../assets/sub-run-tree.css");
    assert!(css.contains("@container (min-width: 42rem)"));
    assert!(css.contains(".kit-srt__children > .kit-srt__unit:first-child::after"));
}
