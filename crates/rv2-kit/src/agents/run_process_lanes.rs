//! One agent run as a process diagram: a lane per person, agent or tool, a box per
//! step in its owner's lane, a connector per hand-over, and a diamond per approval.
//! A waiting approval can be approved or rejected from its card.
//!
//! Design choice: one column per step, in dependency order (Kahn's sort, ties
//! broken by array order), so every connector runs left to right.
//! Design choice: columns share the container's width after the lane labels; when
//! they would fall below `MIN_COLUMN_PX` the whole diagram scales as one unit, so no
//! horizontal scrolling is offered.
//! Design choice: connectors and the connector light are SVG in the stage's px space.
//! Design choice: a lane label opens the card of the lane's current step
//! (running, else waiting, else last); a lane with no steps has no card.
//! Browser checks: boxes settle and connectors draw column by column; the light
//! runs into a waiting approval; running boxes shimmer; Approve and Reject return
//! focus to the box; Tab visits lane labels then boxes column by column; reduced
//! motion removes all of it.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id, use_element_width};

const CSS: Asset = asset!("/assets/run-process-lanes.css");

/// Container width below which lane labels show only their marks.
pub const NARROW_PX: f64 = 480.0;
const ROW_PX: f64 = 64.0;
/// Column width when the container is not measured yet (SSR, tests).
const UNMEASURED_COLUMN_PX: f64 = 200.0;
/// Narrowest column before the whole diagram scales down as one unit.
const MIN_COLUMN_PX: f64 = 96.0;
/// Empty stage room below the lanes while a card is open, so the card stays in the stage.
const CARD_ROOM_PX: f64 = 260.0;
const CARD_WIDTH_PX: f64 = 288.0;
const LABEL_WIDE_PX: f64 = 136.0;
const LABEL_NARROW_PX: f64 = 44.0;
/// Widest gap between columns; narrower columns tighten the gap before boxes shrink.
const GAP_MAX_PX: f64 = 28.0;
const GAP_RATIO: f64 = 0.2;
/// Height inside a lane where connectors meet boxes and shapes.
const PORT_Y_PX: f64 = 38.0;
const SHAPE_PX: f64 = 24.0;
const BOX_HEIGHT_PX: f64 = 52.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LaneKind {
    Person,
    #[default]
    Agent,
    Tool,
}

impl LaneKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Agent => "agent",
            Self::Tool => "tool",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lane {
    pub id: String,
    pub name: String,
    pub kind: LaneKind,
    pub role: Option<String>,
    pub mark: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepStatus {
    #[default]
    Queued,
    Running,
    Waiting,
    Done,
    Failed,
    Skipped,
}

impl StepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Waiting => "Needs you",
            Self::Done => "Done",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StepShape {
    #[default]
    Step,
    Approval,
    Start,
    Finish,
}

impl StepShape {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Step => "step",
            Self::Approval => "approval",
            Self::Start => "start",
            Self::Finish => "finish",
        }
    }

    fn is_box(self) -> bool {
        matches!(self, Self::Step)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub id: String,
    pub lane_id: String,
    pub title: String,
    pub status: StepStatus,
    pub shape: StepShape,
    /// Ids of the steps that hand over to this one. `None` means the step
    /// before it in the array; `Some(vec![])` starts a new branch.
    pub after: Option<Vec<String>>,
    pub duration_label: Option<String>,
    pub detail: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Waiting,
    Done,
    Failed,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Waiting => "Waiting",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    pub step: Step,
    pub lane: usize,
    pub column: usize,
}

/// Normalised diagram: steps in array order, each with its lane and column,
/// and connectors as (from, to) indexes into `steps`.
#[derive(Clone, Debug, PartialEq)]
pub struct Diagram {
    pub lanes: Vec<Lane>,
    pub steps: Vec<Placed>,
    pub edges: Vec<(usize, usize)>,
}

fn reaches(successors: &[Vec<usize>], from: usize, to: usize) -> bool {
    let mut stack = vec![from];
    let mut seen = vec![false; successors.len()];
    while let Some(node) = stack.pop() {
        if node == to {
            return true;
        }
        if std::mem::replace(&mut seen[node], true) {
            continue;
        }
        stack.extend(successors[node].iter().copied());
    }
    false
}

