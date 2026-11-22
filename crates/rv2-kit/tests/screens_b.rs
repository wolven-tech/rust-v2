use dioxus::prelude::*;
use rv2_kit::screens::revenue_overview::{
    Account, AverageValue, BreakdownRow, ChangeSign, ChannelSeries, KeyFigure, RankedItem,
    RevenueOverview, SessionPeriod, SessionsSeries, bar_percentages, change_tone, daily_mean,
    format_total, share_width, sparkline_points,
};

use rv2_kit::screens::prompt_test_board::Account as PromptAccount;
use rv2_kit::screens::prompt_test_board::KeyFigure as PromptFigure;
use rv2_kit::screens::prompt_test_board::{
    CaseFilter, CaseStatus, ChangeKind, Decision, InlineSegment, PassPoint, PromptTestBoard,
    ProposedChange, TestCase, Trend, Version, figure_tone, inline_segments, pending_count,
    visible_cases,
};

fn prompt_change(id: &str, before: &str, after: &str) -> ProposedChange {
    ProposedChange {
        id: id.into(),
        kind: ChangeKind::Replace,
        before: before.into(),
        after: after.into(),
        author: "Mina".into(),
    }
}

fn prompt_cases() -> Vec<TestCase> {
    let case = |id: &str, status: CaseStatus| TestCase {
        id: id.into(),
        input: format!("input {id}"),
        expected: "Polite refusal".into(),
        output: None,
        score: None,
        max_score: 5.0,
        latency_ms: Some(820.0),
        status,
    };
    vec![
        case("c1", CaseStatus::Pass),
        case("c2", CaseStatus::Fail),
        case("c3", CaseStatus::Fail),
        case("c4", CaseStatus::Partial),
    ]
}

#[test]
fn inline_marks_replace_and_leaves_missing_targets_out() {
    let changes = vec![prompt_change("p1", "be brief", "be concise")];
    let segments = inline_segments("Answer, be brief.", &changes, &[]);
    assert_eq!(
        segments,
        vec![
            InlineSegment::Plain("Answer, ".into()),
            InlineSegment::Removed {
                change_id: "p1".into(),
                text: "be brief".into()
            },
            InlineSegment::Added {
                change_id: "p1".into(),
                text: "be concise".into()
            },
            InlineSegment::Plain(".".into()),
        ]
    );
    let accepted = inline_segments(
        "Answer, be brief.",
        &changes,
        &[("p1".into(), Decision::Accepted)],
    );
    assert_eq!(
        accepted,
        vec![
            InlineSegment::Plain("Answer, ".into()),
            InlineSegment::Plain("be concise".into()),
            InlineSegment::Plain(".".into()),
        ]
    );
    let rejected = inline_segments(
        "Answer, be brief.",
        &changes,
        &[("p1".into(), Decision::Rejected)],
    );
    assert_eq!(
        rejected,
        vec![InlineSegment::Plain("Answer, be brief.".into())]
    );
    let missing = inline_segments("Answer.", &[prompt_change("p2", "absent", "x")], &[]);
    assert_eq!(missing, vec![InlineSegment::Plain("Answer.".into())]);
}

#[test]
fn pending_count_drops_once_a_change_is_decided() {
    let changes = vec![prompt_change("a", "x", "y"), prompt_change("b", "x", "y")];
    assert_eq!(pending_count(&changes, &[]), 2);
    assert_eq!(
        pending_count(&changes, &[("a".into(), Decision::Accepted)]),
        1
    );
}

#[test]
fn fail_filter_admits_only_failing_cases() {
    let cases = prompt_cases();
    let fails = visible_cases(&cases, CaseFilter::Fail);
    assert_eq!(fails.len(), 2);
    assert!(fails.iter().all(|c| c.status == CaseStatus::Fail));
    assert_eq!(visible_cases(&cases, CaseFilter::All).len(), 4);
}

#[test]
fn figure_tone_uses_the_direction_that_is_good() {
    assert_eq!(figure_tone(Trend::Up, Trend::Up), "positive");
    assert_eq!(figure_tone(Trend::Up, Trend::Down), "negative");
    assert_eq!(figure_tone(Trend::Flat, Trend::Down), "neutral");
}

fn prompt_loaded_app() -> Element {
    rsx! {
        PromptTestBoard {
            account: PromptAccount { name: "Mina Okafor".into(), avatar_url: None },
            prompt_name: "Support reply",
            versions: vec![
                Version { id: "v1".into(), label: "v1".into(), created_at: "Nov 2".into(), pass_rate: 62.0 },
                Version { id: "v2".into(), label: "v2".into(), created_at: "Nov 12".into(), pass_rate: 78.0 },
            ],
            current_version: "v2",
            model: "Model A",
            last_run_text: "last run two hours ago",
            key_figures: vec![PromptFigure {
                label: "Median latency".into(),
                value: "1.2 s".into(),
                delta: "−0.3 s".into(),
                trend: Trend::Down,
                good_when: Trend::Down,
            }],
            pass_rate_series: vec![
                PassPoint { version_label: "v1".into(), pass_rate: 62.0 },
                PassPoint { version_label: "v2".into(), pass_rate: 78.0 },
            ],
            prompt_text: "Answer, be brief. Cite the policy.",
            proposed_changes: vec![prompt_change("p1", "be brief", "be concise")],
            test_cases: prompt_cases(),
        }
    }
}

fn prompt_empty_app() -> Element {
    rsx! {
        PromptTestBoard {
            account: PromptAccount { name: "Mina Okafor".into(), avatar_url: None },
            prompt_name: "Support reply",
            is_empty: true,
        }
    }
}

fn prompt_loading_app() -> Element {
    rsx! {
        PromptTestBoard {
            account: PromptAccount { name: "Mina Okafor".into(), avatar_url: None },
            prompt_name: "Support reply",
            is_loading: true,
        }
    }
}

#[test]
fn prompt_board_marks_tones_bars_and_pending_changes() {
    let html = render(prompt_loaded_app);
    assert!(html.contains("data-tone=\"positive\""));
    assert!(html.contains("Pass rate by version"));
    assert!(html.contains("<del"));
    assert!(html.contains("<ins"));
    assert!(html.contains("Accept all (1)"));
    assert!(html.contains("Fail "));
}

#[test]
fn prompt_board_empty_hides_charts_and_changes() {
    let html = render(prompt_empty_app);
    assert!(html.contains("No test cases yet"));
    assert!(html.contains("Add a test case"));
    assert!(!html.contains("Pass rate by version"));
    assert!(!html.contains("Changes"));
}

#[test]
fn prompt_board_loading_keeps_run_control_usable() {
    let html = render(prompt_loading_app);
    assert!(html.contains("Run evaluation"));
    assert!(html.contains("aria-busy=\"true\""));
    assert!(!html.contains("No test cases yet"));
}

use rv2_kit::components::sheet_grid::GridRow;
use rv2_kit::screens::company_register::{
    Account as RegisterAccount, CompanyRecord, CompanyRegister, changed_cell, empty_title,
    view_admits, view_columns, view_rows,
};

