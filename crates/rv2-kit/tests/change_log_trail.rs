use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::components::change_log_timeline::{
    Change, ChangeLogTimeline, ChangeWriter, KindStyle, KindTone, Timestamp, kind_tone,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn writer(name: &str, is_agent: bool) -> ChangeWriter {
    ChangeWriter {
        writer_name: name.into(),
        is_agent,
        lane_color: None,
        writer_mark: None,
    }
}

fn change(id: &str, message: &str, merged: Vec<Change>) -> Change {
    Change {
        change_id: id.into(),
        change_message: message.into(),
        change_author: writer("Ada", false),
        co_author: None,
        changed_at: Some(Timestamp::Epoch(1_000)),
        change_kind: Some("correction".into()),
        file_count: Some(4),
        lines_added: Some(88),
        lines_removed: Some(3),
        change_body: None,
        merged_changes: merged,
        in_progress: false,
    }
}

#[test]
fn kind_without_configured_colour_renders_neutral_even_when_built_in() {
    let mut overrides: BTreeMap<String, KindStyle> = BTreeMap::new();
    assert_eq!(kind_tone("housekeeping", &overrides), KindTone::Neutral);
    overrides.insert(
        "housekeeping".to_string(),
        KindStyle {
            label: None,
            color: Some("var(--kit-id-1)".into()),
        },
    );
    assert_eq!(kind_tone("housekeeping", &overrides), KindTone::Custom);
}

#[test]
fn each_row_lists_marker_message_kind_and_time_in_that_order() {
    fn app() -> Element {
        let changes = vec![change("c1", "Add `lib.rs` note", vec![])];
        rsx! { ChangeLogTimeline { change_list: changes, current_branch: "main", now_secs: 2_000 } }
    }
    let html = render(app);
    let row = &html[html.find("kit-log__row").unwrap()..];
    let marker = row.find("kit-log__marker").unwrap();
    let message = row.find("kit-log__message").unwrap();
    let kind = row.find("kit-log__kind").unwrap();
    let time = row.find("kit-log__time").unwrap();
    assert!(marker < message && message < kind && kind < time);
}

#[test]
fn header_names_branch_and_count_with_menu_closed_for_two_branches() {
    fn app() -> Element {
        let changes = vec![change("c1", "a", vec![]), change("c2", "b", vec![])];
        rsx! {
            ChangeLogTimeline {
                change_list: changes,
                branch_names: vec!["ledger-sync".to_string(), "main".to_string()],
                current_branch: "ledger-sync",
            }
        }
    }
    let html = render(app);
    assert!(html.contains("2 changes"));
    assert!(html.contains("aria-haspopup=\"menu\""));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("menuitemradio"));
}

#[test]
fn merge_row_counts_its_changes_and_hides_them_while_folded() {
    fn app() -> Element {
        let merged = vec![
            change("a", "one", vec![]),
            change("b", "two", vec![]),
            change("c", "three", vec![]),
        ];
        rsx! {
            ChangeLogTimeline {
                change_list: vec![change("m", "`ledger-sync/retry`", merged)],
                current_branch: "main",
                expanded_merge_ids: Some(vec![]),
            }
        }
    }
    let html = render(app);
    assert!(html.contains("Merged 3 changes"));
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(!html.contains("kit-log__side"));
}

#[test]
fn expanded_merge_lists_its_changes_in_a_nested_list() {
    fn app() -> Element {
        let merged = vec![change("a", "one", vec![]), change("b", "two", vec![])];
        rsx! {
            ChangeLogTimeline {
                change_list: vec![change("m", "`ledger-sync/retry`", merged)],
                current_branch: "main",
                expanded_merge_ids: Some(vec!["m".to_string()]),
            }
        }
    }
    let html = render(app);
    let side = &html[html.find("kit-log__side").unwrap()..];
    assert!(side.contains("data-change-id=\"a\""));
    assert!(side.contains("data-change-id=\"b\""));
}

#[test]
fn open_detail_shows_agent_co_author_and_size_counts() {
    fn app() -> Element {
        let mut open = change("c1", "Add `ledger.toml` schema", vec![]);
        open.co_author = Some(writer("Writer agent", true));
        rsx! {
            ChangeLogTimeline {
                change_list: vec![open],
                current_branch: "main",
                open_change_id: Some("c1".to_string()),
                on_open_change: |_: Option<String>| {},
            }
        }
    }
    let html = render(app);
    assert!(html.contains("kit-log__details"));
    assert!(html.contains("with Writer agent"));
    assert!(html.contains("kit-log__agent"));
    assert!(html.contains("(agent)"));
    assert!(html.contains("4 files"));
    assert!(html.contains("+88"));
    assert!(html.contains("−3"));
    assert!(html.contains("c1"));
}

