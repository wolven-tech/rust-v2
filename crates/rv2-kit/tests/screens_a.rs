//! Lane screens-a: SSR and pure-logic tests for the assistant and workspace screens.

use dioxus::prelude::*;
use rv2_kit::components::prompt_input_bar::Submission;
use rv2_kit::screens::ai_spend_chat_planner::{
    FORECAST_MONTHS, Lever, LeverKind, forecast, lever_saving, plan_snapshot, planned, range_step,
    saving_words, total_savings,
};
use rv2_kit::screens::ai_workspace_home::{
    ActivityDay, activity_level, activity_total, change_words, middle_truncate, move_index,
    usage_ratio, usage_words,
};
use rv2_kit::screens::assistant_chat_progress_panel::{
    CallState, Step, StepState, UsageRecord, bar_state_text, card_status, folded_count,
    usage_present,
};
use rv2_kit::screens::assistant_chat_sidebars::{
    ApprovalMode, ChatItem, Project, clamp_work_width, group_chats, policy_for, search_summary,
    visible_chats,
};
use rv2_kit::screens::assistant_naming_step::{greeting, initial_names, names_line, proposed_name};
use rv2_kit::screens::assistant_persona_step::{
    ReplyLength, VoiceTone, can_continue as persona_can_continue, default_handover_email,
    handover_email_valid, handover_line, sample_reply,
};
use rv2_kit::screens::assistant_ready_step::{
    also_answers_to, handover_summary, learns_from, status_word, voice_summary,
};
use rv2_kit::screens::assistant_start_page::{
    OutputKind as StartOutputKind, active_after_close, can_send, send_text, tab_after_move,
};
use rv2_kit::screens::knowledge_source_step::{
    PreviewSource, SourceStatus, clean_domain, count_line, split_entries, step_state,
};
use rv2_kit::screens::workspace_chat::{empty_section_line, tab_title};
use rv2_kit::screens::workspace_setup_checklist::{
    progress_line, slugify, step_valid, valid_invites,
};

/// Renders a component to HTML without a browser.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn domains(list: &[&str]) -> Vec<String> {
    list.iter().map(|d| d.to_string()).collect()
}

#[test]
fn clean_domain_keeps_only_the_bare_host() {
    assert_eq!(
        clean_domain("https://www.Harbor-Lane.com/docs?x=1"),
        Some("harbor-lane.com".into())
    );
    assert_eq!(
        clean_domain("  help.example.org/a/b "),
        Some("help.example.org".into())
    );
    assert_eq!(clean_domain("localhost"), None);
    assert_eq!(clean_domain("bad domain.com"), None);
    assert_eq!(clean_domain("a..com"), None);
    assert_eq!(clean_domain("site.c1"), None);
}

#[test]
fn pasted_lists_split_on_commas_and_line_breaks() {
    assert_eq!(
        split_entries("a.com, b.com\nc.com"),
        domains(&["a.com", "b.com", "c.com"])
    );
    assert!(split_entries(" , \n").is_empty());
}

#[test]
fn knowledge_count_line_and_steps() {
    assert_eq!(count_line(0), "Add at least one website to continue.");
    assert_eq!(count_line(1), "1 website added.");
    assert_eq!(count_line(2), "2 websites added.");
    assert_eq!(step_state(1, 3), "done");
    assert_eq!(step_state(3, 3), "current");
    assert_eq!(step_state(4, 3), "upcoming");
    assert_eq!(SourceStatus::Learning.label(), "Learning");
    let _row = PreviewSource {
        domain: "a.com".into(),
        status: SourceStatus::Queued,
    };
}

#[test]
fn proposed_name_title_cases_the_first_label() {
    assert_eq!(proposed_name("harbor-lane.com"), "Harbor lane");
    assert_eq!(proposed_name(""), "");
    assert_eq!(
        initial_names("harbor-lane.com", &[]),
        vec!["Harbor lane".to_string()]
    );
    assert_eq!(
        initial_names("harbor-lane.com", &domains(&["Harbor Co", "HC"])),
        domains(&["Harbor Co", "HC"])
    );
    assert_eq!(
        greeting("Harbor lane"),
        "Hi, I'm Harbor lane. How can I help?"
    );
    assert!(names_line(&domains(&["A", "B", "C"])).contains("Short forms: B, C."));
}

