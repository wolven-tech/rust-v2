//! Model failover flow: one request tried against each model in its fallback order, until one answers.
//!
//! Design choice: `elapsed_ms` is the caller's running clock, shown as the total time. Per-attempt durations come
//! from `duration` on each attempt, so no wall clock is read and SSR stays deterministic.
//! Design choice: marks are text glyphs, and an End's detail lines replace the spec's hover-card node content.
//! Browser checks: running pulse, timer count-up, hover and focus cards, narrow reflow into a vertical tree.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/model-failover-flow.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttemptState {
    Queued,
    InProgress,
    Failed,
    Answered,
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowStatus {
    Running,
    Answered,
    Exhausted,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Attempt {
    pub attempt_id: String,
    pub model_name: String,
    pub provider_name: String,
    pub attempt_state: AttemptState,
    /// Milliseconds the attempt took, once it has finished.
    pub duration: Option<f64>,
    /// Cost in dollars; `None` shows no cost rather than zero.
    pub attempt_cost: Option<f64>,
    pub error_text: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct End {
    pub node_name: String,
    pub node_mark: Option<String>,
    pub node_text: Option<String>,
    pub node_details: Vec<String>,
}

/// Overall status of the request from its attempts.
pub fn flow_status(attempts: &[Attempt]) -> FlowStatus {
    if attempts
        .iter()
        .any(|a| a.attempt_state == AttemptState::Answered)
    {
        FlowStatus::Answered
    } else if !attempts.is_empty()
        && attempts
            .iter()
            .all(|a| a.attempt_state == AttemptState::Failed)
    {
        FlowStatus::Exhausted
    } else {
        FlowStatus::Running
    }
}

/// Index of the attempt in progress, else the next queued one.
pub fn current_attempt(attempts: &[Attempt]) -> Option<usize> {
    attempts
        .iter()
        .position(|a| a.attempt_state == AttemptState::InProgress)
        .or_else(|| {
            attempts
                .iter()
                .position(|a| a.attempt_state == AttemptState::Queued)
        })
}

/// "Attempt n of N": the position of the current attempt, or the last one tried when none is running.
pub fn attempt_position(attempts: &[Attempt]) -> (usize, usize) {
    let total = attempts.len();
    let n = current_attempt(attempts)
        .map(|i| i + 1)
        .or_else(|| {
            attempts
                .iter()
                .rposition(|a| a.attempt_state != AttemptState::Queued)
                .map(|i| i + 1)
        })
        .unwrap_or(0);
    (n, total)
}

/// State as shown: attempts queued behind an answer read as skipped, not failed.
pub fn shown_state(attempts: &[Attempt], index: usize) -> AttemptState {
    let answered_at = attempts
        .iter()
        .position(|a| a.attempt_state == AttemptState::Answered);
    match answered_at {
        Some(a) if index > a && attempts[index].attempt_state == AttemptState::Queued => {
            AttemptState::Skipped
        }
        _ => attempts[index].attempt_state,
    }
}

/// Spelled-out state for an attempt, so no state is conveyed by tone alone.
pub fn state_word(state: AttemptState, error: Option<&str>) -> String {
    match state {
        AttemptState::Queued => "Up next".to_string(),
        AttemptState::InProgress => "Running".to_string(),
        AttemptState::Failed => error
            .map(str::to_string)
            .unwrap_or_else(|| "Failed".to_string()),
        AttemptState::Answered => "Answered".to_string(),
        AttemptState::Skipped => "Not needed".to_string(),
    }
}

/// Dollar text for a cost; `None` when the attempt has no cost.
pub fn cost_text(cost: Option<f64>) -> Option<String> {
    cost.map(|c| format!("${c:.4}"))
}

/// Total cost across attempts that report one.
pub fn total_cost(attempts: &[Attempt]) -> f64 {
    attempts.iter().filter_map(|a| a.attempt_cost).sum()
}

/// Milliseconds as a short duration: "850 ms", "2.4 s".
pub fn duration_text(ms: f64) -> String {
    if ms < 1000.0 {
        format!("{ms:.0} ms")
    } else {
        format!("{:.1} s", ms / 1000.0)
    }
}

/// Request routed through its fallback models. Each attempt is a branch; the answering branch reaches the result.
#[component]
pub fn ModelFailoverFlow(
    attempt_list: Vec<Attempt>,
    #[props(default = "Request".to_string(), into)] heading_text: String,
    #[props(default)] routing_label: Option<String>,
    #[props(default)] origin_node: Option<End>,
    #[props(default)] result_node: Option<End>,
    #[props(default)] elapsed_ms: Option<f64>,
    #[props(default)] on_retry: Option<EventHandler<()>>,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-mfl-{}", next_id()));
    let status = flow_status(&attempt_list);
    let (n, total) = attempt_position(&attempt_list);
    let cost = total_cost(&attempt_list);
    let answered = attempt_list
        .iter()
        .position(|a| a.attempt_state == AttemptState::Answered);
    let class = class_list("kit-mfl", &extra_class);
    let summary = match status {
        FlowStatus::Answered => format!(
            "Answered on attempt {} of {total}",
            answered.map_or(0, |i| i + 1)
        ),
        FlowStatus::Exhausted => format!("All {total} attempts failed"),
        FlowStatus::Running => format!("Attempt {n} of {total}"),
    };
    let elapsed = elapsed_ms.map(duration_text).unwrap_or_default();
    let cost_label = cost_text(Some(cost)).unwrap_or_default();
    let wires_height = 128 * attempt_list.len().max(1);
    let box_height = format!("--mfl-h: {wires_height}px;");
    let routing = routing_label.clone().unwrap_or_else(|| "Route".to_string());
    let origin = origin_node.clone().unwrap_or_else(|| End {
        node_name: "Request".into(),
        ..Default::default()
    });
    let result = result_node.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title", "data-status": match status {
            FlowStatus::Running => "running",
            FlowStatus::Answered => "answered",
            FlowStatus::Exhausted => "exhausted",
        },
            header { class: "kit-mfl__head",
                h2 { id: "{base}-title", class: "kit-mfl__title", "{heading_text}" }
                p { class: "kit-mfl__summary", role: "status", "aria-live": "polite",
                    span { "{summary}" }
                    if !elapsed.is_empty() { span { class: "kit-mfl__time", " · {elapsed}" } }
                    if !cost_label.is_empty() { span { class: "kit-mfl__cost", " · {cost_label}" } }
                }
            }
            div { class: "kit-mfl__flow", style: "{box_height}",
                svg { class: "kit-mfl__wires", view_box: "0 0 100 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    for (i, a) in attempt_list.iter().enumerate() {
                        {
                            let y = 6.0 + (i as f64 + 0.5) / attempt_list.len().max(1) as f64 * 88.0;
                            let state = shown_state(&attempt_list, i);
                            let on = matches!(state, AttemptState::Answered);
                            let running = state == AttemptState::InProgress;
                            let failed = state == AttemptState::Failed;
                            let key = a.attempt_id.clone();
                            rsx! {
                                line { key: "{key}-in", class: "kit-mfl__wire", "data-state": "{state:?}", "data-running": running.then_some("true"),
                                    "data-failed": failed.then_some("true"), x1: "28", y1: "50", x2: "30", y2: "{y}" }
                                if on { line { key: "{key}-out", class: "kit-mfl__wire", "data-state": "Answered", x1: "70", y1: "{y}", x2: "72", y2: "50" } }
                            }
                        }
                    }
                }
                div { class: "kit-mfl__origin", role: "group", "aria-label": "Request",
                    span { class: "kit-mfl__name", "{origin.node_name}" }
                    span { class: "kit-mfl__meta", "{routing}" }
                    if let Some(text) = origin.node_text.clone() { span { class: "kit-mfl__meta", "{text}" } }
                }
                ol { class: "kit-mfl__attempts", "aria-label": "Attempts, in order",
                    for (i, a) in attempt_list.iter().enumerate() {
                        {
                            let state = shown_state(&attempt_list, i);
                            let word = state_word(state, a.error_text.as_deref());
                            let time = match state {
                                AttemptState::Answered | AttemptState::Failed => a.duration.map(duration_text).unwrap_or_default(),
                                _ => String::new(),
                            };
                            let cost = cost_text(a.attempt_cost);
                            let label = format!("{}, {}, {word}", a.model_name, a.provider_name);
                            let style = format!("--mfl-y: {:.2}%;", 6.0 + (i as f64 + 0.5) / attempt_list.len().max(1) as f64 * 88.0);
                            let dom_id = format!("{base}-a-{i}");
                            let model = a.model_name.clone();
                            let provider = a.provider_name.clone();
                            let state_attr = format!("{state:?}");
                            let mut card_parts = vec![provider.clone(), word.clone()];
                            if !time.is_empty() {
                                card_parts.push(time.clone());
                            }
                            card_parts.extend(cost.clone());
                            let card = card_parts.join(" · ");
                            let card_id = format!("{dom_id}-card");
                            rsx! {
                                li { key: "{a.attempt_id}", class: "kit-mfl__attempt", style: "{style}", "data-state": "{state_attr}",
                                    button {
                                        id: "{dom_id}", r#type: "button", class: "kit-mfl__node", "aria-label": "{label}",
                                        "aria-describedby": "{card_id}",
                                        span { class: "kit-mfl__name", "{model}" }
                                        span { class: "kit-mfl__provider", "{provider}" }
                                        span { class: "kit-mfl__state", "{word}" }
                                        if !time.is_empty() { span { class: "kit-mfl__meta", "{time}" } }
                                        if let Some(c) = cost { span { class: "kit-mfl__meta", "{c}" } }
                                    }
                                    span { class: "kit-mfl__card", id: "{card_id}", role: "tooltip", "{card}" }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-mfl__result", "aria-label": "Result",
                    if let Some(end) = result.clone() {
                        span { class: "kit-mfl__name", "{end.node_name}" }
                        if status == FlowStatus::Answered {
                            if let Some(text) = end.node_text.clone() { span { class: "kit-mfl__preview", "{text}" } }
                        }
                    }
                }
            }
            if status == FlowStatus::Exhausted {
                if let Some(handler) = on_retry {
                    button { r#type: "button", class: "kit-mfl__retry", onclick: move |_| handler.call(()), "Retry" }
                }
            }
        }
    }
}
