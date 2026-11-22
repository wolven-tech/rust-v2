use dioxus::prelude::*;
use rv2_kit::screens::automation_canvas::{
    AutomationCanvas, CanvasComment, CanvasEdge, CanvasNode, MAX_ZOOM, MIN_ZOOM, NODE_H, NODE_W,
    NodeKind, NodeStatus, Port, View, arrow_target, board_point, connect, edge_path,
    first_free_port, fit_view, input_point, next_step_id, port_caption, port_point, ports_of,
    reading_order, run_order, status_line, zoom_about,
};
use rv2_kit::screens::automation_canvas::{insert_between, link_midpoint, pinch_view};

fn node(id: &str, kind: NodeKind, x: f64, y: f64) -> CanvasNode {
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

fn edge(id: &str, from: &str, port: Port, to: &str) -> CanvasEdge {
    CanvasEdge {
        id: id.into(),
        source_node: from.into(),
        source_port: port,
        target_node: to.into(),
    }
}

fn flow() -> (Vec<CanvasNode>, Vec<CanvasEdge>) {
    (
        vec![
            node("t", NodeKind::Trigger, 40.0, 40.0),
            node("b", NodeKind::Logic, 40.0, 160.0),
            node("yes", NodeKind::Action, 0.0, 280.0),
            node("no", NodeKind::Action, 240.0, 280.0),
        ],
        vec![
            edge("e1", "t", Port::Out, "b"),
            edge("e2", "b", Port::Yes, "yes"),
            edge("e3", "b", Port::No, "no"),
        ],
    )
}

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn loaded_app() -> Element {
    let (nodes, edges) = flow();
    let comments = vec![CanvasComment {
        id: "c1".into(),
        pos_x: 300.0,
        pos_y: 120.0,
        comment_text: "Check the threshold".into(),
        author_name: Some("Ana".into()),
        resolved: false,
    }];
    rsx! { AutomationCanvas { canvas_nodes: nodes, canvas_edges: edges, canvas_comments: comments } }
}

fn loading_app() -> Element {
    let (nodes, edges) = flow();
    rsx! { AutomationCanvas { canvas_nodes: nodes, canvas_edges: edges, is_loading: true } }
}

fn empty_app() -> Element {
    rsx! { AutomationCanvas { is_empty: true } }
}

#[test]
fn branch_exposes_yes_and_no_outputs_and_other_kinds_one_output() {
    assert_eq!(ports_of(NodeKind::Logic), &[Port::Yes, Port::No]);
    assert_eq!(ports_of(NodeKind::Tool), &[Port::Out]);
    assert_eq!(port_caption(Port::Yes), "Connect yes to…");
    assert_eq!(port_caption(Port::Out), "Connect to…");
}

#[test]
fn branch_ports_sit_apart_on_the_bottom_edge_and_input_on_the_left() {
    let b = node("b", NodeKind::Logic, 40.0, 160.0);
    let yes = port_point(&b, Port::Yes);
    let no = port_point(&b, Port::No);
    assert!(yes.0 < no.0, "yes port left of no port");
    assert_eq!(yes.1, 160.0 + NODE_H);
    assert_eq!(no.1, 160.0 + NODE_H);
    assert_eq!(input_point(&b), (40.0, 160.0 + NODE_H / 2.0));
}

#[test]
fn edge_path_runs_top_to_bottom_between_the_two_ends() {
    let d = edge_path((100.0, 88.0), (100.0, 200.0));
    assert!(d.starts_with("M 100.0 88.0 C 100.0 "), "{d}");
    assert!(d.ends_with("100.0 200.0"), "{d}");
}

#[test]
fn first_free_port_skips_linked_outputs_and_falls_back_to_the_first() {
    let (nodes, edges) = flow();
    let branch = &nodes[1];
    assert_eq!(first_free_port(branch, &edges), Port::Yes);
    let open_edges: Vec<CanvasEdge> = vec![edge("e2", "b", Port::Yes, "yes")];
    assert_eq!(first_free_port(branch, &open_edges), Port::No);
    let all_linked = vec![
        edge("e2", "b", Port::Yes, "yes"),
        edge("e3", "b", Port::No, "no"),
    ];
    assert_eq!(first_free_port(branch, &all_linked), Port::Yes);
}

#[test]
fn next_step_id_never_reuses_an_existing_id() {
    let (nodes, _) = flow();
    assert_eq!(next_step_id(&nodes), "step-5");
    let clash = vec![node("step-2", NodeKind::Tool, 0.0, 0.0)];
    assert_eq!(next_step_id(&clash), "step-3");
}

#[test]
fn connect_adds_one_link_per_source_port_and_target() {
    let (_, edges) = flow();
    let linked = connect(&edges, "t", Port::Out, "no").expect("new link");
    assert_eq!(linked.len(), edges.len() + 1);
    assert!(
        connect(&edges, "b", Port::Yes, "yes").is_none(),
        "duplicate refused"
    );
    assert!(
        connect(&edges, "t", Port::Out, "t").is_none(),
        "self link refused"
    );
    let other_port = connect(&edges, "b", Port::Out, "yes");
    assert!(
        other_port.is_some(),
        "a different port to the same step is a new link"
    );
}

#[test]
fn connect_keeps_edge_ids_unique_when_ids_collide() {
    let edges = vec![edge("b-yes-yes", "x", Port::Out, "y")];
    let next = connect(&edges, "b", Port::Yes, "yes").expect("new link");
    let ids: Vec<&str> = next.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(
        ids.len(),
        ids.iter().collect::<std::collections::BTreeSet<_>>().len()
    );
}

#[test]
fn board_point_undoes_pan_and_zoom() {
    let view = View {
        pan_x: 24.0,
        pan_y: 24.0,
        zoom: 0.5,
    };
    let (x, y) = board_point((224.0, 124.0), (200.0, 100.0), view);
    assert!((x - 0.0).abs() < 1e-9);
    assert!((y - 0.0).abs() < 1e-9);
    let (x, y) = board_point((274.0, 174.0), (200.0, 100.0), view);
    assert!((x - 100.0).abs() < 1e-9);
    assert!((y - 100.0).abs() < 1e-9);
}

#[test]
fn fit_never_enlarges_past_actual_size_and_floors_at_legibility() {
    let (nodes, _) = flow();
    let view = fit_view(&nodes, 1200.0, 800.0);
    assert!(view.zoom <= 1.0, "no enlargement: {}", view.zoom);
    assert_eq!(view.zoom, 1.0, "small flow fits at actual size");

    let spread = vec![
        node("a", NodeKind::Trigger, 0.0, 0.0),
        node("b", NodeKind::Action, 4000.0, 4000.0),
    ];
    let zoomed_out = fit_view(&spread, 400.0, 300.0);
    assert_eq!(
        zoomed_out.zoom, MIN_ZOOM,
        "floors at the legibility minimum"
    );
    assert!(fit_view(&[], 400.0, 300.0) == View::default());
}

#[test]
fn zoom_about_keeps_the_anchor_point_fixed_and_clamps_the_zoom() {
    let view = View {
        pan_x: 10.0,
        pan_y: 20.0,
        zoom: 1.0,
    };
    let anchored = zoom_about(view, 2.0, 110.0, 120.0);
    let before = (110.0 - view.pan_x) / view.zoom;
    let after = (110.0 - anchored.pan_x) / anchored.zoom;
    assert!((before - after).abs() < 1e-9);
    assert_eq!(zoom_about(view, 9.0, 0.0, 0.0).zoom, MAX_ZOOM);
    assert_eq!(zoom_about(view, 0.01, 0.0, 0.0).zoom, MIN_ZOOM);
}

#[test]
fn status_line_shows_elapsed_time_only_for_finished_steps() {
    assert_eq!(status_line(NodeStatus::Done, Some(2.44)), "Done · 2.4 s");
    assert_eq!(status_line(NodeStatus::Done, None), "Done");
    assert_eq!(status_line(NodeStatus::Running, Some(1.0)), "Running");
}

#[test]
fn run_walks_links_from_the_trigger_and_arrows_follow_them() {
    let (nodes, edges) = flow();
    assert_eq!(run_order(&nodes, &edges), vec!["t", "b", "yes", "no"]);
    assert_eq!(
        arrow_target(&nodes, &edges, "b", "down"),
        Some("yes".to_string())
    );
    assert_eq!(
        arrow_target(&nodes, &edges, "yes", "up"),
        Some("b".to_string())
    );
    assert_eq!(arrow_target(&nodes, &edges, "t", "left"), None);
    assert_eq!(reading_order(&nodes).first().map(String::as_str), Some("t"));
    assert_eq!(reading_order(&nodes).last().map(String::as_str), Some("no"));
}

#[test]
fn constants_match_the_card_geometry_used_by_the_stylesheet() {
    assert_eq!(NODE_W, 170.0);
    assert_eq!(NODE_H, 88.0);
    assert_eq!(MIN_ZOOM, 0.5);
    assert_eq!(MAX_ZOOM, 1.5);
}

#[test]
fn loaded_board_names_steps_ports_comments_and_the_run_tabs() {
    let html = render(loaded_app);
    assert!(html.contains("aria-label=\"Step t, Trigger, Ready\""));
    assert!(html.contains("data-port=\"yes\""));
    assert!(html.contains("data-port=\"no\""));
    assert!(html.contains("aria-label=\"Comment by Ana: Check the threshold\""));
    for tab in ["payload", "triggers", "errors", "timeline", "runs"] {
        assert!(
            html.contains("id=\"kit-ac-") && html.contains(&format!("-tab-{tab}\"")),
            "tab {tab}"
        );
    }
    for control in ["Select", "Comment", "Zoom in", "Zoom out", "Fit to view"] {
        assert!(
            html.contains(&format!("aria-label=\"{control}\"")),
            "control {control}"
        );
    }
    assert!(html.contains("role=\"status\""));
    assert!(html.contains("aria-live=\"polite\""));
}

#[test]
fn sidebar_lists_nine_destinations_with_runs_and_a_count() {
    let html = render(loaded_app);
    let order = [
        "Workflows",
        "Runs",
        "Templates",
        "Integrations",
        "Variables",
        "Data tables",
        "Workspace",
        "Knowledge",
        "Settings",
    ];
    let mut at = 0;
    for name in order {
        let found = html[at..]
            .find(&format!(">{name}"))
            .or_else(|| html[at..].find(name));
        let idx = found.unwrap_or_else(|| panic!("{name} missing after position {at}"));
        at += idx + name.len();
    }
    assert!(html.contains("kit-ac__count"));
}

#[test]
fn loading_board_shows_placeholders_and_keeps_the_palette_usable() {
    let html = render(loading_app);
    assert!(html.contains("kit-ac__ph"));
    assert!(
        !html.contains("kit-ac__node\""),
        "no live step cards while loading"
    );
    assert!(html.contains("aria-busy=\"true\""));
    let add = html
        .find("aria-label=\"Add Webhook trigger\"")
        .expect("palette item");
    let window = &html[add..add + 200];
    assert!(
        !window.contains("disabled"),
        "palette item stays enabled while loading"
    );
}

#[test]
fn empty_board_offers_one_centred_add_trigger_action() {
    let html = render(empty_app);
    assert_eq!(html.matches("Add a trigger").count(), 1);
    assert!(!html.contains("kit-ac__node\""));
}

#[test]
fn link_midpoint_sits_between_the_two_ends() {
    let (nodes, edges) = flow();
    let mid = link_midpoint(&nodes, &edges[0]).expect("both ends exist");
    let from = port_point(&nodes[0], Port::Out);
    let to = input_point(&nodes[1]);
    assert_eq!(mid, ((from.0 + to.0) / 2.0, (from.1 + to.1) / 2.0));
    let dangling = edge("x", "t", Port::Out, "missing");
    assert!(link_midpoint(&nodes, &dangling).is_none());
}

#[test]
fn inserting_on_a_link_replaces_it_with_two_links_through_the_new_step() {
    let (nodes, edges) = flow();
    let step = node("mid", NodeKind::Tool, 60.0, 100.0);
    let (next_nodes, next_edges) = insert_between(&nodes, &edges, "e1", step).expect("link exists");
    assert_eq!(next_nodes.len(), nodes.len() + 1);
    assert_eq!(next_edges.len(), edges.len() + 1);
    assert!(!next_edges.iter().any(|e| e.id == "e1"), "old link removed");
    assert!(
        next_edges
            .iter()
            .any(|e| e.source_node == "t" && e.target_node == "mid")
    );
    assert!(
        next_edges
            .iter()
            .any(|e| e.source_node == "mid" && e.target_node == "b")
    );
    assert!(insert_between(&nodes, &edges, "nope", node("z", NodeKind::Tool, 0.0, 0.0)).is_none());
}

#[test]
fn pinch_keeps_the_point_under_the_midpoint_and_scales_by_spread() {
    let origin = View {
        pan_x: 24.0,
        pan_y: 24.0,
        zoom: 1.0,
    };
    let board = (100.0, 50.0);
    let mid0 = (300.0, 250.0);
    let before = board_point(mid0, board, origin);
    let next = pinch_view(origin, mid0, 100.0, mid0, 200.0, board);
    assert!(
        (next.zoom - 2.0).abs() < 1e-9 || next.zoom == MAX_ZOOM,
        "zoom doubles, clamped"
    );
    let after = board_point(mid0, board, next);
    assert!((before.0 - after.0).abs() < 1e-9);
    assert!((before.1 - after.1).abs() < 1e-9);
    let moved = pinch_view(origin, mid0, 100.0, (340.0, 250.0), 100.0, board);
    assert!(
        (moved.pan_x - origin.pan_x - 40.0).abs() < 1e-9,
        "two-finger drag pans"
    );
    assert_eq!(moved.zoom, origin.zoom);
}

#[test]
fn start_test_is_offered_with_a_loaded_flow() {
    let html = render(loaded_app);
    assert!(html.contains("Start test"));
    assert!(html.contains("Production"));
    assert!(html.contains("Select a step to configure it."));
}
