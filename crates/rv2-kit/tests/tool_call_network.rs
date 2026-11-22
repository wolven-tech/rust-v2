use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::agents::tool_call_network::{
    Condition, Entity, Exchange, ExchangeResult, NodeRole, ToolCallNetwork, first_in_condition,
    layout, node_name, recent_exchanges, text_summary,
};

const NOW: u64 = 1_000_000;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn node(
    id: &str,
    name: &str,
    role: NodeRole,
    condition: Condition,
    centre: bool,
    activity: Option<&str>,
) -> Entity {
    Entity {
        id: id.to_string(),
        name: name.to_string(),
        node_role: role,
        compact_label: None,
        glyph: None,
        summary_line: None,
        condition,
        activity: activity.map(str::to_string),
        spend: Some(0.05),
        is_centre: centre,
    }
}

fn exchange(
    id: &str,
    from: &str,
    to: &str,
    ago_ms: u64,
    result: ExchangeResult,
    latency: Option<u64>,
) -> Exchange {
    Exchange {
        id: id.to_string(),
        from_id: from.to_string(),
        to_id: to.to_string(),
        sent_at_ms: NOW - ago_ms,
        result,
        caption: None,
        latency_ms: latency,
    }
}

/// The gallery's graph: four agents, six tools and one person, ten links, five exchanges.
fn demo_entities() -> Vec<Entity> {
    vec![
        node(
            "orchestrator",
            "Orchestrator",
            NodeRole::Agent,
            Condition::Working,
            true,
            Some("Planning the brief"),
        ),
        node(
            "researcher",
            "Researcher",
            NodeRole::Agent,
            Condition::Working,
            false,
            Some("Reading competitor pages"),
        ),
        node(
            "analyst",
            "Analyst",
            NodeRole::Agent,
            Condition::Waiting,
            false,
            Some("Needs a threshold decision"),
        ),
        node(
            "coder",
            "Coder",
            NodeRole::Agent,
            Condition::Idle,
            false,
            None,
        ),
        node(
            "search",
            "Web search",
            NodeRole::Tool,
            Condition::Idle,
            false,
            None,
        ),
        node(
            "notion",
            "Notion",
            NodeRole::Tool,
            Condition::Idle,
            false,
            None,
        ),
        node(
            "database",
            "Database",
            NodeRole::Tool,
            Condition::Error,
            false,
            None,
        ),
        node(
            "calendar",
            "Calendar",
            NodeRole::Tool,
            Condition::Idle,
            false,
            None,
        ),
        node(
            "github",
            "GitHub",
            NodeRole::Tool,
            Condition::Idle,
            false,
            None,
        ),
        node(
            "slack",
            "Slack",
            NodeRole::Tool,
            Condition::Done,
            false,
            None,
        ),
        node("you", "You", NodeRole::User, Condition::Idle, false, None),
    ]
}

fn demo_links() -> Vec<(String, String)> {
    [
        ("orchestrator", "researcher"),
        ("orchestrator", "analyst"),
        ("orchestrator", "coder"),
        ("orchestrator", "you"),
        ("researcher", "search"),
        ("researcher", "notion"),
        ("analyst", "database"),
        ("analyst", "calendar"),
        ("coder", "github"),
        ("coder", "slack"),
    ]
    .map(|(from, to)| (from.to_string(), to.to_string()))
    .to_vec()
}

fn demo_exchanges() -> Vec<Exchange> {
    vec![
        exchange(
            "x1",
            "orchestrator",
            "researcher",
            40_000,
            ExchangeResult::Ok,
            Some(1_800),
        ),
        exchange(
            "x2",
            "researcher",
            "search",
            30_000,
            ExchangeResult::Ok,
            Some(640),
        ),
        exchange(
            "x3",
            "analyst",
            "database",
            12_000,
            ExchangeResult::Error,
            Some(4_200),
        ),
        exchange(
            "x4",
            "coder",
            "github",
            20_000,
            ExchangeResult::Ok,
            Some(2_400),
        ),
        exchange(
            "x5",
            "orchestrator",
            "researcher",
            3_000,
            ExchangeResult::Pending,
            None,
        ),
    ]
}

fn demo_app() -> Element {
    rsx! {
        ToolCallNetwork {
            entities: demo_entities(),
            links: demo_links(),
            exchanges: demo_exchanges(),
            clock_now_ms: Some(NOW),
        }
    }
}

fn error_app() -> Element {
    rsx! {
        ToolCallNetwork {
            entities: vec![
                node("agent", "Agent", NodeRole::Agent, Condition::Working, true, None),
                node("db", "Database", NodeRole::Tool, Condition::Error, false, None),
            ],
            links: vec![("agent".to_string(), "db".to_string())],
            exchanges: vec![exchange("e1", "agent", "db", 3_000, ExchangeResult::Error, None)],
            clock_now_ms: Some(NOW),
        }
    }
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn demo_positions() -> BTreeMap<String, (f64, f64)> {
    layout(&demo_entities(), &demo_links())
        .into_iter()
        .map(|(id, x, y)| (id, (x, y)))
        .collect()
}

#[test]
fn orchestrator_is_centred_and_inner_agents_sit_on_the_inner_ring() {
    let pos = demo_positions();
    let centre = (500.0, 500.0);
    assert_eq!(pos["orchestrator"], centre);
    for agent in ["researcher", "analyst", "coder"] {
        assert!(
            (distance(pos[agent], centre) - 230.0).abs() < 1e-6,
            "{agent}"
        );
    }
}

#[test]
fn six_tools_and_the_person_sit_on_the_outer_ring() {
    let pos = demo_positions();
    let centre = (500.0, 500.0);
    for id in [
        "search", "notion", "database", "calendar", "github", "slack", "you",
    ] {
        assert!((distance(pos[id], centre) - 400.0).abs() < 1e-6, "{id}");
    }
    assert_eq!(pos.len(), 11);
}

#[test]
fn text_list_names_all_eleven_nodes_with_condition_and_exchange_count() {
    let lines = text_summary(&demo_entities(), &demo_exchanges(), NOW);
    assert_eq!(lines.len(), 11);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("Orchestrator: Working, 2 exchanges in the last minute"))
    );
    assert!(lines.iter().any(|l| {
        l.starts_with("Analyst: Needs you, 1 exchange in the last minute; contacted Database (1)")
    }));
}

