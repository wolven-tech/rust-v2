//! Single run status card: one agent run kept in view, with its current step, a
//! clock that excludes paused time, its spend, and the approval it needs.
//!
//! Design choice: marks are image URLs; a missing mark shows the label's initial.
//! Design choice: the clock reads the browser wall clock and freezes on entry to
//! `held`. Enter does not grant on the card, because a footer button would fire
//! twice; Escape closes the details panel first, and refuses only when none is open.
//! Browser checks: collapse and re-expand keeps the clock; details focus moves to
//! the close control and back to the row; strip widths follow durations; the step
//! list scrolls after three rows; reduced motion removes transitions.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id, use_now_ms};

const CSS: Asset = asset!("/assets/single-run-status-card.css");

/// Moves keyboard focus to the element with `id`; a missing element is ignored.
pub(crate) fn focus_by_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// `m:ss`, minutes unpadded and not wrapped at an hour.
pub fn format_mm_ss(ms: u64) -> String {
    let total = ms / 1000;
    format!("{}:{:02}", total / 60, total % 60)
}

/// The spoken form of a duration for screen readers, such as `2 minutes 5 seconds`.
pub fn spoken_duration(ms: u64) -> String {
    let total = ms / 1000;
    let (minutes, seconds) = (total / 60, total % 60);
    let unit = |n: u64, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
    format!("{} {}", unit(minutes, "minute"), unit(seconds, "second"))
}

/// Dollar figure with two decimals.
pub fn format_usd(amount: f64) -> String {
    format!("${amount:.2}")
}

/// Upper-case first character of a label, or `?` for an empty one.
pub fn initial_of(label: &str) -> String {
    label
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_else(|| "?".to_string())
}

/// Stable hue (0..360) per worker label, so a run keeps its tint across renders.
fn identity_hue(label: &str) -> u32 {
    label.bytes().fold(0u32, |hash, b| {
        hash.wrapping_mul(31).wrapping_add(u32::from(b))
    }) % 360
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunPhase {
    Standby,
    Active,
    AwaitingGrant,
    Held,
    Complete,
    Errored,
    Stopped,
}

impl RunPhase {
    pub fn word(self) -> &'static str {
        match self {
            Self::Standby => "Standby",
            Self::Active => "Active",
            Self::AwaitingGrant => "Awaiting grant",
            Self::Held => "Held",
            Self::Complete => "Complete",
            Self::Errored => "Errored",
            Self::Stopped => "Stopped",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Standby => "standby",
            Self::Active => "active",
            Self::AwaitingGrant => "awaiting_grant",
            Self::Held => "held",
            Self::Complete => "complete",
            Self::Errored => "errored",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    Inspect,
    Find,
    Modify,
    Execute,
    Reason,
    Network,
}

impl StepKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::Find => "find",
            Self::Modify => "modify",
            Self::Execute => "execute",
            Self::Reason => "reason",
            Self::Network => "network",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepState {
    Pending,
    InProgress,
    Succeeded,
    Errored,
}

impl StepState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Succeeded => "succeeded",
            Self::Errored => "errored",
        }
    }
}

/// A control in the card footer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunAction {
    Hold,
    Stop,
    Resume,
    Grant,
    Refuse,
    Retry,
    OpenLog,
}

