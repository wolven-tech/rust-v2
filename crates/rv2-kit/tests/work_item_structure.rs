//! Expandable work item card structure at rest: the gallery demo renders open, with the
//! header, body and footer stacked and the subtask, member, status and ring rows in place.

use dioxus::prelude::*;
use rv2_kit::components::expandable_work_item_card::{
    ExpandableWorkItemCard, Member, OpenOn, PrimaryAction, StatusOption, Subtask, Tone,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn task(id: &str, text: &str, done: bool, assignee: Option<&str>) -> Subtask {
    Subtask {
        subtask_id: id.into(),
        subtask_text: text.into(),
        is_done: done,
        assignee_name: assignee.map(Into::into),
    }
}

fn offsite_app() -> Element {
    rsx! {
        ExpandableWorkItemCard {
            item_title: "Plan the offsite",
            subtask_list: vec![
                task("t1", "Draft the agenda", true, Some("Ada")),
                task("t2", "Book the venue", true, Some("Ben")),
                task("t3", "Confirm speakers", false, Some("Ada")),
                task("t4", "Send invites", false, None),
                task("t5", "Arrange catering", false, Some("Chloe")),
            ],
            status_id: "doing",
            status_options: vec![
                StatusOption { option_id: "todo".into(), option_label: "To do".into(), emphasis: Tone::Neutral, option_mark: None },
                StatusOption { option_id: "doing".into(), option_label: "In progress".into(), emphasis: Tone::Caution, option_mark: None },
                StatusOption { option_id: "done".into(), option_label: "Done".into(), emphasis: Tone::Positive, option_mark: None },
            ],
            due_text: Some("Fri".to_string()),
            member_list: vec![
                Member { member_name: "Ada".into(), avatar: None },
                Member { member_name: "Ben".into(), avatar: None },
                Member { member_name: "Chloe".into(), avatar: None },
            ],
            primary_action: Some(PrimaryAction { action_label: "Open thread".into(), on_activate: EventHandler::new(move |_| {}) }),
            footer_hint: Some("Updated 10:42 by Ada".to_string()),
            open_on: OpenOn::Activate,
            default_open: true,
        }
    }
}

/// The HTML of the element whose opening tag contains `marker`, up to its first closing `</li>`.
fn row_containing<'a>(html: &'a str, marker: &str) -> &'a str {
    let at = html.find(marker).expect("row rendered");
    let end = at + html[at..].find("</li>").expect("row closes");
    &html[at..end]
}

#[test]
fn work_item_card_renders_open_with_header_body_and_footer_in_order() {
    let html = render(offsite_app);
    let head = html.find("kit-item__head").expect("header");
    let body = html.find("kit-item__body").expect("body");
    let foot = html.find("kit-item__foot").expect("footer");
    assert!(head < body && body < foot);
    assert!(html.contains("data-open=\"true\""));
}

#[test]
fn work_item_header_reads_ring_title_count_due_avatars_then_status() {
    let html = render(offsite_app);
    let order = [
        "kit-item__ring\"",
        "kit-item__title",
        "kit-item__frac",
        "kit-item__due",
        "kit-item__avatars",
        "kit-item__status\"",
    ]
    .map(|marker| {
        html.find(marker)
            .unwrap_or_else(|| panic!("{marker} rendered"))
    });
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(html.contains("2/5"));
    assert!(html.contains("Due Fri"));
}

/// The tick is a DOM property, which SSR does not serialise; the two ticked rows are checked in the browser.
#[test]
fn work_item_body_lists_five_checkbox_rows_and_no_name_on_unassigned() {
    let html = render(offsite_app);
    assert_eq!(html.matches("class=\"kit-item__subtask\"").count(), 5);
    assert_eq!(html.matches("type=\"checkbox\"").count(), 5);
    let unassigned = row_containing(&html, "Send invites");
    assert!(!unassigned.contains("kit-item__assignee"));
    let assigned = row_containing(&html, "Arrange catering");
    assert!(assigned.contains("Chloe"));
}

#[test]
fn work_item_member_row_shows_three_unpressed_toggles() {
    let html = render(offsite_app);
    assert_eq!(html.matches("class=\"kit-item__member\"").count(), 3);
    assert_eq!(html.matches("aria-pressed=\"false\"").count(), 3);
    assert_eq!(html.matches("aria-pressed=\"true\"").count(), 0);
    assert!(html.contains("kit-item__member-name"));
}

#[test]
fn work_item_status_options_stay_closed_at_rest() {
    let html = render(offsite_app);
    assert!(!html.contains("kit-item__options"));
    assert!(!html.contains("role=\"radio\""));
    assert!(html.contains("kit-item__status-btn"));
    assert!(html.contains("In progress"));
}

#[test]
fn work_item_footer_puts_update_line_before_primary_action() {
    let html = render(offsite_app);
    let line = html.find("Updated 10:42 by Ada").expect("update line");
    let action = html.find("Open thread").expect("primary action");
    assert!(line < action);
}

#[test]
fn work_item_ring_shows_two_solid_segments_of_five() {
    let html = render(offsite_app);
    assert_eq!(html.matches("kit-item__seg is-done").count(), 2);
    assert_eq!(html.matches("class=\"kit-item__seg\"").count(), 3);
}

#[test]
fn work_item_header_groups_count_due_avatars_and_status_in_one_meta_row() {
    let html = render(offsite_app);
    let meta = html.find("kit-item__meta").expect("meta row");
    assert!(html.find("kit-item__title").expect("title") < meta);
    assert!(meta < html.find("kit-item__frac").expect("count"));
    assert!(
        html.find("kit-item__status\"").expect("status")
            < html.rfind("</button>").expect("header close")
    );
    assert!(html.contains("<span class=\"kit-item__due-word\">Due </span>Fri"));
}
