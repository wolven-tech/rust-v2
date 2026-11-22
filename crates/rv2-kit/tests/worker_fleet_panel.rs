use dioxus::prelude::*;
use rv2_kit::agents::worker_fleet_panel::{
    FleetState, LeadAgent, RespondAction, SubAgent, WorkItem, WorkState, WorkerFleetPanel,
    current_step, elapsed_of, item_word,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn item(label: &str, state: Option<WorkState>, secs: Option<u64>) -> WorkItem {
    WorkItem {
        label: label.to_string(),
        state,
        tool_name: None,
        duration: secs.map(|s| s * 1_000),
    }
}

fn sub(
    id: &str,
    name: &str,
    state: FleetState,
    work_items: Vec<WorkItem>,
    question: Option<&str>,
) -> SubAgent {
    SubAgent {
        id: id.to_string(),
        name: name.to_string(),
        assignment: format!("{name} task"),
        model_label: None,
        state,
        work_items,
        spend: Some(0.05),
        question: question.map(str::to_string),
        failure_text: None,
        children: Vec::new(),
    }
}

fn lead(id: &str, name: &str, state: FleetState, subs: Vec<SubAgent>) -> LeadAgent {
    LeadAgent {
        id: id.to_string(),
        name: name.to_string(),
        assignment: format!("{name} assignment"),
        model_label: None,
        picture: None,
        tool_marks: Vec::new(),
        state,
        spend: Some(0.1),
        started_at_ms: None,
        ended_at_ms: None,
        sub_agents: subs,
    }
}

/// Every sub-agent in the fleet, nested ones included.
fn count_subs(subs: &[SubAgent]) -> usize {
    subs.iter().map(|s| 1 + count_subs(&s.children)).sum()
}

/// The gallery's fleet: four leads, nine sub-agents, Research lead open.
fn gallery_fleet() -> Vec<LeadAgent> {
    let mut ad_copy = sub(
        "s4",
        "Ad copy",
        FleetState::Failed,
        vec![item("Draft headlines", Some(WorkState::Failed), Some(21))],
        None,
    );
    ad_copy.failure_text = Some("Rate limit hit after 3 retries".to_string());
    let mut pricing = sub(
        "s1",
        "Pricing",
        FleetState::Running,
        vec![
            item("Pull price pages", None, Some(38)),
            item("Compare tiers", Some(WorkState::Running), Some(74)),
            item("Write price table", Some(WorkState::Queued), None),
        ],
        None,
    );
    pricing.children = vec![ad_copy];
    vec![
        lead(
            "lead-1",
            "Research lead",
            FleetState::Running,
            vec![
                pricing,
                sub(
                    "s2",
                    "Reviews",
                    FleetState::Waiting,
                    vec![item(
                        "Pull app store reviews",
                        Some(WorkState::Waiting),
                        Some(52),
                    )],
                    Some("Include app store reviews?"),
                ),
                sub(
                    "s3",
                    "Traffic",
                    FleetState::Done,
                    vec![item("Read export", None, Some(96))],
                    None,
                ),
            ],
        ),
        lead(
            "lead-2",
            "Writer lead",
            FleetState::Queued,
            vec![
                sub("s6", "Outline", FleetState::Queued, Vec::new(), None),
                sub("s7", "Draft", FleetState::Queued, Vec::new(), None),
            ],
        ),
        lead(
            "lead-3",
            "Data lead",
            FleetState::Done,
            vec![sub(
                "s5",
                "Dedupe",
                FleetState::Done,
                vec![item("Merge", None, Some(12))],
                None,
            )],
        ),
        lead(
            "lead-4",
            "Ops lead",
            FleetState::Running,
            vec![
                sub(
                    "s8",
                    "Monitor",
                    FleetState::Running,
                    vec![item("Check uptime", Some(WorkState::Running), Some(9))],
                    None,
                ),
                sub("s9", "Alerts", FleetState::Queued, Vec::new(), None),
            ],
        ),
    ]
}

fn gallery_app() -> Element {
    rsx! {
        WorkerFleetPanel {
            lead_agents: gallery_fleet(),
            expanded_id: Some("lead-1".to_string()),
            clock_now_ms: Some(0u64),
            on_respond: move |_: (SubAgent, RespondAction)| {},
        }
    }
}

fn gallery_without_handler_app() -> Element {
    rsx! {
        WorkerFleetPanel {
            lead_agents: gallery_fleet(),
            expanded_id: Some("lead-1".to_string()),
            clock_now_ms: Some(0u64),
        }
    }
}

fn gallery_window_app() -> Element {
    rsx! {
        WorkerFleetPanel {
            lead_agents: gallery_fleet(),
            expanded_id: Some("lead-1".to_string()),
            default_work_open: Some("s2".to_string()),
            clock_now_ms: Some(0u64),
            on_respond: move |_: (SubAgent, RespondAction)| {},
        }
    }
}

#[test]
fn gallery_header_counts_four_agents_and_nine_sub_agents_including_nested() {
    let fleet = gallery_fleet();
    let subs: usize = fleet.iter().map(|l| count_subs(&l.sub_agents)).sum();
    assert_eq!(subs, 9);
    let html = render(gallery_app);
    assert!(html.contains("4 agents"));
    assert!(html.contains("9 sub-agents"));
}

#[test]
fn only_the_open_lead_renders_its_sub_agent_tree() {
    let html = render(gallery_app);
    assert_eq!(html.matches("role=\"treeitem\"").count(), 4);
    assert!(html.contains("aria-label=\"Pricing, Running\""));
    assert!(!html.contains("Outline"));
    assert!(!html.contains("Dedupe"));
    assert!(!html.contains("Monitor"));
}

#[test]
fn research_lead_summary_names_needs_you_before_failed() {
    let html = render(gallery_app);
    assert!(html.contains("kit-wfp__phrase\">Needs you<"));
    assert!(!html.contains("kit-wfp__phrase\">Failed<"));
}

#[test]
fn response_actions_are_offered_only_with_an_on_respond_handler() {
    let with_handler = render(gallery_app);
    assert!(with_handler.contains("data-action=\"Approve\""));
    assert!(with_handler.contains("data-action=\"Retry\""));
    let without = render(gallery_without_handler_app);
    assert!(!without.contains("data-action="));
}

#[test]
fn open_work_window_lists_response_actions_above_its_items() {
    let html = render(gallery_window_app);
    assert!(html.contains("aria-label=\"Work of Reviews\""));
    let actions = html.find("kit-wfp__actions").expect("response actions");
    let items = html.find("kit-wfp__items").expect("work items");
    assert!(actions < items);
    assert!(html.contains("Pull app store reviews"));
}

#[test]
fn a_running_sub_agent_footer_names_its_current_step() {
    let pricing = sub(
        "s1",
        "Pricing",
        FleetState::Running,
        vec![
            item("Pull price pages", None, Some(38)),
            item("Compare tiers", Some(WorkState::Running), Some(74)),
        ],
        None,
    );
    assert_eq!(current_step(&pricing), Some("Compare tiers"));
    assert_eq!(elapsed_of(&pricing), None);
}

#[test]
fn a_finished_sub_agent_footer_shows_the_time_its_work_took() {
    let traffic = sub(
        "s3",
        "Traffic",
        FleetState::Done,
        vec![
            item("Read export", None, Some(60)),
            item("Summarise", None, Some(36)),
        ],
        None,
    );
    assert_eq!(current_step(&traffic), None);
    assert_eq!(elapsed_of(&traffic), Some(96_000));
}

#[test]
fn work_item_without_a_state_reads_as_done() {
    assert_eq!(item_word(None), "Done");
    assert_eq!(item_word(Some(WorkState::Waiting)), "Needs you");
    assert_eq!(item_word(Some(WorkState::Skipped)), "Skipped");
}

#[test]
fn dimmed_lead_returns_to_full_strength_on_hover() {
    let css = include_str!("../assets/worker-fleet-panel.css");
    assert!(css.contains(".kit-wfp__lead[data-dimmed=\"true\"]:hover"));
}

#[test]
fn open_tree_is_joined_to_its_lead_by_connector_lines() {
    let css = include_str!("../assets/worker-fleet-panel.css");
    assert!(css.contains(".kit-wfp__lead[aria-expanded=\"true\"]::after"));
    assert!(css.contains(".kit-wfp__rail::before"));
    assert!(css.contains(".kit-wfp__rail::after"));
}

#[test]
fn four_lead_cards_form_one_column_with_only_the_open_one_at_full_strength() {
    let html = render(gallery_app);
    assert_eq!(html.matches("class=\"kit-wfp__lead\"").count(), 4);
    assert_eq!(html.matches("data-dimmed=\"true\"").count(), 3);
    assert_eq!(html.matches("aria-expanded=\"true\"").count(), 1);
}

#[test]
fn three_direct_cards_hang_from_one_bus_and_ad_copy_hangs_from_pricing() {
    let html = render(gallery_app);
    assert_eq!(html.matches("class=\"kit-wfp__sub\"").count(), 3);
    assert_eq!(html.matches("kit-wfp__sub kit-wfp__sub--nested").count(), 1);
    assert_eq!(html.matches("class=\"kit-wfp__rail\"").count(), 4);
    let pricing = html
        .find("aria-label=\"Pricing, Running\"")
        .expect("pricing");
    let children = html.find("class=\"kit-wfp__children\"").expect("children");
    let ad_copy = html
        .find("aria-label=\"Ad copy, Failed\"")
        .expect("ad copy");
    let reviews = html
        .find("aria-label=\"Reviews, Needs you\"")
        .expect("reviews");
    assert!(pricing < children && children < ad_copy && ad_copy < reviews);
}

#[test]
fn open_tree_has_no_panel_or_subtotal_heading_around_it() {
    let html = render(gallery_app);
    assert!(!html.contains("kit-wfp__tree-head"));
    let css = include_str!("../assets/worker-fleet-panel.css");
    let start = css.find(".kit-wfp__tree {").expect("tree rule");
    let rule = &css[start..start + css[start..].find('}').expect("rule end")];
    assert!(!rule.contains("border"));
    assert!(!rule.contains("background"));
}

#[test]
fn response_buttons_sit_inside_the_footer_of_their_card() {
    let html = render(gallery_app);
    let buttons: Vec<usize> = html
        .match_indices("data-action=")
        .map(|(at, _)| at)
        .collect();
    assert_eq!(buttons.len(), 4);
    for at in buttons {
        let card = html[..at]
            .rfind("kit-wfp__sub-card\"")
            .expect("owning card");
        let footer = html[..at].rfind("kit-wfp__sub-footer\"").expect("footer");
        assert!(card < footer);
    }
}
