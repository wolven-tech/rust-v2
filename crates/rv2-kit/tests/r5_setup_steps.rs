//! SSR checks for the assistant setup steps at rest (amendment 5).

use dioxus::prelude::*;
use rv2_kit::screens::assistant_chat_sidebars::{
    AssistantChatSidebars, ChatItem, ChatMessage, OutputFile, OutputKind as SidebarOutputKind,
    Project, SearchState, SearchStep, Section as RailSection, cell_ref, sheet_rows,
};
use rv2_kit::screens::assistant_naming_step::AssistantNamingStep;
use rv2_kit::screens::company_register::{Account, CompanyRecord, CompanyRegister};
use rv2_kit::screens::knowledge_source_step::{
    KnowledgeSourceStep, can_continue as knowledge_can_continue,
};
use rv2_kit::screens::workspace_chat::{
    ActivityEntry, AgentMessage, AgentState, AgentStatus, Brief, BriefField, BriefFile, FieldType,
    FileKind, Product, WorkspaceChat, is_selected, steps_summary, toggle_option,
};
use rv2_kit::screens::workspace_frame_flat::{FlatStartPanel, StartRow, WorkspaceFrameFlat};
use rv2_kit::screens::workspace_frame_inset::{ChatRow, open_tab_index};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn naming_at_rest() -> Element {
    rsx! {
        AssistantNamingStep {
            website_domain: "northwind.com",
            default_names: vec!["Northwind".to_string(), "Northwind Co.".to_string()],
            on_continue: move |_: Vec<String>| {},
            on_back: move |_| {},
            on_logout: move |_| {},
            account_email: "ana@northwind.com",
        }
    }
}

fn knowledge_at_rest() -> Element {
    rsx! {
        KnowledgeSourceStep {
            default_domains: vec!["harbor-lane.com".to_string()],
            on_continue: move |_: Vec<String>| {},
            account_email: "ana@harbor-lane.com",
            on_logout: move |_| {},
        }
    }
}

#[test]
fn naming_step_at_rest_holds_one_field_with_two_chips_and_a_count_line() {
    let html = render(naming_at_rest);
    assert_eq!(html.matches("kit-nm__input").count(), 1, "{html}");
    assert_eq!(html.matches("class=\"kit-nm__chip\"").count(), 2, "{html}");
    assert!(
        html.contains("2 names. Primary: Northwind. Short forms: Northwind Co."),
        "{html}"
    );
}

#[test]
fn naming_step_at_rest_has_back_and_log_out_and_one_question_with_one_field() {
    let html = render(naming_at_rest);
    assert!(html.contains("Back"), "{html}");
    assert!(html.contains("Log out"), "{html}");
    assert_eq!(html.matches("<input").count(), 1, "{html}");
    assert_eq!(html.matches("aria-label=\"Continue\"").count(), 1, "{html}");
    assert!(!html.contains(">Continue<"), "{html}");
}

#[test]
fn naming_step_preview_shows_draft_card_tag_greeting_and_test_chat() {
    let html = render(naming_at_rest);
    assert!(html.contains("Draft"), "{html}");
    assert!(
        html.contains("Hi, I&#39;m Northwind. How can I help?"),
        "{html}"
    );
    assert!(html.contains("kit-onb__pv-tag"), "{html}");
    assert!(html.contains("What are your opening hours?"), "{html}");
    assert!(
        html.contains("We are open Monday to Friday, 9 to 5."),
        "{html}"
    );
}

#[test]
fn naming_step_chip_row_sits_under_the_field_and_the_hint_under_the_chips() {
    let html = render(naming_at_rest);
    let field = html.find("kit-nm__field").expect("field");
    let chips = html.find("kit-nm__chips").expect("chip row");
    let hint = html.find("kit-nm__hint").expect("hint");
    assert!(field < chips && chips < hint, "{html}");
}

#[test]
fn naming_step_primary_chip_carries_one_dot_and_no_primary_word() {
    let html = render(naming_at_rest);
    assert_eq!(html.matches("kit-nm__dot").count(), 1, "{html}");
    assert!(html.contains("data-primary=\"true\""), "{html}");
    assert!(!html.contains(">Primary<"), "{html}");
    assert!(!html.contains("Primary</"), "{html}");
}

