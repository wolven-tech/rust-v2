//! Agent fleet overview: every agent on one screen, grouped under its parent, with the
//! inbox of what needs the person and one status filter that narrows all three views.
//!
//! Design choice: the activity curve is the running total of events since `last_seen_at`;
//! each step is one event, and the filter removes the events of agents it hides.
//! Design choice: a child whose parent the filter hides moves to the top level, so a
//! matching agent never disappears from the fleet.
//! Design choice: the child count counts every child, filter or not; it describes the
//! agent, not the view.
//! Design choice: the focus fade is a CSS opacity transition on each card and item; reduced
//! motion removes it, so the change applies in one step.
//! Browser checks: filter narrowing, focus dimming, dismiss confirmation, Left/Right on the
//! filter, Escape clears focus, phone stacking with no sideways scroll.

use dioxus::prelude::*;
use rv2_affect::badge::{Badge, BadgeTone};
use rv2_affect::segmented_control::{SegmentOption, SegmentedControl};

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/agent-fleet-overview.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentStatus {
    Running,
    Waiting,
    Failed,
    Done,
}

impl AgentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Failed => "failed",
            Self::Done => "done",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Waiting => "Waiting",
            Self::Failed => "Failed",
            Self::Done => "Done",
        }
    }

    fn tone(self) -> BadgeTone {
        match self {
            Self::Running => BadgeTone::Info,
            Self::Waiting => BadgeTone::Warning,
            Self::Failed => BadgeTone::Danger,
            Self::Done => BadgeTone::Success,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatusFilter {
    #[default]
    All,
    Running,
    Waiting,
    Failed,
}

impl StatusFilter {
    pub const ALL: [StatusFilter; 4] = [Self::All, Self::Running, Self::Waiting, Self::Failed];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.as_str() == value)
    }

    pub fn word(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Running => "Running",
            Self::Waiting => "Needs you",
            Self::Failed => "Failed",
        }
    }

    pub fn matches(self, status: AgentStatus) -> bool {
        match self {
            Self::All => true,
            Self::Running => status == AgentStatus::Running,
            Self::Waiting => status == AgentStatus::Waiting,
            Self::Failed => status == AgentStatus::Failed,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FleetAgent {
    pub id: String,
    pub name: String,
    pub model_label: String,
    pub parent_id: Option<String>,
    pub status: AgentStatus,
    pub cost_usd: f64,
    pub elapsed_secs: u64,
    pub current_task: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InboxKind {
    Decision,
    Failure,
    Review,
}

impl InboxKind {
    pub fn word(self) -> &'static str {
        match self {
            Self::Decision => "Decision",
            Self::Failure => "Failure",
            Self::Review => "Review",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxAction {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxItem {
    pub id: String,
    pub agent_id: String,
    pub kind: InboxKind,
    pub title: String,
    pub detail: String,
    pub waiting_secs: u64,
    pub actions: Vec<InboxAction>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Done,
    NeedsYou,
    Failed,
}

impl Outcome {
    pub fn word(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::NeedsYou => "needs you",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivityPoint {
    pub at: u64,
    pub agent_id: String,
    pub outcome: Outcome,
}

/// Agents per filter, in `StatusFilter::ALL` order.
pub fn status_counts(agents: &[FleetAgent]) -> [usize; 4] {
    StatusFilter::ALL.map(|f| agents.iter().filter(|a| f.matches(a.status)).count())
}

/// How many agents name `id` as their parent, whatever the filter shows.
pub fn child_count(agents: &[FleetAgent], id: &str) -> usize {
    agents
        .iter()
        .filter(|a| a.parent_id.as_deref() == Some(id))
        .count()
}

fn agent_of<'a>(agents: &'a [FleetAgent], id: &str) -> Option<&'a FleetAgent> {
    agents.iter().find(|a| a.id == id)
}

/// An inbox item shows when its agent passes the filter; an item for an unknown agent
/// shows only under `All`.
pub fn inbox_item_visible(item: &InboxItem, agents: &[FleetAgent], filter: StatusFilter) -> bool {
    match agent_of(agents, &item.agent_id) {
        Some(agent) => filter.matches(agent.status),
        None => filter == StatusFilter::All,
    }
}

/// Counts of (done, needs you, failed) among events whose agent passes the filter.
pub fn activity_summary(
    points: &[ActivityPoint],
    agents: &[FleetAgent],
    filter: StatusFilter,
) -> (usize, usize, usize) {
    points
        .iter()
        .filter(|p| agent_of(agents, &p.agent_id).is_some_and(|a| filter.matches(a.status)))
        .fold((0, 0, 0), |(d, n, f), p| match p.outcome {
            Outcome::Done => (d + 1, n, f),
            Outcome::NeedsYou => (d, n + 1, f),
            Outcome::Failed => (d, n, f + 1),
        })
}

/// Running total of the events that pass the filter, as `(x in 0..=1, total)` pairs in
/// time order. Events outside `from..=to` are clamped to the window edges.
pub fn activity_curve(
    points: &[ActivityPoint],
    agents: &[FleetAgent],
    filter: StatusFilter,
    from: u64,
    to: u64,
) -> Vec<(f64, usize)> {
    let mut kept: Vec<&ActivityPoint> = points
        .iter()
        .filter(|p| agent_of(agents, &p.agent_id).is_some_and(|a| filter.matches(a.status)))
        .collect();
    kept.sort_by_key(|p| p.at);
    let span = to.saturating_sub(from).max(1) as f64;
    kept.iter()
        .enumerate()
        .map(|(i, p)| {
            let x = (p.at.saturating_sub(from) as f64 / span).clamp(0.0, 1.0);
            (x, i + 1)
        })
        .collect()
}

/// Event markers of the activity curve, as `(x in 0..=1, total, outcome)` in time order: the
/// same events and positions as `activity_curve`, each with its outcome.
pub fn activity_markers(
    points: &[ActivityPoint],
    agents: &[FleetAgent],
    filter: StatusFilter,
    from: u64,
    to: u64,
) -> Vec<(f64, usize, Outcome)> {
    let mut kept: Vec<&ActivityPoint> = points
        .iter()
        .filter(|p| agent_of(agents, &p.agent_id).is_some_and(|a| filter.matches(a.status)))
        .collect();
    kept.sort_by_key(|p| p.at);
    let span = to.saturating_sub(from).max(1) as f64;
    kept.iter()
        .enumerate()
        .map(|(i, p)| {
            let x = (p.at.saturating_sub(from) as f64 / span).clamp(0.0, 1.0);
            (x, i + 1, p.outcome)
        })
        .collect()
}

/// `45 s`, `12 min`, `3 h`: how long something has been waiting.
pub fn wait_label(secs: u64) -> String {
    match secs {
        0..=59 => format!("{secs} s"),
        60..=3599 => format!("{} min", secs / 60),
        _ => format!("{} h", secs / 3600),
    }
}

/// The agents a branch shows: those `parent` names among `visible`, or, for `None`, those
/// whose parent is not visible.
fn branch<'a>(visible: &[&'a FleetAgent], parent: Option<&str>) -> Vec<&'a FleetAgent> {
    visible
        .iter()
        .copied()
        .filter(|a| match parent {
            None => !a
                .parent_id
                .as_deref()
                .is_some_and(|p| visible.iter().any(|v| v.id == p)),
            Some(p) => a.parent_id.as_deref() == Some(p),
        })
        .collect()
}

#[component]
pub fn AgentFleetOverview(
    #[props(default = "Agent fleet".to_string(), into)] heading_text: String,
    agents: Vec<FleetAgent>,
    inbox_items: Vec<InboxItem>,
    activity: Vec<ActivityPoint>,
    last_seen_at: u64,
    now_at: u64,
    #[props(default)] status_filter: StatusFilter,
    #[props(default)] on_status_filter_change: Option<EventHandler<StatusFilter>>,
    #[props(default)] spotlight_agent_id: Option<String>,
    #[props(default)] on_spotlight_change: Option<EventHandler<Option<String>>>,
    #[props(default)] on_inbox_answer: Option<EventHandler<(String, String)>>,
    #[props(default)] on_clear_failed: Option<EventHandler<String>>,
    /// Such as `Since 9:10 AM · 2h`.
    #[props(default, into)]
    since_label: String,
    /// Text at the start and the end of the activity curve.
    #[props(default)]
    range_labels: (String, String),
    #[props(default, into)] extra_classes: String,
) -> Element {
    let mut expanded_item = use_signal(|| None::<String>);
    let confirm_clear = use_signal(|| None::<String>);
    let mut answering = use_signal(|| None::<(String, String)>);

    let counts = status_counts(&agents);
    let visible: Vec<&FleetAgent> = agents
        .iter()
        .filter(|a| status_filter.matches(a.status))
        .collect();
    let inbox: Vec<&InboxItem> = inbox_items
        .iter()
        .filter(|i| inbox_item_visible(i, &agents, status_filter))
        .collect();
    let (done, needs, failed) = activity_summary(&activity, &agents, status_filter);
    let curve = activity_curve(&activity, &agents, status_filter, last_seen_at, now_at);
    let polyline = curve
        .iter()
        .map(|(x, total)| {
            format!(
                "{:.1},{:.1}",
                10.0 + x * 580.0,
                70.0 - (*total).min(10) as f64 * 6.0
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let markers: Vec<(f64, f64, &'static str)> =
        activity_markers(&activity, &agents, status_filter, last_seen_at, now_at)
            .into_iter()
            .map(|(x, total, outcome)| {
                (
                    (10.0 + x * 580.0) / 600.0 * 100.0,
                    (70.0 - total.min(10) as f64 * 6.0) / 80.0 * 100.0,
                    outcome.word(),
                )
            })
            .collect();
    let leads = agents.iter().filter(|a| a.parent_id.is_none()).count();
    let subs = agents.len() - leads;
    let running = agents
        .iter()
        .filter(|a| a.status == AgentStatus::Running)
        .count();
    let waiting = agents
        .iter()
        .filter(|a| a.status == AgentStatus::Waiting)
        .count();
    let spend: f64 = agents.iter().map(|a| a.cost_usd).sum();
    let fleet_summary = format!(
        "{leads} leads · {subs} sub-agents · {running} running · {waiting} waiting on you · ${spend:.2}"
    );
    let spotlight = spotlight_agent_id.clone();
    let has_spotlight = spotlight.is_some();
    let roots = branch(&visible, None);
    let root_class = class_list("kit-fleet", &extra_classes);

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-labelledby": "kit-fleet-title",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape && has_spotlight {
                    if let Some(handler) = on_spotlight_change {
                        handler.call(None);
                    }
                }
            },
            header { class: "kit-fleet__head",
                h2 { id: "kit-fleet-title", class: "kit-fleet__title", "{heading_text}" }
                SegmentedControl {
                    aria_label: "Agents by status",
                    value: status_filter.as_str().to_string(),
                    on_change: move |value: String| {
                        if let (Some(filter), Some(handler)) = (StatusFilter::parse(&value), on_status_filter_change) {
                            handler.call(filter);
                        }
                    },
                    options: StatusFilter::ALL
                        .iter()
                        .enumerate()
                        .map(|(i, f)| SegmentOption {
                            value: f.as_str().to_string(),
                            label: Some(rsx! { "{f.word()} {counts[i]}" }),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>(),
                }
            }
            section { class: "kit-fleet__summary", "aria-label": "Since you last looked",
                header { class: "kit-fleet__summary-head",
                    h3 { class: "kit-fleet__summary-title", "Since you looked" }
                    if !since_label.is_empty() {
                        span { class: "kit-fleet__since", "{since_label}" }
                    }
                }
                div { class: "kit-fleet__curve-wrap",
                    svg { class: "kit-fleet__curve", "viewBox": "0 0 600 80", "preserveAspectRatio": "none", "aria-hidden": "true",
                        polyline { class: "kit-fleet__curve-line", points: "{polyline}" }
                    }
                    for (i, (left, top, outcome)) in markers.iter().enumerate() {
                        span { key: "{i}", class: "kit-fleet__marker", "data-outcome": "{outcome}", "aria-hidden": "true", style: "left: {left:.2}%; top: {top:.2}%" }
                    }
                    div { class: "kit-fleet__curve-ends", "aria-hidden": "true",
                        span { "{range_labels.0}" }
                        span { "{range_labels.1}" }
                    }
                }
                p { class: "kit-fleet__tallies", "aria-live": "polite",
                    span { "{done} done" }
                    span { "{needs} needs you" }
                    span { "{failed} failed" }
                }
            }
            div { class: "kit-fleet__body",
                div { class: "kit-fleet__cards",
                    header { class: "kit-fleet__cards-head",
                        h3 { class: "kit-fleet__cards-title", "Agents" }
                        p { class: "kit-fleet__cards-summary", "{fleet_summary}" }
                    }
                    if visible.is_empty() {
                        p { class: "kit-fleet__empty", "No agents match this filter." }
                    } else {
                        ul { class: "kit-fleet__tree", role: "tree", "aria-label": "Agents",
                            for agent in roots.iter().copied() {
                                {fleet_branch(agent, &agents, &visible, spotlight.as_deref(), confirm_clear, on_spotlight_change, on_clear_failed)}
                            }
                        }
                    }
                }
                aside { class: "kit-fleet__inbox", "aria-label": "Inbox",
                    h3 { class: "kit-fleet__inbox-title", "Inbox " span { class: "kit-fleet__count", "{inbox.len()}" } }
                    if inbox.is_empty() {
                        p { class: "kit-fleet__empty", "Nothing needs you right now." }
                    }
                    for item in inbox.iter().copied() {
                        {
                            let item_id = item.id.clone();
                            let agent_id = item.agent_id.clone();
                            let dimmed = spotlight.as_deref().is_some_and(|s| s != agent_id);
                            let focused = spotlight.as_deref() == Some(agent_id.as_str());
                            let open = expanded_item() == Some(item_id.clone());
                            let busy = answering().filter(|(i, _)| *i == item_id).map(|(_, a)| a);
                            rsx! {
                                article { key: "{item_id}", class: "kit-fleet__item", "data-dimmed": dimmed.then_some("true"),
                                    "data-focused": focused.then_some("true"),
                                    button {
                                        r#type: "button",
                                        class: "kit-fleet__item-head",
                                        "aria-expanded": if open { "true" } else { "false" },
                                        onclick: {
                                            let item_id = item_id.clone();
                                            move |_| {
                                                let next = if expanded_item() == Some(item_id.clone()) { None } else { Some(item_id.clone()) };
                                                expanded_item.set(next);
                                            }
                                        },
                                        span { class: "kit-fleet__kind", "{item.kind.word()}" }
                                        span { class: "kit-fleet__item-title", "{item.title}" }
                                        span { class: "kit-fleet__wait", "waiting {wait_label(item.waiting_secs)}" }
                                    }
                                    if open {
                                        div { class: "kit-fleet__item-body",
                                            p { class: "kit-fleet__detail", "{item.detail}" }
                                            div { class: "kit-fleet__actions",
                                                for action in item.actions.iter() {
                                                    {
                                                        let action_id = action.id.clone();
                                                        let is_busy = busy.as_deref() == Some(action.id.as_str());
                                                        let item_id = item_id.clone();
                                                        let label = action.label.clone();
                                                        rsx! {
                                                            button {
                                                                key: "{action.id}",
                                                                r#type: "button",
                                                                class: "kit-fleet__action",
                                                                "aria-busy": is_busy.then_some("true"),
                                                                disabled: busy.is_some(),
                                                                onclick: move |_| {
                                                                    answering.set(Some((item_id.clone(), action_id.clone())));
                                                                    if let Some(handler) = on_inbox_answer {
                                                                        handler.call((item_id.clone(), action_id.clone()));
                                                                    }
                                                                },
                                                                if is_busy { "Sending…" } else { "{label}" }
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
                }
            }
        }
    }
}

/// One agent card, and under it the visible children of that agent.
fn fleet_branch(
    agent: &FleetAgent,
    all: &[FleetAgent],
    visible: &[&FleetAgent],
    spotlight: Option<&str>,
    mut confirm_clear: Signal<Option<String>>,
    on_spotlight_change: Option<EventHandler<Option<String>>>,
    on_clear_failed: Option<EventHandler<String>>,
) -> Element {
    let id = agent.id.clone();
    let kids = branch(visible, Some(&agent.id));
    let focused = spotlight == Some(agent.id.as_str());
    let dimmed = spotlight.is_some() && !focused;
    let total_children = child_count(all, &agent.id);
    let child_word = if total_children == 1 {
        "child agent"
    } else {
        "child agents"
    };
    let confirming = confirm_clear() == Some(id.clone());
    let pick_id = id.clone();
    let clear_id = id.clone();
    let dismiss_id = id.clone();
    let elapsed = wait_label(agent.elapsed_secs);
    let cost = format!("${:.2}", agent.cost_usd);
    rsx! {
        li { key: "{id}", role: "treeitem", "aria-label": "{agent.name}",
            "aria-expanded": (!kids.is_empty()).then_some("true"),
            div { class: "kit-fleet__card", "data-status": agent.status.as_str(),
                "data-focused": focused.then_some("true"),
                "data-dimmed": dimmed.then_some("true"),
                button {
                    r#type: "button",
                    class: "kit-fleet__card-pick",
                    "aria-pressed": if focused { "true" } else { "false" },
                    onclick: move |_| {
                        if let Some(handler) = on_spotlight_change {
                            handler.call(if focused { None } else { Some(pick_id.clone()) });
                        }
                    },
                    span { class: "kit-fleet__name", "{agent.name}" }
                    span { class: "kit-fleet__model", "{agent.model_label}" }
                    Badge { tone: agent.status.tone(), "{agent.status.label()}" }
                    span { class: "kit-fleet__task", "{agent.current_task}" }
                    span { class: "kit-fleet__meta", "{elapsed} · {cost}" }
                    if total_children > 0 {
                        span { class: "kit-fleet__children", "{total_children} {child_word}" }
                    }
                }
                if agent.status == AgentStatus::Failed {
                    if confirming {
                        div { class: "kit-fleet__confirm", role: "group", "aria-label": "Confirm clear",
                            span { "Clear {agent.name} from the fleet?" }
                            button {
                                r#type: "button",
                                class: "kit-fleet__action",
                                onclick: move |_| {
                                    confirm_clear.set(None);
                                    if let Some(handler) = on_clear_failed {
                                        handler.call(clear_id.clone());
                                    }
                                },
                                "Clear"
                            }
                            button {
                                r#type: "button",
                                class: "kit-fleet__action",
                                onclick: move |_| confirm_clear.set(None),
                                "Keep"
                            }
                        }
                    } else {
                        button {
                            r#type: "button",
                            class: "kit-fleet__dismiss",
                            onclick: move |_| confirm_clear.set(Some(dismiss_id.clone())),
                            "Dismiss"
                        }
                    }
                }
            }
            if !kids.is_empty() {
                ul { class: "kit-fleet__tree kit-fleet__tree--nested", role: "group",
                    for child in kids.iter().copied() {
                        {fleet_branch(child, all, visible, spotlight, confirm_clear, on_spotlight_change, on_clear_failed)}
                    }
                }
            }
        }
    }
}
