use dioxus::prelude::*;
use rv2_kit::components::agent_context_map::{
    AgentContextMap, AgentNode, Memory, SourceArtifact, Tool, replacement_links,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn memory(id: &str, text: &str, outdated: bool, source: (&str, &str)) -> Memory {
    Memory {
        memory_id: id.into(),
        memory_text: text.into(),
        memory_kind: Some("Preference".into()),
        memory_source_line: Some("Learned from you".into()),
        source_artifact: SourceArtifact {
            artifact_name: source.0.into(),
            artifact_kind: Some(source.1.into()),
            artifact_mark: None,
            artifact_link: None,
        },
        learned_via_tool_id: None,
        replaces_memory_id: None,
        is_outdated: outdated,
        related_memory_ids: vec![],
        learned_at: Some(1_700_000_000_000.0),
        use_count: 3,
        is_disabled: false,
    }
}

fn tool(id: &str, name: &str) -> Tool {
    Tool {
        tool_id: id.into(),
        tool_mark: None,
        tool_name: name.into(),
        access_level: "Read".into(),
        scope_text: "Workspace".into(),
        last_activity_text: "2 min ago".into(),
        last_used_at: Some(1_700_000_000_000.0),
        call_count: 7,
        is_disabled: false,
    }
}

fn writer() -> AgentNode {
    AgentNode {
        agent_id: "writer".into(),
        agent_name: "Writer".into(),
        agent_subtitle: "Drafts copy".into(),
        agent_mark: None,
    }
}

fn demo_memories() -> Vec<Memory> {
    let mut replacing = memory(
        "m3",
        "Pricing tier: Team",
        false,
        ("Pricing sheet", "Sheet"),
    );
    replacing.replaces_memory_id = Some("m2".into());
    vec![
        memory(
            "m1",
            "Prefers short sentences",
            false,
            ("Chat, Mar 3", "Chat"),
        ),
        memory("m2", "Old pricing tier", true, ("Pricing sheet", "Sheet")),
        replacing,
        memory("m4", "Signs off as Maya", false, ("Style guide", "Doc")),
    ]
}

fn demo_tools() -> Vec<Tool> {
    vec![
        tool("t1", "Docs search"),
        tool("t2", "Calendar"),
        tool("t3", "Mail"),
        tool("t4", "CRM"),
        tool("t5", "Sheets"),
    ]
}

fn demo() -> Element {
    rsx! {
        AgentContextMap {
            agent_node: writer(),
            memory_items: demo_memories(),
            tool_items: demo_tools(),
            active_tool_ids: vec!["t1".to_string()],
            recalled_memory_ids: vec!["m1".to_string()],
            heading_text: "Writer context",
        }
    }
}

fn idle_demo() -> Element {
    rsx! {
        AgentContextMap {
            agent_node: writer(),
            memory_items: demo_memories(),
            tool_items: demo_tools(),
            heading_text: "Writer context",
        }
    }
}

fn grown_demo() -> Element {
    let mut tools = demo_tools();
    tools.push(tool("t6", "Drive"));
    rsx! {
        AgentContextMap {
            agent_node: writer(),
            memory_items: demo_memories(),
            tool_items: tools,
            active_tool_ids: vec!["t1".to_string()],
            recalled_memory_ids: vec!["m1".to_string()],
            heading_text: "Writer context",
        }
    }
}

#[test]
fn ring_lays_memories_then_agent_then_tools_in_dom_order() {
    let html = render(demo);
    let memories = html
        .find("data-side=\"memory\"")
        .expect("memory column rendered");
    let hub = html.find("kit-actx__hub").expect("agent hub rendered");
    let tools = html
        .find("data-side=\"tool\"")
        .expect("tool column rendered");
    assert!(memories < hub && hub < tools);
}

#[test]
fn header_counts_four_memories_and_five_tools() {
    let html = render(demo);
    assert!(html.contains("4 memories · 5 tools"));
}

#[test]
fn logo_row_shows_five_marks_and_only_the_in_use_one_at_full_strength() {
    let html = render(demo);
    assert_eq!(count(&html, "class=\"kit-actx__logo\""), 5);
    assert_eq!(count(&html, "kit-actx__logo\" data-state=\"active\""), 1);
    assert_eq!(count(&html, "kit-actx__logo\" data-state=\"idle\""), 4);
}

#[test]
fn memory_cards_show_source_line_and_mark_and_tool_cards_show_scope_and_calls() {
    let html = render(demo);
    assert_eq!(count(&html, "class=\"kit-actx__source\">Chat, Mar 3<"), 1);
    assert_eq!(
        count(&html, "class=\"kit-actx__mark\" aria-hidden=\"true\">C<"),
        3
    );
    assert_eq!(
        count(&html, "class=\"kit-actx__source\">Read · Workspace<"),
        5
    );
    assert_eq!(count(&html, "7 calls"), 5);
}

#[test]
fn agent_subtitle_names_the_in_use_tool_and_falls_back_to_default() {
    let active = render(demo);
    assert!(active.contains("Using Docs search"));
    assert!(!active.contains("Drafts copy"));
    let idle = render(idle_demo);
    assert!(idle.contains("Drafts copy"));
    assert!(!idle.contains("Using "));
}

#[test]
fn demo_at_rest_shows_no_details_or_add_panel() {
    let html = render(demo);
    assert_eq!(count(&html, "kit-actx__panel"), 0);
}

#[test]
fn adding_a_tool_leaves_existing_slots_in_place() {
    let before = render(demo);
    let after = render(grown_demo);
    for y in ["8.33", "25.00", "41.67", "58.33", "75.00"] {
        let slot = format!("--actx-y: {y}%;");
        assert_eq!(
            count(&before, &slot),
            count(&after, &slot),
            "slot {y} moved"
        );
    }
    assert!(after.contains("--actx-y: 91.67%;"));
}

#[test]
fn outdated_memory_is_linked_to_its_replacement_and_says_so_in_text() {
    let html = render(demo);
    assert_eq!(count(&html, "class=\"kit-actx__bracket\""), 1);
    assert_eq!(count(&html, "kit-actx__tag\">outdated<"), 1);
}

#[test]
fn replacement_link_points_from_the_replaced_slot_to_its_replacement() {
    let links = replacement_links(&demo_memories());
    assert_eq!(links, vec![(1, 2)]);
}