#[test]
fn naming_step_submit_arrow_sits_inside_the_field_and_names_continue() {
    let html = render(naming_at_rest);
    let field = html.find("kit-nm__field").expect("field");
    let submit = html.find("kit-nm__submit").expect("submit");
    let chips = html.find("kit-nm__chips").expect("chip row");
    assert!(field < submit && submit < chips, "{html}");
    assert!(html.contains("aria-label=\"Continue\""), "{html}");
}

#[test]
fn naming_step_back_control_sits_before_the_product_mark_at_the_top_left() {
    let html = render(naming_at_rest);
    let back = html.find(">Back<").expect("back");
    let product = html.find("kit-onb__product").expect("product");
    assert!(back < product, "{html}");
}

#[test]
fn naming_step_card_lists_also_answers_to_with_one_tag_per_short_form() {
    let html = render(naming_at_rest);
    assert!(html.contains("Also answers to"), "{html}");
    assert_eq!(
        html.matches("class=\"kit-onb__pv-tag\"").count(),
        1,
        "{html}"
    );
    assert!(html.contains("Northwind Co.<"), "{html}");
}

#[test]
fn naming_step_domain_row_reads_learning_under_the_knowledge_table() {
    let html = render(naming_at_rest);
    assert!(html.contains("data-status=\"learning\""), "{html}");
    assert!(html.contains("northwind.com<"), "{html}");
    assert_eq!(html.matches("kit-onb__pv-skel-row").count(), 3, "{html}");
}

#[test]
fn naming_step_preview_is_a_sketch_with_tiles_chart_and_test_chat_headings() {
    let html = render(naming_at_rest);
    assert_eq!(html.matches("kit-onb__pv-tile").count(), 3, "{html}");
    assert_eq!(html.matches("kit-onb__pv-col").count(), 7, "{html}");
    assert_eq!(html.matches("kit-onb__pv-bar\"").count(), 6, "{html}");
    assert!(html.contains("Test chat"), "{html}");
    assert!(!html.contains("Conversations"), "{html}");
}

#[test]
fn naming_step_hint_states_the_first_name_rule_in_one_line() {
    let html = render(naming_at_rest);
    assert!(
        html.contains("Your assistant calls itself by the first name."),
        "{html}"
    );
}

#[test]
fn naming_step_count_line_is_visually_hidden_and_polite() {
    let html = render(naming_at_rest);
    assert!(
        html.contains("class=\"kit-sr\" role=\"status\" aria-live=\"polite\""),
        "{html}"
    );
}

#[test]
fn names_line_ends_a_short_form_that_already_ends_in_a_stop_with_one_stop() {
    use rv2_kit::screens::assistant_naming_step::names_line;
    let names = vec!["Northwind".to_string(), "Northwind Co.".to_string()];
    assert_eq!(
        names_line(&names),
        "2 names. Primary: Northwind. Short forms: Northwind Co."
    );
}

#[test]
fn adding_names_skips_empty_and_repeated_parts_and_keeps_order() {
    use rv2_kit::screens::assistant_naming_step::{add_names, promote_primary};
    let list = vec!["Northwind".to_string()];
    let parts = vec![
        "Northwind Co.".to_string(),
        "Northwind".to_string(),
        "NW".to_string(),
    ];
    assert_eq!(
        add_names(&list, &parts),
        vec![
            "Northwind".to_string(),
            "Northwind Co.".to_string(),
            "NW".to_string()
        ]
    );
    assert_eq!(
        promote_primary(&add_names(&list, &parts), "NW"),
        vec![
            "NW".to_string(),
            "Northwind".to_string(),
            "Northwind Co.".to_string()
        ]
    );
}

#[test]
fn knowledge_step_at_rest_holds_one_question_one_chip_field_and_continue() {
    let html = render(knowledge_at_rest);
    assert_eq!(html.matches("kit-src__input").count(), 1, "{html}");
    assert_eq!(html.matches("kit-src__chip\"").count(), 1, "{html}");
    assert_eq!(html.matches("<input").count(), 1, "{html}");
    assert_eq!(html.matches("aria-label=\"Continue\"").count(), 1, "{html}");
    assert!(!html.contains(">Continue<"), "{html}");
    assert!(!html.contains("kit-src__hint\""), "{html}");
    assert!(html.contains("ana@harbor-lane.com"), "{html}");
    assert!(html.contains("Log out"), "{html}");
    assert!(!html.contains("Back"), "{html}");
}

