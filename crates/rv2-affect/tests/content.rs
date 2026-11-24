use dioxus::prelude::*;
use rv2_affect::change_summary::{
    ChangeSummary, MetricChange, MetricDirection, MetricTone, points_down, row_delay_ms,
};
use rv2_affect::code_block::{
    CodeBlock, Lang, TokenKind, language_name, language_of, line_digits, normalise_source,
    split_lines, tokenize,
};
use rv2_affect::comment_thread::{
    CommentThread, ThreadAuthor, ThreadMessage, ago, edit_command, initials as thread_initials,
};
use rv2_affect::decision_inbox::{
    Decision, DecisionInbox, DecisionState, initial_state, next_folded, pending_count,
};
use rv2_affect::email_draft::{
    DraftContact, DraftLink, EmailDraft, Phase, badge_for, email_body, first_name, initials,
    retarget_names,
};
use rv2_affect::message_actions::{
    MessageActionKey, MessageActions, MessageText, announcement, clamp_index, feedback_reason,
    speak_duration_ms, unique_actions as unique_message_actions, version_count,
};
use rv2_affect::notification_stack::{
    Notice, NoticePattern, NotificationStack, PEEK, card_depth, card_scale, card_y, ray_end,
    stack_height,
};
use rv2_affect::progress_tracker::{
    ProgressStep, ProgressTracker, initial_done, newly_done, progress_fraction, toggle_id,
};
use rv2_affect::reaction_bar::{
    ReactionBar, ReactionKey, ReactionState, compact_count, digit_slots, exact_count, short_count,
    toggled, unique_actions,
};
use rv2_affect::smart_recommendation::{
    Recommendation, SmartRecommendation, accept_text, lower_first, signal_level,
};
use rv2_affect::thinking_loader::{
    ThinkingLoader, elapsed_seconds, normalise_phases, phase_index, thought_for, tick_ms, took,
};
use rv2_affect::tips_panel::{Tip, TipsPanel, done_count, tip_keys};
use rv2_affect::tool_call_card::{
    GroupCall, ToolCallCard, ToolCallGroup, ToolCallStatus, format_ms, group_title, highlight_json,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn tokens(text: &str, lang: Lang) -> Vec<(Option<TokenKind>, String)> {
    tokenize(text, Some(lang))
}

fn has(tokens: &[(Option<TokenKind>, String)], kind: TokenKind, text: &str) -> bool {
    tokens.iter().any(|(k, t)| *k == Some(kind) && t == text)
}

#[test]
fn toggling_an_active_reaction_removes_one_and_never_goes_negative() {
    let on = toggled(ReactionState {
        count: 4,
        active: false,
    });
    assert_eq!(
        on,
        ReactionState {
            count: 5,
            active: true
        }
    );
    assert_eq!(
        toggled(on),
        ReactionState {
            count: 4,
            active: false
        }
    );
    assert_eq!(
        toggled(ReactionState {
            count: 0,
            active: true
        }),
        ReactionState {
            count: 0,
            active: false
        }
    );
}

#[test]
fn compact_count_shows_one_decimal_below_ten_thousand() {
    assert_eq!(compact_count(1234), "1.2K");
    assert_eq!(compact_count(12_345), "12K");
    assert_eq!(compact_count(999), "999");
    assert_eq!(compact_count(1_000), "1K");
    assert_eq!(compact_count(1_000_000), "1M");
}

#[test]
fn short_count_rounds_to_whole_thousands_before_compacting() {
    assert_eq!(short_count(999_950), "1M");
    assert_eq!(short_count(10_400), "10K");
}

#[test]
fn exact_count_groups_thousands() {
    assert_eq!(exact_count(7), "7");
    assert_eq!(exact_count(1234), "1,234");
    assert_eq!(exact_count(1_000_000), "1,000,000");
}

#[test]
fn digit_slots_key_each_digit_by_its_position_from_the_end() {
    let slots = digit_slots("1212");
    let keys: Vec<&str> = slots.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(keys, ["4-1", "3-2", "2-1", "1-2"]);
}

#[test]
fn reaction_actions_keep_first_occurrence_only() {
    assert_eq!(
        unique_actions(&[ReactionKey::Share, ReactionKey::Like, ReactionKey::Share]),
        [ReactionKey::Share, ReactionKey::Like]
    );
}

#[test]
fn reaction_bar_renders_pressed_state_and_spoken_count() {
    let html = render(|| {
        rsx! {
            ReactionBar {
                reactions: vec![(ReactionKey::Like, ReactionState { count: 1234, active: true })],
                actions: vec![ReactionKey::Like, ReactionKey::Share],
            }
        }
    });
    assert!(html.contains("class=\"ui-rbc\""));
    assert!(html.contains("data-kind=\"like\""));
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("aria-label=\"Like, 1,234 likes\""));
    assert!(html.contains("data-kind=\"share\""));
    assert!(html.contains("aria-pressed=\"false\""));
    assert!(!html.contains("data-kind=\"comment\""));
}

