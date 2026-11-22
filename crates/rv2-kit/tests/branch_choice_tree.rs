//! Branch choice tree: the question card, the tree of branch nodes, follow-up groups and the header count.

use dioxus::prelude::*;
use rv2_kit::components::branch_choice_map::{
    Branch, BranchChoiceMap, Consequence, FollowUp, Fork, Risk,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn plain(text: &str) -> Consequence {
    Consequence {
        text: text.into(),
        flag_explanation: None,
        reference_url: None,
    }
}

fn option(label: &str) -> Branch {
    Branch {
        branch_label: label.into(),
        branch_mark: None,
        consequences: vec![],
        touched_apps: vec![],
        agent_preferred: false,
        follow_up: None,
    }
}

fn branch(
    label: &str,
    lines: Vec<Consequence>,
    preferred: bool,
    follow_up: Option<FollowUp>,
) -> Branch {
    Branch {
        branch_label: label.into(),
        branch_mark: None,
        consequences: lines,
        touched_apps: vec![],
        agent_preferred: preferred,
        follow_up,
    }
}

fn follow(question: &str, first: &str, second: &str) -> Option<FollowUp> {
    Some(FollowUp {
        question_text: question.into(),
        branches: vec![option(first), option(second)],
    })
}

fn gallery_fork(fork_id: &str) -> Fork {
    Fork {
        fork_id: fork_id.into(),
        question_text: "Apply the schema change now, or after the replica catches up?".into(),
        context_text: None,
        agent_name: "Migration agent".into(),
        model_name: Some("model A".into()),
        agent_mark: None,
        risk_level: Some(Risk::High),
        branches: vec![
            branch(
                "Apply now",
                vec![plain("Table locks for about 40 seconds")],
                false,
                follow(
                    "Notify the owner first?",
                    "Notify now",
                    "Notify after cutover",
                ),
            ),
            branch(
                "Wait for replica",
                vec![plain("Migration finishes once the replica catches up")],
                true,
                follow(
                    "Run the backfill in the same window?",
                    "Backfill tonight",
                    "Backfill next week",
                ),
            ),
            branch(
                "Stop and report",
                vec![plain("No schema change is made")],
                false,
                None,
            ),
        ],
    }
}

fn gallery_app() -> Element {
    let second = gallery_fork("f2");
    rsx! { BranchChoiceMap { pending_forks: vec![gallery_fork("f1"), second] } }
}

fn picked_app() -> Element {
    rsx! { BranchChoiceMap { pending_forks: vec![gallery_fork("f1")], initial_pick: Some(0) } }
}

fn position(html: &str, needle: &str) -> usize {
    html.find(needle)
        .unwrap_or_else(|| panic!("{needle} missing from {html}"))
}

#[test]
fn the_question_card_sits_before_the_tree_and_each_rail_is_hidden_from_assistive_tech() {
    let html = render(gallery_app);
    assert!(position(&html, "kit-bcm__question-card") < position(&html, "kit-bcm__tree-list"));
    assert_eq!(html.matches("kit-bcm__rail").count(), 2);
    assert_eq!(
        html.matches("kit-bcm__rail\" aria-hidden=\"true\"").count(),
        2
    );
}

#[test]
fn each_follow_up_group_sits_right_of_its_consequences_inside_its_own_row() {
    let html = render(gallery_app);
    let cons = position(&html, "kit-bcm__consequences");
    let group = cons + html[cons..].find("kit-bcm__follow").unwrap();
    assert!(!html[cons..group].contains("kit-bcm__row\""));
    assert!(html[group..].starts_with("kit-bcm__follow\""));
}

#[test]
fn both_follow_up_previews_show_dimmed_with_their_options_out_of_tab_order() {
    let html = render(gallery_app);
    assert_eq!(
        html.matches("kit-bcm__follow\" data-live=\"false\"")
            .count(),
        2
    );
    assert_eq!(html.matches("disabled").count(), 4);
    assert!(!html.contains("data-live=\"true\""));
}

#[test]
fn the_preference_and_follow_up_tags_mark_their_nodes_and_the_header_counts_waiting_forks() {
    let html = render(gallery_app);
    assert_eq!(html.matches("kit-bcm__tag").count(), 3);
    assert_eq!(html.matches(">Follow-up<").count(), 2);
    assert!(html.contains("2 waiting"));
    assert!(html.contains("1 of 2"));
    assert_eq!(html.matches("data-current=\"true\"").count(), 1);
    assert_eq!(html.matches("kit-bcm__dot\"").count(), 2);
}

#[test]
fn a_first_branch_picked_makes_its_follow_up_live_and_leaves_the_others_dimmed_without_an_outcome()
{
    let html = render(picked_app);
    assert_eq!(html.matches("data-live=\"true\"").count(), 1);
    assert_eq!(html.matches("data-live=\"false\"").count(), 1);
    assert_eq!(html.matches("disabled").count(), 2);
    assert_eq!(html.matches("data-dimmed=\"true\"").count(), 2);
    assert!(!html.contains("kit-bcm__outcome"));
}
