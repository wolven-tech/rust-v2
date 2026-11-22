//! Quota load scheduler: places scheduled agent runs across a day so that their
//! combined tokens per minute stay under a provider ceiling. Each run's load is
//! a curve; the curves are summed, the peak is marked, and runs can be moved,
//! spread out, edited or removed.
//!
//! Design choice: the day is 1440 one-minute samples; the chart is drawn from
//! `window_start_hour`, wrapping past midnight. The run rows sit in the same
//! columns as the chart, so a bar and its curve share one hour axis.
//! Design choice: `curve_emphasis` has two values; a focused run is drawn strong
//! regardless of its own emphasis.
//! Design choice: "spread out" moves each unlocked run, in turn, to the start
//! (on a 15-minute grid) that lowers the day's peak most; it never raises it.
//! Design choice: a run counts as active at a deadline while it is within
//! twice its tail half-life after its peak.
//! Design choice: drag measures the track width once, on pointer down.
//! Design choice: each run's number is shown beside its curve and in its row, so
//! runs are told apart without colour.
//! Browser checks: the curve easing after moves, the hatched overflow, the
//! drag tracking, the settings dialog focus, and the stacked rows below a
//! moderate width.

use dioxus::prelude::*;

use crate::components::agent_context_map::focus_on_mount;
use crate::internal::{KIT_CSS, class_list, focus_after_frame, next_id};

const CSS: Asset = asset!("/assets/quota-load-scheduler.css");