#[test]
fn repeated_tip_ids_get_their_own_keys() {
    assert_eq!(tip_keys(&["a", "b", "a", "a"]), ["a", "b", "a#1", "a#2"]);
}

#[test]
fn done_count_ignores_keys_of_removed_tips() {
    let keys = tip_keys(&["a", "b"]);
    assert_eq!(done_count(&keys, &["b".to_string(), "gone".to_string()]), 1);
}

#[test]
fn tips_panel_renders_progress_and_primary_action() {
    let html = render(|| {
        rsx! {
            TipsPanel {
                tips: vec![
                    Tip { id: "a".into(), title: "Shortcut".into(), description: rsx! { "Press it" }, action: "Enable".into(), done_label: None, primary: true, on_action: None },
                    Tip { id: "b".into(), title: "Sync".into(), description: rsx! { "Sync often" }, action: "Try".into(), done_label: None, primary: false, on_action: None },
                ],
            }
        }
    });
    assert!(html.contains("0/2 done"));
    assert!(html.contains("class=\"ui-tpc__btn ui-tpc__btn--primary\""));
    assert!(!html.contains("ui-tpc__foot"));
}

#[test]
fn took_formats_seconds_minutes_hours_and_days() {
    assert_eq!(took(5), "5s");
    assert_eq!(took(75), "1m 15s");
    assert_eq!(took(125), "2m 5s");
    assert_eq!(took(3725), "1h 02m");
    assert_eq!(took(90_000), "1d 1h");
    assert_eq!(thought_for(6), "Thought for 6s");
}

#[test]
fn elapsed_seconds_round_and_never_read_zero() {
    assert_eq!(elapsed_seconds(0.0), 1);
    assert_eq!(elapsed_seconds(6_400.0), 6);
    assert_eq!(elapsed_seconds(6_600.0), 7);
}

#[test]
fn tick_interval_defaults_at_zero_and_has_a_floor() {
    assert_eq!(tick_ms(0), 2200);
    assert_eq!(tick_ms(50), 200);
    assert_eq!(tick_ms(3000), 3000);
}

#[test]
fn controlled_phase_wins_over_the_timer_and_is_clamped() {
    assert_eq!(phase_index(Some(1), 3, 4), 1);
    assert_eq!(phase_index(Some(9), 0, 4), 3);
    assert_eq!(phase_index(None, 2, 4), 2);
    assert_eq!(normalise_phases(Vec::new()), ["Thinking"]);
}

#[test]
fn thinking_loader_renders_the_current_phase() {
    let html = render(|| rsx! { ThinkingLoader {} });
    assert!(html.contains("role=\"status\""));
    assert!(html.contains("data-size=\"md\""));
    assert!(html.contains("data-variant=\"pill\""));
    assert!(html.contains("Thinking"));
    assert!(!html.contains("data-done"));
}

#[test]
fn thinking_loader_marks_done_and_shows_the_done_label() {
    let html = render(|| rsx! { ThinkingLoader { done: true } });
    assert!(html.contains("data-done=\"true\""));
    assert!(html.contains("Thought for"));
}

#[test]
fn toggling_an_id_flips_its_membership() {
    let once = toggle_id(&[], "a");
    assert_eq!(once, ["a"]);
    assert!(toggle_id(&once, "a").is_empty());
}

#[test]
fn progress_fraction_of_an_empty_list_is_zero() {
    assert_eq!(progress_fraction(0, 0), 0.0);
    assert_eq!(progress_fraction(1, 4), 0.25);
}

