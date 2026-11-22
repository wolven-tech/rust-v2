//! Agent run timeline: one agent run, step by step, along a time axis. It shows
//! which steps took the time, which failed, what each received and returned, and
//! what the run cost. While the run is live the bars extend and a now line moves.
//!
//! Design choice: a live run is one with no `total_seconds`. Its running steps
//! grow to `elapsed_seconds`, which the host supplies on each update. This module
//! does not read a clock.
//! Design choice: tick intervals are picked from a fixed list of round values so
//! that a run of any length gets six or fewer ticks.
//! Design choice: steps are rows of buttons inside a list, so each row has a name,
//! and Enter or a click opens it in the panel.
//! Design choice: at 640 px and below a clicked step opens the panel as a full-width
//! sheet with a Back control. Arrow and J/K keys move from the focused row.
//! Design choice: a running bar carries a sweep highlight, stopped under reduced
//! motion. While live, the elapsed figure advances each second from the last value
//! the host sent.
//! Browser checks: a running step's bar grows with `elapsed_seconds`; a failed
//! bar uses the failed variant and its error shows when selected; sub-agent rows
//! are indented under their parent; Stop appears only while live and Rerun only
//! when finished; Copy on the input copies the exact text; tick labels are round;
//! no horizontal scroll at 320 px.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, copy_to_clipboard, focus_after_frame, next_id, sleep};

const CSS: Asset = asset!("/assets/agent-run-timeline.css");

