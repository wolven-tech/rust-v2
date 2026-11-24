use dioxus::prelude::*;
use rv2_kit::components::inline_edit_review::{
    Decision, InlineEditReview, Segment, change_label, next_pending, popover_hangs_from_end,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn gallery_segments() -> Vec<Segment> {
    vec![
        Segment::Same("Hello team, the revised console is now ".to_string()),
        Segment::Change {
            removed: "available".to_string(),
            inserted: "live".to_string(),
        },
        Segment::Same(" for every account. It loads ".to_string()),
        Segment::Change {
            removed: "a lot quicker than the old one".to_string(),
            inserted: "faster than before".to_string(),
        },
        Segment::Same(", and reports can be exported as CSV ".to_string()),
        Segment::Change {
            removed: "with a single".to_string(),
            inserted: "in one".to_string(),
        },
        Segment::Same(" step.".to_string()),
    ]
}

fn gallery() -> Element {
    rsx! {
        InlineEditReview {
            segments: gallery_segments(),
            caption_text: "Proposed edit · shorten the note",
        }
    }
}

#[test]
fn toolbar_at_rest_reads_badge_pending_words_arrows_then_bulk_buttons() {
    let html = render(gallery);
    let badge = html.find("kit-ier__badge").expect("count badge");
    let words = html.find("pending changes").expect("pending words");
    let previous = html.find("Previous change").expect("previous arrow");
    let next = html.find("Next change").expect("next arrow");
    let reject = html.find("kit-ier__reject-all").expect("reject all");
    let accept = html.find("kit-ier__accept-all").expect("accept all");
    assert!(badge < words && words < previous && previous < next);
    assert!(next < reject && reject < accept);
    assert!(html.contains("kit-ier__badge\">3<"));
    assert!(!html.contains("kit-ier__undo"));
}

#[test]
fn paragraph_at_rest_shows_three_changes_with_both_word_sets_and_no_popover() {
    let html = render(gallery);
    assert_eq!(
        count(&html, "class=\"kit-ier__change\" data-state=\"pending\""),
        3
    );
    assert_eq!(count(&html, "kit-ier__removed"), 3);
    assert_eq!(count(&html, "kit-ier__inserted"), 3);
    assert_eq!(count(&html, "kit-ier__pop"), 0);
}

#[test]
fn next_arrow_from_nothing_highlights_the_first_pending_change_and_wraps() {
    let open = vec![None, None, None];
    assert_eq!(next_pending(&open, None, 1), Some(0));
    assert_eq!(next_pending(&open, Some(2), 1), Some(0));
    assert_eq!(next_pending(&open, None, -1), Some(2));
    assert_eq!(next_pending(&open, Some(0), -1), Some(2));
}

#[test]
fn arrows_skip_changes_that_are_already_decided() {
    let decided = vec![None, Some(Decision::Accepted), None];
    assert_eq!(next_pending(&decided, Some(0), 1), Some(2));
    assert_eq!(next_pending(&decided, Some(2), 1), Some(0));
    assert_eq!(next_pending(&[Some(Decision::Rejected)], None, 1), None);
}

#[test]
fn first_gallery_change_is_named_with_its_position_and_both_word_sets() {
    assert_eq!(
        change_label(0, 3, "available", "live"),
        "Change 1 of 3: replaces “available” with “live”. Y accepts, N rejects."
    );
    let html = render(gallery);
    assert!(html.contains(&change_label(0, 3, "available", "live")));
}

#[test]
fn pure_deletion_reads_removes_and_pure_insertion_reads_adds() {
    assert_eq!(
        change_label(1, 3, "old words", ""),
        "Change 2 of 3: removes “old words”. Y accepts, N rejects."
    );
    assert_eq!(
        change_label(2, 3, "", "new"),
        "Change 3 of 3: adds “new”. Y accepts, N rejects."
    );
}

#[test]
fn only_the_accessible_names_show_a_position_of_three() {
    let html = render(gallery);
    assert_eq!(count(&html, " of 3"), 3);
}

#[test]
fn popover_hangs_from_the_end_only_for_changes_past_the_midpoint() {
    let segments = gallery_segments();
    assert!(!popover_hangs_from_end(&segments, 0));
    assert!(popover_hangs_from_end(&segments, 2));
}
