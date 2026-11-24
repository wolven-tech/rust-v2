//! Amendment 6 of the automation canvas: the at-rest flow, palette, sidebar and run area.

use dioxus::prelude::*;
use rv2_kit::screens::automation_canvas::{
    AutomationCanvas, CanvasEdge, CanvasNode, NODE_W, NodeKind, NodeStatus, Port, fit_view,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn css_rule<'a>(css: &'a str, selector: &str) -> &'a str {
    css.split(&format!("{selector} {{"))
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .unwrap_or_else(|| panic!("rule {selector} missing"))
}

#[test]
fn node_titles_and_palette_labels_wrap_instead_of_truncating() {
    let css = include_str!("../assets/automation-canvas.css");
    for selector in [".kit-ac__nodetitle", ".kit-ac__label"] {
        let rule = css_rule(css, selector);
        assert!(!rule.contains("text-overflow"), "{selector} truncates");
        assert!(!rule.contains("nowrap"), "{selector} does not wrap");
    }
    assert!(css_rule(css, ".kit-ac__nodehead").contains("flex-wrap: wrap;"));
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

fn step(
    id: &str,
    kind: NodeKind,
    title: &str,
    subtitle: Option<&str>,
    x: f64,
    y: f64,
    status: NodeStatus,
) -> CanvasNode {
    CanvasNode {
        id: id.into(),
        node_type: kind,
        node_title: title.into(),
        subtitle: subtitle.map(str::to_string),
        pos_x: x,
        pos_y: y,
        node_status: status,
        chip: None,
    }
}

fn link(id: &str, from: &str, port: Port, to: &str) -> CanvasEdge {
    CanvasEdge {
        id: id.into(),
        source_node: from.into(),
        source_port: port,
        target_node: to.into(),
    }
}

fn rest_app() -> Element {
    rsx! {
        AutomationCanvas {
            project_name: "Onboarding",
            path_branch: "Main",
            canvas_nodes: vec![
                step("t1", NodeKind::Trigger, "New order", Some("Source: shop webhook"), 40.0, 0.0, NodeStatus::Ready),
                step("t2", NodeKind::Trigger, "Every morning", None, 250.0, 0.0, NodeStatus::Unset),
                step("agent", NodeKind::Agent, "Ask an assistant", Some("3 tools · auto model"), 145.0, 120.0, NodeStatus::Ready),
                step("tool", NodeKind::Tool, "Look up the order", Some("GET /orders/{id}"), 145.0, 240.0, NodeStatus::Ready),
                step("branch", NodeKind::Logic, "Amount over 500", Some("2 paths · amount > 500"), 145.0, 360.0, NodeStatus::Ready),
                step("yes", NodeKind::Action, "Ask a manager", Some("Target: manager inbox"), 40.0, 480.0, NodeStatus::Ready),
                step("no", NodeKind::Action, "Send receipt", Some("Target: buyer email"), 250.0, 480.0, NodeStatus::Ready),
            ],
            canvas_edges: vec![
                link("e1", "t1", Port::Out, "agent"),
                link("e2", "agent", Port::Out, "tool"),
                link("e3", "tool", Port::Out, "branch"),
                link("e4", "branch", Port::Yes, "yes"),
                link("e5", "branch", Port::No, "no"),
            ],
            run_history: vec![
                "succeeded, 3.1 s".to_string(),
                "failed at Ask a manager".to_string(),
                "succeeded, 2.8 s".to_string(),
            ],
        }
    }
}

/// Item counts per palette group, in the order the groups are drawn.
fn palette_group_sizes(html: &str) -> Vec<usize> {
    html.split("class=\"kit-ac__group\"")
        .skip(1)
        .map(|group| count(group, "class=\"kit-ac__item\""))
        .collect()
}

#[test]
fn palette_shows_four_groups_of_four_five_four_and_five_items() {
    let html = render(rest_app);
    assert_eq!(count(&html, "class=\"kit-ac__grouptitle\""), 4);
    assert_eq!(palette_group_sizes(&html), vec![4, 5, 4, 5]);
}

#[test]
fn at_rest_board_holds_seven_step_cards_and_one_unconfigured_trigger() {
    let html = render(rest_app);
    assert_eq!(count(&html, "class=\"kit-ac__node\""), 7);
    assert!(html.contains("Not configured"));
    assert_eq!(count(&html, "data-status=\"Ready\""), 6);
}

#[test]
fn at_rest_board_draws_five_links_with_yes_and_no_tags_on_the_branch() {
    let html = render(rest_app);
    assert_eq!(count(&html, "data-port=\"yes\""), 1);
    assert_eq!(count(&html, "data-port=\"no\""), 1);
    assert_eq!(count(&html, "class=\"kit-ac__edge"), 5);
}

#[test]
fn sidebar_runs_entry_shows_the_run_count_three() {
    let html = render(rest_app);
    assert!(html.contains("<span class=\"kit-ac__count\">3</span>"));
}

#[test]
fn configuration_panel_is_closed_at_rest() {
    let html = render(rest_app);
    assert!(html.contains("kit-ac__config is-closed"));
}

#[test]
fn run_area_is_closed_at_rest_and_the_run_control_sits_in_the_strip() {
    let html = render(rest_app);
    assert!(!html.contains("No runs yet"));
    assert!(!html.contains("kit-ac__tabs"));
    assert!(html.contains("aria-label=\"Run area\""));
}

#[test]
fn every_card_has_a_kind_tag_and_each_unlinked_output_has_an_add_control() {
    let html = render(rest_app);
    assert_eq!(count(&html, "class=\"kit-ac__kindtag\""), 7);
    assert_eq!(count(&html, "class=\"kit-ac__portadd\""), 3);
}

#[test]
fn palette_heading_is_nodes_with_a_close_control_and_group_counts() {
    let html = render(rest_app);
    assert!(html.contains("Nodes"));
    assert!(html.contains("aria-label=\"Close node list\""));
    assert_eq!(count(&html, "class=\"kit-ac__groupcount\""), 4);
}

#[test]
fn control_strip_shows_the_current_zoom_percent_beside_zoom_out() {
    let html = render(rest_app);
    assert!(html.contains("kit-ac__zoom"));
}

fn rest_positions() -> Vec<CanvasNode> {
    vec![
        step(
            "t1",
            NodeKind::Trigger,
            "New order",
            None,
            167.0,
            22.0,
            NodeStatus::Ready,
        ),
        step(
            "t2",
            NodeKind::Trigger,
            "Every morning",
            None,
            374.0,
            22.0,
            NodeStatus::Unset,
        ),
        step(
            "agent",
            NodeKind::Agent,
            "Ask an assistant",
            None,
            167.0,
            169.0,
            NodeStatus::Ready,
        ),
        step(
            "tool",
            NodeKind::Tool,
            "Look up the order",
            None,
            167.0,
            293.0,
            NodeStatus::Ready,
        ),
        step(
            "branch",
            NodeKind::Logic,
            "Amount over 500",
            None,
            167.0,
            418.0,
            NodeStatus::Ready,
        ),
        step(
            "yes",
            NodeKind::Action,
            "Ask a manager",
            None,
            68.0,
            543.0,
            NodeStatus::Ready,
        ),
        step(
            "no",
            NodeKind::Action,
            "Send receipt",
            None,
            266.0,
            543.0,
            NodeStatus::Ready,
        ),
    ]
}

#[test]
fn fitted_flow_spans_seventy_seven_percent_of_the_board_width_at_ninety_four_percent_zoom() {
    let nodes = rest_positions();
    let view = fit_view(&nodes, 581.0, 689.0);
    assert_eq!((view.zoom * 100.0).round(), 94.0);
    let min_x = nodes.iter().map(|n| n.pos_x).fold(f64::INFINITY, f64::min);
    let max_x = nodes
        .iter()
        .map(|n| n.pos_x + NODE_W)
        .fold(f64::NEG_INFINITY, f64::max);
    let share = (max_x - min_x) * view.zoom / 581.0;
    assert!((share - 0.77).abs() < 0.01, "spans {share}");
    let card_share = NODE_W * view.zoom / 581.0;
    assert!((card_share - 0.275).abs() < 0.005, "card {card_share}");
}
