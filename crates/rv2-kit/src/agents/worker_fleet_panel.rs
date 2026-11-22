//! Worker fleet panel: several top-level agents in a column of cards. Each card condenses
//! its sub-agents into one line and a progress ring; opening a card unfolds its sub-agent
//! tree beside it, and a sub-agent's work window lists its items.
//!
//! Design choice: the open card is seeded from `expanded_id` and then kept internally;
//! `on_expand_change` reports each change. This lets a card always fold.
//! Design choice: the open tree is rendered under its card and CSS moves it beside the
//! column when the container is wide (container query), so the bus starts at its lead.
//! Design choice: spend is summed here from each branch; callers pass per-agent spend only.
//! Design choice: response actions are offered only when `on_respond` is supplied.

use dioxus::prelude::*;

use crate::agents::delegation_card_tree::accent_hue;
use crate::agents::single_run_status_card::{focus_by_id, format_mm_ss, initial_of};
use crate::internal::{KIT_CSS, focus_after_frame, next_id, use_now_ms};

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

    fn word(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Waiting => "Needs you",
            Self::Failed => "Failed",
            Self::Queued => "Queued",
            Self::Skipped => "Skipped",
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
    /// Sub-agents this one delegated to. Drawn one step in from it.
    pub children: Vec<SubAgent>,
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

/// Every sub-agent in `subs`, nested ones included, parents before their children.
fn all_subs(subs: &[SubAgent]) -> Vec<&SubAgent> {
    subs.iter()
        .flat_map(|sub| std::iter::once(sub).chain(all_subs(&sub.children)))
        .collect()
}

/// Spend of a lead's sub-agents, nested ones included.
pub fn branch_spend(lead: &LeadAgent) -> f64 {
    all_subs(&lead.sub_agents)
        .iter()
        .filter_map(|s| s.spend)
        .sum()
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
        for sub in all_subs(&lead.sub_agents) {
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
    let all = all_subs(subs);
    [
        (FleetState::Waiting, "Needs you"),
        (FleetState::Failed, "Failed"),
        (FleetState::Running, "Running"),
        (FleetState::Done, "Done"),
    ]
    .into_iter()
    .find(|(state, _)| all.iter().any(|s| s.state == *state))
    .map(|(_, phrase)| phrase)
}

/// Finished sub-agents and the total: the progress ring's share.
pub fn done_share(subs: &[SubAgent]) -> (usize, usize) {
    let all = all_subs(subs);
    (
        all.iter().filter(|s| s.state == FleetState::Done).count(),
        all.len(),
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

fn offered_actions(state: FleetState, can_respond: bool) -> &'static [RespondAction] {
    if can_respond {
        response_actions(state)
    } else {
        &[]
    }
}

/// The step a sub-agent is on: its running work item, if any.
pub fn current_step(sub: &SubAgent) -> Option<&str> {
    sub.work_items
        .iter()
        .find(|item| item.state == Some(WorkState::Running))
        .map(|item| item.label.as_str())
}

/// Time its work took, summed, shown in the footer when no step is running.
pub fn elapsed_of(sub: &SubAgent) -> Option<u64> {
    if current_step(sub).is_some() {
        return None;
    }
    let durations: Vec<u64> = sub
        .work_items
        .iter()
        .filter_map(|item| item.duration)
        .collect();
    (!durations.is_empty()).then(|| durations.iter().sum())
}

/// Word for a work item's state. An item without a state counts as done.
pub fn item_word(state: Option<WorkState>) -> &'static str {
    state.map_or("Done", WorkState::word)
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
    /// Sub-agent id whose work window is open on first render.
    #[props(default)]
    default_work_open: Option<String>,
    #[props(default)] clock_now_ms: Option<u64>,
    #[props(default)] format_spend: Option<Callback<f64, String>>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-wfp-{}", next_id()));
    let now = use_now_ms(clock_now_ms, 1000);
    let mut expanded = use_signal(|| expanded_id.clone());
    let work_open = use_signal(|| default_work_open.clone());
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
                        li { key: "{key}", class: "kit-wfp__slot",
                            button {
                                id: format!("{id}-lead-{}", lead.index),
                                class: "kit-wfp__lead",
                                r#type: "button",
                                style: format!("--kit-wfp-hue: {}", accent_hue(&lead.id)),
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
                            if lead.expanded {
                                if let Some(open_agent) = open_lead.clone() {
                                    SubAgentTree {
                                        lead: open_agent,
                                        tree_id: lead.tree_id.clone(),
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
            }
        }
    }
}

#[component]
fn SubAgentTree(
    lead: LeadAgent,
    tree_id: String,
    work_open: Signal<Option<String>>,
    on_pick: Option<EventHandler<SubAgent>>,
    on_respond: Option<EventHandler<(SubAgent, RespondAction)>>,
    id_prefix: String,
) -> Element {
    let tree_label = format!("Sub-agents of {}", lead.name);
    rsx! {
        div { id: "{tree_id}", class: "kit-wfp__tree", role: "tree", "aria-label": "{tree_label}",
            for sub in lead.sub_agents.clone() {
                SubAgentNode {
                    key: "{sub.id}",
                    sub: sub,
                    work_open: work_open,
                    on_pick: on_pick,
                    on_respond: on_respond,
                    id_prefix: id_prefix.clone(),
                    nested: false,
                }
            }
        }
    }
}

/// One sub-agent: a rail cell that carries its connector, then its card and any
/// children. Rail and card are siblings in the parent's grid, so the rail runs
/// the full height of the card and of the children beneath it.
#[component]
fn SubAgentNode(
    sub: SubAgent,
    work_open: Signal<Option<String>>,
    on_pick: Option<EventHandler<SubAgent>>,
    on_respond: Option<EventHandler<(SubAgent, RespondAction)>>,
    id_prefix: String,
    nested: bool,
) -> Element {
    let mut work_open = work_open;
    let can_respond = on_respond.is_some();
    let actions = offered_actions(sub.state, can_respond);
    let has_window = !sub.work_items.is_empty() || !actions.is_empty();
    let window_open = work_open().as_deref() == Some(sub.id.as_str());
    let open_id = format!("{id_prefix}-sub-{}", sub.id);
    let window_id = format!("{id_prefix}-window-{}", sub.id);
    let step = current_step(&sub).map(str::to_string);
    let elapsed = elapsed_of(&sub);
    let class = if nested {
        "kit-wfp__sub kit-wfp__sub--nested"
    } else {
        "kit-wfp__sub"
    };
    let actions = actions.to_vec();
    let target = sub.clone();
    let name = format!("{}, {}", sub.name, sub.state.word());
    let hue = format!("--kit-wfp-hue: {}", accent_hue(&sub.id));
    rsx! {
        span { class: "kit-wfp__rail", "aria-hidden": "true" }
        div { class: class, role: "treeitem", "data-state": sub.state.as_str(), "aria-label": "{name}", style: "{hue}",
            div { class: "kit-wfp__sub-card",
                button {
                    id: "{open_id}",
                    class: "kit-wfp__sub-open",
                    r#type: "button",
                    "aria-haspopup": has_window.then_some("dialog"),
                    "aria-expanded": has_window.then_some(if window_open { "true" } else { "false" }),
                    onclick: {
                        let target = target.clone();
                        move |_| {
                            if !has_window {
                                if let Some(handler) = &on_pick {
                                    handler.call(target.clone());
                                }
                            } else {
                                let next = if work_open().as_deref() == Some(target.id.as_str()) { None } else { Some(target.id.clone()) };
                                work_open.set(next);
                            }
                        }
                    },
                    span { class: "kit-wfp__sub-mark", "aria-hidden": "true", {initial_of(&sub.name)} }
                    span { class: "kit-wfp__sub-name", {sub.name.clone()} }
                    span { class: "kit-wfp__sub-state", "data-state": sub.state.as_str(), {sub.state.word()} }
                    span { class: "kit-wfp__sub-assignment", {sub.assignment.clone()} }
                }
                div { class: "kit-wfp__sub-footer",
                    span { class: "kit-wfp__sub-spend", {format!("${:.2}", sub.spend.unwrap_or(0.0))} }
                    if let Some(step) = step {
                        span { class: "kit-wfp__sub-step", {step} }
                    }
                    if let Some(ms) = elapsed {
                        span { class: "kit-wfp__elapsed", {format_mm_ss(ms)} }
                    }
                    if !window_open {
                        for action in actions {
                            button {
                                key: "{action.label()}",
                                class: "kit-wfp__respond",
                                r#type: "button",
                                "data-action": action.label(),
                                onclick: {
                                    let target = target.clone();
                                    move |_| {
                                        if let Some(handler) = &on_respond {
                                            handler.call((target.clone(), action));
                                        }
                                    }
                                },
                                {action.label()}
                            }
                        }
                    }
                }
            }
            if window_open {
                WorkWindow {
                    sub: sub.clone(),
                    window_id: window_id,
                    close_card: open_id.clone(),
                    work_open: work_open,
                    on_respond: on_respond,
                    can_respond: can_respond,
                }
            }
            if !sub.children.is_empty() {
                div { class: "kit-wfp__children", role: "group",
                    for child in sub.children.clone() {
                        SubAgentNode {
                            key: "{child.id}",
                            sub: child,
                            work_open: work_open,
                            on_pick: on_pick,
                            on_respond: on_respond,
                            id_prefix: id_prefix.clone(),
                            nested: true,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn WorkWindow(
    sub: SubAgent,
    window_id: String,
    close_card: String,
    work_open: Signal<Option<String>>,
    on_respond: Option<EventHandler<(SubAgent, RespondAction)>>,
    can_respond: bool,
) -> Element {
    let mut work_open = work_open;
    let window_name = format!("Work of {}", sub.name);
    let actions = offered_actions(sub.state, can_respond);
    let question = sub.question.clone().or(sub.failure_text.clone());
    rsx! {
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
            if let Some(question) = question {
                p { class: "kit-wfp__question", {question} }
            }
            if !actions.is_empty() {
                div { class: "kit-wfp__actions",
                    for action in actions.iter().copied() {
                        button {
                            key: "{action.label()}",
                            class: "kit-wfp__respond",
                            r#type: "button",
                            "data-action": action.label(),
                            onclick: {
                                let target = sub.clone();
                                move |_| {
                                    if let Some(handler) = &on_respond {
                                        handler.call((target.clone(), action));
                                    }
                                }
                            },
                            {action.label()}
                        }
                    }
                }
            }
            ul { class: "kit-wfp__items",
                for (index, item) in sub.work_items.iter().enumerate() {
                    li { key: "{index}", class: "kit-wfp__item", "data-state": item.state.map_or("done", WorkState::as_str),
                        span { class: "kit-wfp__item-label", {item.label.clone()} }
                        if let Some(tool) = item.tool_name.clone() {
                            span { class: "kit-wfp__item-tool", {tool} }
                        }
                        span { class: "kit-wfp__item-state", {item_word(item.state)} }
                        span { class: "kit-wfp__item-time", {item.duration.map(format_mm_ss).unwrap_or_default()} }
                    }
                }
            }
        }
    }
}