#[test]
fn handover_address_needs_a_dotted_domain() {
    assert!(!handover_email_valid("support@harbor-lane"));
    assert!(handover_email_valid("support@harbor-lane.com"));
    assert!(!handover_email_valid("support@a..com"));
    assert_eq!(
        default_handover_email("harbor-lane.com"),
        "support@harbor-lane.com"
    );
    assert!(!persona_can_continue(
        "Assistant",
        true,
        "support@harbor-lane"
    ));
    assert!(persona_can_continue("Assistant", false, ""));
    assert!(!persona_can_continue("  ", false, ""));
}

#[test]
fn persona_reply_follows_tone_and_length() {
    let formal = sample_reply(VoiceTone::Formal, ReplyLength::Short);
    let friendly = sample_reply(VoiceTone::Friendly, ReplyLength::Detailed);
    assert_ne!(formal, friendly);
    assert!(friendly.len() > sample_reply(VoiceTone::Friendly, ReplyLength::Short).len());
    assert_eq!(
        handover_line(false, "x@y.com"),
        "Nobody takes over; the assistant answers itself."
    );
    assert_eq!(
        handover_line(true, "support@harbor-lane.com"),
        "Hands over to support@harbor-lane.com when unsure."
    );
}

#[test]
fn ready_summary_rows_read_plainly() {
    assert_eq!(
        learns_from(&domains(&[
            "harbor-lane.com",
            "docs.harbor-lane.com",
            "help.x.org"
        ])),
        "harbor-lane.com and 2 more"
    );
    assert_eq!(learns_from(&[]), "No websites yet");
    assert_eq!(
        voice_summary(VoiceTone::Friendly, ReplyLength::Balanced),
        "Friendly, balanced replies"
    );
    assert_eq!(handover_summary("  "), "nobody; it answers itself");
    assert_eq!(also_answers_to(&domains(&["HL", " "])), Some("HL".into()));
    assert_eq!(also_answers_to(&[]), None);
    assert_eq!(status_word(false), "Draft");
    assert_eq!(status_word(true), "Live");
}

#[test]
fn start_page_send_and_tab_rules() {
    assert_eq!(send_text("  hi \n"), Some("hi".into()));
    assert!(!can_send("   ", false));
    assert!(!can_send("hi", true), "sending waits while loading");
    assert_eq!(tab_after_move(2, 3, true), 0);
    assert_eq!(tab_after_move(0, 3, false), 2);
    assert_eq!(active_after_close(1, 3), 1);
    assert_eq!(active_after_close(2, 3), 1);
    assert_eq!(active_after_close(0, 1), 0);
    let _kind = StartOutputKind::Folder;
}

#[test]
fn activity_grid_totals_and_keyboard_moves() {
    let days: Vec<ActivityDay> = (0..21)
        .map(|i| ActivityDay {
            date: format!("2026-11-{:02}", i + 1),
            chat_count: (i % 4) as u32,
        })
        .collect();
    assert_eq!(
        activity_total(&days),
        (0..21).map(|i| (i % 4) as u32).sum::<u32>()
    );
    assert_eq!(activity_level(0, 4), 0);
    assert_eq!(activity_level(4, 4), 4);
    assert_eq!(move_index(0, 21, &Key::ArrowDown), 1);
    assert_eq!(move_index(0, 21, &Key::ArrowRight), 7);
    assert_eq!(move_index(20, 21, &Key::ArrowRight), 20);
    assert_eq!(move_index(9, 21, &Key::ArrowLeft), 2);
    assert_eq!(move_index(9, 21, &Key::End), 20);
    assert_eq!(move_index(9, 21, &Key::Home), 0);
}