#[test]
fn newly_done_reports_only_finished_arrivals() {
    let steps = vec![
        ProgressStep {
            id: "old".into(),
            label: "Old".into(),
            done: true,
            icon: None,
        },
        ProgressStep {
            id: "new".into(),
            label: "New".into(),
            done: true,
            icon: None,
        },
        ProgressStep {
            id: "later".into(),
            label: "Later".into(),
            done: false,
            icon: None,
        },
    ];
    assert_eq!(newly_done(&["old".to_string()], &steps), ["new"]);
    assert_eq!(initial_done(&steps), ["old", "new"]);
}

#[test]
fn progress_tracker_reports_value_and_completion() {
    let html = render(|| {
        rsx! {
            ProgressTracker {
                steps: vec![
                    ProgressStep { id: "a".into(), label: "Connect".into(), done: true, icon: None },
                    ProgressStep { id: "b".into(), label: "Invite".into(), done: false, icon: None },
                ],
            }
        }
    });
    assert!(html.contains("role=\"progressbar\""));
    assert!(html.contains("aria-valuenow=\"1\""));
    assert!(html.contains("aria-valuemax=\"2\""));
    assert!(html.contains("aria-valuetext=\"1 of 2 done\""));
    assert!(html.contains("role=\"checkbox\""));
}

#[test]
fn card_depth_is_none_once_a_notice_is_leaving() {
    assert_eq!(card_depth(&["a", "b"], "b"), Some(1));
    assert_eq!(card_depth(&["a", "b"], "gone"), None);
}

#[test]
fn stack_height_counts_at_most_two_cards_behind() {
    assert_eq!(stack_height(0, 80.0), 0.0);
    assert_eq!(stack_height(1, 80.0), 80.0);
    assert_eq!(stack_height(5, 80.0), 80.0 + 2.0 * PEEK);
}

#[test]
fn cards_behind_sit_on_the_front_edge_and_cut_taller_cards() {
    assert_eq!(card_y(0, 80.0, 120.0), 0.0);
    assert_eq!(card_y(1, 80.0, 100.0), 80.0 - 80.0 + PEEK);
    assert_eq!(card_y(2, 80.0, 40.0), 80.0 - 40.0 + 2.0 * PEEK);
    assert_eq!(card_scale(0), 1.0);
    assert!((card_scale(2) - 0.9).abs() < 1e-9);
}

#[test]
fn first_ray_points_left_from_the_corner() {
    let (x, y) = ray_end(0);
    assert!((x + 20.0).abs() < 1e-9);
    assert!((y - 6.0).abs() < 1e-9);
}

#[test]
fn notification_stack_marks_the_front_card_and_hides_the_rest() {
    let html = render(|| {
        let notice = |id: &str| Notice {
            id: id.into(),
            badge: Some("Notice".into()),
            title: format!("Title {id}"),
            text: "Body".into(),
            action: None,
            pattern: NoticePattern::Grid,
        };
        let notices = vec![notice("n1"), notice("n2")];
        rsx! { NotificationStack { notices } }
    });
    assert!(html.contains("aria-label=\"Notifications\""));
    assert!(html.contains("data-depth=\"0\""));
    assert!(html.contains("data-depth=\"1\""));
    assert!(html.contains("aria-hidden=\"true\""));
    assert!(html.contains("aria-labelledby=\"n1-title\""));
}

#[test]
fn signal_level_clamps_into_zero_to_three() {
    assert_eq!(signal_level(-1), 0);
    assert_eq!(signal_level(2), 2);
    assert_eq!(signal_level(7), 3);
}

#[test]
fn lower_first_keeps_capitals_of_acronyms() {
    assert_eq!(lower_first("Incremental sync"), "incremental sync");
    assert_eq!(lower_first("AWS backup"), "AWS backup");
    assert_eq!(lower_first("S3 sync"), "S3 sync");
    assert_eq!(accept_text("Incremental sync"), "Accept incremental sync");
}

#[test]
fn smart_recommendation_offers_accept_for_the_first_option() {
    let html = render(|| {
        rsx! {
            SmartRecommendation {
                options: vec![
                    Recommendation { id: "a".into(), label: "Incremental sync".into(), reason: rsx! { "Fewer writes" }, signal: 3 },
                    Recommendation { id: "b".into(), label: "Full sync".into(), reason: rsx! { "Simple" }, signal: 1 },
                ],
            }
        }
    });
    assert!(html.contains("Accept incremental sync"));
    assert!(html.contains("ui-src__btn--primary"));
    assert!(html.contains("data-level=\"3\""));
    assert!(!html.contains("ui-src__pill"));
}