fn company_records() -> Vec<CompanyRecord> {
    let record = |id: &str, name: &str, value: f64, stage: &str| CompanyRecord {
        id: id.into(),
        company_name: name.into(),
        company_category: "Software".into(),
        website_domain: format!("{id}.example"),
        account_owner: "Ada".into(),
        deal_value: value,
        pipeline_stage: stage.into(),
        primary_contact: "Sam Reyes".into(),
        last_contact: "Nov 18".into(),
        logo_url: None,
    };
    vec![
        record("acme", "Acme Labs", 12_400.0, "Proposal"),
        record("birch", "Birch & Co", 0.0, "Renewal due"),
    ]
}

#[test]
fn deals_view_keeps_only_valued_records() {
    let records = company_records();
    assert!(view_admits("Deals", &records[0]));
    assert!(!view_admits("Deals", &records[1]));
    assert_eq!(view_rows(&records, "Deals").len(), 1);
    assert!(view_rows(&records, "Renewals")[0].values["stage"].starts_with("Renewal"));
    assert_eq!(view_columns("Deals")[1], "deal_value");
    assert_eq!(empty_title("Renewals"), "No renewals yet");
    assert_eq!(empty_title("Company CRM"), "No companies yet");
}

#[test]
fn changed_cell_reports_the_first_edited_value_only() {
    let old = view_rows(&company_records(), "Company CRM");
    let mut new = old.clone();
    new[1].values.insert("stage".into(), "Won".into());
    assert_eq!(
        changed_cell(&old, &new),
        Some(("birch".into(), "stage".into(), "Won".into()))
    );
    assert_eq!(changed_cell(&old, &old), None);
    let mut grown: Vec<GridRow> = old.clone();
    grown.push(rv2_kit::screens::company_register::blank_row("new-1"));
    assert_eq!(changed_cell(&old, &grown), None);
}

fn register_loaded_app() -> Element {
    rsx! {
        CompanyRegister {
            account: RegisterAccount { company_name: "Juniper Works".into(), avatar_url: None },
            company_records: company_records(),
        }
    }
}

fn register_deals_empty_app() -> Element {
    rsx! {
        CompanyRegister {
            account: RegisterAccount { company_name: "Juniper Works".into(), avatar_url: None },
            is_empty: true,
        }
    }
}

fn register_loading_app() -> Element {
    rsx! {
        CompanyRegister {
            account: RegisterAccount { company_name: "Juniper Works".into(), avatar_url: None },
            is_loading: true,
        }
    }
}

#[test]
fn register_renders_the_grid_with_tabs_and_add_row() {
    let html = render(register_loaded_app);
    assert!(html.contains("role=\"grid\""));
    assert!(html.contains("Acme Labs"));
    assert!(html.contains("role=\"tab\""));
    assert!(html.contains("Add row"));
    assert!(html.contains("Double-click a cell to edit"));
    assert!(html.contains("aria-selected=\"true\""));
}

#[test]
fn register_loading_shows_secondary_rows_under_real_headings() {
    let html = render(register_loading_app);
    assert!(html.contains("aria-busy=\"true\""));
    assert!(html.contains("Last contact"));
    assert!(html.contains("kit-cr__ph"));
    assert!(!html.contains("role=\"grid\""));
}

#[test]
fn register_empty_view_replaces_the_table_with_its_add_action() {
    let html = render(register_deals_empty_app);
    assert!(html.contains("No companies yet"));
    assert!(html.contains("Add a record"));
    assert!(!html.contains("role=\"grid\""));
}

use std::collections::BTreeMap;

use rv2_kit::screens::connected_apps_catalogue::{
    Account as CatalogueAccount, App, ConnectedAppsCatalogue, Tab, can_submit, effective_connected,
    tab_count, visible_apps,
};

fn catalogue_apps() -> Vec<App> {
    let app = |id: &str, name: &str, category: &str, connected: bool, popular: bool| App {
        id: id.into(),
        name: name.into(),
        summary: format!("{name} summary"),
        category: category.into(),
        connected,
        popular,
        logo_url: None,
        bundle: Vec::new(),
    };
    vec![
        app("slack", "Slack", "Messaging", true, true),
        app("teams", "Microsoft Teams", "Messaging", false, false),
        app("drive", "Google Drive", "Files", false, true),
        app("sheets", "Spreadsheets", "Files", true, false),
    ]
}

#[test]
fn tabs_filter_on_connection_and_popularity() {
    let apps = catalogue_apps();
    let none = BTreeMap::new();
    assert_eq!(visible_apps(&apps, Tab::Connected, "", &none).len(), 2);
    assert_eq!(visible_apps(&apps, Tab::Available, "", &none).len(), 2);
    assert!(
        visible_apps(&apps, Tab::Connected, "", &none)
            .iter()
            .all(|a| a.connected)
    );
    assert_eq!(tab_count(&apps, Tab::Popular, &none), 2);
}

#[test]
fn search_matches_across_categories_and_ignores_case() {
    let apps = catalogue_apps();
    let none = BTreeMap::new();
    let hits = visible_apps(&apps, Tab::All, "SHEET", &none);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].name, "Spreadsheets");
    assert_eq!(visible_apps(&apps, Tab::All, "  ", &none).len(), 4);
}

#[test]
fn own_connect_overrides_the_host_flag() {
    let apps = catalogue_apps();
    let mut overrides = BTreeMap::new();
    overrides.insert("teams".to_string(), true);
    assert!(effective_connected(&apps[1], &overrides));
    assert_eq!(tab_count(&apps, Tab::Available, &overrides), 1);
}

#[test]
fn request_needs_a_name() {
    assert!(!can_submit("   "));
    assert!(can_submit("Notion"));
}

fn catalogue_loaded_app() -> Element {
    rsx! {
        ConnectedAppsCatalogue {
            account: CatalogueAccount { name: "Ada Lovelace".into(), avatar_url: None },
            apps: catalogue_apps(),
        }
    }
}

fn catalogue_empty_app() -> Element {
    rsx! { ConnectedAppsCatalogue { account: CatalogueAccount { name: "Ada Lovelace".into(), avatar_url: None }, is_empty: true } }
}

fn catalogue_loading_app() -> Element {
    rsx! { ConnectedAppsCatalogue { account: CatalogueAccount { name: "Ada Lovelace".into(), avatar_url: None }, is_loading: true } }
}

#[test]
fn catalogue_rows_name_their_connect_control() {
    let html = render(catalogue_loaded_app);
    assert!(html.contains("aria-label=\"Connect Microsoft Teams\""));
    assert!(html.contains("aria-label=\"Connected: Slack, open menu\""));
    assert!(html.contains("role=\"tab\""));
    assert!(html.contains("Request integration"));
}

#[test]
fn catalogue_loading_shows_placeholder_sections_and_empty_offers_request() {
    let loading = render(catalogue_loading_app);
    assert!(loading.contains("aria-busy=\"true\""));
    assert!(loading.contains("kit-cac__ph"));
    let empty = render(catalogue_empty_app);
    assert!(empty.contains("No integrations yet"));
    assert!(empty.contains("Request integration"));
}

use rv2_kit::screens::file_library::{
    Account as LibraryAccount, FileLibrary, Folder, FolderFilter, Kind, Scope,
    SortKey as LibrarySortKey, Template, TemplateFilter, matches_query, sort_folders,
    sort_templates,
};