#[test]
fn usage_and_changes_are_stated_in_words() {
    assert_eq!(
        usage_words(2400, 2000),
        "Limit reached: 2,400 of 2,000 units used."
    );
    assert_eq!(usage_words(1200, 2000), "1,200 of 2,000 units used.");
    assert_eq!(usage_ratio(3000, 2000), 1.0);
    assert_eq!(change_words(12, "last month"), "up 12 on last month");
    assert_eq!(change_words(-3, "last month"), "down 3 on last month");
    assert_eq!(change_words(0, "last month"), "same as last month");
}

#[test]
fn long_file_names_keep_their_extension() {
    let name = "quarterly-board-deck-final-version-seven.pptx";
    let short = middle_truncate(name, 24);
    assert!(short.chars().count() <= 24, "{short}");
    assert!(short.ends_with(".pptx"), "{short}");
    assert!(short.contains('…'));
    assert_eq!(middle_truncate("short.md", 24), "short.md");
}

#[test]
fn tool_call_states_map_to_card_status() {
    assert_eq!(
        card_status(CallState::Failed),
        rv2_affect::tool_call_card::ToolCallStatus::Error
    );
    assert_eq!(
        card_status(CallState::Done),
        rv2_affect::tool_call_card::ToolCallStatus::Success
    );
}

#[test]
fn earlier_finished_steps_fold_and_the_bar_reads_its_state() {
    let step = |n: usize, state: StepState| Step {
        id: format!("s{n}"),
        entry_label: format!("Step {n}"),
        step_state: state,
    };
    let steps = vec![
        step(1, StepState::Done),
        step(2, StepState::Done),
        step(3, StepState::Done),
        step(4, StepState::Running),
    ];
    assert_eq!(folded_count(&steps, 2), 2);
    assert_eq!(folded_count(&steps[..2], 2), 0);
    assert_eq!(
        bar_state_text(false, 3),
        "Progress and outputs, 3 outputs, closed"
    );
    assert!(!usage_present(&UsageRecord::default()));
    assert!(usage_present(&UsageRecord {
        web_search_pages: Some(4),
        ..Default::default()
    }));
}

#[test]
fn chat_filters_group_under_projects() {
    let chat = |id: &str, project: Option<&str>, pinned: bool| ChatItem {
        id: id.into(),
        entry_title: id.into(),
        project_id: project.map(str::to_string),
        is_pinned: pinned,
        updated_at: "09:00".into(),
    };
    let chats = vec![
        chat("a", Some("p1"), false),
        chat("b", Some("p2"), false),
        chat("c", None, false),
    ];
    let refs: Vec<&ChatItem> = chats.iter().collect();
    assert_eq!(visible_chats(&chats, Some("p1")).len(), 1);
    assert_eq!(visible_chats(&chats, None).len(), 3);
    let projects = vec![
        Project {
            id: "p1".into(),
            entry_name: "Sales".into(),
        },
        Project {
            id: "p2".into(),
            entry_name: "Ops".into(),
        },
    ];
    let groups = group_chats(&refs, &projects);
    let headings: Vec<&str> = groups.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(headings, vec!["Sales", "Ops", "No project"]);
    assert_eq!(search_summary(3), "Searched the web, 3 searches");
    assert_eq!(search_summary(1), "Searched the web, 1 search");
}

#[test]
fn work_area_width_and_approval_mapping() {
    assert_eq!(clamp_work_width(100.0, 1000.0), 280.0);
    assert_eq!(clamp_work_width(900.0, 1000.0), 600.0);
    assert_eq!(
        policy_for(ApprovalMode::Approve),
        rv2_kit::components::prompt_input_bar::ApprovalPolicy::ConfirmEach
    );
    assert_eq!(
        policy_for(ApprovalMode::Auto),
        rv2_kit::components::prompt_input_bar::ApprovalPolicy::AutoSafe
    );
}

