use dioxus::prelude::*;
use rv2_kit::components::branch_choice_map::{
    Branch, BranchChoiceMap, Consequence, FollowUp, Fork, Risk,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn line(text: &str, flag: Option<&str>) -> Consequence {
    Consequence {
        text: text.into(),
        flag_explanation: flag.map(str::to_string),
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

fn migration_fork() -> Fork {
    Fork {
        fork_id: "f1".into(),
        question_text: "Apply the change now?".into(),
        context_text: None,
        agent_name: "Migration agent".into(),
        model_name: Some("model A".into()),
        agent_mark: None,
        risk_level: Some(Risk::High),
        branches: vec![
            Branch {
                consequences: vec![line("Table locks for 40 seconds", Some("Writes queue."))],
                ..option("Apply now")
            },
            Branch {
                consequences: vec![line("Migration waits for the replica", None)],
                agent_preferred: true,
                follow_up: Some(FollowUp {
                    question_text: "Backfill in the same window?".into(),
                    branches: vec![option("Backfill tonight"), option("Backfill next week")],
                }),
                ..option("Wait for replica")
            },
            option("Stop and report"),
        ],
    }
}

#[test]
fn gallery_at_rest_shows_pager_risk_agent_branches_preference_and_custom_row() {
    fn app() -> Element {
        let second = Fork {
            fork_id: "f2".into(),
            ..migration_fork()
        };
        rsx! { BranchChoiceMap { pending_forks: vec![migration_fork(), second] } }
    }
    let html = render(app);
    assert!(html.contains("1 of 2"));
    let risk = html.find("High risk").unwrap();
    let agent = html.find("Migration agent").unwrap();
    assert!(risk < agent);
    assert!(html.contains("model A"));
    assert_eq!(html.matches("kit-bcm__key").count(), 3 + 2 + 1);
    assert!(html.contains("kit-bcm__tag"));
    assert!(html.contains("pick"));
    let custom = html.find("Write your own instruction").unwrap();
    assert!(custom > html.find("Stop and report").unwrap());
}

#[test]
fn follow_up_preview_is_dimmed_and_its_controls_are_disabled_until_picked() {
    fn app() -> Element {
        rsx! { BranchChoiceMap { pending_forks: vec![migration_fork()] } }
    }
    let html = render(app);
    let follow = &html[html.find("kit-bcm__follow").unwrap()..];
    assert!(follow.starts_with("kit-bcm__follow\" data-live=\"false\""));
    assert!(follow.contains("Backfill tonight"));
    let end = follow.find("</section>").unwrap();
    assert_eq!(follow[..end].matches("disabled").count(), 2);
}

#[test]
fn consequence_lines_share_the_branch_row_band() {
    fn app() -> Element {
        rsx! { BranchChoiceMap { pending_forks: vec![migration_fork()] } }
    }
    let html = render(app);
    let row = &html[html.find("kit-bcm__row").unwrap()..];
    let row = &row[..row.find("</li>").unwrap()];
    let button = row.find("kit-bcm__branch").unwrap();
    let consequences = row.find("kit-bcm__consequences").unwrap();
    assert!(button < consequences);
    assert!(row.contains("Table locks for 40 seconds"));
}

#[test]
fn second_instance_with_branch_picked_shows_live_follow_up_and_no_outcome() {
    fn app() -> Element {
        rsx! { BranchChoiceMap { pending_forks: vec![migration_fork()], initial_pick: Some(1) } }
    }
    let html = render(app);
    assert!(html.contains("data-live=\"true\""));
    assert!(!html.contains("data-live=\"false\""));
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(!html.contains("kit-bcm__outcome"));
    assert!(!html.contains("disabled"));
}

#[test]
fn flagged_consequence_shows_marker_with_note_hidden_at_rest() {
    fn app() -> Element {
        rsx! { BranchChoiceMap { pending_forks: vec![migration_fork()] } }
    }
    let html = render(app);
    assert!(html.contains("kit-bcm__flag"));
    assert!(html.contains("aria-describedby"));
    assert!(!html.contains("role=\"note\""));
    assert!(!html.contains("Writes queue."));
}
