//! Agent timeline overview: every agent on one shared clock. One lane per agent, spans as
//! bars on the lane, an inbox of decisions, and one status filter for all three.
//!
//! Design choice: the filter hides spans, never lanes, so a filtered lane keeps its row.
//! Design choice: a running span's end is the `now_ms` marker; the host moves `now_ms`,
//! and the bar grows with it.
//! Design choice: the summary counts spans and inbox items that changed since
//! `last_seen_at`; the spec's activity feed is derived from them.
//! Design choice: the time axis pans by drag or wheel in whole minutes and reports each new
//! window through `on_window_change`; the host owns the window itself.
//! Browser checks: running bar stops at the marker, filter keeps lane rows, focus fades lanes,
//! hover detail, axis drag and wheel, Left/Right/Up/Down span navigation, phone lanes as a list.

use dioxus::prelude::*;
use rv2_affect::badge::{Badge, BadgeTone};
use rv2_affect::segmented_control::{SegmentOption, SegmentedControl};

use crate::blocks::agent_fleet_overview::{
    AgentStatus, InboxItem, InboxKind, StatusFilter, wait_label,
};
use crate::internal::{KIT_CSS, class_list, use_element_width};

/// Moves a window by `delta_ms`, snapped to whole minutes and kept at its length.
pub fn pan_window(from: u64, to: u64, delta_ms: i64) -> (u64, u64) {
    const MINUTE: i64 = 60_000;
    let shift = delta_ms / MINUTE * MINUTE;
    let new_from = (from as i64 + shift).max(0) as u64;
    (new_from, new_from + to.saturating_sub(from))
}

const CSS: Asset = asset!("/assets/agent-timeline-overview.css");