#[test]
fn workspace_chat_tab_titles_and_empty_lines() {
    assert_eq!(tab_title("brief"), "Brief");
    assert_eq!(tab_title("file:plan.md"), "plan.md");
    assert!(empty_section_line("files").contains("appear"));
}

#[test]
fn spend_levers_save_their_share_and_the_forecast_follows() {
    let range = Lever {
        id: "batch".into(),
        entry_label: "Batch jobs".into(),
        detail_text: "Run overnight".into(),
        lever_kind: LeverKind::Range,
        monthly_base_dollars: 1200.0,
        max_saving_share: 0.5,
        max_percent: 100.0,
        entry_value: 20.0,
    };
    let switch = Lever {
        id: "cache".into(),
        entry_label: "Cache".into(),
        lever_kind: LeverKind::Switch,
        max_percent: 0.0,
        entry_value: 1.0,
        monthly_base_dollars: 800.0,
        max_saving_share: 0.25,
        ..range.clone()
    };
    let levers = vec![range.clone(), switch.clone()];
    assert!((lever_saving(&range, 20.0) - 1200.0 * 0.5 * 0.2).abs() < 1e-9);
    assert!((lever_saving(&switch, 0.0)).abs() < 1e-9);
    let values = [20.0, 1.0];
    let total = total_savings(&levers, &values);
    let current = 4000.0;
    let baseline = forecast(current, 0.04, FORECAST_MONTHS);
    assert_eq!(baseline.len(), FORECAST_MONTHS);
    let path = planned(&baseline, current, total);
    assert!(path[0] < baseline[0]);
    let off = planned(&baseline, current, total_savings(&levers, &[0.0, 0.0]));
    assert_eq!(off, baseline);
    assert_eq!(range_step(100.0), 5.0);
    assert_eq!(
        saving_words("Batch jobs", 120.0),
        "Batch jobs: 120 dollars a month saved"
    );
    assert!(plan_snapshot(&levers, &values, current).contains("Batch jobs at 20%"));
}

#[test]
fn setup_checklist_rules() {
    assert_eq!(slugify("Harbor Lane & Co."), "harbor-lane-co");
    assert_eq!(progress_line(2), "2 of 4 done");
    assert!(!step_valid(1, "  "));
    assert!(step_valid(2, ""));
    let (valid, rejected) = valid_invites("ana@harbor-lane.com, bob@harbor");
    assert_eq!(valid, domains(&["ana@harbor-lane.com"]));
    assert_eq!(rejected, domains(&["bob@harbor"]));
}

fn knowledge_app() -> Element {
    rsx! {
        rv2_kit::screens::knowledge_source_step::KnowledgeSourceStep {
            default_domains: domains(&["harbor-lane.com"]),
            on_continue: move |_: Vec<String>| {},
        }
    }
}

#[test]
fn knowledge_step_renders_the_first_domain_chip_and_live_line() {
    let html = render(knowledge_app);
    assert!(html.contains("harbor-lane.com"), "{html}");
    assert!(html.contains("1 website added."), "{html}");
    assert!(
        html.contains("aria-hidden=\"true\""),
        "preview is hidden from assistive technology"
    );
    assert!(!html.contains("Back"), "no back control without on_back");
}

fn naming_app() -> Element {
    rsx! {
        rv2_kit::screens::assistant_naming_step::AssistantNamingStep {
            website_domain: "harbor-lane.com",
            on_continue: move |_: Vec<String>| {},
        }
    }
}

#[test]
fn naming_step_proposes_the_name_before_any_input() {
    let html = render(naming_app);
    assert!(html.contains("Harbor lane"), "{html}");
    assert!(html.contains("Primary: Harbor lane."), "{html}");
}

fn persona_app() -> Element {
    rsx! {
        rv2_kit::screens::assistant_persona_step::AssistantPersonaStep {
            website_domain: "harbor-lane.com",
            on_continue: move |_| {},
        }
    }
}

