//! Run tree inspector: one agent run on one screen. Agents form a tree; each card lists its
//! calls as bars, and a chosen call opens in place under its row, with its error text.
//!
//! Design choice: the error count in the header is counted from the calls, so it cannot
//! disagree with the rows shown.
//! Design choice: a call's bar length is its duration against the longest call in the run; a
//! running call's bar grows as the host raises its `duration_secs`.
//! Design choice: the tree is a `tree` with `treeitem`s; agent cards are its items and calls
//! are the children of an expanded card.
//! Design choice: hovering a card lights its own card, its parent's card, its child cards and
//! the child lines that join them; the lines are drawn at all times.
//! Browser checks: card click expands and collapses, failed call shows its error under its row,
//! error count equals error calls, Arrow Up/Down across cards, Right and Left expand and collapse,
//! Escape clears the selected call, hover lights parent and child lines, phone indented list.

use std::collections::BTreeSet;

use dioxus::prelude::*;
use rv2_affect::badge::{Badge, BadgeTone};

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/run-tree-inspector.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Waiting,
    Completed,
    Error,
}

impl RunStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Waiting => "Waiting",
            Self::Completed => "Done",
            Self::Error => "Error",
        }
    }

    fn tone(self) -> BadgeTone {
        match self {
            Self::Running => BadgeTone::Info,
            Self::Waiting => BadgeTone::Neutral,
            Self::Completed => BadgeTone::Success,
            Self::Error => BadgeTone::Danger,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunInfo {
    pub id: String,
    pub title: String,
    pub status: RunStatus,
    pub duration_secs: f64,
    pub cost_usd: f64,
    pub token_count: u64,
    /// Totals for the whole run; the calls passed in may be a subset.
    pub model_call_count: usize,
    pub tool_call_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunAgent {
    pub id: String,
    pub name: String,
    pub model_label: String,
    pub status: RunStatus,
    pub parent_id: Option<String>,
    pub cost_usd: f64,
    pub elapsed_secs: f64,
    pub current_step: Option<String>,
    /// Drawn faded so the card reads as background to the focused agents.
    pub faded: bool,
}

/// Call rows drawn per agent card; the rest are counted in the card's footer.
pub const CALL_PREVIEW: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallKind {
    Model,
    Tool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallStatus {
    Ok,
    Running,
    Error,
}

impl CallStatus {
    fn word(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Running => "running",
            Self::Error => "error",
        }
    }

    /// Glyph shown beside the call name; the word is still in the row's accessible name.
    fn mark(self) -> &'static str {
        match self {
            Self::Ok => "✓",
            Self::Running => "…",
            Self::Error => "✕",
        }
    }
}

/// `96k`, `1.2M`, or the exact count below one thousand.
pub fn token_label(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{}k", count / 1_000)
    } else {
        count.to_string()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunCall {
    pub id: String,
    pub agent_id: String,
    pub kind: CallKind,
    pub name: String,
    pub status: CallStatus,
    pub duration_secs: f64,
    pub error_message: Option<String>,
}

/// Calls with error status: the run's error count.
pub fn error_count(calls: &[RunCall]) -> usize {
    calls
        .iter()
        .filter(|c| c.status == CallStatus::Error)
        .count()
}

/// Bar width of a call as a percent of the longest call in the run.
pub fn bar_percent(duration: f64, longest: f64) -> f64 {
    if longest <= 0.0 {
        0.0
    } else {
        (duration / longest * 100.0).clamp(0.0, 100.0)
    }
}

/// `4.2 s`, `1 min 5 s`, `2 h 3 min`.
pub fn duration_label(secs: f64) -> String {
    let s = secs.max(0.0);
    if s < 60.0 {
        format!("{s:.1} s")
    } else if s < 3600.0 {
        format!("{} min {} s", (s / 60.0) as u64, (s % 60.0) as u64)
    } else {
        format!(
            "{} h {} min",
            (s / 3600.0) as u64,
            ((s % 3600.0) / 60.0) as u64
        )
    }
}

/// Cards in tree order (depth first, agents in the order given), as ids.
pub fn tree_order(agents: &[RunAgent]) -> Vec<String> {
    fn walk(agents: &[RunAgent], parent: Option<&str>, out: &mut Vec<String>) {
        for agent in agents.iter().filter(|a| a.parent_id.as_deref() == parent) {
            out.push(agent.id.clone());
            walk(agents, Some(&agent.id), out);
        }
    }
    let mut out = Vec::new();
    walk(agents, None, &mut out);
    out
}

/// The parent of the hovered card: its parent's child lines light up with the hovered card's.
pub fn parent_of_hovered(agents: &[RunAgent], hovered: Option<&str>) -> Option<String> {
    let id = hovered?;
    agents.iter().find(|a| a.id == id)?.parent_id.clone()
}

/// Which agent cards are expanded.
#[derive(Clone, Copy)]
struct Expansion {
    set: Signal<BTreeSet<String>>,
}

impl Expansion {
    fn toggle(self, id: String) {
        let mut set = self.set;
        let mut next = set();
        if !next.remove(&id) {
            next.insert(id);
        }
        set.set(next);
    }
}

fn focus_card(id: &str) {
    document::eval(&format!(
        "document.getElementById({id:?})?.focus({{ preventScroll: true }});"
    ));
}

fn card_dom(agent: &str) -> String {
    format!("kit-rti-card-{agent}")
}

#[component]
fn AgentNode(
    agent: RunAgent,
    agents: Vec<RunAgent>,
    calls: Vec<RunCall>,
    longest: f64,
    expanded: BTreeSet<String>,
    selected_call_id: Option<String>,
    hovered: Option<String>,
    hovered_parent: Option<String>,
    on_toggle: EventHandler<String>,
    on_select_call: EventHandler<Option<String>>,
    on_hover: EventHandler<Option<String>>,
) -> Element {
    let open = expanded.contains(&agent.id);
    let link = if hovered.as_deref() == Some(agent.id.as_str()) {
        Some("self")
    } else if hovered_parent.as_deref() == Some(agent.id.as_str()) {
        Some("parent")
    } else if hovered.is_some() && agent.parent_id == hovered {
        Some("child")
    } else {
        None
    };
    let children_linked = hovered.as_deref() == Some(agent.id.as_str())
        || hovered_parent.as_deref() == Some(agent.id.as_str());
    let hover_id = agent.id.clone();
    let own: Vec<RunCall> = calls
        .iter()
        .filter(|c| c.agent_id == agent.id)
        .cloned()
        .collect();
    let children: Vec<RunAgent> = agents
        .iter()
        .filter(|a| a.parent_id.as_deref() == Some(agent.id.as_str()))
        .cloned()
        .collect();
    let dom = card_dom(&agent.id);
    let toggle_id = agent.id.clone();
    let key_id = agent.id.clone();
    let elapsed = duration_label(agent.elapsed_secs);
    let cost = format!("${:.2}", agent.cost_usd);
    let hidden_calls = own.len().saturating_sub(CALL_PREVIEW);
    let step = agent.current_step.clone();
    let order = tree_order(&agents);
    let me = order.iter().position(|a| *a == agent.id).unwrap_or(0);
    let prev = me.checked_sub(1).map(|i| order[i].clone());
    let next = order.get(me + 1).cloned();
    rsx! {
        li { key: "{agent.id}", role: "treeitem", "aria-level": "1", "aria-label": "{agent.name}",
            "aria-expanded": if open { "true" } else { "false" },
            div { class: "kit-rti__card", "data-status": agent.status.label().to_lowercase(),
                "data-link": link,
                "data-faded": agent.faded.then_some("true"),
                onmouseenter: move |_| on_hover.call(Some(hover_id.clone())),
                onmouseleave: move |_| on_hover.call(None),
                button {
                    id: "{dom}",
                    r#type: "button",
                    class: "kit-rti__card-head",
                    "aria-expanded": if open { "true" } else { "false" },
                    onclick: move |_| on_toggle.call(toggle_id.clone()),
                    onkeydown: move |e: KeyboardEvent| {
                        let target = match e.key() {
                            Key::ArrowDown => next.clone(),
                            Key::ArrowUp => prev.clone(),
                            _ => None,
                        };
                        if let Some(target) = target {
                            e.prevent_default();
                            focus_card(&card_dom(&target));
                        }
                        if e.key() == Key::ArrowRight && !open {
                            e.prevent_default();
                            on_toggle.call(key_id.clone());
                        }
                        if e.key() == Key::ArrowLeft && open {
                            e.prevent_default();
                            on_toggle.call(key_id.clone());
                        }
                    },
                    span { class: "kit-rti__name", "{agent.name}" }
                    span { class: "kit-rti__model", "{agent.model_label}" }
                    Badge { tone: agent.status.tone(), "{agent.status.label()}" }
                    if let Some(step) = step {
                        span { class: "kit-rti__step", "{step}" }
                    }
                }
                if open {
                    ul { class: "kit-rti__calls", role: "group",
                        for call in own.iter().take(CALL_PREVIEW) {
                            {
                                let picked = selected_call_id.as_deref() == Some(call.id.as_str());
                                let width = bar_percent(call.duration_secs, longest);
                                let call_id = call.id.clone();
                                let aria = format!("{} {}, {}, {}", if call.kind == CallKind::Model { "model" } else { "tool" }, call.name, call.status.word(), duration_label(call.duration_secs));
                                let error = call.error_message.clone();
                                let kind_mark = if call.kind == CallKind::Model { "M" } else { "T" };
                                let status_mark = call.status.mark();
                                rsx! {
                                    li { key: "{call.id}", class: "kit-rti__call", "data-status": call.status.word(), "data-picked": picked.then_some("true"),
                                        button {
                                            r#type: "button",
                                            class: "kit-rti__call-row",
                                            "aria-pressed": if picked { "true" } else { "false" },
                                            "aria-label": "{aria}",
                                            onclick: move |_| on_select_call.call(if picked { None } else { Some(call_id.clone()) }),
                                            span { class: "kit-rti__kind", "aria-hidden": "true", "{kind_mark}" }
                                            span { class: "kit-rti__call-name", "{call.name}" }
                                            span { class: "kit-rti__status-mark", "aria-hidden": "true", "{status_mark}" }
                                            span { class: "kit-rti__bar-track",
                                                span { class: "kit-rti__bar", style: "width: {width:.2}%", "data-status": call.status.word() }
                                            }
                                            span { class: "kit-rti__call-time", "{duration_label(call.duration_secs)}" }
                                        }
                                        if picked {
                                            div { class: "kit-rti__detail",
                                                p { "{call.name}, {call.status.word()}, {duration_label(call.duration_secs)}" }
                                                if let Some(message) = error {
                                                    p { class: "kit-rti__error", role: "alert", "{message}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if own.is_empty() {
                            li { class: "kit-rti__none", "No calls yet" }
                        }
                    }
                }
                div { class: "kit-rti__foot",
                    span { class: "kit-rti__meta", "{cost} · {elapsed}" }
                    span { class: "kit-rti__more", "aria-hidden": "true",
                        "{hidden_calls}"
                        span { class: "kit-rti__chevron" }
                    }
                }
            }
            if !children.is_empty() && open {
                ul { class: "kit-rti__children", role: "group", "data-linked": children_linked.then_some("true"),
                    for child in children.iter().cloned() {
                        AgentNode {
                            key: "{child.id}",
                            agent: child,
                            agents: agents.clone(),
                            calls: calls.clone(),
                            longest: longest,
                            expanded: expanded.clone(),
                            selected_call_id: selected_call_id.clone(),
                            hovered: hovered.clone(),
                            hovered_parent: hovered_parent.clone(),
                            on_toggle: on_toggle,
                            on_select_call: on_select_call,
                            on_hover: on_hover,
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn RunTreeInspector(
    run: RunInfo,
    agents: Vec<RunAgent>,
    calls: Vec<RunCall>,
    #[props(default)] selected_call_id: Option<String>,
    #[props(default)] on_select_call: Option<EventHandler<Option<String>>>,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let expanded = use_signal(|| {
        agents
            .iter()
            .map(|a| a.id.clone())
            .collect::<BTreeSet<String>>()
    });
    let longest = calls
        .iter()
        .map(|c| c.duration_secs)
        .fold(0.0_f64, f64::max);
    let errors = error_count(&calls);
    let agent_count = agents.len();
    let running_agents = agents
        .iter()
        .filter(|a| a.status == RunStatus::Running)
        .count();
    let root_class = class_list("kit-rti", &extra_classes);
    let roots: Vec<RunAgent> = agents
        .iter()
        .filter(|a| a.parent_id.is_none())
        .cloned()
        .collect();
    let select: EventHandler<Option<String>> =
        on_select_call.unwrap_or_else(|| EventHandler::new(|_| {}));
    let expansion = Expansion { set: expanded };
    let selected_now = selected_call_id.clone();
    let has_selection = selected_call_id.is_some();
    let shown_expanded = expanded();
    let mut hovered = use_signal(|| None::<String>);
    let hovered_now = hovered();
    let hovered_parent = parent_of_hovered(&agents, hovered_now.as_deref());

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-label": "Run {run.title}",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape && has_selection {
                    select.call(None);
                }
            },
            header { class: "kit-rti__head",
                div { class: "kit-rti__title-row",
                    span { class: "kit-rti__sparkle", "aria-hidden": "true", "✦" }
                    h2 { class: "kit-rti__title", "{run.title}" }
                    Badge { tone: run.status.tone(), "{run.status.label()}" }
                    p { class: "kit-rti__summary",
                        span { "{agent_count} agents · {running_agents} running" }
                        span { "${run.cost_usd:.2}" }
                    }
                }
                dl { class: "kit-rti__figures",
                    div { dt { "Duration" } dd { "{duration_label(run.duration_secs)}" } }
                    div { dt { "Cost" } dd { "${run.cost_usd:.2}" } }
                    div { dt { "Tokens" } dd { "{token_label(run.token_count)}" } }
                    div { dt { "Model calls" } dd { "{run.model_call_count}" } }
                    div { dt { "Tool calls" } dd { "{run.tool_call_count}" } }
                    div { dt { "Errors" } dd { "{errors}" } }
                }
            }
            if agents.is_empty() {
                p { class: "kit-rti__none", "This run has no agents yet." }
            }
            ul { class: "kit-rti__tree", role: "tree", "aria-label": "Agents in this run",
                for agent in roots.iter().cloned() {
                    AgentNode {
                        key: "{agent.id}",
                        agent: agent,
                        agents: agents.clone(),
                        calls: calls.clone(),
                        longest: longest,
                        expanded: shown_expanded.clone(),
                        selected_call_id: selected_now.clone(),
                        hovered: hovered_now.clone(),
                        hovered_parent: hovered_parent.clone(),
                        on_toggle: move |id: String| expansion.toggle(id),
                        on_select_call: select,
                        on_hover: move |id: Option<String>| hovered.set(id),
                    }
                }
            }
        }
    }
}
