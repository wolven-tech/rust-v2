use dioxus::prelude::*;
use rv2_kit::components::absence_digest::{
    AbsenceDigest, DigestEntry, EntryButton, EntryState, counts, format_elapsed, merge_order,
};
use rv2_kit::components::access_grant_prompt::{
    AccessGrantPrompt, Decision, PermissionWindow, Requester, RiskLevel, allow_label,
    format_remaining, initials, is_expired, offered_windows, remaining_fraction, split_impact,
};
use rv2_kit::components::agent_question_card::{
    AgentQuestionCard, number_key_choice, pager_text, step_row,
};
use rv2_kit::components::consent_request_card::{
    DecisionState, DeclineCause, format_clock, format_countdown, outcome_word, tick,
};
use rv2_kit::components::decision_review_queue::{
    ReviewRisk, WaitState, format_wait, reason_for_digit, wait_state,
};
use rv2_kit::components::grounded_reply::{Block, Inline, parse_answer, take_words, valid_cites};
use rv2_kit::components::inline_drafting_assistant::dictation_message;
use rv2_kit::components::inline_edit_review::{
    Decision as EditDecision, Segment, apply_decisions, pending_count, word_diff,
};
use rv2_kit::components::pending_decision_stack::{
    DecisionItem, RequestKind, RequestStatus, RequestStatus as Status, StatusOverrides,
    blocking_counts, rotate, run_state_word, top_position,
};
use rv2_kit::components::prompt_input_bar::{folder_after_pick, sendable, toggle_link};
use rv2_kit::components::run_control_composer::{
    AgentIdentity, RunState, enter_action, split_at_char,
};
use rv2_kit::components::source_linked_answer::{
    Segment as CitedSegment, citing_text, mark_passage,
};
use rv2_kit::components::trust_level_selector::{
    Capability, RiskClass, TrustLevelSelector, clamp_level, needs_confirmation, step_level,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn destructive_risk_withholds_always_and_falls_back_to_once() {
    let all = [
        PermissionWindow::Once,
        PermissionWindow::Timed,
        PermissionWindow::Always,
    ];
    assert_eq!(
        offered_windows(RiskLevel::Destructive, &all),
        vec![PermissionWindow::Once, PermissionWindow::Timed]
    );
    assert_eq!(
        offered_windows(RiskLevel::Risky, &[PermissionWindow::Once]),
        vec![PermissionWindow::Once]
    );
    assert_eq!(
        offered_windows(RiskLevel::Destructive, &[PermissionWindow::Always]),
        vec![PermissionWindow::Once]
    );
}

#[test]
fn allow_label_names_the_duration() {
    assert_eq!(
        allow_label(PermissionWindow::Timed, 60),
        "Allow for one hour"
    );
    assert_eq!(
        allow_label(PermissionWindow::Timed, 90),
        "Allow for 90 minutes"
    );
    assert_eq!(allow_label(PermissionWindow::Once, 90), "Allow once");
}

#[test]
fn timed_grant_expires_strictly_after_valid_until() {
    let grant = Decision {
        allowed: true,
        window: PermissionWindow::Timed,
        decided_at: 100,
        valid_until: Some(200),
    };
    assert!(!is_expired(&grant, 200));
    assert!(is_expired(&grant, 201));
    assert_eq!(format_remaining(200 - 150), "50s left");
    assert_eq!(format_remaining(4_320), "1h 12m left");
    assert!((remaining_fraction(100, 200, 150) - 0.5).abs() < 1e-9);
}

#[test]
fn impact_rows_split_into_shown_and_hidden() {
    assert_eq!(split_impact(5, 3), (3, 2));
    assert_eq!(split_impact(2, 3), (2, 0));
    assert_eq!(initials("ada lovelace"), "AL");
}

#[test]
fn access_grant_waiting_card_names_requester_and_default_duration() {
    fn app() -> Element {
        rsx! {
            AccessGrantPrompt {
                requester: Requester { name: "Ada".into(), model_name: None, avatar: None },
                requested_action: "delete the staging bucket",
                risk_level: RiskLevel::Destructive,
                allowed_windows: vec![PermissionWindow::Once, PermissionWindow::Always],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("role=\"dialog\""));
    assert!(html.contains("Ada wants to delete the staging bucket"));
    assert!(html.contains("Allow once"));
    assert!(
        !html.contains("Always"),
        "destructive actions withhold the always window"
    );
}

#[test]
fn access_grant_single_window_has_no_duration_group() {
    fn app() -> Element {
        rsx! {
            AccessGrantPrompt {
                requester: Requester { name: "Ada".into(), model_name: None, avatar: None },
                requested_action: "read mail",
                allowed_windows: vec![PermissionWindow::Once],
            }
        }
    }
    assert!(!render(app).contains("kit-agp__choices"));
}

#[test]
fn countdown_ticks_once_per_second_and_freezes_while_paused() {
    assert_eq!(tick(3, false), (2, false));
    assert_eq!(tick(1, false), (0, true));
    assert_eq!(tick(5, true), (5, false));
    assert_eq!(format_countdown(65), "1:05");
    assert_eq!(
        outcome_word(DecisionState::Declined, DeclineCause::Timeout),
        "Expired"
    );
    assert_eq!(format_clock(3_600 + 125), "01:02");
}

#[test]
fn consent_pending_card_renders_group_and_countdown() {
    fn app() -> Element {
        rsx! {
            rv2_kit::components::consent_request_card::ConsentRequestCard {
                request_title: "Send the weekly digest",
                context_line: "To 12 recipients",
                timeout_seconds: 45u32,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("role=\"group\""));
    assert!(html.contains("Send the weekly digest"));
    assert!(html.contains("0:45"));
}

fn item(id: &str, status: RequestStatus, waiting: &[&str]) -> DecisionItem {
    DecisionItem {
        id: id.into(),
        requester: "agent".into(),
        requester_mark: None,
        request_kind: RequestKind::Tool,
        request_summary: format!("summary {id}"),
        request_reason: "reason".into(),
        request_status: status,
        requested_at: None,
        request_payload: None,
        change_lines: Vec::new(),
        touched_paths: Vec::new(),
        prior_steps: Vec::new(),
        waiting_steps: waiting.iter().map(|s| s.to_string()).collect(),
        person_note: None,
    }
}

#[test]
fn blocking_banner_counts_steps_and_items_held_by_open_work() {
    let items = vec![
        item("a", RequestStatus::Open, &["deploy", "notify"]),
        item("b", RequestStatus::Open, &[]),
        item("c", RequestStatus::Approved, &["x"]),
    ];
    let overrides: StatusOverrides = Vec::new();
    assert_eq!(blocking_counts(&items, &overrides), (2, 1));
    assert_eq!(run_state_word(1, true, 0), "Paused");
    assert_eq!(run_state_word(0, true, 0), "Running");
    assert_eq!(run_state_word(0, false, 1), "Stopped");
    assert_eq!(run_state_word(0, false, 0), "All decided");
}

#[test]
fn stack_rotation_wraps_and_top_follows_it() {
    assert_eq!(rotate(0, -1, 3), 2);
    assert_eq!(rotate(2, 1, 3), 0);
    assert_eq!(top_position(2, 3), Some(1));
    assert_eq!(top_position(0, 0), None);
    assert_eq!(Status::Rejected.label(), "Rejected");
}

#[test]
fn review_wait_state_moves_fresh_to_warning_to_overdue() {
    assert_eq!(wait_state(100, 3600), WaitState::Fresh);
    assert_eq!(wait_state(3000, 3600), WaitState::Warning);
    assert_eq!(wait_state(3601, 3600), WaitState::Overdue);
    assert_eq!(format_wait(125), "2m");
    assert_eq!(
        reason_for_digit('2', &["a".into(), "b".into()]),
        Some("b".into())
    );
    assert_eq!(
        reason_for_digit(
            '5',
            &["a".into(), "b".into(), "c".into(), "d".into(), "e".into()]
        ),
        None
    );
    assert_eq!(ReviewRisk::High.as_str(), "high");
}

fn cap(id: &str, risk: RiskClass) -> Capability {
    Capability {
        id: id.into(),
        capability_name: id.into(),
        capability_note: None,
        risk_class: risk,
        capability_mark: None,
    }
}

#[test]
fn needs_confirmation_follows_level_and_pins_table() {
    let risks = [
        RiskClass::Routine,
        RiskClass::Sensitive,
        RiskClass::Irreversible,
    ];
    let pins: Vec<Vec<String>> = vec![vec![], vec!["c".to_string()]];
    for level in 0..=2_u8 {
        for risk in risks {
            for pinned in &pins {
                let capability = cap("c", risk);
                let expected = if pinned.contains(&"c".to_string()) {
                    true
                } else {
                    match level {
                        0 => true,
                        1 => risk != RiskClass::Routine,
                        _ => false,
                    }
                };
                assert_eq!(
                    needs_confirmation(&capability, level, pinned),
                    expected,
                    "level {level} risk {risk:?}"
                );
            }
        }
    }
}

#[test]
fn level_steps_wrap_at_both_ends() {
    assert_eq!(step_level(0, -1), 2);
    assert_eq!(step_level(2, 1), 0);
    assert_eq!(clamp_level(9), 0);
}

#[test]
fn trust_selector_renders_level_radios_and_count_line() {
    fn app() -> Element {
        rsx! {
            TrustLevelSelector {
                capabilities: vec![cap("read", RiskClass::Routine), cap("wire", RiskClass::Irreversible)],
                policy_level: 1u8,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("role=\"radiogroup\""));
    assert!(html.contains("1 run without confirmation, 1 need confirmation"));
}

#[test]
fn answer_grammar_parses_paragraphs_items_emphasis_and_citations() {
    let blocks = parse_answer("Revenue rose **12%** [1,2].\n\nNext step [3]\n- first item [1]");
    assert_eq!(blocks.len(), 3);
    match &blocks[0] {
        Block::Paragraph(runs) => {
            assert!(runs.contains(&Inline::Strong("12%".into())));
            assert!(runs.contains(&Inline::Cite(vec![1, 2])));
        }
        other => panic!("expected paragraph, got {other:?}"),
    }
    assert!(matches!(blocks[2], Block::ListItem(_)));
}

#[test]
fn reveal_takes_whole_words_and_missing_citations_drop() {
    assert_eq!(take_words("one two three", 2), "one two");
    assert_eq!(take_words("one two", 5), "one two");
    assert_eq!(valid_cites(&[1, 4, 0], 3), vec![1]);
}

#[test]
fn grounded_reply_renders_markers_for_existing_sources_only() {
    fn app() -> Element {
        rsx! {
            rv2_kit::components::grounded_reply::GroundedReply {
                heading_text: "Q",
                source_entries: vec![rv2_kit::components::grounded_reply::SourceEntry {
                    entry_id: "a".into(),
                    chip_text: "Report".into(),
                    link_target: None,
                    leading_mark: None,
                    snippet: None,
                }],
                answer_text: "Fact [1] and fake [9].",
                reveal_animated: false,
            }
        }
    }
    let html = render(app);
    assert!(html.contains("Source 1: Report"));
    assert!(!html.contains("Source 9"));
}

#[test]
fn passage_marks_shared_words_and_exact_phrase_wins() {
    let runs = mark_passage(
        "Quarterly revenue grew sharply",
        "revenue grew last quarter",
        None,
    );
    assert!(
        runs.iter()
            .any(|(text, flagged)| *flagged && text.contains("revenue"))
    );
    let exact = mark_passage("alpha beta gamma", "unrelated", Some("beta gamma"));
    assert_eq!(exact.iter().filter(|(_, flagged)| *flagged).count(), 1);
    assert_eq!(exact[1].0, "beta gamma");
}

#[test]
fn citing_text_joins_segments_that_cite_the_reference() {
    let segments = vec![
        CitedSegment {
            text: "First.".into(),
            cited_ids: vec!["b".into()],
            starts_paragraph: false,
        },
        CitedSegment {
            text: "Other.".into(),
            cited_ids: vec!["a".into()],
            starts_paragraph: false,
        },
    ];
    assert_eq!(citing_text(&segments, "b"), "First.");
}

#[test]
fn send_ignores_blank_text_and_links_toggle_both_ways() {
    assert_eq!(sendable("   \n "), None);
    assert_eq!(sendable("  hi  "), Some("hi".into()));
    let linked = toggle_link(&["a".into()], "b");
    assert_eq!(linked, vec!["a".to_string(), "b".to_string()]);
    assert_eq!(toggle_link(&linked, "a"), vec!["b".to_string()]);
    assert_eq!(folder_after_pick(Some("docs"), "docs"), None);
    assert_eq!(folder_after_pick(None, "docs"), Some("docs".into()));
}

#[test]
fn enter_steers_when_running_and_queues_with_alt() {
    assert_eq!(enter_action(true, false), "steer");
    assert_eq!(enter_action(true, true), "queue");
    assert_eq!(enter_action(false, true), "submit");
    assert_eq!(split_at_char("héllo", 2), ("hé", "llo"));
    assert_eq!(split_at_char("ab", 9), ("ab", ""));
}

#[test]
fn run_composer_renders_stopped_controls() {
    fn app() -> Element {
        rsx! {
            rv2_kit::components::run_control_composer::RunControlComposer {
                agent_identity: AgentIdentity { agent_id: "a1".into(), agent_name: "Scout".into() },
                run_state: RunState::Stopped,
                request_text: "find leads",
            }
        }
    }
    let html = render(app);
    assert!(html.contains("Resume"));
    assert!(html.contains("Retry"));
    assert!(html.contains("Dismiss"));
}

#[test]
fn dictation_produces_a_message_only_when_not_blank() {
    assert_eq!(
        dictation_message("  Make it shorter. "),
        Some("Make it shorter.".into())
    );
    assert_eq!(dictation_message("   "), None);
}

#[test]
fn word_diff_keeps_punctuation_and_single_spaces_when_accepting_all() {
    let segments = word_diff(
        "The team will ship the update next week.",
        "The team shall ship the update next week .",
    );
    let accepted = apply_decisions(&segments, &[Some(EditDecision::Accepted); 2]);
    assert_eq!(accepted, "The team shall ship the update next week .");
    let rejected = apply_decisions(&segments, &[Some(EditDecision::Rejected); 2]);
    assert_eq!(rejected, "The team will ship the update next week.");
}

#[test]
fn word_diff_reports_changes_and_decisions_rebuild_text() {
    let segments = word_diff("the quick brown fox", "the swift brown fox");
    let changes = segments
        .iter()
        .filter(|s| matches!(s, Segment::Change { .. }))
        .count();
    assert_eq!(changes, 1);
    let pending = vec![None];
    assert_eq!(pending_count(&pending), 1);
    assert_eq!(
        apply_decisions(&segments, &[Some(EditDecision::Accepted)]),
        "the swift brown fox"
    );
    assert_eq!(
        apply_decisions(&segments, &[Some(EditDecision::Rejected)]),
        "the quick brown fox"
    );
    assert_eq!(apply_decisions(&segments, &pending), "the quick brown fox");
}

fn entry(id: &str, state: EntryState) -> DigestEntry {
    DigestEntry {
        entry_id: id.into(),
        entry_name: id.into(),
        entry_state: state,
        entry_note: None,
        button_text: EntryButton::Default,
    }
}

#[test]
fn digest_orders_by_state_and_keeps_index_when_a_row_resolves() {
    let first = vec![
        entry("done", EntryState::Completed),
        entry("wait", EntryState::AwaitingInput),
        entry("broke", EntryState::Failed),
    ];
    let order = merge_order(&[], &first);
    assert_eq!(order, vec!["wait", "broke", "done"]);
    let resolved = vec![
        entry("done", EntryState::Completed),
        entry("wait", EntryState::Completed),
        entry("broke", EntryState::Failed),
    ];
    assert_eq!(
        merge_order(&order, &resolved),
        vec!["wait", "broke", "done"]
    );
    assert_eq!(counts(&resolved), (0, 1, 2));
    assert_eq!(format_elapsed(3_700), "1h 1m");
}

#[test]
fn digest_renders_busy_row_and_hides_its_button_when_asked() {
    fn app() -> Element {
        rsx! {
            AbsenceDigest {
                away_since: 0i64,
                current_time: 3_600i64,
                digest_entries: vec![
                    DigestEntry {
                        entry_id: "x".into(),
                        entry_name: "Invoice run".into(),
                        entry_state: EntryState::AwaitingInput,
                        entry_note: None,
                        button_text: EntryButton::Hidden,
                    },
                ],
            }
        }
    }
    let html = render(app);
    assert!(html.contains("Since your last visit"));
    assert!(html.contains("Invoice run"));
    assert!(!html.contains("kit-ad__action"));
}

#[test]
fn number_keys_map_to_choices_within_range() {
    assert_eq!(number_key_choice("2", 3), Some(1));
    assert_eq!(number_key_choice("4", 3), None);
    assert_eq!(number_key_choice("0", 3), None);
    assert_eq!(pager_text(0, 3), "1 of 3");
    assert_eq!(step_row(0, 3, -1), 2);
}

#[test]
fn question_card_renders_nothing_without_questions() {
    fn app() -> Element {
        rsx! { AgentQuestionCard { pending_questions: Vec::new() } }
    }
    assert!(!render(app).contains("kit-aqc"));
}
