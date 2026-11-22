//! Schedule variance timeline: an agent run against its plan, step by step.
//! Each row carries a planned outline and an actual fill on one shared time axis,
//! with the variance in words and signed figures.
//!
//! Design choice: times are offsets in one shared unit. `format_time` and
//! `time_unit` say how that unit reads; callers convert their own timestamps.
//! Design choice: a step's expected start is the latest expected end of its
//! dependencies, or zero when it has none; an unknown dependency key is ignored.
//! Design choice: dependency links are drawn only for the hovered, focused or
//! selected row, so nothing is linked at rest.
//! Design choice: the chart area is `aria-hidden`; each row carries its state
//! and variance in its accessible name.
//! Browser checks: the glide of later rows when an earlier step overruns, the
//! moving highlight on running bars, and the axis reduced to its end values at
//! narrow width.

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
            Self::Waiting => "Waiting",
            Self::Running => "Running",
            Self::Done => "Done",
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

/// `two minutes behind plan`, `one second ahead of plan`, `on time`.
pub fn variance_words(variance: f64, unit: &str) -> String {
    let rounded = variance.round() as i64;
    if rounded == 0 {
        return "on time".to_string();
    }
    let amount = rounded.unsigned_abs();
    let noun = if amount == 1 {
        unit.to_string()
    } else {
        format!("{unit}s")
    };
    let count = count_word(amount);
    if rounded > 0 {
        format!("{count} {noun} behind plan")
    } else {
        format!("{count} {noun} ahead of plan")
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

/// Signed duration in the axis format: `+2 min`, `−0:30`, `0 min`.
pub fn signed_time(variance: f64, format: &dyn Fn(f64) -> String) -> String {
    let rounded = variance.round();
    if rounded > 0.0 {
        format!("+{}", format(rounded))
    } else if rounded < 0.0 {
        format!("−{}", format(-rounded))
    } else {
        format(0.0)
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

/// A round step, from 1, 2, 5 or 10 times a power of ten, giving at most five
/// intervals across `span`.
pub fn tick_step(span: f64) -> f64 {
    let raw = (sane(span) / 5.0).max(f64::MIN_POSITIVE);
    let magnitude = 10f64.powf(raw.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .iter()
        .map(|m| m * magnitude)
        .find(|step| *step >= raw)
        .unwrap_or(10.0 * magnitude)
}

fn toggle_selection(mut selected: Signal<Option<String>>, key: String) {
    if selected.peek().as_deref() == Some(key.as_str()) {
        selected.set(None);
    } else {
        selected.set(Some(key));
    }
}

const ROW_HEIGHT: f64 = 44.0;
const ROW_GAP: f64 = 4.0;
const ROW_PITCH: f64 = ROW_HEIGHT + ROW_GAP;
const GUTTER_PX: f64 = 14.0;
const AXIS_HEIGHT: f64 = 52.0;

fn row_centre(index: usize) -> f64 {
    index as f64 * ROW_PITCH + ROW_HEIGHT / 2.0
}

fn edge_path(from: usize, to: usize) -> String {
    let (y1, y2) = (row_centre(from), row_centre(to));
    format!("M {GUTTER_PX} {y1:.1} H 4 V {y2:.1} H {GUTTER_PX}")
}

/// Plan against actual for a run, row by row.
#[component]
pub fn ScheduleVarianceTimeline(
    #[props(default)] step_list: Vec<PlanStep>,
    #[props(default)] clock_point: f64,
    #[props(default)] run_purpose: Option<String>,
    #[props(default)] format_time: Option<Callback<f64, String>>,
    #[props(default = "second".to_string(), into)] time_unit: String,
    #[props(default = "Plan vs actual".to_string(), into)] board_title: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-variance-{}", next_id()));
    let mut selected = use_signal(|| Option::<String>::None);
    let mut focused = use_signal(|| Option::<String>::None);
    let mut hovered = use_signal(|| Option::<String>::None);
    let mut announce = use_signal(String::new);
    let mut last_states = use_signal(Vec::<(String, StepState)>::new);

    let rows = project(&step_list, clock_point);
    let planned = plan(&step_list);
    let planned_finish = planned.iter().map(|(_, e)| *e).fold(0.0_f64, f64::max);
    let expected_finish = rows.iter().map(|r| r.expected_end).fold(0.0_f64, f64::max);
    let finish_delta = expected_finish - planned_finish;
    let axis_max = rows
        .iter()
        .flat_map(|r| [r.bar_end, r.expected_end])
        .chain([planned_finish, sane(clock_point), 1.0])
        .fold(1.0_f64, f64::max);
    let fmt = |v: f64| match format_time {
        Some(cb) => cb.call(v),
        None => default_format(v),
    };
    let pct = |v: f64| v / axis_max * 100.0;
    let summary = format!(
        "Elapsed {} against planned finish {}; estimated finish {} ({} vs plan)",
        fmt(sane(clock_point)),
        fmt(planned_finish),
        fmt(expected_finish),
        signed_time(finish_delta, &fmt)
    );

    let step_size = tick_step(axis_max);
    let last_tick = (axis_max / step_size).floor() as usize;
    let ticks: Vec<(String, String, bool)> = (0..=last_tick)
        .map(|k| {
            let t = k as f64 * step_size;
            (
                format!("left: {:.3}%;", pct(t)),
                fmt(t),
                k == 0 || k == last_tick,
            )
        })
        .collect();
    let finish_marks: Vec<(String, String, &'static str)> = if finish_delta.round() == 0.0 {
        vec![(
            format!("left: {:.3}%;", pct(planned_finish)),
            format!("Finish {}", fmt(planned_finish)),
            "finish",
        )]
    } else {
        vec![
            (
                format!("left: {:.3}%;", pct(planned_finish)),
                format!("Plan {}", fmt(planned_finish)),
                "plan",
            ),
            (
                format!("left: {:.3}%;", pct(expected_finish)),
                format!("Expected {}", fmt(expected_finish)),
                "expected",
            ),
        ]
    };

    let selected_now = selected();
    let focused_now = focused();
    let shown_now = hovered()
        .or_else(|| focused_now.clone())
        .or_else(|| selected_now.clone());
    let links: Vec<(String, String)> = match &shown_now {
        Some(key) => links_for(&step_list, key),
        None => Vec::new(),
    };
    let edge_paths: Vec<String> = match &shown_now {
        None => Vec::new(),
        Some(key) => {
            let mut paths = Vec::new();
            for (to, step) in step_list.iter().enumerate() {
                for dep in &step.waits_for {
                    if dep != key && &step.step_key != key {
                        continue;
                    }
                    if let Some(from) = step_list.iter().position(|p| &p.step_key == dep) {
                        paths.push(edge_path(from, to));
                    }
                }
            }
            paths
        }
    };
    let links_height = (step_list.len() as f64 * ROW_PITCH - ROW_GAP).max(0.0);
    let announcement = announce();
    let empty = step_list.is_empty();
    let purpose = run_purpose.clone().unwrap_or_default();
    let title_text = board_title.clone();
    let root_cls = class_list("kit-variance", &root_class);

    let states: Vec<(String, StepState)> =
        rows.iter().map(|r| (r.step_key.clone(), r.state)).collect();
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
                    svg { class: "kit-variance__links", "aria-hidden": "true", width: "{GUTTER_PX}", height: "{links_height}", style: "top: {AXIS_HEIGHT}px;",
                        for (i, d) in edge_paths.iter().enumerate() {
                            path { key: "{i}", class: "kit-variance__edge", d: "{d}" }
                        }
                    }
                    div { class: "kit-variance__axis", "aria-hidden": "true", style: "height: {AXIS_HEIGHT}px;",
                        span { class: "kit-variance__pad" }
                        span { class: "kit-variance__pad" }
                        div { class: "kit-variance__scale",
                            for (style, text, edge) in ticks.iter().cloned() {
                                span { key: "{style}", class: "kit-variance__tick", "data-edge": edge.then_some("true"), style: "{style}", "{text}" }
                            }
                            for (style, text, kind) in finish_marks.iter().cloned() {
                                span { key: "{kind}", class: "kit-variance__finish", "data-kind": kind, style: "{style}", "{text}" }
                            }
                        }
                    }
                    ol { class: "kit-variance__rows",
                        for (index, row) in rows.iter().enumerate() {
                            {
                                let step = step_list[index].clone();
                                let key = row.step_key.clone();
                                let dom = format!("{base}-row-{index}");
                                let projected = row.expected_end - row.planned_end;
                                let variance_value = match row.state {
                                    StepState::Waiting => projected,
                                    _ => row.variance,
                                };
                                let words = match row.state {
                                    StepState::Waiting => format!("projected {}", variance_words(projected, &time_unit)),
                                    _ => variance_words(row.variance, &time_unit),
                                };
                                let name = format!("{}, {}, {}", step.step_label, row.state.word(), words);
                                let duration = match row.state {
                                    StepState::Waiting => row.planned_end - row.planned_start,
                                    _ => row.bar_end - row.start,
                                };
                                let duration_text = fmt(duration.max(0.0));
                                let figure_text = signed_time(variance_value, &fmt);
                                let planned_style = format!(
                                    "left: {:.3}%; width: {:.3}%;",
                                    pct(row.planned_start),
                                    pct((row.planned_end - row.planned_start).max(0.0))
                                );
                                let actual_style = format!(
                                    "left: {:.3}%; width: {:.3}%;",
                                    pct(row.start),
                                    pct((row.bar_end - row.start).max(0.0))
                                );
                                let late = row.state != StepState::Waiting && row.variance > 0.5;
                                let row_class = if late { "kit-variance__row is-late" } else { "kit-variance__row" };
                                let is_selected = selected_now.as_deref() == Some(key.as_str());
                                let selected_attr = if is_selected { "true" } else { "false" };
                                let link_dom = links.iter().any(|(a, b)| a == &key || b == &key);
                                let link_attr = if link_dom { "true" } else { "false" };
                                let mark = step.step_mark.clone().unwrap_or_default();
                                let state_word = row.state.word();
                                let shown_key = format!("{dom}-{}", row.step_key);
                                let focus_key = key.clone();
                                let hover_key = key.clone();
                                let click_key = key.clone();
                                let enter_key = key.clone();
                                rsx! {
                                    li {
                                        key: "{shown_key}",
                                        id: "{dom}",
                                        class: row_class,
                                        tabindex: "0",
                                        "aria-label": "{name}",
                                        "data-state": state_word.to_ascii_lowercase(),
                                        "data-selected": selected_attr,
                                        "data-linked": link_attr,
                                        onfocus: move |_| focused.set(Some(focus_key.clone())),
                                        onblur: move |_| focused.set(None),
                                        onmouseenter: move |_| hovered.set(Some(hover_key.clone())),
                                        onmouseleave: move |_| hovered.set(None),
                                        onclick: move |_| toggle_selection(selected, click_key.clone()),
                                        onkeydown: move |event: KeyboardEvent| {
                                            if matches!(event.key(), Key::Enter) {
                                                event.prevent_default();
                                                toggle_selection(selected, enter_key.clone());
                                            }
                                        },
                                        span { class: "kit-variance__mark", "aria-hidden": "true", "{mark}" }
                                        span { class: "kit-variance__label", "{step.step_label}" }
                                        span { class: "kit-variance__track", "aria-hidden": "true",
                                            span { class: "kit-variance__planned", style: "{planned_style}" }
                                            span { class: "kit-variance__actual", style: "{actual_style}" }
                                        }
                                        span { class: "kit-variance__duration", "{duration_text}" }
                                        span { class: "kit-variance__figure", "{figure_text}" }
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