#[test]
fn persona_step_defaults_to_neutral_balanced_and_handover_on() {
    let html = render(persona_app);
    assert!(html.contains("value=\"neutral\""), "{html}");
    assert!(
        html.contains("Hands over to support@harbor-lane.com when unsure."),
        "{html}"
    );
    assert!(
        html.contains("Harbor Assistant") || html.contains("Harbor lane Assistant"),
        "{html}"
    );
}

fn ready_app() -> Element {
    rsx! {
        rv2_kit::screens::assistant_ready_step::AssistantReadyStep {
            source_domains: domains(&["harbor-lane.com"]),
            brand: "Harbor lane",
            assistant_name: "Harbor lane Assistant",
            on_finish: move |_| {},
            on_edit: move |_: u8| {},
        }
    }
}

#[test]
fn ready_step_lists_each_row_with_a_named_edit_control() {
    let html = render(ready_app);
    assert!(html.contains("Edit voice"), "{html}");
    assert!(html.contains("Edit learns from"), "{html}");
    assert!(html.contains("nobody; it answers itself"), "{html}");
    assert!(
        html.contains("Draft"),
        "first paint is Draft until the glide ends: {html}"
    );
}

fn start_app() -> Element {
    rsx! {
        rv2_kit::screens::assistant_start_page::AssistantStartPage {
            greeting_text: "Good afternoon",
            on_send: move |_| {},
        }
    }
}

#[test]
fn start_page_greets_and_offers_connect_apps_when_none_are_connected() {
    let html = render(start_app);
    assert!(html.contains("Good afternoon"), "{html}");
    assert!(html.contains("Connect apps"), "{html}");
}

fn start_rest_app() -> Element {
    use rv2_kit::screens::assistant_start_page::{
        AssistantStartPage, ConnectedApp, Suggestion, ToolChip,
    };
    let chip = |id: &str, label: &str| ToolChip {
        id: id.into(),
        entry_label: label.into(),
        prompt_template: format!("{label} "),
    };
    let app = |name: &str| ConnectedApp {
        id: name.into(),
        entry_name: name.into(),
        app_mark: None,
    };
    rsx! {
        AssistantStartPage {
            greeting_text: "Good afternoon, Ana",
            invitation_text: "What should we work on today?",
            working_folder: "Harbor lane",
            approval_mode: "Automatic",
            tool_chip_list: vec![
                chip("company", "Company knowledge"),
                chip("doc", "Document"),
                chip("slides", "Presentation"),
                chip("table", "Table"),
                chip("image", "Image"),
                chip("research", "Deep research"),
                chip("chart", "Data visualisation"),
            ],
            connected_app_list: vec![app("Mail"), app("Drive"), app("Calendar")],
            suggestion_list: vec![Suggestion {
                id: "s1".into(),
                entry_label: "Summarise last week's orders".into(),
                prompt_text: "Summarise last week's orders".into(),
                artifact: None,
            }],
            on_send: move |_| {},
        }
    }
}

#[test]
fn start_page_at_rest_shows_seven_chips_and_no_suggestion_cards() {
    let html = render(start_rest_app);
    assert_eq!(
        html.matches("class=\"kit-start__tool\"").count(),
        7,
        "{html}"
    );
    assert!(!html.contains("Summarise last week"), "{html}");
}

#[test]
fn start_page_composer_footer_shows_folder_mode_and_app_marks_without_names() {
    let html = render(start_rest_app);
    assert!(html.contains("Harbor lane"), "{html}");
    assert!(html.contains("Automatic"), "{html}");
    assert!(html.contains("What should we work on today?"), "{html}");
    let connect = html.find("Connect apps").expect("connect action present");
    let mark = html.find("aria-label=\"Mail\"").expect("app mark present");
    assert!(connect < mark, "{html}");
    assert_eq!(
        html.matches("class=\"kit-start__app-mark\"").count(),
        3,
        "{html}"
    );
}

fn start_open_app() -> Element {
    use rv2_kit::screens::assistant_start_page::{AssistantStartPage, Suggestion};
    rsx! {
        AssistantStartPage {
            greeting_text: "Good afternoon",
            open_work_area: true,
            suggestion_list: vec![Suggestion {
                id: "s1".into(),
                entry_label: "Summarise last week's orders".into(),
                prompt_text: "Summarise last week's orders".into(),
                artifact: None,
            }],
            on_send: move |_| {},
        }
    }
}

