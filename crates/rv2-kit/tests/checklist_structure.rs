//! Onboarding checklist structure at rest: header, progress band and the group list stacked in
//! order, with the open, locked and complete groups in their demo states.

use dioxus::prelude::*;
use rv2_kit::components::onboarding_checklist_panel::{
    CheckGroup, CheckItem, OnboardingChecklistPanel,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn item(id: &str, text: &str, done: bool, locked: bool) -> CheckItem {
    CheckItem {
        item_id: id.into(),
        item_text: text.into(),
        is_done: done,
        is_locked: locked,
    }
}

fn setup_groups() -> Vec<CheckGroup> {
    vec![
        CheckGroup {
            group_id: "setup".into(),
            group_title: "Set up".into(),
            task_list: vec![
                item("a", "Connect a model", true, false),
                item("b", "Invite a teammate", true, false),
                item("c", "Import the first document set", false, false),
            ],
        },
        CheckGroup {
            group_id: "safety".into(),
            group_title: "Safety".into(),
            task_list: vec![
                item("d", "Review permissions", false, true),
                item("e", "Set a spend limit", false, false),
            ],
        },
        CheckGroup {
            group_id: "team".into(),
            group_title: "Team".into(),
            task_list: vec![item("f", "Name the workspace owner", true, false)],
        },
    ]
}

fn checklist_app() -> Element {
    rsx! {
        OnboardingChecklistPanel {
            group_list: setup_groups(),
            on_edit: move |_| {},
            on_dismiss: move |_| {},
        }
    }
}

fn checklist_without_edit_app() -> Element {
    rsx! {
        OnboardingChecklistPanel { group_list: setup_groups() }
    }
}

#[test]
fn checklist_stacks_header_then_progress_then_groups() {
    let html = render(checklist_app);
    let head = html.find("kit-ocp__head").expect("header");
    let progress = html.find("kit-ocp__progress").expect("progress band");
    let groups = html.find("kit-ocp__group\"").expect("group list");
    assert!(head < progress && progress < groups);
}

#[test]
fn checklist_header_places_edit_and_close_at_trailing_edge() {
    let html = render(checklist_app);
    let title = html.find("Setup guide").expect("title");
    let edit = html.find(">Edit<").expect("edit control");
    let close = html.find("aria-label=\"Close\"").expect("close control");
    assert!(title < edit && edit < close);
}

#[test]
fn checklist_progress_is_half_and_reads_three_of_six() {
    let html = render(checklist_app);
    assert!(html.contains("aria-valuenow=\"50\""));
    assert!(html.contains("3 of 6 done"));
    assert!(!html.contains("Setup complete"));
}

#[test]
fn checklist_groups_show_titles_in_order_with_their_own_counts() {
    let html = render(checklist_app);
    let one = html.find("Set up").expect("group one");
    let two = html.find("Safety").expect("group two");
    let three = html.find("Team").expect("group three");
    assert!(one < two && two < three);
    assert!(html.contains("2 of 3"));
    assert!(html.contains("0 of 2"));
    assert!(html.contains("1 of 1"));
}

#[test]
fn checklist_open_groups_show_their_rows_and_complete_group_hides_them() {
    let html = render(checklist_app);
    assert_eq!(html.matches("class=\"kit-ocp__toggle\"").count(), 5);
    assert!(html.contains("Import the first document set"));
    assert!(html.contains("Set a spend limit"));
    assert!(!html.contains("Name the workspace owner"));
}

/// Ticked and disabled states are DOM properties that SSR does not serialise; the open rows are
/// counted here and their state is checked in the browser.
#[test]
fn checklist_open_groups_render_five_toggle_rows() {
    let html = render(checklist_app);
    assert_eq!(html.matches("aria-pressed=").count(), 5);
}

#[test]
fn checklist_complete_group_header_is_marked_done() {
    let html = render(checklist_app);
    assert!(html.contains("data-done=\"true\""));
}

#[test]
fn checklist_omits_edit_control_without_on_edit() {
    let html = render(checklist_without_edit_app);
    assert!(!html.contains(">Edit<"));
    assert!(!html.contains("aria-label=\"Close\""));
}