#[test]
fn header_chips_read_needs_you_one_and_failed_one_at_rest() {
    let html = render(demo_app);
    assert!(html.contains("Needs you 1"));
    assert!(html.contains("Failed 1"));
}

#[test]
fn waiting_node_shows_the_needs_you_badge() {
    let html = render(demo_app);
    assert!(html.contains("data-condition=\"waiting\""));
    assert!(html.contains("kit-tcn__badge\" aria-hidden=\"true\">Needs you<"));
}

#[test]
fn chip_opens_the_first_node_in_its_condition() {
    let entities = demo_entities();
    assert_eq!(
        first_in_condition(&entities, Condition::Waiting).as_deref(),
        Some("analyst")
    );
    assert_eq!(
        first_in_condition(&entities, Condition::Error).as_deref(),
        Some("database")
    );
}

#[test]
fn analyst_card_lists_its_exchange_with_target_and_duration() {
    let entities = demo_entities();
    let exchanges = demo_exchanges();
    assert_eq!(
        recent_exchanges(&entities, &exchanges, "analyst", 3),
        vec![(
            "x3".to_string(),
            "Database".to_string(),
            "4.2 s".to_string()
        )]
    );
    let analyst = entities.iter().find(|e| e.id == "analyst").unwrap();
    assert!(node_name(analyst, 1).contains("Needs you"));
}

#[test]
fn each_outer_node_has_one_link_and_no_two_outer_nodes_link() {
    let entities = demo_entities();
    let links = demo_links();
    let outer: Vec<&str> = entities
        .iter()
        .filter(|e| e.node_role != NodeRole::Agent)
        .map(|e| e.id.as_str())
        .collect();
    for id in &outer {
        let degree = links.iter().filter(|(a, b)| a == id || b == id).count();
        assert_eq!(degree, 1, "{id}");
    }
    for (a, b) in &links {
        assert!(
            !(outer.contains(&a.as_str()) && outer.contains(&b.as_str())),
            "{a}-{b}"
        );
    }
}

fn orient(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

fn segments_cross(a: ((f64, f64), (f64, f64)), b: ((f64, f64), (f64, f64))) -> bool {
    let d1 = orient(b.0, b.1, a.0);
    let d2 = orient(b.0, b.1, a.1);
    let d3 = orient(a.0, a.1, b.0);
    let d4 = orient(a.0, a.1, b.1);
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

#[test]
fn no_two_links_cross_in_the_demo_layout() {
    let pos = demo_positions();
    let links = demo_links();
    for (i, (a1, a2)) in links.iter().enumerate() {
        for (b1, b2) in &links[i + 1..] {
            if a1 == b1 || a1 == b2 || a2 == b1 || a2 == b2 {
                continue;
            }
            let a = (pos[a1], pos[a2]);
            let b = (pos[b1], pos[b2]);
            assert!(!segments_cross(a, b), "{a1}-{a2} crosses {b1}-{b2}");
        }
    }
}

/// Checks every pair of node boxes at a canvas `canvas_px` wide. Nodes are 17.6% of the canvas
/// wide; `height_px` gives each node's rendered height in px.
fn assert_no_node_overlaps(canvas_px: f64, height_px: impl Fn(&Entity) -> f64) {
    let scale = canvas_px / 1000.0;
    let width = 176.0;
    let entities = demo_entities();
    let height = |id: &str| {
        let entity = entities.iter().find(|e| e.id == id).unwrap();
        height_px(entity) / scale
    };
    let pos = demo_positions();
    let ids: Vec<&String> = pos.keys().collect();
    for (i, a) in ids.iter().enumerate() {
        for b in &ids[i + 1..] {
            let dx = (pos[*a].0 - pos[*b].0).abs();
            let dy = (pos[*a].1 - pos[*b].1).abs();
            assert!(
                dx >= width || dy >= (height(a) + height(b)) / 2.0,
                "{a} overlaps {b}: dx={dx:.1} dy={dy:.1}"
            );
        }
    }
}

#[test]
fn no_two_node_boxes_overlap_at_the_widest_canvas() {
    // 600 px container less 16 px padding and 1 px border each side.
    assert_no_node_overlaps(566.0, |_| 96.0);
}

#[test]
fn no_two_node_boxes_overlap_at_the_narrowest_canvas_beside_the_tree() {
    // 421 px container, the first width below the stacked list: canvas 387 px, role line hidden.
    assert_no_node_overlaps(387.0, |e| match e.condition {
        Condition::Waiting | Condition::Error => 81.0,
        _ => 62.0,
    });
}

#[test]
fn error_exchange_sends_a_critical_return_pulse_and_flashes_its_link() {
    let html = render(error_app);
    assert_eq!(html.matches("data-phase=\"out\"").count(), 1);
    assert_eq!(html.matches("data-phase=\"back\"").count(), 1);
    assert!(html.contains("data-result=\"error\""));
    assert!(html.contains("data-flash=\"true\""));
}
