//! Cause analysis tree: a failure at the top and every hypothesis beneath it, with the evidence behind each verdict.
//!
//! Design choice: a confirmed cause with children counts as confirmed only once its join rule is met. An unmet
//! join shows as `Pending`, never `Confirmed`.
//! Design choice: marks are text glyphs (`cause_mark: Option<String>`), not nodes.
//! Design choice: the layout is `agent_context_map::tree_positions`. Below the narrow breakpoint CSS drops the
//! absolute positions and the same DOM reads as an indented list with join rules as text.
//! Browser checks: path draw-in on a confirmed root, evidence height easing, arrow focus movement, narrow reflow.

use dioxus::prelude::*;

use crate::components::agent_context_map::tree_positions;
use crate::internal::*;

const CSS: Asset = asset!("/assets/cause-analysis-tree.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JoinRule {
    /// One satisfied child is enough.
    #[default]
    Any,
    /// The children only cause the failure together.
    All,
}

impl JoinRule {
    pub fn word(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::All => "all",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckState {
    #[default]
    Open,
    Checking,
    RuledOut,
    Confirmed,
}

/// What a cause's node shows once its children are taken into account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effective {
    Open,
    Checking,
    RuledOut,
    Confirmed,
    Root,
    Pending,
}

impl Effective {
    pub fn word(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Checking => "checking",
            Self::RuledOut => "ruled out",
            Self::Confirmed => "confirmed",
            Self::Root => "root cause",
            Self::Pending => "pending",
        }
    }

