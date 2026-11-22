//! Sub-run tree: a job split across a coordinating unit and the units it spawned, with
//! each unit's progress, its own cost and its elapsed time.
//!
//! Design choice: the tree is a nested list with a left rail at every width, not a
//! drawn link layer. Depth reads from indentation, and the cards share one width.
//! Design choice: the fold toggle comes before its card in the tree order, so Tab
//! reaches toggles ahead of cards; the card sits after it visually.
//! Design choice: the errored or awaiting window closes on Escape or on its badge; an
//! outside press does not close it.
//! Design choice: the host supplies `clock_ms`; the component never schedules a tick.
//! Browser checks: branch fold and unfold; the rising spend figure while a unit accrues;
//! the window focuses its first answer; phone width scrolls inside the container.

use dioxus::prelude::*;

use crate::agents::single_run_status_card::{format_mm_ss, format_usd};
use crate::internal::{KIT_CSS, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/sub-run-tree.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodePhase {
    Pending,
    Active,
    AwaitingInput,
    Complete,
    Errored,
    Halted,
}

impl NodePhase {
    pub fn word(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Active => "Running",
            Self::AwaitingInput => "Needs you",
            Self::Complete => "Done",
            Self::Errored => "Failed",
            Self::Halted => "Halted",
        }
    }

    fn glyph(self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::Active => "◌",
            Self::AwaitingInput => "?",
            Self::Complete => "✓",
            Self::Errored => "✕",
            Self::Halted => "■",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::AwaitingInput => "awaiting_input",
            Self::Complete => "complete",
            Self::Errored => "errored",
            Self::Halted => "halted",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolveChoice {
    Confirm,
    Refuse,
    Retry,
    Dismiss,
}

impl ResolveChoice {
    fn label(self) -> &'static str {
        match self {
            Self::Confirm => "Confirm",
            Self::Refuse => "Refuse",
            Self::Retry => "Retry",
            Self::Dismiss => "Dismiss",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Confirm => "confirm",
            Self::Refuse => "refuse",
            Self::Retry => "retry",
            Self::Dismiss => "dismiss",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ToolMark {
    pub mark_source: Option<String>,
    pub caption: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Node {
    pub node_key: String,
    pub caption: String,
    pub brief: String,
    pub model_tag: Option<String>,
    pub tool_marks: Vec<ToolMark>,
    pub phase: NodePhase,
    pub spend: f64,
    pub began_at_ms: Option<u64>,
    pub ended_at_ms: Option<u64>,
    pub needs_text: Option<String>,
    pub fault_text: Option<String>,
    pub branches: Vec<Node>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TreeSummary {
    pub units: usize,
    pub active: usize,
    pub awaiting: usize,
    pub errored: usize,
    pub spend: f64,
}

/// Spend of a unit plus everything under it.
pub fn subtree_spend(node: &Node) -> f64 {
    node.spend + node.branches.iter().map(subtree_spend).sum::<f64>()
}

/// Elapsed time of a unit: its end (or `clock_ms` while open) minus its start. None while pending.
pub fn elapsed_ms(node: &Node, clock_ms: u64) -> Option<u64> {
    if node.phase == NodePhase::Pending {
        return None;
    }
    let began = node.began_at_ms?;
    Some(node.ended_at_ms.unwrap_or(clock_ms).saturating_sub(began))
}

/// Counts over the whole hierarchy, folded branches included.
pub fn summarize(root: Option<&Node>) -> TreeSummary {
    fn walk(node: &Node, acc: &mut TreeSummary) {
        acc.units += 1;
        acc.spend += node.spend;
        match node.phase {
            NodePhase::Active => acc.active += 1,
            NodePhase::AwaitingInput => acc.awaiting += 1,
            NodePhase::Errored => acc.errored += 1,
            _ => {}
        }
        for child in &node.branches {
            walk(child, acc);
        }
    }
    let mut acc = TreeSummary::default();
    if let Some(root) = root {
        walk(root, &mut acc);
    }
    acc
}

/// Keys of the units drawn when the given branches are folded. A folded unit is drawn; its descendants are not.
pub fn visible_keys(root: &Node, folded: &[String]) -> Vec<String> {
    fn walk(node: &Node, folded: &[String], out: &mut Vec<String>) {
        out.push(node.node_key.clone());
        if folded.contains(&node.node_key) {
            return;
        }
        for child in &node.branches {
            walk(child, folded, out);
        }
    }
    let mut out = Vec::new();
    walk(root, folded, &mut out);
    out
}

/// Identity slot (1..=6) per unit key, so a unit keeps its colour across renders.
pub fn identity_slot(key: &str) -> u32 {
    key.bytes().fold(0u32, |hash, b| {
        hash.wrapping_mul(31).wrapping_add(u32::from(b))
    }) % 6
        + 1
}

/// Label of a fold toggle: how many units sit below, and their spend.
pub fn fold_label(node: &Node, format: &dyn Fn(f64) -> String) -> String {
    let below = node.branches.len();
    let units = if below == 1 { "unit" } else { "units" };
    format!(
        "{below} {units} · {}",
        format(subtree_spend(node) - node.spend)
    )
}

#[derive(Clone, PartialEq, Debug)]
struct Summary {
    units: usize,
    active: usize,
    awaiting: usize,
    errored: usize,
    spend: String,
}

#[component]
pub fn SubRunTree(
    #[props(default)] root_node: Option<Node>,
    #[props(default = "Sub-runs".to_string(), into)] heading: String,
    #[props(default)] clock_ms: u64,
    #[props(default)] folded_keys: Option<Vec<String>>,
    #[props(default)] default_folded_keys: Vec<String>,
    #[props(default)] format_spend: Option<Callback<f64, String>>,
    #[props(default)] on_folded_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] on_node_pick: Option<EventHandler<Node>>,
    #[props(default)] on_node_resolve: Option<EventHandler<(Node, ResolveChoice)>>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-srt-{}", next_id()));
    let mut folded_local = use_signal(|| default_folded_keys.clone());
    let window = use_signal(|| None::<String>);
    let controlled = folded_keys.is_some();
    let folded = folded_keys.unwrap_or_else(|| folded_local());

    let summary = summarize(root_node.as_ref());
    let spend_text = match format_spend {
        Some(format) => format.call(summary.spend),
        None => format_usd(summary.spend),
    };
    let counts = Summary {
        units: summary.units,
        active: summary.active,
        awaiting: summary.awaiting,
        errored: summary.errored,
        spend: spend_text,
    };
    let units_text = format!("{} units", counts.units);
    let active_text = format!("{} active", counts.active);
    let awaiting_text = format!("{} awaiting input", counts.awaiting);
    let errored_text = format!("{} errored", counts.errored);
    let announce = format!(
        "{}, {}, {}, {}, {}",
        units_text, active_text, awaiting_text, errored_text, counts.spend
    );
    let heading_id = format!("{id}-heading");
    let class = format!("kit-srt {class_name}");

    let on_toggle = Callback::new(move |key: String| {
        let mut next = folded_local();
        match next.iter().position(|k| *k == key) {
            Some(at) => {
                next.remove(at);
            }
            None => next.push(key),
        }
        if !controlled {
            folded_local.set(next.clone());
        }
        if let Some(handler) = &on_folded_change {
            handler.call(next);
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{heading_id}",
            header { class: "kit-srt__strip",
                h3 { id: "{heading_id}", class: "kit-srt__heading", {heading} }
                span { class: "kit-srt__stat", {units_text} }
                span { class: "kit-srt__stat", "data-tone": "active", {active_text} }
                span { class: "kit-srt__stat", "data-tone": "awaiting", {awaiting_text} }
                span { class: "kit-srt__stat", "data-tone": "errored", {errored_text} }
                span { class: "kit-srt__stat kit-srt__spend", {counts.spend.clone()} }
            }
            span { class: "kit-srt__live", role: "status", "aria-live": "polite", {announce} }
            if let Some(root) = root_node {
                ul { class: "kit-srt__tree", role: "tree", "aria-label": "Sub-run hierarchy",
                    SubRunBranch {
                        node: root,
                        depth: 0,
                        folded: folded.clone(),
                        clock_ms: clock_ms,
                        format_spend: format_spend,
                        on_toggle: on_toggle,
                        on_pick: on_node_pick,
                        on_resolve: on_node_resolve,
                        window: window,
                        id_prefix: id.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn SubRunBranch(
    node: Node,
    depth: usize,
    folded: Vec<String>,
    clock_ms: u64,
    format_spend: Option<Callback<f64, String>>,
    on_toggle: Callback<String>,
    on_pick: Option<EventHandler<Node>>,
    on_resolve: Option<EventHandler<(Node, ResolveChoice)>>,
    window: Signal<Option<String>>,
    id_prefix: String,
) -> Element {
    let mut window = window;
    let key = node.node_key.clone();
    let is_folded = folded.contains(&key);
    let has_children = !node.branches.is_empty();
    let format = |amount: f64| match format_spend {
        Some(format) => format.call(amount),
        None => format_usd(amount),
    };
    let fold_text = if has_children {
        fold_label(&node, &format)
    } else {
        String::new()
    };
    let elapsed = elapsed_ms(&node, clock_ms)
        .map(format_mm_ss)
        .unwrap_or_default();
    let spend_text = format(node.spend);
    let phase = node.phase;
    let phase_word = phase.word();
    let name = format!("{}, {}", node.caption, phase_word);
    let row_id = format!("{id_prefix}-{}", key);
    let badge_id = format!("{id_prefix}-badge-{}", key);
    let window_id = format!("{id_prefix}-window-{}", key);
    let window_open = window().as_deref() == Some(key.as_str());
    let pickable = on_pick.is_some();
    let depth_attr = depth.to_string();
    let caption = node.caption.clone();
    let brief = node.brief.clone();
    let tags = node.model_tag.clone();
    let marks: Vec<ToolMark> = node.tool_marks.clone();
    let detail_text = match phase {
        NodePhase::AwaitingInput => node.needs_text.clone(),
        NodePhase::Errored => node.fault_text.clone(),
        _ => None,
    };
    let answers: Vec<ResolveChoice> = match phase {
        NodePhase::AwaitingInput => vec![ResolveChoice::Confirm, ResolveChoice::Refuse],
        NodePhase::Errored => vec![ResolveChoice::Retry, ResolveChoice::Dismiss],
        _ => Vec::new(),
    };
    let has_badge = !answers.is_empty();
    let slot = identity_slot(&key);
    let below = node.branches.len();
    let unit_word = if below == 1 { "unit" } else { "units" };
    let fold_action = if is_folded { "Show" } else { "Hide" };
    let fold_aria = format!("{fold_action} {below} {unit_word} under {caption}");
    let folded_text = format!("{below} {unit_word} folded");
    let folded_spend = format(subtree_spend(&node) - node.spend);
    let initial = caption
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    let answer_targets: Vec<(usize, ResolveChoice, Node)> = answers
        .into_iter()
        .enumerate()
        .map(|(index, choice)| (index, choice, node.clone()))
        .collect();
    let children: Vec<(String, Node)> = node
        .branches
        .iter()
        .map(|child| (child.node_key.clone(), child.clone()))
        .collect();
    let click_node = node.clone();
    let key_node = node.clone();
    let toggle_key = key.clone();
    let branch_key = key.clone();
    let branch_folded = folded.clone();
    let pick_handler = on_pick;
    let resolve_handler = on_resolve;
    let mut close_window = window;

    rsx! {
        li {
            class: "kit-srt__unit",
            role: "treeitem",
            "aria-label": "{name}",
            "aria-expanded": has_children.then_some(if is_folded { "false" } else { "true" }),
            "data-phase": phase.as_str(),
            "data-depth": depth_attr,
            if has_children {
                button {
                    class: "kit-srt__fold",
                    r#type: "button",
                    "aria-expanded": if is_folded { "false" } else { "true" },
                    "aria-label": "{fold_aria}",
                    onclick: move |_| on_toggle.call(toggle_key.clone()),
                    if is_folded { "▸ " } else { "▾ " }
                    span { {fold_text.clone()} }
                }
            }
            div {
                id: "{row_id}",
                class: "kit-srt__card",
                tabindex: if pickable { "0" } else { "-1" },
                "data-picked": if pickable { None } else { Some("false") },
                onclick: move |_| {
                    if let Some(handler) = &pick_handler {
                        handler.call(click_node.clone());
                    }
                },
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Enter {
                        if let Some(handler) = &pick_handler {
                            handler.call(key_node.clone());
                        }
                    }
                },
                div { class: "kit-srt__head",
                    span { class: "kit-srt__mark", "aria-hidden": "true",
                        style: "--srt-id: var(--kit-id-{slot}); --srt-id-soft: var(--kit-id-{slot}-soft);",
                        {initial}
                    }
                    span { class: "kit-srt__caption", {caption} }
                    if let Some(tag) = tags {
                        span { class: "kit-srt__model", {tag} }
                    }
                    if !has_badge {
                        span { class: "kit-srt__status", "data-phase": phase.as_str(),
                            span { class: "kit-srt__glyph", "aria-hidden": "true", {phase.glyph()} }
                            {phase_word}
                        }
                    }
                    if has_badge {
                        button {
                            id: "{badge_id}",
                            class: "kit-srt__badge",
                            r#type: "button",
                            "aria-expanded": if window_open { "true" } else { "false" },
                            "aria-haspopup": "dialog",
                            onclick: move |_| {
                                window.set(if window_open { None } else { Some(branch_key.clone()) });
                            },
                            {phase_word}
                        }
                    }
                }
                p { class: "kit-srt__brief", {brief} }
                if !marks.is_empty() {
                    div { class: "kit-srt__tools",
                        for mark in marks {
                            span { class: "kit-srt__tool", title: mark.caption.clone(),
                                span { class: "kit-srt__tool-mark", "aria-hidden": "true",
                                    if let Some(src) = mark.mark_source.clone() {
                                        img { src: src, alt: "" }
                                    }
                                }
                                span { class: "kit-srt__tool-caption", {mark.caption.clone()} }
                            }
                        }
                    }
                }
                div { class: "kit-srt__figures",
                    span { class: "kit-srt__spend-figure", {spend_text} }
                    if !elapsed.is_empty() {
                        span { class: "kit-srt__elapsed", {elapsed} }
                    }
                }
                if window_open && has_badge {
                    div {
                        id: "{window_id}",
                        class: "kit-srt__window",
                        role: "dialog",
                        "aria-label": "{name}",
                        onkeydown: move |event: KeyboardEvent| {
                            event.stop_propagation();
                            if event.key() == Key::Escape {
                                close_window.set(None);
                                focus_after_frame(&badge_id);
                            }
                        },
                        if let Some(text) = detail_text {
                            p { class: "kit-srt__window-text", {text} }
                        }
                        div { class: "kit-srt__answers",
                            for (index, choice, target) in answer_targets {
                                button {
                                    key: "{index}",
                                    class: "kit-srt__answer",
                                    r#type: "button",
                                    "data-choice": choice.as_str(),
                                    onmounted: move |event: MountedEvent| {
                                        if index == 0 {
                                            let node = event.data();
                                            spawn(async move {
                                                let _ = node.set_focus(true).await;
                                            });
                                        }
                                    },
                                    onclick: move |event: MouseEvent| {
                                        event.stop_propagation();
                                        if let Some(handler) = &resolve_handler {
                                            handler.call((target.clone(), choice));
                                        }
                                    },
                                    {choice.label()}
                                }
                            }
                        }
                    }
                }
            }
            if is_folded && has_children {
                div { class: "kit-srt__folded",
                    span { class: "kit-srt__folded-text", {folded_text} }
                    span { class: "kit-srt__spend-figure", {folded_spend} }
                }
            }
            if !is_folded && has_children {
                ul { class: "kit-srt__children", role: "group",
                    for (child_key, child) in children {
                        SubRunBranch {
                            key: "{child_key}",
                            node: child,
                            depth: depth + 1,
                            folded: branch_folded.clone(),
                            clock_ms: clock_ms,
                            format_spend: format_spend,
                            on_toggle: on_toggle,
                            on_pick: pick_handler,
                            on_resolve: resolve_handler,
                            window: window,
                            id_prefix: id_prefix.clone(),
                        }
                    }
                }
            }
        }
    }
}