#[test]
fn knowledge_table_at_rest_has_one_domain_row_ready_and_skeleton_rows_below() {
    let html = render(knowledge_at_rest);
    assert_eq!(html.matches("kit-onb__pv-badge").count(), 1, "{html}");
    assert!(html.contains("data-status=\"ready\""), "{html}");
    assert!(html.contains("harbor-lane.com<"), "{html}");
    assert_eq!(html.matches("kit-onb__pv-skel-row").count(), 3, "{html}");
}

#[test]
fn knowledge_preview_card_is_not_in_a_loading_state_at_rest() {
    let html = render(knowledge_at_rest);
    assert!(!html.contains("data-loading=\"true\""), "{html}");
    assert!(html.contains("kit-onb__pv-heading"), "{html}");
    assert!(html.contains("Test chat"), "{html}");
}

#[test]
fn knowledge_preview_is_a_sketch_with_one_question_and_one_answer_bubble() {
    let html = render(knowledge_at_rest);
    assert!(html.contains("data-sketch=\"true\""), "{html}");
    assert_eq!(
        html.matches("kit-onb__pv-bubble--user").count(),
        1,
        "{html}"
    );
    assert_eq!(
        html.matches("kit-onb__pv-bubble--assistant").count(),
        1,
        "{html}"
    );
}

#[test]
fn next_status_cycles_queued_learning_ready() {
    use rv2_kit::screens::knowledge_source_step::{SourceStatus, next_status};
    assert_eq!(next_status(SourceStatus::Queued), SourceStatus::Learning);
    assert_eq!(next_status(SourceStatus::Learning), SourceStatus::Ready);
    assert_eq!(next_status(SourceStatus::Ready), SourceStatus::Queued);
}

#[test]
fn knowledge_step_without_entries_cannot_continue() {
    assert!(!knowledge_can_continue(&[]));
    assert!(knowledge_can_continue(&["harbor-lane.com".to_string()]));
}

#[test]
fn opening_a_suggested_row_twice_activates_the_tab_it_already_has() {
    let names = vec![None, Some("Brand guidelines".to_string())];
    assert_eq!(open_tab_index(&names, "Brand guidelines"), Some(1));
    assert_eq!(open_tab_index(&names, "Pipeline risks"), None);
    assert_eq!(open_tab_index(&[None], "Start"), None);
}

fn flat_at_rest() -> Element {
    let row = |title: &str, source: &str| StartRow {
        title: title.into(),
        source: source.into(),
    };
    rsx! {
        WorkspaceFrameFlat {
            workspace_name: "Northwind",
            current_item: "Folders",
            project_list: vec!["Growth".to_string(), "Operations".to_string(), "Finance".to_string()],
            chat_list: vec![
                ChatRow { id: "f1".into(), title: "Pricing review".into(), pinned: true, unread: false, project_id: None, section: None },
                ChatRow { id: "f2".into(), title: "Hiring plan Q1".into(), pinned: false, unread: false, project_id: Some("Growth".into()), section: None },
                ChatRow { id: "f3".into(), title: "Churn analysis".into(), pinned: false, unread: false, project_id: Some("Growth".into()), section: None },
                ChatRow { id: "f4".into(), title: "Vendor renewals".into(), pinned: false, unread: false, project_id: Some("Operations".into()), section: None },
            ],
            work_area: true,
            panel_heading: "Start",
            side_panel: rsx! {
                FlatStartPanel {
                    tools: vec![row("Web search", "Searches the open web"), row("Spreadsheet", "Formulas and charts"), row("Document", "Drafts and edits")],
                    suggested: vec![row("Weekly revenue summary", "From Revenue overview"), row("Pipeline risks", "From Company register"), row("Onboarding checklist", "From Workspace setup"), row("Brand guidelines", "From File library")],
                }
            },
            content: rsx! { p { "Page heading" } p { "First line of placeholder text." } p { "Second line." } },
        }
    }
}

#[test]
fn flat_frame_at_rest_opens_start_tab_with_tools_and_suggested_rows() {
    let html = render(flat_at_rest);
    assert_eq!(html.matches("kit-flat-start__row").count(), 7, "{html}");
    assert_eq!(html.matches("kit-flat-start__group").count(), 2, "{html}");
    assert!(html.contains("Tools"), "{html}");
    assert!(html.contains("Suggested"), "{html}");
    assert!(html.contains("aria-selected=\"true\""), "{html}");
    assert!(html.contains("New tab"), "{html}");
}

