use dioxus::prelude::*;
use rv2_kit::screens::automation_canvas::{
    AutomationCanvas, CanvasComment, CanvasEdge, CanvasNode, NodeKind, NodeStatus, Port,
};

use super::Item;
use super::screens_b::{Mode, ModePick};

#[component]
pub fn AutomationCanvasDemo() -> Element {
    let mode = use_signal(|| Mode::Data);
    let mut last = use_signal(|| "no action yet".to_string());
    let loading = mode() == Mode::Loading;
    let empty = mode() == Mode::Empty;
    let mode_key = match mode() {
        Mode::Data => "data",
        Mode::Loading => "loading",
        Mode::Empty => "empty",
    };
    let node = |id: &str,
                kind: NodeKind,
                title: &str,
                sub: Option<&str>,
                chip: Option<&str>,
                status: NodeStatus,
                x: f64,
                y: f64| CanvasNode {
        id: id.into(),
        node_type: kind,
        node_title: title.into(),
        subtitle: sub.map(str::to_string),
        pos_x: x,
        pos_y: y,
        node_status: status,
        chip: chip.map(str::to_string),
    };
    let edge = |id: &str, from: &str, port: Port, to: &str| CanvasEdge {
        id: id.into(),
        source_node: from.into(),
        source_port: port,
        target_node: to.into(),
    };
    // Board units: a card is 170 wide, so the seven cards at rest span 77% of a board whose width is 618 units.
    let flow_nodes = if empty {
        Vec::new()
    } else {
        vec![
            node(
                "t1",
                NodeKind::Trigger,
                "New order",
                Some("shop webhook · order created"),
                None,
                NodeStatus::Ready,
                167.0,
                22.0,
            ),
            node(
                "t2",
                NodeKind::Trigger,
                "Every morning",
                Some("Set a schedule to start"),
                None,
                NodeStatus::Unset,
                374.0,
                22.0,
            ),
            node(
                "agent",
                NodeKind::Agent,
                "Ask an assistant",
                Some("Auto · Order API · Web search"),
                Some("3 tools"),
                NodeStatus::Ready,
                167.0,
                169.0,
            ),
            node(
                "tool",
                NodeKind::Tool,
                "Look up the order",
                Some("GET orders/{id}"),
                Some("GET"),
                NodeStatus::Ready,
                167.0,
                293.0,
            ),
            node(
                "branch",
                NodeKind::Logic,
                "Amount over 500",
                Some("Yes → Ask a manager · No → Send receipt"),
                Some("2 paths"),
                NodeStatus::Ready,
                167.0,
                418.0,
            ),
            node(
                "yes",
                NodeKind::Action,
                "Ask a manager",
                Some("Payload: order summary"),
                None,
                NodeStatus::Ready,
                68.0,
                543.0,
            ),
            node(
                "no",
                NodeKind::Action,
                "Send receipt",
                Some("Payload: receipt email"),
                None,
                NodeStatus::Ready,
                266.0,
                543.0,
            ),
        ]
    };
    let flow_edges = if empty {
        Vec::new()
    } else {
        vec![
            edge("e1", "t1", Port::Out, "agent"),
            edge("e2", "agent", Port::Out, "tool"),
            edge("e3", "tool", Port::Out, "branch"),
            edge("e4", "branch", Port::Yes, "yes"),
            edge("e5", "branch", Port::No, "no"),
        ]
    };
    rsx! {
        Item { slug: "automation-canvas", name: "AutomationCanvas",
            ModePick { slug: "automation-canvas", mode, last: last() }
            for flow_key in [mode_key] {
                div { key: "{flow_key}", style: "width: min(100%, 887px); height: 733px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                    AutomationCanvas {
                        project_name: "Onboarding",
                        path_branch: "Main",
                        is_loading: loading,
                        is_empty: empty,
                        canvas_nodes: flow_nodes.clone(),
                        canvas_edges: flow_edges.clone(),
                        run_history: vec![
                            "succeeded, 3.1 s".to_string(),
                            "failed at Ask a manager".to_string(),
                            "succeeded, 2.8 s".to_string(),
                        ],
                        payload: "{{\n  \"order\": \"ord_1042\",\n  \"amount\": 640\n}}",
                        on_change: move |(nodes, edges): (Vec<CanvasNode>, Vec<CanvasEdge>)| last.set(format!("change {} nodes, {} links", nodes.len(), edges.len())),
                        on_comments: move |list: Vec<CanvasComment>| last.set(format!("comments {}", list.len())),
                        on_run: move |_| last.set("run".into()),
                        on_publish: move |_| last.set("publish".into()),
                        on_navigate: move |label: String| last.set(format!("navigate {label}")),
                    }
                }
            }
        }
    }
}