const TICK_STEPS: [f64; 13] = [
    1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0, 1800.0, 3600.0,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepType {
    Model,
    Tool,
    SubAgent,
}

impl StepType {
    fn key(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Tool => "tool",
            Self::SubAgent => "sub-agent",
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Model => "Model call",
            Self::Tool => "Tool",
            Self::SubAgent => "Sub-agent",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StepRecord {
    pub id: String,
    pub entry_label: String,
    pub step_type: StepType,
    pub start_seconds: f64,
    pub duration_seconds: Option<f64>,
    pub nesting_depth: u8,
    pub app_name: Option<String>,
    pub sub_agent_name: Option<String>,
    pub has_failed: bool,
    pub error_text: Option<String>,
    pub token_counts: Option<(u64, u64)>,
    pub cost_label: Option<String>,
    pub input_text: Option<String>,
    pub output_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunRecord {
    pub id: String,
    pub entry_title: String,
    pub agent_name: String,
    pub model_name: String,
    pub started_at_ms: u64,
    /// Absent while the run is live.
    pub total_seconds: Option<f64>,
    pub cost_label: Option<String>,
    pub step_records: Vec<StepRecord>,
}

/// A round tick interval (seconds) giving at most six ticks across `span`.
pub fn tick_interval(span: f64) -> f64 {
    TICK_STEPS
        .iter()
        .copied()
        .find(|step| span / step <= 6.0)
        .unwrap_or(3600.0)
}

/// Tick positions in seconds from zero to `span`, at round intervals.
pub fn axis_ticks(span: f64) -> Vec<f64> {
    let step = tick_interval(span);
    let mut ticks = Vec::new();
    let mut at = 0.0;
    while at <= span + f64::EPSILON {
        ticks.push(at);
        at += step;
    }
    ticks
}

/// `0:30`, `2:00`, `1:02:00` for a tick in seconds.
pub fn tick_label(seconds: f64) -> String {
    let total = seconds.round() as u64;
    let (h, m, s) = (total / 3600, (total % 3600) / 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// Where a step's bar starts and how wide it is, as percent of `span`, and whether
/// it is still running.
pub fn bar_geometry(step: &StepRecord, span: f64, elapsed: Option<f64>) -> (f64, f64, bool) {
    let span = span.max(f64::EPSILON);
    let (end, running) = match step.duration_seconds {
        Some(duration) => (step.start_seconds + duration, false),
        None => (elapsed.unwrap_or(step.start_seconds), true),
    };
    let start_pct = (step.start_seconds / span * 100.0).clamp(0.0, 100.0);
    let width_pct =
        ((end - step.start_seconds).max(0.0) / span * 100.0).clamp(0.0, 100.0 - start_pct);
    (start_pct, width_pct, running)
}

/// Length of the axis: the run's total, else the furthest point reached so far.
pub fn axis_span(run: &RunRecord, elapsed: Option<f64>) -> f64 {
    if let Some(total) = run.total_seconds {
        return total;
    }
    let reached = run
        .step_records
        .iter()
        .map(|s| s.start_seconds + s.duration_seconds.unwrap_or(0.0))
        .fold(0.0_f64, f64::max);
    reached.max(elapsed.unwrap_or(0.0)).max(1.0)
}

/// Token totals for the summary tile: input and output.
pub fn token_totals(steps: &[StepRecord]) -> (u64, u64) {
    steps
        .iter()
        .filter_map(|s| s.token_counts)
        .fold((0, 0), |(i, o), (ti, to)| (i + ti, o + to))
}

/// The step `delta` rows from `current`, clamped at both ends.
pub fn move_index(order: &[String], current: Option<&str>, delta: isize) -> Option<String> {
    if order.is_empty() {
        return None;
    }
    let last = order.len() as isize - 1;
    let next = match current.and_then(|id| order.iter().position(|o| o == id)) {
        Some(index) => (index as isize + delta).clamp(0, last),
        None if delta >= 0 => 0,
        None => last,
    };
    Some(order[next as usize].clone())
}

fn step_name(step: &StepRecord) -> String {
    match &step.app_name {
        Some(app) => format!("{} · {}", step.entry_label, app),
        None => step.entry_label.clone(),
    }
}

fn duration_word(step: &StepRecord, running: bool) -> String {
    if step.has_failed {
        "failed".to_string()
    } else if running {
        "running".to_string()
    } else {
        format!("{:.1} s", step.duration_seconds.unwrap_or(0.0))
    }
}

/// One run as a timeline with a detail panel for the selected step.
#[component]
pub fn AgentRunTimeline(
    #[props(default)] run_record: Option<RunRecord>,
    #[props(default)] elapsed_seconds: Option<f64>,
    #[props(default)] initial_step_id: Option<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_stop: Option<EventHandler<()>>,
    #[props(default)] on_rerun: Option<EventHandler<()>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
) -> Element {
    let uid = use_hook(next_id);
    let mut selected = use_signal(move || initial_step_id.clone());
    let mut focused = use_signal(|| None::<String>);
    let mut sheet_open = use_signal(|| false);
    let mut live_flag = use_signal(|| false);
    let mut extra = use_signal(|| 0.0_f64);
    let mut seen_elapsed = use_signal(|| elapsed_seconds);

    // Between host updates the live elapsed figure advances once a second. A host
    // update restarts the count from the value it sent.
    use_future(move || async move {
        loop {
            sleep(1000).await;
            if *live_flag.peek() {
                extra.with_mut(|seconds| *seconds += 1.0);
            }
        }
    });

    let steps: Vec<StepRecord> = run_record
        .as_ref()
        .map_or_else(Vec::new, |r| r.step_records.clone());
    let live = run_record
        .as_ref()
        .is_some_and(|r| r.total_seconds.is_none());
    if *live_flag.peek() != live {
        live_flag.set(live);
    }
    if *seen_elapsed.peek() != elapsed_seconds {
        seen_elapsed.set(elapsed_seconds);
        extra.set(0.0);
    }
    let elapsed_now = if live {
        elapsed_seconds.map(|seconds| seconds + extra())
    } else {
        elapsed_seconds
    };
    let span = run_record
        .as_ref()
        .map_or(1.0, |r| axis_span(r, elapsed_now));
    let (tokens_in, tokens_out) = token_totals(&steps);
    let failed = steps.iter().filter(|s| s.has_failed).count();
    let order: Vec<String> = steps.iter().map(|s| s.id.clone()).collect();
    let ticks = axis_ticks(span);
    let now_pct = elapsed_now.map(|e| (e / span * 100.0).clamp(0.0, 100.0));
    let elapsed_label = tick_label(
        elapsed_now
            .or(run_record.as_ref().and_then(|r| r.total_seconds))
            .unwrap_or(0.0),
    );
    let token_text = format!("{tokens_in} in · {tokens_out} out");
    let steps_text = format!("{} · {failed} failed", steps.len());
    let open = selected().and_then(|id| steps.iter().find(|s| s.id == id).cloned());
    let sheet_now = sheet_open();
    let state_key = if live { "live" } else { "finished" };
    let state_label = if live { "Live" } else { "Finished" };
    let key_order = order.clone();
    let announce = if live { "Run live" } else { "Run finished" };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-art",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            "data-sheet": sheet_now.then_some("open"),
            onkeydown: move |event: KeyboardEvent| {
                let delta = match event.key() {
                    Key::ArrowDown => 1,
                    Key::ArrowUp => -1,
                    Key::Character(c) if c == "j" => 1,
                    Key::Character(c) if c == "k" => -1,
                    _ => return,
                };
                let from = focused().or_else(|| selected());
                if let Some(next) = move_index(&key_order, from.as_deref(), delta) {
                    selected.set(Some(next.clone()));
                    focused.set(Some(next.clone()));
                    focus_after_frame(&format!("kit-art-{uid}-{next}"));
                }
            },
            if is_loading {
                div { class: "kit-art__ph kit-art__ph--head", "aria-hidden": "true" }
                div { class: "kit-art__ph kit-art__ph--plan", "aria-hidden": "true" }
            } else if is_empty || run_record.is_none() {
                div { class: "kit-art__empty", p { "This run has no steps yet." } }
            } else if let Some(run) = run_record.clone() {
                {
                    let title = run.entry_title.clone();
                    let agent = format!("{} · {}", run.agent_name, run.model_name);
                    let cost = run.cost_label.clone().unwrap_or_else(|| "—".to_string());
                    let total_text = match run.total_seconds {
                        Some(total) => format!("{:.1} s", total),
                        None => format!("{} s", elapsed_label),
                    };
                    rsx! {
                        header { class: "kit-art__head",
                            div { class: "kit-art__title-group",
                                h2 { class: "kit-art__title", "{title}" }
                                p { class: "kit-art__meta", "{agent}" }
                            }
                            div { class: "kit-art__actions",
                                span { class: "kit-art__state", "data-state": "{state_key}", "{state_label}" }
                                if live {
                                    button {
                                        r#type: "button",
                                        class: "kit-art__danger",
                                        onclick: move |_| {
                                            if let Some(handler) = on_stop {
                                                handler.call(());
                                            }
                                        },
                                        "Stop"
                                    }
                                } else {
                                    button {
                                        r#type: "button",
                                        class: "kit-art__primary",
                                        onclick: move |_| {
                                            if let Some(handler) = on_rerun {
                                                handler.call(());
                                            }
                                        },
                                        "Rerun"
                                    }
                                }
                            }
                        }

                        section { class: "kit-art__tiles", "aria-label": "Run figures",
                            div { class: "kit-art__tile",
                                span { class: "kit-art__label", "Elapsed" }
                                span { class: "kit-art__value", "{total_text}" }
                            }
                            div { class: "kit-art__tile",
                                span { class: "kit-art__label", "Cost" }
                                span { class: "kit-art__value", "{cost}" }
                            }
                            div { class: "kit-art__tile",
                                span { class: "kit-art__label", "Tokens" }
                                span { class: "kit-art__value", "{token_text}" }
                            }
                            div { class: "kit-art__tile",
                                span { class: "kit-art__label", "Steps" }
                                span { class: "kit-art__value", "{steps_text}" }
                            }
                        }

                        div { class: "kit-art__body",
                            div { class: "kit-art__timeline", "aria-label": "Timeline",
                                div { class: "kit-art__axis",
                                    span { class: "kit-art__axis-spacer" }
                                    div { class: "kit-art__ticks",
                                        for tick in ticks.iter().copied() {
                                            {
                                                let pos = format!("left: {:.2}%;", tick / span * 100.0);
                                                let label = tick_label(tick);
                                                let key = format!("t-{}", tick as u64);
                                                rsx! { span { key: "{key}", class: "kit-art__tick", style: "{pos}", "{label}" } }
                                            }
                                        }
                                    }
                                }
                                ul { class: "kit-art__rows", role: "list",
                                    for step in steps.iter().cloned() {
                                        {
                                            let (start_pct, width_pct, running) = bar_geometry(&step, span, elapsed_now);
                                            let bar_style = format!("left: {start_pct:.2}%; width: {width_pct:.2}%;");
                                            let row_id = format!("kit-art-{uid}-{}", step.id);
                                            let row_key = step.id.clone();
                                            let row_label = format!(
                                                "{}, {}, {}, {}",
                                                step_name(&step),
                                                step.step_type.word(),
                                                step.app_name.clone().unwrap_or_else(|| "no app".to_string()),
                                                duration_word(&step, running)
                                            );
                                            let type_key = step.step_type.key();
                                            let is_selected = selected().as_deref() == Some(step.id.as_str());
                                            let indent = if step.nesting_depth > 0 { "kit-art__row kit-art__row--nested" } else { "kit-art__row" };
                                            let bar_state = if step.has_failed { "failed" } else if running { "running" } else { "done" };
                                            let pick_id = step.id.clone();
                                            let focus_id = step.id.clone();
                                            let label_text = step_name(&step);
                                            let sub_text = step.sub_agent_name.clone().unwrap_or_default();
                                            rsx! {
                                                li { key: "{row_key}", class: "{indent}",
                                                    button {
                                                        id: "{row_id}",
                                                        r#type: "button",
                                                        class: "kit-art__hit",
                                                        "aria-label": "{row_label}",
                                                        "aria-current": is_selected.then_some("true"),
                                                        onfocus: move |_| focused.set(Some(focus_id.clone())),
                                                        onclick: move |_| {
                                                            selected.set(Some(pick_id.clone()));
                                                            sheet_open.set(true);
                                                        },
                                                        span { class: "kit-art__label-col",
                                                            span { class: "kit-art__type", "data-type": "{type_key}", "{type_key}" }
                                                            span { class: "kit-art__step-name", "{label_text}" }
                                                            if !sub_text.is_empty() {
                                                                span { class: "kit-art__meta", "{sub_text}" }
                                                            }
                                                        }
                                                        span { class: "kit-art__track",
                                                            span { class: "kit-art__bar", "data-bar": "{bar_state}", style: "{bar_style}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if live {
                                    if let Some(pos) = now_pct {
                                        {
                                            let now_style = format!("left: {pos:.2}%;");
                                            rsx! { span { class: "kit-art__now", "aria-hidden": "true", style: "{now_style}" } }
                                        }
                                    }
                                }
                            }

                            if let Some(step) = open.clone() {
                                {
                                    let (_, _, running) = bar_geometry(&step, span, elapsed_now);
                                    let panel_title = step_name(&step);
                                    let kind = step.step_type.word();
                                    let duration = match (step.duration_seconds, running) {
                                        (Some(seconds), _) => format!("{seconds:.1} s"),
                                        (None, true) => "running".to_string(),
                                        (None, false) => "—".to_string(),
                                    };
                                    let result = if step.has_failed {
                                        Some("Failed")
                                    } else if running {
                                        None
                                    } else {
                                        Some("Done")
                                    };
                                    let back_target = format!("kit-art-{uid}-{}", step.id);
                                    let tokens = step
                                        .token_counts
                                        .map_or_else(|| "—".to_string(), |(i, o)| format!("{i} in · {o} out"));
                                    let cost_text = step.cost_label.clone().unwrap_or_else(|| "—".to_string());
                                    let input = step.input_text.clone().unwrap_or_else(|| "Waiting for input.".to_string());
                                    let output = step.output_text.clone().unwrap_or_else(|| "Pending.".to_string());
                                    let input_copy = step.input_text.clone().unwrap_or_default();
                                    let error = step.error_text.clone().unwrap_or_default();
                                    let has_error = step.has_failed && !error.is_empty();
                                    let input_line = format!("Input\n{input}");
                                    let output_line = format!("Output\n{output}");
                                    let task_target = step.app_name.clone().unwrap_or_default();
                                    rsx! {
                                        aside { class: "kit-art__panel", "aria-label": "Step detail",
                                            button {
                                                r#type: "button",
                                                class: "kit-art__back",
                                                onclick: move |_| {
                                                    sheet_open.set(false);
                                                    focus_after_frame(&back_target);
                                                },
                                                "Back to timeline"
                                            }
                                            h3 { class: "kit-art__panel-title", "{panel_title}" }
                                            dl { class: "kit-art__facts",
                                                dt { "Type" }
                                                dd { "{kind}" }
                                                dt { "Duration" }
                                                dd { "{duration}" }
                                                if let Some(text) = result {
                                                    dt { "Result" }
                                                    dd { "{text}" }
                                                }
                                                dt { "Tokens" }
                                                dd { "{tokens}" }
                                                dt { "Cost" }
                                                dd { "{cost_text}" }
                                            }
                                            if has_error {
                                                p { class: "kit-art__error", role: "alert", "{error}" }
                                            }
                                            section { class: "kit-art__block",
                                                div { class: "kit-art__block-head",
                                                    span { class: "kit-art__label", "Input" }
                                                    button {
                                                        r#type: "button",
                                                        class: "kit-art__ghost",
                                                        onclick: move |_| copy_to_clipboard(&input_copy),
                                                        "Copy input"
                                                    }
                                                }
                                                pre { class: "kit-art__pre", "{input_line}" }
                                            }
                                            section { class: "kit-art__block",
                                                span { class: "kit-art__label", "Output" }
                                                pre { class: "kit-art__pre", "{output_line}" }
                                            }
                                            if !task_target.is_empty() {
                                                button {
                                                    r#type: "button",
                                                    class: "kit-art__link",
                                                    onclick: move |_| {
                                                        if let Some(nav) = on_navigate {
                                                            nav.call("Permissions".to_string());
                                                        }
                                                    },
                                                    "See app access"
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

            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