impl RunAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Hold => "Hold",
            Self::Stop => "Stop",
            Self::Resume => "Resume",
            Self::Grant => "Grant",
            Self::Refuse => "Refuse",
            Self::Retry => "Retry",
            Self::OpenLog => "Open log",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct WorkerIdentity {
    pub display_label: String,
    pub role_label: Option<String>,
    pub mark: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct StepRow {
    pub step_key: String,
    pub step_label: String,
    pub step_kind: Option<StepKind>,
    pub step_state: StepState,
    pub started_at_ms: Option<u64>,
    pub elapsed_ms: Option<u64>,
    pub tool_mark: Option<String>,
    pub tool_label: Option<String>,
    pub detail_text: Option<String>,
    /// The command run or the place touched.
    pub target_text: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PendingRequest {
    pub request_title: String,
    pub request_command: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct ResultSummary {
    pub lines_added: Option<u32>,
    pub lines_removed: Option<u32>,
    pub items_touched: Option<u32>,
}

/// Wall time from start to end (or `now` while open), minus paused time.
pub fn elapsed_ms(started: Option<u64>, ended: Option<u64>, paused_total: u64, now: u64) -> u64 {
    match started {
        None => 0,
        Some(start) => ended
            .unwrap_or(now)
            .saturating_sub(start)
            .saturating_sub(paused_total),
    }
}

/// Recorded elapsed time wins; an in-progress step without one counts up from its start.
pub fn step_duration_ms(step: &StepRow, now: u64) -> u64 {
    match (step.elapsed_ms, step.started_at_ms, step.step_state) {
        (Some(ms), _, _) => ms,
        (None, Some(start), StepState::InProgress) => now.saturating_sub(start),
        _ => 0,
    }
}

/// Footer controls for a phase, in display order.
pub fn footer_actions(phase: RunPhase) -> &'static [RunAction] {
    match phase {
        RunPhase::Standby | RunPhase::Complete => &[],
        RunPhase::Active => &[RunAction::Hold, RunAction::Stop],
        RunPhase::Held => &[RunAction::Resume, RunAction::Stop],
        RunPhase::AwaitingGrant => &[RunAction::Grant, RunAction::Refuse],
        RunPhase::Errored => &[RunAction::Retry, RunAction::OpenLog],
        RunPhase::Stopped => &[RunAction::OpenLog],
    }
}

/// Time-strip segment weights, at least one unit each so every step stays visible.
pub fn strip_weights(steps: &[StepRow], now: u64) -> Vec<u64> {
    steps
        .iter()
        .map(|step| step_duration_ms(step, now).max(1))
        .collect()
}

#[component]
pub fn SingleRunStatusCard(
    run_phase: RunPhase,
    worker_identity: WorkerIdentity,
    #[props(default)] step_rows: Vec<StepRow>,
    #[props(default)] started_at_ms: Option<u64>,
    #[props(default)] ended_at_ms: Option<u64>,
    #[props(default)] paused_total_ms: u64,
    #[props(default)] spend_usd: Option<f64>,
    #[props(default)] pending_request: Option<PendingRequest>,
    #[props(default)] result_summary: Option<ResultSummary>,
    #[props(default)] error_text: Option<String>,
    #[props(default)] focus_label: Option<String>,
    #[props(default)] on_grant: Option<EventHandler<()>>,
    #[props(default)] on_refuse: Option<EventHandler<()>>,
    #[props(default)] on_hold: Option<EventHandler<()>>,
    #[props(default)] on_resume: Option<EventHandler<()>>,
    #[props(default)] on_stop: Option<EventHandler<()>>,
    #[props(default)] on_retry: Option<EventHandler<()>>,
    #[props(default)] on_open_log: Option<EventHandler<()>>,
    #[props(default)] start_collapsed: bool,
    #[props(default)] clock_ms: Option<u64>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-srsc-{}", next_id()));
    let wall = use_now_ms(clock_ms, 1000);
    let held = run_phase == RunPhase::Held;
    let mut frozen = use_signal(|| None::<u64>);
    use_effect(use_reactive!(|(held, wall)| {
        let current = *frozen.peek();
        if !held {
            if current.is_some() {
                frozen.set(None);
            }
        } else if current.is_none() && wall > 0 {
            frozen.set(Some(wall));
        }
    }));
    let now = frozen().unwrap_or(wall);

    let mut collapsed = use_signal(|| start_collapsed);
    let mut open_step = use_signal(|| None::<String>);
    let mut toggle_step = move |key: String| {
        let opening = open_step().as_deref() != Some(key.as_str());
        open_step.set(opening.then_some(key));
    };

    let elapsed = elapsed_ms(started_at_ms, ended_at_ms, paused_total_ms, now);
    let clock = format_mm_ss(elapsed);
    let in_progress = step_rows
        .iter()
        .find(|step| step.step_state == StepState::InProgress)
        .map(|step| step.step_label.clone());
    let current_line = match (run_phase, focus_label, in_progress) {
        (RunPhase::Active, Some(label), _) => label,
        (_, _, Some(label)) => label,
        (RunPhase::Standby, _, _) => "Not started".to_string(),
        _ => run_phase.word().to_string(),
    };
    let phase_word = run_phase.word();
    let phase_key = run_phase.as_str();
    let region_name = format!(
        "{} · {} · {}",
        worker_identity.display_label, phase_word, current_line
    );
    let display_label = worker_identity.display_label.clone();
    let initial = initial_of(&display_label);
    let hue_style = format!("--kit-srsc-hue: {};", identity_hue(&display_label));
    let role_label = worker_identity.role_label.clone();
    let mark = worker_identity.mark.clone();
    let spend_text = spend_usd.map(format_usd);
    let strip: Vec<(String, String, &'static str, u64, String)> = step_rows
        .iter()
        .zip(strip_weights(&step_rows, now))
        .map(|(step, grow)| {
            (
                step.step_key.clone(),
                step.step_label.clone(),
                step.step_state.as_str(),
                grow,
                format_mm_ss(step_duration_ms(step, now)),
            )
        })
        .collect();
    let strip_len = strip.len();
    let rows: Vec<(String, String, String, String, String, String, &'static str)> = step_rows
        .iter()
        .rev()
        .filter(|step| matches!(step.step_state, StepState::Succeeded | StepState::Errored))
        .map(|step| {
            let tool = step.tool_label.clone().unwrap_or_default();
            let glyph = initial_of(if tool.is_empty() {
                &step.step_label
            } else {
                &tool
            });
            (
                step.step_key.clone(),
                format!("{id}-row-{}", step.step_key),
                glyph,
                step.step_label.clone(),
                tool,
                format_mm_ss(step_duration_ms(step, now)),
                step.step_state.as_str(),
            )
        })
        .collect();
    let detail = step_rows
        .iter()
        .find(|step| open_step().as_deref() == Some(step.step_key.as_str()))
        .cloned();
    let detail_text = detail
        .as_ref()
        .and_then(|step| step.detail_text.clone())
        .unwrap_or_default();
    let detail_target = detail.as_ref().and_then(|step| step.target_text.clone());
    let detail_label = detail
        .as_ref()
        .map(|step| format!("Details of {}", step.step_label))
        .unwrap_or_default();
    let detail_key = detail.as_ref().map(|step| step.step_key.clone());
    let detail_row_id = detail_key
        .as_ref()
        .map(|key| format!("{id}-row-{key}"))
        .unwrap_or_default();
    let close_id = format!("{id}-close");
    let escape_row_id = detail_row_id.clone();
    let request = pending_request.filter(|_| run_phase == RunPhase::AwaitingGrant);
    let actions: Vec<(RunAction, &'static str)> = footer_actions(run_phase)
        .iter()
        .map(|action| (*action, action.label()))
        .collect();
    let tools: Vec<(String, String)> = step_rows
        .iter()
        .filter_map(|step| {
            step.tool_mark
                .clone()
                .map(|mark| (mark, step.tool_label.clone().unwrap_or_default()))
        })
        .take(4)
        .collect();
    let class = class_list("kit-srsc", &class_name);
    let spend_label = spend_text.clone().unwrap_or_default();
    let clock_label = format!("Elapsed {}", spoken_duration(elapsed));
    let steps_label = format!("{strip_len} steps");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class,
            style: "{hue_style}",
            role: "region",
            "aria-label": "{region_name}",
            "data-phase": phase_key,
            onkeydown: move |event: KeyboardEvent| {
                if event.key() != Key::Escape {
                    return;
                }
                if open_step().is_some() {
                    open_step.set(None);
                    focus_after_frame(&escape_row_id);
                } else if run_phase == RunPhase::AwaitingGrant {
                    if let Some(handler) = &on_refuse {
                        handler.call(());
                    }
                }
            },
            if collapsed() {
                button {
                    class: "kit-srsc__bar",
                    r#type: "button",
                    "aria-label": "{region_name}",
                    "aria-expanded": "false",
                    onclick: move |_| collapsed.set(false),
                    span { class: "kit-srsc__identity", "{display_label}" }
                    span { class: "kit-srsc__phase", "data-phase": phase_key, "{phase_word}" }
                    span { class: "kit-srsc__clock", "{clock}" }
                    span { class: "kit-srsc__tools", "aria-hidden": "true",
                        for (src, _label) in tools.clone() {
                            img { key: "{src}", src: "{src}", alt: "" }
                        }
                    }
                }
            } else {
                header { class: "kit-srsc__head",
                    span { class: "kit-srsc__mark", "aria-hidden": "true",
                        if let Some(src) = mark {
                            img { src: "{src}", alt: "" }
                        } else {
                            "{initial}"
                        }
                    }
                    div { class: "kit-srsc__identity-block",
                        span { class: "kit-srsc__identity", "{display_label}" }
                        if let Some(role) = role_label {
                            span { class: "kit-srsc__role", "{role}" }
                        }
                    }
                    span { class: "kit-srsc__phase", "data-phase": phase_key, "{phase_word}" }
                    if spend_text.is_some() {
                        span { class: "kit-srsc__spend", "aria-label": "Spend {spend_label}", "{spend_label}" }
                    }
                    button {
                        class: "kit-srsc__collapse",
                        r#type: "button",
                        "aria-expanded": "true",
                        "aria-label": "Collapse run card",
                        onclick: move |_| collapsed.set(true),
                        "–"
                    }
                }

                div { class: "kit-srsc__current",
                    span { class: "kit-srsc__current-text", "{current_line}" }
                    span { class: "kit-srsc__clock", "aria-label": "{clock_label}", "{clock}" }
                }

                if strip_len > 0 {
                    div { class: "kit-srsc__strip", role: "img", "aria-label": "{steps_label}",
                        for (key, label, state, grow, dur) in strip.clone() {
                            button {
                                key: "{key}",
                                class: "kit-srsc__seg",
                                r#type: "button",
                                "data-state": state,
                                style: format!("flex-grow: {grow}"),
                                "aria-label": "{label}, {dur}",
                                onclick: move |_| toggle_step(key.clone()),
                            }
                        }
                    }
                }

                if let Some(request) = request {
                    div { class: "kit-srsc__request",
                        p { class: "kit-srsc__request-title", "{request.request_title}" }
                        if let Some(command) = request.request_command {
                            code { class: "kit-srsc__command", "{command}" }
                        }
                    }
                }

                if run_phase == RunPhase::Errored {
                    if let Some(text) = error_text {
                        p { class: "kit-srsc__error", role: "alert", "{text}" }
                    }
                }

                if run_phase == RunPhase::Complete {
                    if let Some(summary) = result_summary {
                        p { class: "kit-srsc__result",
                            if let Some(n) = summary.lines_added { span { "+{n} lines" } }
                            if let Some(n) = summary.lines_removed { span { "−{n} lines" } }
                            if let Some(n) = summary.items_touched { span { "{n} items touched" } }
                        }
                    }
                }

                if !rows.is_empty() {
                    ol { class: "kit-srsc__steps",
                        for (key, row_id, glyph, label, tool, dur, state) in rows.clone() {
                            li { key: "{key}",
                                button {
                                    id: "{row_id}",
                                    class: "kit-srsc__row",
                                    r#type: "button",
                                    "data-state": state,
                                    "aria-expanded": if detail_key.as_deref() == Some(key.as_str()) { "true" } else { "false" },
                                    onclick: move |_| toggle_step(key.clone()),
                                    span { class: "kit-srsc__row-tile", "aria-hidden": "true", "{glyph}" }
                                    span { class: "kit-srsc__row-label", "{label}" }
                                    span { class: "kit-srsc__tool", "{tool}" }
                                    span { class: "kit-srsc__row-time", "{dur}" }
                                }
                            }
                        }
                    }
                }

                if detail.is_some() {
                    div {
                        class: "kit-srsc__details",
                        role: "dialog",
                        "aria-label": "{detail_label}",
                        p { class: "kit-srsc__detail-text", "{detail_text}" }
                        if let Some(target) = detail_target {
                            code { class: "kit-srsc__command", "{target}" }
                        }
                        button {
                            id: "{close_id}",
                            class: "kit-srsc__close",
                            r#type: "button",
                            onmounted: move |event: MountedEvent| {
                                let node = event.data();
                                spawn(async move {
                                    let _ = node.set_focus(true).await;
                                });
                            },
                            onclick: move |_| {
                                open_step.set(None);
                                focus_after_frame(&detail_row_id);
                            },
                            "Close details"
                        }
                    }
                }

                footer { class: "kit-srsc__footer",
                    for (action, label) in actions {
                        button {
                            key: "{label}",
                            class: "kit-srsc__action",
                            r#type: "button",
                            "data-action": label,
                            onclick: move |_| {
                                let handler = match action {
                                    RunAction::Hold => &on_hold,
                                    RunAction::Stop => &on_stop,
                                    RunAction::Resume => &on_resume,
                                    RunAction::Grant => &on_grant,
                                    RunAction::Refuse => &on_refuse,
                                    RunAction::Retry => &on_retry,
                                    RunAction::OpenLog => &on_open_log,
                                };
                                if let Some(handler) = handler {
                                    handler.call(());
                                }
                            },
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
