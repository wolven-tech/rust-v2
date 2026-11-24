//! Multi-agent run canvas: a live run of cooperating agents as a graph. Each
//! agent is a node, placed by its depth under the orchestrator and its position
//! among its siblings. A person sees who started whom, what is running, what
//! waits for them and what it costs. The trace below lists every call.
//!
//! Design choice: the layout is computed from `parent_id` alone. Depth is the
//! number of parents above a node; nodes at one depth share a row, spaced evenly.
//! Design choice: edges are drawn from each parent to each child in an SVG overlay
//! with a 0 to 100 viewBox, so they follow the node positions at any width.
//! Design choice: the canvas scrolls inside itself. The page never scrolls
//! sideways.
//! Design choice: Stop asks for confirmation inline, and only that confirmation
//! calls `on_stop`.
//! Design choice: the trace is the list-equivalent view of the graph. Its rows are
//! buttons that open the agent they belong to.
//! Design choice: the live clock is the host's `elapsed_seconds`. When `is_live`
//! is false the figures do not change between renders.
//! Design choice: zoom scales the stage with CSS `zoom`, so the canvas scrolls at
//! every level. Fit uses the canvas width measured in the browser.
//! Design choice: with the canvas focused, + and - zoom, F fits and the arrow keys
//! pan. On a node the arrow keys move focus to the nearest node in that direction.
//! Browser checks: every edge joins a node to its parent; a queued node reads
//! "queued" with no cost; Pause calls `on_pause`; Stop calls `on_stop` only after
//! the confirmation; a node's panel shows its tasks and cost; a failed agent shows
//! its error and a retry is a separate trace row; zoom in, out and fit change the
//! stage; no horizontal scroll on the page at 320 px.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, focus_after_frame, next_id, use_element_width};

const CSS: Asset = asset!("/assets/multi-agent-run-canvas.css");

/// Smallest and largest zoom the canvas controls reach.
pub const ZOOM_MIN: f64 = 0.5;
pub const ZOOM_MAX: f64 = 1.5;
/// Natural width of the stage at zoom 1, in CSS px. Matches `.kit-amc__stage`.
/// Four lanes of 150 px nodes need this width before they stop overlapping.
const STAGE_WIDTH: f64 = 720.0;
/// Pixels one arrow press pans the canvas.
const PAN_STEP: i32 = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasRunState {
    Running,
    Paused,
    Done,
    Failed,
}