#[test]
fn ago_picks_the_coarsest_unit_that_fits() {
    let now = 10_000_000_000u64;
    assert_eq!(ago(now, now), "just now");
    assert_eq!(ago(now - 120_000, now), "2min ago");
    assert_eq!(ago(now - 3 * 3_600_000, now), "3h ago");
    assert_eq!(ago(now - 3 * 86_400_000, now), "3d ago");
}

#[test]
fn thread_initials_skip_emoji_and_spaces() {
    assert_eq!(thread_initials("🦊 Fox"), "F");
    assert_eq!(thread_initials("Ada Lovelace"), "AL");
    assert_eq!(thread_initials(""), "");
}

#[test]
fn edit_command_needs_a_word_boundary() {
    assert_eq!(edit_command("/edit the title"), Some("the title"));
    assert_eq!(edit_command("/EDIT"), Some(""));
    assert_eq!(edit_command("/editor"), None);
    assert_eq!(edit_command("hello"), None);
}

#[test]
fn comment_thread_renders_message_and_placeholder() {
    let html = render(|| {
        let author = ThreadAuthor {
            name: "Ada Lovelace".into(),
            avatar: None,
            assistant: false,
        };
        let bot = ThreadAuthor {
            name: "Helper".into(),
            avatar: None,
            assistant: true,
        };
        let messages = vec![ThreadMessage {
            id: "m1".into(),
            author: author.clone(),
            created_at: 1,
            text: "/edit tighten the intro".into(),
            status: None,
            details: Vec::new(),
        }];
        rsx! { CommentThread { messages, current_user: author, assistant: bot } }
    });
    assert!(html.contains("1 message"));
    assert!(html.contains("ui-ctc__command"));
    assert!(html.contains("tighten the intro"));
    assert!(html.contains("placeholder=\"Reply, or ask @Helper…\""));
}

fn decision(id: &str, collapsed: bool) -> Decision {
    Decision {
        id: id.into(),
        title: format!("Decision {id}"),
        detail: None,
        primary: "Approve".into(),
        secondary: "Decline".into(),
        collapsed,
    }
}

#[test]
fn collapsed_decisions_start_folded() {
    let decisions = vec![decision("a", false), decision("b", true)];
    let states = initial_state(&decisions);
    assert!(states["a"].open);
    assert!(!states["b"].open);
}

#[test]
fn answered_decisions_leave_the_pending_count() {
    let decisions = vec![decision("a", false), decision("b", true)];
    let mut states = initial_state(&decisions);
    assert_eq!(pending_count(&decisions, &states), 2);
    states.insert(
        "a".into(),
        DecisionState {
            open: true,
            answered: Some("Approve".into()),
            gone: false,
        },
    );
    assert_eq!(pending_count(&decisions, &states), 1);
}

#[test]
fn next_folded_skips_open_and_answered_decisions() {
    let decisions = vec![
        decision("a", false),
        decision("b", true),
        decision("c", true),
    ];
    let mut states = initial_state(&decisions);
    states.insert(
        "b".into(),
        DecisionState {
            open: false,
            answered: Some("Decline".into()),
            gone: false,
        },
    );
    assert_eq!(next_folded(&decisions, &states, "a"), Some("c"));
    assert_eq!(next_folded(&decisions, &states, "c"), None);
}

#[test]
fn decision_inbox_counts_pending_items_and_names_the_empty_state() {
    let html = render(|| {
        rsx! { DecisionInbox { decisions: vec![decision("a", false), decision("b", false)] } }
    });
    assert!(html.contains("2 need your input"));
    let empty = render(|| rsx! { DecisionInbox { decisions: Vec::new() } });
    assert!(empty.contains("All caught up"));
    assert!(empty.contains("data-clear=\"true\""));
}