fn register_at_rest() -> Element {
    let record = |id: &str, name: &str| CompanyRecord {
        id: id.into(),
        company_name: name.into(),
        company_category: "Software".into(),
        website_domain: format!("{id}.example"),
        account_owner: "Ada".into(),
        deal_value: 1200.0,
        pipeline_stage: "Lead".into(),
        primary_contact: "Sam Reyes".into(),
        last_contact: "Nov 18".into(),
        logo_url: None,
    };
    rsx! {
        CompanyRegister {
            account: Account { company_name: "Juniper Works".into(), avatar_url: None },
            company_records: vec![
                record("acme", "Acme Labs"),
                record("birch", "Birch and Co"),
                record("cobalt", "Cobalt Health"),
                record("delta", "Delta Freight"),
                record("ember", "Ember Foods"),
                record("fjord", "Fjord Design"),
                record("granite", "Granite Mining"),
                record("harbor", "Harbor Legal"),
                record("iris", "Iris Analytics"),
                record("juno", "Juno Retail"),
            ],
        }
    }
}

#[test]
fn company_register_at_rest_shows_seven_headed_columns_and_ten_rows() {
    let html = render(register_at_rest);
    for heading in [
        "Company",
        "Category",
        "Domain",
        "Owner",
        "Value",
        "Stage",
        "Last contact",
    ] {
        assert!(html.contains(heading), "{heading}: {html}");
    }
    assert!(!html.contains("Website"), "{html}");
    assert!(!html.contains("Deal value"), "{html}");
    assert_eq!(html.matches("aria-rowindex=").count(), 10, "{html}");
}

fn brief_field(
    id: &str,
    label: &str,
    value: &str,
    field_type: FieldType,
    options: &[&str],
) -> BriefField {
    BriefField {
        id: id.into(),
        entry_label: label.into(),
        entry_value: value.into(),
        field_type,
        option_list: options.iter().map(|o| o.to_string()).collect(),
    }
}

fn workspace_at_rest() -> Element {
    let agent = |id: &str, name: &str, role: &str, state: AgentState, step: &str| AgentStatus {
        id: id.into(),
        entry_name: name.into(),
        node_role: role.into(),
        entry_state: state,
        current_step: Some(step.into()),
    };
    let brief = Brief {
        entry_title: "Q1 launch brief".into(),
        product: Some(Product {
            entry_name: "Harbor Pro".into(),
            entry_price: "49 a month".into(),
            sales_line: "312 sold in October".into(),
            is_top_seller: true,
        }),
        field_list: vec![
            brief_field(
                "angle",
                "Selling angle",
                "Speed",
                FieldType::Choice,
                &["Price", "Speed", "Support"],
            ),
            brief_field(
                "audience",
                "Audience",
                "Small teams",
                FieldType::Chips,
                &["Small teams", "Agencies", "Solo founders", "Enterprise"],
            ),
            brief_field(
                "channels",
                "Channels",
                "Email",
                FieldType::Switches,
                &["Email", "Social", "Search"],
            ),
            brief_field(
                "palette",
                "Palette",
                "",
                FieldType::Swatches,
                &["Harbor", "Dune", "Fern", "Ink"],
            ),
            brief_field(
                "formats",
                "Formats",
                "Long form",
                FieldType::Choice,
                &["Long form", "Short form", "Carousel"],
            ),
            brief_field("launch", "Launch date", "2027-01-12", FieldType::Date, &[]),
            brief_field(
                "markets",
                "Markets",
                "DACH",
                FieldType::MultiChoice,
                &["DACH", "UK", "Nordics"],
            ),
            brief_field("budget", "Budget", "6000", FieldType::Number, &[]),
        ],
        file_list: vec![
            BriefFile {
                entry_name: "launch-plan.md".into(),
                file_kind: FileKind::Document,
            },
            BriefFile {
                entry_name: "pricing-sheet.xlsx".into(),
                file_kind: FileKind::Sheet,
            },
            BriefFile {
                entry_name: "channel-copy.docx".into(),
                file_kind: FileKind::Document,
            },
        ],
        activity_log: vec![
            ActivityEntry {
                at: "09:42".into(),
                text: "Budget table delivered".into(),
            },
            ActivityEntry {
                at: "09:38".into(),
                text: "Pricing pages read".into(),
            },
            ActivityEntry {
                at: "09:31".into(),
                text: "Brief split".into(),
            },
        ],
    };
    rsx! {
        WorkspaceChat {
            message_list: vec![
                AgentMessage::User { id: "m1".into(), text: "Plan the launch.".into() },
                AgentMessage::Agent {
                    id: "m2".into(),
                    author: "Orchestrator".into(),
                    text: "Understood.".into(),
                    work_time: Some("Worked for 2 min".into()),
                    step_list: vec![],
                },
                AgentMessage::Agent {
                    id: "m3".into(),
                    author: "Orchestrator".into(),
                    text: "Which angle should the copy lead with?".into(),
                    work_time: None,
                    step_list: vec![],
                },
            ],
            agent_status_list: vec![
                agent("r", "Researcher", "Worker", AgentState::Working, "Reading pages"),
                agent("w", "Writer", "Worker", AgentState::Working, "Drafting copy"),
                agent("a", "Analyst", "Worker", AgentState::Working, "Checking margins"),
                agent("b", "Budget analyst", "Worker", AgentState::Done, "Table delivered"),
                agent("d", "Designer", "Worker", AgentState::Waiting, "Needs a palette choice"),
            ],
            orchestrator_status: "Orchestrator: splits the goal and asks the open questions.",
            brief_record: Some(brief),
            generate_eta: "Usually takes a few minutes.".to_string(),
            on_send: move |_: String| {},
        }
    }
}