fn library_folders() -> Vec<Folder> {
    vec![
        Folder {
            id: "f1".into(),
            name: "Q3 Campaign".into(),
            scope: Scope::Shared,
            file_count: 12,
        },
        Folder {
            id: "f2".into(),
            name: "Drafts".into(),
            scope: Scope::Private,
            file_count: 3,
        },
        Folder {
            id: "f3".into(),
            name: "Board packs".into(),
            scope: Scope::Shared,
            file_count: 7,
        },
    ]
}

fn library_templates() -> Vec<Template> {
    vec![
        Template {
            id: "t1".into(),
            name: "Weekly update".into(),
            kind: Kind::Doc,
            owner: "Ada".into(),
            scope: Scope::Mine,
        },
        Template {
            id: "t2".into(),
            name: "Budget".into(),
            kind: Kind::Sheet,
            owner: "Lin".into(),
            scope: Scope::Shared,
        },
    ]
}

#[test]
fn folder_scope_tab_keeps_only_that_scope() {
    let shared: Vec<Scope> = library_folders()
        .iter()
        .filter(|f| FolderFilter::Shared.admits(f.scope))
        .map(|f| f.scope)
        .collect();
    assert_eq!(shared, vec![Scope::Shared, Scope::Shared]);
    assert!(TemplateFilter::Mine.admits(Scope::Mine));
    assert!(!TemplateFilter::Mine.admits(Scope::Shared));
}

#[test]
fn search_is_case_insensitive_and_empty_matches_all() {
    assert!(matches_query("Q3 Campaign", "campaign"));
    assert!(!matches_query("Drafts", "budget"));
    assert!(matches_query("Drafts", "  "));
}

#[test]
fn name_sort_orders_each_section_alphabetically() {
    let names: Vec<String> = sort_folders(&library_folders(), LibrarySortKey::Name)
        .into_iter()
        .map(|f| f.name)
        .collect();
    assert_eq!(names, vec!["Board packs", "Drafts", "Q3 Campaign"]);
    let templates: Vec<String> = sort_templates(&library_templates(), LibrarySortKey::Name)
        .into_iter()
        .map(|t| t.name)
        .collect();
    assert_eq!(templates, vec!["Budget", "Weekly update"]);
    assert_eq!(Kind::Sheet.code(), "XLS");
}

fn library_loaded_app() -> Element {
    rsx! {
        FileLibrary {
            account: LibraryAccount { name: "Ada Lovelace".into(), avatar_url: None },
            folders: library_folders(),
            templates: library_templates(),
        }
    }
}

fn library_empty_app() -> Element {
    rsx! { FileLibrary { account: LibraryAccount { name: "Ada Lovelace".into(), avatar_url: None }, is_empty: true } }
}

fn library_loading_app() -> Element {
    rsx! { FileLibrary { account: LibraryAccount { name: "Ada Lovelace".into(), avatar_url: None }, is_loading: true } }
}

#[test]
fn library_names_folder_and_template_buttons_with_scope_and_count() {
    let html = render(library_loaded_app);
    assert!(html.contains("aria-label=\"Q3 Campaign, Shared, 12 files\""));
    assert!(html.contains("aria-label=\"Budget, XLS, owned by Lin, Shared\""));
    assert!(html.contains("aria-pressed=\"false\""));
}

#[test]
fn library_empty_shows_each_section_its_own_action() {
    let html = render(library_empty_app);
    assert!(html.contains("No folders yet"));
    assert!(html.contains("No templates yet"));
    assert!(html.contains("New folder"));
    assert!(html.contains("Add template"));
}

#[test]
fn library_loading_keeps_headings() {
    let html = render(library_loading_app);
    assert!(html.contains("aria-busy=\"true\""));
    assert!(html.contains("Folders"));
    assert!(html.contains("File templates"));
}

use rv2_kit::screens::folder_detail::{
    Account as FolderAccount, FileItem, FileKind, FolderDetail, ShareState, crumb_path, range_ids,
    type_ahead_index,
};

fn folder_files() -> Vec<FileItem> {
    let file = |id: &str, name: &str, kind: FileKind| FileItem {
        id: id.into(),
        name: name.into(),
        kind,
        updated_at: "Nov 18".into(),
        subtitle: "Ada · Nov 18".into(),
        preview_url: None,
    };
    vec![
        file("a", "Brief.pdf", FileKind::Pdf),
        file("b", "Budget.xlsx", FileKind::Sheet),
        file("c", "Brand notes", FileKind::Doc),
        file("d", "Brief v2.pdf", FileKind::Pdf),
    ]
}

#[test]
fn shift_range_is_inclusive_in_either_direction() {
    let files = folder_files();
    assert_eq!(range_ids(&files, 1, 3), vec!["b", "c", "d"]);
    assert_eq!(range_ids(&files, 2, 0), vec!["a", "b", "c"]);
}

#[test]
fn type_ahead_finds_the_next_match_from_the_focused_card() {
    let files = folder_files();
    assert_eq!(type_ahead_index(&files, "bri", 0), Some(0));
    assert_eq!(type_ahead_index(&files, "bri", 1), Some(3));
    assert_eq!(type_ahead_index(&files, "zz", 0), None);
}

#[test]
fn breadcrumb_path_runs_from_root_to_the_chosen_level() {
    let crumbs = vec![
        "Clients".to_string(),
        "Juniper".to_string(),
        "Q3 Campaign".to_string(),
    ];
    assert_eq!(crumb_path(&crumbs, 1), "Clients / Juniper");
    assert_eq!(crumb_path(&crumbs, 2), "Clients / Juniper / Q3 Campaign");
}

fn folder_loaded_app() -> Element {
    rsx! {
        FolderDetail {
            folder_name: "Q3 Campaign",
            crumbs: vec!["Clients".to_string(), "Juniper".to_string(), "Q3 Campaign".to_string()],
            account: FolderAccount { name: "Ada Lovelace".into(), avatar_url: None },
            files: folder_files(),
            share_state: ShareState::Shared,
        }
    }
}

fn folder_empty_app() -> Element {
    rsx! {
        FolderDetail {
            account: FolderAccount { name: "Ada Lovelace".into(), avatar_url: None },
            is_empty: true,
        }
    }
}

fn folder_loading_app() -> Element {
    rsx! {
        FolderDetail {
            account: FolderAccount { name: "Ada Lovelace".into(), avatar_url: None },
            is_loading: true,
        }
    }
}

#[test]
fn folder_cards_name_their_kind_and_the_breadcrumb_marks_the_current_level() {
    let html = render(folder_loaded_app);
    assert!(html.contains("aria-label=\"Brief.pdf, PDF\""));
    assert!(html.contains("role=\"listbox\""));
    assert!(html.contains("aria-multiselectable=\"true\""));
    assert!(html.contains("aria-current=\"page\""));
    assert!(html.contains("Shared"));
}

#[test]
fn folder_empty_offers_a_chat_and_loading_marks_busy() {
    let empty = render(folder_empty_app);
    assert!(empty.contains("No files yet"));
    assert!(empty.contains("Start a chat about this folder"));
    let loading = render(folder_loading_app);
    assert!(loading.contains("aria-busy=\"true\""));
    assert!(!loading.contains("No files yet"));
}

use rv2_kit::screens::autonomy_settings::{
    ActionClass, ActionRule, AppEntry, AutonomyLevel, AutonomySettings, Setting, WeekCounts,
    apply_level, implied_setting, is_changed, menu_options, parse_limit, setting_counts,
};