#[test]
fn decision_inbox_at_rest_gives_each_open_decision_both_answers() {
    let html = render(|| {
        rsx! { DecisionInbox { decisions: vec![decision("a", false), decision("b", false)] } }
    });
    assert_eq!(html.matches("ui-dic__btn ui-dic__btn--primary").count(), 2);
    assert_eq!(html.matches("ui-dic__title--folded").count(), 0);
    let folded = render(|| {
        rsx! { DecisionInbox { decisions: vec![decision("a", false), decision("b", true)] } }
    });
    assert_eq!(folded.matches("ui-dic__title--folded").count(), 1);
}

#[test]
fn first_name_and_initials_follow_the_name() {
    assert_eq!(first_name("  Jonas Weber "), "Jonas");
    assert_eq!(initials("Finance team"), "FT");
    assert_eq!(initials("🦊 Fox"), "F");
}

#[test]
fn retargeting_replaces_full_then_first_names_as_whole_words() {
    let from = DraftContact {
        name: "Jonas Weber".into(),
        email: String::new(),
    };
    let to = DraftContact {
        name: "Lena Hart".into(),
        email: String::new(),
    };
    assert_eq!(
        retarget_names("Hi Jonas Weber, Jonas here", &from, &to),
        "Hi Lena Hart, Lena here"
    );
    assert_eq!(retarget_names("Jonasz wrote", &from, &to), "Jonasz wrote");
}

#[test]
fn email_body_appends_the_link() {
    let link = DraftLink {
        label: "Review quote".into(),
        href: "https://example.com/q".into(),
    };
    assert_eq!(
        email_body("Hi Jonas,", &link),
        "Hi Jonas, Review quote: https://example.com/q"
    );
}

#[test]
fn badge_reflects_the_phase() {
    assert_eq!(badge_for(Phase::Sent), ("Sent", "ok"));
    assert_eq!(badge_for(Phase::Discarded), ("Discarded", "muted"));
    assert_eq!(badge_for(Phase::Draft), ("Ready for review", "ok"));
}

#[test]
fn email_draft_renders_the_recipient_and_the_send_action() {
    let html = render(|| rsx! { EmailDraft {} });
    assert!(html.contains(">jonas@northwind.io</span>"));
    assert!(html.contains("Hi Jonas"));
    assert!(html.contains("Send email"));
    assert!(html.contains("Renewal-quote.pdf"));
}

#[test]
fn message_action_helpers_bound_their_inputs() {
    assert_eq!(
        unique_message_actions(&[MessageActionKey::Speak, MessageActionKey::Speak]),
        [MessageActionKey::Speak]
    );
    assert_eq!(version_count(0), 1);
    assert_eq!(clamp_index(5, 3), 2);
    assert_eq!(speak_duration_ms(None, None), 2600);
    assert_eq!(
        speak_duration_ms(Some("word ".repeat(20).as_str()), None),
        7600
    );
    assert_eq!(speak_duration_ms(Some("text"), Some(900)), 900);
}

#[test]
fn feedback_reason_sends_typed_text_for_other() {
    assert_eq!(feedback_reason("Other", "  slow  "), "slow");
    assert_eq!(feedback_reason("Other", ""), "Other");
    assert_eq!(feedback_reason("Too long", "ignored"), "Too long");
}

#[test]
fn repeated_announcements_get_a_trailing_space() {
    assert_eq!(announcement("Copied", "Copied"), "Copied ");
    assert_eq!(announcement("Copied", "Regenerating"), "Regenerating");
}

#[test]
fn message_actions_render_the_default_bar_and_feedback_panel() {
    let html = render(|| rsx! { MessageActions { text: "Hello".to_string(), versions: 2 } });
    assert!(html.contains("data-quiet"));
    assert!(html.contains("ui-mac__btn--copy"));
    assert!(html.contains("ui-mac__btn--speak"));
    assert!(html.contains("aria-label=\"Bad response\""));
    assert!(html.contains("ui-mac__panel"));
    assert!(html.contains("ui-mac__versions"));
}

#[test]
fn message_actions_start_with_the_feedback_panel_open_when_asked() {
    let open =
        render(|| rsx! { MessageActions { text: "Hello".to_string(), feedback_open: true } });
    assert!(open.contains("data-open=\"\""));
    let closed = render(|| rsx! { MessageActions { text: "Hello".to_string() } });
    assert!(!closed.contains("data-open"));
}

#[test]
fn message_actions_drop_the_feedback_row_when_not_listed() {
    let html = render(|| rsx! { MessageActions { actions: vec![MessageActionKey::Copy] } });
    assert!(html.contains("ui-mac__btn--copy"));
    assert!(!html.contains("ui-mac__fold"));
}