#[test]
fn in_progress_marker_differs_from_saved_marker() {
    fn app() -> Element {
        let mut writing = change("w", "Wire `retry.rs`", vec![]);
        writing.in_progress = true;
        let saved = change("s", "Pin `Cargo.lock`", vec![]);
        rsx! { ChangeLogTimeline { change_list: vec![writing, saved], current_branch: "main" } }
    }
    let html = render(app);
    assert!(html.contains("kit-log__marker is-progress"));
    assert!(html.contains("kit-log__marker is-saved"));
    assert_eq!(html.matches("kit-log__marker is-saved").count(), 1);
}

#[test]
fn saved_change_renders_saved_marker_and_no_progress_marker() {
    fn app() -> Element {
        let saved = change("w", "Wire `retry.rs`", vec![]);
        rsx! { ChangeLogTimeline { change_list: vec![saved], current_branch: "main" } }
    }
    let html = render(app);
    assert!(html.contains("kit-log__marker is-saved"));
    assert!(!html.contains("kit-log__marker is-progress"));
}

#[test]
fn trailing_fade_is_visible_only_when_older_history_exists() {
    fn with_older() -> Element {
        rsx! { ChangeLogTimeline { current_branch: "main", has_older: true } }
    }
    fn without_older() -> Element {
        rsx! { ChangeLogTimeline { current_branch: "main" } }
    }
    assert!(render(with_older).contains("kit-log__fade is-visible"));
    assert!(!render(without_older).contains("is-visible"));
}

fn at_rest_changes() -> Vec<Change> {
    let mut writing = change("c1", "Wire `retry.rs` into the sync loop", vec![]);
    writing.in_progress = true;
    let merged = vec![
        change("m1a", "Spread retries with jitter", vec![]),
        change("m1b", "Cap total retry time", vec![]),
        change("m1c", "Log each attempt", vec![]),
    ];
    let mut rows = vec![
        writing,
        change("c2", "Add `cursor.toml` schema", vec![]),
        change("m1", "`sync-cursor/retry`", merged),
    ];
    rows.extend((3..=6).map(|n| change(&format!("c{n}"), "Saved change", vec![])));
    rows
}

fn at_rest_trail() -> Element {
    rsx! {
        ChangeLogTimeline {
            change_list: at_rest_changes(),
            current_branch: "sync-cursor",
            release_tag: Some("rc 3".to_string()),
            has_older: true,
            expanded_merge_ids: Some(vec!["m1".to_string()]),
        }
    }
}

#[test]
fn at_rest_trail_has_seven_top_level_rows_on_one_trunk() {
    let html = render(at_rest_trail);
    assert_eq!(html.matches("data-depth=\"0\"").count(), 7);
    assert!(html.contains("7 changes"));
    assert!(html.contains("rc 3"));
}

#[test]
fn at_rest_merge_is_expanded_with_three_nested_changes_on_a_side_lane() {
    let html = render(at_rest_trail);
    assert!(html.contains("aria-expanded=\"true\""));
    assert_eq!(html.matches("class=\"kit-log__side\"").count(), 1);
    assert_eq!(html.matches("data-depth=\"1\"").count(), 3);
    let side = &html[html.find("class=\"kit-log__side\"").unwrap()..];
    assert!(side.contains("data-change-id=\"m1a\""));
    assert!(side.contains("data-change-id=\"m1c\""));
}

#[test]
fn at_rest_trail_opens_no_detail_and_shows_no_branch_menu() {
    let html = render(at_rest_trail);
    assert!(!html.contains("kit-log__details"));
    assert!(!html.contains("aria-haspopup"));
    assert!(html.contains("sync-cursor"));
}

#[test]
fn at_rest_in_progress_row_is_first_and_not_a_button() {
    let html = render(at_rest_trail);
    let first = &html
        [html.find("data-change-id=\"c1\"").unwrap()..html.find("data-change-id=\"c2\"").unwrap()];
    assert!(first.contains("kit-log__progress-label"));
    assert!(!first.contains("<button"));
}

#[test]
fn folding_the_merge_removes_its_side_lane_and_keeps_the_count() {
    fn folded() -> Element {
        rsx! {
            ChangeLogTimeline {
                change_list: at_rest_changes(),
                current_branch: "sync-cursor",
                expanded_merge_ids: Some(vec![]),
            }
        }
    }
    let html = render(folded);
    assert!(!html.contains("class=\"kit-log__side\""));
    assert!(html.contains("Merged 3 changes"));
}