fn autonomy_rule(id: &str, class: ActionClass, setting: Setting, fixed: bool) -> ActionRule {
    ActionRule {
        id: id.into(),
        app_id: "mailroom".into(),
        action_name: format!("Action {id}"),
        detail_text: None,
        mark: None,
        rule_setting: setting,
        default_setting: setting,
        action_class: class,
        is_fixed: fixed,
        amount_limit: None,
        week_counts: WeekCounts {
            done: 3,
            asked: 1,
            blocked: 0,
        },
        recent_events: Vec::new(),
    }
}

fn autonomy_rules() -> Vec<ActionRule> {
    vec![
        autonomy_rule("read", ActionClass::Read, Setting::Allow, false),
        autonomy_rule("write", ActionClass::Write, Setting::Ask, false),
        autonomy_rule("risky", ActionClass::Risky, Setting::Ask, false),
        autonomy_rule("fixed", ActionClass::Risky, Setting::Never, true),
    ]
}

#[test]
fn careful_reads_run_and_changes_ask() {
    assert_eq!(
        implied_setting(AutonomyLevel::Careful, ActionClass::Read),
        Setting::Allow
    );
    assert_eq!(
        implied_setting(AutonomyLevel::Careful, ActionClass::Write),
        Setting::Ask
    );
    let applied = apply_level(&autonomy_rules(), AutonomyLevel::Careful);
    assert!(
        applied
            .iter()
            .find(|r| r.id == "read")
            .is_some_and(|r| r.rule_setting == Setting::Allow)
    );
    assert!(
        applied
            .iter()
            .find(|r| r.id == "write")
            .is_some_and(|r| r.rule_setting == Setting::Ask)
    );
}

#[test]
fn a_hand_change_is_marked_and_the_fixed_rule_is_never_marked() {
    let mut rules = autonomy_rules();
    assert!(!is_changed(&rules[0], AutonomyLevel::Balanced));
    rules[0].rule_setting = Setting::Ask;
    assert!(is_changed(&rules[0], AutonomyLevel::Balanced));
    assert!(!is_changed(&rules[3], AutonomyLevel::HandsOff));
    let reapplied = apply_level(&rules, AutonomyLevel::Balanced);
    assert!(!is_changed(&reapplied[0], AutonomyLevel::Balanced));
}

#[test]
fn a_fixed_rule_offers_only_ask_and_never() {
    let rules = autonomy_rules();
    assert_eq!(menu_options(&rules[3]), vec![Setting::Ask, Setting::Never]);
    assert_eq!(menu_options(&rules[0]).len(), 3);
}

#[test]
fn summary_counts_equal_the_rules_at_each_setting() {
    assert_eq!(setting_counts(&autonomy_rules()), (1, 2, 1));
}

#[test]
fn limit_must_be_a_non_negative_number() {
    assert_eq!(parse_limit("250.5"), Some(250.5));
    assert_eq!(parse_limit("-3"), None);
    assert_eq!(parse_limit("ten"), None);
    assert_eq!(parse_limit(""), None);
}

fn autonomy_mailroom() -> Vec<AppEntry> {
    vec![AppEntry {
        id: "mailroom".into(),
        entry_name: "Mailroom".into(),
        account: Some("ops@juniper.example".into()),
        mark: None,
    }]
}

fn autonomy_loaded_app() -> Element {
    rsx! {
        AutonomySettings {
            initial_level: AutonomyLevel::Balanced,
            app_list: autonomy_mailroom(),
            action_rules: autonomy_rules(),
            initial_app_id: Some("mailroom".to_string()),
        }
    }
}

fn autonomy_empty_app() -> Element {
    rsx! { AutonomySettings { is_empty: true } }
}

#[test]
fn autonomy_names_the_app_counts_and_the_checked_level() {
    let html = render(autonomy_loaded_app);
    assert!(html.contains("aria-checked=\"true\""));
    assert!(html.contains("Mailroom, account: ops@juniper.example: 1 on its own, 2 asks, 1 never"));
    assert!(html.contains("Fixed rules"));
    assert!(html.contains("Reads"));
    assert!(html.contains("Needs care"));
}

#[test]
fn autonomy_empty_offers_connect_and_hides_fixed_rules() {
    let html = render(autonomy_empty_app);
    assert!(html.contains("Connect app"));
    assert!(!html.contains("Fixed rules"));
}

use rv2_kit::screens::decision_queue::{
    AnswerRecord, AnswerValue, DecisionQueue, Request, RequestKind, RequestOption, answer_for,
    move_to_end, move_to_top, option_for_digit, row_label, step_index,
};

fn queue_request(id: &str, kind: RequestKind, answered: bool) -> Request {
    Request {
        id: id.into(),
        request_kind: kind,
        question_text: format!("Question {id}?"),
        context_text: Some("Context for the question.".into()),
        task_name: "Refund batch".into(),
        asked_at: "12 min ago".into(),
        is_unread: !answered,
        option_list: vec![
            RequestOption {
                id: "a".into(),
                entry_title: "Refund in full".into(),
                summary: Some("Customer keeps nothing".into()),
                is_recommended: true,
                advantages: vec!["Fast".into()],
                drawbacks: vec!["Costs more".into()],
                preview_text: None,
            },
            RequestOption {
                id: "b".into(),
                entry_title: "Partial refund".into(),
                summary: None,
                is_recommended: false,
                advantages: Vec::new(),
                drawbacks: Vec::new(),
                preview_text: None,
            },
        ],
        fact_rows: Vec::new(),
        suggested_answers: vec!["Yes, go ahead".into()],
        answer_record: answered.then(|| AnswerRecord {
            answer_text: "Refund in full".into(),
            answered_at: "5 min ago".into(),
            entry_label: None,
        }),
    }
}

#[test]
fn keys_step_and_clamp_at_both_ends() {
    assert_eq!(step_index(3, 0, -1), Some(0));
    assert_eq!(step_index(3, 2, 1), Some(2));
    assert_eq!(step_index(3, 1, 1), Some(2));
    assert_eq!(step_index(0, 0, 1), None);
}

#[test]
fn later_moves_to_the_end_and_reopen_to_the_top() {
    let order = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(move_to_end(&order, "a"), vec!["b", "c", "a"]);
    assert_eq!(move_to_top(&order, "c"), vec!["c", "a", "b"]);
}

#[test]
fn digits_pick_options_by_position_and_ignore_out_of_range() {
    assert_eq!(option_for_digit('1', 2), Some(0));
    assert_eq!(option_for_digit('2', 2), Some(1));
    assert_eq!(option_for_digit('3', 2), None);
    assert_eq!(option_for_digit('0', 2), None);
    assert_eq!(option_for_digit('x', 2), None);
}

#[test]
fn each_request_kind_sends_its_own_answer_shape() {
    assert_eq!(
        answer_for(RequestKind::Pick, Some("b"), "", None),
        Some(AnswerValue::Option {
            option_id: "b".into()
        })
    );
    assert_eq!(
        answer_for(RequestKind::Approval, None, "", Some(false)),
        Some(AnswerValue::Decline)
    );
    assert_eq!(answer_for(RequestKind::Question, None, "   ", None), None);
    assert_eq!(
        answer_for(RequestKind::Question, None, " ship it ", None),
        Some(AnswerValue::Text {
            free_text: "ship it".into()
        })
    );
}

