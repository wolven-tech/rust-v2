//! Automation canvas: a flow of linked steps on a pannable, zoomable board. Steps are placed by dragging from the palette,
//! linked by dragging between ports, from the settings list, or by inserting a step on a link. Nudged by drag or Alt+arrow,
//! and walked with the keyboard. A test run feeds the sample payload through the flow and reports each step as it finishes.
//!
//! Design choice: the flow editor in `components` is a top-down list with its own state, so it does not fit a free board
//! with positioned nodes and yes/no ports. Its `StepIcon` glyph set marks the palette items.
//! Design choice: the board is one tab stop until a step is selected; the selected step then holds the tab stop and
//! arrows move DOM focus between steps (roving focus).
//! Design choice: the board clips nodes; pins, link controls, the node picker and the control strip are siblings of it,
//! so they are never cut off by its overflow and their keys never reach the board's arrow handling.
//! Design choice: tooltips are native `title` attributes that carry the shortcut.
//! Design choice: the sidebar is drawn inline so the screen stands alone, without the workspace frame.
//! Design choice: the flow opens fitted to the board, at zoom 1 at most; zoom floors at 0.5 so steps stay legible.
//! Design choice: the flow is seeded from `canvas_nodes`, `canvas_edges` and `canvas_comments` at mount; a host that
//! opens another flow keys the component by the flow id. `is_loading` and `is_empty` apply live.
//! Design choice: touch pinch is taken from the two pointers that go down on the empty backdrop; a step under a finger
//! starts a move instead. Space held over the backdrop pans, as a pointer drag does.
//! Design choice: Escape closes the topmost layer in this order: link drag, node picker, comment, settings panel, palette
//! sheet, run area, comment tool, selection. Each close returns focus to the control that opened the layer.
//! Not built: the add control on a link opens the picker (no separate inline menu); comments resolve from their popover.
//! Browser checks: pointer drag, touch pinch and pan, Space pan, wheel zoom, focus movement, Escape order and the run timeline
//! are verified in the gallery, not in the SSR tests.

use std::collections::BTreeMap;
use std::rc::Rc;

use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

use crate::components::automation_flow_editor::StepIcon;
use crate::internal::*;

const CSS: Asset = asset!("/assets/automation-canvas.css");

/// Pixels a nudge moves a step, and the larger step Alt+Shift uses.
pub const NUDGE: f64 = 8.0;
pub const NUDGE_FAR: f64 = 40.0;

/// Delay per step during a test run, in milliseconds.
pub const STEP_MS: u32 = 400;

/// Card width in board units. The stylesheet sizes the card to match.
pub const NODE_W: f64 = 170.0;
/// Card height in board units for an ordinary step; triggers and tools are taller, see `node_height`.
pub const NODE_H: f64 = 88.0;

/// Card height in board units for a kind. Triggers are the tallest card, tools a little taller than
/// the rest; the fit and the port positions both read it.
pub fn node_height(kind: NodeKind) -> f64 {
    match kind {
        NodeKind::Trigger => 110.0,
        NodeKind::Tool => 92.0,
        _ => NODE_H,
    }
}

pub const MIN_ZOOM: f64 = 0.5;
pub const MAX_ZOOM: f64 = 1.5;
const ZOOM_FACTOR: f64 = 1.25;
/// A fitted flow spans this share of the board width and height, and its top edge sits this far down.
const FIT_WIDTH: f64 = 0.77;
const FIT_HEIGHT: f64 = 0.86;
const FIT_TOP: f64 = 0.03;
const PAN_STEP: f64 = 40.0;
const DRAG_SLOP: f64 = 4.0;

pub const RUN_MIN: f64 = 120.0;
pub const RUN_MAX: f64 = 480.0;
const RUN_DEFAULT: f64 = 120.0;
const RUN_STEP: f64 = 20.0;

const TABS: [(&str, &str); 5] = [
    ("payload", "Payload"),
    ("triggers", "Triggers"),
    ("errors", "Errors"),
    ("timeline", "Timeline"),
    ("runs", "Runs"),
];

/// Palette items in group order: group name, the kind the step takes, and the item's label.
const PALETTE: [(&str, NodeKind, &str); 18] = [
    ("Triggers", NodeKind::Trigger, "Webhook"),
    ("Triggers", NodeKind::Trigger, "Schedule"),
    ("Triggers", NodeKind::Trigger, "Form submitted"),
    ("Triggers", NodeKind::Trigger, "Email received"),
    ("Tools", NodeKind::Tool, "HTTP request"),
    ("Tools", NodeKind::Tool, "Sheet row"),
    ("Tools", NodeKind::Tool, "Database query"),
    ("Tools", NodeKind::Tool, "Web search"),
    ("Tools", NodeKind::Agent, "Assistant step"),
    ("Logic", NodeKind::Logic, "Branch"),
    ("Logic", NodeKind::Logic, "Wait"),
    ("Logic", NodeKind::Logic, "Loop"),
    ("Logic", NodeKind::Logic, "Merge"),
    ("Actions", NodeKind::Action, "Send message"),
    ("Actions", NodeKind::Action, "Send email"),
    ("Actions", NodeKind::Action, "Create record"),
    ("Actions", NodeKind::Action, "Post to channel"),
    ("Actions", NodeKind::Action, "Update record"),
];

const GROUP_NAMES: [&str; 4] = ["Triggers", "Tools", "Logic", "Actions"];

const NAV: [&str; 9] = [
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Trigger,
    Tool,
    Agent,
    Logic,
    Action,
}

