use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::screens::automation_canvas::{
    AutomationCanvas, CanvasComment, CanvasEdge, CanvasNode, NodeKind, NodeStatus, Port, StepResult,
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
    let node =
        |id: &str, kind: NodeKind, title: &str, x: f64, y: f64, status: NodeStatus| CanvasNode {
            id: id.into(),
            node_type: kind,
            node_title: title.into(),
            subtitle: Some("Runs on every new order".into()),
            pos_x: x,
            pos_y: y,
            node_status: status,
        };
    let edge = |id: &str, from: &str, port: Port, to: &str| CanvasEdge {
        id: id.into(),
        source_node: from.into(),
        source_port: port,
        target_node: to.into(),
    };
    let flow_nodes = if empty {
        Vec::new()
    } else {
        vec![
            node(
                "t",
                NodeKind::Trigger,
                "New order",
                40.0,
                40.0,
                NodeStatus::Ready,
            ),
            node(
                "b",
                NodeKind::Logic,
                "Amount over 500",
                40.0,
                180.0,
                NodeStatus::Ready,
            ),
            node(
                "yes",
                NodeKind::Agent,
                "Ask a manager",
                0.0,
                320.0,
                NodeStatus::Ready,
            ),
            node(
                "no",
                NodeKind::Action,
                "Send receipt",
                240.0,
                320.0,
                NodeStatus::Ready,
            ),
        ]
    };
    let flow_edges = if empty {
        Vec::new()
    } else {
        vec![
            edge("e1", "t", Port::Out, "b"),
            edge("e2", "b", Port::Yes, "yes"),
            edge("e3", "b", Port::No, "no"),
        ]
    };
    let flow_comments = if empty {
        Vec::new()
    } else {
        vec![
            CanvasComment {
                id: "c1".into(),
                pos_x: 300.0,
                pos_y: 120.0,
                comment_text: "Check the threshold with finance".into(),
                author_name: Some("Ana".into()),
                resolved: false,
            },
            CanvasComment {
                id: "c2".into(),
                pos_x: 300.0,
                pos_y: 420.0,
                comment_text: "Receipt copy confirmed".into(),
                author_name: Some("Ben".into()),
                resolved: true,
            },
        ]
    };
    rsx! {
        Item { slug: "automation-canvas", name: "AutomationCanvas",
            ModePick { slug: "automation-canvas", mode, last: last() }
            for flow_key in [mode_key] {
                div { key: "{flow_key}", style: "height: 900px; overflow: hidden; border: 1px solid var(--kit-rule-soft);",
                    AutomationCanvas {
                        project_name: "Onboarding",
                        path_branch: "Main",
                        is_loading: loading,
                        is_empty: empty,
                        canvas_nodes: flow_nodes.clone(),
                        canvas_edges: flow_edges.clone(),
                        canvas_comments: flow_comments.clone(),
                        step_results: BTreeMap::from([
                            ("t".to_string(), StepResult { duration: Some(0.4), credit_cost: None, error_message: None }),
                            ("b".to_string(), StepResult { duration: Some(0.2), credit_cost: Some(0.1), error_message: None }),
                            ("yes".to_string(), StepResult { duration: Some(2.4), credit_cost: Some(0.3), error_message: Some("Manager did not answer") }),
                        ]),
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