#[test]
fn message_text_wraps_the_version_in_the_crossfade_block() {
    let html = render(|| rsx! { MessageText { version: "2".to_string(), p { "Version two" } } });
    assert!(html.contains("ui-mac-text__inner"));
    assert!(html.contains("Version two"));
}

#[test]
fn language_aliases_resolve_case_insensitively() {
    assert_eq!(language_of("TS"), Some(Lang::Js));
    assert_eq!(language_of("shell"), Some(Lang::Bash));
    assert_eq!(language_of("rust"), None);
    assert_eq!(language_name("ts"), "TypeScript");
    assert_eq!(language_name("rust"), "rust");
}

#[test]
fn source_is_normalised_and_loses_one_trailing_newline() {
    assert_eq!(normalise_source("a\r\nb\r\n"), "a\nb");
    assert_eq!(normalise_source("x\n\n"), "x\n");
    assert_eq!(line_digits(0), 1);
    assert_eq!(line_digits(10), 2);
}

#[test]
fn javascript_colors_keywords_strings_comments_and_calls() {
    let t = tokens("const x = 'a' // hi\nrun(1)", Lang::Js);
    assert!(has(&t, TokenKind::Keyword, "const"));
    assert!(has(&t, TokenKind::Str, "'a'"));
    assert!(has(&t, TokenKind::Comment, "// hi"));
    assert!(has(&t, TokenKind::Function, "run"));
    assert!(has(&t, TokenKind::Number, "1"));
}

#[test]
fn javascript_does_not_color_inside_a_longer_word() {
    let t = tokens("format(x)", Lang::Js);
    assert!(!has(&t, TokenKind::Keyword, "for"));
    assert!(has(&t, TokenKind::Function, "format"));
}

#[test]
fn json_colors_keys_separately_from_string_values() {
    let t = tokens(r#"{"a": 1, "b": true, "c": "x"}"#, Lang::Json);
    assert!(has(&t, TokenKind::Key, "\"a\""));
    assert!(has(&t, TokenKind::Number, "1"));
    assert!(has(&t, TokenKind::Key, "\"b\""));
    assert!(has(&t, TokenKind::Literal, "true"));
    assert!(has(&t, TokenKind::Str, "\"x\""));
}

#[test]
fn python_colors_keywords_literals_and_comments() {
    let t = tokens("def f(x): return None  # c", Lang::Python);
    assert!(has(&t, TokenKind::Keyword, "def"));
    assert!(has(&t, TokenKind::Function, "f"));
    assert!(has(&t, TokenKind::Literal, "None"));
    assert!(has(&t, TokenKind::Comment, "# c"));
}

#[test]
fn bash_colors_commands_variables_and_comments() {
    let t = tokens("echo $HOME | grep x # c", Lang::Bash);
    assert!(has(&t, TokenKind::Function, "echo"));
    assert!(has(&t, TokenKind::Variable, "$HOME"));
    assert!(has(&t, TokenKind::Function, "grep"));
    assert!(has(&t, TokenKind::Comment, "# c"));
    assert!(!has(&t, TokenKind::Function, "x"));
}

#[test]
fn unknown_language_is_one_plain_token() {
    assert_eq!(tokenize("let x", None), vec![(None, "let x".to_string())]);
}

#[test]
fn a_multi_line_comment_keeps_its_color_on_every_line() {
    let lines = split_lines(&tokens("/* a\nb */", Lang::Js));
    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines[0],
        vec![(Some(TokenKind::Comment), "/* a".to_string())]
    );
    assert_eq!(
        lines[1],
        vec![(Some(TokenKind::Comment), "b */".to_string())]
    );
}

#[test]
fn code_block_marks_highlighted_lines_and_names_the_language() {
    let html = render(|| {
        rsx! { CodeBlock { code: "const a = 1\nb()".to_string(), language: "ts", show_line_numbers: true, highlight_lines: vec![2] } }
    });
    assert!(html.contains("TypeScript"));
    assert!(html.contains("data-marked=\"true\""));
    assert!(html.contains("ui-codec__ln"));
    assert!(html.contains("aria-label=\"Copy code\""));
    assert!(html.contains("ui-codec__t-kw"));
}