#[test]
fn workspace_thread_at_rest_ends_on_a_question_with_one_work_time_line_and_no_folded_steps() {
    let html = render(workspace_at_rest);
    assert_eq!(html.matches("class=\"kit-wc__msg\"").count(), 2, "{html}");
    assert_eq!(
        html.matches("class=\"kit-wc__msg kit-wc__msg--user\"")
            .count(),
        1,
        "{html}"
    );
    assert_eq!(html.matches("kit-wc__worked\"").count(), 1, "{html}");
    assert_eq!(html.matches("<details").count(), 0, "{html}");
    let question = html
        .find("Which angle should the copy lead with?")
        .expect("question");
    let worked = html.find("Worked for 2 min").expect("work time");
    assert!(worked < question, "{html}");
}

#[test]
fn workspace_worker_chips_sit_under_the_last_message_with_state_words() {
    let html = render(workspace_at_rest);
    assert_eq!(html.matches("class=\"kit-wc__agent\"").count(), 5, "{html}");
    assert_eq!(html.matches("Working</span>").count(), 3, "{html}");
    assert!(html.contains("Done</span>"), "{html}");
    assert!(html.contains("Waiting for you</span>"), "{html}");
    let question = html.find("Which angle").expect("question");
    let chips = html.find("kit-wc__agents").expect("worker chips");
    assert!(question < chips, "{html}");
}

#[test]
fn orchestrator_line_sits_above_the_composer_and_is_not_a_worker_chip() {
    let html = render(workspace_at_rest);
    let chips = html.find("kit-wc__agents").expect("worker chips");
    let line = html.find("kit-wc__status").expect("orchestrator line");
    let composer = html.find("kit-wc__composer").expect("composer");
    assert!(chips < line && line < composer, "{html}");
    assert_eq!(html.matches("class=\"kit-wc__agent\"").count(), 5, "{html}");
}

#[test]
fn workspace_brief_at_rest_lists_product_then_eight_fields_in_order_and_no_goal_field() {
    let html = render(workspace_at_rest);
    assert!(html.contains("Harbor Pro"), "{html}");
    assert!(html.contains("Top seller"), "{html}");
    let labels = [
        "Selling angle",
        "Audience",
        "Channels",
        "Palette",
        "Formats",
        "Launch date",
        "Markets",
        "Budget",
    ];
    let mut at = html.find("Harbor Pro").expect("product");
    for label in labels {
        let found = html[at..]
            .find(&format!(">{label}<"))
            .unwrap_or_else(|| panic!("{label}: {html}"));
        at += found + 1;
    }
    assert!(!html.contains(">Goal<"), "{html}");
    let fields = html.matches("kit-wc__field").count() - html.matches("kit-wc__fields").count();
    assert_eq!(fields, 8, "{html}");
}

#[test]
fn brief_renders_channels_as_switches_audience_as_chips_and_palette_as_swatches() {
    let html = render(workspace_at_rest);
    assert_eq!(html.matches("kit-wc__switch\"").count(), 3, "{html}");
    assert_eq!(html.matches("kit-wc__chip\"").count(), 4, "{html}");
    assert_eq!(
        html.matches("class=\"kit-wc__swatch\"").count(),
        4,
        "{html}"
    );
    assert!(html.contains(">Carousel</span>"), "{html}");
}