/// Applies the spec's normalisation: repeated ids keep the first, steps on an
/// unknown lane are dropped with their connectors, a connector from an unknown id
/// is dropped, and a connector that would close a cycle is cut.
pub fn normalise(lanes: &[Lane], steps: &[Step]) -> Diagram {
    let mut unique: Vec<&Step> = Vec::new();
    for step in steps {
        if !unique.iter().any(|s| s.id == step.id) {
            unique.push(step);
        }
    }
    let lane_of = |lane_id: &str| lanes.iter().position(|l| l.id == lane_id);
    let kept: Vec<bool> = unique
        .iter()
        .map(|s| lane_of(&s.lane_id).is_some())
        .collect();

    let mut raw: Vec<(usize, usize)> = Vec::new();
    for (index, step) in unique.iter().enumerate() {
        let preds: Vec<&str> = match &step.after {
            Some(list) => list.iter().map(String::as_str).collect(),
            None if index > 0 => vec![unique[index - 1].id.as_str()],
            None => Vec::new(),
        };
        for pred in preds {
            let Some(from) = unique.iter().position(|s| s.id == pred) else {
                continue;
            };
            if from != index && kept[from] && kept[index] && !raw.contains(&(from, index)) {
                raw.push((from, index));
            }
        }
    }

    let mut successors: Vec<Vec<usize>> = vec![Vec::new(); unique.len()];
    let mut accepted: Vec<(usize, usize)> = Vec::new();
    for (from, to) in raw {
        if reaches(&successors, to, from) {
            continue;
        }
        successors[from].push(to);
        accepted.push((from, to));
    }

    let mut placed = vec![false; unique.len()];
    let mut order: Vec<usize> = Vec::new();
    loop {
        let next = (0..unique.len()).find(|&i| {
            kept[i] && !placed[i] && accepted.iter().all(|&(from, to)| to != i || placed[from])
        });
        let Some(next) = next else { break };
        placed[next] = true;
        order.push(next);
    }

    let mut column_of = vec![0usize; unique.len()];
    for (column, &index) in order.iter().enumerate() {
        column_of[index] = column;
    }
    let mut slot_of = vec![usize::MAX; unique.len()];
    let mut out_steps = Vec::new();
    for (index, step) in unique.iter().enumerate() {
        if !placed[index] {
            continue;
        }
        slot_of[index] = out_steps.len();
        out_steps.push(Placed {
            step: (*step).clone(),
            lane: lane_of(&step.lane_id).unwrap_or(0),
            column: column_of[index],
        });
    }
    let edges = accepted
        .into_iter()
        .filter(|&(from, to)| placed[from] && placed[to])
        .map(|(from, to)| (slot_of[from], slot_of[to]))
        .collect();

    Diagram {
        lanes: lanes.to_vec(),
        steps: out_steps,
        edges,
    }
}

/// Run status from the steps: failed when a step failed and none runs; waiting
/// when an approval waits; done when every step is done or skipped; else running.
pub fn run_status(steps: &[Placed]) -> RunStatus {
    let any_running = steps.iter().any(|s| s.step.status == StepStatus::Running);
    let any_failed = steps.iter().any(|s| s.step.status == StepStatus::Failed);
    let approval_waiting = steps
        .iter()
        .any(|s| s.step.shape == StepShape::Approval && s.step.status == StepStatus::Waiting);
    let all_settled = !steps.is_empty()
        && steps
            .iter()
            .all(|s| matches!(s.step.status, StepStatus::Done | StepStatus::Skipped));
    if any_failed && !any_running {
        RunStatus::Failed
    } else if approval_waiting {
        RunStatus::Waiting
    } else if all_settled {
        RunStatus::Done
    } else {
        RunStatus::Running
    }
}

/// Boxes and approval diamonds: the steps the header's counter tallies.
pub fn is_counted(step: &Step) -> bool {
    matches!(step.shape, StepShape::Step | StepShape::Approval)
}

/// Counted steps done, for the header's `done of total`.
pub fn done_count(steps: &[Placed]) -> usize {
    steps
        .iter()
        .filter(|s| is_counted(&s.step) && s.step.status == StepStatus::Done)
        .count()
}

/// Counted steps, for the header's `done of total`.
pub fn counted_total(steps: &[Placed]) -> usize {
    steps.iter().filter(|s| is_counted(&s.step)).count()
}

/// Geometry in stage px. `label` is the lane-label column beside the stage;
/// `stage_width` the columns' width before `scale`, which shrinks the whole diagram
/// as one unit when the columns would fall below `MIN_COLUMN_PX`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub label: f64,
    pub column: f64,
    pub gap: f64,
    pub stage_width: f64,
    pub scale: f64,
}

pub fn frame_for(measured: Option<f64>, columns: usize) -> Frame {
    let measured = measured.filter(|w| *w > 0.0);
    let columns = columns.max(1) as f64;
    let label = if measured.is_some_and(|w| w < NARROW_PX) {
        LABEL_NARROW_PX
    } else {
        LABEL_WIDE_PX
    };
    let column = match measured {
        Some(width) => ((width - label) / columns).max(MIN_COLUMN_PX),
        None => UNMEASURED_COLUMN_PX,
    };
    let stage_width = columns * column;
    let scale = match measured {
        Some(width) => (width / (label + stage_width)).min(1.0),
        None => 1.0,
    };
    Frame {
        label,
        column,
        gap: (column * GAP_RATIO).min(GAP_MAX_PX),
        stage_width,
        scale,
    }
}

impl Frame {
    fn centre_x(&self, column: usize) -> f64 {
        (column as f64 + 0.5) * self.column
    }

    fn cell_left(&self, column: usize) -> f64 {
        column as f64 * self.column + self.gap / 2.0
    }

    fn cell_width(&self) -> f64 {
        self.column - self.gap
    }

    /// X of the gap just before `column`. A connector entering that column turns here, so its vertical
    /// run stays between cells and clear of the labels inside them.
    pub fn channel(&self, column: usize) -> f64 {
        column as f64 * self.column
    }