#[test]
fn artifact_tabs_index_past_the_last_tab_is_the_plus_control() {
    use rv2_kit::screens::decision_queue::artifact_tab_id;
    assert_eq!(artifact_tab_id("kit-dq-1", 0, 2), "kit-dq-1-art-0");
    assert_eq!(artifact_tab_id("kit-dq-1", 1, 2), "kit-dq-1-art-1");
    assert_eq!(artifact_tab_id("kit-dq-1", 2, 2), "kit-dq-1-art-add");
    assert_eq!(artifact_tab_id("kit-dq-1", 0, 0), "kit-dq-1-art-add");
}

fn runs_of(parts: &[(&str, bool)]) -> Vec<rv2_kit::screens::decision_queue::Run> {
    use rv2_kit::screens::decision_queue::Run;
    parts
        .iter()
        .map(|(text, bold)| Run {
            text: text.to_string(),
            bold: *bold,
            ..Run::default()
        })
        .collect()
}

#[test]
fn bold_on_a_middle_word_splits_only_that_word() {
    use rv2_kit::screens::decision_queue::{Mark, paragraph_text, toggle_mark};
    let plain = runs_of(&[("The quick fox", false)]);
    let bolded = toggle_mark(&plain, 4, 9, Mark::Bold);
    assert_eq!(
        bolded,
        runs_of(&[("The ", false), ("quick", true), (" fox", false)])
    );
    assert_eq!(toggle_mark(&bolded, 4, 9, Mark::Bold), plain);
    assert_eq!(paragraph_text(&bolded), "The quick fox");
}

#[test]
fn toggle_across_partly_bold_text_sets_the_whole_selection() {
    use rv2_kit::screens::decision_queue::{Mark, toggle_mark};
    let mixed = runs_of(&[("ab", true), ("cd", false)]);
    assert_eq!(
        toggle_mark(&mixed, 1, 3, Mark::Bold),
        runs_of(&[("abc", true), ("d", false)])
    );
}

#[test]
fn line_starts_turn_a_plain_paragraph_into_each_kind() {
    use rv2_kit::screens::decision_queue::{Paragraph, ParagraphKind, line_start};
    let typed = |text: &str| Paragraph::plain(text);
    assert_eq!(
        line_start(&typed("- item")),
        Some((ParagraphKind::Bullet, 2))
    );
    assert_eq!(
        line_start(&typed("* item")),
        Some((ParagraphKind::Bullet, 2))
    );
    assert_eq!(
        line_start(&typed("1. first")),
        Some((ParagraphKind::Numbered, 3))
    );
    assert_eq!(
        line_start(&typed("12. twelfth")),
        Some((ParagraphKind::Numbered, 4))
    );
    assert_eq!(
        line_start(&typed("[ ] task")),
        Some((ParagraphKind::Checklist { done: false }, 4))
    );
    assert_eq!(
        line_start(&typed("# Title")),
        Some((ParagraphKind::Heading, 2))
    );
    assert_eq!(line_start(&typed("-item")), None);
    assert_eq!(line_start(&typed("1 item")), None);
    assert_eq!(line_start(&typed("mid - line")), None);
}

#[test]
fn enter_splits_runs_and_keeps_list_kinds() {
    use rv2_kit::screens::decision_queue::{Paragraph, ParagraphKind, Run, split_paragraph};
    let bullet = Paragraph {
        kind: ParagraphKind::Bullet,
        runs: runs_of(&[("one two", false)]),
    };
    let (left, right) = split_paragraph(&bullet, 3);
    assert_eq!(left.runs, runs_of(&[("one", false)]));
    assert_eq!(right.runs, runs_of(&[(" two", false)]));
    assert_eq!(right.kind, ParagraphKind::Bullet);
    let done = Paragraph {
        kind: ParagraphKind::Checklist { done: true },
        runs: vec![Run {
            text: "ab".into(),
            ..Run::default()
        }],
    };
    assert_eq!(
        split_paragraph(&done, 2).1.kind,
        ParagraphKind::Checklist { done: false }
    );
}

#[test]
fn readback_and_markup_round_trip_marks_and_escape_values() {
    use rv2_kit::screens::decision_queue::{Run, parse_readback, runs_html};
    let runs = vec![
        Run {
            text: "a<b> & \"c\"".into(),
            bold: true,
            link: Some("https://example.com/?a=1&b=2".into()),
            ..Run::default()
        },
        Run {
            text: " tail".into(),
            italic: true,
            ..Run::default()
        },
    ];
    let html = runs_html(&runs);
    assert!(html.contains("a&lt;b&gt; &amp; &quot;c&quot;"));
    assert!(html.contains("data-href=\"https://example.com/?a=1&amp;b=2\""));
    assert!(html.contains("data-b=\"1\""));
    let message = "3\u{1e}3\u{1e}b\u{1f}bold text\u{1f}https://x.test\u{1e}\u{1f}plain\u{1f}\u{1e}i\u{1f} tail\u{1f}";
    let (start, end, parsed) = parse_readback(message).expect("readback parses");
    assert_eq!((start, end), (3, 3));
    assert_eq!(parsed.len(), 3);
    assert!(parsed[0].bold && parsed[0].link.as_deref() == Some("https://x.test"));
    assert_eq!(parsed[1].text, "plain");
    assert!(parsed[2].italic);
}

#[test]
fn numbered_paragraphs_count_within_their_run() {
    use rv2_kit::screens::decision_queue::{Paragraph, ParagraphKind, numbers};
    let numbered = |text: &str| Paragraph {
        kind: ParagraphKind::Numbered,
        runs: runs_of(&[(text, false)]),
    };
    let paragraphs = vec![
        numbered("a"),
        numbered("b"),
        Paragraph::plain("gap"),
        numbered("c"),
    ];
    assert_eq!(numbers(&paragraphs), vec![1, 2, 0, 1]);
}

#[test]
fn links_accept_only_web_addresses() {
    use rv2_kit::screens::decision_queue::normalise_link;
    assert_eq!(
        normalise_link("  https://example.com/a "),
        Some("https://example.com/a".to_string())
    );
    assert_eq!(
        normalise_link("http://x.test"),
        Some("http://x.test".to_string())
    );
    assert_eq!(normalise_link("javascript:alert(1)"), None);
    assert_eq!(normalise_link("https://"), None);
    assert_eq!(normalise_link("   "), None);
}

#[test]
fn sheet_edits_read_back_in_column_order() {
    use rv2_kit::components::sheet_grid::GridRow;
    use rv2_kit::screens::decision_queue::sheet_rows_back;
    let grid = vec![GridRow {
        row_id: "r0".into(),
        row_logo: None,
        values: BTreeMap::from([
            ("c0".to_string(), "Order 1042".to_string()),
            ("c1".to_string(), "$700".to_string()),
        ]),
    }];
    assert_eq!(
        sheet_rows_back(2, &grid),
        vec![vec!["Order 1042".to_string(), "$700".to_string()]]
    );
}

#[test]
fn row_label_names_kind_question_task_age_and_unread() {
    let open = queue_request("q1", RequestKind::Decision, false);
    assert_eq!(
        row_label(&open),
        "Decision, Question q1?, Refund batch, 12 min ago, unread"
    );
}

