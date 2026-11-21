use super::render;
use dioxus::prelude::*;
use rv2_kit::agents::co_editing_presence::{
    AppliedEdit, Block, BlockGroup, BlockKind, CoEditingPresence, MarkSegment, Participant, Pin,
    Position, Presence, Proposal, TableCell, View, announce_proposal, apply_proposal, clamp_offset,
    face_label, followed_after_click, format_spend, group_blocks, mark_segments, new_proposals,
    pin_window_side, presence, proposal_label,
};
use std::collections::BTreeMap;

fn text_block(id: &str, kind: BlockKind, text: &str) -> Block {
    Block {
        id: id.to_string(),
        kind,
        text: text.to_string(),
        image: None,
        cell_grid: Vec::new(),
    }
}

fn proposal(
    id: &str,
    block: &str,
    offset: usize,
    removal: Option<&str>,
    insertion: Option<&str>,
) -> Proposal {
    Proposal {
        id: id.to_string(),
        participant_id: "analyst".to_string(),
        block_id: block.to_string(),
        offset,
        removal: removal.map(str::to_string),
        insertion: insertion.map(str::to_string),
        reason: None,
    }
}

fn caret(id: &str) -> MarkSegment {
    MarkSegment::Caret {
        participant_id: id.to_string(),
    }
}

#[test]
fn removal_is_struck_and_insertion_follows_it() {
    let p = proposal("p", "b", 6, Some("world"), Some("there"));
    let segments = mark_segments("Hello world", &[&p], &[]);
    assert_eq!(
        segments,
        vec![
            MarkSegment::Text("Hello ".to_string()),
            MarkSegment::Removed {
                proposal_id: "p".to_string(),
                participant_id: "analyst".to_string(),
                text: "world".to_string(),
                focusable: true,
            },
            MarkSegment::Inserted {
                proposal_id: "p".to_string(),
                participant_id: "analyst".to_string(),
                text: "there".to_string(),
                focusable: false,
            },
        ]
    );
}

#[test]
fn insertion_alone_is_focusable_and_sits_at_its_offset() {
    let p = proposal("i", "b", 1, None, Some("X"));
    let segments = mark_segments("ab", &[&p], &[]);
    assert_eq!(
        segments[1],
        MarkSegment::Inserted {
            proposal_id: "i".to_string(),
            participant_id: "analyst".to_string(),
            text: "X".to_string(),
            focusable: true,
        }
    );
    assert_eq!(segments.len(), 3);
}

#[test]
fn caret_inside_a_removal_survives_after_the_struck_text() {
    let p = proposal("p", "b", 6, Some("world"), None);
    let segments = mark_segments("Hello world", &[&p], &[(8, caret("analyst"))]);
    assert_eq!(segments.last(), Some(&caret("analyst")));
    assert_eq!(segments.len(), 3);
}

#[test]
fn caret_past_the_end_clamps_to_the_text() {
    let segments = mark_segments("abc", &[], &[(99, caret("analyst"))]);
    assert_eq!(
        segments,
        vec![MarkSegment::Text("abc".to_string()), caret("analyst")]
    );
    assert_eq!(clamp_offset(99, "abc"), 3);
}

#[test]
fn applying_a_replacement_rewrites_text_and_shifts_later_proposals() {
    let blocks = vec![text_block("b", BlockKind::Paragraph, "Hello world")];
    let proposals = vec![
        proposal("p", "b", 6, Some("world"), Some("a")),
        proposal("q", "b", 11, None, Some("!")),
    ];
    let AppliedEdit { blocks, proposals } = apply_proposal(&blocks, &proposals, "p");
    assert_eq!(blocks[0].text, "Hello a");
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].id, "q");
    assert_eq!(proposals[0].offset, 7);
}

#[test]
fn applying_an_unknown_id_changes_nothing() {
    let blocks = vec![text_block("b", BlockKind::Paragraph, "Hello")];
    let proposals = vec![proposal("p", "b", 0, None, Some("x"))];
    let applied = apply_proposal(&blocks, &proposals, "missing");
    assert_eq!(applied.blocks, blocks);
    assert_eq!(applied.proposals, proposals);
}

#[test]
fn consecutive_items_share_one_group() {
    let blocks = vec![
        text_block("a", BlockKind::Item, "one"),
        text_block("b", BlockKind::Item, "two"),
        text_block("c", BlockKind::Paragraph, "para"),
        text_block("d", BlockKind::Item, "three"),
    ];
    let groups = group_blocks(&blocks);
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0], BlockGroup::Items(vec![&blocks[0], &blocks[1]]));
    assert_eq!(groups[1], BlockGroup::Single(&blocks[2]));
    assert_eq!(groups[2], BlockGroup::Items(vec![&blocks[3]]));
}