    /// Left and right connector ports of a step's cell.
    pub fn ports(&self, column: usize, shape: StepShape) -> (f64, f64) {
        if shape.is_box() {
            let left = self.cell_left(column);
            (left, left + self.cell_width())
        } else {
            let centre = self.centre_x(column);
            (centre - SHAPE_PX / 2.0, centre + SHAPE_PX / 2.0)
        }
    }
}

pub fn port_y(lane: usize) -> f64 {
    lane as f64 * ROW_PX + PORT_Y_PX
}

/// Elbow connector from `(x1, y1)` to `(x2, y2)` with its vertical run at `channel`, and the arrow head at its end.
pub fn connector_paths(x1: f64, y1: f64, x2: f64, y2: f64, channel: f64) -> (String, String) {
    let body = format!("M {x1} {y1} L {channel} {y1} L {channel} {y2} L {x2} {y2}");
    let head = format!(
        "M {} {} L {x2} {y2} L {} {}",
        x2 - 7.0,
        y2 - 4.0,
        x2 - 7.0,
        y2 + 4.0
    );
    (body, head)
}

/// Lane count and stage height for the grid.
pub fn stage_height(lanes: usize) -> f64 {
    lanes.max(1) as f64 * ROW_PX
}

fn frame_dom_id(id: &str) -> String {
    format!("{id}-frame")
}

fn outside_script(root: &str) -> String {
    format!(
        r#"const key = '__kitRpl{root}';
window[key]?.();
const onDown = (event) => {{
    if (event.target.closest?.('[data-rpl-card="{root}"], [data-rpl-node="{root}"]')) return;
    cleanup();
    dioxus.send(true);
}};
const cleanup = () => {{ document.removeEventListener('pointerdown', onDown, true); delete window[key]; }};
window[key] = () => {{ cleanup(); dioxus.send(false); }};
document.addEventListener('pointerdown', onDown, true);
await new Promise(() => {{}});"#
    )
}

fn cleanup_script(root: &str) -> String {
    format!("window['__kitRpl{root}']?.();")
}

/// Lane colours as (tint, ink): the lane's own colour when given, else a hue
/// from its id composed with the kit's tint tokens, so both themes follow.
fn lane_colours(lane: &Lane) -> (String, String) {
    match &lane.color {
        Some(colour) => (colour.clone(), colour.clone()),
        None => {
            let hue = lane.id.bytes().fold(0u32, |hash, b| {
                hash.wrapping_mul(31).wrapping_add(u32::from(b))
            }) % 360;
            (
                format!("oklch(var(--kit-tint-strong-l) var(--kit-tint-strong-c) {hue})"),
                format!("oklch(var(--kit-tint-ink-l) var(--kit-tint-ink-c) {hue})"),
            )
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Open {
    step: String,
    from_lane: bool,
}

#[derive(Clone)]
struct Hooks {
    open: Signal<Option<Open>>,
    opener: Signal<String>,
    keyboard: Signal<bool>,
    focus_card: Signal<bool>,
    on_select: Option<EventHandler<Option<Step>>>,
    on_approve: Option<EventHandler<Step>>,
    on_reject: Option<EventHandler<Step>>,
}

impl Hooks {
    fn activate(&self, step: &Step, dom_id: String, from_lane: bool) {
        let mut open = self.open;
        let previous = open.peek().clone();
        let next = match &previous {
            Some(current) if current.step == step.id => None,
            _ => Some(Open {
                step: step.id.clone(),
                from_lane,
            }),
        };
        let mut keyboard = self.keyboard;
        let from_keys = keyboard();
        keyboard.set(false);
        let mut focus_card = self.focus_card;
        focus_card.set(from_keys && next.is_some());
        let mut opener = self.opener;
        opener.set(dom_id);
        if previous != next {
            if let Some(h) = &self.on_select {
                h.call(next.as_ref().map(|_| step.clone()));
            }
        }
        open.set(next);
    }

    fn close(&self) {
        let mut open = self.open;
        let previous = open.peek().clone();
        open.set(None);
        if previous.is_some() {
            if let Some(h) = &self.on_select {
                h.call(None);
            }
        }
    }

    fn decide(&self, step: &Step, approve: bool) {
        let handler = if approve {
            &self.on_approve
        } else {
            &self.on_reject
        };
        if let Some(h) = handler {
            h.call(step.clone());
        }
        let id = self.opener.peek().clone();
        self.close();
        focus_after_frame(&id);
    }

    fn keyed(&self, event: &KeyboardEvent) {
        if event.key() == Key::Enter || event.key() == Key::Character(" ".to_string()) {
            let mut keyboard = self.keyboard;
            keyboard.set(true);
        }
    }
}

struct LaneView {
    dom_id: String,
    name: String,
    role: String,
    mark: String,
    kind: &'static str,
    tint: String,
    ink: String,
    top: f64,
    expanded: bool,
    target: Option<Step>,
    running: bool,
    has_role: bool,
    aria: String,
}

struct StepView {
    dom_id: String,
    step: Step,
    shape: &'static str,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    tint: String,
    ink: String,
    column: usize,
    expanded: bool,
    aria: String,
    status: &'static str,
    skipped: bool,
}

struct ConnectorView {
    key: String,
    body: String,
    head: String,
    column: usize,
    live: bool,
    stroke: String,
    waiting: bool,
}

struct PhoneHandover {
    lane: String,
    live: bool,
}

struct PhoneItem {
    dom_id: String,
    index: usize,
    step: Step,
    kind: &'static str,
    shape: &'static str,
    status: &'static str,
    state: &'static str,
    aria: String,
    expanded: bool,
    skipped: bool,
    handovers: Vec<PhoneHandover>,
}

struct PhoneLane {
    dom_id: String,
    name: String,
    role: String,
    has_role: bool,
    mark: String,
    kind: &'static str,
    tint: String,
    ink: String,
    running: bool,
    items: Vec<PhoneItem>,
}

fn shape_word(shape: StepShape) -> &'static str {
    match shape {
        StepShape::Step => "Step",
        StepShape::Approval => "Approval",
        StepShape::Start => "Start",
        StepShape::Finish => "Finish",
    }
}

fn lane_mark(lane: &Lane) -> String {
    lane.mark.clone().unwrap_or_else(|| {
        lane.name
            .chars()
            .find(|c| c.is_alphabetic())
            .map_or_else(|| "?".to_string(), |c| c.to_uppercase().to_string())
    })
}

/// Focuses the card that is laid out. The wide grid and the phone list each render
/// a card for the open step; the hidden one cannot take focus.
fn focus_visible_card(root: &str) -> String {
    format!(
        r#"requestAnimationFrame(() => requestAnimationFrame(() => {{
    const cards = [...document.querySelectorAll('[data-rpl-card="{root}"]')];
    cards.find((card) => card.offsetParent !== null)?.focus({{ preventScroll: true }});
}}));"#
    )
}