fn queue_open_app() -> Element {
    rsx! {
        DecisionQueue {
            request_list: vec![
                queue_request("q1", RequestKind::Decision, false),
                queue_request("q2", RequestKind::Question, false),
            ],
            running_count: 2,
        }
    }
}

fn queue_caught_up_app() -> Element {
    rsx! {
        DecisionQueue {
            request_list: vec![queue_request("q1", RequestKind::Approval, true)],
        }
    }
}

#[test]
fn queue_open_item_shows_its_answer_area_and_the_running_count() {
    let html = render(queue_open_app);
    assert!(html.contains("role=\"listbox\""));
    assert!(html.contains("Refund in full"));
    assert!(html.contains("Recommended"));
    assert!(html.contains("2 tasks still running"));
    assert!(html.contains("Later"));
}

#[test]
fn queue_with_only_answered_items_says_caught_up() {
    let html = render(queue_caught_up_app);
    assert!(html.contains("all caught up."));
    assert!(html.contains("Go to Thread"));
}

use rv2_kit::screens::issue_board_screen::{
    BoardColumn, Issue, IssueBoardScreen, IssueStatus, PriorityTone, SortKey, Target, before_for,
    filtered, move_issue, sorted, step_target,
};

fn board_columns() -> Vec<BoardColumn> {
    vec![
        BoardColumn {
            id: "high".into(),
            column_label: "High".into(),
            priority_tone: PriorityTone::High,
        },
        BoardColumn {
            id: "medium".into(),
            column_label: "Medium".into(),
            priority_tone: PriorityTone::Medium,
        },
        BoardColumn {
            id: "done".into(),
            column_label: "Done".into(),
            priority_tone: PriorityTone::Done,
        },
    ]
}

fn board_issues() -> Vec<Issue> {
    let issue =
        |id: &str, column: &str, title: &str, status: Option<IssueStatus>, who: Option<&str>| {
            Issue {
                id: id.into(),
                column_id: column.into(),
                title: title.into(),
                description: None,
                issue_status: status,
                comment_count: 2,
                attachment_count: 0,
                assignee: who.map(str::to_string),
                labels: Vec::new(),
            }
        };
    vec![
        issue(
            "i1",
            "high",
            "Refund webhook fails",
            Some(IssueStatus::Attention),
            Some("Ada"),
        ),
        issue(
            "i2",
            "high",
            "Invoice PDF layout",
            Some(IssueStatus::Progress),
            Some("Lin"),
        ),
        issue(
            "i3",
            "medium",
            "Onboarding copy",
            Some(IssueStatus::Review),
            None,
        ),
        issue(
            "i4",
            "done",
            "Retire old export",
            Some(IssueStatus::Fixed),
            Some("Ada"),
        ),
    ]
}

#[test]
fn move_places_the_card_before_the_named_card_in_the_new_column() {
    let moved = move_issue(&board_issues(), "i1", "medium", Some("i3"));
    let order: Vec<&str> = moved.iter().map(|i| i.id.as_str()).collect();
    assert_eq!(order, vec!["i2", "i1", "i3", "i4"]);
    assert_eq!(moved[1].column_id, "medium");
    let to_end = move_issue(&board_issues(), "i1", "done", None);
    assert_eq!(to_end.last().map(|i| i.id.as_str()), Some("i1"));
}

#[test]
fn keyboard_steps_change_column_and_position_and_clamp() {
    let columns = board_columns();
    let issues = board_issues();
    let start = Target {
        column_id: "high".into(),
        index: 1,
    };
    let right = step_target(&columns, &issues, &start, 1, 0);
    assert_eq!(
        right,
        Target {
            column_id: "medium".into(),
            index: 1
        }
    );
    let down = step_target(&columns, &issues, &start, 0, 1);
    assert_eq!(
        down,
        Target {
            column_id: "high".into(),
            index: 2
        }
    );
    let left = step_target(
        &columns,
        &issues,
        &Target {
            column_id: "high".into(),
            index: 0,
        },
        -1,
        0,
    );
    assert_eq!(left.column_id, "high");
    assert_eq!(
        before_for(
            &issues,
            "i1",
            &Target {
                column_id: "high".into(),
                index: 0
            }
        ),
        Some("i2".into())
    );
}

#[test]
fn list_view_filters_by_status_and_sorts_by_priority_then_title() {
    let columns = board_columns();
    let issues = board_issues();
    let ada = filtered(&issues, None, Some("Ada"));
    assert_eq!(ada.len(), 2);
    let by_priority: Vec<String> = sorted(&issues, &columns, SortKey::Priority, false)
        .into_iter()
        .map(|i| i.id)
        .collect();
    assert_eq!(by_priority, vec!["i1", "i2", "i3", "i4"]);
    let by_title: Vec<String> = sorted(&issues, &columns, SortKey::Title, true)
        .into_iter()
        .map(|i| i.title)
        .collect();
    assert_eq!(
        by_title.first().map(String::as_str),
        Some("Retire old export")
    );
}

fn board_loaded_app() -> Element {
    rsx! {
        IssueBoardScreen {
            board_columns: board_columns(),
            issues: board_issues(),
        }
    }
}

fn board_empty_app() -> Element {
    rsx! { IssueBoardScreen { board_columns: board_columns(), is_empty: true } }
}

fn board_loading_app() -> Element {
    rsx! { IssueBoardScreen { board_columns: board_columns(), is_loading: true } }
}

#[test]
fn board_names_each_column_and_card_with_its_status_word() {
    let html = render(board_loaded_app);
    assert!(html.contains("aria-label=\"Refund webhook fails, Needs attention, High\""));
    assert!(html.contains("New issue in Medium"));
    assert!(html.contains("kit-ib__col"));
}

#[test]
fn board_empty_offers_one_new_issue_action() {
    let html = render(board_empty_app);
    assert!(html.contains("No tasks yet"));
    assert!(!html.contains("kit-ib__col"));
}

#[test]
fn board_loading_shows_placeholder_cards_under_real_headings() {
    let html = render(board_loading_app);
    assert!(html.contains("kit-ib__ph--card"));
    assert!(html.contains("High"));
    assert!(html.contains("aria-busy=\"true\""));
}

use rv2_kit::screens::automation_canvas::{
    AutomationCanvas, CanvasEdge, CanvasNode, NodeKind, NodeStatus, Port, StepResult, arrow_target,
    failing_step, reading_order, run_order, spoken_duration,
};
use rv2_kit::screens::revenue_chat_analyst::{
    AnswerBody, ChartBlock, FocusRange, MetricCard, RevenueChatAnalyst, SplitGroup, SplitPart,
    Turn, clamp_range, forecast_series, line_focus, split_total,
};
fn canvas_node(id: &str, kind: NodeKind, x: f64, y: f64) -> CanvasNode {
    CanvasNode {
        id: id.into(),
        node_type: kind,
        node_title: format!("Step {id}"),
        subtitle: None,
        pos_x: x,
        pos_y: y,
        node_status: NodeStatus::Ready,
    }
}

fn canvas_edge(id: &str, from: &str, port: Port, to: &str) -> CanvasEdge {
    CanvasEdge {
        id: id.into(),
        source_node: from.into(),
        source_port: port,
        target_node: to.into(),
    }
}

