//! Schedule variance timeline: an agent run against its plan, step by step.
//! Each step shows its planned span, its actual span, and the variance in words
//! and signed figures.
//!
//! Design choice: times are offsets in one shared unit (seconds in the default
//! formatter); callers convert their own timestamps.
//! Design choice: a step's expected start is the latest expected end of its
//! dependencies, or zero when it has none; an unknown dependency key is ignored.
//! Design choice: the chart area is `aria-hidden`; each row carries its state
//! and variance in its accessible name.
//! Browser checks: the glide of later rows when an earlier step overruns, the
//! moving highlight on running bars, and the hidden purpose text at narrow width.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/schedule-variance-timeline.css");

#[derive(Clone, PartialEq, Debug)]
pub struct PlanStep {
    pub step_key: String,
    pub step_label: String,
    pub planned_span: f64,
    pub waits_for: Vec<String>,
    pub actual_start: Option<f64>,
    pub actual_end: Option<f64>,
    pub step_mark: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepState {
    Waiting,
    Running,
    Done,
}

impl StepState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Running => "running",
            Self::Done => "done",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Scheduled {
    pub step_key: String,
    pub state: StepState,
    pub start: f64,
    /// Actual end when done; the running bar's current end otherwise.
    pub bar_end: f64,
    pub planned_start: f64,
    pub planned_end: f64,
    /// Actual minus planned, in the run's unit. Positive is late.
    pub variance: f64,
    pub expected_end: f64,
}

fn sane(value: f64) -> f64 {
    if value.is_finite() { value } else { 0.0 }
}

/// Planned start and end for each step, from dependencies alone.
pub fn plan(steps: &[PlanStep]) -> Vec<(f64, f64)> {
    let mut ends: Vec<(String, f64)> = Vec::new();
    let mut out = Vec::new();
    for step in steps {
        let start = step
            .waits_for
            .iter()
            .filter_map(|k| ends.iter().find(|(key, _)| key == k).map(|(_, e)| *e))
            .fold(0.0_f64, f64::max);
        let end = start + sane(step.planned_span).max(0.0);
        ends.push((step.step_key.clone(), end));
        out.push((start, end));
    }
    out
}

/// Projects every step at run position `clock`.
pub fn project(steps: &[PlanStep], clock: f64) -> Vec<Scheduled> {
    let planned = plan(steps);
    let mut expected: Vec<(String, f64)> = Vec::new();
    let mut rows = Vec::new();
    for (step, (planned_start, planned_end)) in steps.iter().zip(planned.iter().copied()) {
        let dep_end = step
            .waits_for
            .iter()
            .filter_map(|k| expected.iter().find(|(key, _)| key == k).map(|(_, e)| *e))
            .fold(0.0_f64, f64::max);
        let planned_span = sane(step.planned_span).max(0.0);
        let (state, start, bar_end, expected_end) = match (step.actual_start, step.actual_end) {
            (Some(s), Some(e)) => (StepState::Done, s, e, e),
            (Some(s), None) => {
                let start = sane(s);
                let grown = (sane(clock) - start).max(0.0);
                (
                    StepState::Running,
                    start,
                    start + grown,
                    (start + planned_span).max(sane(clock)),
                )
            }
            (None, _) => {
                let start = dep_end;
                let state = if sane(clock) > start {
                    StepState::Running
                } else {
                    StepState::Waiting
                };
                let grown = if state == StepState::Running {
                    (sane(clock) - start).max(0.0)
                } else {
                    0.0
                };
                (
                    state,
                    start,
                    start + grown,
                    (start + planned_span).max(sane(clock)),
                )
            }
        };
        let actual_span = match state {
            StepState::Done => bar_end - start,
            StepState::Running => bar_end - start,
            StepState::Waiting => 0.0,
        };
        let variance = match state {
            StepState::Waiting => 0.0,
            StepState::Done => actual_span - planned_span,
            StepState::Running => expected_end - (start + planned_span),
        };
        expected.push((step.step_key.clone(), expected_end));
        rows.push(Scheduled {
            step_key: step.step_key.clone(),
            state,
            start,
            bar_end,
            planned_start,
            planned_end,
            variance,
            expected_end,
        });
    }
    rows
}

/// `two seconds behind plan`, `one second ahead of plan`, `on time`.
pub fn variance_words(variance: f64) -> String {
    let rounded = variance.round() as i64;
    if rounded == 0 {
        return "on time".to_string();
    }
    let amount = rounded.unsigned_abs();
    let unit = if amount == 1 { "second" } else { "seconds" };
    let count = count_word(amount);
    if rounded > 0 {
        format!("{count} {unit} behind plan")
    } else {
        format!("{count} {unit} ahead of plan")
    }
}

fn count_word(n: u64) -> String {
    const WORDS: [&str; 11] = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    ];
    WORDS
        .get(n as usize)
        .map(|w| w.to_string())
        .unwrap_or_else(|| n.to_string())
}

