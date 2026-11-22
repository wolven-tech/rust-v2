//! Cause analysis tree: a failure at the top and every hypothesis beneath it, with the evidence behind each verdict.
//!
//! Design choice: a confirmed cause with children counts as confirmed only once its join rule is met. An unmet
//! join shows as `Pending`, never `Confirmed`.
//! Design choice: marks are text glyphs (`cause_mark: Option<String>`), not nodes.
//! Design choice: the layout is CSS grid, one row per depth and one column per leaf. Connectors are pseudo
//! elements on the cards, so spacing follows the real card height. Below the narrow breakpoint CSS drops the
//! grid and the same DOM reads as an indented list with join rules as text.
//! Browser checks: path draw-in on a confirmed root, evidence height easing, arrow focus movement, narrow reflow.

use dioxus::prelude::*;

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
            Self::Open => "Waiting",
            Self::Checking => "Running",
            Self::RuledOut => "Skipped",
            Self::Confirmed => "Done",
            Self::Root => "Failed",
            Self::Pending => "Waiting",
        }
    }

    /// Lowercase hook for `data-state`, independent of the displayed word.
    pub fn key(self) -> &'static str {
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

/// Half-open leaf-column span of each node. Leaves take consecutive columns depth-first and a
/// parent spans its children, so cards on one row never share a column.
fn leaf_columns(children: &[Vec<usize>]) -> Vec<(usize, usize)> {
    fn walk(children: &[Vec<usize>], node: usize, next: &mut usize, out: &mut [(usize, usize)]) {
        let start = *next;
        if children[node].is_empty() {
            *next += 1;
        }
        for &child in &children[node] {
            walk(children, child, next, out);
        }
        out[node] = (start, *next);
    }
    let mut out = vec![(0, 0); children.len()];
    if !children.is_empty() {
        let mut next = 0;
        walk(children, 0, &mut next, &mut out);
    }
    out
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
    let columns = leaf_columns(&children);
    let leaves = nodes
        .iter()
        .filter(|n| n.children.is_empty())
        .count()
        .max(1);
    let mut sibling_edge: Vec<Option<&'static str>> = vec![None; nodes.len()];
    for node in &nodes {
        let n = node.children.len();
        for (k, &child) in node.children.iter().enumerate() {
            sibling_edge[child] = Some(match k {
                _ if n == 1 => "only",
                0 => "first",
                _ if k + 1 == n => "last",
                _ => "mid",
            });
        }
    }
    let lit: Vec<bool> = (0..nodes.len())
        .map(|i| nodes[i].parent.is_some_and(|p| on_path[i] && on_path[p]))
        .collect();
    let down_lit: Vec<bool> = nodes
        .iter()
        .map(|n| n.children.iter().any(|&c| lit[c]))
        .collect();

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
            div { class: "kit-cat__tree", role: "tree", "aria-label": "Hypotheses", style: "--cat-cols: {leaves};",
                for (i, node) in nodes.iter().enumerate() {
                    {
                        let (start, end) = columns[i];
                        let style = format!(
                            "--cat-row: {}; --cat-col-start: {}; --cat-col-end: {}; --cat-depth: {};",
                            node.depth + 1,
                            start + 1,
                            end + 1,
                            node.depth
                        );
                        let edge = sibling_edge[i];
                        let dom_id = format!("{base}-n-{i}");
                        let is_failure = i == 0;
                        let is_selected = selected_idx == Some(i);
                        let title = node.cause.cause_title.clone();
                        let state_word = if is_failure { "Failed".to_string() } else { node.state.word().to_string() };
                        let join_text = (!node.children.is_empty()).then(|| node.cause.join_rule.word());
                        let when = node.cause.checked_at.map(|t| match format_time {
                            Some(f) => f.call(t),
                            None => format!("{:.0}s", t),
                        });
                        let file = node.cause.file_paths.first().cloned();
                        let more_files = node.cause.file_paths.len().saturating_sub(1);
                        let label = format!("{title}, {state_word}{}", join_text.map(|j| format!(", join {j}")).unwrap_or_default());
                        let tile_text = node.cause.cause_mark.clone().unwrap_or_else(|| title.chars().take(1).collect());
                        let detail = if is_failure { failure_detail.clone() } else { None };
                        let key_nav = nodes.clone();
                        rsx! {
                            div {
                                key: "{i}", class: "kit-cat__cell", style: "{style}",
                                "data-edge": edge,
                                "data-path": lit[i].then_some("true"),
                            div {
                                class: "kit-cat__node",
                                "data-branch": (!node.children.is_empty()).then_some("true"),
                                "data-path-down": down_lit[i].then_some("true"),
                                "data-state": node.state.key(), "data-failure": is_failure.then_some("true"),
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
                                span { class: "kit-cat__mark", "aria-hidden": "true", "{tile_text}" }
                                span { class: "kit-cat__body",
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
