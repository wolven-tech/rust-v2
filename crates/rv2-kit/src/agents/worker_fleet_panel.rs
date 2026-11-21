//! Worker fleet panel: several top-level agents in a column of cards. Each card condenses
//! its sub-agents into one line and a progress ring; opening a card unfolds its sub-agent
//! tree beside it, and a sub-agent's work window lists its items.
//!
//! Design choice: the open card is seeded from `expanded_id` and then kept internally;
//! `on_expand_change` reports each change. This lets a card always fold.
//! Design choice: the open tree is placed beside the column when the container is wide
//! and under its card otherwise (CSS container query).
//! Design choice: spend is summed here from each branch; callers pass per-agent spend only.
//! Browser checks: opening a tree retracts the old one first; the work window drops down
//! with its items fading in; Up, Down, Home and End move between cards; state words stay
//! visible at phone width; reduced motion switches trees at once.

use dioxus::prelude::*;

use crate::agents::single_run_status_card::{focus_by_id, format_mm_ss, initial_of};
use crate::internal::{KIT_CSS, focus_after_frame, next_id, use_element_width, use_now_ms};

/// Below this container width the open tree sits under its lead card instead of beside the list.
const NARROW_WIDTH_PX: f64 = 560.0;

const CSS: Asset = asset!("/assets/worker-fleet-panel.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FleetState {
    Queued,
    Running,
    Waiting,
    Done,
    Failed,
    Stopped,
}

impl FleetState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Waiting => "Needs you",
            Self::Done => "Done",
            Self::Failed => "Failed",
            Self::Stopped => "Stopped",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkState {
    Running,
    Waiting,
    Failed,
    Queued,
    Skipped,
}

impl WorkState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Failed => "failed",
            Self::Queued => "queued",
            Self::Skipped => "skipped",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RespondAction {
    Approve,
    Reject,
    Retry,
    Dismiss,
}

