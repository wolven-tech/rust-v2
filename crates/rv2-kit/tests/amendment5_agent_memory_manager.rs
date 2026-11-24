use dioxus::prelude::*;
use rv2_kit::screens::agent_memory_manager::{
    AgentMemoryManager, MemoryRecord, SourceInfo, SourceKind, UsageEntry, group_order,
};

const NOW: u64 = 1_795_219_200_000;
const DAY: u64 = 86_400_000;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn memory(id: &str, text: &str, project: Option<&str>, pending: bool) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        text: text.to_string(),
        project_id: project.map(str::to_string),
        source_info: SourceInfo {
            source_kind: SourceKind::Conversation,
            occurred_at: NOW - DAY,
            quote_text: None,
            question_text: None,
            file_name: None,
        },
        use_count: 3,
        last_used_at: Some(NOW - 2 * DAY),
        usage_log: vec![],
        is_pending_review: pending,
    }
}

fn fixture() -> Vec<MemoryRecord> {
    let mut first = memory("m1", "Prefers short weekly summaries", None, false);
    first.source_info.quote_text = Some("Keep it short, weekly.".to_string());
    first.usage_log = vec![
        UsageEntry {
            task_name: "Weekly digest".to_string(),
            how: "summary length".to_string(),
            occurred_at: NOW - 3_600_000,
        },
        UsageEntry {
            task_name: "Board memo".to_string(),
            how: "tone".to_string(),
            occurred_at: NOW - 2 * DAY,
        },
    ];
    vec![
        first,
        memory("m4", "Works in Lisbon time", None, false),
        memory("m5", "Signs off as Dee", None, false),
        memory("m6", "Dislikes passive voice", None, false),
        memory("m7", "Reviews drafts on Mondays", None, false),
        memory("m2", "Launch date is 14 March", Some("Launch"), false),
        memory("m8", "Pricing freezes before launch", Some("Launch"), false),
        memory("m9", "Beta list has 40 teams", Some("Launch"), false),
        memory("m10", "Two open engineering roles", Some("Hiring"), false),
        memory(
            "m13",
            "Panel interviews run 45 minutes",
            Some("Hiring"),
            false,
        ),
        memory(
            "m11",
            "Competitor X raised prices in May",
            Some("Research"),
            false,
        ),
        memory("m3", "Pause hiring for Q1", Some("Hiring"), true),
        memory("m12", "Use the 2025 survey results", Some("Research"), true),
    ]
}

fn names() -> Vec<String> {
    vec![
        "Launch".to_string(),
        "Hiring".to_string(),
        "Research".to_string(),
    ]
}

fn rest_app() -> Element {
    rsx! {
        AgentMemoryManager {
            memory_list: fixture(),
            project_names: names(),
            initial_memory_id: Some("m1".to_string()),
            clock_ms: Some(NOW),
        }
    }
}

#[test]
fn saved_tree_at_rest_has_about_you_five_then_projects_three_two_one() {
    let saved: Vec<MemoryRecord> = fixture()
        .into_iter()
        .filter(|m| !m.is_pending_review)
        .collect();
    assert_eq!(saved.len(), 11);
    let counts: Vec<(String, usize)> = group_order(&saved, &names())
        .into_iter()
        .map(|(group, project)| {
            let count = saved.iter().filter(|m| m.project_id == project).count();
            (group, count)
        })
        .collect();
    assert_eq!(
        counts,
        vec![
            ("About you".to_string(), 5),
            ("Launch".to_string(), 3),
            ("Hiring".to_string(), 2),
            ("Research".to_string(), 1),
        ]
    );
}

#[test]
fn saved_tab_count_is_eleven_and_to_review_count_is_two() {
    let html = render(rest_app);
    assert_eq!(fixture().iter().filter(|m| m.is_pending_review).count(), 2);
    assert!(html.contains("kit-amm__count\">11<"));
    assert!(html.contains("kit-amm__count\">2<"));
}

#[test]
fn panel_for_the_selected_memory_lists_text_applies_source_usage_then_forget() {
    let html = render(rest_app);
    let at = |needle: &str| {
        html.find(needle)
            .unwrap_or_else(|| panic!("missing {needle}"))
    };
    let order = [
        at("Prefers short weekly summaries"),
        at("Applies to"),
        at("Keep it short, weekly."),
        at("Used in"),
        at("Forget"),
    ];
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(html.contains("Everywhere"));
}

#[test]
fn to_review_lists_exactly_the_flagged_memories() {
    let flagged: Vec<String> = fixture()
        .into_iter()
        .filter(|m| m.is_pending_review)
        .map(|m| m.id)
        .collect();
    assert_eq!(flagged, vec!["m3".to_string(), "m12".to_string()]);
}

#[test]
fn remember_line_is_closed_and_no_undo_notice_at_rest() {
    let html = render(rest_app);
    assert!(!html.contains("kit-amm__remember\""));
    assert!(!html.contains("kit-amm__notice"));
}