fn canvas_flow() -> (Vec<CanvasNode>, Vec<CanvasEdge>) {
    (
        vec![
            canvas_node("t", NodeKind::Trigger, 40.0, 40.0),
            canvas_node("b", NodeKind::Logic, 40.0, 160.0),
            canvas_node("yes", NodeKind::Action, 0.0, 280.0),
            canvas_node("no", NodeKind::Action, 240.0, 280.0),
        ],
        vec![
            canvas_edge("e1", "t", Port::Out, "b"),
            canvas_edge("e2", "b", Port::Yes, "yes"),
            canvas_edge("e3", "b", Port::No, "no"),
        ],
    )
}

#[test]
fn run_follows_links_from_the_trigger_and_stops_at_the_first_error() {
    let (nodes, edges) = canvas_flow();
    let order = run_order(&nodes, &edges);
    assert_eq!(order, vec!["t", "b", "yes", "no"]);
    let mut results = BTreeMap::new();
    results.insert(
        "b".to_string(),
        StepResult {
            duration: Some(1.0),
            credit_cost: None,
            error_message: None,
        },
    );
    results.insert(
        "yes".to_string(),
        StepResult {
            duration: None,
            credit_cost: None,
            error_message: Some("Mail server refused"),
        },
    );
    assert_eq!(failing_step(&order, &results), Some("yes".to_string()));
}

#[test]
fn arrows_follow_links_and_move_by_row() {
    let (nodes, edges) = canvas_flow();
    assert_eq!(
        arrow_target(&nodes, &edges, "t", "down"),
        Some("b".to_string())
    );
    assert_eq!(
        arrow_target(&nodes, &edges, "b", "up"),
        Some("t".to_string())
    );
    assert_eq!(
        arrow_target(&nodes, &edges, "yes", "right"),
        Some("no".to_string())
    );
    assert_eq!(arrow_target(&nodes, &edges, "no", "right"), None);
    assert_eq!(reading_order(&nodes).first().map(String::as_str), Some("t"));
    assert_eq!(reading_order(&nodes).last().map(String::as_str), Some("no"));
}

#[test]
fn spoken_durations_name_minutes_and_seconds() {
    assert_eq!(spoken_duration(45.0), "45 seconds");
    assert_eq!(spoken_duration(125.0), "2 minutes 5 seconds");
}

fn canvas_loaded_app() -> Element {
    let (nodes, edges) = canvas_flow();
    rsx! { AutomationCanvas { canvas_nodes: nodes, canvas_edges: edges } }
}

fn canvas_empty_app() -> Element {
    rsx! { AutomationCanvas { is_empty: true } }
}

fn canvas_loading_app() -> Element {
    let (nodes, edges) = canvas_flow();
    rsx! { AutomationCanvas { canvas_nodes: nodes, canvas_edges: edges, is_loading: true } }
}

#[test]
fn canvas_names_steps_with_kind_and_status_and_offers_the_run_control() {
    let html = render(canvas_loaded_app);
    assert!(html.contains("aria-label=\"Step t, Trigger, Ready\""));
    assert!(html.contains("Start test"));
    assert!(html.contains("Production"));
    assert!(html.contains("Select a step to configure it."));
}

#[test]
fn canvas_empty_offers_add_a_trigger_and_loading_disables_the_run() {
    let empty = render(canvas_empty_app);
    assert!(empty.contains("Add a trigger"));
    let loading = render(canvas_loading_app);
    assert!(loading.contains("aria-busy=\"true\""));
    assert!(loading.contains("kit-ac__ph"));
}

fn chat_trend() -> ChartBlock {
    ChartBlock::Trend {
        category_labels: vec!["Mon".into(), "Tue".into(), "Wed".into()],
        current_values: vec![10.0, 12.0, 15.0],
        previous_values: vec![8.0, 9.0, 11.0],
        marker: Some("Up 20% on last year".into()),
    }
}

fn chat_loaded_app() -> Element {
    rsx! {
        RevenueChatAnalyst {
            turn_list: vec![
                Turn::Question { id: "q1".into(), text: "How did revenue move?".into(), quoted_focus: None },
                Turn::Answer {
                    id: "a1".into(),
                    body: AnswerBody {
                        text: "Revenue rose this week.".into(),
                        metric_cards: vec![MetricCard { entry_label: "Revenue".into(), value: 48_210.0, change_text: Some("+12%".into()) }],
                        chart_block: Some(chat_trend()),
                        caption_text: Some("Daily revenue".into()),
                        source_list: vec!["Ledger".into()],
                        follow_up_list: vec!["Split by region".into()],
                    },
                },
            ],
        }
    }
}

fn chat_empty_app() -> Element {
    rsx! { RevenueChatAnalyst { is_empty: true } }
}

#[test]
fn chat_answer_shows_metric_chart_sources_and_follow_ups() {
    let html = render(chat_loaded_app);
    assert!(html.contains("48.2k"));
    assert!(html.contains("Revenue rose this week."));
    assert!(html.contains("Sources: Ledger"));
    assert!(html.contains("Split by region"));
    assert!(html.contains("kit-trend"));
}

#[test]
fn chat_empty_offers_starter_questions() {
    let html = render(chat_empty_app);
    assert!(html.contains("Ask about revenue"));
    assert!(html.contains("What will next month look like?"));
}

#[test]
fn forecast_continuation_joins_the_last_actual_point() {
    let (actual, continuation) = forecast_series(&[1.0, 2.0], &[3.0, 4.0]);
    assert_eq!(actual, vec![Some(1.0), Some(2.0), None, None]);
    assert_eq!(continuation, vec![None, Some(2.0), Some(3.0), Some(4.0)]);
}

#[test]
fn split_parts_add_to_the_column_total_and_picks_clamp() {
    let group = SplitGroup {
        entry_name: "EMEA".into(),
        parts: vec![
            SplitPart {
                entry_name: "Web".into(),
                entry_value: 120.0,
            },
            SplitPart {
                entry_name: "Partners".into(),
                entry_value: 80.0,
            },
            SplitPart {
                entry_name: "Refunds".into(),
                entry_value: -5.0,
            },
        ],
    };
    assert_eq!(split_total(&group), 200.0);
    assert_eq!(clamp_range(0, 9, 3), (0, 2));
    let labels: Vec<String> = vec!["Mon".into(), "Tue".into(), "Wed".into()];
    let picked: FocusRange = line_focus("trend", 2, 0, &labels);
    assert_eq!(picked.label_text, "Mon to Wed");
    assert_eq!(picked.detail_text, "3 points");
}

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn account() -> Account {
    Account {
        name: "Ada Lovelace".into(),
        avatar_url: None,
    }
}

fn figures() -> Vec<KeyFigure> {
    vec![
        KeyFigure {
            item_name: "Revenue".into(),
            item_value: "$48,210".into(),
            change_text: "12.4% vs last period".into(),
            change_sign: ChangeSign::Up,
            good_when: ChangeSign::Up,
            trend_bars: vec![3.0, 5.0, 4.0, 8.0],
        },
        KeyFigure {
            item_name: "Refund rate".into(),
            item_value: "2.1%".into(),
            change_text: "0.4 pts vs last period".into(),
            change_sign: ChangeSign::Up,
            good_when: ChangeSign::Down,
            trend_bars: vec![1.0, 2.0],
        },
    ]
}