    /// True once the cause is confirmed and its join is met, so it can carry the investigation.
    pub fn is_satisfied(self) -> bool {
        matches!(self, Self::Confirmed | Self::Root)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Evidence {
    pub evidence_kind: String,
    pub evidence_source: Option<String>,
    pub evidence_content: String,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Cause {
    pub cause_id: String,
    pub cause_title: String,
    pub cause_mark: Option<String>,
    pub file_paths: Vec<String>,
    pub join_rule: JoinRule,
    pub check_state: CheckState,
    pub verdict_text: Option<String>,
    pub evidence: Option<Evidence>,
    pub checked_at: Option<f64>,
    pub sub_causes: Vec<Cause>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CauseAction {
    Fix,
    Retry,
}

/// Effective state of a cause. Confirmed with children needs its join rule met; with none it is the root cause.
pub fn effective(cause: &Cause) -> Effective {
    match cause.check_state {
        CheckState::Open => Effective::Open,
        CheckState::Checking => Effective::Checking,
        CheckState::RuledOut => Effective::RuledOut,
        CheckState::Confirmed if cause.sub_causes.is_empty() => Effective::Root,
        CheckState::Confirmed => {
            let satisfied: Vec<bool> = cause
                .sub_causes
                .iter()
                .map(|c| effective(c).is_satisfied())
                .collect();
            let met = match cause.join_rule {
                JoinRule::Any => satisfied.iter().any(|s| *s),
                JoinRule::All => satisfied.iter().all(|s| *s),
            };
            if met {
                Effective::Confirmed
            } else {
                Effective::Pending
            }
        }
    }
}

/// One row of the flattened tree; index 0 is the failure.
#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub cause: Cause,
    pub state: Effective,
    pub depth: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

/// Flattens the failure and its hypotheses depth-first. Children keep their given order.
pub fn flatten(failure: &Cause) -> Vec<Node> {
    fn push(nodes: &mut Vec<Node>, cause: &Cause, depth: usize, parent: Option<usize>) -> usize {
        let index = nodes.len();
        nodes.push(Node {
            cause: cause.clone(),
            state: effective(cause),
            depth,
            parent,
            children: Vec::new(),
        });
        for child in &cause.sub_causes {
            let c = push(nodes, child, depth + 1, Some(index));
            nodes[index].children.push(c);
        }
        index
    }
    let mut nodes = Vec::new();
    push(&mut nodes, failure, 0, None);
    nodes
}

/// Which nodes sit on the path from a root cause up to the failure.
pub fn highlighted_path(nodes: &[Node]) -> Vec<bool> {
    let mut on = vec![false; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        if node.state == Effective::Root {
            let mut cursor = Some(i);
            while let Some(c) = cursor {
                on[c] = true;
                cursor = nodes[c].parent;
            }
        }
    }
    on
}

/// Hypotheses, excluding the failure, and how many are ruled out.
pub fn hypothesis_counts(nodes: &[Node]) -> (usize, usize) {
    let hypotheses = nodes.len().saturating_sub(1);
    let ruled = nodes
        .iter()
        .skip(1)
        .filter(|n| n.state == Effective::RuledOut)
        .count();
    (hypotheses, ruled)
}

/// Investigation status for the summary line.
pub fn investigation_status(nodes: &[Node]) -> &'static str {
    if nodes.iter().any(|n| n.state == Effective::Root) {
        "Root cause found"
    } else if nodes.iter().any(|n| n.state == Effective::Checking) {
        "Investigating"
    } else {
        "Open"
    }
}

/// Keyboard target for a node: up to parent, down to first child, left/right among siblings,
/// Home to the failure, End to the last node.
pub fn keyboard_target(nodes: &[Node], current: usize, key: &Key) -> usize {
    let node = &nodes[current];
    match key {
        Key::ArrowUp => node.parent.unwrap_or(current),
        Key::ArrowDown => node.children.first().copied().unwrap_or(current),
        Key::ArrowLeft | Key::ArrowRight => {
            let Some(parent) = node.parent else {
                return current;
            };
            let siblings = &nodes[parent].children;
            let pos = siblings.iter().position(|&s| s == current).unwrap_or(0);
            if *key == Key::ArrowRight {
                siblings.get(pos + 1).copied().unwrap_or(current)
            } else {
                pos.checked_sub(1).map(|p| siblings[p]).unwrap_or(current)
            }
        }
        Key::Home => 0,
        Key::End => nodes.len() - 1,
        _ => current,
    }
}

/// Evidence shown for the selected cause; `None` falls back to the investigation summary.
pub fn evidence_for(nodes: &[Node], selected: Option<usize>) -> Option<&Cause> {
    selected.and_then(|i| nodes.get(i)).map(|n| &n.cause)
}

/// Cause tree with evidence. A confirmed root cause draws the path from itself to the failure.
#[component]
pub fn CauseAnalysisTree(
    failure_node: Cause,
    #[props(default)] failure_detail: Option<String>,
    #[props(default)] on_cause_action: Option<EventHandler<(Cause, CauseAction)>>,
    #[props(default)] format_time: Option<Callback<f64, String>>,
    #[props(default = "Root cause".to_string(), into)] board_title: String,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-cat-{}", next_id()));
    let mut selected = use_signal(|| None::<usize>);
    let mut focused = use_signal(|| 0usize);

    let nodes = flatten(&failure_node);
    let (hypotheses, ruled) = hypothesis_counts(&nodes);
    let status = investigation_status(&nodes);
    let on_path = highlighted_path(&nodes);
    let class = class_list("kit-cat", &extra_class);

    let children: Vec<Vec<usize>> = nodes.iter().map(|n| n.children.clone()).collect();
    let raw = tree_positions(&children, 0, 1.0, 1.0);
    let (min_x, max_x) = raw.iter().fold((f64::MAX, f64::MIN), |(lo, hi), (x, _)| {
        (lo.min(*x), hi.max(*x))
    });
    let span = (max_x - min_x).max(1.0);
    let levels = nodes.iter().map(|n| n.depth).max().unwrap_or(0) + 1;
    // Nodes are centred on x and up to 26% of the tree wide, so the outermost centres sit 13% in.
    let pct_x = |x: f64| 13.0 + (x - min_x) / span * 74.0;
    let pct_y = |y: f64| 6.0 + y / levels.max(2) as f64 * 88.0;
    let box_height = format!("height: {}px;", 96 * levels + 24);

    let selected_idx = selected();
    let evidence = evidence_for(&nodes, selected_idx).cloned();
    let evidence_node = selected_idx.and_then(|i| nodes.get(i)).cloned();
    let fallback_text = format!("{status} · {hypotheses} hypotheses · {ruled} ruled out");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title",
            header { class: "kit-cat__head",
                h2 { id: "{base}-title", class: "kit-cat__title", "{board_title}" }
                p { class: "kit-cat__summary", role: "status", "aria-live": "polite", "{fallback_text}" }
            }
            div { class: "kit-cat__tree", role: "tree", "aria-label": "Hypotheses", style: "{box_height}",
                svg { class: "kit-cat__wires", view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    for (i, node) in nodes.iter().enumerate() {
                        for child in node.children.iter() {
                            {
                                let (px, py) = (pct_x(raw[i].0), pct_y(raw[i].1));
                                let (cx, cy) = (pct_x(raw[*child].0), pct_y(raw[*child].1));
                                let lit = on_path[*child] && on_path[i];
                                rsx! {
                                    line {
                                        key: "{i}-{child}", class: "kit-cat__wire",
                                        "data-path": lit.then_some("true"),
                                        x1: "{px}", y1: "{py}", x2: "{cx}", y2: "{cy}",
                                    }
                                }
                            }
                        }
                    }
                }
                for (i, node) in nodes.iter().enumerate() {
                    {
                        let (x, y) = (pct_x(raw[i].0), pct_y(raw[i].1));
                        let style = format!("--cat-x: {x:.2}%; --cat-y: {y:.2}%; --cat-depth: {};", node.depth);
                        let dom_id = format!("{base}-n-{i}");
                        let is_failure = i == 0;
                        let is_selected = selected_idx == Some(i);
                        let title = node.cause.cause_title.clone();
                        let state_word = if is_failure { "failure".to_string() } else { node.state.word().to_string() };
                        let join_text = (!node.children.is_empty()).then(|| node.cause.join_rule.word());
                        let when = node.cause.checked_at.map(|t| match format_time {
                            Some(f) => f.call(t),
                            None => format!("{:.0}s", t),
                        });
                        let file = node.cause.file_paths.first().cloned();
                        let more_files = node.cause.file_paths.len().saturating_sub(1);
                        let label = format!("{title}, {state_word}{}", join_text.map(|j| format!(", join {j}")).unwrap_or_default());
                        let mark = node.cause.cause_mark.clone();
                        let detail = if is_failure { failure_detail.clone() } else { None };
                        let key_nav = nodes.clone();
                        rsx! {
                            div {
                                key: "{i}", class: "kit-cat__node", style: "{style}",
                                "data-state": node.state.word(), "data-failure": is_failure.then_some("true"),
                                role: "treeitem", "aria-level": "{node.depth + 1}", "aria-selected": "{is_selected}",
                                "aria-label": "{label}",
                                id: "{dom_id}",
                                tabindex: if focused() == i { "0" } else { "-1" },
                                onclick: move |_| {
                                    selected.set(if selected() == Some(i) { None } else { Some(i) });
                                    focused.set(i);
                                },
                                onkeydown: {
                                    let base_nav = base.clone();
                                    move |event| {
                                        match event.key() {
                                            Key::Enter => {
                                                event.prevent_default();
                                                selected.set(if selected() == Some(i) { None } else { Some(i) });
                                            }
                                            Key::Character(c) if c == " " => {
                                                event.prevent_default();
                                                selected.set(if selected() == Some(i) { None } else { Some(i) });
                                            }
                                            Key::Escape => selected.set(None),
                                            key => {
                                                let next = keyboard_target(&key_nav, i, &key);
                                                if next != i {
                                                    event.prevent_default();
                                                    focused.set(next);
                                                    document::eval(&format!("document.getElementById('{base_nav}-n-{next}')?.focus();"));
                                                }
                                            }
                                        }
                                    }
                                },
                                if let Some(m) = mark { span { class: "kit-cat__mark", "aria-hidden": "true", "{m}" } }
                                span { class: "kit-cat__title-text", "{title}" }
                                span { class: "kit-cat__badge", "{state_word}" }
                                if let Some(j) = join_text { span { class: "kit-cat__join", "{j}" } }
                                if let Some(f) = file { span { class: "kit-cat__file", "{f}" } }
                                if more_files > 0 { span { class: "kit-cat__file", "+{more_files}" } }
                                if let Some(w) = when { span { class: "kit-cat__time", "{w}" } }
                                if let Some(d) = detail { span { class: "kit-cat__detail", "{d}" } }
                            }
                        }
                    }
                }
            }
            section { class: "kit-cat__evidence", "aria-live": "polite", "aria-label": "Evidence",
                if let Some(cause) = evidence {
                    p { class: "kit-cat__verdict", "{cause.verdict_text.clone().unwrap_or_default()}" }
                    if let Some(ev) = cause.evidence.clone() {
                        p { class: "kit-cat__label",
                            "{ev.evidence_kind}"
                            if let Some(src) = ev.evidence_source.clone() { span { " · {src}" } }
                        }
                        pre { class: "kit-cat__code", tabindex: "0", "{ev.evidence_content}" }
                    }
                    if !cause.file_paths.is_empty() {
                        ul { class: "kit-cat__files",
                            for file in cause.file_paths.iter() { li { key: "{file}", "{file}" } }
                        }
                    }
                    if let (Some(handler), Some(node)) = (on_cause_action, evidence_node.clone()) {
                        if node.depth > 0 && node.state == Effective::Root {
                            div { class: "kit-cat__actions",
                                button { r#type: "button", class: "kit-cat__action", onclick: {
                                    let cause = node.cause.clone();
                                    move |_| handler.call((cause.clone(), CauseAction::Fix))
                                }, "Fix" }
                                button { r#type: "button", class: "kit-cat__action", onclick: {
                                    let cause = node.cause.clone();
                                    move |_| handler.call((cause.clone(), CauseAction::Retry))
                                }, "Retry" }
                            }
                        }
                    }
                } else {
                    p { class: "kit-cat__verdict", "Select a hypothesis to see its evidence." }
                }
            }
        }
    }
}