#[test]
fn proposal_label_names_the_edit() {
    let insert = proposal("i", "b", 0, None, Some("38%"));
    assert_eq!(
        proposal_label("Analyst", &insert),
        "Analyst proposes inserting: 38%"
    );
    let replace = proposal("r", "b", 0, Some("old"), Some("new"));
    assert_eq!(
        proposal_label("Analyst", &replace),
        "Analyst proposes replacing old with new"
    );
    let remove = proposal("d", "b", 0, Some("cut"), None);
    assert_eq!(
        proposal_label("Analyst", &remove),
        "Analyst proposes removing: cut"
    );
}

#[test]
fn only_proposals_unknown_at_mount_are_announced_with_reason() {
    let mut fresh = proposal("p2", "b", 0, None, Some("38%"));
    fresh.reason = Some("Source says so".to_string());
    let all = vec![proposal("p1", "b", 0, None, Some("x")), fresh];
    let known = vec!["p1".to_string()];
    let news = new_proposals(&known, &all);
    assert_eq!(news.len(), 1);
    assert_eq!(
        announce_proposal("Analyst", news[0]),
        "Analyst proposes inserting: 38%. Reason: Source says so"
    );
}

#[test]
fn spend_formats_in_dollars_and_small_amounts_in_words() {
    assert_eq!(format_spend(1.5), "$1.50");
    assert_eq!(format_spend(0.004), "less than one cent");
    assert_eq!(format_spend(0.0), "$0.00");
    assert_eq!(format_spend(f64::NAN), "$0.00");
}

#[test]
fn clicking_a_face_toggles_following() {
    assert_eq!(followed_after_click(None, "a"), Some("a".to_string()));
    assert_eq!(followed_after_click(Some("a"), "a"), None);
    assert_eq!(followed_after_click(Some("b"), "a"), Some("a".to_string()));
}

#[test]
fn absent_participant_has_no_position_and_says_so() {
    assert_eq!(presence(None), Presence::Absent);
    assert_eq!(face_label("Coder", None), "Coder, away");
    let typing = Position {
        block_id: "b".into(),
        is_typing: true,
        ..Position::default()
    };
    assert_eq!(presence(Some(&typing)), Presence::Typing);
    let named = Position {
        block_id: "b".into(),
        activity: Some("Checking".into()),
        ..Position::default()
    };
    assert_eq!(face_label("Analyst", Some(&named)), "Analyst: Checking");
}

#[test]
fn pin_window_opens_left_only_near_the_right_edge() {
    assert_eq!(pin_window_side(0.9), "left");
    assert_eq!(pin_window_side(0.5), "right");
}

fn analyst() -> Participant {
    Participant {
        id: "analyst".to_string(),
        name: "Analyst".to_string(),
        palette_key: Some("analyst".to_string()),
        picture: None,
        wash_key: None,
        spend: Some(0.02),
    }
}

fn coder() -> Participant {
    Participant {
        id: "coder".to_string(),
        name: "Coder".to_string(),
        palette_key: None,
        picture: None,
        wash_key: None,
        spend: None,
    }
}

fn content() -> Vec<Block> {
    vec![
        text_block("t", BlockKind::Title, "Plan"),
        text_block("b2", BlockKind::Paragraph, "Hello world"),
        text_block("i1", BlockKind::Item, "one"),
        text_block("i2", BlockKind::Item, "two"),
    ]
}

fn board() -> Element {
    let positions = BTreeMap::from([(
        "analyst".to_string(),
        Position {
            block_id: "b2".to_string(),
            offset: Some(6),
            activity: Some("Checking sources".to_string()),
            ..Position::default()
        },
    )]);
    let mut edit = proposal("p", "b2", 6, Some("world"), Some("there"));
    edit.reason = Some("Clearer".to_string());
    rsx! {
        CoEditingPresence {
            participants: vec![analyst(), coder()],
            content: content(),
            positions,
            proposals: vec![edit],
        }
    }
}

#[test]
fn board_marks_present_and_absent_participants() {
    let html = render(board);
    assert!(html.contains("data-presence=\"absent\""));
    assert!(html.contains("data-presence=\"idle\""));
    assert!(html.contains("aria-label=\"Analyst: Checking sources\""));
    assert!(html.contains("aria-label=\"Coder, away\""));
    assert!(html.contains("aria-pressed=\"false\""));
    assert!(html.contains("1 pending"));
}