fn focus_card_on_mount(hooks: Hooks, root: String) -> impl FnMut(MountedEvent) {
    move |_| {
        let mut focus = hooks.focus_card;
        if focus() {
            focus.set(false);
            document::eval(&focus_visible_card(&root));
        }
    }
}

fn step_card(diagram: &Diagram, index: usize, hooks: &Hooks) -> Element {
    let placed = &diagram.steps[index];
    let step = &placed.step;
    let lane = &diagram.lanes[placed.lane];
    let kicker = shape_word(step.shape);
    let lane_line = match &lane.role {
        Some(role) => format!("{}, {role}", lane.name),
        None => lane.name.clone(),
    };
    let from: Vec<String> = diagram
        .edges
        .iter()
        .filter(|(_, to)| *to == index)
        .map(|(from, _)| diagram.lanes[diagram.steps[*from].lane].name.clone())
        .collect();
    let to: Vec<String> = diagram
        .edges
        .iter()
        .filter(|(from, _)| *from == index)
        .map(|(_, to)| diagram.lanes[diagram.steps[*to].lane].name.clone())
        .collect();
    let from_line = if from.is_empty() {
        "Starts the run".to_string()
    } else {
        format!("Came from {}", from.join(", "))
    };
    let to_line = if to.is_empty() {
        "Ends the run".to_string()
    } else {
        format!("Goes to {}", to.join(", "))
    };
    let title = step.title.clone();
    let status = step.status.word();
    let status_key = step.status.as_str();
    let detail = step.detail.clone();
    let duration = step.duration_label.clone();
    let can_decide = step.shape == StepShape::Approval && step.status == StepStatus::Waiting;
    let show_approve = can_decide && hooks.on_approve.is_some();
    let show_reject = can_decide && hooks.on_reject.is_some();
    let approve_step = step.clone();
    let reject_step = step.clone();
    let approve_hooks = hooks.clone();
    let reject_hooks = hooks.clone();
    rsx! {
        p { class: "kit-rpl__kicker", "{kicker}" }
        h3 { class: "kit-rpl__card-title", "{title}" }
        p { class: "kit-rpl__lane-line", "{lane_line}" }
        span { class: "kit-rpl__pill", "data-status": "{status_key}",
            span { class: "kit-rpl__dot", "aria-hidden": "true" }
            "{status}"
        }
        if let Some(text) = detail {
            p { class: "kit-rpl__detail", "{text}" }
        }
        if let Some(text) = duration {
            p { class: "kit-rpl__duration", "{text}" }
        }
        p { class: "kit-rpl__flow", "{from_line}" }
        p { class: "kit-rpl__flow", "{to_line}" }
        if show_approve || show_reject {
            div { class: "kit-rpl__decide", role: "group", "aria-label": "Decision",
                if show_approve {
                    button {
                        r#type: "button",
                        class: "kit-rpl__approve",
                        onclick: move |_| approve_hooks.decide(&approve_step, true),
                        "Approve"
                    }
                }
                if show_reject {
                    button {
                        r#type: "button",
                        class: "kit-rpl__reject",
                        onclick: move |_| reject_hooks.decide(&reject_step, false),
                        "Reject"
                    }
                }
            }
        }
    }
}