#[test]
fn generate_sits_below_the_last_field_and_names_its_output_in_one_line() {
    let html = render(workspace_at_rest);
    let budget = html.find("Budget</label>").expect("budget field");
    let generate = html.find("Generate writes a draft").expect("generate line");
    let files = html.find("Files</h3>").expect("files heading");
    assert!(budget < generate && generate < files, "{html}");
    assert!(html.contains("Usually takes a few minutes."), "{html}");
}

#[test]
fn brief_at_rest_lists_three_files_each_with_its_kind_word() {
    let html = render(workspace_at_rest);
    assert_eq!(html.matches("class=\"kit-wc__file\"").count(), 3, "{html}");
    assert_eq!(
        html.matches("class=\"kit-wc__file-kind\"").count(),
        3,
        "{html}"
    );
}

#[test]
fn work_area_at_rest_opens_with_brief_assets_and_activity_tabs_and_the_brief_active() {
    let html = render(workspace_at_rest);
    let brief = html.find(">Brief</button>").expect("brief tab");
    let assets = html.find(">Assets</button>").expect("assets tab");
    let activity = html.find(">Activity</button>").expect("activity tab");
    assert!(brief < assets && assets < activity, "{html}");
    assert!(html.contains("aria-selected=\"true\""), "{html}");
    assert!(!html.contains("kit-wc__open\""), "{html}");
}

#[test]
fn activity_entries_stay_out_of_the_brief_at_rest() {
    let html = render(workspace_at_rest);
    assert!(!html.contains("09:42"), "{html}");
    assert!(!html.contains("kit-wc__log"), "{html}");
}