#[test]
fn pointer_sits_in_text_with_a_labelled_flag() {
    let html = render(board);
    assert!(html.contains("data-participant=\"analyst\""));
    assert!(html.contains("Checking sources"));
    assert!(html.contains("$0.02"));
    assert!(html.contains("aria-hidden=\"true\""));
}

#[test]
fn proposal_renders_struck_removal_and_underlined_insertion_in_palette() {
    let html = render(board);
    assert!(html.contains("kit-cocopy__removed"));
    assert!(html.contains("kit-cocopy__inserted"));
    assert!(html.contains("data-proposal=\"p\""));
    assert!(html.contains("tabindex=\"0\""));
    assert!(html.contains("data-palette=\"analyst\""));
}

#[test]
fn document_blocks_use_true_headings_paragraphs_and_lists() {
    let html = render(board);
    assert!(html.contains("<h1"));
    assert!(html.contains("<p class=\"kit-cocopy__para\""));
    assert!(html.contains("kit-cocopy__list"));
    assert!(html.contains("<li"));
}

#[test]
fn proposal_window_is_closed_until_its_mark_is_focused() {
    let html = render(board);
    assert!(!html.contains("kit-cocopy__window"));
    assert!(html.contains("aria-live=\"polite\""));
}

#[test]
fn new_proposal_is_not_announced_when_present_at_mount() {
    let html = render(board);
    let live = html.split("kit-cocopy__sr").nth(1).unwrap_or_default();
    assert!(
        !live
            .split("</div>")
            .next()
            .unwrap_or_default()
            .contains("proposes")
    );
}

fn pinned() -> Element {
    let pin = Pin {
        id: "n".to_string(),
        participant_id: "analyst".to_string(),
        block_id: "b2".to_string(),
        offset: None,
        horizontal_fraction: Some(0.9),
        vertical_offset: Some(0.0),
        body: "Check this figure".to_string(),
        is_open: true,
    };
    rsx! {
        CoEditingPresence {
            participants: vec![analyst()],
            content: content(),
            pins: vec![pin],
            on_pin_reply: move |_: (Pin, String)| {},
            on_pin_close: move |_: Pin| {},
        }
    }
}

#[test]
fn pin_on_block_edge_opens_with_body_close_and_reply() {
    let html = render(pinned);
    assert!(html.contains("left: 90.00%"));
    assert!(html.contains("data-side=\"left\""));
    assert!(html.contains("Check this figure"));
    assert!(html.contains("kit-cocopy__pin-close"));
    assert!(html.contains("kit-cocopy__pin-reply"));
    assert!(html.contains("aria-expanded=\"true\""));
}

fn media() -> Element {
    let table = Block {
        id: "tb".to_string(),
        kind: BlockKind::Table,
        text: String::new(),
        image: None,
        cell_grid: vec![
            vec![TableCell {
                text: "Metric".to_string(),
                badge_key: None,
            }],
            vec![TableCell {
                text: "Ok".to_string(),
                badge_key: Some("ok".to_string()),
            }],
        ],
    };
    let image = Block {
        id: "im".to_string(),
        kind: BlockKind::Image,
        text: "Chart caption".to_string(),
        image: Some(rv2_kit::agents::co_editing_presence::ImageBlock {
            column_fraction: 0.5,
            aspect: 1.5,
            wrap_side: rv2_kit::agents::co_editing_presence::WrapSide::Left,
            is_loading: true,
            source_url: None,
            description: None,
        }),
        cell_grid: Vec::new(),
    };
    rsx! {
        CoEditingPresence {
            participants: vec![analyst()],
            content: vec![image, table],
            views: vec![
                View { id: "draft".to_string(), caption: "Draft".to_string() },
                View { id: "final".to_string(), caption: "Final".to_string() },
            ],
            active_view: "final".to_string(),
            page_height: 320,
        }
    }
}

#[test]
fn table_badges_and_loading_image_render() {
    let html = render(media);
    assert!(html.contains("<table"));
    assert!(html.contains("data-badge=\"ok\""));
    assert!(html.contains("aria-busy=\"true\""));
    assert!(html.contains("kit-cocopy__sweep"));
    assert!(html.contains("Chart caption"));
}

#[test]
fn active_view_is_pressed_and_short_page_is_marked_faded() {
    let html = render(media);
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("data-faded=\"true\""));
    assert!(html.contains("max-height: 320px"));
}
