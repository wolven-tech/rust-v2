//! Run timeline rows: when each person and agent in a run was reasoning, calling a
//! tool, or waiting on the person, on one shared time axis, with the hand-offs
//! between them.
//!
//! Design choice: time comes from `clock_s` only. Absent, the run is finished and
//! nothing pulses; the component never reads the wall clock, so every position
//! derives from the same number.
//! Design choice: a span with no `end_s` and no clock is not drawn, because its
//! length is unknown.
//! Design choice: step heights are fixed per mode (reasoning low, tool high, waiting
//! mid-lane with a dashed line); the card words carry the same meaning.
//! Browser checks: the axis unit switches with range; arrows and Esc on a focused
//! step; the pulse on running steps; reduced motion stops the pulse; no horizontal
//! overflow at phone width.

use dioxus::prelude::*;

use crate::agents::single_run_status_card::{focus_by_id, format_mm_ss};
use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/run-timeline-rows.css");

/// Height of one party's lane in px; the transfer overlay is positioned from it.
const LANE_PX: f64 = 56.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    Person,
    Agent,
}

impl RowKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Agent => "agent",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RowSpec {
    pub row_key: String,
    pub display_name: String,
    pub kind: RowKind,
    pub tint_key: Option<String>,
    pub mark_source: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanMode {
    Reasoning,
    Tool,
    Waiting,
}

impl SpanMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Reasoning => "reasoning",
            Self::Tool => "tool",
            Self::Waiting => "waiting",
        }
    }

    /// Height of the step above the baseline, as a percentage of the lane.
    fn level_pct(self) -> f64 {
        match self {
            Self::Reasoning => 30.0,
            Self::Tool => 70.0,
            Self::Waiting => 50.0,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Span {
    pub span_key: Option<String>,
    pub row_key: String,
    pub mode: SpanMode,
    pub begin_s: f64,
    pub end_s: Option<f64>,
    pub tool_name: Option<String>,
    pub caption: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Transfer {
    pub transfer_key: Option<String>,
    pub from_row_key: String,
    pub to_row_key: String,
    pub at_s: f64,
    pub caption: Option<String>,
}

/// `(begin, end)` when a span can be drawn: finite times, and an end (given, or the
/// clock for a live run) not before its start.
pub fn span_bounds(span: &Span, clock_s: Option<f64>) -> Option<(f64, f64)> {
    if !span.begin_s.is_finite() {
        return None;
    }
    let end = match (span.end_s, clock_s) {
        (Some(end), _) => end,
        (None, Some(clock)) => clock,
        (None, None) => return None,
    };
    if !end.is_finite() || end < span.begin_s {
        return None;
    }
    Some((span.begin_s, end))
}

/// A span is shown once the clock has passed its start (always, for a finished run).
pub fn span_started(span: &Span, clock_s: Option<f64>) -> bool {
    clock_s.is_none_or(|clock| span.begin_s <= clock)
}

/// A transfer is drawn once the clock passes its time.
pub fn transfer_started(transfer: &Transfer, clock_s: Option<f64>) -> bool {
    clock_s.is_none_or(|clock| transfer.at_s <= clock)
}

/// Axis length: the override, or the latest visible time with a little room.
pub fn axis_length(spans: &[Span], clock_s: Option<f64>, override_s: Option<f64>) -> f64 {
    if let Some(length) = override_s.filter(|l| l.is_finite() && *l > 0.0) {
        return length;
    }
    let latest = spans
        .iter()
        .filter_map(|span| span_bounds(span, clock_s).map(|(_, end)| end))
        .fold(0.0_f64, f64::max);
    (latest * 1.08).max(1.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TickUnit {
    Seconds,
    Minutes,
    Hours,
}

/// Seconds below two minutes, minutes below two hours, hours beyond.
pub fn tick_unit(axis_s: f64) -> TickUnit {
    if axis_s < 120.0 {
        TickUnit::Seconds
    } else if axis_s < 7200.0 {
        TickUnit::Minutes
    } else {
        TickUnit::Hours
    }
}

/// The smallest nice interval that keeps the axis to about eight ticks.
pub fn tick_step(axis_s: f64) -> f64 {
    const STEPS: [f64; 14] = [
        1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 900.0, 1800.0, 3600.0, 7200.0,
    ];
    STEPS
        .into_iter()
        .find(|step| axis_s / step <= 8.0)
        .unwrap_or(7200.0)
}

pub fn format_tick(seconds: f64, unit: TickUnit) -> String {
    let total = seconds.round().max(0.0) as u64;
    match unit {
        TickUnit::Seconds => format!("{total}s"),
        TickUnit::Minutes => format_mm_ss(total * 1000),
        TickUnit::Hours => format!("{}h {:02}m", total / 3600, (total % 3600) / 60),
    }
}

/// Next focused step for an arrow or Home/End key. `positions` holds (row index, begin
/// time) for each drawn step; the result indexes the same slice.
pub fn next_step(positions: &[(usize, f64)], current: usize, key: &str) -> Option<usize> {
    let (row, begin) = *positions.get(current)?;
    let in_row = |target: usize| {
        let mut steps: Vec<usize> = (0..positions.len())
            .filter(|&i| positions[i].0 == target)
            .collect();
        steps.sort_by(|&a, &b| positions[a].1.total_cmp(&positions[b].1));
        steps
    };
    match key {
        "ArrowLeft" | "ArrowRight" => {
            let steps = in_row(row);
            let at = steps.iter().position(|&i| i == current)?;
            if key == "ArrowLeft" {
                at.checked_sub(1).map(|p| steps[p])
            } else {
                steps.get(at + 1).copied()
            }
        }
        "ArrowUp" | "ArrowDown" => {
            let target = if key == "ArrowUp" {
                row.checked_sub(1)?
            } else {
                row + 1
            };
            (0..positions.len())
                .filter(|&i| positions[i].0 == target)
                .min_by(|&a, &b| {
                    (positions[a].1 - begin)
                        .abs()
                        .total_cmp(&(positions[b].1 - begin).abs())
                })
        }
        "Home" => in_row(row).first().copied(),
        "End" => in_row(row).last().copied(),
        _ => None,
    }
}

/// Words for what a row is doing now, from its running step.
pub fn row_note(running: Option<(SpanMode, Option<&str>)>) -> String {
    match running {
        Some((SpanMode::Reasoning, _)) => "thinking".to_string(),
        Some((SpanMode::Tool, Some(tool))) => format!("calling {tool}"),
        Some((SpanMode::Tool, None)) => "calling a tool".to_string(),
        Some((SpanMode::Waiting, _)) => "waiting on you".to_string(),
        None => "idle".to_string(),
    }
}

/// Accessible name of a step: who, what, when and how long.
pub fn describe_span(row_name: &str, span: &Span, end_s: f64) -> String {
    let what = match (span.mode, span.tool_name.as_deref()) {
        (SpanMode::Tool, Some(tool)) => format!("calling {tool}"),
        (mode, _) => mode.as_str().to_string(),
    };
    let duration = end_s - span.begin_s;
    format!(
        "{row_name}, {what}, from {} for {duration:.1} s",
        format_mm_ss((span.begin_s * 1000.0) as u64)
    )
}

#[derive(Clone, PartialEq, Debug)]
struct StepView {
    index: usize,
    pos: usize,
    row: usize,
    dom_id: String,
    aria: String,
    style: String,
    mode: &'static str,
    running: bool,
    tool: Option<String>,
    tab: &'static str,
}

#[component]
pub fn RunTimelineRows(
    run_heading: String,
    rows: Vec<RowSpec>,
    #[props(default)] spans: Vec<Span>,
    #[props(default)] transfers: Vec<Transfer>,
    #[props(default)] clock_s: Option<f64>,
    #[props(default)] axis_length_s: Option<f64>,
    #[props(default)] class_name: String,
) -> Element {
    let id = use_hook(|| format!("kit-rtr-{}", next_id()));
    let mut open = use_signal(|| None::<usize>);
    let mut focused = use_signal(|| None::<usize>);

    let axis = axis_length(&spans, clock_s, axis_length_s);
    let unit = tick_unit(axis);
    let step = tick_step(axis);
    let ticks: Vec<(String, String)> = (0..)
        .map(|k| k as f64 * step)
        .take_while(|t| *t <= axis)
        .map(|t| (format!("{:.4}%", t / axis * 100.0), format_tick(t, unit)))
        .collect();

    let row_of = |key: &str| rows.iter().position(|r| r.row_key == key);
    let drawn: Vec<(usize, usize, (f64, f64))> = spans
        .iter()
        .enumerate()
        .filter(|(_, span)| span_started(span, clock_s))
        .filter_map(|(i, span)| {
            let row = row_of(&span.row_key)?;
            span_bounds(span, clock_s).map(|bounds| (i, row, bounds))
        })
        .collect();
    let positions: Vec<(usize, f64)> = drawn
        .iter()
        .map(|&(i, row, _)| (row, spans[i].begin_s))
        .collect();

    let steps: Vec<StepView> = drawn
        .iter()
        .enumerate()
        .map(|(pos, &(index, row, (begin, end)))| {
            let span = &spans[index];
            let left = begin / axis * 100.0;
            let width = ((end - begin) / axis * 100.0).max(0.4);
            StepView {
                index,
                pos,
                row,
                dom_id: format!("{id}-s{index}"),
                aria: describe_span(&rows[row].display_name, span, end),
                style: format!(
                    "left: {left:.4}%; width: {width:.4}%; bottom: {:.4}%",
                    span.mode.level_pct() * 0.5
                ),
                mode: span.mode.as_str(),
                running: span.end_s.is_none() && clock_s.is_some(),
                tool: span.tool_name.clone(),
                tab: if focused() == Some(pos) || (focused().is_none() && pos == 0) {
                    "0"
                } else {
                    "-1"
                },
            }
        })
        .collect();

    let lanes: Vec<(String, String, String, String, &'static str, Vec<StepView>)> = rows
        .iter()
        .enumerate()
        .map(|(r, row)| {
            let lane_steps: Vec<StepView> = steps.iter().filter(|s| s.row == r).cloned().collect();
            let running = lane_steps.iter().find(|s| s.running).map(|s| {
                let span = &spans[s.index];
                (span.mode, span.tool_name.as_deref())
            });
            let count = format!("{} steps", lane_steps.len());
            (
                row.row_key.clone(),
                row.display_name.clone(),
                row_note(running),
                count,
                row.kind.as_str(),
                lane_steps,
            )
        })
        .collect();

    let waiting_now = drawn.iter().any(|&(i, _, _)| {
        let span = &spans[i];
        span.mode == SpanMode::Waiting
            && clock_s.is_some_and(|clock| span.end_s.is_none_or(|end| end > clock))
    });
    let transfer_views: Vec<(String, String, String)> = transfers
        .iter()
        .filter(|t| transfer_started(t, clock_s))
        .filter_map(|t| {
            let from = row_of(&t.from_row_key)?;
            let to = row_of(&t.to_row_key)?;
            let top = from.min(to) as f64 * LANE_PX + LANE_PX / 2.0;
            let height = from.abs_diff(to) as f64 * LANE_PX;
            let left = t.at_s / axis * 100.0;
            let key = t
                .transfer_key
                .clone()
                .unwrap_or_else(|| format!("{}-{}-{}", t.from_row_key, t.to_row_key, t.at_s));
            let label = t.caption.clone().unwrap_or_else(|| {
                format!(
                    "Hand-off from {} to {}",
                    rows[from].display_name, rows[to].display_name
                )
            });
            Some((
                key,
                format!("left: {left:.4}%; top: {top:.1}px; height: {height:.1}px"),
                label,
            ))
        })
        .collect();

    let detail = open()
        .and_then(|index| spans.get(index).map(|span| (index, span)))
        .map(|(_, span)| {
            let row_name = row_of(&span.row_key)
                .map(|r| rows[r].display_name.clone())
                .unwrap_or_default();
            let end = span_bounds(span, clock_s)
                .map(|(_, e)| e)
                .unwrap_or(span.begin_s);
            (
                describe_span(&row_name, span, end),
                span.caption.clone().unwrap_or_default(),
            )
        });
    let bar_count = format!("{} rows · {} steps", rows.len(), drawn.len());
    let heading_id = format!("{id}-heading");
    let plot_id = format!("{id}-plot");
    let class = format!("kit-rtr {class_name}");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{heading_id}",
            header { class: "kit-rtr__bar",
                h3 { id: "{heading_id}", class: "kit-rtr__heading", {run_heading} }
                span { class: "kit-rtr__count", {bar_count} }
                span { class: "kit-rtr__sr", role: "status", "aria-live": "polite", if waiting_now { "Waiting on you" } }
                if waiting_now {
                    span { class: "kit-rtr__waiting", "Waiting on you" }
                }
            }
            div {
                id: "{plot_id}",
                class: "kit-rtr__plot",
                role: "group",
                "aria-label": "Steps by party",
                onkeydown: move |event: KeyboardEvent| {
                    let key = event.key();
                    if key == Key::Escape {
                        open.set(None);
                        return;
                    }
                    let name = match key {
                        Key::ArrowLeft => "ArrowLeft",
                        Key::ArrowRight => "ArrowRight",
                        Key::ArrowUp => "ArrowUp",
                        Key::ArrowDown => "ArrowDown",
                        Key::Home => "Home",
                        Key::End => "End",
                        _ => return,
                    };
                    let current = focused().unwrap_or(0);
                    if let Some(next) = next_step(&positions, current, name) {
                        event.prevent_default();
                        focused.set(Some(next));
                        open.set(Some(drawn[next].0));
                        focus_by_id(&format!("{id}-s{}", drawn[next].0));
                    }
                },
                div { class: "kit-rtr__axis", "aria-hidden": "true",
                    for (pos, label) in ticks {
                        span { key: "{pos}", class: "kit-rtr__tick", style: "left: {pos}", {label} }
                    }
                }
                div { class: "kit-rtr__rows",
                    for (key, name, note, count, kind, lane_steps) in lanes {
                        div { key: "{key}", class: "kit-rtr__row", "data-kind": kind,
                            div { class: "kit-rtr__name",
                                span { class: "kit-rtr__who", {name} }
                                span { class: "kit-rtr__note", {note} }
                                span { class: "kit-rtr__counts", {count} }
                            }
                            div { class: "kit-rtr__lane",
                                span { class: "kit-rtr__baseline", "aria-hidden": "true" }
                                for (skey, step) in lane_steps.into_iter().map(|s| (format!("{}", s.index), s)) {
                                    button {
                                        key: "{skey}",
                                        id: step.dom_id.clone(),
                                        class: "kit-rtr__step",
                                        r#type: "button",
                                        style: step.style.clone(),
                                        "data-mode": step.mode,
                                        "data-running": step.running.then_some("true"),
                                        "aria-label": step.aria.clone(),
                                        tabindex: step.tab,
                                        onfocus: move |_| {
                                            focused.set(Some(step.pos));
                                            open.set(Some(step.index));
                                        },
                                        onclick: move |_| {
                                            open.set(if open() == Some(step.index) { None } else { Some(step.index) });
                                        },
                                        if let Some(tool) = step.tool.clone() {
                                            span { class: "kit-rtr__tool", {tool} }
                                        }
                                        if step.mode == "waiting" {
                                            span { class: "kit-rtr__wait", "Waiting on you" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-rtr__overlay", "aria-hidden": "true",
                        for (key, style, label) in transfer_views {
                            span { key: "{key}", class: "kit-rtr__transfer", style: style, "aria-label": label }
                        }
                    }
                }
            }
            if let Some((text, caption)) = detail {
                div { class: "kit-rtr__card", role: "dialog", "aria-label": "Step details",
                    p { class: "kit-rtr__card-text", {text} }
                    if !caption.is_empty() {
                        p { class: "kit-rtr__card-caption", {caption} }
                    }
                }
            }
        }
    }
}