/// Signed figure for a variance, such as `+2` or `−1`.
pub fn signed(variance: f64) -> String {
    let rounded = variance.round() as i64;
    if rounded > 0 {
        format!("+{rounded}")
    } else if rounded < 0 {
        format!("−{}", rounded.unsigned_abs())
    } else {
        "0".to_string()
    }
}

/// Dependency edges touching `key`, as `(from, to)` pairs.
pub fn links_for(steps: &[PlanStep], key: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for step in steps {
        for dep in &step.waits_for {
            if dep == key || step.step_key == key {
                out.push((dep.clone(), step.step_key.clone()));
            }
        }
    }
    out
}

/// Default formatter: seconds as `m:ss`.
pub fn default_format(seconds: f64) -> String {
    let total = sane(seconds).max(0.0).round() as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

fn focus_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

const ROW_HEIGHT: f64 = 44.0;
const ROW_GAP: f64 = 4.0;
const ROW_PITCH: f64 = ROW_HEIGHT + ROW_GAP;
const GUTTER_PX: f64 = 14.0;

fn row_centre(index: usize) -> f64 {
    index as f64 * ROW_PITCH + ROW_HEIGHT / 2.0
}

/// Plan against actual for a run, row by row.
#[component]
pub fn ScheduleVarianceTimeline(
    #[props(default)] step_list: Vec<PlanStep>,
    #[props(default)] clock_point: f64,
    #[props(default)] run_purpose: Option<String>,
    #[props(default)] format_time: Option<Callback<f64, String>>,
    #[props(default = "Plan vs actual".to_string(), into)] board_title: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-variance-{}", next_id()));
    let mut selected = use_signal(|| Option::<String>::None);
    let mut hovered = use_signal(|| Option::<String>::None);
    let mut announce = use_signal(String::new);
    let mut last_states = use_signal(Vec::<(String, StepState)>::new);

    let rows = project(&step_list, clock_point);
    let planned = plan(&step_list);
    let planned_finish = planned.iter().map(|(_, e)| *e).fold(0.0_f64, f64::max);
    let expected_finish = rows.iter().map(|r| r.expected_end).fold(0.0_f64, f64::max);
    let finish_delta = expected_finish - planned_finish;
    let fmt = |v: f64| match format_time {
        Some(cb) => cb.call(v),
        None => default_format(v),
    };
    let summary = format!(
        "Elapsed {} against planned finish {}; estimated finish {} ({} vs plan)",
        fmt(sane(clock_point)),
        fmt(planned_finish),
        fmt(expected_finish),
        signed(finish_delta)
    );
    let purpose = run_purpose.clone().unwrap_or_default();
    let title_text = board_title.clone();
    let root_cls = class_list("kit-variance", &root_class);
    let selected_now = selected();
    let states: Vec<(String, StepState)> =
        rows.iter().map(|r| (r.step_key.clone(), r.state)).collect();
    let shown_now = hovered().or_else(|| selected_now.clone());
    let links: Vec<(String, String)> = match &shown_now {
        Some(key) => links_for(&step_list, key),
        None => Vec::new(),
    };
    let edge_paths: Vec<(String, bool)> = step_list
        .iter()
        .enumerate()
        .flat_map(|(to, step)| {
            let shown = shown_now.clone();
            let to_key = step.step_key.clone();
            step.waits_for
                .iter()
                .filter_map(|dep| {
                    step_list
                        .iter()
                        .position(|p| &p.step_key == dep)
                        .map(|from| (from, dep.clone()))
                })
                .map(move |(from, dep)| {
                    let lit = shown.as_deref().is_some_and(|k| k == to_key || k == dep);
                    let (y1, y2) = (row_centre(from), row_centre(to));
                    (
                        format!("M {GUTTER_PX} {y1:.1} H 4 V {y2:.1} H {GUTTER_PX}"),
                        lit,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let links_height = (step_list.len() as f64 * ROW_PITCH - ROW_GAP).max(0.0);
    let announcement = announce();
    let empty = step_list.is_empty();

    let states_for_effect = states.clone();
    use_effect(use_reactive((&states_for_effect,), move |(now,)| {
        let before = last_states.peek().clone();
        let mut messages = Vec::new();
        for (key, state) in &now {
            let previous = before.iter().find(|(k, _)| k == key).map(|(_, s)| *s);
            if previous.is_some_and(|p| p != *state) && *state == StepState::Done {
                messages.push(format!("{key} finished"));
            }
        }
        if !messages.is_empty() {
            announce.set(messages.join(". "));
        }
        last_states.set(now);
    }));

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: root_cls, "aria-label": "{title_text}",
            onkeydown: move |event: KeyboardEvent| {
                if matches!(event.key(), Key::Escape) {
                    selected.set(None);
                }
            },
            header { class: "kit-variance__head",
                h2 { class: "kit-variance__title", "{title_text}" }
                if !purpose.is_empty() {
                    p { class: "kit-variance__purpose", "{purpose}" }
                }
                p { class: "kit-variance__summary", "{summary}" }
            }
            if empty {
                p { class: "kit-variance__empty", "No steps planned" }
            } else {
                div { class: "kit-variance__body",
                svg { class: "kit-variance__links", "aria-hidden": "true", width: "{GUTTER_PX}", height: "{links_height}",
                    for (i, (d, lit)) in edge_paths.iter().cloned().enumerate() {
                        path { key: "{i}", class: "kit-variance__edge", "data-lit": lit.then_some("true"), d: "{d}" }
                    }
                }
                ol { class: "kit-variance__rows", "aria-hidden": "false",
                    for (index, row) in rows.iter().enumerate() {
                        {
                            let step = step_list[index].clone();
                            let key = row.step_key.clone();
                            let dom = format!("{base}-row-{index}");
                            let planned_left = row.planned_start;
                            let planned_width = (row.planned_end - row.planned_start).max(0.0);
                            let actual_width = (row.bar_end - row.start).max(0.0);
                            let grid_scale = planned.iter().map(|(_, e)| *e).fold(1.0_f64, f64::max);
                            let planned_style = format!("left: {:.3}%; width: {:.3}%;", planned_left / grid_scale * 100.0, planned_width / grid_scale * 100.0);
                            let actual_style = format!("left: {:.3}%; width: {:.3}%;", row.start / grid_scale * 100.0, actual_width / grid_scale * 100.0);
                            let words = match row.state {
                                StepState::Waiting => "waiting for dependencies".to_string(),
                                _ => variance_words(row.variance),
                            };
                            let name = format!("{}, {}, {}", step.step_label, row.state.word(), words);
                            let state_word = row.state.word();
                            let variance_text = match row.state {
                                StepState::Waiting => String::new(),
                                _ => signed(row.variance),
                            };
                            let late = row.variance > 0.5 && row.state != StepState::Waiting;
                            let row_class = if late { "kit-variance__row is-late" } else { "kit-variance__row" };
                            let is_selected = selected_now.as_deref() == Some(key.as_str());
                            let selected_attr = if is_selected { "true" } else { "false" };
                            let focus_key = key.clone();
                            let click_key = key.clone();
                            let hover_key = key.clone();
                            let mark = step.step_mark.clone().unwrap_or_default();
                            let link_dom = links.iter().any(|(a, b)| a == &key || b == &key);
                            let link_attr = if link_dom { "true" } else { "false" };
                            let shown_key = format!("{dom}-{}", row.step_key);
                            rsx! {
                                li {
                                    key: "{shown_key}",
                                    id: "{dom}",
                                    class: row_class,
                                    tabindex: "0",
                                    role: "listitem",
                                    "aria-label": "{name}",
                                    "aria-selected": "{selected_attr}",
                                    "data-state": state_word,
                                    "data-linked": "{link_attr}",
                                    onfocus: move |_| selected.set(Some(focus_key.clone())),
                                    onmouseenter: move |_| hovered.set(Some(hover_key.clone())),
                                    onmouseleave: move |_| hovered.set(None),
                                    onclick: move |_| {
                                        if selected().as_deref() == Some(click_key.as_str()) {
                                            selected.set(None);
                                        } else {
                                            selected.set(Some(click_key.clone()));
                                        }
                                    },
                                    onkeydown: move |event: KeyboardEvent| {
                                        if matches!(event.key(), Key::Enter) {
                                            event.prevent_default();
                                            focus_id(&dom.to_string());
                                        }
                                    },
                                    span { class: "kit-variance__mark", "aria-hidden": "true", "{mark}" }
                                    span { class: "kit-variance__label", "{step.step_label}" }
                                    span { class: "kit-variance__track", "aria-hidden": "true",
                                        span { class: "kit-variance__planned", style: "{planned_style}" }
                                        span { class: "kit-variance__actual", style: "{actual_style}" }
                                    }
                                    span { class: "kit-variance__figure", "{variance_text}" }
                                    span { class: "kit-variance__words", "{words}" }
                                    span { class: "kit-variance__state", "{state_word}" }
                                }
                            }
                        }
                    }
                }
                }
            }
            p { class: "kit-variance__sr", role: "status", "aria-live": "polite", "{announcement}" }
        }
    }
}