const DAY: usize = 1440;
const STEP: usize = 15;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RunDays {
    #[default]
    Daily,
    Weekdays,
    Weekend,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CurveEmphasis {
    #[default]
    Normal,
    Strong,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DayKind {
    #[default]
    Weekdays,
    Weekend,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoadStatus {
    Under,
    Close,
    Over,
}

impl LoadStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Under => "Under limit",
            Self::Close => "Close to limit",
            Self::Over => "Over limit",
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct PlannedRun {
    pub run_id: String,
    pub run_name: String,
    /// Minutes after midnight.
    pub start_minute: f64,
    pub peak_tokens_per_min: f64,
    /// Minutes from start to peak.
    pub rise_minutes: f64,
    /// Minutes for the tail to fall to half.
    pub tail_half_life_minutes: f64,
    pub run_days: RunDays,
    pub curve_emphasis: CurveEmphasis,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Deadline {
    pub at_minute: f64,
    pub marker_label: String,
}

fn finite_nonneg(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

/// Starts outside the day are pulled back into it; NaN counts as midnight.
pub fn clamp_start(minute: f64) -> f64 {
    if minute.is_finite() {
        minute.clamp(0.0, (DAY - 1) as f64)
    } else {
        0.0
    }
}

pub fn applies(days: RunDays, day: DayKind) -> bool {
    match days {
        RunDays::Daily => true,
        RunDays::Weekdays => day == DayKind::Weekdays,
        RunDays::Weekend => day == DayKind::Weekend,
    }
}

/// Tokens per minute contributed by `run` at `minute`: a linear rise, then an exponential tail.
pub fn run_load(run: &PlannedRun, minute: f64) -> f64 {
    let start = clamp_start(run.start_minute);
    if minute < start {
        return 0.0;
    }
    let peak = finite_nonneg(run.peak_tokens_per_min);
    let rise = finite_nonneg(run.rise_minutes);
    let half = finite_nonneg(run.tail_half_life_minutes);
    let elapsed = minute - start;
    if rise > 0.0 && elapsed < rise {
        return peak * elapsed / rise;
    }
    let tail = elapsed - rise;
    if half <= 0.0 {
        return if tail > 0.0 { 0.0 } else { peak };
    }
    peak * 0.5_f64.powf(tail / half)
}

/// Combined load for each minute of the day, from the runs that apply to `day`.
pub fn combined_curve(runs: &[PlannedRun], day: DayKind) -> Vec<f64> {
    let mut curve = vec![0.0; DAY];
    for run in runs.iter().filter(|r| applies(r.run_days, day)) {
        for (minute, value) in curve.iter_mut().enumerate() {
            *value += run_load(run, minute as f64);
        }
    }
    curve
}

/// Highest sample and its minute.
pub fn peak_of(curve: &[f64]) -> (usize, f64) {
    curve.iter().copied().enumerate().fold(
        (0, 0.0),
        |best, (m, v)| if v > best.1 { (m, v) } else { best },
    )
}

pub fn status(peak: f64, ceiling: f64, safety: f64) -> LoadStatus {
    if peak > ceiling {
        LoadStatus::Over
    } else if peak > ceiling * safety {
        LoadStatus::Close
    } else {
        LoadStatus::Under
    }
}

/// `HH:MM` for minutes after midnight.
pub fn clock(minute: f64) -> String {
    let total = clamp_start(minute).round() as usize % DAY;
    format!("{:02}:{:02}", total / 60, total % 60)
}

/// Starts for every run after one greedy spread: each unlocked run moves to the
/// grid start that lowers the day's peak most, and stays put on ties.
pub fn spread(runs: &[PlannedRun], day: DayKind, now: Option<f64>) -> Vec<f64> {
    let mut starts: Vec<f64> = runs.iter().map(|r| clamp_start(r.start_minute)).collect();
    for index in 0..runs.len() {
        let run = &runs[index];
        if !applies(run.run_days, day) || is_locked(run, now) {
            continue;
        }
        let mut others = vec![0.0; DAY];
        for (j, other) in runs.iter().enumerate() {
            if j == index || !applies(other.run_days, day) {
                continue;
            }
            let placed = PlannedRun {
                start_minute: starts[j],
                ..other.clone()
            };
            for (minute, value) in others.iter_mut().enumerate() {
                *value += run_load(&placed, minute as f64);
            }
        }
        let score_at = |start: f64| {
            let trial = PlannedRun {
                start_minute: start,
                ..run.clone()
            };
            (0..DAY)
                .map(|m| others[m] + run_load(&trial, m as f64))
                .fold(0.0_f64, f64::max)
        };
        let mut best_start = starts[index];
        let mut best_score = score_at(best_start);
        for candidate in (0..DAY).step_by(STEP) {
            let start = candidate as f64;
            if now.is_some_and(|n| start < n) {
                continue;
            }
            let score = score_at(start);
            if score + 1e-9 < best_score {
                best_score = score;
                best_start = start;
            }
        }
        starts[index] = best_start;
    }
    starts
}

/// A run that began before `now` cannot move.
pub fn is_locked(run: &PlannedRun, now: Option<f64>) -> bool {
    now.is_some_and(|n| clamp_start(run.start_minute) < n)
}

/// Moves one run to `to`; `None` when the run is locked or unknown.
pub fn move_run(
    runs: &[PlannedRun],
    id: &str,
    to: f64,
    now: Option<f64>,
) -> Option<Vec<PlannedRun>> {
    let index = runs.iter().position(|r| r.run_id == id)?;
    if is_locked(&runs[index], now) {
        return None;
    }
    let mut next = runs.to_vec();
    next[index].start_minute = clamp_start(to);
    Some(next)
}

/// Runs still within their peak-and-tail span at `minute`.
pub fn active_at(runs: &[PlannedRun], minute: f64) -> Vec<String> {
    runs.iter()
        .filter(|r| {
            let start = clamp_start(r.start_minute);
            let span =
                finite_nonneg(r.rise_minutes) + 2.0 * finite_nonneg(r.tail_half_life_minutes);
            minute >= start && minute < start + span
        })
        .map(|r| r.run_name.clone())
        .collect()
}

/// SVG path for the parts of a curve above `ceiling`, in a 0 to 100 wide, 0 to `height` box.
pub fn overflow_path(curve: &[f64], ceiling: f64, scale: f64, height: f64) -> String {
    let mut out = String::new();
    let mut open = false;
    for (i, value) in curve.iter().enumerate() {
        let x = i as f64 / (curve.len().max(2) - 1) as f64 * 100.0;
        if *value > ceiling {
            let y = height - value / scale * height;
            let base = height - ceiling / scale * height;
            if !open {
                out.push_str(&format!("M{x:.2} {base:.2} "));
                open = true;
            }
            out.push_str(&format!("L{x:.2} {y:.2} "));
        } else if open {
            let base = height - ceiling / scale * height;
            out.push_str(&format!("L{x:.2} {base:.2} Z "));
            open = false;
        }
    }
    if open {
        let base = height - ceiling / scale * height;
        out.push_str(&format!("L100 {base:.2} Z"));
    }
    out.trim_end().to_string()
}

/// The day's samples in window order: `window` is the minute the chart starts at.
fn display_curve(values: &[f64], window: usize) -> Vec<f64> {
    (0..DAY)
        .map(|i| values.get((window + i) % DAY).copied().unwrap_or(0.0))
        .collect()
}

/// SVG points for `(display index, value)` pairs in the 0 to 100 by 0 to 40 box.
fn points(pairs: impl Iterator<Item = (usize, f64)>, scale: f64) -> String {
    pairs
        .map(|(i, v)| {
            let x = i as f64 / (DAY - 1) as f64 * 100.0;
            let y = 40.0 - v / scale * 40.0;
            format!("{x:.2},{y:.2}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn focus_id(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// Places scheduled runs across a day, under a token-per-minute ceiling.
#[component]
pub fn QuotaLoadScheduler(
    #[props(default)] run_list: Option<Vec<PlannedRun>>,
    #[props(default)] starting_runs: Vec<PlannedRun>,
    #[props(default)] on_runs_change: Option<EventHandler<Vec<PlannedRun>>>,
    #[props(default = 80_000.0)] provider_ceiling_per_min: f64,
    #[props(default = 0.75)] safety_ratio: f64,
    #[props(default = 5.0)] window_start_hour: f64,
    #[props(default)] deadline_marker: Option<Deadline>,
    #[props(default)] now_minute: Option<f64>,
    #[props(default)] measured_load: Option<Vec<f64>>,
    #[props(default)] day_kind: DayKind,
    #[props(default)] focused_run_id: Option<String>,
    #[props(default)] on_focus_change: Option<EventHandler<String>>,
    #[props(default = "Rate limit".to_string(), into)] board_title: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-quota-{}", next_id()));
    let mut internal_runs = use_signal(|| starting_runs.clone());
    let mut internal_focus = use_signal(|| Option::<String>::None);
    let mut editing = use_signal(|| Option::<String>::None);
    let mut announce = use_signal(String::new);
    let mut drag = use_signal(|| Option::<(String, f64, f64, f64)>::None);
    let mut track_width = use_signal(|| 600.0_f64);
    let mut move_choice = use_signal(|| 0.0_f64);

    let controlled = run_list.is_some();
    let runs: Vec<PlannedRun> = run_list.clone().unwrap_or_else(|| internal_runs());
    let ceiling = provider_ceiling_per_min.max(1.0);
    let safety = safety_ratio.clamp(0.0, 1.0);
    let curve = combined_curve(&runs, day_kind);
    let (peak_minute, peak_value) = peak_of(&curve);
    let level = status(peak_value, ceiling, safety);
    let focused = if focused_run_id.is_some() {
        focused_run_id.clone()
    } else {
        internal_focus()
    };
    let total_day: f64 = curve.iter().sum();
    let scale = ceiling.max(peak_value).max(1.0) * 1.1;
    let window = (window_start_hour.rem_euclid(24.0) * 60.0).round() as usize;
    let index_of = move |minute: f64| (clamp_start(minute) as usize + DAY - window) % DAY;
    let x_of = |index: usize| index as f64 / (DAY - 1) as f64 * 100.0;
    let y_pct = |value: f64| (1.0 - value / scale) * 100.0;
    let shape = display_curve(&curve, window);
    let overflow = overflow_path(&shape, ceiling, scale, 40.0);
    let peak_index = index_of(peak_minute as f64);
    // Shared by the axis, the guide lines, the Now line and the checkpoint, so
    // every marker lands on the same hour as the run rows beneath it.
    let at_index = |index: usize| {
        format!(
            "left: calc(var(--quota-gutter) + (100% - var(--quota-gutter) - var(--quota-end)) * {:.5});",
            index as f64 / (DAY - 1) as f64
        )
    };
    // The axis and marker rows already sit inside the gutters, so they use their own width.
    let percent_at =
        |index: usize| format!("left: {:.4}%;", index as f64 / (DAY - 1) as f64 * 100.0);
    let axis_ticks: Vec<(String, String)> = (0..8)
        .map(|k| {
            let minute = ((window + k * 180) % DAY) as f64;
            (percent_at(index_of(minute)), clock(minute))
        })
        .collect();

    let hour_guides: Vec<String> = (0..24)
        .map(|h| at_index(index_of(((window + h * 60) % DAY) as f64)))
        .collect();
    let now_index = now_minute.map(index_of);
    let now_line = now_index.map(&at_index);
    let now_label = now_index
        .zip(now_minute)
        .map(|(index, m)| (percent_at(index), format!("Now {}", clock(m))));
    let measured: Option<Vec<f64>> = measured_load.clone();
    let measured_line = measured.as_ref().map(|values| {
        let count = now_index.map_or(values.len(), |n| n + 1).min(values.len());
        points(values.iter().copied().take(count).enumerate(), scale)
    });
    let (total_line, total_part) = match (&measured_line, now_index) {
        (Some(_), Some(now)) => (
            points(shape.iter().copied().enumerate().skip(now), scale),
            "forward",
        ),
        _ => (points(shape.iter().copied().enumerate(), scale), "full"),
    };

    let slot_of = |index: usize| index % 6 + 1;
    let mut run_lines: Vec<(usize, String)> = Vec::new();
    let mut run_tags: Vec<(usize, String, String)> = Vec::new();
    for (index, run) in runs.iter().enumerate() {
        if !applies(run.run_days, day_kind) {
            continue;
        }
        let values: Vec<f64> = (0..DAY).map(|m| run_load(run, m as f64)).collect();
        let own = display_curve(&values, window);
        let (own_peak, own_value) = peak_of(&own);
        run_lines.push((
            slot_of(index),
            points(own.iter().copied().enumerate(), scale),
        ));
        run_tags.push((
            slot_of(index),
            format!(
                "left: {:.2}%; top: {:.2}%;",
                x_of(own_peak),
                y_pct(own_value)
            ),
            (index + 1).to_string(),
        ));
    }

    let checkpoint_line = deadline_marker
        .as_ref()
        .map(|d| at_index(index_of(d.at_minute)));
    let checkpoint_label = deadline_marker.as_ref().map(|d| {
        (
            percent_at(index_of(d.at_minute)),
            format!("{} {}", d.marker_label, clock(d.at_minute)),
        )
    });
    let run_count = runs.len();
    let total_text = format!("{:.0}", total_day);
    let summary =
        format!("{run_count} runs · {total_text} tokens today · ceiling {ceiling:.0}/min");
    let ceiling_label = format!("Ceiling {ceiling:.0}/min");
    let ceiling_top = format!("top: {:.2}%;", y_pct(ceiling));
    let peak_label = clock(peak_minute as f64);
    let peak_text = format!("Peak {:.0}/min at {}", peak_value, peak_label);
    let peak_style = format!(
        "left: {:.2}%; top: {:.2}%;",
        x_of(peak_index),
        y_pct(peak_value)
    );
    let peak_side = if x_of(peak_index) > 80.0 {
        "left"
    } else {
        "right"
    };
    let peak_at_text = format!("{:.0}/min at {}", peak_value, peak_label);
    let over_text = if level == LoadStatus::Over {
        format!(" Above the {ceiling:.0}/min ceiling.")
    } else {
        String::new()
    };
    let note = match &measured {
        Some(values) => {
            let measured_peak = values.iter().copied().fold(0.0_f64, f64::max);
            let (word, gap) = if measured_peak > peak_value {
                ("ahead of", measured_peak - peak_value)
            } else {
                ("behind", peak_value - measured_peak)
            };
            format!(
                "Measured load runs {word} plan by {gap:.0}/min. Expected peak {peak_at_text}.{over_text}"
            )
        }
        None => format!("Expected peak {peak_at_text}.{over_text}"),
    };
    let level_text = level.label();
    let level_attr = match level {
        LoadStatus::Under => "under",
        LoadStatus::Close => "close",
        LoadStatus::Over => "over",
    };
    let root_cls = class_list("kit-quota", &root_class);
    let announcement = announce();
    let spread_id = format!("{base}-spread");
    let add_id = format!("{base}-add");
    let edit_open = editing();
    let edit_run = edit_open
        .as_ref()
        .and_then(|id| runs.iter().find(|r| &r.run_id == id).cloned());
    let hatch_id = format!("{base}-hatch");
    let empty = runs.is_empty();
    let deadline_text = deadline_marker.as_ref().map(|d| {
        let names = active_at(&runs, d.at_minute);
        format!(
            "{} at {}: {}",
            d.marker_label,
            clock(d.at_minute),
            if names.is_empty() {
                "nothing running".to_string()
            } else {
                names.join(", ")
            }
        )
    });
    let times: Vec<f64> = (0..24).map(|h| (h * 60) as f64).collect();

    let mut write_runs = move |next: Vec<PlannedRun>| {
        if !controlled {
            internal_runs.set(next.clone());
        }
        if let Some(handler) = on_runs_change {
            handler.call(next);
        }
    };

    let focus_run = move |id: String| {
        if focused_run_id.is_none() {
            internal_focus.set(Some(id.clone()));
        }
        if let Some(handler) = on_focus_change {
            handler.call(id);
        }
    };

    let spread_now = {
        let runs = runs.clone();
        move |_| {
            let starts = spread(&runs, day_kind, now_minute);
            let next: Vec<PlannedRun> = runs
                .iter()
                .zip(starts.iter())
                .map(|(r, s)| PlannedRun {
                    start_minute: *s,
                    ..r.clone()
                })
                .collect();
            let (_, before) = peak_of(&combined_curve(&runs, day_kind));
            let (_, after) = peak_of(&combined_curve(&next, day_kind));
            announce.set(format!(
                "Spread out: peak {:.0} to {:.0} per minute",
                before, after
            ));
            write_runs(next);
        }
    };

    let add_run = {
        let runs = runs.clone();
        let add_prefix = base.clone();
        move |_| {
            let mut next = runs.clone();
            let index = next.len() + 1;
            next.push(PlannedRun {
                run_id: format!("{add_prefix}-run-{}", next_id()),
                run_name: format!("Run {index}"),
                start_minute: 9.0 * 60.0,
                peak_tokens_per_min: 10_000.0,
                rise_minutes: 10.0,
                tail_half_life_minutes: 20.0,
                run_days: RunDays::Daily,
                curve_emphasis: CurveEmphasis::Normal,
            });
            write_runs(next);
        }
    };

    let move_selected = {
        let runs = runs.clone();
        let focused = focused.clone();
        move |_| {
            let Some(id) = focused.clone() else {
                return;
            };
            let to = move_choice();
            match move_run(&runs, &id, to, now_minute) {
                Some(next) => {
                    announce.set(format!("Moved to {}", clock(to)));
                    write_runs(next);
                }
                None => announce.set("That run has started and cannot move".to_string()),
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class: root_cls,
            "data-level": level_attr,
            onpointermove: move |event: PointerEvent| {
                if let Some((id, x0, start0, width)) = drag() {
                    let dx = event.client_coordinates().x - x0;
                    let delta = dx / width.max(1.0) * DAY as f64;
                    let target = clamp_start(start0 + delta).round();
                    if let Some(next) = move_run(&runs_for_drag(&run_list, &internal_runs), &id, target, now_minute) {
                        write_runs(next);
                    }
                }
            },
            onpointerup: move |_| drag.set(None),
            onpointerleave: move |_| drag.set(None),
            header { class: "kit-quota__head",
                h2 { class: "kit-quota__title", "{board_title}" }
                span { class: "kit-quota__status", "data-level": level_attr, "{level_text}" }
                span { class: "kit-quota__peak", "{peak_text}" }
            }
            p { class: "kit-quota__summary", "{summary}" }
            div { class: "kit-quota__body",
                div { class: "kit-quota__axis", "aria-hidden": "true",
                    for (style, text) in axis_ticks.iter() {
                        span { key: "{style}", style: "{style}", "{text}" }
                    }
                }
                div { class: "kit-quota__plot",
                    svg { class: "kit-quota__chart", view_box: "0 0 100 40", "preserveAspectRatio": "none", "aria-hidden": "true",
                        defs {
                            pattern { id: "{hatch_id}", width: "4", height: "4", "patternUnits": "userSpaceOnUse", "patternTransform": "rotate(45)",
                                line { class: "kit-quota__hatch", x1: "0", y1: "0", x2: "0", y2: "4", "stroke-width": "1" }
                            }
                        }
                        line { class: "kit-quota__ceiling", x1: "0", y1: "{y_pct(ceiling) * 0.4:.2}", x2: "100", y2: "{y_pct(ceiling) * 0.4:.2}" }
                        if !overflow.is_empty() {
                            path { class: "kit-quota__overflow", d: "{overflow}", fill: "url(#{hatch_id})" }
                        }
                        for (slot, line) in run_lines.iter() {
                            polyline { key: "{slot}-{line.len()}", class: "kit-quota__run", "data-slot": "{slot}", points: "{line}", fill: "none" }
                        }
                        polyline { class: "kit-quota__total", "data-part": total_part, points: "{total_line}", fill: "none" }
                        if let Some(line) = measured_line.as_ref() {
                            polyline { class: "kit-quota__measured", points: "{line}", fill: "none" }
                        }
                    }
                    span { class: "kit-quota__ceiling-label", style: "{ceiling_top}", "{ceiling_label}" }
                    span { class: "kit-quota__dot", style: "{peak_style}", "aria-hidden": "true" }
                    span { class: "kit-quota__tag kit-quota__tag--peak", "data-side": peak_side, style: "{peak_style}", "aria-hidden": "true", "{peak_label}" }
                    for (slot, style, text) in run_tags.iter() {
                        span { key: "{slot}-{text}", class: "kit-quota__tag", "data-slot": "{slot}", style: "{style}", "aria-hidden": "true", "{text}" }
                    }
                }
                div { class: "kit-quota__marks", "aria-hidden": "true",
                    if let Some((style, text)) = checkpoint_label.as_ref() {
                        span { class: "kit-quota__mark", style: "{style}", "{text}" }
                    }
                }
                div { class: "kit-quota__guides", "aria-hidden": "true",
                    for (hour, style) in hour_guides.iter().enumerate() {
                        span { key: "{hour}", class: "kit-quota__guide", style: "{style}" }
                    }
                    if let Some(style) = checkpoint_line.as_ref() {
                        span { class: "kit-quota__guide kit-quota__guide--checkpoint", style: "{style}" }
                    }
                    if let Some(style) = now_line.as_ref() {
                        span { class: "kit-quota__guide kit-quota__guide--now", style: "{style}",
                            if let Some((_, text)) = now_label.as_ref() {
                                span { class: "kit-quota__now-label", {text.clone()} }
                            }
                        }
                    }
                }
                ol { class: "kit-quota__rows",
                    for (index, run) in runs.iter().enumerate() {
                        {
                            let id = run.run_id.clone();
                            let name = run.run_name.clone();
                            let locked = is_locked(run, now_minute);
                            let start_text = clock(run.start_minute);
                            let value_text = format!(
                                "starts {}, peak {:.0} tokens per minute{}",
                                start_text,
                                run.peak_tokens_per_min,
                                if locked { ", started" } else { "" }
                            );
                            let left = x_of(index_of(run.start_minute));
                            let span_pct = (finite_nonneg(run.rise_minutes)
                                + 2.0 * finite_nonneg(run.tail_half_life_minutes))
                                / (DAY - 1) as f64
                                * 100.0;
                            let bar_style = format!(
                                "left: {left:.2}%; width: {:.2}%;",
                                span_pct.min(100.0 - left)
                            );
                            let peak_run_text = format!("peak {:.0}/min", run.peak_tokens_per_min);
                            let slot = slot_of(index);
                            let key_text = (index + 1).to_string();
                            let is_focus = focused.as_deref() == Some(id.as_str());
                            let row_class = if is_focus { "kit-quota__row is-focus" } else { "kit-quota__row" };
                            let settings_id = format!("{base}-settings-{id}");
                            let slider_id = format!("{base}-bar-{id}");
                            let key = id.clone();
                            let bar_id = id.clone();
                            let bar_runs = runs.clone();
                            let remove_id = id.clone();
                            let remove_runs = runs.clone();
                            let edit_id = id.clone();
                            let focus_bar = focus_run.clone();
                            let focus_click = focus_run.clone();
                            let drag_id = id.clone();
                            let drag_slider = slider_id.clone();
                            let key_slider = slider_id.clone();
                            let key_settings = settings_id.clone();
                            let key_base = base.clone();
                            let disabled = if locked { "true" } else { "false" };
                            let name_for_remove = name.clone();
                            rsx! {
                                li { key: "{key}", class: row_class, "data-run-id": "{key}", "data-slot": "{slot}",
                                    div { class: "kit-quota__label",
                                        span { class: "kit-quota__key", "aria-hidden": "true", "{key_text}" }
                                        div { class: "kit-quota__name",
                                            span { "{name}" }
                                            span { class: "kit-quota__start", "{start_text}" }
                                        }
                                    }
                                    div {
                                        id: "{slider_id}",
                                        class: "kit-quota__bar",
                                        role: "slider",
                                        tabindex: "0",
                                        "aria-label": "{name}",
                                        "aria-valuemin": "0",
                                        "aria-valuemax": "1439",
                                        "aria-valuenow": "{clamp_start(run.start_minute):.0}",
                                        "aria-valuetext": "{value_text}",
                                        "aria-disabled": "{disabled}",
                                        onfocus: {
                                            let mut focus_bar = focus_bar.clone();
                                            let bar_id = bar_id.clone();
                                            move |_| focus_bar(bar_id.clone())
                                        },
                                        onpointerdown: {
                                            let bar_runs = bar_runs.clone();
                                            let drag_id = drag_id.clone();
                                            let drag_slider = drag_slider.clone();
                                            move |event: PointerEvent| {
                                            if locked {
                                                return;
                                            }
                                            let start = clamp_start(bar_runs.iter().find(|r| r.run_id == drag_id).map(|r| r.start_minute).unwrap_or(0.0));
                                            let x0 = event.client_coordinates().x;
                                            let element = drag_slider.clone();
                                            let measure = format!(
                                                "document.getElementById({element:?})?.parentElement?.getBoundingClientRect().width || 600"
                                            );
                                            spawn(async move {
                                                let width = document::eval(&measure)
                                                    .await
                                                    .ok()
                                                    .and_then(|v| v.as_f64())
                                                    .unwrap_or(600.0);
                                                track_width.set(width);
                                            });
                                            let width = track_width();
                                            drag.set(Some((drag_id.clone(), x0, start, width)));
                                            }
                                        },
                                        onkeydown: {
                                            let bar_id = bar_id.clone();
                                            let bar_runs = bar_runs.clone();
                                            let remove_id = remove_id.clone();
                                            let remove_runs = remove_runs.clone();
                                            let edit_id = edit_id.clone();
                                            let key_settings = key_settings.clone();
                                            let key_slider = key_slider.clone();
                                            let key_base = key_base.clone();
                                            let name_for_remove = name_for_remove.clone();
                                            move |event: KeyboardEvent| {
                                            if locked {
                                                return;
                                            }
                                            let step = if event.modifiers().shift() { 60.0 } else { 15.0 };
                                            let delta = match event.key() {
                                                Key::ArrowRight => step,
                                                Key::ArrowLeft => -step,
                                                Key::PageUp => 60.0,
                                                Key::PageDown => -60.0,
                                                Key::Home => -clamp_start(9999.0),
                                                Key::End => 1439.0,
                                                Key::Enter => {
                                                    event.prevent_default();
                                                    editing.set(Some(edit_id.clone()));
                                                    focus_id(&key_settings);
                                                    return;
                                                }
                                                Key::Delete => {
                                                    event.prevent_default();
                                                    let removed_at = remove_runs.iter().position(|r| r.run_id == remove_id).unwrap_or(0);
                                                    let next: Vec<PlannedRun> = remove_runs.iter().filter(|r| r.run_id != remove_id).cloned().collect();
                                                    announce.set(format!("Removed {name_for_remove}"));
                                                    let target = next
                                                        .get(removed_at)
                                                        .or_else(|| removed_at.checked_sub(1).and_then(|p| next.get(p)))
                                                        .map(|r| format!("{key_base}-bar-{}", r.run_id))
                                                        .unwrap_or_else(|| format!("{key_base}-spread"));
                                                    write_runs(next);
                                                    focus_after_frame(&target);
                                                    return;
                                                }
                                                _ => return,
                                            };
                                            event.prevent_default();
                                            let current = bar_runs.iter().find(|r| r.run_id == bar_id).map(|r| r.start_minute).unwrap_or(0.0);
                                            let target = match event.key() {
                                                Key::Home => 0.0,
                                                Key::End => (DAY - 1) as f64,
                                                _ => clamp_start(current + delta),
                                            };
                                            if let Some(next) = move_run(&bar_runs, &bar_id, target, now_minute) {
                                                announce.set(format!("Starts at {}", clock(target)));
                                                write_runs(next);
                                            }
                                            focus_id(&key_slider);
                                            }
                                        },
                                        span { class: "kit-quota__fill", style: "{bar_style}" }
                                    }
                                    div { class: "kit-quota__end",
                                        span { class: "kit-quota__peak-run", "{peak_run_text}" }
                                        div { class: "kit-quota__controls",
                                            button {
                                                id: "{settings_id}",
                                                r#type: "button",
                                                class: "kit-quota__icon",
                                                "aria-label": "Settings for {name}",
                                                onclick: {
                                                    let edit_id = edit_id.clone();
                                                    move |_| {
                                                        editing.set(Some(edit_id.clone()));
                                                    }
                                                },
                                                "⚙"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-quota__icon",
                                                "aria-label": "Remove {name}",
                                                onclick: {
                                                    let remove_id = remove_id.clone();
                                                    let remove_runs = remove_runs.clone();
                                                    move |_| {
                                                        let next: Vec<PlannedRun> = remove_runs.iter().filter(|r| r.run_id != remove_id).cloned().collect();
                                                        write_runs(next);
                                                    }
                                                },
                                                "×"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-quota__focus",
                                                "aria-label": "Highlight {name}",
                                                onclick: move |_| {
                                                    let mut focus = focus_click.clone();
                                                    focus(id.clone());
                                                },
                                                "Show"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(text) = deadline_text {
                p { class: "kit-quota__deadline", "{text}" }
            }
            if empty {
                p { class: "kit-quota__empty", "No runs for this day" }
            }
            if let Some(run) = edit_run {
                {
                    let return_id = format!("{base}-settings-{}", run.run_id);
                    let done_return = return_id.clone();
                    rsx! {
                        div {
                            class: "kit-quota__dialog",
                            role: "dialog",
                            "aria-modal": "true",
                            "aria-label": "Settings for {run.run_name}",
                            tabindex: "-1",
                            onmounted: focus_on_mount,
                            onkeydown: move |event: KeyboardEvent| {
                                if matches!(event.key(), Key::Escape) {
                                    event.prevent_default();
                                    editing.set(None);
                                    focus_id(&return_id);
                                }
                            },
                            p { class: "kit-quota__dialog-title", "{run.run_name}" }
                            p { "Starts {clock(run.start_minute)}, peak {run.peak_tokens_per_min:.0} per minute" }
                            button {
                                r#type: "button",
                                class: "kit-quota__btn",
                                onclick: move |_| {
                                    editing.set(None);
                                    focus_id(&done_return);
                                },
                                "Done"
                            }
                        }
                    }
                }
            }
            footer { class: "kit-quota__foot",
                button {
                    id: "{spread_id}",
                    r#type: "button",
                    class: "kit-quota__btn",
                    onclick: spread_now,
                    "Spread out"
                }
                label { class: "kit-quota__moveto",
                    span { "Move to" }
                    select {
                        "aria-label": "Move focused run to",
                        onchange: move |event: FormEvent| {
                            if let Ok(value) = event.value().parse::<f64>() {
                                move_choice.set(value);
                            }
                        },
                        for time in times.iter().copied() {
                            option { key: "{time}", value: "{time}", "{clock(time)}" }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "kit-quota__btn",
                        onclick: move_selected,
                        "Move"
                    }
                }
                button {
                    id: "{add_id}",
                    r#type: "button",
                    class: "kit-quota__btn",
                    onclick: add_run,
                    "Add run"
                }
                p { class: "kit-quota__note", role: "status", "aria-live": "polite", "{note}" }
            }
            p { class: "kit-quota__sr", role: "status", "aria-live": "polite", "{announcement}" }
        }
    }
}

/// The run list a drag starts from: the host's list when it owns the runs, else the panel's.
fn runs_for_drag(
    host: &Option<Vec<PlannedRun>>,
    internal: &Signal<Vec<PlannedRun>>,
) -> Vec<PlannedRun> {
    match host {
        Some(runs) => runs.clone(),
        None => internal.peek().clone(),
    }
}