impl NodeKind {
    pub fn word(self) -> &'static str {
        match self {
            Self::Trigger => "Trigger",
            Self::Tool => "Tool",
            Self::Agent => "Agent",
            Self::Logic => "Logic",
            Self::Action => "Action",
        }
    }

    /// The icon glyph this kind's palette item shows, taken from the shared step icons.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Trigger => StepIcon::Webhook.glyph(),
            Self::Tool => StepIcon::Http.glyph(),
            Self::Agent => StepIcon::Message.glyph(),
            Self::Logic => StepIcon::Branch.glyph(),
            Self::Action => StepIcon::Task.glyph(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NodeStatus {
    #[default]
    Unset,
    Ready,
    Running,
    Done,
    Failed,
}

impl NodeStatus {
    pub fn word(self) -> &'static str {
        match self {
            Self::Unset => "Not configured",
            Self::Ready => "Ready",
            Self::Running => "Running",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Port {
    #[default]
    Out,
    Yes,
    No,
}

impl Port {
    pub fn word(self) -> &'static str {
        match self {
            Self::Out => "output",
            Self::Yes => "yes",
            Self::No => "no",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasNode {
    pub id: String,
    pub node_type: NodeKind,
    pub node_title: String,
    pub subtitle: Option<String>,
    pub pos_x: f64,
    pub pos_y: f64,
    pub node_status: NodeStatus,
    /// The header chip of an agent, tool or logic card ("3 tools", "GET", "2 paths"). Triggers and
    /// actions show their status word there instead.
    pub chip: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasEdge {
    pub id: String,
    pub source_node: String,
    pub source_port: Port,
    pub target_node: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasComment {
    pub id: String,
    pub pos_x: f64,
    pub pos_y: f64,
    pub comment_text: String,
    pub author_name: Option<String>,
    pub resolved: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepResult {
    pub duration: Option<f64>,
    pub credit_cost: Option<f64>,
    pub error_message: Option<&'static str>,
}

/// Pan offset in screen pixels and zoom factor of the board.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
}

impl Default for View {
    fn default() -> Self {
        Self {
            pan_x: 24.0,
            pan_y: 24.0,
            zoom: 1.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RunStep {
    status: NodeStatus,
    secs: Option<f64>,
    error: Option<String>,
}

/// Steps reachable from the trigger, in the order links lead to them, each once.
pub fn run_order(nodes: &[CanvasNode], edges: &[CanvasEdge]) -> Vec<String> {
    let Some(start) = nodes
        .iter()
        .find(|n| n.node_type == NodeKind::Trigger)
        .or_else(|| nodes.first())
    else {
        return Vec::new();
    };
    let mut order = vec![start.id.clone()];
    let mut cursor = 0;
    while cursor < order.len() {
        let current = order[cursor].clone();
        for edge in edges.iter().filter(|e| e.source_node == current) {
            if !order.contains(&edge.target_node) {
                order.push(edge.target_node.clone());
            }
        }
        cursor += 1;
    }
    order
}

/// The step a test run stops at: the first whose result carries an error message.
pub fn failing_step(order: &[String], results: &BTreeMap<String, StepResult>) -> Option<String> {
    order
        .iter()
        .find(|id| results.get(*id).is_some_and(|r| r.error_message.is_some()))
        .cloned()
}

/// Spoken duration: minutes and seconds, or seconds alone under a minute.
pub fn spoken_duration(seconds: f64) -> String {
    let total = seconds.max(0.0).round() as u64;
    let (m, s) = (total / 60, total % 60);
    if m == 0 {
        format!("{s} seconds")
    } else {
        format!("{m} minutes {s} seconds")
    }
}

/// The node an arrow key reaches from `current`: Down follows the first outgoing link, Up the first incoming one,
/// Left and Right the nearest node in the same row. None when no step lies that way.
pub fn arrow_target(
    nodes: &[CanvasNode],
    edges: &[CanvasEdge],
    current: &str,
    key: &str,
) -> Option<String> {
    let here = nodes.iter().find(|n| n.id == current)?;
    match key {
        "down" => edges
            .iter()
            .find(|e| e.source_node == current)
            .map(|e| e.target_node.clone()),
        "up" => edges
            .iter()
            .find(|e| e.target_node == current)
            .map(|e| e.source_node.clone()),
        "left" | "right" => nodes
            .iter()
            .filter(|n| n.id != current && (n.pos_y - here.pos_y).abs() < 40.0)
            .filter(|n| {
                if key == "right" {
                    n.pos_x > here.pos_x
                } else {
                    n.pos_x < here.pos_x
                }
            })
            .min_by(|a, b| {
                (a.pos_x - here.pos_x)
                    .abs()
                    .total_cmp(&(b.pos_x - here.pos_x).abs())
            })
            .map(|n| n.id.clone()),
        _ => None,
    }
}

/// Reading order: by row, then by column, as the board reads.
pub fn reading_order(nodes: &[CanvasNode]) -> Vec<String> {
    let mut sorted: Vec<&CanvasNode> = nodes.iter().collect();
    sorted.sort_by(|a, b| {
        (a.pos_y as i64 / 40)
            .cmp(&(b.pos_y as i64 / 40))
            .then(a.pos_x.total_cmp(&b.pos_x))
    });
    sorted.into_iter().map(|n| n.id.clone()).collect()
}

/// The output ports a kind exposes: a branch has yes and no, every other kind one output.
pub fn ports_of(kind: NodeKind) -> &'static [Port] {
    match kind {
        NodeKind::Logic => &[Port::Yes, Port::No],
        _ => &[Port::Out],
    }
}

/// Label of the settings control that links a port.
pub fn port_caption(port: Port) -> &'static str {
    match port {
        Port::Out => "Connect to…",
        Port::Yes => "Connect yes to…",
        Port::No => "Connect no to…",
    }
}

/// Where a port sits in board units: outputs on the bottom edge; a branch's yes and no at 29% and 71% of the width.
pub fn port_point(node: &CanvasNode, port: Port) -> (f64, f64) {
    let y = node.pos_y + node_height(node.node_type);
    match port {
        Port::Out => (node.pos_x + NODE_W / 2.0, y),
        Port::Yes => (node.pos_x + NODE_W * 0.29, y),
        Port::No => (node.pos_x + NODE_W * 0.71, y),
    }
}

/// The input port of a card: the middle of its top edge.
pub fn input_point(node: &CanvasNode) -> (f64, f64) {
    (node.pos_x + NODE_W / 2.0, node.pos_y)
}

/// A straight line when the ends share a column, otherwise a vertical S-curve; the flow reads top to bottom.
pub fn edge_path(from: (f64, f64), to: (f64, f64)) -> String {
    let (x1, y1) = from;
    let (x2, y2) = to;
    if (x1 - x2).abs() < 0.5 {
        return format!("M {x1:.1} {y1:.1} L {x2:.1} {y2:.1}");
    }
    let reach = ((y2 - y1).abs() / 2.0).max(40.0);
    let c1 = y1 + reach;
    let c2 = y2 - reach;
    format!("M {x1:.1} {y1:.1} C {x1:.1} {c1:.1}, {x2:.1} {c2:.1}, {x2:.1} {y2:.1}")
}

/// The midpoint of a link's curve, where its add control sits. None when an end is missing.
pub fn link_midpoint(nodes: &[CanvasNode], edge: &CanvasEdge) -> Option<(f64, f64)> {
    let source = nodes.iter().find(|n| n.id == edge.source_node)?;
    let target = nodes.iter().find(|n| n.id == edge.target_node)?;
    let (x1, y1) = port_point(source, edge.source_port);
    let (x2, y2) = input_point(target);
    Some(((x1 + x2) / 2.0, (y1 + y2) / 2.0))
}

/// The flow with `step` placed on the link `edge_id`: the link is replaced by source to step and step to target.
/// None when the link does not exist.
pub fn insert_between(
    nodes: &[CanvasNode],
    edges: &[CanvasEdge],
    edge_id: &str,
    step: CanvasNode,
) -> Option<(Vec<CanvasNode>, Vec<CanvasEdge>)> {
    let link = edges.iter().find(|e| e.id == edge_id)?.clone();
    let id = step.id.clone();
    let mut next_nodes = nodes.to_vec();
    next_nodes.push(step);
    let kept: Vec<CanvasEdge> = edges.iter().filter(|e| e.id != edge_id).cloned().collect();
    let into = connect(&kept, &link.source_node, link.source_port, &id)?;
    let next_edges = connect(&into, &id, Port::Out, &link.target_node)?;
    Some((next_nodes, next_edges))
}

/// The first output of a node with no outgoing link, or the first output when every one is linked.
pub fn first_free_port(node: &CanvasNode, edges: &[CanvasEdge]) -> Port {
    let ports = ports_of(node.node_type);
    ports
        .iter()
        .copied()
        .find(|p| {
            !edges
                .iter()
                .any(|e| e.source_node == node.id && e.source_port == *p)
        })
        .unwrap_or(ports[0])
}

/// An id no node uses yet.
pub fn next_step_id(nodes: &[CanvasNode]) -> String {
    (nodes.len() + 1..)
        .map(|n| format!("step-{n}"))
        .find(|id| nodes.iter().all(|node| node.id != *id))
        .unwrap_or_default()
}

/// The edge list with a new link, or None when the link would loop onto its own step or already exists.
pub fn connect(
    edges: &[CanvasEdge],
    source: &str,
    port: Port,
    target: &str,
) -> Option<Vec<CanvasEdge>> {
    if source == target
        || edges
            .iter()
            .any(|e| e.source_node == source && e.source_port == port && e.target_node == target)
    {
        return None;
    }
    let base = format!("{source}-{}-{target}", port.word());
    let mut id = base.clone();
    let mut suffix = 2;
    while edges.iter().any(|e| e.id == id) {
        id = format!("{base}-{suffix}");
        suffix += 1;
    }
    let mut next = edges.to_vec();
    next.push(CanvasEdge {
        id,
        source_node: source.to_string(),
        source_port: port,
        target_node: target.to_string(),
    });
    Some(next)
}

/// A client point converted to board units, given the board's client origin and the current view.
pub fn board_point(client: (f64, f64), board_origin: (f64, f64), view: View) -> (f64, f64) {
    (
        (client.0 - board_origin.0 - view.pan_x) / view.zoom,
        (client.1 - board_origin.1 - view.pan_y) / view.zoom,
    )
}

/// The view after a two-pointer gesture moved from (`mid0`, `dist0`) to (`mid`, `dist`): the board point under the
/// starting midpoint stays under the current midpoint, and the zoom follows the change in spread.
pub fn pinch_view(
    origin: View,
    mid0: (f64, f64),
    dist0: f64,
    mid: (f64, f64),
    dist: f64,
    board_origin: (f64, f64),
) -> View {
    let zoom = (origin.zoom * dist / dist0.max(1.0)).clamp(MIN_ZOOM, MAX_ZOOM);
    let (bx, by) = board_point(mid0, board_origin, origin);
    View {
        pan_x: mid.0 - board_origin.0 - bx * zoom,
        pan_y: mid.1 - board_origin.1 - by * zoom,
        zoom,
    }
}

/// The view that shows every step in a board of `width` by `height` px: the flow spans `FIT_WIDTH` of the width
/// and `FIT_HEIGHT` of the height, centred across and set `FIT_TOP` down. Never enlarges past actual size, and
/// never zooms below the legibility floor.
pub fn fit_view(nodes: &[CanvasNode], width: f64, height: f64) -> View {
    if nodes.is_empty() || width <= 0.0 || height <= 0.0 {
        return View::default();
    }
    let min_x = nodes.iter().map(|n| n.pos_x).fold(f64::INFINITY, f64::min);
    let min_y = nodes.iter().map(|n| n.pos_y).fold(f64::INFINITY, f64::min);
    let max_x = nodes
        .iter()
        .map(|n| n.pos_x + NODE_W)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = nodes
        .iter()
        .map(|n| n.pos_y + node_height(n.node_type))
        .fold(f64::NEG_INFINITY, f64::max);
    let span_x = (max_x - min_x).max(1.0);
    let span_y = (max_y - min_y).max(1.0);
    let zoom = (FIT_WIDTH * width / span_x)
        .min(FIT_HEIGHT * height / span_y)
        .min(1.0)
        .clamp(MIN_ZOOM, MAX_ZOOM);
    let cx = (min_x + max_x) / 2.0;
    View {
        pan_x: width / 2.0 - cx * zoom,
        pan_y: FIT_TOP * height - min_y * zoom,
        zoom,
    }
}

/// The view after zooming to `zoom`, keeping the board point under (`cx`, `cy`) where it is.
pub fn zoom_about(view: View, zoom: f64, cx: f64, cy: f64) -> View {
    let z = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    let k = z / view.zoom;
    View {
        pan_x: cx - (cx - view.pan_x) * k,
        pan_y: cy - (cy - view.pan_y) * k,
        zoom: z,
    }
}

/// A status line for a card: the word, plus the elapsed time for a finished step.
pub fn status_line(status: NodeStatus, secs: Option<f64>) -> String {
    match (status, secs) {
        (NodeStatus::Done, Some(s)) => format!("Done · {s:.1} s"),
        _ => status.word().to_string(),
    }
}

fn dom_safe(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

fn node_dom(base: usize, id: &str) -> String {
    format!("kit-ac-{base}-n-{}", dom_safe(id))
}

fn link_dom(base: usize, id: &str) -> String {
    format!("kit-ac-{base}-l-{}", dom_safe(id))
}

fn link_add_dom(base: usize, id: &str) -> String {
    format!("kit-ac-{base}-la-{}", dom_safe(id))
}

fn pin_dom(base: usize, id: &str) -> String {
    format!("kit-ac-{base}-pin-{}", dom_safe(id))
}

fn board_dom(base: usize) -> String {
    format!("kit-ac-{base}-board")
}

fn ui_dom(base: usize, name: &str) -> String {
    format!("kit-ac-{base}-{name}")
}

fn tab_dom(base: usize, name: &str) -> String {
    format!("kit-ac-{base}-tab-{name}")
}

fn picker_dom(base: usize, index: usize) -> String {
    format!("kit-ac-{base}-pick-{index}")
}

fn focus_now(id: &str) {
    document::eval(&format!(
        "document.getElementById({id:?})?.focus({{ preventScroll: true }});"
    ));
}

/// The board's client rectangle, in CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Board {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// Runs `f` with the board's client rectangle once the browser has laid it out. Layout is read here, in an event
/// handler, never during render.
fn with_board(mount: Signal<Option<Rc<MountedData>>>, f: impl FnOnce(Board) + 'static) {
    let Some(data) = mount() else {
        return;
    };
    spawn(async move {
        if let Ok(rect) = data.get_client_rect().await {
            f(Board {
                x: rect.origin.x,
                y: rect.origin.y,
                w: rect.size.width,
                h: rect.size.height,
            });
        }
    });
}

/// What a pointer gesture is doing between pointerdown and pointerup.
#[derive(Clone, Debug, PartialEq)]
enum Drag {
    Pan {
        pointer: i32,
        start: (f64, f64),
        origin: (f64, f64),
        moved: bool,
    },
    Move {
        id: String,
        start: (f64, f64),
        origin: (f64, f64),
        moved: bool,
    },
    Link {
        from: String,
        port: Port,
        start: (f64, f64),
        from_at: (f64, f64),
        cursor: (f64, f64),
    },
    Palette {
        kind: NodeKind,
        title: &'static str,
        start: (f64, f64),
        cursor: (f64, f64),
        moved: bool,
    },
    Run {
        start: f64,
        origin: f64,
    },
    Pinch {
        dist: f64,
        mid: (f64, f64),
        view: View,
    },
}

/// Where the node picker was opened from: a step's output, or the add control on a link.
#[derive(Clone, Debug, PartialEq)]
enum Anchor {
    Port { from: String, port: Port },
    Link { edge: String },
}

/// Outputs with no link, as (step id, port, screen x, screen y of the add control under the output dot).
fn unlinked_outputs(
    nodes: &[CanvasNode],
    edges: &[CanvasEdge],
    view: View,
) -> Vec<(String, Port, f64, f64)> {
    let mut found = Vec::new();
    for node in nodes {
        for port in ports_of(node.node_type).iter().copied() {
            if edges
                .iter()
                .any(|e| e.source_node == node.id && e.source_port == port)
            {
                continue;
            }
            let (x, y) = port_point(node, port);
            found.push((
                node.id.clone(),
                port,
                view.pan_x + x * view.zoom,
                view.pan_y + (y + 14.0) * view.zoom,
            ));
        }
    }
    found
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn mid(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

fn slop(start: (f64, f64), now: (f64, f64)) -> bool {
    (now.0 - start.0).abs() + (now.1 - start.1).abs() > DRAG_SLOP
}

/// Automation canvas: sidebar, breadcrumb with publish, palette, board, settings panel and run area.
#[component]
pub fn AutomationCanvas(
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    #[props(default = "Onboarding".to_string(), into)] project_name: String,
    #[props(default = "Main".to_string(), into)] path_branch: String,
    #[props(default)] canvas_nodes: Vec<CanvasNode>,
    #[props(default)] canvas_edges: Vec<CanvasEdge>,
    #[props(default)] canvas_comments: Vec<CanvasComment>,
    #[props(default)] step_results: BTreeMap<String, StepResult>,
    #[props(default)] run_history: Vec<String>,
    #[props(default = "{}".to_string(), into)] payload: String,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_change: EventHandler<(Vec<CanvasNode>, Vec<CanvasEdge>)>,
    #[props(default)] on_comments: EventHandler<Vec<CanvasComment>>,
    #[props(default)] on_run: EventHandler<()>,
    #[props(default)] on_publish: EventHandler<()>,
    #[props(default)] on_navigate: EventHandler<String>,
) -> Element {
    let base = use_hook(next_id);
    let mut nodes = use_signal(|| canvas_nodes.clone());
    let mut edges = use_signal(|| canvas_edges.clone());
    let mut comments = use_signal(|| canvas_comments.clone());
    let mut selected = use_signal(|| None::<String>);
    let mut focus_link = use_signal(|| None::<String>);
    let mut view = use_signal(View::default);
    let mut board_mount = use_signal(|| None::<Rc<MountedData>>);
    let mut board_origin = use_signal(|| (0.0_f64, 0.0_f64));
    let mut pointers = use_signal(Vec::<(i32, (f64, f64))>::new);
    let mut space_down = use_signal(|| false);
    let mut drag = use_signal(|| None::<Drag>);
    let mut suppress_click = use_signal(|| false);
    let mut comment_tool = use_signal(|| false);
    let mut draft = use_signal(|| None::<(f64, f64)>);
    let mut draft_text = use_signal(String::new);
    let mut open_comment = use_signal(|| None::<String>);
    let mut undo = use_signal(Vec::<(CanvasNode, Vec<CanvasEdge>)>::new);
    let mut picker = use_signal(|| None::<Anchor>);
    let mut palette_open = use_signal(|| false);
    let mut side_open = use_signal(|| false);
    let mut config_open = use_signal(|| false);
    let mut run_open = use_signal(|| false);
    let mut tab = use_signal(|| "timeline");
    let mut run_h = use_signal(|| RUN_DEFAULT);
    let mut running = use_signal(|| false);
    let mut run_state = use_signal(BTreeMap::<String, RunStep>::new);
    let mut run_timeline = use_signal(Vec::<(String, f64)>::new);
    let mut run_log = use_signal(|| run_history.clone());
    let mut announce = use_signal(String::new);

    let node_now = nodes();
    let edge_now = edges();
    let comment_now = comments();
    let sel = selected().filter(|id| node_now.iter().any(|n| n.id == *id));
    let selected_node = sel
        .as_ref()
        .and_then(|id| node_now.iter().find(|n| n.id == *id).cloned());
    let outgoing: Vec<CanvasEdge> = sel
        .as_ref()
        .map(|id| {
            edge_now
                .iter()
                .filter(|e| e.source_node == *id)
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let run_now = run_state();
    let running_id: Option<String> = run_now
        .iter()
        .find(|(_, s)| s.status == NodeStatus::Running)
        .map(|(id, _)| id.clone());
    let view_now = view();
    let drag_now = drag();
    let linking = matches!(drag_now, Some(Drag::Link { .. }));
    let loading_disabled = is_loading || running() || node_now.is_empty();
    let board_class = if is_loading {
        "kit-ac__board is-loading"
    } else {
        "kit-ac__board"
    };
    let palette_class = if palette_open() {
        "kit-ac__palette is-open"
    } else {
        "kit-ac__palette"
    };
    let side_class = if side_open() {
        "kit-ac__side is-open"
    } else {
        "kit-ac__side"
    };
    let config_class = if config_open() {
        "kit-ac__config"
    } else {
        "kit-ac__config is-closed"
    };
    let run_class = if run_open() {
        "kit-ac__run"
    } else {
        "kit-ac__run is-closed"
    };
    let timeline = run_timeline();
    let log_count = run_log().len();
    let tab_now = tab();
    let failed_message = run_now.iter().find_map(|(_, s)| s.error.clone());
    let mut error_list: Vec<String> = node_now
        .iter()
        .filter(|n| n.node_status == NodeStatus::Unset)
        .map(|n| format!("{} is not configured", n.node_title))
        .collect();
    error_list.extend(failed_message.clone());
    let error_count = error_list.len();
    let branch_status = if path_branch == "Main" {
        "Production"
    } else {
        "Draft"
    };
    let board_id = board_dom(base);
    let board_tab = if sel.is_none() { "0" } else { "-1" };
    let bg_x = view_now.pan_x.rem_euclid(20.0);
    let bg_y = view_now.pan_y.rem_euclid(20.0);
    let board_style = format!("background-position: {bg_x:.1}px {bg_y:.1}px;");
    let world_style = format!(
        "transform: translate({:.1}px, {:.1}px) scale({:.3});",
        view_now.pan_x, view_now.pan_y, view_now.zoom
    );
    let run_style = format!("height: {:.0}px;", run_h());
    let zoom_pct = format!("{:.0}%", view_now.zoom * 100.0);
    let picker_now = picker();
    let draft_now = draft();
    let draft_value = draft_text();
    let anchor_point = picker_now.as_ref().and_then(|a| match a {
        Anchor::Port { from, port } => node_now
            .iter()
            .find(|n| n.id == *from)
            .map(|n| port_point(n, *port)),
        Anchor::Link { edge } => edge_now
            .iter()
            .find(|e| e.id == *edge)
            .and_then(|e| link_midpoint(&node_now, e)),
    });

    let mut commit = move |next_nodes: Vec<CanvasNode>, next_edges: Vec<CanvasEdge>| {
        nodes.set(next_nodes.clone());
        edges.set(next_edges.clone());
        on_change.call((next_nodes, next_edges));
    };

    let mut select_node = move |id: String| {
        selected.set(Some(id.clone()));
        focus_link.set(None);
        config_open.set(true);
        focus_after_frame(&node_dom(base, &id));
    };

    let add_step = move |kind: NodeKind,
                         title: &'static str,
                         pos: (f64, f64),
                         link_from: Option<(String, Port)>| {
        let mut list = nodes();
        let id = next_step_id(&list);
        let mut next_edges = edges();
        list.push(CanvasNode {
            id: id.clone(),
            node_type: kind,
            node_title: title.to_string(),
            subtitle: None,
            pos_x: pos.0.max(0.0),
            pos_y: pos.1.max(0.0),
            node_status: NodeStatus::Unset,
            chip: None,
        });
        if let Some((from, port)) = link_from {
            if let Some(linked) = connect(&next_edges, &from, port, &id) {
                next_edges = linked;
            }
        }
        nodes.set(list.clone());
        edges.set(next_edges.clone());
        selected.set(Some(id.clone()));
        focus_link.set(None);
        config_open.set(true);
        announce.set(format!("Added {title}"));
        on_change.call((list, next_edges));
        focus_after_frame(&node_dom(base, &id));
    };

    let mut nudge_node = move |id: String, dx: f64, dy: f64| {
        let next: Vec<CanvasNode> = nodes()
            .into_iter()
            .map(|mut n| {
                if n.id == id {
                    n.pos_x = (n.pos_x + dx).max(0.0);
                    n.pos_y = (n.pos_y + dy).max(0.0);
                }
                n
            })
            .collect();
        commit(next, edges());
    };

    let mut remove_step = move |id: String| {
        let list = nodes();
        let Some(removed) = list.iter().find(|n| n.id == id).cloned() else {
            return;
        };
        let gone: Vec<CanvasEdge> = edges()
            .into_iter()
            .filter(|e| e.source_node == id || e.target_node == id)
            .collect();
        let kept_nodes: Vec<CanvasNode> = list.into_iter().filter(|n| n.id != id).collect();
        let kept_edges: Vec<CanvasEdge> = edges()
            .into_iter()
            .filter(|e| e.source_node != id && e.target_node != id)
            .collect();
        undo.with_mut(|stack| stack.push((removed, gone)));
        selected.set(None);
        focus_link.set(None);
        announce.set("Step removed. Control Z restores it.".to_string());
        commit(kept_nodes, kept_edges);
        focus_after_frame(&board_dom(base));
    };

    let zoom_by = move |factor: f64| {
        with_board(board_mount, move |b| {
            let v = view();
            view.set(zoom_about(v, v.zoom * factor, b.w / 2.0, b.h / 2.0));
        });
    };

    let fit = move || {
        with_board(board_mount, move |b| {
            view.set(fit_view(&nodes(), b.w, b.h));
        });
    };

    let follow = move |centre: (f64, f64)| {
        with_board(board_mount, move |b| {
            let v = view();
            view.set(View {
                pan_x: b.w / 2.0 - centre.0 * v.zoom,
                pan_y: b.h / 2.0 - centre.1 * v.zoom,
                zoom: v.zoom,
            });
        });
    };

    let mut node_down = move |e: PointerEvent, id: String| {
        e.stop_propagation();
        if e.trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        suppress_click.set(false);
        let c = e.client_coordinates();
        let origin = nodes()
            .iter()
            .find(|n| n.id == id)
            .map(|n| (n.pos_x, n.pos_y))
            .unwrap_or((0.0, 0.0));
        selected.set(Some(id.clone()));
        focus_link.set(None);
        config_open.set(true);
        drag.set(Some(Drag::Move {
            id,
            start: (c.x, c.y),
            origin,
            moved: false,
        }));
    };

    let mut node_up = move |_e: PointerEvent, id: String| {
        let Some(Drag::Link { from, port, .. }) = drag() else {
            return;
        };
        drag.set(None);
        suppress_click.set(true);
        match connect(&edges(), &from, port, &id) {
            Some(next) => {
                announce.set(format!("Linked {} to {id}", port.word()));
                commit(nodes(), next);
            }
            None => announce.set("Those steps are already linked".to_string()),
        }
    };

    let mut port_down = move |e: PointerEvent, id: String, port: Port, at: (f64, f64)| {
        e.stop_propagation();
        if e.trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        suppress_click.set(false);
        let c = e.client_coordinates();
        drag.set(Some(Drag::Link {
            from: id,
            port,
            start: (c.x, c.y),
            from_at: at,
            cursor: at,
        }));
    };

    let board_down = move |e: PointerEvent| {
        if e.trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        let c = e.client_coordinates();
        let p = (c.x, c.y);
        let pointer = e.pointer_id();
        pointers.with_mut(|list| {
            if !list.iter().any(|(id, _)| *id == pointer) {
                list.push((pointer, p));
            }
        });
        with_board(board_mount, move |b| board_origin.set((b.x, b.y)));
        let list = pointers();
        if list.len() >= 2 {
            let (a, b) = (list[0].1, list[1].1);
            drag.set(Some(Drag::Pinch {
                dist: dist(a, b),
                mid: mid(a, b),
                view: view(),
            }));
            return;
        }
        if comment_tool() && !space_down() {
            return;
        }
        suppress_click.set(false);
        let v = view();
        drag.set(Some(Drag::Pan {
            pointer,
            start: p,
            origin: (v.pan_x, v.pan_y),
            moved: false,
        }));
    };

    let board_click = move |e: MouseEvent| {
        if suppress_click() {
            suppress_click.set(false);
            return;
        }
        if comment_tool() {
            let c = e.client_coordinates();
            let client = (c.x, c.y);
            with_board(board_mount, move |b| {
                let (x, y) = board_point(client, (b.x, b.y), view());
                draft.set(Some((x, y)));
                draft_text.set(String::new());
                focus_after_frame(&format!("kit-ac-{base}-cinput"));
            });
            return;
        }
        selected.set(None);
        focus_link.set(None);
        open_comment.set(None);
    };

    let board_wheel = move |e: WheelEvent| {
        let mods = e.modifiers();
        if !(mods.ctrl() || mods.meta()) {
            return;
        }
        e.prevent_default();
        let dy = e.delta().strip_units().y;
        let factor = if dy < 0.0 {
            ZOOM_FACTOR
        } else {
            1.0 / ZOOM_FACTOR
        };
        let c = e.client_coordinates();
        let client = (c.x, c.y);
        with_board(board_mount, move |b| {
            let v = view();
            view.set(zoom_about(
                v,
                v.zoom * factor,
                client.0 - b.x,
                client.1 - b.y,
            ));
        });
    };

    let root_move = move |e: PointerEvent| {
        let c = e.client_coordinates();
        let p = (c.x, c.y);
        let pointer = e.pointer_id();
        pointers.with_mut(|list| {
            list.iter_mut()
                .filter(|entry| entry.0 == pointer)
                .for_each(|entry| entry.1 = p);
        });
        let Some(current) = drag() else {
            return;
        };
        let z = view.peek().zoom;
        match current {
            Drag::Pinch {
                dist: dist0,
                mid: mid0,
                view: view0,
            } => {
                let list = pointers();
                if list.len() >= 2 {
                    let (a, b) = (list[0].1, list[1].1);
                    view.set(pinch_view(
                        view0,
                        mid0,
                        dist0,
                        mid(a, b),
                        dist(a, b),
                        board_origin(),
                    ));
                }
            }
            Drag::Pan {
                pointer: owner,
                start,
                origin,
                moved,
            } => {
                if owner != pointer {
                    return;
                }
                let v = view();
                let now = moved || slop(start, p);
                view.set(View {
                    pan_x: origin.0 + p.0 - start.0,
                    pan_y: origin.1 + p.1 - start.1,
                    zoom: v.zoom,
                });
                drag.set(Some(Drag::Pan {
                    pointer: owner,
                    start,
                    origin,
                    moved: now,
                }));
            }
            Drag::Move {
                id,
                start,
                origin,
                moved,
            } => {
                let dx = (p.0 - start.0) / z;
                let dy = (p.1 - start.1) / z;
                let now = moved || slop(start, p);
                nodes.with_mut(|list| {
                    list.iter_mut().filter(|n| n.id == id).for_each(|n| {
                        n.pos_x = (origin.0 + dx).max(0.0);
                        n.pos_y = (origin.1 + dy).max(0.0);
                    });
                });
                drag.set(Some(Drag::Move {
                    id,
                    start,
                    origin,
                    moved: now,
                }));
            }
            Drag::Link {
                from,
                port,
                start,
                from_at,
                ..
            } => {
                let cursor = (
                    from_at.0 + (p.0 - start.0) / z,
                    from_at.1 + (p.1 - start.1) / z,
                );
                drag.set(Some(Drag::Link {
                    from,
                    port,
                    start,
                    from_at,
                    cursor,
                }));
            }
            Drag::Palette {
                kind,
                title,
                start,
                moved,
                ..
            } => {
                let now = moved || slop(start, p);
                drag.set(Some(Drag::Palette {
                    kind,
                    title,
                    start,
                    cursor: p,
                    moved: now,
                }));
            }
            Drag::Run { start, origin } => {
                run_h.set((origin - (p.1 - start)).clamp(RUN_MIN, RUN_MAX));
            }
        }
    };

    let root_up = move |e: PointerEvent| {
        let pointer = e.pointer_id();
        pointers.with_mut(|list| list.retain(|entry| entry.0 != pointer));
        let current = drag();
        if matches!(current, Some(Drag::Pinch { .. })) {
            drag.set(None);
            suppress_click.set(true);
            return;
        }
        if let Some(Drag::Pan { pointer: owner, .. }) = current {
            if owner != pointer {
                return;
            }
        }
        drag.set(None);
        match current {
            Some(Drag::Move { moved: true, .. }) => {
                suppress_click.set(true);
                commit(nodes(), edges());
            }
            Some(Drag::Pan { moved: true, .. }) => suppress_click.set(true),
            Some(Drag::Palette {
                kind,
                title,
                cursor,
                moved: true,
                ..
            }) => {
                suppress_click.set(true);
                let client = cursor;
                with_board(board_mount, move |b| {
                    let inside = client.0 >= b.x
                        && client.0 <= b.x + b.w
                        && client.1 >= b.y
                        && client.1 <= b.y + b.h;
                    if !inside {
                        return;
                    }
                    let (x, y) = board_point(client, (b.x, b.y), view());
                    let mut add = add_step;
                    add(kind, title, (x - NODE_W / 2.0, y - NODE_H / 2.0), None);
                });
            }
            Some(Drag::Link { .. }) => suppress_click.set(true),
            _ => {}
        }
    };

    let root_cancel = move |e: PointerEvent| {
        let pointer = e.pointer_id();
        pointers.with_mut(|list| list.retain(|entry| entry.0 != pointer));
        drag.set(None);
    };

    let board_keyup = move |e: KeyboardEvent| {
        if matches!(e.key(), Key::Character(c) if c == " ") {
            space_down.set(false);
        }
    };

    let root_keys = move |e: KeyboardEvent| {
        if !matches!(e.key(), Key::Escape) {
            return;
        }
        if matches!(drag(), Some(Drag::Link { .. })) {
            drag.set(None);
            announce.set("Link cancelled".to_string());
            return;
        }
        if let Some(anchor) = picker() {
            picker.set(None);
            let opener = match anchor {
                Anchor::Port { from, .. } => node_dom(base, &from),
                Anchor::Link { edge } => link_add_dom(base, &edge),
            };
            focus_after_frame(&opener);
            return;
        }
        if let Some(name) = open_comment() {
            open_comment.set(None);
            focus_after_frame(&pin_dom(base, &name));
            return;
        }
        if draft().is_some() {
            draft.set(None);
            focus_after_frame(&ui_dom(base, "comment"));
            return;
        }
        if config_open() {
            if let Some(id) = selected() {
                config_open.set(false);
                focus_after_frame(&node_dom(base, &id));
                return;
            }
        }
        if palette_open() {
            palette_open.set(false);
            focus_after_frame(&ui_dom(base, "addnode"));
            return;
        }
        if run_open() {
            run_open.set(false);
            focus_after_frame(&ui_dom(base, "run"));
            return;
        }
        if comment_tool() {
            comment_tool.set(false);
            focus_after_frame(&ui_dom(base, "comment"));
            return;
        }
        if selected().is_some() {
            selected.set(None);
            focus_link.set(None);
            focus_after_frame(&board_dom(base));
        }
    };

    let mut board_keys = move |e: KeyboardEvent| {
        let mods = e.modifiers();
        let key = e.key();
        if matches!(&key, Key::Character(c) if c == " ") {
            e.prevent_default();
            space_down.set(true);
            return;
        }
        if mods.ctrl() || mods.meta() {
            if matches!(&key, Key::Character(c) if c.eq_ignore_ascii_case("z")) {
                e.prevent_default();
                let mut stack = undo();
                let Some((node, restored)) = stack.pop() else {
                    return;
                };
                undo.set(stack);
                let id = node.id.clone();
                let mut list = nodes();
                list.push(node);
                let mut next_edges = edges();
                for edge in restored {
                    if !next_edges.iter().any(|x| x.id == edge.id) {
                        next_edges.push(edge);
                    }
                }
                selected.set(Some(id.clone()));
                announce.set("Step restored".to_string());
                commit(list, next_edges);
                focus_after_frame(&node_dom(base, &id));
            }
            return;
        }
        let current = selected().filter(|id| nodes.peek().iter().any(|n| n.id == *id));
        match &key {
            Key::Home | Key::End => {
                e.prevent_default();
                let order = reading_order(&nodes());
                let target = if matches!(key, Key::Home) {
                    order.first().cloned()
                } else {
                    order.last().cloned()
                };
                if let Some(id) = target {
                    select_node(id);
                }
            }
            Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
                e.prevent_default();
                let dir = match &key {
                    Key::ArrowLeft => "left",
                    Key::ArrowRight => "right",
                    Key::ArrowUp => "up",
                    _ => "down",
                };
                let (ux, uy) = match dir {
                    "left" => (-1.0, 0.0),
                    "right" => (1.0, 0.0),
                    "up" => (0.0, -1.0),
                    _ => (0.0, 1.0),
                };
                if mods.shift() && !mods.alt() {
                    let mut v = view();
                    v.pan_x += ux * PAN_STEP;
                    v.pan_y += uy * PAN_STEP;
                    view.set(v);
                    return;
                }
                let Some(id) = current else {
                    if let Some(first) = reading_order(&nodes()).first().cloned() {
                        select_node(first);
                    }
                    return;
                };
                if mods.alt() {
                    let step = if mods.shift() { NUDGE_FAR } else { NUDGE };
                    nudge_node(id, ux * step, uy * step);
                    return;
                }
                if let Some(target) = arrow_target(&nodes(), &edges(), &id, dir) {
                    select_node(target);
                }
            }
            Key::Delete | Key::Backspace => {
                e.prevent_default();
                if let Some(link) = focus_link() {
                    let next: Vec<CanvasEdge> =
                        edges().into_iter().filter(|x| x.id != link).collect();
                    focus_link.set(None);
                    announce.set("Link removed".to_string());
                    commit(nodes(), next);
                    return;
                }
                if let Some(id) = current {
                    remove_step(id);
                }
            }
            Key::Enter => {
                e.prevent_default();
                config_open.set(true);
                focus_after_frame(&ui_dom(base, "config"));
            }
            Key::Character(c) => match c.as_str() {
                "e" | "E" => {
                    e.prevent_default();
                    let Some(id) = current else {
                        return;
                    };
                    config_open.set(true);
                    match edges().iter().find(|x| x.source_node == id).cloned() {
                        Some(link) => {
                            focus_link.set(Some(link.id.clone()));
                            focus_after_frame(&link_dom(base, &link.id));
                        }
                        None => announce.set("This step has no links".to_string()),
                    }
                }
                "a" | "A" => {
                    e.prevent_default();
                    let Some(id) = current else {
                        return;
                    };
                    let list = nodes();
                    if let Some(node) = list.iter().find(|n| n.id == id) {
                        let port = first_free_port(node, &edges());
                        picker.set(Some(Anchor::Port { from: id, port }));
                        focus_after_frame(&picker_dom(base, 0));
                    }
                }
                "c" | "C" => {
                    e.prevent_default();
                    comment_tool.set(true);
                    announce
                        .set("Comment tool on. Click the board to place a comment.".to_string());
                }
                "+" | "=" => {
                    e.prevent_default();
                    zoom_by(ZOOM_FACTOR);
                }
                "-" | "_" => {
                    e.prevent_default();
                    zoom_by(1.0 / ZOOM_FACTOR);
                }
                "0" => {
                    e.prevent_default();
                    fit();
                }
                _ => {}
            },
            _ => {}
        }
    };

    let add_from_picker = move |kind: NodeKind, title: &'static str| {
        let Some(anchor) = picker() else {
            return;
        };
        picker.set(None);
        let list = nodes();
        match anchor {
            Anchor::Port { from, port } => {
                let Some(source) = list.iter().find(|n| n.id == from).cloned() else {
                    return;
                };
                let at = port_point(&source, port);
                let mut add = add_step;
                add(
                    kind,
                    title,
                    (at.0 - NODE_W / 2.0, at.1 + 48.0),
                    Some((from.clone(), port)),
                );
            }
            Anchor::Link { edge } => {
                let Some(at) = edges()
                    .iter()
                    .find(|e| e.id == edge)
                    .and_then(|e| link_midpoint(&list, e))
                else {
                    return;
                };
                let id = next_step_id(&list);
                let step = CanvasNode {
                    id: id.clone(),
                    node_type: kind,
                    node_title: title.to_string(),
                    subtitle: None,
                    pos_x: (at.0 - NODE_W / 2.0).max(0.0),
                    pos_y: (at.1 - NODE_H / 2.0).max(0.0),
                    node_status: NodeStatus::Unset,
                    chip: None,
                };
                if let Some((next_nodes, next_edges)) = insert_between(&list, &edges(), &edge, step)
                {
                    nodes.set(next_nodes.clone());
                    edges.set(next_edges.clone());
                    selected.set(Some(id.clone()));
                    focus_link.set(None);
                    config_open.set(true);
                    announce.set(format!("Inserted {title} on the link"));
                    on_change.call((next_nodes, next_edges));
                    focus_after_frame(&node_dom(base, &id));
                }
            }
        }
    };

    let post_draft = move |_: ()| {
        let text = draft_text().trim().to_string();
        let Some((x, y)) = draft() else {
            return;
        };
        if text.is_empty() {
            return;
        }
        let mut list = comments();
        let mut n = list.len() + 1;
        while list.iter().any(|c| c.id == format!("comment-{n}")) {
            n += 1;
        }
        list.push(CanvasComment {
            id: format!("comment-{n}"),
            pos_x: x,
            pos_y: y,
            comment_text: text,
            author_name: Some("You".to_string()),
            resolved: false,
        });
        comments.set(list.clone());
        on_comments.call(list);
        draft.set(None);
        draft_text.set(String::new());
        announce.set("Comment posted".to_string());
    };

    let toggle_resolved = move |id: String| {
        let mut list = comments();
        let mut now_resolved = false;
        if let Some(c) = list.iter_mut().find(|c| c.id == id) {
            c.resolved = !c.resolved;
            now_resolved = c.resolved;
        }
        comments.set(list.clone());
        on_comments.call(list);
        announce.set(if now_resolved {
            "Comment resolved".to_string()
        } else {
            "Comment reopened".to_string()
        });
    };

    let start_run = move |_e: MouseEvent| {
        if running() {
            return;
        }
        let list = nodes();
        let order = run_order(&list, &edges());
        let titles: BTreeMap<String, String> = list
            .iter()
            .map(|n| (n.id.clone(), n.node_title.clone()))
            .collect();
        let centres: BTreeMap<String, (f64, f64)> = list
            .iter()
            .map(|n| {
                (
                    n.id.clone(),
                    (n.pos_x + NODE_W / 2.0, n.pos_y + NODE_H / 2.0),
                )
            })
            .collect();
        let results = step_results.clone();
        running.set(true);
        run_state.set(BTreeMap::new());
        run_timeline.set(Vec::new());
        announce.set("Test run started".to_string());
        on_run.call(());
        spawn(async move {
            let failure = failing_step(&order, &results);
            let mut outcome = "Finished".to_string();
            for id in order.iter() {
                let title = titles.get(id).cloned().unwrap_or_default();
                run_state.with_mut(|m| {
                    m.insert(
                        id.clone(),
                        RunStep {
                            status: NodeStatus::Running,
                            secs: None,
                            error: None,
                        },
                    )
                });
                if let Some(centre) = centres.get(id).copied() {
                    follow(centre);
                }
                sleep(STEP_MS).await;
                let result = results.get(id).copied().unwrap_or_default();
                if failure.as_deref() == Some(id.as_str()) {
                    let message = result.error_message.unwrap_or_default().to_string();
                    run_state.with_mut(|m| {
                        m.insert(
                            id.clone(),
                            RunStep {
                                status: NodeStatus::Failed,
                                secs: result.duration,
                                error: Some(message.clone()),
                            },
                        )
                    });
                    announce.set(format!("Failed at {title}: {message}"));
                    outcome = format!("Failed at {title}");
                    break;
                }
                let secs = result.duration.unwrap_or(0.0);
                run_state.with_mut(|m| {
                    m.insert(
                        id.clone(),
                        RunStep {
                            status: NodeStatus::Done,
                            secs: Some(secs),
                            error: None,
                        },
                    )
                });
                run_timeline.with_mut(|t| t.push((id.clone(), secs)));
                announce.set(format!("{title} done in {}", spoken_duration(secs)));
            }
            running.set(false);
            run_log.with_mut(|l| l.push(outcome.clone()));
            if outcome == "Finished" {
                announce.set(format!("Test run finished, {} steps", order.len()));
            }
        });
    };

    let mut root_tab_keys = move |e: KeyboardEvent, index: usize| {
        let next = match e.key() {
            Key::ArrowRight => (index + 1) % TABS.len(),
            Key::ArrowLeft => (index + TABS.len() - 1) % TABS.len(),
            Key::Home => 0,
            Key::End => TABS.len() - 1,
            _ => return,
        };
        e.prevent_default();
        let name = TABS[next].0;
        tab.set(name);
        focus_now(&tab_dom(base, name));
    };

    let sep_keys = move |e: KeyboardEvent| {
        let next = match e.key() {
            Key::ArrowUp => (run_h() + RUN_STEP).clamp(RUN_MIN, RUN_MAX),
            Key::ArrowDown => (run_h() - RUN_STEP).clamp(RUN_MIN, RUN_MAX),
            Key::Home => RUN_MIN,
            Key::End => RUN_MAX,
            _ => return,
        };
        e.prevent_default();
        run_h.set(next);
    };

    let mut link_keys = move |e: KeyboardEvent, edge_id: String, source: String| {
        let siblings: Vec<String> = edges()
            .iter()
            .filter(|x| x.source_node == source)
            .map(|x| x.id.clone())
            .collect();
        let index = siblings.iter().position(|x| *x == edge_id).unwrap_or(0);
        match e.key() {
            Key::ArrowRight | Key::ArrowDown if !siblings.is_empty() => {
                e.prevent_default();
                let next = siblings[(index + 1) % siblings.len()].clone();
                focus_now(&link_dom(base, &next));
            }
            Key::ArrowLeft | Key::ArrowUp if !siblings.is_empty() => {
                e.prevent_default();
                let prev = siblings[(index + siblings.len() - 1) % siblings.len()].clone();
                focus_now(&link_dom(base, &prev));
            }
            Key::Delete | Key::Backspace => {
                e.prevent_default();
                let next: Vec<CanvasEdge> =
                    edges().into_iter().filter(|x| x.id != edge_id).collect();
                let remaining: Vec<String> =
                    siblings.into_iter().filter(|x| *x != edge_id).collect();
                focus_link.set(None);
                announce.set("Link removed".to_string());
                commit(nodes(), next);
                match remaining.first() {
                    Some(id) => focus_after_frame(&link_dom(base, id)),
                    None => focus_after_frame(&node_dom(base, &source)),
                }
            }
            Key::Escape => {
                e.stop_propagation();
                focus_link.set(None);
                focus_after_frame(&node_dom(base, &source));
            }
            _ => {}
        }
    };

    let picker_keys = move |e: KeyboardEvent, index: usize| {
        let last = PALETTE.len() - 1;
        let next = match e.key() {
            Key::ArrowDown => (index + 1).min(last),
            Key::ArrowUp => index.saturating_sub(1),
            Key::Home => 0,
            Key::End => last,
            _ => return,
        };
        e.prevent_default();
        focus_now(&picker_dom(base, next));
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class: "kit-ac",
            "aria-labelledby": "kit-ac-{base}-title",
            onpointermove: root_move,
            onpointerup: root_up,
            onpointercancel: root_cancel,
            onkeydown: root_keys,
            h2 { id: "kit-ac-{base}-title", class: "kit-sr", "Automation canvas, {project_name}" }
            nav { class: "{side_class}", "aria-label": "Sidebar",
                p { class: "kit-ac__ws", "{workspace_name}" }
                ul { class: "kit-ac__navlist",
                    for item in NAV {
                        li { key: "{item}",
                            button {
                                class: "kit-ac__navitem",
                                r#type: "button",
                                "aria-current": if item == "Workflows" { "page" } else { "false" },
                                onclick: move |_| on_navigate.call(item.to_string()),
                                "{item}"
                                if item == "Runs" {
                                    span { class: "kit-ac__count", "{log_count}" }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "kit-ac__main",
                header { class: "kit-ac__crumbs",
                    button {
                        class: "kit-ac__btn kit-ac__sidetoggle",
                        r#type: "button",
                        "aria-expanded": "{side_open()}",
                        onclick: move |_| side_open.toggle(),
                        "Menu"
                    }
                    nav { "aria-label": "Breadcrumb",
                        ol { class: "kit-ac__trail",
                            li { "{workspace_name}" }
                            li { "{project_name}" }
                            li { "aria-current": "page", "{path_branch}" }
                        }
                    }
                    span { class: "kit-ac__chip", "{branch_status}" }
                    button {
                        class: "kit-ac__btn kit-ac__btn--primary",
                        r#type: "button",
                        onclick: move |_| on_publish.call(()),
                        "Publish"
                    }
                }

                div { class: "kit-ac__body",
                    aside { class: "{palette_class}", "aria-label": "Steps",
                        div { class: "kit-ac__palhead",
                            h3 { class: "kit-ac__palttl", "Nodes" }
                            button {
                                class: "kit-ac__btn kit-ac__palclose",
                                r#type: "button",
                                "aria-label": "Close node list",
                                onclick: move |_| palette_open.set(false),
                                "×"
                            }
                        }
                        button {
                            id: "{ui_dom(base, \"addnode\")}",
                            class: "kit-ac__btn kit-ac__addnode",
                            r#type: "button",
                            "aria-expanded": "{palette_open()}",
                            onclick: move |_| palette_open.toggle(),
                            "Add node"
                        }
                        for group in GROUP_NAMES {
                            section { key: "{group}", class: "kit-ac__group", "aria-label": "{group}",
                                h3 { class: "kit-ac__grouptitle", "{group}",
                                    span { class: "kit-ac__groupcount", {PALETTE.iter().filter(|(g, _, _)| *g == group).count().to_string()} }
                                }
                                ul { class: "kit-ac__items",
                                    for (kind, title) in PALETTE.iter().filter(|(g, _, _)| *g == group).map(|(_, k, t)| (*k, *t)) {
                                        {
                                            let glyph = kind.glyph();
                                            let word = kind.word();
                                            rsx! {
                                                li { key: "{title}",
                                                    button {
                                                        class: "kit-ac__item",
                                                        r#type: "button",
                                                        "aria-label": "Add {title}",
                                                        onpointerdown: move |e: PointerEvent| {
                                                            if e.trigger_button() != Some(MouseButton::Primary) {
                                                                return;
                                                            }
                                                            suppress_click.set(false);
                                                            let c = e.client_coordinates();
                                                            drag.set(Some(Drag::Palette {
                                                                kind,
                                                                title,
                                                                start: (c.x, c.y),
                                                                cursor: (c.x, c.y),
                                                                moved: false,
                                                            }));
                                                        },
                                                        onclick: move |_| {
                                                            let mut add = add_step;
                                                            let anchor = selected().and_then(|s| nodes().into_iter().find(|n| n.id == s));
                                                            match anchor {
                                                                Some(a) => {
                                                                    let port = first_free_port(&a, &edges());
                                                                    add(kind, title, (a.pos_x, a.pos_y + NODE_H + 40.0), Some((a.id.clone(), port)));
                                                                }
                                                                None => {
                                                                    let c = nodes().len() as f64 * 16.0 + 40.0;
                                                                    add(kind, title, (c, c), None);
                                                                }
                                                            }
                                                        },
                                                        span { class: "kit-ac__glyph", "aria-hidden": "true", "{glyph}" }
                                                        span { class: "kit-ac__kind", "{word}" }
                                                        span { class: "kit-ac__label", "{title}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "kit-ac__center",
                        div { class: "kit-ac__stage",
                            div {
                                id: "{board_id}",
                                class: "{board_class}",
                                role: "group",
                                tabindex: "{board_tab}",
                                "aria-label": "Flow board",
                                "aria-busy": "{is_loading}",
                                "aria-describedby": "kit-ac-{base}-hint",
                                style: "{board_style}",
                                onmounted: move |e: Event<MountedData>| {
                                    board_mount.set(Some(e.data()));
                                    // Fits on every size change, so a stylesheet that lands after mount is measured too.
                                    let observed = board_id.clone();
                                    spawn(async move {
                                        each_resize(&observed, move || {
                                            with_board(board_mount, move |b| {
                                                board_origin.set((b.x, b.y));
                                                view.set(fit_view(&nodes(), b.w, b.h));
                                            });
                                        })
                                        .await;
                                    });
                                },
                                onkeydown: move |e: KeyboardEvent| board_keys(e),
                                onkeyup: board_keyup,
                                onpointerdown: board_down,
                                onclick: board_click,
                                onwheel: board_wheel,
                                div { class: "kit-ac__world", style: "{world_style}",
                                    if is_loading {
                                        for (i, n) in node_now.iter().enumerate() {
                                            span {
                                                key: "{i}",
                                                class: "kit-ac__ph",
                                                style: "left: {n.pos_x:.0}px; top: {n.pos_y:.0}px; width: {NODE_W:.0}px; height: {NODE_H:.0}px;",
                                            }
                                        }
                                    }
                                    svg { class: "kit-ac__links", "aria-hidden": "true",
                                        for edge in edge_now.iter() {
                                            {
                                                let from = node_now.iter().find(|n| n.id == edge.source_node);
                                                let to = node_now.iter().find(|n| n.id == edge.target_node);
                                                match (from, to) {
                                                    (Some(a), Some(b)) => {
                                                        let d = edge_path(port_point(a, edge.source_port), input_point(b));
                                                        let lit = running_id.as_deref() == Some(edge.target_node.as_str());
                                                        let mut class = String::from("kit-ac__edge");
                                                        if is_loading {
                                                            class.push_str(" is-muted");
                                                        }
                                                        if lit {
                                                            class.push_str(" is-lit is-travel");
                                                        }
                                                        if focus_link().as_deref() == Some(edge.id.as_str()) {
                                                            class.push_str(" is-focused");
                                                        }
                                                        let click_id = edge.id.clone();
                                                        let click_source = edge.source_node.clone();
                                                        rsx! {
                                                            path {
                                                                key: "{edge.id}",
                                                                class: "{class}",
                                                                d: "{d}",
                                                                onclick: move |e: MouseEvent| {
                                                                    e.stop_propagation();
                                                                    selected.set(Some(click_source.clone()));
                                                                    focus_link.set(Some(click_id.clone()));
                                                                    config_open.set(true);
                                                                },
                                                            }
                                                        }
                                                    }
                                                    _ => rsx! {},
                                                }
                                            }
                                        }
                                        if let Some(Drag::Link { from_at, cursor, .. }) = drag_now.clone() {
                                            path {
                                                class: "kit-ac__edge is-drag",
                                                d: "{edge_path(from_at, cursor)}",
                                            }
                                        }
                                    }
                                    if !is_loading {
                                        for node in node_now.iter() {
                                            {
                                                let run = run_now.get(&node.id).cloned();
                                                let status = run.as_ref().map(|r| r.status).unwrap_or(node.node_status);
                                                let secs = run.as_ref().and_then(|r| r.secs);
                                                let error = run.as_ref().and_then(|r| r.error.clone());
                                                let line = status_line(status, secs);
                                                let is_sel = sel.as_deref() == Some(node.id.as_str());
                                                let tab_stop = if is_sel { "0" } else { "-1" };
                                                let running_flag = if status == NodeStatus::Running { "true" } else { "false" };
                                                let drop_flag = if linking && drag_now.as_ref().map(|d| matches!(d, Drag::Link { from, .. } if *from != node.id)).unwrap_or(false) { "true" } else { "false" };
                                                let style = format!(
                                                    "left: {:.0}px; top: {:.0}px; width: {NODE_W:.0}px; height: {:.0}px;",
                                                    node.pos_x, node.pos_y, node_height(node.node_type)
                                                );
                                                let label = format!("{}, {}, {}", node.node_title, node.node_type.word(), status.word());
                                                let kind_word = node.node_type.word();
                                                let glyph = node.node_type.glyph();
                                                let chip = match node.node_type {
                                                    NodeKind::Trigger | NodeKind::Action => line.clone(),
                                                    _ => node.chip.clone().unwrap_or_default(),
                                                };
                                                let node_id = node.id.clone();
                                                let node_id_up = node.id.clone();
                                                let node_id_click = node.id.clone();
                                                let node_at = node.clone();
                                                rsx! {
                                                    button {
                                                        key: "{node.id}",
                                                        id: "{node_dom(base, &node.id)}",
                                                        class: "kit-ac__node",
                                                        r#type: "button",
                                                        "aria-pressed": "{is_sel}",
                                                        "aria-label": "{label}",
                                                        tabindex: "{tab_stop}",
                                                        style: "{style}",
                                                        "data-status": "{status.word()}",
                                                        "data-running": "{running_flag}",
                                                        "data-drop": "{drop_flag}",
                                                        onpointerdown: move |e: PointerEvent| node_down(e, node_id.clone()),
                                                        onpointerup: move |e: PointerEvent| node_up(e, node_id_up.clone()),
                                                        onclick: move |e: MouseEvent| {
                                                            e.stop_propagation();
                                                            if suppress_click() {
                                                                suppress_click.set(false);
                                                                return;
                                                            }
                                                            select_node(node_id_click.clone());
                                                        },
                                                        span { class: "kit-ac__kindtag", "aria-hidden": "true", "{kind_word}" }
                                                        span { class: "kit-ac__nodehead",
                                                            span { class: "kit-ac__glyph", "aria-hidden": "true", "{glyph}" }
                                                            span { class: "kit-ac__nodetitle", "{node.node_title}" }
                                                            span { class: "kit-ac__nodechip", "{chip}" }
                                                        }
                                                        if let Some(sub) = node.subtitle.clone() {
                                                            span { class: "kit-ac__nodesub", "{sub}" }
                                                        }
                                                        if let Some(message) = error {
                                                            span { class: "kit-ac__nodeerr", "{message}" }
                                                        }
                                                        if node_at.node_type != NodeKind::Trigger {
                                                            span { class: "kit-ac__inport", "aria-hidden": "true" }
                                                        }
                                                        for port in ports_of(node_at.node_type).iter().copied() {
                                                            {
                                                                let at = port_point(&node_at, port);
                                                                let port_id = node.id.clone();
                                                                let word = port.word();
                                                                let class = format!("kit-ac__port kit-ac__port--{word}");
                                                                let left = at.0 - node_at.pos_x;
                                                                let style = format!("left: {left:.0}px;");
                                                                rsx! {
                                                                    span {
                                                                        key: "{word}",
                                                                        class: "{class}",
                                                                        style: "{style}",
                                                                        "data-port": "{word}",
                                                                        "aria-hidden": "true",
                                                                        onpointerdown: move |e: PointerEvent| port_down(e, port_id.clone(), port, at),
                                                                        if node_at.node_type == NodeKind::Logic { "{word}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "kit-ac__overlay",
                                if is_empty && !is_loading {
                                    div { class: "kit-ac__empty",
                                        button {
                                            class: "kit-ac__btn kit-ac__btn--primary",
                                            r#type: "button",
                                            onclick: move |_| {
                                                let mut add = add_step;
                                                add(NodeKind::Trigger, "Webhook trigger", (40.0, 40.0), None);
                                            },
                                            "Add a trigger"
                                        }
                                    }
                                }
                                if !is_loading {
                                    for edge in edge_now.iter() {
                                        {
                                            let mid_point = link_midpoint(&node_now, edge);
                                            match mid_point {
                                                Some((mx, my)) => {
                                                    let sx = view_now.pan_x + mx * view_now.zoom;
                                                    let sy = view_now.pan_y + my * view_now.zoom;
                                                    let style = format!("left: {sx:.0}px; top: {sy:.0}px;");
                                                    let src_title = node_now.iter().find(|n| n.id == edge.source_node).map(|n| n.node_title.clone()).unwrap_or_default();
                                                    let dst_title = node_now.iter().find(|n| n.id == edge.target_node).map(|n| n.node_title.clone()).unwrap_or_default();
                                                    let label = format!("Add a step on the link from {src_title} to {dst_title}");
                                                    let edge_id = edge.id.clone();
                                                    rsx! {
                                                        button {
                                                            key: "{edge.id}",
                                                            id: "{link_add_dom(base, &edge.id)}",
                                                            class: "kit-ac__linkadd",
                                                            r#type: "button",
                                                            style: "{style}",
                                                            "aria-label": "{label}",
                                                            title: "Add a step on this link",
                                                            onclick: move |_| {
                                                                picker.set(Some(Anchor::Link { edge: edge_id.clone() }));
                                                                focus_after_frame(&picker_dom(base, 0));
                                                            },
                                                            "+"
                                                        }
                                                    }
                                                }
                                                None => rsx! {},
                                            }
                                        }
                                    }
                                }
                                if !is_loading {
                                    for (from, port, sx, sy) in unlinked_outputs(&node_now, &edge_now, view_now) {
                                        button {
                                            key: "{from}-{port.word()}",
                                            class: "kit-ac__portadd",
                                            r#type: "button",
                                            style: "left: {sx:.0}px; top: {sy:.0}px;",
                                            "aria-label": "Add a step after this output",
                                            title: "Add a step",
                                            onclick: {
                                                let from = from.clone();
                                                move |_| {
                                                    picker.set(Some(Anchor::Port { from: from.clone(), port }));
                                                    focus_after_frame(&picker_dom(base, 0));
                                                }
                                            },
                                            "+"
                                        }
                                    }
                                }
                                for pin in comment_now.iter() {
                                    {
                                        let sx = view_now.pan_x + pin.pos_x * view_now.zoom;
                                        let sy = view_now.pan_y + pin.pos_y * view_now.zoom;
                                        let author = pin.author_name.clone().unwrap_or_else(|| "Unknown author".to_string());
                                        let text = pin.comment_text.clone();
                                        let resolved = pin.resolved;
                                        let open = open_comment().as_deref() == Some(pin.id.as_str());
                                        let pin_id = pin.id.clone();
                                        let pin_id_toggle = pin.id.clone();
                                        let pin_id_resolve = pin.id.clone();
                                        let state = if resolved { "Resolved comment" } else { "Comment" };
                                        let label = format!("{state} by {author}: {text}");
                                        let resolve_label = if resolved { "Reopen comment" } else { "Resolve comment" };
                                        let resolve_text = if resolved { "Reopen" } else { "Resolve" };
                                        let class = if resolved { "kit-ac__pin is-resolved" } else { "kit-ac__pin" };
                                        let style = format!("left: {sx:.0}px; top: {sy:.0}px;");
                                        rsx! {
                                            button {
                                                key: "{pin.id}",
                                                id: "{pin_dom(base, &pin.id)}",
                                                class: "{class}",
                                                r#type: "button",
                                                style: "{style}",
                                                "aria-label": "{label}",
                                                "aria-expanded": "{open}",
                                                onpointerdown: move |e: PointerEvent| e.stop_propagation(),
                                                onclick: move |_| {
                                                    if open_comment().as_deref() == Some(pin_id_toggle.as_str()) {
                                                        open_comment.set(None);
                                                    } else {
                                                        open_comment.set(Some(pin_id.clone()));
                                                    }
                                                },
                                                span { class: "kit-sr", "{label}" }
                                                if open {
                                                    span { class: "kit-ac__pinpop",
                                                        span { class: "kit-ac__pinauthor", "{author}" }
                                                        span { "{text}" }
                                                        button {
                                                            class: "kit-ac__btn",
                                                            r#type: "button",
                                                            "aria-label": "{resolve_label}",
                                                            onclick: move |e: MouseEvent| {
                                                                e.stop_propagation();
                                                                let mut toggle = toggle_resolved;
                                                                toggle(pin_id_resolve.clone());
                                                            },
                                                            "{resolve_text}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if let Some((x, y)) = draft_now {
                                    {
                                        let sx = view_now.pan_x + x * view_now.zoom;
                                        let sy = view_now.pan_y + y * view_now.zoom;
                                        let style = format!("left: {sx:.0}px; top: {sy:.0}px;");
                                        rsx! {
                                            div { class: "kit-ac__draft", style: "{style}",
                                                input {
                                                    id: "kit-ac-{base}-cinput",
                                                    class: "kit-ac__cinput",
                                                    r#type: "text",
                                                    "aria-label": "Comment text",
                                                    placeholder: "Write a comment",
                                                    value: "{draft_value}",
                                                    oninput: move |e: FormEvent| draft_text.set(e.value()),
                                                    onkeydown: move |e: KeyboardEvent| {
                                                        if matches!(e.key(), Key::Enter) {
                                                            e.prevent_default();
                                                            let mut post = post_draft;
                                                            post(());
                                                        }
                                                    },
                                                }
                                                span { class: "kit-ac__hint", "Enter posts. Escape discards." }
                                            }
                                        }
                                    }
                                }
                                if let (Some(anchor), Some(at)) = (picker_now.clone(), anchor_point) {
                                    {
                                        let sx = view_now.pan_x + at.0 * view_now.zoom;
                                        let sy = view_now.pan_y + at.1 * view_now.zoom + 10.0;
                                        let style = format!("left: {sx:.0}px; top: {sy:.0}px;");
                                        let anchor_port = match &anchor {
                                            Anchor::Port { port, .. } => port.word(),
                                            Anchor::Link { .. } => "link",
                                        };
                                        let menu_label = match &anchor {
                                            Anchor::Port { from, .. } => {
                                                let title = node_now.iter().find(|n| n.id == *from).map(|n| n.node_title.clone()).unwrap_or_default();
                                                format!("Add a step after {title}")
                                            }
                                            Anchor::Link { .. } => "Add a step on this link".to_string(),
                                        };
                                        rsx! {
                                            div {
                                                class: "kit-ac__picker",
                                                role: "menu",
                                                "aria-label": "{menu_label}",
                                                "data-port": "{anchor_port}",
                                                style: "{style}",
                                                for (index, (_, kind, title)) in PALETTE.iter().copied().enumerate() {
                                                    {
                                                        let glyph = kind.glyph();
                                                        let word = kind.word();
                                                        rsx! {
                                                            button {
                                                                key: "{title}",
                                                                id: "{picker_dom(base, index)}",
                                                                class: "kit-ac__item",
                                                                r#type: "button",
                                                                role: "menuitem",
                                                                onkeydown: move |e: KeyboardEvent| picker_keys(e, index),
                                                                onclick: move |_| {
                                                                    let mut add = add_from_picker;
                                                                    add(kind, title);
                                                                },
                                                                span { class: "kit-ac__glyph", "aria-hidden": "true", "{glyph}" }
                                                                span { class: "kit-ac__kind", "{word}" }
                                                                span { class: "kit-ac__label", "{title}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "kit-ac__strip", role: "toolbar", "aria-label": "Board controls",
                                    button {
                                        class: "kit-ac__btn kit-ac__tool",
                                        r#type: "button",
                                        title: "Add node (A)",
                                        "aria-label": "Add node",
                                        onclick: move |_| palette_open.toggle(),
                                        "⊞"
                                    }
                                    button {
                                        class: "kit-ac__btn kit-ac__tool",
                                        r#type: "button",
                                        "aria-pressed": "{!comment_tool()}",
                                        title: "Select (Escape)",
                                        "aria-label": "Select",
                                        onclick: move |_| comment_tool.set(false),
                                        "↖"
                                    }
                                    button {
                                        id: "{ui_dom(base, \"comment\")}",
                                        class: "kit-ac__btn kit-ac__tool",
                                        r#type: "button",
                                        "aria-pressed": "{comment_tool()}",
                                        title: "Comment (C)",
                                        "aria-label": "Comment",
                                        onclick: move |_| comment_tool.set(true),
                                        "✎"
                                    }
                                    button {
                                        class: "kit-ac__btn kit-ac__tool",
                                        r#type: "button",
                                        title: "Fit to view (0)",
                                        "aria-label": "Fit to view",
                                        onclick: move |_| fit(),
                                        "⤢"
                                    }
                                    button {
                                        class: "kit-ac__btn kit-ac__tool",
                                        r#type: "button",
                                        title: "Zoom out (-)",
                                        "aria-label": "Zoom out",
                                        onclick: move |_| zoom_by(1.0 / ZOOM_FACTOR),
                                        "−"
                                    }
                                    span { class: "kit-ac__zoom", "{zoom_pct}" }
                                    button {
                                        class: "kit-ac__btn kit-ac__tool",
                                        r#type: "button",
                                        title: "Zoom in (+)",
                                        "aria-label": "Zoom in",
                                        onclick: move |_| zoom_by(ZOOM_FACTOR),
                                        "+"
                                    }
                                    button {
                                        id: "{ui_dom(base, \"run\")}",
                                        class: "kit-ac__btn kit-ac__btn--primary kit-ac__tool",
                                        r#type: "button",
                                        "aria-pressed": "{run_open()}",
                                        title: "Run area",
                                        "aria-label": "Run area",
                                        onclick: move |_| run_open.toggle(),
                                        "▶"
                                    }
                                }
                            }
                        }
                        p { id: "kit-ac-{base}-hint", class: "kit-sr",
                            "Arrow keys move between steps. Alt with an arrow nudges the step. E opens its links. A adds a step."
                        }

                        if run_open() {
                            div {
                                class: "kit-ac__sep",
                                role: "separator",
                                tabindex: "0",
                                "aria-orientation": "horizontal",
                                "aria-label": "Resize run area",
                                "aria-valuemin": "{RUN_MIN:.0}",
                                "aria-valuemax": "{RUN_MAX:.0}",
                                "aria-valuenow": "{run_h():.0}",
                                onpointerdown: move |e: PointerEvent| {
                                    if e.trigger_button() != Some(MouseButton::Primary) {
                                        return;
                                    }
                                    e.prevent_default();
                                    let c = e.client_coordinates();
                                    suppress_click.set(false);
                                    drag.set(Some(Drag::Run { start: c.y, origin: run_h() }));
                                },
                                onkeydown: sep_keys,
                            }
                        }
                        if run_open() {
                            section { class: "{run_class}", style: "{run_style}", "aria-label": "Run area",
                                div { class: "kit-ac__tabs", role: "tablist", "aria-label": "Run details",
                                    for (index, (name, label)) in TABS.iter().copied().enumerate() {
                                        button {
                                            key: "{name}",
                                            id: "{tab_dom(base, name)}",
                                            class: "kit-ac__tab",
                                            r#type: "button",
                                            role: "tab",
                                            "aria-selected": if tab_now == name { "true" } else { "false" },
                                            tabindex: if tab_now == name { "0" } else { "-1" },
                                            onclick: move |_| tab.set(name),
                                            onkeydown: move |e: KeyboardEvent| root_tab_keys(e, index),
                                            "{label}"
                                            if name == "errors" && error_count > 0 {
                                                span { class: "kit-ac__count", "{error_count}" }
                                            }
                                        }
                                    }
                                    button {
                                        class: "kit-ac__btn kit-ac__start",
                                        r#type: "button",
                                        disabled: loading_disabled,
                                        onclick: start_run,
                                        if running() { "Running…" } else { "Start test run" }
                                    }
                                    button {
                                        class: "kit-ac__btn kit-ac__runclose",
                                        r#type: "button",
                                        "aria-label": "Close run area",
                                        onclick: move |_| {
                                            run_open.set(false);
                                            focus_after_frame(&ui_dom(base, "run"));
                                        },
                                        "Close"
                                    }
                                }
                                div { class: "kit-ac__panel", role: "tabpanel",
                                    if tab_now == "payload" {
                                        pre { class: "kit-ac__payload", "{payload}" }
                                    } else if tab_now == "triggers" {
                                        ul { class: "kit-ac__timeline",
                                            for node in node_now.iter().filter(|n| n.node_type == NodeKind::Trigger) {
                                                li { key: "{node.id}", "{node.node_title} · {node.node_status.word()}" }
                                            }
                                            if !node_now.iter().any(|n| n.node_type == NodeKind::Trigger) {
                                                li { class: "kit-ac__muted", "No trigger on this flow" }
                                            }
                                        }
                                    } else if tab_now == "errors" {
                                        if error_list.is_empty() {
                                            p { class: "kit-ac__muted", "No errors" }
                                        } else {
                                            ul { class: "kit-ac__timeline",
                                                for (index, item) in error_list.iter().enumerate() {
                                                    li { key: "{index}", "{item}" }
                                                }
                                            }
                                        }
                                    } else if tab_now == "runs" {
                                        ol { class: "kit-ac__timeline",
                                            for (index, outcome) in run_log().iter().enumerate() {
                                                li { key: "{index}", "Run {index + 1} · {outcome}" }
                                            }
                                            if log_count == 0 {
                                                li { class: "kit-ac__muted", "No runs in this session" }
                                            }
                                        }
                                    } else {
                                        ol { class: "kit-ac__timeline",
                                            for (id, secs) in timeline.iter() {
                                                {
                                                    let title = node_now.iter().find(|n| n.id == *id).map(|n| n.node_title.clone()).unwrap_or_else(|| id.clone());
                                                    let spoken = spoken_duration(*secs);
                                                    rsx! { li { key: "{id}", "{title} · {spoken}" } }
                                                }
                                            }
                                            if timeline.is_empty() {
                                                li { class: "kit-ac__muted", "No runs yet" }
                                                li { class: "kit-ac__muted", "Start a test run to time each step." }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    aside {
                        id: "kit-ac-{base}-panel",
                        class: "{config_class}",
                        tabindex: "-1",
                        "aria-label": "Step settings",
                        if let Some(node) = selected_node.clone() {
                            div { class: "kit-ac__cfghead",
                                h2 { class: "kit-ac__cfgtitle", "{node.node_title}" }
                                button {
                                    id: "{ui_dom(base, \"config\")}",
                                    class: "kit-ac__btn",
                                    r#type: "button",
                                    "aria-label": "Close step settings",
                                    onclick: {
                                        let close_id = node.id.clone();
                                        move |_| {
                                            config_open.set(false);
                                            focus_after_frame(&node_dom(base, &close_id));
                                        }
                                    },
                                    "Close"
                                }
                            }
                            p { class: "kit-ac__muted", "{node.node_type.word()} · {node.node_status.word()}" }
                            h3 { class: "kit-ac__cfgsub", "Outgoing links" }
                            ul { class: "kit-ac__links-list",
                                for edge in outgoing.iter() {
                                    {
                                        let target_title = node_now.iter().find(|n| n.id == edge.target_node).map(|n| n.node_title.clone()).unwrap_or_else(|| edge.target_node.clone());
                                        let focused = if focus_link().as_deref() == Some(edge.id.as_str()) { "true" } else { "false" };
                                        let word = edge.source_port.word();
                                        let row_label = format!("{} {word} to {target_title}", node.node_title);
                                        let shown = format!("{word} → {target_title}");
                                        let remove_label = format!("Remove link from {} to {target_title}", node.node_title);
                                        let key_id = edge.id.clone();
                                        let key_source = node.id.clone();
                                        let focus_id = edge.id.clone();
                                        let remove_id = edge.id.clone();
                                        rsx! {
                                            li {
                                                key: "{edge.id}",
                                                id: "{link_dom(base, &edge.id)}",
                                                class: "kit-ac__linkrow",
                                                tabindex: "0",
                                                "aria-label": "{row_label}",
                                                "data-focused": "{focused}",
                                                onfocus: move |_| focus_link.set(Some(focus_id.clone())),
                                                onkeydown: move |e: KeyboardEvent| link_keys(e, key_id.clone(), key_source.clone()),
                                                span { "{shown}" }
                                                button {
                                                    class: "kit-ac__btn",
                                                    r#type: "button",
                                                    "aria-label": "{remove_label}",
                                                    onclick: move |_| {
                                                        let next: Vec<CanvasEdge> = edges().into_iter().filter(|e| e.id != remove_id).collect();
                                                        announce.set("Link removed".to_string());
                                                        commit(nodes(), next);
                                                    },
                                                    "Remove"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            for port in ports_of(node.node_type).iter().copied() {
                                {
                                    let caption = port_caption(port);
                                    let source = node.id.clone();
                                    let source_title = node.node_title.clone();
                                    let select_label = format!("{caption} {source_title}");
                                    let select_key = format!("{}-{}", node.id, port.word());
                                    rsx! {
                                        label { key: "{select_key}", class: "kit-ac__connect",
                                            span { "{caption}" }
                                            select {
                                                class: "kit-ac__select",
                                                "aria-label": "{select_label}",
                                                onchange: move |e: FormEvent| {
                                                    let target = e.value();
                                                    if target.is_empty() {
                                                        return;
                                                    }
                                                    if let Some(next) = connect(&edges(), &source, port, &target) {
                                                        announce.set(format!("Connected {} to {target}", port.word()));
                                                        commit(nodes(), next);
                                                    }
                                                },
                                                option { value: "", "Choose a step" }
                                                for other in node_now.iter().filter(|n| n.id != node.id) {
                                                    option { key: "{other.id}", value: "{other.id}", "{other.node_title}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            p { class: "kit-ac__muted", "Select a step to configure it." }
                        }
                    }
                }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