#[test]
fn start_page_new_tab_shows_tools_and_suggested_sections() {
    let html = render(start_open_app);
    assert!(html.contains("Tools"), "{html}");
    assert!(html.contains("Suggested"), "{html}");
    assert!(html.contains("Summarise last week"), "{html}");
}

fn home_empty_app() -> Element {
    rsx! {
        rv2_kit::screens::ai_workspace_home::AiWorkspaceHome {
            usage_meter: rv2_kit::screens::ai_workspace_home::UsageMeter { used_units: 0, unit_limit: 100, resets_on: "1 Dec".into() },
            month_stats: rv2_kit::screens::ai_workspace_home::MonthStats {
                chats: rv2_kit::screens::ai_workspace_home::MonthStat { entry_value: 0, change_vs_last: 0 },
                files_made: rv2_kit::screens::ai_workspace_home::MonthStat { entry_value: 0, change_vs_last: 0 },
            },
            is_empty: true,
            on_send: move |_: String| {},
        }
    }
}

#[test]
fn workspace_home_empty_state_names_the_composer_button() {
    let html = render(home_empty_app);
    assert!(html.contains("Nothing here yet"), "{html}");
    assert!(html.contains("Start a chat"), "{html}");
}

fn spend_app() -> Element {
    rsx! {
        rv2_kit::screens::ai_spend_chat_planner::AiSpendChatPlanner {
            is_empty: true,
            starter_questions: domains(&["Where does the money go?"]),
            on_ask: move |_| {},
        }
    }
}

#[test]
fn spend_planner_empty_state_offers_starters() {
    let html = render(spend_app);
    assert!(html.contains("Where does the money go?"), "{html}");
}

fn setup_app() -> Element {
    rsx! {
        rv2_kit::screens::workspace_setup_checklist::WorkspaceSetupChecklist {
            workspace_name: "Harbor lane",
            default_step: 3usize,
            on_complete: move |_| {},
        }
    }
}

#[test]
fn setup_checklist_opens_at_step_three_with_earlier_steps_done() {
    let html = render(setup_app);
    assert!(html.contains("2 of 4 done"), "{html}");
    assert!(html.contains("aria-current=\"step\""), "{html}");
    assert!(html.contains("Invite your team"), "{html}");
}

#[test]
fn submission_type_is_reachable_from_the_composer() {
    let submission = Submission {
        text: "hi".into(),
        model_id: "standard".into(),
    };
    assert_eq!(submission.text, "hi");
}

#[test]
fn file_names_fit_the_files_card_and_keep_their_extension() {
    use rv2_kit::screens::ai_workspace_home::FILE_NAME_CHARS;
    let name = "quarterly-board-deck-final-version-seven.pptx";
    let short = middle_truncate(name, FILE_NAME_CHARS);
    assert!(short.chars().count() <= FILE_NAME_CHARS, "{short}");
    assert!(short.ends_with(".pptx"), "{short}");
}

fn home_eyebrow_app() -> Element {
    use rv2_kit::screens::ai_workspace_home::{AiWorkspaceHome, MonthStat, MonthStats, UsageMeter};
    rsx! {
        AiWorkspaceHome {
            workspace_name: "Harbor lane",
            account_name: "Harbor Co",
            usage_meter: UsageMeter { used_units: 0, unit_limit: 10, resets_on: "1 Dec".to_string() },
            month_stats: MonthStats {
                chats: MonthStat { entry_value: 0, change_vs_last: 0 },
                files_made: MonthStat { entry_value: 0, change_vs_last: 0 },
            },
            on_send: move |_: String| {},
        }
    }
}

#[test]
fn home_eyebrow_names_the_workspace_and_the_account() {
    let html = render(home_eyebrow_app);
    assert!(html.contains("Harbor lane · Harbor Co"), "{html}");
}