fn channels() -> Vec<ChannelSeries> {
    vec![
        ChannelSeries {
            name: "Web".into(),
            daily: vec![100.0, 120.0, 90.0],
        },
        ChannelSeries {
            name: "Partners".into(),
            daily: vec![40.0, 35.0, 50.0],
        },
    ]
}

#[test]
fn change_tone_follows_good_when() {
    assert_eq!(
        change_tone(ChangeSign::Up, ChangeSign::Up).key(),
        "positive"
    );
    assert_eq!(
        change_tone(ChangeSign::Up, ChangeSign::Down).key(),
        "negative"
    );
    assert_eq!(
        change_tone(ChangeSign::Down, ChangeSign::Down).key(),
        "positive"
    );
    assert_eq!(
        change_tone(ChangeSign::Flat, ChangeSign::Up).key(),
        "neutral"
    );
}

#[test]
fn bars_scale_to_the_largest_value() {
    assert_eq!(bar_percentages(&[2.0, 4.0, 1.0]), vec![50.0, 100.0, 25.0]);
    assert_eq!(bar_percentages(&[0.0, 0.0]), vec![0.0, 0.0]);
    assert_eq!(bar_percentages(&[f64::NAN, 3.0]), vec![0.0, 100.0]);
}

#[test]
fn share_is_clamped_to_the_track() {
    assert_eq!(share_width(50.0), 50.0);
    assert_eq!(share_width(140.0), 100.0);
    assert_eq!(share_width(-3.0), 0.0);
    assert_eq!(share_width(f64::NAN), 0.0);
}

#[test]
fn totals_group_digits_and_means_average_the_days() {
    assert_eq!(format_total(48_210.4), "48,210");
    assert_eq!(format_total(-1_234_567.0), "-1,234,567");
    assert_eq!(format_total(999.0), "999");
    assert!((daily_mean(&[1.0, 2.0, 3.0]) - 2.0).abs() < 1e-9);
    assert_eq!(daily_mean(&[]), 0.0);
}

#[test]
fn sparkline_is_flat_for_constant_values() {
    assert_eq!(
        sparkline_points(&[5.0, 5.0], 10.0, 10.0),
        "0.0,5.0 10.0,5.0"
    );
    assert_eq!(
        sparkline_points(&[0.0, 10.0], 10.0, 10.0),
        "0.0,10.0 10.0,0.0"
    );
}

fn revenue_loaded() -> Element {
    rsx! {
        RevenueOverview {
            account: account(),
            key_figures: figures(),
            day_labels: vec!["Mon".into(), "Tue".into(), "Wed".into()],
            channel_series: channels(),
            chart_total: "$48,210",
            chart_change_text: "12.4%",
            chart_change_sign: ChangeSign::Up,
            breakdown_rows: vec![BreakdownRow {
                item_name: "Subscriptions".into(),
                item_value: "$31,000".into(),
                trend_points: vec![1.0, 2.0, 3.0],
                trend_direction: ChangeSign::Up,
            }],
            ranked_items: vec![
                RankedItem { item_name: "Pro plan".into(), item_value: "$20,000".into(), share_pct: 50.0 },
                RankedItem { item_name: "Team add-on".into(), item_value: "$4,000".into(), share_pct: 10.0 },
            ],
            sessions_series: vec![
                SessionsSeries { item_name: SessionPeriod::This, daily: vec![3.0, 4.0] },
                SessionsSeries { item_name: SessionPeriod::Previous, daily: vec![2.0, 2.0] },
            ],
            average_value: Some(AverageValue {
                current: "$42.10".into(),
                previous: "$39.80".into(),
                daily: vec![40.0, 44.0, 42.0],
            }),
        }
    }
}

fn revenue_loaded_app() -> Element {
    revenue_loaded()
}

fn revenue_empty_app() -> Element {
    rsx! { RevenueOverview { account: account(), is_empty: true } }
}

fn revenue_loading_app() -> Element {
    rsx! { RevenueOverview { account: account(), is_loading: true } }
}

#[test]
fn revenue_loaded_names_figures_tones_and_sign_words() {
    let html = render(revenue_loaded_app);
    assert!(html.contains("aria-labelledby"));
    assert!(html.contains("Revenue overview"));
    assert!(html.contains("data-tone=\"positive\""));
    assert!(html.contains("data-tone=\"negative\""));
    assert!(html.contains("Up"));
    assert!(html.contains("$48,210"));
}

#[test]
fn revenue_ranked_share_bar_draws_at_its_percentage() {
    let html = render(revenue_loaded_app);
    assert!(html.contains("width: 50.0%;"));
    assert!(html.contains("width: 10.0%;"));
}

#[test]
fn revenue_chart_has_a_keyboard_day_cursor_and_sessions_totals() {
    let html = render(revenue_loaded_app);
    assert!(html.contains("kit-trend"));
    assert!(html.contains("Sessions"));
    assert!(html.contains("This period: "));
    assert!(html.contains("Previous period: "));
}

#[test]
fn revenue_empty_hides_figures_charts_and_lists() {
    let html = render(revenue_empty_app);
    assert!(html.contains("No revenue yet"));
    assert!(html.contains("Connect a payment account"));
    assert!(!html.contains("kit-rev__figure-value"));
    assert!(!html.contains("kit-trend"));
    assert!(!html.contains("Breakdown"));
    assert!(html.contains("Save as report"));
}

#[test]
fn revenue_loading_keeps_titles_and_marks_busy() {
    let html = render(revenue_loading_app);
    assert!(html.contains("aria-busy=\"true\""));
    assert!(html.contains("Breakdown"));
    assert!(html.contains("kit-rev__ph--big"));
    assert!(!html.contains("kit-rev__figure-value"));
}

#[test]
fn tablist_keys_move_the_filter_and_wrap() {
    use rv2_kit::screens::prompt_test_board::filter_after_key;
    assert_eq!(
        filter_after_key(CaseFilter::All, "ArrowRight"),
        Some(CaseFilter::Pass)
    );
    assert_eq!(
        filter_after_key(CaseFilter::All, "ArrowLeft"),
        Some(CaseFilter::Fail)
    );
    assert_eq!(
        filter_after_key(CaseFilter::Fail, "ArrowRight"),
        Some(CaseFilter::All)
    );
    assert_eq!(
        filter_after_key(CaseFilter::Partial, "End"),
        Some(CaseFilter::Fail)
    );
    assert_eq!(
        filter_after_key(CaseFilter::Partial, "Home"),
        Some(CaseFilter::All)
    );
    assert_eq!(filter_after_key(CaseFilter::All, "Enter"), None);
}

#[test]
fn sheet_tab_moves_right_and_wraps_to_the_next_row() {
    use rv2_kit::components::sheet_grid::{Cell, tab_cell};
    let at = |row, col| Cell { row, col };
    assert_eq!(tab_cell(at(0, 0), 3, 2, false), Some(at(0, 1)));
    assert_eq!(tab_cell(at(0, 1), 3, 2, false), Some(at(1, 0)));
    assert_eq!(tab_cell(at(2, 1), 3, 2, false), None);
    assert_eq!(tab_cell(at(1, 0), 3, 2, true), Some(at(0, 1)));
    assert_eq!(tab_cell(at(0, 0), 3, 2, true), None);
    assert_eq!(tab_cell(at(0, 0), 0, 2, false), None);
}
