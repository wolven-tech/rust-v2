use dioxus::prelude::*;
use rv2_kit::screens::agent_skill_library::{AgentSkillLibrary, SkillRecord, SkillSource};

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
        trigger_phrases: vec!["weekly report".to_string()],
        step_texts: vec![
            "Collect figures".to_string(),
            "Write the summary".to_string(),
        ],
        requirement_list: vec![],
        usage_log: vec![],
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
        }
    }
}

#[test]
fn installed_section_has_nine_cards_and_discover_has_six_at_rest() {
    let skills = fixture();
    assert_eq!(skills.iter().filter(|s| s.is_installed).count(), 9);
    assert_eq!(skills.iter().filter(|s| !s.is_installed).count(), 6);
    let html = render(rest_app);
    assert_eq!(html.matches("class=\"kit-asl__card\"").count(), 9);
    assert_eq!(html.matches("kit-asl__card--discover").count(), 6);
}

#[test]
fn installed_cards_show_source_and_discover_cards_show_install_button() {
    let html = render(rest_app);
    assert!(html.contains("Built-in"));
    assert!(html.contains("Own"));
    assert!(html.contains("@lin"));
    assert_eq!(html.matches(">Install<").count(), 6);
}

#[test]
fn category_row_shows_all_and_five_categories() {
    let html = render(rest_app);
    for label in ["All", "Ops", "Sales", "Support", "Finance", "Research"] {
        assert!(html.contains(label), "missing chip {label}");
    }
}

#[test]
fn only_one_installed_skill_is_disabled() {
    let skills = fixture();
    assert_eq!(
        skills
            .iter()
            .filter(|s| s.is_installed && !s.is_enabled)
            .count(),
        1
    );
}