#[test]
fn durations_switch_units_at_their_boundaries() {
    assert_eq!(format_ms(400, true), "0.4s");
    assert_eq!(format_ms(400, false), "400ms");
    assert_eq!(format_ms(12_300, true), "12.3s");
    assert_eq!(format_ms(125_000, true), "2m 05s");
    assert_eq!(format_ms(3 * 3_600_000 + 12 * 60_000, true), "3h 12m");
    assert_eq!(format_ms(2 * 86_400_000 + 4 * 3_600_000, true), "2d 4h");
}

#[test]
fn group_title_counts_tools_unless_labelled() {
    assert_eq!(group_title(3, false, None, "Using tools…"), "Used 3 tools");
    assert_eq!(group_title(1, false, None, "Using tools…"), "Used 1 tool");
    assert_eq!(group_title(3, true, None, "Using tools…"), "Using tools…");
    assert_eq!(
        group_title(3, false, Some("Checked"), "Using tools…"),
        "Checked"
    );
}

#[test]
fn json_highlighting_classifies_keys_strings_numbers_and_literals() {
    let segments = highlight_json(r#"{"a": [1, true, "x"]}"#);
    assert!(segments.contains(&(Some("key"), "\"a\"".to_string())));
    assert!(segments.contains(&(Some("num"), "1".to_string())));
    assert!(segments.contains(&(Some("lit"), "true".to_string())));
    assert!(segments.contains(&(Some("str"), "\"x\"".to_string())));
}

#[test]
fn failed_card_shows_its_error_and_the_failed_label() {
    let html = render(|| {
        rsx! { ToolCallCard { name: "search_web", status: ToolCallStatus::Error, error: "boom" } }
    });
    assert!(html.contains("data-status=\"error\""));
    assert!(html.contains("ui-tclc__error-text"));
    assert!(html.contains("boom"));
    assert!(html.contains("Failed"));
}

#[test]
fn successful_card_shows_its_result_block() {
    let html = render(|| {
        rsx! { ToolCallCard { name: "lookup", status: ToolCallStatus::Success, result: "{{\"ok\": true}}", duration: 400u64 } }
    });
    assert!(html.contains("data-status=\"success\""));
    assert!(html.contains("ui-tclc__code"));
    assert!(html.contains("0.4s") || html.contains("400ms"));
    assert!(html.contains("ui-tclc__t-key"));
}

#[test]
fn group_header_summarises_calls_and_failures() {
    let html = render(|| {
        let calls = vec![
            GroupCall {
                name: "a".into(),
                icon: None,
                status: ToolCallStatus::Success,
                duration: Some(100),
                started_at: None,
            },
            GroupCall {
                name: "b".into(),
                icon: None,
                status: ToolCallStatus::Success,
                duration: Some(200),
                started_at: None,
            },
            GroupCall {
                name: "c".into(),
                icon: None,
                status: ToolCallStatus::Error,
                duration: Some(50),
                started_at: None,
            },
        ];
        rsx! { ToolCallGroup { calls, span { "child" } } }
    });
    assert!(html.contains("Used 3 tools"));
    assert!(html.contains("· 1 failed"));
    assert!(html.contains("child"));
}

#[test]
fn bad_metrics_point_down_unless_told_otherwise() {
    assert!(points_down(MetricTone::Bad, None));
    assert!(!points_down(MetricTone::Good, None));
    assert!(points_down(MetricTone::Good, Some(MetricDirection::Down)));
    assert!(!points_down(MetricTone::Neutral, None));
}

#[test]
fn row_stagger_stops_after_twelve_rows() {
    assert_eq!(row_delay_ms(0), 120);
    assert_eq!(row_delay_ms(20), 120 + 12 * 70);
}

#[test]
fn change_summary_renders_each_row_with_its_tone() {
    let html = render(|| {
        let changes = vec![MetricChange {
            label: "Revenue".into(),
            change: "Up 6% to 18.2K".into(),
            tone: MetricTone::Good,
            direction: None,
        }];
        rsx! { ChangeSummary { changes, period: "This week".to_string() } }
    });
    assert!(html.contains("class=\"ui-csc__row\""));
    assert!(html.contains("data-tone=\"good\""));
    assert!(html.contains("Up 6% to 18.2K"));
    assert!(html.contains("This week"));
}