impl CanvasRunState {
    fn key(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeState {
    Running,
    Done,
    Queued,
    NeedsYou,
    Failed,
    Retrying,
}

impl NodeState {
    fn key(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Queued => "queued",
            Self::NeedsYou => "needs-you",
            Self::Failed => "failed",
            Self::Retrying => "retrying",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Done => "Done",
            Self::Queued => "Queued",
            Self::NeedsYou => "Needs you",
            Self::Failed => "Failed",
            Self::Retrying => "Retrying",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceState {
    Running,
    Done,
    Failed,
    Retry,
}

impl TraceState {
    fn key(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Retry => "retry",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasRun {
    pub id: String,
    pub entry_title: String,
    pub run_state: CanvasRunState,
    pub elapsed_seconds: f64,
    pub cost_label: String,
    pub token_count: u64,
    pub agents_done: usize,
    pub agents_total: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentNode {
    pub id: String,
    pub entry_name: String,
    pub node_role: String,
    pub model_name: String,
    pub node_state: NodeState,
    pub parent_id: Option<String>,
    pub cost_label: Option<String>,
    pub duration_seconds: Option<f64>,
    pub token_count: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TraceRow {
    pub id: String,
    pub agent_id: String,
    pub entry_label: String,
    pub start_seconds: f64,
    pub duration_seconds: Option<f64>,
    pub row_state: TraceState,
    pub error_text: Option<String>,
}

/// Node positions as `(id, x, y)` in percent of the canvas. Roots sit on the top
/// row. Each child of a root heads a lane, one column per lane; the lane's other
/// nodes stack beneath its head in input order, so every edge is a short drop.
pub fn layout(nodes: &[AgentNode]) -> Vec<(String, f64, f64)> {
    let find = |id: &str| nodes.iter().find(|n| n.id == id);
    let depth_of = |id: &str| -> usize {
        let mut depth = 0;
        let mut current = find(id);
        while let Some(node) = current.and_then(|n| n.parent_id.as_deref()) {
            depth += 1;
            current = find(node);
            if depth > nodes.len() {
                break;
            }
        }
        depth
    };
    let depths: Vec<usize> = nodes.iter().map(|n| depth_of(&n.id)).collect();
    let lane_heads: Vec<&str> = nodes
        .iter()
        .zip(depths.iter())
        .filter(|(_, d)| **d == 1)
        .map(|(n, _)| n.id.as_str())
        .collect();
    let lane_of = |id: &str| -> Option<usize> {
        let mut current = find(id)?;
        for _ in 0..nodes.len() {
            let parent = current.parent_id.as_deref()?;
            if depth_of(&current.id) == 1 {
                return lane_heads.iter().position(|head| *head == current.id);
            }
            current = find(parent)?;
        }
        None
    };
    let lane_count = lane_heads.len().max(1);
    let lanes: Vec<Option<usize>> = nodes.iter().map(|n| lane_of(&n.id)).collect();
    let mut by_depth: Vec<usize> = (0..nodes.len()).collect();
    by_depth.sort_by_key(|&index| depths[index]);
    let mut rows = vec![0usize; nodes.len()];
    let mut next_row = vec![1usize; lane_count];
    for &index in &by_depth {
        if let Some(lane) = lanes[index] {
            rows[index] = next_row[lane];
            next_row[lane] += 1;
        }
    }
    let deepest_row = next_row.iter().copied().max().unwrap_or(1) - 1;
    let root_count = lanes.iter().filter(|lane| lane.is_none()).count().max(1);
    let mut roots_seen = 0usize;
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let (x, y) = match lanes[index] {
                Some(lane) => {
                    let x = (lane as f64 + 0.5) / lane_count as f64 * 100.0;
                    let y = 10.0 + rows[index] as f64 / deepest_row.max(1) as f64 * 80.0;
                    (x, y)
                }
                None => {
                    let x = (roots_seen as f64 + 0.5) / root_count as f64 * 100.0;
                    roots_seen += 1;
                    let y = if deepest_row == 0 { 50.0 } else { 10.0 };
                    (x, y)
                }
            };
            (node.id.clone(), x, y)
        })
        .collect()
}

/// Edges from each parent to each child; nodes without a parent have none.
pub fn edges(nodes: &[AgentNode]) -> Vec<(String, String)> {
    nodes
        .iter()
        .filter_map(|n| {
            let parent = n.parent_id.as_ref()?;
            nodes
                .iter()
                .any(|p| p.id == *parent)
                .then(|| (parent.clone(), n.id.clone()))
        })
        .collect()
}

/// The node nearest `from` by straight-line distance among those lying in the
/// direction (`dx`, `dy`), each -1, 0 or 1, in layout position.
pub fn nearest_in_direction(
    positions: &[(String, f64, f64)],
    from: &str,
    dx: i8,
    dy: i8,
) -> Option<String> {
    let (_, x0, y0) = positions.iter().find(|(id, _, _)| id == from)?;
    positions
        .iter()
        .filter(|(id, _, _)| id != from)
        .filter_map(|(id, x, y)| {
            let (ax, ay) = (x - x0, y - y0);
            let along = ax * f64::from(dx) + ay * f64::from(dy);
            if along <= 0.0 {
                return None;
            }
            Some((ax.hypot(ay), id))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| id.clone())
}

/// Cost text for a node: `queued` for a queued node, else its cost or a dash.
pub fn node_cost(node: &AgentNode) -> String {
    match node.node_state {
        NodeState::Queued => "queued".to_string(),
        _ => node.cost_label.clone().unwrap_or_else(|| "—".to_string()),
    }
}

/// Trace bar geometry as percent of `span`: start and width.
pub fn bar_percent(row: &TraceRow, span: f64) -> (f64, f64) {
    let span = span.max(f64::EPSILON);
    let start = (row.start_seconds / span * 100.0).clamp(0.0, 100.0);
    let end = row.start_seconds + row.duration_seconds.unwrap_or(0.0);
    let width = ((end - row.start_seconds).max(0.0) / span * 100.0).clamp(0.0, 100.0 - start);
    (start, width)
}

/// Zoom after one step in (`delta` 1) or out (`delta` -1), in tenths, within bounds.
pub fn zoom_step(current: f64, delta: i32) -> f64 {
    let next = (current * 10.0).round() / 10.0 + f64::from(delta) * 0.1;
    (next.clamp(ZOOM_MIN, ZOOM_MAX) * 10.0).round() / 10.0
}

/// The largest tenth of zoom at which the stage fits `viewport_width`, at most 1.
/// Without a measurement (SSR, tests) the stage stays at 1.
pub fn fit_zoom(viewport_width: Option<f64>) -> f64 {
    match viewport_width {
        Some(width) if width > 0.0 => {
            let tenths = ((width / STAGE_WIDTH).min(1.0) * 10.0).floor();
            (tenths.max(ZOOM_MIN * 10.0)) / 10.0
        }
        _ => 1.0,
    }
}

fn set_zoom(mut zoom: Signal<f64>, mut announce: Signal<String>, next: f64) {
    zoom.set(next);
    announce.set(format!("Zoom {:.0}%", next * 100.0));
}

fn pan_canvas(id: &str, dx: i32, dy: i32) {
    document::eval(&format!(
        "document.getElementById({id:?})?.scrollBy({{ left: {dx}, top: {dy} }});"
    ));
}

/// The canvas of agents with the summary, the trace and the agent panel.
#[component]
pub fn MultiAgentRunCanvas(
    #[props(default)] run_record: Option<CanvasRun>,
    #[props(default)] agent_nodes: Vec<AgentNode>,
    #[props(default)] trace_rows: Vec<TraceRow>,
    #[props(default)] is_live: bool,
    #[props(default)] initial_open_agent_id: Option<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_pause: Option<EventHandler<bool>>,
    #[props(default)] on_stop: Option<EventHandler<()>>,
    #[props(default)] on_open_agent: Option<EventHandler<String>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] on_resolve: Option<EventHandler<(String, String)>>,
) -> Element {
    let uid = use_hook(next_id);
    let mut open_id = use_signal(move || initial_open_agent_id.clone());
    let mut tab = use_signal(|| 0usize);
    let mut stop_asked = use_signal(|| false);
    let mut paused_local = use_signal(|| None::<bool>);
    let mut announce = use_signal(String::new);
    let zoom = use_signal(|| 1.0_f64);

    let state = run_record.as_ref().map(|r| r.run_state);
    let paused = paused_local().unwrap_or(state == Some(CanvasRunState::Paused));
    let positions = layout(&agent_nodes);
    let link_list = edges(&agent_nodes);
    let span = trace_rows
        .iter()
        .map(|r| r.start_seconds + r.duration_seconds.unwrap_or(0.0))
        .fold(
            run_record.as_ref().map_or(1.0, |r| r.elapsed_seconds),
            f64::max,
        )
        .max(1.0);
    let open_node = open_id().and_then(|id| agent_nodes.iter().find(|n| n.id == id).cloned());
    let open_trace: Vec<TraceRow> = open_node.as_ref().map_or_else(Vec::new, |n| {
        trace_rows
            .iter()
            .filter(|r| r.agent_id == n.id)
            .cloned()
            .collect()
    });
    let done_text = run_record.as_ref().map_or_else(String::new, |r| {
        format!("{} of {} agents done", r.agents_done, r.agents_total)
    });
    let tab_now = tab();
    let tab_names = ["Trace", "Tokens and cost", "Events"];
    let order: Vec<String> = agent_nodes.iter().map(|n| n.id.clone()).collect();
    let key_order = order.clone();
    let focus_base = format!("kit-amc-{uid}-node");
    let canvas_id = format!("kit-amc-{uid}-canvas");
    let canvas_width = use_element_width(canvas_id.clone());
    let zoom_now = zoom();
    let zoom_style = format!("zoom: {zoom_now:.2};");
    let zoom_text = format!("{:.0}%", zoom_now * 100.0);
    let elapsed_now = run_record.as_ref().map_or(0.0, |r| r.elapsed_seconds);
    let now_style = format!(
        "left: {:.2}%;",
        (elapsed_now / span * 100.0).clamp(0.0, 100.0)
    );

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-amc",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            if is_loading {
                div { class: "kit-amc__ph kit-amc__ph--canvas", "aria-hidden": "true" }
            } else if is_empty || run_record.is_none() {
                div { class: "kit-amc__empty",
                    p { "No agents yet. The orchestrator adds agents as it plans the run." }
                }
            } else if let Some(run) = run_record.clone() {
                {
                    let title = run.entry_title.clone();
                    let run_key = run.run_state.key();
                    let run_label = run.run_state.label();
                    let short_id = run.id.chars().take(6).collect::<String>();
                    let run_name = format!("{title} · {short_id}");
                    let elapsed = format!("{:.0} s", run.elapsed_seconds);
                    let tokens = format!("{}", run.token_count);
                    let cost = run.cost_label.clone();
                    let progress = format!("{}", run.agents_done);
                    let total = format!("{}", run.agents_total);
                    let pause_label = if paused { "Resume run" } else { "Pause run" };
                    let live_text = if is_live { "Live" } else { "Frozen" };
                    rsx! {
                        header { class: "kit-amc__head",
                            div { class: "kit-amc__heading",
                                h2 { class: "kit-amc__title", "{run_name}" }
                                span { class: "kit-amc__state", "data-state": "{run_key}", "{run_label}" }
                                span { class: "kit-amc__live", "data-live": "{is_live}", "{live_text}" }
                            }
                            div { class: "kit-amc__actions",
                                button {
                                    r#type: "button",
                                    class: "kit-amc__ghost",
                                    onclick: move |_| {
                                        let next = !paused;
                                        paused_local.set(Some(next));
                                        announce.set(if next { "Run paused".to_string() } else { "Run resumed".to_string() });
                                        if let Some(handler) = on_pause {
                                            handler.call(next);
                                        }
                                    },
                                    "{pause_label}"
                                }
                                if stop_asked() {
                                    span { class: "kit-amc__confirm", role: "group", "aria-label": "Confirm stop",
                                        span { "Stop the whole run?" }
                                        button {
                                            r#type: "button",
                                            class: "kit-amc__danger",
                                            onclick: move |_| {
                                                stop_asked.set(false);
                                                if let Some(handler) = on_stop {
                                                    handler.call(());
                                                }
                                            },
                                            "Stop run"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-amc__ghost",
                                            onclick: move |_| stop_asked.set(false),
                                            "Keep running"
                                        }
                                    }
                                } else {
                                    button {
                                        r#type: "button",
                                        class: "kit-amc__danger",
                                        onclick: move |_| stop_asked.set(true),
                                        "Stop"
                                    }
                                }
                            }
                        }

                        div { class: "kit-amc__main",
                            div { class: "kit-amc__canvas-wrap",
                                div { class: "kit-amc__zoom", role: "group", "aria-label": "Canvas zoom",
                                    span { class: "kit-amc__zoom-value", "{zoom_text}" }
                                    button {
                                        r#type: "button",
                                        class: "kit-amc__ghost",
                                        "aria-label": "Zoom out",
                                        disabled: zoom_now <= ZOOM_MIN,
                                        onclick: move |_| set_zoom(zoom, announce, zoom_step(zoom(), -1)),
                                        "−"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "kit-amc__ghost",
                                        "aria-label": "Zoom in",
                                        disabled: zoom_now >= ZOOM_MAX,
                                        onclick: move |_| set_zoom(zoom, announce, zoom_step(zoom(), 1)),
                                        "+"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "kit-amc__ghost",
                                        "aria-label": "Fit to view",
                                        onclick: move |_| set_zoom(zoom, announce, fit_zoom(canvas_width)),
                                        "Fit"
                                    }
                                }
                                div {
                                id: "{canvas_id}",
                                class: "kit-amc__canvas",
                                role: "region",
                                tabindex: "0",
                                "aria-label": "Agents",
                                onkeydown: {
                                    let pan_id = canvas_id.clone();
                                    move |event: KeyboardEvent| {
                                        match event.key() {
                                            Key::Character(c) => match c.as_str() {
                                                "+" | "=" => set_zoom(zoom, announce, zoom_step(zoom(), 1)),
                                                "-" | "_" => set_zoom(zoom, announce, zoom_step(zoom(), -1)),
                                                "f" | "F" => set_zoom(zoom, announce, fit_zoom(canvas_width)),
                                                _ => {}
                                            },
                                            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                                                event.prevent_default();
                                                let (dx, dy) = match event.key() {
                                                    Key::ArrowLeft => (-PAN_STEP, 0),
                                                    Key::ArrowRight => (PAN_STEP, 0),
                                                    Key::ArrowUp => (0, -PAN_STEP),
                                                    _ => (0, PAN_STEP),
                                                };
                                                pan_canvas(&pan_id, dx, dy);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                                div { class: "kit-amc__stage", style: "{zoom_style}",
                                    svg { class: "kit-amc__edges", view_box: "0 0 100 100", "aria-hidden": "true",
                                        for (parent, child) in link_list.iter().cloned() {
                                            {
                                                let from = positions.iter().find(|(id, _, _)| *id == parent).map(|(_, x, y)| (*x, *y));
                                                let to = positions.iter().find(|(id, _, _)| *id == child).map(|(_, x, y)| (*x, *y));
                                                let key = format!("{parent}-{child}");
                                                match (from, to) {
                                                    (Some((x1, y1)), Some((x2, y2))) => {
                                                        let (x1s, y1s, x2s, y2s) = (format!("{x1:.2}"), format!("{y1:.2}"), format!("{x2:.2}"), format!("{y2:.2}"));
                                                        rsx! { line { key: "{key}", class: "kit-amc__edge", x1: "{x1s}", y1: "{y1s}", x2: "{x2s}", y2: "{y2s}" } }
                                                    }
                                                    _ => rsx! {},
                                                }
                                            }
                                        }
                                    }
                                    for node in agent_nodes.iter().cloned() {
                                        {
                                            let (x, y) = positions.iter().find(|(id, _, _)| *id == node.id).map(|(_, x, y)| (*x, *y)).unwrap_or((50.0, 50.0));
                                            let style = format!("left: {x:.2}%; top: {y:.2}%;");
                                            let node_key = node.id.clone();
                                            let node_state = node.node_state.key();
                                            let node_label = format!(
                                                "{}, {}, {}, {}, {}",
                                                node.entry_name,
                                                node.node_role,
                                                node.node_state.label(),
                                                node.model_name,
                                                node_cost(&node)
                                            );
                                            let node_name = node.entry_name.clone();
                                            let node_role = node.node_role.clone();
                                            let node_badge = node.node_state.label();
                                            let node_cost_text = node_cost(&node);
                                            let pick = node.id.clone();
                                            let focus_id = format!("{focus_base}-{}", node.id);
                                            let is_open = open_id().as_deref() == Some(node.id.as_str());
                                            let queued = node.node_state == NodeState::Queued;
                                            let order_here = key_order.clone();
                                            let spatial_here = positions.clone();
                                            let here = node.id.clone();
                                            rsx! {
                                                button {
                                                    key: "{node_key}",
                                                    id: "{focus_id}",
                                                    r#type: "button",
                                                    class: "kit-amc__node",
                                                    style: "{style}",
                                                    "data-state": "{node_state}",
                                                    "data-queued": "{queued}",
                                                    "aria-label": "{node_label}",
                                                    "aria-current": is_open.then_some("true"),
                                                    onclick: move |_| {
                                                        open_id.set(Some(pick.clone()));
                                                        if let Some(handler) = on_open_agent {
                                                            handler.call(pick.clone());
                                                        }
                                                    },
                                                    onkeydown: {
                                                        let base = focus_base.clone();
                                                        let spatial = spatial_here.clone();
                                                        move |event: KeyboardEvent| {
                                                            let (dx, dy): (i8, i8) = match event.key() {
                                                                Key::ArrowRight => (1, 0),
                                                                Key::ArrowLeft => (-1, 0),
                                                                Key::ArrowDown => (0, 1),
                                                                Key::ArrowUp => (0, -1),
                                                                Key::Home | Key::End => {
                                                                    event.stop_propagation();
                                                                    let target = if event.key() == Key::Home {
                                                                        order_here.first()
                                                                    } else {
                                                                        order_here.last()
                                                                    };
                                                                    if let Some(target) = target {
                                                                        focus_after_frame(&format!("{base}-{target}"));
                                                                    }
                                                                    return;
                                                                }
                                                                _ => return,
                                                            };
                                                            event.stop_propagation();
                                                            if let Some(target) = nearest_in_direction(&spatial, &here, dx, dy) {
                                                                focus_after_frame(&format!("{base}-{target}"));
                                                            }
                                                        }
                                                    },
                                                    span { class: "kit-amc__node-name", "{node_name}" }
                                                    span { class: "kit-amc__node-role", "{node_role}" }
                                                    span { class: "kit-amc__badge", "{node_badge}" }
                                                    span { class: "kit-amc__node-cost", "{node_cost_text}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            }

                            section { class: "kit-amc__tiles", "aria-label": "Run figures",
                                div { class: "kit-amc__tile",
                                    span { class: "kit-amc__label", "Agents done" }
                                    span { class: "kit-amc__value", "{progress} of {total}" }
                                }
                                div { class: "kit-amc__tile",
                                    span { class: "kit-amc__label", "Elapsed" }
                                    span { class: "kit-amc__value", "{elapsed}" }
                                }
                                div { class: "kit-amc__tile",
                                    span { class: "kit-amc__label", "Tokens" }
                                    span { class: "kit-amc__value", "{tokens}" }
                                }
                                div { class: "kit-amc__tile",
                                    span { class: "kit-amc__label", "Cost" }
                                    span { class: "kit-amc__value", "{cost}" }
                                }
                            }

                            div { class: "kit-amc__lower",
                                div { class: "kit-amc__tabs", role: "tablist", "aria-label": "Run detail",
                                    for (index, name) in tab_names.iter().enumerate() {
                                        {
                                            let tab_key = format!("{index}");
                                            let tab_selected = (tab_now == index).to_string();
                                            let tab_focus = format!("kit-amc-{uid}-tab-{index}");
                                            rsx! {
                                                button {
                                                    key: "{tab_key}",
                                                    id: "{tab_focus}",
                                                    r#type: "button",
                                                    role: "tab",
                                                    class: "kit-amc__tab",
                                                    "aria-selected": "{tab_selected}",
                                                    onclick: move |_| tab.set(index),
                                                    onkeydown: move |event: KeyboardEvent| {
                                                        let next = match event.key() {
                                                            Key::ArrowRight => (index + 1) % 3,
                                                            Key::ArrowLeft => (index + 2) % 3,
                                                            _ => return,
                                                        };
                                                        tab.set(next);
                                                        focus_after_frame(&format!("kit-amc-{uid}-tab-{next}"));
                                                    },
                                                    "{name}"
                                                }
                                            }
                                        }
                                    }
                                }
                                if tab_now == 0 {
                                    ul { class: "kit-amc__trace", role: "list", "aria-label": "Trace",
                                        for row in trace_rows.iter().cloned() {
                                            {
                                                let row_key = row.id.clone();
                                                let mut shown = row.clone();
                                                if shown.duration_seconds.is_none() && is_live {
                                                    shown.duration_seconds = Some((elapsed_now - shown.start_seconds).max(0.0));
                                                }
                                                let (start, width) = bar_percent(&shown, span);
                                                let bar = format!("left: {start:.2}%; width: {width:.2}%;");
                                                let row_state = row.row_state.key();
                                                let row_label = format!("{} · {}", row.entry_label, row_state);
                                                let row_error = row.error_text.clone().unwrap_or_default();
                                                let agent = row.agent_id.clone();
                                                rsx! {
                                                    li { key: "{row_key}", class: "kit-amc__trace-row", "data-state": "{row_state}",
                                                        button {
                                                            r#type: "button",
                                                            class: "kit-amc__trace-hit",
                                                            "aria-label": "{row_label}",
                                                            onclick: move |_| {
                                                                open_id.set(Some(agent.clone()));
                                                                if let Some(handler) = on_open_agent {
                                                                    handler.call(agent.clone());
                                                                }
                                                            },
                                                            span { class: "kit-amc__trace-name", "{row_label}" }
                                                            span { class: "kit-amc__track",
                                                                span { class: "kit-amc__bar", style: "{bar}" }
                                                                if is_live {
                                                                    span { class: "kit-amc__now", style: "{now_style}", "aria-hidden": "true" }
                                                                }
                                                            }
                                                        }
                                                        if !row_error.is_empty() {
                                                            span { class: "kit-amc__error", role: "alert", "{row_error}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else if tab_now == 1 {
                                    table { class: "kit-amc__table",
                                        thead { tr { th { "Agent" } th { "Tokens" } th { "Cost" } } }
                                        tbody {
                                            for node in agent_nodes.iter().cloned() {
                                                {
                                                    let cell_key = node.id.clone();
                                                    let cell_name = node.entry_name.clone();
                                                    let cell_tokens = node.token_count.map_or_else(|| "—".to_string(), |t| t.to_string());
                                                    let cell_cost = node_cost(&node);
                                                    rsx! { tr { key: "{cell_key}", td { "{cell_name}" } td { "{cell_tokens}" } td { "{cell_cost}" } } }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    ol { class: "kit-amc__events",
                                        for row in trace_rows.iter().cloned() {
                                            {
                                                let event_key = row.id.clone();
                                                let event_text = format!("{:.0} s · {} · {}", row.start_seconds, row.entry_label, row.row_state.key());
                                                rsx! { li { key: "{event_key}", "{event_text}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(node) = open_node.clone() {
                    {
                        let close_focus = format!("{focus_base}-{}", node.id);
                        let panel_title = node.entry_name.clone();
                        let panel_role = node.node_role.clone();
                        let panel_state = node.node_state.label();
                        let panel_cost = node_cost(&node);
                        let panel_tokens = node.token_count.map_or_else(|| "—".to_string(), |t| t.to_string());
                        let panel_model = node.model_name.clone();
                        let asks_here = node.node_state == NodeState::NeedsYou;
                        let approve_id = node.id.clone();
                        let decline_id = node.id.clone();
                        let task_lines: Vec<String> = open_trace.iter().map(|r| format!("{} · {}", r.entry_label, r.row_state.key())).collect();
                        rsx! {
                            aside { class: "kit-amc__panel", "aria-label": "Agent detail",
                                onkeydown: move |event: KeyboardEvent| {
                                    if event.key() == Key::Escape {
                                        open_id.set(None);
                                        focus_after_frame(&close_focus);
                                    }
                                },
                                h3 { class: "kit-amc__panel-title", "{panel_title}" }
                                p { class: "kit-amc__meta", "{panel_role} · {panel_model}" }
                                p { class: "kit-amc__meta", "{panel_state}" }
                                dl { class: "kit-amc__facts",
                                    dt { "Cost" }
                                    dd { "{panel_cost}" }
                                    dt { "Tokens" }
                                    dd { "{panel_tokens}" }
                                }
                                section { class: "kit-amc__block", "aria-label": "Tasks",
                                    h4 { class: "kit-amc__section", "Tasks" }
                                    if task_lines.is_empty() {
                                        p { class: "kit-amc__meta", "No tasks yet." }
                                    }
                                    ul { class: "kit-amc__list",
                                        for (index, line) in task_lines.iter().enumerate() {
                                            li { key: "{index}", "{line}" }
                                        }
                                    }
                                }
                                if asks_here {
                                    section { class: "kit-amc__block kit-amc__ask", "aria-label": "Waiting for you",
                                        h4 { class: "kit-amc__section", "Waiting for you" }
                                        div { class: "kit-amc__row-actions",
                                            button {
                                                r#type: "button",
                                                class: "kit-amc__ghost",
                                                onclick: move |_| {
                                                    if let Some(handler) = on_resolve {
                                                        handler.call((decline_id.clone(), "decline".to_string()));
                                                    }
                                                },
                                                "Decline"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-amc__primary",
                                                onclick: move |_| {
                                                    if let Some(handler) = on_resolve {
                                                        handler.call((approve_id.clone(), "approve".to_string()));
                                                    }
                                                },
                                                "Approve"
                                            }
                                        }
                                    }
                                }
                                if let Some(nav) = on_navigate {
                                    button {
                                        r#type: "button",
                                        class: "kit-amc__link",
                                        onclick: move |_| nav.call("Thread".to_string()),
                                        "Open in Thread"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            p { class: "kit-sr", role: "status", "aria-live": "polite", "{done_text}" }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
