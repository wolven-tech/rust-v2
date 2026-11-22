use dioxus::prelude::*;
use rv2_kit::components::assistant_memory_manager::{AssistantMemoryManager, Memory};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn memory(id: &str, text: &str, group: &str, paused: bool) -> Memory {
    Memory {
        memory_id: id.into(),
        memory_text: text.into(),
        memory_group: group.into(),
        source_label: Some("Chat".into()),
        date_label: Some("Mar 3".into()),
        is_paused: paused,
    }
}

fn six() -> Vec<Memory> {
    vec![
        memory("m1", "Prefers metric units", "Preferences", false),
        memory("m2", "Works in Berlin time", "Preferences", false),
        memory(
            "m3",
            "Writes status updates on Fridays",
            "Preferences",
            true,
        ),
        memory("m4", "Project Atlas launches 14 June", "Projects", false),
        memory("m5", "Budget cap for Atlas", "Projects", false),
        memory("m6", "Dana reports to Priya Raman", "People", false),
    ]
}

fn panel() -> String {
    render(|| {
        rsx! {
            AssistantMemoryManager {
                memory_items: six(),
                group_order: vec!["Preferences".to_string(), "Projects".to_string(), "People".to_string()],
            }
        }
    })
}

#[test]
fn every_memory_is_rendered_as_a_row_and_the_all_chip_counts_them() {
    let html = panel();
    assert_eq!(html.matches("data-memory-id=").count(), 6);
    assert!(html.contains("All 6"));
    assert!(html.contains("6 memories · 1 paused"));
}

#[test]
fn each_group_chip_count_matches_the_rows_under_its_heading() {
    let html = panel();
    assert!(html.contains("Preferences 3"));
    assert!(html.contains("Projects 2"));
    assert!(html.contains("People 1"));
    let people = html.find("aria-label=\"People\"").expect("People region");
    assert!(html[people..].contains("Dana reports to Priya Raman"));
    assert_eq!(html.matches("data-memory-id=").count(), 3 + 2 + 1);
}

#[test]
fn the_list_is_a_scroll_region_with_a_scroll_cue() {
    let css = include_str!("../assets/assistant-memory-manager.css");
    let rule = css
        .split(".kit-memory__list {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("list rule");
    assert!(rule.contains("overflow-y: auto"));
    assert!(rule.contains("background-attachment: local, local, scroll, scroll"));
}
