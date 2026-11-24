use dioxus::prelude::*;
use rv2_kit::screens::multi_agent_run_canvas::{
    AgentNode, CanvasRun, CanvasRunState, MultiAgentRunCanvas, NodeState, TraceRow, TraceState,
    edges, layout,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn node(id: &str, role: &str, parent: Option<&str>, state: NodeState) -> AgentNode {
    AgentNode {
        id: id.to_string(),
        entry_name: id.to_string(),
        node_role: role.to_string(),
        model_name: "Claude Sonnet".to_string(),
        node_state: state,
        parent_id: parent.map(str::to_string),
        cost_label: (state != NodeState::Queued).then(|| "$0.42".to_string()),
        duration_seconds: None,
        token_count: None,
    }
}

fn fixture_nodes() -> Vec<AgentNode> {
    use NodeState::{Done, Failed, NeedsYou, Queued, Running};
    vec![
        node("orch", "orchestrator", None, Running),
        node("res", "research", Some("orch"), Done),
        node("r1", "research", Some("res"), Done),
        node("r2", "research", Some("res"), Done),
        node("r3", "research", Some("res"), Running),
        node("r4", "research", Some("res"), Done),
        node("ana", "analysis", Some("orch"), Done),
        node("a1", "analysis", Some("ana"), Done),
        node("a2", "analysis", Some("ana"), NeedsYou),
        node("a3", "analysis", Some("ana"), Done),
        node("wri", "writing", Some("orch"), Done),
        node("w1", "writing", Some("wri"), Failed),
        node("w2", "writing", Some("wri"), Queued),
        node("rev", "review", Some("orch"), Done),
        node("v1", "review", Some("rev"), Done),
        node("v2", "review", Some("rev"), Done),
    ]
}

fn trace(id: &str, agent: &str, start: f64, duration: Option<f64>, state: TraceState) -> TraceRow {
    TraceRow {
        id: id.to_string(),
        agent_id: agent.to_string(),
        entry_label: format!("{id} call"),
        start_seconds: start,
        duration_seconds: duration,
        row_state: state,
        error_text: None,
    }
}

fn fixture_trace() -> Vec<TraceRow> {
    let mut failed = trace("t12", "w1", 80.0, Some(12.0), TraceState::Failed);
    failed.error_text = Some("Draft refused: broke the style guide".to_string());
    vec![
        trace("t1", "orch", 0.0, Some(12.0), TraceState::Done),
        trace("t2", "res", 12.0, Some(28.0), TraceState::Done),
        trace("t3", "ana", 14.0, Some(10.0), TraceState::Done),
        trace("t4", "a1", 24.0, Some(26.0), TraceState::Done),
        trace("t5", "r1", 40.0, Some(18.0), TraceState::Done),
        trace("t6", "r2", 40.0, Some(26.0), TraceState::Done),
        trace("t7", "a2", 50.0, None, TraceState::Running),
        trace("t8", "a3", 52.0, Some(18.0), TraceState::Done),
        trace("t9", "r3", 58.0, None, TraceState::Running),
        trace("t10", "r4", 66.0, Some(18.0), TraceState::Done),
        trace("t11", "wri", 70.0, Some(10.0), TraceState::Done),
        failed,
        trace("t13", "w1", 92.0, Some(18.0), TraceState::Retry),
        trace("t14", "v1", 100.0, Some(20.0), TraceState::Done),
    ]
}

fn fixture_run() -> CanvasRun {
    CanvasRun {
        id: "run-01".to_string(),
        entry_title: "Market brief".to_string(),
        run_state: CanvasRunState::Running,
        elapsed_seconds: 120.0,
        cost_label: "$4.82".to_string(),
        token_count: 184_000,
        agents_done: 11,
        agents_total: 16,
    }
}

fn canvas_app() -> Element {
    rsx! {
        MultiAgentRunCanvas {
            run_record: Some(fixture_run()),
            agent_nodes: fixture_nodes(),
            trace_rows: fixture_trace(),
            is_live: true,
        }
    }
}

#[test]
fn canvas_at_rest_places_sixteen_nodes_in_lanes_of_five_four_three_and_three() {
    let positions = layout(&fixture_nodes());
    assert_eq!(positions.len(), 16);
    let orchestrator_x = positions
        .iter()
        .find(|(id, _, _)| id == "orch")
        .map(|p| p.1);
    assert_eq!(orchestrator_x, Some(50.0));
    let mut lanes: Vec<(f64, usize)> = Vec::new();
    for (_, x, _) in positions.iter().filter(|(id, _, _)| id != "orch") {
        match lanes
            .iter_mut()
            .find(|(lane_x, _)| (lane_x - x).abs() < 1e-6)
        {
            Some((_, count)) => *count += 1,
            None => lanes.push((*x, 1)),
        }
    }
    lanes.sort_by(|a, b| a.0.total_cmp(&b.0));
    let counts: Vec<usize> = lanes.iter().map(|(_, count)| *count).collect();
    assert_eq!(counts, vec![5, 4, 3, 3]);
}

#[test]
fn canvas_at_rest_has_one_edge_from_each_child_to_its_parent() {
    let links = edges(&fixture_nodes());
    assert_eq!(links.len(), 15);
    let nodes = fixture_nodes();
    for (parent, child) in &links {
        let child_node = nodes.iter().find(|n| &n.id == child).expect("child exists");
        assert_eq!(child_node.parent_id.as_deref(), Some(parent.as_str()));
    }
}

#[test]
fn canvas_at_rest_shows_eleven_done_two_running_one_needs_you_one_failed_one_queued() {
    let nodes = fixture_nodes();
    let count = |state: NodeState| nodes.iter().filter(|n| n.node_state == state).count();
    assert_eq!(count(NodeState::Done), 11);
    assert_eq!(count(NodeState::Running), 2);
    assert_eq!(count(NodeState::NeedsYou), 1);
    assert_eq!(count(NodeState::Failed), 1);
    assert_eq!(count(NodeState::Queued), 1);
}

#[test]
fn canvas_at_rest_has_fourteen_trace_rows_with_the_retry_directly_beneath_its_failure() {
    let rows = fixture_trace();
    assert_eq!(rows.len(), 14);
    let failed = rows
        .iter()
        .position(|r| r.row_state == TraceState::Failed)
        .expect("one failed row");
    let retry = &rows[failed + 1];
    assert_eq!(retry.row_state, TraceState::Retry);
    assert_eq!(retry.agent_id, rows[failed].agent_id);
    assert!(retry.duration_seconds.is_some());
}

#[test]
fn canvas_elapsed_equals_the_end_of_the_last_trace_row_while_live() {
    let last_end = fixture_trace()
        .iter()
        .map(|r| r.start_seconds + r.duration_seconds.unwrap_or(0.0))
        .fold(0.0_f64, f64::max);
    assert!((last_end - fixture_run().elapsed_seconds).abs() < 1e-9);
}

#[test]
fn canvas_at_rest_stacks_header_canvas_tiles_then_trace_with_the_panel_closed() {
    let html = render(canvas_app);
    let at = |needle: &str| {
        html.find(needle)
            .unwrap_or_else(|| panic!("missing {needle}"))
    };
    assert!(at("kit-amc__head") < at("kit-amc__canvas-wrap"));
    assert!(at("kit-amc__canvas-wrap") < at("kit-amc__tiles"));
    assert!(at("kit-amc__tiles") < at("kit-amc__lower"));
    assert!(!html.contains("kit-amc__panel"));
    assert!(html.contains("Pause run"));
    assert_eq!(html.matches("kit-amc__now").count(), 14);
}

#[test]
fn canvas_tile_reads_agents_done_of_total_and_failure_text_renders() {
    let html = render(canvas_app);
    assert!(html.contains("11 of 16"));
    assert!(html.contains("Agents done"));
    assert!(html.contains("Draft refused: broke the style guide"));
}
