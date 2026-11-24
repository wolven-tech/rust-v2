use dioxus::prelude::*;
use rv2_kit::screens::agent_skill_library::{
    AgentSkillLibrary, Requirement, SkillRecord, SkillSource, SkillUsage,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn skill(
    id: &str,
    name: &str,
    category: &str,
    installed: bool,
    source: SkillSource,
    author: Option<&str>,
) -> SkillRecord {
    SkillRecord {
        id: id.to_string(),
        entry_name: name.to_string(),
        entry_description: format!("{name} for weekly work"),
        category_name: category.to_string(),
        source_kind: source,
        author_name: author.map(str::to_string),
        version_label: Some("1.2".to_string()),
        is_installed: installed,
        is_enabled: installed,
        weekly_use_count: installed.then_some(9),
        install_count: (!installed).then_some(1200),
        trigger_phrases: vec!["customer asks for a digest".to_string()],
        step_texts: vec!["Collect the week's tickets".to_string()],
        requirement_list: vec![Requirement {
            entry_label: "Read the ticket queue".to_string(),
            permissions_link: true,
        }],
        usage_log: vec![SkillUsage {
            task_name: "Monday digest".to_string(),
            occurred_at: 0,
        }],
        daily_run_counts: vec![1, 0, 2, 3, 1, 0, 2, 2, 4, 1, 0, 3, 2, 1],
    }
}

fn fixture() -> Vec<SkillRecord> {
    use SkillSource::{BuiltIn, Community, Own};
    let mut skills = vec![
        skill("w", "Weekly report", "Ops", true, BuiltIn, None),
        skill("c", "Customer digest", "Support", true, Own, None),
        skill(
            "d",
            "Invoice check",
            "Finance",
            true,
            Community,
            Some("@lin"),
        ),
        skill("e", "Lead scoring", "Sales", true, BuiltIn, None),
        skill("f", "Refund triage", "Support", true, Own, None),
        skill("g", "Board pack", "Finance", true, BuiltIn, None),
        skill(
            "h",
            "Competitor brief",
            "Research",
            true,
            Community,
            Some("@research_crew"),
        ),
        skill("i", "Expense sort", "Finance", true, BuiltIn, None),
        skill("j", "Meeting notes", "Ops", true, BuiltIn, None),
        skill(
            "k",
            "Churn alert",
            "Sales",
            false,
            Community,
            Some("@sales_ops"),
        ),
        skill("l", "Ticket macros", "Support", false, BuiltIn, None),
        skill(
            "m",
            "Forecast sheet",
            "Finance",
            false,
            Community,
            Some("@cfo_kit"),
        ),
        skill("n", "Survey readout", "Research", false, BuiltIn, None),
        skill(
            "o",
            "Onboarding email",
            "Sales",
            false,
            Community,
            Some("@growth"),
        ),
        skill("p", "Stand-up summary", "Ops", false, BuiltIn, None),
    ];
    skills[7].is_enabled = false;
    skills
}

fn rest_app() -> Element {
    rsx! {
        AgentSkillLibrary {
            skill_records: fixture(),
            category_names: vec![
                "Ops".to_string(),
                "Sales".to_string(),
                "Support".to_string(),
                "Finance".to_string(),
                "Research".to_string(),
            ],
            initial_skill_id: Some("c".to_string()),
        }
    }
}

#[test]
fn search_field_sits_on_the_header_row_beside_new_skill_and_the_chips_below_it() {
    let html = render(rest_app);
    let head = html
        .find("class=\"kit-asl__head-actions\"")
        .expect("header actions");
    let search = html.find("class=\"kit-asl__search\"").expect("search");
    let new_skill = html.find(">New skill<").expect("new skill");
    let filters = html.find("class=\"kit-asl__filters\"").expect("filter row");
    let chips = html.find("class=\"kit-asl__chips\"").expect("chips");
    assert!(head < search && search < new_skill && new_skill < filters);
    assert!(filters < chips);
}

#[test]
fn installed_heading_counts_nine_and_discover_heading_counts_six_with_a_line_under_it() {
    let html = render(rest_app);
    assert!(html.contains("Installed · 9"));
    assert!(html.contains("Discover · 6"));
    assert!(html.contains("Skills from the catalogue that are not installed yet."));
    assert_eq!(html.matches("class=\"kit-asl__grid\"").count(), 2);
}

#[test]
fn installed_cards_have_no_weekly_count_and_discover_cards_show_a_use_count() {
    let html = render(rest_app);
    assert_eq!(html.matches("9 this week").count(), 0);
    assert_eq!(html.matches("1200 use it").count(), 6);
    assert_eq!(html.matches(">Install<").count(), 6);
}

#[test]
fn one_built_in_card_is_off_and_stays_listed_at_rest() {
    let html = render(rest_app);
    assert_eq!(html.matches("data-enabled=\"false\"").count(), 1);
    assert_eq!(html.matches("class=\"kit-asl__card\"").count(), 9);
}

#[test]
fn panel_is_open_on_the_first_own_skill_with_its_sections_in_order() {
    let html = render(rest_app);
    assert!(html.contains("aria-label=\"Skill: Customer digest\""));
    let header = html.find("kit-asl__panel-title").expect("header");
    let actions = html.find(">Turn off<").expect("actions");
    let runs = html.find("Last 14 days").expect("runs chart");
    let triggers = html.find("When it applies").expect("trigger phrases");
    let steps = html.find(">Steps<").expect("steps");
    let needs = html.find(">Needs<").expect("requirements");
    let recent = html.find("Recent use").expect("recent usage");
    assert!(header < actions && actions < runs && runs < triggers);
    assert!(triggers < steps && steps < needs && needs < recent);
}

#[test]
fn panel_shows_a_remove_control_for_own_and_handle_skills_only() {
    let html = render(rest_app);
    assert_eq!(html.matches(">Remove<").count(), 1);
}

#[test]
fn panel_runs_chart_has_fourteen_bars_and_a_count_for_the_week() {
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-asl__bar\"").count(), 14);
    assert!(html.contains("13 runs this week"));
}

#[test]
fn panel_requirements_each_link_to_permissions() {
    let html = render(rest_app);
    assert_eq!(html.matches(">Permissions<").count(), 1);
}