/// Intervals on the time axis.
const TICKS: u64 = 6;

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineAgent {
    pub id: String,
    pub name: String,
    pub model_label: String,
    pub status: AgentStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimelineSpan {
    pub id: String,
    pub agent_id: String,
    pub label: String,
    pub start_at: u64,
    /// `None` while the span is still running.
    pub end_at: Option<u64>,
    pub status: AgentStatus,
    /// Error line shown on hover when the span failed.
    pub error_line: Option<String>,
}

/// Left and width of a span as percentages of the window. Spans that run past either edge
/// are clipped; a span wholly outside the window has zero width.
pub fn span_geometry(start: u64, end: u64, from: u64, to: u64) -> (f64, f64) {
    let span = to.saturating_sub(from).max(1) as f64;
    let s = start.clamp(from, to);
    let e = end.clamp(from, to).max(s);
    let left = (s - from) as f64 / span * 100.0;
    let width = (e - s) as f64 / span * 100.0;
    (left, width)
}

/// End of a span for drawing: its own end, or `now` while it runs.
pub fn effective_end(span: &TimelineSpan, now: u64) -> u64 {
    span.end_at.unwrap_or(now).max(span.start_at)
}

/// Minutes before `now`, as the axis label: `now` at the marker, `12 min ago` before it.
pub fn axis_label(t: u64, now: u64) -> String {
    let mins = now.saturating_sub(t) / 60_000;
    if mins == 0 {
        "now".to_string()
    } else {
        format!("{mins} min ago")
    }
}

/// Spans of an agent in time order.
pub fn lane_spans(spans: &[TimelineSpan], agent_id: &str) -> Vec<TimelineSpan> {
    let mut lane: Vec<TimelineSpan> = spans
        .iter()
        .filter(|s| s.agent_id == agent_id)
        .cloned()
        .collect();
    lane.sort_by_key(|s| s.start_at);
    lane
}

/// Counts of (done, needs you, failed) since `last_seen_at`: spans that finished or
/// failed, and inbox items, which all wait on a person.
pub fn since_last_seen(
    spans: &[TimelineSpan],
    inbox: &[InboxItem],
    last_seen_at: u64,
) -> (usize, usize, usize) {
    let done = spans
        .iter()
        .filter(|s| s.status == AgentStatus::Done && s.end_at.is_some_and(|e| e >= last_seen_at))
        .count();
    let failed = spans
        .iter()
        .filter(|s| s.status == AgentStatus::Failed && s.end_at.is_some_and(|e| e >= last_seen_at))
        .count();
    (done, inbox.len(), failed)
}

fn focus_element(target: &str) {
    let script = match target.strip_prefix("lane:") {
        Some(agent) => format!(
            "document.querySelector('[data-lane=\"{agent}\"] .kit-tl__lane-label')?.focus();"
        ),
        None => format!("document.getElementById('kit-tl-span-{target}')?.focus();"),
    };
    document::eval(&script);
}

#[component]
pub fn AgentTimelineOverview(
    #[props(default = "Timeline".to_string(), into)] heading_text: String,
    agents: Vec<TimelineAgent>,
    spans: Vec<TimelineSpan>,
    inbox_items: Vec<InboxItem>,
    window_from: u64,
    window_to: u64,
    now_ms: u64,
    last_seen_at: u64,
    #[props(default)] status_filter: StatusFilter,
    #[props(default)] on_status_filter_change: Option<EventHandler<StatusFilter>>,
    #[props(default)] spotlight_agent_id: Option<String>,
    #[props(default)] on_spotlight_change: Option<EventHandler<Option<String>>>,
    #[props(default)] on_inbox_answer: Option<EventHandler<(String, String)>>,
    #[props(default)] on_window_change: Option<EventHandler<(u64, u64)>>,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let counts: Vec<usize> = StatusFilter::ALL
        .iter()
        .map(|f| agents.iter().filter(|a| f.matches(a.status)).count())
        .collect();
    let (done, needs, failed) = since_last_seen(&spans, &inbox_items, last_seen_at);
    let spotlight = spotlight_agent_id.clone();
    let inbox: Vec<&InboxItem> = inbox_items
        .iter()
        .filter(|i| {
            agents
                .iter()
                .find(|a| a.id == i.agent_id)
                .map_or(status_filter == StatusFilter::All, |a| {
                    status_filter.matches(a.status)
                })
        })
        .collect();
    let root_class = class_list("kit-tl", &extra_classes);
    let ticks: Vec<String> = (0..=TICKS)
        .map(|i| axis_label(window_from + (window_to - window_from) * i / TICKS, now_ms))
        .collect();
    let now_pct = span_geometry(now_ms, now_ms, window_from, window_to).0;
    let axis_px = use_element_width("kit-tl-axis".to_string()).unwrap_or(0.0);
    let mut drag = use_signal(|| None::<(f64, u64, u64)>);
    let lanes: Vec<(TimelineAgent, Vec<TimelineSpan>)> = agents
        .iter()
        .map(|a| (a.clone(), lane_spans(&spans, &a.id)))
        .collect();
    let lane_ids: Vec<String> = agents.iter().map(|a| a.id.clone()).collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-labelledby": "kit-tl-title",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    if let Some(handler) = on_spotlight_change { handler.call(None); }
                }
            },
            header { class: "kit-tl__head",
                h2 { id: "kit-tl-title", class: "kit-tl__title", "{heading_text}" }
                div { class: "kit-tl__filter",
                    SegmentedControl {
                        aria_label: "Show agents",
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
            }
            section { class: "kit-tl__summary", "aria-label": "Since you last looked",
                div { class: "kit-tl__pan", role: "group", "aria-label": "Move the time window",
                    button {
                        r#type: "button",
                        class: "kit-tl__action",
                        onclick: move |_| {
                            let (nf, nt) = pan_window(window_from, window_to, -5 * 60_000);
                            if let Some(handler) = on_window_change { handler.call((nf, nt)); }
                        },
                        "Earlier"
                    }
                    button {
                        r#type: "button",
                        class: "kit-tl__action",
                        onclick: move |_| {
                            let (nf, nt) = pan_window(window_from, window_to, 5 * 60_000);
                            if let Some(handler) = on_window_change { handler.call((nf, nt)); }
                        },
                        "Later"
                    }
                }
                p { class: "kit-tl__tallies", "aria-live": "polite",
                    span { "{done} done" }
                    span { "{needs} needs you" }
                    span { "{failed} failed" }
                }
            }
            div { class: "kit-tl__body",
                div { class: "kit-tl__grid", role: "grid", "aria-label": "Agent lanes over time",
                    div { id: "kit-tl-axis", class: "kit-tl__axis", role: "row", "aria-label": "Time axis, drag or scroll to move the window",
                        onpointerdown: move |e: PointerEvent| drag.set(Some((e.client_coordinates().x, window_from, window_to))),
                        onpointermove: move |e: PointerEvent| {
                            if let Some((start_x, from, to)) = drag() {
                                if axis_px > 0.0 {
                                    let dx = e.client_coordinates().x - start_x;
                                    let delta = -(dx / axis_px * (to - from) as f64) as i64;
                                    let (nf, nt) = pan_window(from, to, delta);
                                    if let Some(handler) = on_window_change { handler.call((nf, nt)); }
                                }
                            }
                        },
                        onpointerup: move |_| drag.set(None),
                        onpointerleave: move |_| drag.set(None),
                        onwheel: move |e: WheelEvent| {
                            let dy = e.delta().strip_units().y;
                            if dy != 0.0 {
                                e.prevent_default();
                                let step = if dy > 0.0 { 60_000 } else { -60_000 };
                                let (nf, nt) = pan_window(window_from, window_to, step);
                                if let Some(handler) = on_window_change { handler.call((nf, nt)); }
                            }
                        },
                        span { class: "kit-tl__corner", role: "columnheader", "Agent" }
                        div { class: "kit-tl__ticks",
                            for tick in ticks.iter() {
                                span { class: "kit-tl__tick", "{tick}" }
                            }
                        }
                    }
                    for (agent, lane) in lanes.iter() {
                        {
                            let agent_id = agent.id.clone();
                            let focused = spotlight.as_deref() == Some(agent_id.as_str());
                            let dimmed = spotlight.as_deref().is_some_and(|s| s != agent_id);
                            let shown: Vec<TimelineSpan> = lane.iter().filter(|s| status_filter.matches(s.status)).cloned().collect();
                            let lane_index = lane_ids.iter().position(|id| *id == agent_id).unwrap_or(0);
                            let prev_lane = lane_index.checked_sub(1).map(|i| lane_ids[i].clone());
                            let next_lane = lane_ids.get(lane_index + 1).cloned();
                            let pick = agent_id.clone();
                            let siblings: Vec<String> = shown.iter().map(|s| s.id.clone()).collect();
                            let empty = shown.is_empty();
                            rsx! {
                                div { key: "{agent_id}", class: "kit-tl__lane", role: "row", "data-lane": "{agent_id}",
                                    "data-focused": focused.then_some("true"),
                                    "data-dimmed": dimmed.then_some("true"),
                                    button {
                                        r#type: "button",
                                        class: "kit-tl__lane-label",
                                        role: "rowheader",
                                        "aria-pressed": if focused { "true" } else { "false" },
                                        onclick: move |_| {
                                            if let Some(handler) = on_spotlight_change {
                                                handler.call(if focused { None } else { Some(pick.clone()) });
                                            }
                                        },
                                        span { class: "kit-tl__agent-name", "{agent.name}" }
                                        span { class: "kit-tl__agent-model", "{agent.model_label}" }
                                    }
                                    div { class: "kit-tl__track", role: "gridcell",
                                        div { class: "kit-tl__mini", "aria-hidden": "true",
                                            span { class: "kit-tl__marker", style: "left: {now_pct:.3}%" }
                                            for seg in shown.iter() {
                                                {
                                                    let (left, width) = span_geometry(seg.start_at, effective_end(seg, now_ms), window_from, window_to);
                                                    let state = seg.status.as_str();
                                                    rsx! {
                                                        span { class: "kit-tl__mini-seg", "data-status": "{state}", style: "left: {left:.3}%; width: {width:.3}%" }
                                                    }
                                                }
                                            }
                                        }
                                        span { class: "kit-tl__marker", style: "left: {now_pct:.3}%", "aria-hidden": "true" }
                                        for seg in shown.iter() {
                                            {
                                                let end = effective_end(seg, now_ms);
                                                let (left, width) = span_geometry(seg.start_at, end, window_from, window_to);
                                                let running = seg.end_at.is_none();
                                                let span_id = seg.id.clone();
                                                let pos = siblings.iter().position(|s| *s == span_id).unwrap_or(0);
                                                let prev_span = pos.checked_sub(1).map(|i| siblings[i].clone());
                                                let next_span = siblings.get(pos + 1).cloned();
                                                let up = prev_lane.clone();
                                                let down = next_lane.clone();
                                                let click_agent = seg.agent_id.clone();
                                                let started = wait_label(now_ms.saturating_sub(seg.start_at) / 1000);
                                                let state_word = seg.status.as_str();
                                                let aria = format!("{}, {state_word}", seg.label);
                                                rsx! {
                                                    button {
                                                        key: "{span_id}",
                                                        id: "kit-tl-span-{span_id}",
                                                        r#type: "button",
                                                        class: "kit-tl__span",
                                                        "data-status": "{state_word}",
                                                        "data-running": running.then_some("true"),
                                                        style: "left: {left:.3}%; width: {width:.3}%",
                                                        "aria-label": "{aria}",
                                                        onclick: move |_| {
                                                            if let Some(handler) = on_spotlight_change {
                                                                handler.call(Some(click_agent.clone()));
                                                            }
                                                        },
                                                        onkeydown: move |e: KeyboardEvent| {
                                                            let target = match e.key() {
                                                                Key::ArrowLeft => prev_span.clone(),
                                                                Key::ArrowRight => next_span.clone(),
                                                                Key::ArrowUp => up.clone().map(|l| format!("lane:{l}")),
                                                                Key::ArrowDown => down.clone().map(|l| format!("lane:{l}")),
                                                                _ => None,
                                                            };
                                                            if let Some(target) = target {
                                                                e.prevent_default();
                                                                focus_element(&target);
                                                            }
                                                        },
                                                        span { class: "kit-tl__span-label", "{seg.label}" }
                                                        span { class: "kit-tl__span-state", "{state_word}" }
                                                        span { class: "kit-tl__tip", role: "tooltip",
                                                            span { class: "kit-tl__tip-title", "{seg.label}" }
                                                            span { "{seg.status.label()}, started {started} ago" }
                                                            if let Some(error) = seg.error_line.clone() {
                                                                span { class: "kit-tl__tip-error", "{error}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if empty {
                                            span { class: "kit-tl__empty-lane", "No spans" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                aside { class: "kit-tl__inbox", "aria-label": "Inbox",
                    h3 { class: "kit-tl__inbox-title", "Needs a decision " span { class: "kit-tl__count", "{inbox.len()}" } }
                    if inbox.is_empty() {
                        p { class: "kit-tl__empty", "Nothing needs a decision." }
                    }
                    for item in inbox.iter().copied() {
                        {
                            let item_id = item.id.clone();
                            let agent_id = item.agent_id.clone();
                            let dimmed = spotlight.as_deref().is_some_and(|s| s != agent_id);
                            let focused = spotlight.as_deref() == Some(agent_id.as_str());
                            let tone = if item.kind == InboxKind::Failure { BadgeTone::Danger } else { BadgeTone::Warning };
                            let detail = format!("{} · waiting {}", item.detail, wait_label(item.waiting_secs));
                            rsx! {
                                article { key: "{item_id}", class: "kit-tl__item", "data-dimmed": dimmed.then_some("true"), "data-focused": focused.then_some("true"),
                                    header { class: "kit-tl__item-head",
                                        Badge { tone: tone, "{item.kind.word()}" }
                                        span { class: "kit-tl__item-title", "{item.title}" }
                                    }
                                    p { class: "kit-tl__detail", "{detail}" }
                                    div { class: "kit-tl__actions",
                                        for action in item.actions.iter() {
                                            {
                                                let aid = action.id.clone();
                                                let iid = item_id.clone();
                                                rsx! {
                                                    button {
                                                        key: "{aid}",
                                                        r#type: "button",
                                                        class: "kit-tl__action",
                                                        onclick: move |_| {
                                                            if let Some(handler) = on_inbox_answer {
                                                                handler.call((iid.clone(), aid.clone()));
                                                            }
                                                        },
                                                        "{action.label}"
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