#[test]
fn a_multi_choice_option_toggles_on_and_off_in_field_order() {
    let listed: Vec<String> = ["Email", "Social", "Search"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(toggle_option("Email", "Search", &listed), "Email, Search");
    assert_eq!(toggle_option("Email, Search", "Email", &listed), "Search");
    assert_eq!(toggle_option("Search", "Email", &listed), "Email, Search");
    assert!(is_selected("Email, Social", "Social"));
    assert!(!is_selected("Email, Social", "Soc"));
}

#[test]
fn a_fold_summary_counts_its_steps_in_words() {
    assert_eq!(steps_summary(1), "1 step");
    assert_eq!(steps_summary(3), "3 steps");
}

#[test]
fn flat_frame_at_rest_lists_four_chats_under_pinned_and_folders() {
    let html = render(flat_at_rest);
    assert_eq!(
        html.matches("class=\"kit-frame__row-open\"").count(),
        4,
        "{html}"
    );
    for title in [
        "Pricing review",
        "Hiring plan Q1",
        "Churn analysis",
        "Vendor renewals",
    ] {
        assert!(html.contains(title), "{title}: {html}");
    }
    for project in ["Growth", "Operations", "Finance"] {
        assert!(html.contains(project), "{project}: {html}");
    }
}

fn sidebars_at_rest() -> Element {
    let section = |id: &str, label: &str, icon: &str| RailSection {
        id: id.into(),
        entry_label: label.into(),
        icon_key: icon.into(),
    };
    let project = |id: &str, name: &str| Project {
        id: id.into(),
        entry_name: name.into(),
    };
    let chat = |id: &str, title: &str, group: Option<&str>, pinned: bool| ChatItem {
        id: id.into(),
        entry_title: title.into(),
        project_id: group.map(str::to_string),
        is_pinned: pinned,
        updated_at: "Mon".into(),
    };
    let search = |query: &str| SearchStep {
        query: query.into(),
        state: SearchState::Done,
        pages: 2,
    };
    rsx! {
        AssistantChatSidebars {
            workspace_name: "Harbor lane",
            section_list: vec![
                section("chat", "Chat", "chat"),
                section("folders", "Folders", "folder"),
                section("reports", "Reports", "reports"),
                section("integrations", "Integrations", "plug"),
                section("settings", "Settings", "gear"),
            ],
            active_section_id: "chat".to_string(),
            project_list: vec![project("p1", "Growth"), project("p2", "Operations"), project("p3", "Finance")],
            chat_list: vec![
                chat("c0", "Hosting shortlist", None, true),
                chat("c1", "Hiring plan Q1", Some("p1"), false),
                chat("c2", "Churn analysis", Some("p1"), false),
                chat("c3", "Pricing tests", Some("p1"), false),
                chat("c4", "Vendor renewals", Some("p2"), false),
                chat("c5", "Office move", Some("p2"), false),
                chat("c6", "Budget review", Some("p3"), false),
                chat("c7", "Forecast notes", Some("p3"), false),
            ],
            active_chat: vec![
                ChatMessage::User { id: "u1".into(), text: "Compare hosting plans.".into() },
                ChatMessage::Assistant {
                    id: "a1".into(),
                    text: "Which region do you run in?".into(),
                    searches: vec![],
                    outputs: vec![],
                },
                ChatMessage::User { id: "u2".into(), text: "Europe.".into() },
                ChatMessage::Assistant {
                    id: "a2".into(),
                    text: "Two plans fit.".into(),
                    searches: vec![search("hosting EU price"), search("single sign-on"), search("seat limits")],
                    outputs: vec!["hosting-summary.md".into(), "pricing-sheet.xlsx".into()],
                },
            ],
            output_list: vec![
                OutputFile {
                    entry_name: "hosting-summary.md".into(),
                    output_kind: SidebarOutputKind::Document,
                    content: "Plan A.\n\nPlan B.".into(),
                },
                OutputFile {
                    entry_name: "pricing-sheet.xlsx".into(),
                    output_kind: SidebarOutputKind::Sheet,
                    content: "Plan, Monthly, Seats, Region\nStarter, 49, 5, EU\nTeam, 79, 25, EU\nBusiness, 149, 80, EU\nEnterprise, 399, 500, EU".into(),
                },
            ],
            open_tab_list: vec!["pricing-sheet.xlsx".to_string(), "hosting-summary.md".to_string()],
            on_send: move |_: String| {},
        }
    }
}

#[test]
fn sidebars_at_rest_shows_five_sections_with_one_current() {
    let html = render(sidebars_at_rest);
    assert_eq!(html.matches("class=\"kit-sb__icon\"").count(), 5, "{html}");
    assert_eq!(html.matches("aria-current=\"page\"").count(), 1, "{html}");
}

#[test]
fn sidebars_at_rest_lists_one_pinned_chat_above_three_project_groups_of_seven() {
    let html = render(sidebars_at_rest);
    assert_eq!(html.matches("class=\"kit-sb__chat\"").count(), 8, "{html}");
    assert_eq!(
        html.matches("class=\"kit-sb__heading\"").count(),
        4,
        "{html}"
    );
    let pinned = html.find(">Pinned<").expect("pinned heading");
    let growth = html.find(">Growth<").expect("growth heading");
    assert!(pinned < growth, "{html}");
}

#[test]
fn sidebars_at_rest_conversation_holds_four_messages_with_folded_searches_and_two_chips() {
    let html = render(sidebars_at_rest);
    assert_eq!(html.matches("kit-sb__msg--user").count(), 2, "{html}");
    assert_eq!(html.matches("kit-sb__msg--assistant").count(), 2, "{html}");
    assert_eq!(html.matches("class=\"kit-sb__chip\"").count(), 2, "{html}");
    assert!(html.contains("Searched the web, 3 searches"), "{html}");
}

#[test]
fn sidebars_at_rest_work_area_shows_sheet_tab_active_beside_document_tab() {
    let html = render(sidebars_at_rest);
    assert_eq!(html.matches("role=\"tab\"").count(), 2, "{html}");
    assert!(html.contains("aria-selected=\"true\""), "{html}");
    assert_eq!(html.matches("class=\"kit-sb__sheet\"").count(), 1, "{html}");
    assert_eq!(html.matches("<th>").count(), 4, "{html}");
    assert_eq!(html.matches("<td").count(), 16, "{html}");
    assert!(html.contains("class=\"kit-sb__formula\""), "{html}");
    assert!(html.contains("A2"), "{html}");
}

#[test]
fn a_sheet_reads_its_rows_and_names_a_cell_by_column_and_row() {
    let rows = sheet_rows("Plan, Monthly\nStarter, 49");
    assert_eq!(rows[0], vec!["Plan".to_string(), "Monthly".to_string()]);
    assert_eq!(rows[1], vec!["Starter".to_string(), "49".to_string()]);
    assert_eq!(cell_ref(1, 0), "A2");
    assert_eq!(cell_ref(3, 2), "C4");
}