#[test]
fn pasted_lists_split_on_spaces_and_tabs_too() {
    assert_eq!(
        split_entries("c.com d.com\te.com"),
        domains(&["c.com", "d.com", "e.com"])
    );
}

#[test]
fn typed_comma_commits_the_entries_before_it() {
    use rv2_kit::components::multi_step_wizard_form::split_at_last_comma;
    assert_eq!(
        split_at_last_comma("Harbor Lane,"),
        Some((vec!["Harbor Lane".to_string()], String::new()))
    );
    assert_eq!(
        split_at_last_comma("HL, Harbor, Ti"),
        Some((
            vec!["HL".to_string(), "Harbor".to_string()],
            " Ti".to_string()
        ))
    );
    assert_eq!(split_at_last_comma("HL"), None);
}

#[test]
fn make_primary_moves_the_chosen_name_to_the_front() {
    use rv2_kit::screens::assistant_naming_step::promote_primary;
    let names = domains(&["Harbor Lane", "HL", "Harbor"]);
    assert_eq!(
        promote_primary(&names, "Harbor"),
        domains(&["Harbor", "Harbor Lane", "HL"])
    );
    assert_eq!(promote_primary(&names, "Missing"), names);
}

fn plan_answer_app() -> Element {
    use rv2_kit::components::grounded_reply::SourceEntry;
    use rv2_kit::screens::ai_spend_chat_planner::{
        AiSpendChatPlanner, Answer, AskRequest, Chart, Lever, LeverKind, Turn,
    };
    rsx! {
        AiSpendChatPlanner {
            turn_list: vec![Turn::Answer(Answer {
                id: "a".into(),
                text: "Savings come from two levers.".into(),
                metric_cards: vec![],
                chart_block: Some(Chart::Plan {
                    current_monthly: 1000.0,
                    months: 6,
                    growth_rate: 0.0,
                    lever_list: vec![Lever {
                        id: "cache".into(),
                        entry_label: "Response cache".into(),
                        detail_text: "Reuse answers to repeated questions.".into(),
                        lever_kind: LeverKind::Switch,
                        monthly_base_dollars: 1000.0,
                        max_saving_share: 0.2,
                        max_percent: 0.0,
                        entry_value: 1.0,
                    }],
                }),
                caption_text: "One lever could save part of the spend.".into(),
                source_list: vec![SourceEntry {
                    entry_id: "s1".into(),
                    chip_text: "Usage export".into(),
                    link_target: None,
                    leading_mark: None,
                    snippet: None,
                }],
                follow_up_list: vec![],
            })],
            on_ask: move |_: AskRequest| {},
        }
    }
}

#[test]
fn plan_switch_is_a_styled_switch_and_totals_state_their_units() {
    let html = render(plan_answer_app);
    assert!(html.contains("kit-spend__switch"), "{html}");
    assert!(html.contains("role=\"switch\""), "{html}");
    assert!(html.contains("800 dollars a month"), "{html}");
    assert_eq!(
        html.matches("class=\"kit-spend__source\"").count(),
        1,
        "{html}"
    );
    assert!(!html.contains("1 sources"), "{html}");
}

#[test]
fn section_rail_glyphs_by_key_and_falls_back_to_none() {
    use rv2_kit::screens::assistant_chat_sidebars::section_glyph;
    assert!(section_glyph("chat").is_some());
    assert!(section_glyph("folder").is_some());
    assert!(section_glyph("reports").is_some());
    assert_eq!(section_glyph("ch"), None);
}

#[test]
fn progress_bar_pluralises_one_output() {
    assert_eq!(
        bar_state_text(false, 1),
        "Progress and outputs, 1 output, closed"
    );
    assert_eq!(
        bar_state_text(true, 2),
        "Progress and outputs, 2 outputs, open"
    );
    assert_eq!(
        rv2_kit::screens::assistant_chat_progress_panel::bar_label(1),
        "Progress, 1 output"
    );
}