impl RespondAction {
    fn label(self) -> &'static str {
        match self {
            Self::Approve => "Approve",
            Self::Reject => "Reject",
            Self::Retry => "Retry",
            Self::Dismiss => "Dismiss",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct WorkItem {
    pub label: String,
    /// `None` counts as done.
    pub state: Option<WorkState>,
    pub tool_name: Option<String>,
    pub duration: Option<u64>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SubAgent {
    pub id: String,
    pub name: String,
    pub assignment: String,
    pub model_label: Option<String>,
    pub state: FleetState,
    pub work_items: Vec<WorkItem>,
    pub spend: Option<f64>,
    pub question: Option<String>,
    pub failure_text: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ToolMark {
    pub mark_source: Option<String>,
    pub caption: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LeadAgent {
    pub id: String,
    pub name: String,
    pub assignment: String,
    pub model_label: Option<String>,
    pub picture: Option<String>,
    pub tool_marks: Vec<ToolMark>,
    pub state: FleetState,
    pub spend: Option<f64>,
    pub started_at_ms: Option<u64>,
    pub ended_at_ms: Option<u64>,
    pub sub_agents: Vec<SubAgent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct FleetTotals {
    pub agents: usize,
    pub sub_agents: usize,
    pub running: usize,
    pub needs_you: usize,
    pub spend: f64,
}

/// Spend of a lead's sub-agents.
pub fn branch_spend(lead: &LeadAgent) -> f64 {
    lead.sub_agents.iter().filter_map(|s| s.spend).sum()
}

/// Header totals. Each spend is counted once: a lead's own plus its sub-agents'.
pub fn fleet_totals(leads: &[LeadAgent]) -> FleetTotals {
    let mut totals = FleetTotals {
        agents: leads.len(),
        ..FleetTotals::default()
    };
    for lead in leads {
        totals.spend += lead.spend.unwrap_or(0.0);
        if lead.state == FleetState::Running {
            totals.running += 1;
        }
        if lead.state == FleetState::Waiting {
            totals.needs_you += 1;
        }
        for sub in &lead.sub_agents {
            totals.sub_agents += 1;
            totals.spend += sub.spend.unwrap_or(0.0);
            match sub.state {
                FleetState::Running => totals.running += 1,
                FleetState::Waiting => totals.needs_you += 1,
                _ => {}
            }
        }
    }
    totals
}

/// The most urgent state among sub-agents: needs you, failed, running, then done.
pub fn urgent_phrase(subs: &[SubAgent]) -> Option<&'static str> {
    [
        (FleetState::Waiting, "Needs you"),
        (FleetState::Failed, "Failed"),
        (FleetState::Running, "Running"),
        (FleetState::Done, "Done"),
    ]
    .into_iter()
    .find(|(state, _)| subs.iter().any(|s| s.state == *state))
    .map(|(_, phrase)| phrase)
}

/// Finished sub-agents and the total: the progress ring's share.
pub fn done_share(subs: &[SubAgent]) -> (usize, usize) {
    (
        subs.iter().filter(|s| s.state == FleetState::Done).count(),
        subs.len(),
    )
}

/// Actions a sub-agent's window offers for its state.
pub fn response_actions(state: FleetState) -> &'static [RespondAction] {
    match state {
        FleetState::Waiting => &[RespondAction::Approve, RespondAction::Reject],
        FleetState::Failed => &[RespondAction::Retry, RespondAction::Dismiss],
        _ => &[],
    }
}

/// Next lead for a navigation key, or `None`.
pub fn next_lead(len: usize, current: usize, key: &str) -> Option<usize> {
    if len == 0 {
        return None;
    }
    match key {
        "ArrowDown" => Some((current + 1).min(len - 1)),
        "ArrowUp" => Some(current.saturating_sub(1)),
        "Home" => Some(0),
        "End" => Some(len - 1),
        _ => None,
    }
}

#[derive(Clone, PartialEq, Debug)]
struct LeadView {
    index: usize,
    id: String,
    card_id: String,
    tree_id: String,
    name: String,
    model: Option<String>,
    picture: Option<String>,
    initial: String,
    assignment: String,
    tool_marks: Vec<ToolMark>,
    state: FleetState,
    phrase: &'static str,
    ring: String,
    total: String,
    elapsed: String,
    expanded: bool,
    dimmed: bool,
    tab: &'static str,
}

#[component]
pub fn WorkerFleetPanel(
    #[props(default = "Agents".to_string(), into)] heading: String,
    #[props(default)] lead_agents: Vec<LeadAgent>,
    #[props(default)] expanded_id: Option<String>,
    #[props(default)] on_expand_change: Option<EventHandler<Option<String>>>,
    #[props(default)] on_pick: Option<EventHandler<SubAgent>>,
    #[props(default)] on_respond: Option<EventHandler<(SubAgent, RespondAction)>>,
    #[props(default)] clock_now_ms: Option<u64>,
    #[props(default)] format_spend: Option<Callback<f64, String>>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-wfp-{}", next_id()));
    let now = use_now_ms(clock_now_ms, 1000);
    let mut expanded = use_signal(|| expanded_id.clone());
    let work_open = use_signal(|| None::<String>);
    let mut focus = use_signal(|| 0_usize);

    let open = expanded();
    let totals = fleet_totals(&lead_agents);
    let money = |amount: f64| match format_spend {
        Some(format) => format.call(amount),
        None => format!("${amount:.2}"),
    };
    let spend_text = money(totals.spend);
    let agents_text = format!("{} agents", totals.agents);
    let subs_text = format!("{} sub-agents", totals.sub_agents);
    let running_text = format!("{} running", totals.running);
    let needs_text = format!("{} need you", totals.needs_you);
    let live = format!("{agents_text}, {running_text}, {needs_text}, {spend_text}");
    let focused = focus().min(lead_agents.len().saturating_sub(1));

    let leads: Vec<LeadView> = lead_agents
        .iter()
        .enumerate()
        .map(|(index, lead)| {
            let (done, total) = done_share(&lead.sub_agents);
            let ring = if total == 0 {
                "0%".to_string()
            } else {
                format!("{:.0}%", done as f64 / total as f64 * 100.0)
            };
            let own = lead.spend.unwrap_or(0.0);
            let lead_total = own + branch_spend(lead);
            let elapsed_end = lead.ended_at_ms.unwrap_or(now);
            let elapsed = lead
                .started_at_ms
                .map(|start| format_mm_ss(elapsed_end.saturating_sub(start)))
                .unwrap_or_default();
            LeadView {
                index,
                id: lead.id.clone(),
                card_id: format!("{id}-card-{}", lead.id),
                tree_id: format!("{id}-tree-{}", lead.id),
                name: lead.name.clone(),
                model: lead.model_label.clone(),
                picture: lead.picture.clone(),
                initial: initial_of(&lead.name),
                assignment: lead.assignment.clone(),
                tool_marks: lead.tool_marks.clone(),
                state: lead.state,
                phrase: urgent_phrase(&lead.sub_agents).unwrap_or(""),
                ring,
                total: money(lead_total),
                elapsed,
                expanded: open.as_deref() == Some(lead.id.as_str()),
                dimmed: open.is_some() && open.as_deref() != Some(lead.id.as_str()),
                tab: if index == focused { "0" } else { "-1" },
            }
        })
        .collect();

    let open_lead: Option<LeadAgent> = open
        .as_deref()
        .and_then(|key| lead_agents.iter().find(|l| l.id == key).cloned());
    let on_expand = Callback::new(move |next: Option<String>| {
        expanded.set(next.clone());
        if let Some(handler) = &on_expand_change {
            handler.call(next);
        }
    });
    let order_len = leads.len();
    let heading_id = format!("{id}-heading");
    let class = format!("kit-wfp {class_name}");
    let narrow = use_element_width(id.clone()).is_some_and(|w| w < NARROW_WIDTH_PX);

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", id: "{id}", "aria-labelledby": "{heading_id}",
            header { class: "kit-wfp__head",
                h3 { id: "{heading_id}", class: "kit-wfp__heading", {heading} }
                span { class: "kit-wfp__stat", {agents_text} }
                span { class: "kit-wfp__stat", {subs_text} }
                span { class: "kit-wfp__stat", "data-tone": "running", {running_text} }
                span { class: "kit-wfp__stat", "data-tone": "waiting", {needs_text} }
                span { class: "kit-wfp__spend", {spend_text} }
            }
            span { class: "kit-wfp__live", role: "status", "aria-live": "polite", {live} }
            div { class: "kit-wfp__body",
                ol { class: "kit-wfp__column", "aria-label": "Lead agents",
                    onkeydown: move |event: KeyboardEvent| {
                        let name = match event.key() {
                            Key::ArrowDown => "ArrowDown",
                            Key::ArrowUp => "ArrowUp",
                            Key::Home => "Home",
                            Key::End => "End",
                            _ => return,
                        };
                        if let Some(next) = next_lead(order_len, focused, name) {
                            event.prevent_default();
                            focus.set(next);
                            focus_by_id(&format!("{id}-lead-{next}"));
                        }
                    },
                    for (key, lead) in leads.clone().into_iter().map(|l| (l.index.to_string(), l)) {
                        li { key: "{key}",
                            button {
                                id: format!("{id}-lead-{}", lead.index),
                                class: "kit-wfp__lead",
                                r#type: "button",
                                "data-state": lead.state.as_str(),
                                "data-dimmed": lead.dimmed.then_some("true"),
                                "aria-expanded": if lead.expanded { "true" } else { "false" },
                                "aria-controls": lead.tree_id.clone(),
                                "aria-label": format!("{}, {}", lead.name, lead.state.word()),
                                tabindex: lead.tab,
                                onclick: {
                                    let target = lead.id.clone();
                                    move |_| {
                                        let next = if expanded().as_deref() == Some(target.as_str()) { None } else { Some(target.clone()) };
                                        on_expand.call(next);
                                    }
                                },
                                span { class: "kit-wfp__picture", "aria-hidden": "true",
                                    if let Some(src) = lead.picture.clone() {
                                        img { src: src, alt: "" }
                                    } else {
                                        {lead.initial.clone()}
                                    }
                                }
                                span { class: "kit-wfp__name", {lead.name.clone()} }
                                if let Some(model) = lead.model.clone() {
                                    span { class: "kit-wfp__model", {model} }
                                }
                                span { class: "kit-wfp__state", "data-state": lead.state.as_str(), {lead.state.word()} }
                                span { class: "kit-wfp__assignment", {lead.assignment.clone()} }
                                if !lead.tool_marks.is_empty() {
                                    span { class: "kit-wfp__tools",
                                        for mark in lead.tool_marks.clone() {
                                            span { class: "kit-wfp__tool", title: mark.caption.clone(),
                                                if let Some(src) = mark.mark_source.clone() {
                                                    img { src: src, alt: "" }
                                                }
                                            }
                                        }
                                    }
                                }
                                span { class: "kit-wfp__footer",
                                    span { class: "kit-wfp__phrase", {lead.phrase} }
                                    span { class: "kit-wfp__ring", style: format!("--share: {}", lead.ring), "aria-hidden": "true" }
                                    span { class: "kit-wfp__total", {lead.total.clone()} }
                                    span { class: "kit-wfp__elapsed", {lead.elapsed.clone()} }
                                }
                            }
                            if narrow && lead.expanded {
                                if let Some(open_agent) = open_lead.clone() {
                                    SubAgentTree {
                                        lead: open_agent,
                                        tree_id: format!("{id}-tree-{}", open.clone().unwrap_or_default()),
                                        card_id: format!("{id}-card"),
                                        work_open: work_open,
                                        on_pick: on_pick,
                                        on_respond: on_respond,
                                        id_prefix: id.clone(),
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(lead) = open_lead.filter(|_| !narrow) {
                    SubAgentTree {
                        lead: lead,
                        tree_id: format!("{id}-tree-{}", open.clone().unwrap_or_default()),
                        card_id: format!("{id}-card"),
                        work_open: work_open,
                        on_pick: on_pick,
                        on_respond: on_respond,
                        id_prefix: id.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn SubAgentTree(
    lead: LeadAgent,
    tree_id: String,
    card_id: String,
    work_open: Signal<Option<String>>,
    on_pick: Option<EventHandler<SubAgent>>,
    on_respond: Option<EventHandler<(SubAgent, RespondAction)>>,
    id_prefix: String,
) -> Element {
    let mut work_open = work_open;
    let total_spend = branch_spend(&lead);
    let subs = lead.sub_agents.clone();
    let lead_name = lead.name.clone();
    let spend_text = format!("${total_spend:.2}");
    let window_sub = work_open().and_then(|key| subs.iter().find(|s| s.id == key).cloned());
    let window_key = window_sub
        .as_ref()
        .map(|s| s.id.clone())
        .unwrap_or_default();
    let window_id = format!("{id_prefix}-window-{window_key}");
    let rows: Vec<(String, SubAgent, bool, String, Vec<RespondAction>)> = subs
        .iter()
        .map(|sub| {
            let has_window = !sub.work_items.is_empty() || !response_actions(sub.state).is_empty();
            let card_id = format!("{id_prefix}-sub-{}", sub.id);
            (
                sub.id.clone(),
                sub.clone(),
                has_window,
                card_id,
                response_actions(sub.state).to_vec(),
            )
        })
        .collect();
    let tree_label = format!("Sub-agents of {lead_name}");
    let window_items: Vec<WorkItem> = window_sub
        .as_ref()
        .map(|s| s.work_items.clone())
        .unwrap_or_default();
    let window_name = window_sub
        .as_ref()
        .map(|s| format!("Work of {}", s.name))
        .unwrap_or_default();
    let window_actions: Vec<(RespondAction, SubAgent)> = window_sub
        .as_ref()
        .map(|sub| {
            response_actions(sub.state)
                .iter()
                .map(|action| (*action, sub.clone()))
                .collect()
        })
        .unwrap_or_default();
    let close_card = format!("{id_prefix}-sub-{window_key}");

    rsx! {
        div { id: "{tree_id}", class: "kit-wfp__tree", role: "tree", "aria-label": "{tree_label}",
            div { class: "kit-wfp__tree-head", span { "Sub-agents · " } span { {spend_text} } }
            for (key, sub, has_window, card_id, actions) in rows {
                div { key: "{key}", class: "kit-wfp__sub", role: "treeitem", "data-state": sub.state.as_str(), "aria-label": format!("{}, {}", sub.name, sub.state.word()),
                    button {
                        id: "{card_id}",
                        class: "kit-wfp__sub-card",
                        r#type: "button",
                        "aria-haspopup": has_window.then_some("dialog"),
                        "aria-expanded": has_window.then_some(if work_open().as_deref() == Some(sub.id.as_str()) { "true" } else { "false" }),
                        onclick: {
                            let target = sub.clone();
                            move |_| {
                                if target.work_items.is_empty() && response_actions(target.state).is_empty() {
                                    if let Some(handler) = &on_pick {
                                        handler.call(target.clone());
                                    }
                                } else {
                                    let next = if work_open().as_deref() == Some(target.id.as_str()) { None } else { Some(target.id.clone()) };
                                    work_open.set(next);
                                }
                            }
                        },
                        span { class: "kit-wfp__sub-name", {sub.name.clone()} }
                        span { class: "kit-wfp__sub-state", {sub.state.word()} }
                        span { class: "kit-wfp__sub-assignment", {sub.assignment.clone()} }
                        span { class: "kit-wfp__sub-footer",
                            span { class: "kit-wfp__sub-spend", {format!("${:.2}", sub.spend.unwrap_or(0.0))} }
                        }
                    }
                    if !actions.is_empty() && work_open().is_none() {
                        span { class: "kit-wfp__sub-actions", "aria-hidden": "true", {actions.iter().map(|a| a.label()).collect::<Vec<_>>().join(" · ")} }
                    }
                }
            }
            if let Some(sub) = window_sub {
                div {
                    id: "{window_id}",
                    class: "kit-wfp__window",
                    role: "dialog",
                    "aria-label": "{window_name}",
                    tabindex: "-1",
                    onmounted: move |event: MountedEvent| {
                        let node = event.data();
                        spawn(async move {
                            let _ = node.set_focus(true).await;
                        });
                    },
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            event.prevent_default();
                            work_open.set(None);
                            focus_after_frame(&close_card);
                        }
                    },
                    ul { class: "kit-wfp__items",
                        for (index, item) in window_items.into_iter().enumerate() {
                            li { key: "{index}", class: "kit-wfp__item", "data-state": item.state.map_or("done", WorkState::as_str),
                                span { class: "kit-wfp__item-label", {item.label.clone()} }
                                if let Some(tool) = item.tool_name.clone() {
                                    span { class: "kit-wfp__item-tool", {tool} }
                                }
                                span { class: "kit-wfp__item-time", {item.duration.map(format_mm_ss).unwrap_or_default()} }
                            }
                        }
                    }
                    if let Some(question) = sub.question.clone().or(sub.failure_text.clone()) {
                        p { class: "kit-wfp__question", {question} }
                    }
                    div { class: "kit-wfp__actions",
                        for (action, target) in window_actions {
                            button {
                                key: "{action.label()}",
                                class: "kit-wfp__respond",
                                r#type: "button",
                                "data-action": action.label(),
                                onclick: move |_| {
                                    if let Some(handler) = &on_respond {
                                        handler.call((target.clone(), action));
                                    }
                                },
                                {action.label()}
                            }
                        }
                    }
                }
            }
        }
    }
}