/// A run as lanes and hand-overs. Each step's card shows where its work came
/// from and where it goes, and an approval can be decided from its card.
#[component]
pub fn RunProcessLanes(
    lanes: Vec<Lane>,
    steps: Vec<Step>,
    #[props(default, into)] run_title: String,
    #[props(default)] run_status: Option<RunStatus>,
    #[props(default)] on_select: Option<EventHandler<Option<Step>>>,
    #[props(default)] on_approve: Option<EventHandler<Step>>,
    #[props(default)] on_reject: Option<EventHandler<Step>>,
    #[props(default = "Hand-overs".to_string(), into)] heading: String,
    #[props(default)] extra_class: String,
) -> Element {
    let root_id = use_hook(|| format!("kit-rpl-{}", next_id()));
    let frame_id = frame_dom_id(&root_id);
    let width = use_element_width(frame_id.clone());
    let narrow = width.is_some_and(|w| w < NARROW_PX);

    let open = use_signal(|| None::<Open>);
    let opener = use_signal(String::new);
    let keyboard = use_signal(|| false);
    let focus_card = use_signal(|| false);

    let diagram = normalise(&lanes, &steps);
    let status = run_status.unwrap_or_else(|| run_status_of(&diagram));
    let done = done_count(&diagram.steps);
    let total = counted_total(&diagram.steps);
    let counter = format!("{done} of {total} done");
    let class = class_list("kit-rpl", &extra_class);
    let heading_label = heading.clone();

    let hooks = Hooks {
        open,
        opener,
        keyboard,
        focus_card,
        on_select,
        on_approve,
        on_reject,
    };

    let open_now = open();
    let is_open = open_now.is_some();
    {
        let hooks = hooks.clone();
        let root = frame_id.clone();
        use_effect(use_reactive((&is_open,), move |(opened,)| {
            if opened {
                let script = outside_script(&root);
                let hooks = hooks.clone();
                spawn(async move {
                    let mut eval = document::eval(&script);
                    if let Ok(true) = eval.recv::<bool>().await {
                        hooks.close();
                    }
                });
            } else {
                document::eval(&cleanup_script(&root));
            }
        }));
    }

    if diagram.steps.is_empty() {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            section { class, "aria-label": "{heading_label}",
                header { class: "kit-rpl__head",
                    h2 { class: "kit-rpl__title", "{heading}" }
                }
                p { class: "kit-rpl__empty", "Nothing to hand over yet" }
            }
        };
    }

    let frame = frame_for(
        width,
        diagram
            .steps
            .iter()
            .map(|s| s.column + 1)
            .max()
            .unwrap_or(1),
    );
    let lane_count = diagram.lanes.len();
    let lanes_height = stage_height(lane_count);
    let height = lanes_height
        + if open_now.is_some() {
            CARD_ROOM_PX
        } else {
            0.0
        };
    let body_style = format!(
        "width: {}px; height: {height}px; zoom: {};",
        frame.label + frame.stage_width,
        frame.scale
    );
    let stage_style = format!("width: {}px; height: {height}px;", frame.stage_width);
    let labels_style = format!("width: {}px; height: {height}px;", frame.label);
    let narrow_attr = narrow.to_string();

    let lane_views: Vec<LaneView> = diagram
        .lanes
        .iter()
        .enumerate()
        .map(|(row, lane)| {
            let mut current: Option<&Placed> = None;
            for placed in diagram.steps.iter().filter(|p| p.lane == row) {
                let rank = |p: &Placed| match p.step.status {
                    StepStatus::Running => 0,
                    StepStatus::Waiting => 1,
                    _ => 2,
                };
                if current.is_none_or(|c| rank(placed) < rank(c)) {
                    current = Some(placed);
                }
            }
            let target = current.map(|p| p.step.clone());
            let mark = lane_mark(lane);
            let (tint, ink) = lane_colours(lane);
            LaneView {
                dom_id: format!("{root_id}-lane-{row}"),
                name: lane.name.clone(),
                role: lane.role.clone().unwrap_or_default(),
                has_role: lane.role.is_some(),
                mark,
                kind: lane.kind.as_str(),
                tint,
                ink,
                top: row as f64 * ROW_PX,
                expanded: target.as_ref().is_some_and(|t| {
                    open_now
                        .as_ref()
                        .is_some_and(|o| o.step == t.id && o.from_lane)
                }),
                running: diagram
                    .steps
                    .iter()
                    .any(|p| p.lane == row && p.step.status == StepStatus::Running),
                aria: format!("{}, {}", lane.name, lane.role.clone().unwrap_or_default()),
                target,
            }
        })
        .collect();

    let mut order: Vec<usize> = (0..diagram.steps.len()).collect();
    order.sort_by_key(|&i| (diagram.steps[i].column, diagram.steps[i].lane));

    let step_views: Vec<StepView> = order
        .iter()
        .map(|&index| {
            let placed = &diagram.steps[index];
            let lane = &diagram.lanes[placed.lane];
            let is_box = placed.step.shape.is_box();
            let cell_left = frame.cell_left(placed.column);
            let top = placed.lane as f64 * ROW_PX;
            let (left, width, top, height) = if is_box {
                (
                    cell_left,
                    frame.cell_width(),
                    top + PORT_Y_PX - BOX_HEIGHT_PX / 2.0,
                    BOX_HEIGHT_PX,
                )
            } else {
                (cell_left, frame.cell_width(), top, ROW_PX)
            };
            let (tint, ink) = lane_colours(lane);
            StepView {
                dom_id: format!("{root_id}-step-{index}"),
                step: placed.step.clone(),
                shape: placed.step.shape.as_str(),
                left,
                top,
                width,
                height,
                tint,
                ink,
                column: placed.column,
                expanded: open_now
                    .as_ref()
                    .is_some_and(|o| o.step == placed.step.id && !o.from_lane),
                aria: format!(
                    "{}, {}, {}",
                    placed.step.title,
                    lane.name,
                    placed.step.status.word()
                ),
                status: placed.step.status.as_str(),
                skipped: placed.step.status == StepStatus::Skipped,
            }
        })
        .collect();

    let connectors: Vec<ConnectorView> = diagram
        .edges
        .iter()
        .map(|&(from, to)| {
            let source = &diagram.steps[from];
            let target = &diagram.steps[to];
            let (_, x1) = frame.ports(source.column, source.step.shape);
            let (x2, _) = frame.ports(target.column, target.step.shape);
            let (body, head) = connector_paths(
                x1,
                port_y(source.lane),
                x2,
                port_y(target.lane),
                frame.channel(target.column),
            );
            // Only a hand-over into a waiting step carries the light; a running step shimmers instead.
            let live = target.step.status == StepStatus::Waiting;
            let waiting = target.step.shape == StepShape::Approval;
            let stroke = if waiting {
                "var(--kit-warn)".to_string()
            } else {
                lane_colours(&diagram.lanes[target.lane]).0
            };
            ConnectorView {
                key: format!("{from}-{to}"),
                body,
                head,
                column: target.column,
                live,
                stroke,
                waiting,
            }
        })
        .collect();

    let hooks_for_escape = hooks.clone();
    let card = open_now.as_ref().and_then(|current| {
        let index = diagram
            .steps
            .iter()
            .position(|p| p.step.id == current.step)?;
        let placed = &diagram.steps[index];
        let (anchor_x, anchor_y) = if current.from_lane {
            (frame.label / 2.0, port_y(placed.lane))
        } else {
            (frame.centre_x(placed.column), port_y(placed.lane))
        };
        let vertical = if anchor_y < lanes_height / 2.0 {
            format!("top: {}px", anchor_y + 18.0)
        } else {
            format!("bottom: {}px", height - anchor_y + 18.0)
        };
        let left = (anchor_x - CARD_WIDTH_PX / 2.0)
            .min(frame.stage_width - CARD_WIDTH_PX)
            .max(0.0);
        let style = format!("left: {left}px; width: {CARD_WIDTH_PX}px; {vertical}");
        Some((style, step_card(&diagram, index, &hooks)))
    });
    let card_hooks = hooks.clone();
    let card_id = format!("{root_id}-card");

    let phone_lanes: Vec<PhoneLane> = diagram
        .lanes
        .iter()
        .enumerate()
        .map(|(row, lane)| {
            let (tint, ink) = lane_colours(lane);
            let items = order
                .iter()
                .copied()
                .filter(|&index| diagram.steps[index].lane == row)
                .map(|index| {
                    let placed = &diagram.steps[index];
                    let handovers = diagram
                        .edges
                        .iter()
                        .filter(|(from, _)| *from == index)
                        .map(|(_, to)| {
                            let target = &diagram.steps[*to];
                            PhoneHandover {
                                lane: diagram.lanes[target.lane].name.clone(),
                                live: target.step.status == StepStatus::Waiting,
                            }
                        })
                        .collect();
                    PhoneItem {
                        dom_id: format!("{root_id}-pstep-{index}"),
                        index,
                        step: placed.step.clone(),
                        kind: shape_word(placed.step.shape),
                        shape: placed.step.shape.as_str(),
                        status: placed.step.status.as_str(),
                        state: placed.step.status.word(),
                        aria: format!(
                            "{}, {}, {}",
                            placed.step.title,
                            lane.name,
                            placed.step.status.word()
                        ),
                        expanded: open_now
                            .as_ref()
                            .is_some_and(|o| o.step == placed.step.id && !o.from_lane),
                        skipped: placed.step.status == StepStatus::Skipped,
                        handovers,
                    }
                })
                .collect();
            PhoneLane {
                dom_id: format!("{root_id}-plane-{row}"),
                name: lane.name.clone(),
                role: lane.role.clone().unwrap_or_default(),
                has_role: lane.role.is_some(),
                mark: lane_mark(lane),
                kind: lane.kind.as_str(),
                tint,
                ink,
                running: diagram
                    .steps
                    .iter()
                    .any(|p| p.lane == row && p.step.status == StepStatus::Running),
                items,
            }
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            id: "{root_id}",
            class,
            "aria-label": "{heading_label}",
            "data-status": status.as_str(),
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && hooks_for_escape.open.peek().is_some() {
                    event.prevent_default();
                    let id = hooks_for_escape.opener.peek().clone();
                    hooks_for_escape.close();
                    focus_after_frame(&id);
                }
            },
            header { class: "kit-rpl__head",
                h2 { class: "kit-rpl__title", "{heading}" }
                span { class: "kit-rpl__title-run", "{run_title}" }
                span { class: "kit-rpl__counter", "{counter}" }
                span { class: "kit-rpl__run", "data-status": status.as_str(), "{status.word()}" }
            }
            div { class: "kit-rpl__sr", role: "status", "aria-live": "polite",
                "Run {status.word()}"
            }
            div {
                id: "{frame_id}",
                class: "kit-rpl__frame kit-rpl__wide",
                "data-narrow": "{narrow_attr}",
                div { class: "kit-rpl__body", style: "{body_style}",
                div { class: "kit-rpl__labels", style: "{labels_style}",
                    for lane in lane_views.iter() {
                        if let Some(target) = lane.target.clone() {
                            button {
                                key: "{lane.dom_id}",
                                id: "{lane.dom_id}",
                                r#type: "button",
                                class: "kit-rpl__lane",
                                "data-kind": lane.kind,
                                "data-rpl-node": "{frame_id}",
                                "aria-expanded": if lane.expanded { "true" } else { "false" },
                                "aria-label": "{lane.aria}",
                                style: "top: {lane.top}px; --rpl-lane: {lane.tint}; --rpl-ink: {lane.ink};",
                                onclick: {
                                    let hooks = hooks.clone();
                                    let dom_id = lane.dom_id.clone();
                                    move |_| hooks.activate(&target, dom_id.clone(), true)
                                },
                                onkeydown: {
                                    let hooks = hooks.clone();
                                    move |event: KeyboardEvent| hooks.keyed(&event)
                                },
                                span { class: if lane.running { "kit-rpl__mark is-wave" } else { "kit-rpl__mark" }, "aria-hidden": "true", "{lane.mark}" }
                                if !narrow {
                                    span { class: "kit-rpl__who",
                                        span { class: "kit-rpl__lane-name", "{lane.name}" }
                                        if lane.has_role {
                                            span { class: "kit-rpl__lane-role", "{lane.role}" }
                                        }
                                    }
                                }
                            }
                        } else {
                            div {
                                key: "{lane.dom_id}",
                                class: "kit-rpl__lane is-empty",
                                "data-kind": lane.kind,
                                style: "top: {lane.top}px; --rpl-lane: {lane.tint}; --rpl-ink: {lane.ink};",
                                span { class: "kit-rpl__mark", "aria-hidden": "true", "{lane.mark}" }
                                if !narrow {
                                    span { class: "kit-rpl__who",
                                        span { class: "kit-rpl__lane-name", "{lane.name}" }
                                        if lane.has_role {
                                            span { class: "kit-rpl__lane-role", "{lane.role}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-rpl__canvas", style: "left: {frame.label}px;",
                div { class: "kit-rpl__stage", style: "{stage_style}",
                    for lane in lane_views.iter() {
                        div {
                            key: "{lane.dom_id}-row",
                            class: "kit-rpl__row",
                            style: "top: {lane.top}px;",
                        }
                    }
                    svg {
                        class: "kit-rpl__links",
                        "viewBox": "0 0 {frame.stage_width} {height}",
                        "aria-hidden": "true",
                        for connector in connectors {
                            path {
                                key: "{connector.key}",
                                class: "kit-rpl__edge",
                                "pathLength": "1",
                                d: "{connector.body}",
                                style: "--rpl-i: {connector.column};",
                            }
                            path {
                                class: "kit-rpl__arrow",
                                d: "{connector.head}",
                                style: "--rpl-i: {connector.column};",
                            }
                            if connector.live {
                                path {
                                    class: if connector.waiting { "kit-rpl__light is-waiting" } else { "kit-rpl__light" },
                                    "pathLength": "1",
                                    d: "{connector.body}",
                                    style: "stroke: {connector.stroke};",
                                }
                            }
                        }
                    }
                    for view in step_views {
                        if view.shape == "step" {
                            button {
                                key: "{view.dom_id}",
                                id: "{view.dom_id}",
                                r#type: "button",
                                class: "kit-rpl__box",
                                "data-status": view.status,
                                "data-rpl-node": "{frame_id}",
                                "aria-haspopup": "dialog",
                                "aria-expanded": if view.expanded { "true" } else { "false" },
                                "aria-label": "{view.aria}",
                                style: "left: {view.left}px; top: {view.top}px; width: {view.width}px; height: {view.height}px; --rpl-i: {view.column}; --rpl-lane: {view.tint}; --rpl-ink: {view.ink};",
                                onclick: {
                                    let hooks = hooks.clone();
                                    let step = view.step.clone();
                                    let dom_id = view.dom_id.clone();
                                    move |_| hooks.activate(&step, dom_id.clone(), false)
                                },
                                onkeydown: {
                                    let hooks = hooks.clone();
                                    move |event: KeyboardEvent| hooks.keyed(&event)
                                },
                                span { class: if view.skipped { "kit-rpl__title-text is-skipped" } else { "kit-rpl__title-text" }, "{view.step.title}" }
                                span { class: "kit-rpl__status", "data-status": view.status, "{view.step.status.word()}" }
                            }
                        } else {
                            button {
                                key: "{view.dom_id}",
                                id: "{view.dom_id}",
                                r#type: "button",
                                class: "kit-rpl__shape",
                                "data-shape": view.shape,
                                "data-status": view.status,
                                "data-rpl-node": "{frame_id}",
                                "aria-haspopup": "dialog",
                                "aria-expanded": if view.expanded { "true" } else { "false" },
                                "aria-label": "{view.aria}",
                                style: "left: {view.left}px; top: {view.top}px; width: {view.width}px; height: {view.height}px; --rpl-i: {view.column}; --rpl-lane: {view.tint}; --rpl-ink: {view.ink};",
                                onclick: {
                                    let hooks = hooks.clone();
                                    let step = view.step.clone();
                                    let dom_id = view.dom_id.clone();
                                    move |_| hooks.activate(&step, dom_id.clone(), false)
                                },
                                onkeydown: {
                                    let hooks = hooks.clone();
                                    move |event: KeyboardEvent| hooks.keyed(&event)
                                },
                                span { class: "kit-rpl__glyph", "data-shape": view.shape, "aria-hidden": "true" }
                                span { class: "kit-rpl__shape-title", "{view.step.title}" }
                                span { class: "kit-rpl__status", "data-status": view.status, "{view.step.status.word()}" }
                            }
                        }
                    }
                }
                if let Some((style, body)) = card {
                    section {
                        id: "{card_id}",
                        class: "kit-rpl__card",
                        role: "region",
                        "aria-label": "{title_of(&diagram, &open_now)}",
                        tabindex: "-1",
                        "data-rpl-card": "{frame_id}",
                        style: "{style}",
                        onmounted: focus_card_on_mount(card_hooks.clone(), frame_id.clone()),
                        {body}
                    }
                }
                }
                }
            }
                div { class: "kit-rpl__phone",
                    for lane in phone_lanes.iter() {
                        section {
                            key: "{lane.dom_id}",
                            class: "kit-rpl__pl-lane",
                            "data-kind": lane.kind,
                            "aria-labelledby": "{lane.dom_id}-name",
                            style: "--rpl-lane: {lane.tint}; --rpl-ink: {lane.ink};",
                            header { class: "kit-rpl__pl-head",
                                span { class: if lane.running { "kit-rpl__mark is-wave" } else { "kit-rpl__mark" }, "aria-hidden": "true", "{lane.mark}" }
                                div { class: "kit-rpl__pl-who",
                                    h3 { id: "{lane.dom_id}-name", class: "kit-rpl__pl-name", "{lane.name}" }
                                    if lane.has_role {
                                        span { class: "kit-rpl__pl-role", "{lane.role}" }
                                    }
                                }
                            }
                            if lane.items.is_empty() {
                                p { class: "kit-rpl__pl-none", "No steps" }
                            }
                            ol { class: "kit-rpl__pl-steps",
                                for item in lane.items.iter() {
                                    li { key: "{item.dom_id}", class: "kit-rpl__pl-item",
                                        button {
                                            id: "{item.dom_id}",
                                            r#type: "button",
                                            class: "kit-rpl__pl-step",
                                            "data-shape": item.shape,
                                            "data-status": item.status,
                                            "data-rpl-node": "{frame_id}",
                                            "aria-haspopup": "dialog",
                                            "aria-expanded": if item.expanded { "true" } else { "false" },
                                            "aria-label": "{item.aria}",
                                            onclick: {
                                                let hooks = hooks.clone();
                                                let step = item.step.clone();
                                                let dom_id = item.dom_id.clone();
                                                move |_| hooks.activate(&step, dom_id.clone(), false)
                                            },
                                            onkeydown: {
                                                let hooks = hooks.clone();
                                                move |event: KeyboardEvent| hooks.keyed(&event)
                                            },
                                            span { class: "kit-rpl__pl-glyph", "data-shape": item.shape, "aria-hidden": "true" }
                                            span { class: "kit-rpl__pl-text",
                                                span { class: "kit-rpl__pl-meta",
                                                    span { class: "kit-rpl__pl-kind", "{item.kind}" }
                                                    span { class: "kit-rpl__pl-state", "data-status": item.status, "{item.state}" }
                                                }
                                                span { class: if item.skipped { "kit-rpl__pl-title is-skipped" } else { "kit-rpl__pl-title" }, "{item.step.title}" }
                                            }
                                        }
                                        for hand in item.handovers.iter() {
                                            p {
                                                class: if hand.live { "kit-rpl__pl-handover is-live" } else { "kit-rpl__pl-handover" },
                                                "data-live": if hand.live { "true" } else { "false" },
                                                "Hands over to {hand.lane}"
                                                if hand.live {
                                                    span { class: "kit-rpl__pl-wait", " · Needs you" }
                                                }
                                            }
                                        }
                                        if item.expanded {
                                            section {
                                                id: "{item.dom_id}-card",
                                                class: "kit-rpl__card kit-rpl__pl-card",
                                                role: "region",
                                                "aria-label": "{item.step.title} details",
                                                tabindex: "-1",
                                                "data-rpl-card": "{frame_id}",
                                                onmounted: focus_card_on_mount(card_hooks.clone(), frame_id.clone()),
                                                {step_card(&diagram, item.index, &hooks)}
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

fn title_of(diagram: &Diagram, open: &Option<Open>) -> String {
    open.as_ref()
        .and_then(|o| diagram.steps.iter().find(|p| p.step.id == o.step))
        .map_or_else(String::new, |p| format!("{} details", p.step.title))
}

fn run_status_of(diagram: &Diagram) -> RunStatus {
    run_status(&diagram.steps)
}
