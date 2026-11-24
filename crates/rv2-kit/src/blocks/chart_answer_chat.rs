//! Chart answer chat: a conversation whose answers are data visuals. Figures, a trend against
//! a comparison period, a breakdown, or a forecast, each with its short sentence. A range of
//! a trend or one breakdown bar becomes the subject of the next question.
//!
//! Design choice: figures are `MetricTile`s and trends are `SeriesTrendChart`s; the chat owns
//! only the selection and the composer.
//! Design choice: a trend range is picked by dragging across the chart, or by keyboard with
//! Shift to extend. The chart's own hover marks the points, so a drag follows the pointer.
//! Design choice: a forecast draws its actual line solid, the forecast from its last actual point
//! dashed, and the band between the optional low and high values as a shaded area. The shapes are
//! drawn here, not by `SeriesTrendChart`, which only draws lines.
//! Design choice: within an answer, blocks appear in order, each 700 ms after the one before, and
//! the trend line inside a block draws left to right; the stylesheet sets the timing.
//! Browser checks: drag selects a range and shows its chip, breakdown toggle, Escape order,
//! Shift+Arrow extension, forecast band and dashed forecast, phone stat cards in one column.

use dioxus::prelude::*;

use crate::components::metric_tile::{MetricTile, MetricValue};
use crate::components::series_trend_chart::{group_digits, nice_axis, value_domain};
use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/chart-answer-chat.css");

/// Left edge of point `index` of `last + 1` points, in percent of the card. Points run from 8% to 96%.
pub fn x_pct(index: usize, last: usize) -> f64 {
    8.0 + if last == 0 {
        0.0
    } else {
        index as f64 / last as f64 * 88.0
    }
}

/// Height of `value` in percent of the card, on the axis `lo..hi`: `lo` sits at 78% and `hi` at 14%.
pub fn y_pct(value: f64, lo: f64, hi: f64) -> f64 {
    78.0 - (value - lo) / (hi - lo).max(f64::EPSILON) * 64.0
}

/// Index of the highest value; the first one when two tie.
pub fn peak_index(values: &[f64]) -> Option<usize> {
    values
        .iter()
        .copied()
        .enumerate()
        .fold(None::<(usize, f64)>, |best, (i, v)| match best {
            Some((_, b)) if b >= v => best,
            _ => Some((i, v)),
        })
        .map(|(i, _)| i)
}

/// The value axis of a trend: its own nice bounds over both series, from zero.
pub fn trend_axis(values: &[f64], comparison: &[Option<f64>]) -> (f64, f64) {
    let all = values
        .iter()
        .copied()
        .chain(comparison.iter().flatten().copied());
    let (lo, hi) = value_domain(all, true);
    let (nlo, nhi, _) = nice_axis(lo, hi, 2);
    (nlo, nhi)
}

/// Point lists for the drawing, in percent of the card. The comparison line skips missing points.
#[derive(Clone, Debug, PartialEq)]
pub struct TrendGeometry {
    pub main: String,
    pub comparison: String,
    pub area: String,
}

pub fn trend_geometry(
    values: &[f64],
    comparison: &[Option<f64>],
    axis: (f64, f64),
) -> TrendGeometry {
    let last = values.len().saturating_sub(1);
    let (lo, hi) = axis;
    let main: Vec<(f64, f64)> = values
        .iter()
        .enumerate()
        .map(|(i, v)| (x_pct(i, last), y_pct(*v, lo, hi)))
        .collect();
    let compared: Vec<String> = comparison
        .iter()
        .enumerate()
        .filter_map(|(i, v)| Some(format!("{:.2},{:.2}", x_pct(i, last), y_pct((*v)?, lo, hi))))
        .collect();
    let mut area: Vec<String> = Vec::with_capacity(main.len() + 2);
    if let (Some(first), Some(end)) = (main.first(), main.last()) {
        area.push(format!("{:.2},78.00", first.0));
        area.extend(main.iter().map(|(x, y)| format!("{x:.2},{y:.2}")));
        area.push(format!("{:.2},78.00", end.0));
    }
    TrendGeometry {
        main: main
            .iter()
            .map(|(x, y)| format!("{x:.2},{y:.2}"))
            .collect::<Vec<_>>()
            .join(" "),
        comparison: compared.join(" "),
        area: area.join(" "),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Figure {
    pub label: String,
    pub value: String,
    pub delta_percent: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrendPoint {
    pub x_label: String,
    pub value: f64,
    pub comparison_value: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BreakdownGroup {
    pub label: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForecastPoint {
    pub x_label: String,
    pub value: f64,
    pub band_low: Option<f64>,
    pub band_high: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnswerBlock {
    Figures(Vec<Figure>),
    Trend {
        block_id: String,
        series_label: String,
        comparison_label: String,
        points: Vec<TrendPoint>,
        /// Line under the chart, such as how to ask about a span.
        hint: Option<String>,
    },
    Breakdown {
        block_id: String,
        groups: Vec<BreakdownGroup>,
    },
    Forecast {
        block_id: String,
        points: Vec<ForecastPoint>,
        actual_until_index: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Person,
    Assistant,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub role: Role,
    pub text: Option<String>,
    pub blocks: Vec<AnswerBlock>,
    pub failed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prompt {
    pub id: String,
    pub label: String,
}

/// The chosen part of a block: an inclusive index range (a single bar has `from == to`).
#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    pub block_id: String,
    pub from_index: usize,
    pub to_index: usize,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AskSubmit {
    pub text: String,
    pub selection: Option<Selection>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EscapeAction {
    ClearSelection,
    ClearText,
    None,
}

/// Escape clears the selection chip before the composer text.
pub fn escape_action(has_selection: bool, has_text: bool) -> EscapeAction {
    match (has_selection, has_text) {
        (true, _) => EscapeAction::ClearSelection,
        (false, true) => EscapeAction::ClearText,
        (false, false) => EscapeAction::None,
    }
}

/// Inclusive `(low, high)` of two indices, whichever came first.
pub fn range_bounds(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}

/// The index a Shift+Arrow range grows from: the existing anchor, else where the cursor was.
pub fn selection_anchor(anchor: Option<usize>, cursor: usize) -> usize {
    anchor.unwrap_or(cursor)
}

/// Change in percent from the first to the last value of a range, and the sum of the range.
pub fn range_summary(values: &[f64], from: usize, to: usize) -> (f64, f64) {
    let (lo, hi) = range_bounds(from, to);
    let end = hi.min(values.len().saturating_sub(1));
    let Some(slice) = values.get(lo.min(end)..=end) else {
        return (0.0, 0.0);
    };
    let (Some(first), Some(last)) = (slice.first(), slice.last()) else {
        return (0.0, 0.0);
    };
    let change = if *first == 0.0 {
        0.0
    } else {
        (last - first) / first * 100.0
    };
    (change, slice.iter().sum())
}

/// Each group's share of the total, in percent, in the order given.
pub fn breakdown_shares(groups: &[BreakdownGroup]) -> Vec<f64> {
    let total: f64 = groups.iter().map(|g| g.value.max(0.0)).sum();
    groups
        .iter()
        .map(|g| {
            if total > 0.0 {
                g.value.max(0.0) / total * 100.0
            } else {
                0.0
            }
        })
        .collect()
}

fn range_label(x_labels: &[String], from: usize, to: usize) -> String {
    let (lo, hi) = range_bounds(from, to);
    let first = x_labels.get(lo).cloned().unwrap_or_default();
    let last = x_labels.get(hi).cloned().unwrap_or_default();
    if lo == hi {
        first
    } else {
        format!("{first} to {last}")
    }
}

/// SVG points of a forecast: actual line, dashed forecast line from the last actual point,
/// and the band between the optional low and high values of the forecast points.
#[derive(Clone, Debug, PartialEq)]
pub struct ForecastGeometry {
    pub actual: String,
    pub forecast: String,
    pub band: String,
}

pub fn forecast_geometry(points: &[ForecastPoint], actual_until: usize) -> ForecastGeometry {
    let n = points.len();
    if n == 0 {
        return ForecastGeometry {
            actual: String::new(),
            forecast: String::new(),
            band: String::new(),
        };
    }
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    for p in points {
        for v in [Some(p.value), p.band_low, p.band_high]
            .into_iter()
            .flatten()
        {
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    if hi <= lo {
        hi = lo + 1.0;
    }
    let x = |i: usize| {
        if n == 1 {
            0.0
        } else {
            i as f64 / (n - 1) as f64 * 600.0
        }
    };
    let y = |v: f64| 170.0 - (v - lo) / (hi - lo) * 160.0 + 5.0;
    let join = |items: Vec<(f64, f64)>| {
        items
            .iter()
            .map(|(a, b)| format!("{a:.1},{b:.1}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let actual = join(
        points
            .iter()
            .enumerate()
            .take(actual_until)
            .map(|(i, p)| (x(i), y(p.value)))
            .collect(),
    );
    let start = actual_until.saturating_sub(1);
    let forecast = join(
        points
            .iter()
            .enumerate()
            .skip(start)
            .map(|(i, p)| (x(i), y(p.value)))
            .collect(),
    );
    let banded: Vec<(usize, f64, f64)> = points
        .iter()
        .enumerate()
        .skip(start)
        .filter_map(|(i, p)| Some((i, p.band_low?, p.band_high?)))
        .collect();
    let mut outline: Vec<(f64, f64)> = banded
        .iter()
        .map(|(i, _, high)| (x(*i), y(*high)))
        .collect();
    outline.extend(banded.iter().rev().map(|(i, low, _)| (x(*i), y(*low))));
    ForecastGeometry {
        actual,
        forecast,
        band: join(outline),
    }
}

pub fn forecast_summary(points: &[ForecastPoint], actual_until: usize) -> String {
    match points.last() {
        Some(last) => format!(
            "Forecast of {} points, actual until point {}, ends at {:.0} on {}",
            points.len(),
            actual_until,
            last.value,
            last.x_label
        ),
        None => "Forecast with no points".to_string(),
    }
}

#[component]
fn TrendBlock(
    block_id: String,
    series_label: String,
    comparison_label: String,
    x_labels: Vec<String>,
    values: Vec<f64>,
    comparison: Vec<Option<f64>>,
    hint: Option<String>,
    selection: Option<Selection>,
    on_select: EventHandler<Option<Selection>>,
) -> Element {
    let mut hover = use_signal(|| None::<usize>);
    let mut anchor = use_signal(|| None::<usize>);
    let mut cursor = use_signal(|| None::<usize>);
    let last = x_labels.len().saturating_sub(1);

    let axis = trend_axis(&values, &comparison);
    let (axis_lo, axis_hi) = axis;
    let geometry = trend_geometry(&values, &comparison, axis);
    let shown = hover().or(cursor()).or(peak_index(&values));
    let half_column = 44.0 / last.max(1) as f64;
    let overlay = selection.as_ref().map(|s| {
        let (lo, hi) = range_bounds(s.from_index, s.to_index);
        (x_pct(lo, last), x_pct(hi, last) - x_pct(lo, last))
    });
    let summary = selection.as_ref().map(|s| {
        let (change, sum) = range_summary(&values, s.from_index, s.to_index);
        format!("{change:+.1}% · total {sum:.0}")
    });
    let described = format!(
        "{series_label} against {comparison_label}, {} points",
        x_labels.len()
    );
    let first_label = x_labels.first().cloned().unwrap_or_default();
    let mid_label = x_labels.get(last / 2).cloned().unwrap_or_default();
    let last_label = x_labels.last().cloned().unwrap_or_default();
    let columns: Vec<(f64, f64)> = (0..values.len())
        .map(|i| (x_pct(i, last) - half_column, half_column * 2.0))
        .collect();
    let axis_labels = [
        group_digits(axis_hi),
        group_digits((axis_hi + axis_lo) / 2.0),
        group_digits(axis_lo),
    ];
    let tip = shown.map(|i| {
        let x = x_pct(i, last);
        let y = y_pct(values[i], axis_lo, axis_hi);
        let comparison_y = comparison
            .get(i)
            .copied()
            .flatten()
            .map(|v| y_pct(v, axis_lo, axis_hi));
        let comparison_text = comparison
            .get(i)
            .copied()
            .flatten()
            .map(|v| format!("{comparison_label} {}", group_digits(v)));
        (
            x_labels[i].clone(),
            format!("{series_label} {}", group_digits(values[i])),
            comparison_text,
            x,
            y,
            comparison_y,
            format!("right: {:.2}%", 100.0 - x + 2.0),
        )
    });
    let guide = tip.as_ref().map(|t| (t.3, t.4));
    let drag_labels = x_labels.clone();
    let key_labels = x_labels.clone();
    let drag_block = block_id.clone();
    let key_block = block_id.clone();

    rsx! {
        div { class: "kit-chat__trend", role: "group", "aria-label": "{described}", tabindex: "0",
            onpointerdown: move |_| anchor.set(hover()),
            onpointerup: move |_| {
                if let (Some(a), Some(b)) = (anchor(), hover()) {
                    if a != b {
                        let (lo, hi) = range_bounds(a, b);
                        on_select.call(Some(Selection {
                            block_id: drag_block.clone(),
                            from_index: lo,
                            to_index: hi,
                            label: range_label(&drag_labels, lo, hi),
                        }));
                    }
                }
                anchor.set(None);
            },
            onkeydown: move |e: KeyboardEvent| {
                let step: i64 = match e.key() {
                    Key::ArrowRight => 1,
                    Key::ArrowLeft => -1,
                    _ => 0,
                };
                if step != 0 && last > 0 {
                    e.prevent_default();
                    let current = cursor().or(hover()).unwrap_or(last);
                    let next = (current as i64 + step).clamp(0, last as i64) as usize;
                    cursor.set(Some(next));
                    if e.modifiers().shift() {
                        let from = selection_anchor(anchor(), current);
                        anchor.set(Some(from));
                        let (lo, hi) = range_bounds(from, next);
                        on_select.call(Some(Selection {
                            block_id: key_block.clone(),
                            from_index: lo,
                            to_index: hi,
                            label: range_label(&key_labels, lo, hi),
                        }));
                    }
                }
                if e.key() == Key::Enter {
                    if let Some(c) = cursor() {
                        on_select.call(Some(Selection {
                            block_id: key_block.clone(),
                            from_index: c,
                            to_index: c,
                            label: range_label(&key_labels, c, c),
                        }));
                    }
                }
            },
            div { class: "kit-chat__legend",
                span { class: "kit-chat__legend-item", span { class: "kit-chat__legend-solid", "aria-hidden": "true" }, "{series_label}" }
                span { class: "kit-chat__legend-item", span { class: "kit-chat__legend-dashed", "aria-hidden": "true" }, "{comparison_label}" }
            }
            div { class: "kit-chat__plot", onpointerleave: move |_| { hover.set(None); cursor.set(None); },
                svg { class: "kit-chat__trend-svg", "viewBox": "0 0 100 100", "preserveAspectRatio": "none", "aria-hidden": "true",
                    polygon { class: "kit-chat__area", points: "{geometry.area}" }
                    if !geometry.comparison.is_empty() {
                        polyline { class: "kit-chat__line-last", points: "{geometry.comparison}" }
                    }
                    polyline { class: "kit-chat__line", points: "{geometry.main}" }
                    if let Some((x, y)) = guide {
                        line { class: "kit-chat__guide", x1: "{x:.2}", y1: "{y:.2}", x2: "{x:.2}", y2: "78" }
                    }
                }
                if let Some((left, width)) = overlay {
                    span { class: "kit-chat__range", "aria-hidden": "true", style: "left: {left:.2}%; width: {width:.2}%" }
                }
                for (i, (left, width)) in columns.iter().copied().enumerate() {
                    span { class: "kit-chat__hit", "aria-hidden": "true",
                        style: "left: {left:.2}%; width: {width:.2}%",
                        onpointerenter: move |_| hover.set(Some(i)),
                    }
                }
                span { class: "kit-chat__axis kit-chat__axis--top", "{axis_labels[0]}" }
                span { class: "kit-chat__axis kit-chat__axis--mid", "{axis_labels[1]}" }
                span { class: "kit-chat__axis kit-chat__axis--base", "{axis_labels[2]}" }
                span { class: "kit-chat__week kit-chat__week--first", "{first_label}" }
                span { class: "kit-chat__week kit-chat__week--mid", "{mid_label}" }
                span { class: "kit-chat__week kit-chat__week--last", "{last_label}" }
                if let Some((week, primary, comparison_text, x, y, comparison_y, tip_style)) = tip.clone() {
                    span { class: "kit-chat__dot", style: "left: {x:.2}%; top: {y:.2}%" }
                    if let Some(cy) = comparison_y {
                        span { class: "kit-chat__dot kit-chat__dot--last", style: "left: {x:.2}%; top: {cy:.2}%" }
                    }
                    div { class: "kit-chat__tip", style: "{tip_style}",
                        span { class: "kit-chat__tip-week", "{week}" }
                        span { class: "kit-chat__tip-row",
                            span { class: "kit-chat__legend-solid", "aria-hidden": "true" }
                            "{primary}"
                        }
                        if let Some(text) = comparison_text {
                            span { class: "kit-chat__tip-row",
                                span { class: "kit-chat__legend-dashed", "aria-hidden": "true" }
                                "{text}"
                            }
                        }
                    }
                }
            }
            if let Some(text) = hint.clone() {
                p { class: "kit-chat__hint", "{text}" }
            }
            if let (Some(s), Some(figure)) = (selection.clone(), summary.clone()) {
                button {
                    r#type: "button",
                    class: "kit-chat__chip",
                    "aria-label": "Clear selection, {s.label}",
                    onclick: move |_| on_select.call(None),
                    span { "{s.label}" }
                    span { class: "kit-chat__chip-figure", "{figure}" }
                }
            }
        }
    }
}

#[component]
fn BreakdownBlock(
    block_id: String,
    groups: Vec<BreakdownGroup>,
    selection: Option<Selection>,
    on_select: EventHandler<Option<Selection>>,
) -> Element {
    let shares = breakdown_shares(&groups);
    let max = groups
        .iter()
        .map(|g| g.value)
        .fold(0.0_f64, f64::max)
        .max(f64::EPSILON);
    rsx! {
        ul { class: "kit-chat__bars", "aria-label": "Breakdown",
            for (i, group) in groups.iter().enumerate() {
                {
                    let picked = selection.as_ref().is_some_and(|s| s.from_index == i && s.to_index == i);
                    let width = group.value / max * 100.0;
                    let share = shares[i];
                    let label = group.label.clone();
                    let value = group.value;
                    let aria = format!("{label}, {value:.0}, {share:.0} percent");
                    let bar_block = block_id.clone();
                    rsx! {
                        li { key: "{i}",
                            button {
                                r#type: "button",
                                class: "kit-chat__bar",
                                "aria-pressed": if picked { "true" } else { "false" },
                                "aria-label": "{aria}",
                                onclick: move |_| {
                                    on_select.call(if picked {
                                        None
                                    } else {
                                        Some(Selection { block_id: bar_block.clone(), from_index: i, to_index: i, label: label.clone() })
                                    });
                                },
                                span { class: "kit-chat__bar-label", "{group.label}" }
                                span { class: "kit-chat__bar-track",
                                    span { class: "kit-chat__bar-fill", style: "width: {width:.2}%" }
                                }
                                span { class: "kit-chat__bar-value", "{value:.0} · {share:.0}%" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ChartAnswerChat(
    messages: Vec<ChatMessage>,
    #[props(default)] suggested_prompts: Vec<Prompt>,
    #[props(default = "Ask about your numbers".to_string(), into)] placeholder_text: String,
    #[props(default = "Auto".to_string(), into)] model_label: String,
    #[props(default)] title: Option<String>,
    #[props(default)] is_waiting: bool,
    #[props(default)] on_ask: Option<EventHandler<AskSubmit>>,
    #[props(default)] on_retry: Option<EventHandler<String>>,
    /// Source names shown on the line under the last answer.
    #[props(default)]
    sources: Vec<String>,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let mut draft = use_signal(String::new);
    let mut selection = use_signal(|| None::<Selection>);
    let empty = messages.is_empty();
    let root_class = class_list("kit-chat", &extra_classes);
    let chip = selection();
    let mut send = move || {
        let text = draft().trim().to_string();
        if text.is_empty() {
            return;
        }
        let chosen = selection();
        draft.set(String::new());
        selection.set(None);
        if let Some(handler) = on_ask {
            handler.call(AskSubmit {
                text,
                selection: chosen,
            });
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-label": "Chat about your numbers",
            if let Some(text) = title.clone() {
                header { class: "kit-chat__head",
                    h2 { class: "kit-chat__title", "{text}" }
                }
            }
            div { class: "kit-chat__log", role: "log", "aria-live": "polite",
                for message in messages.iter() {
                    article { key: "{message.id}", class: "kit-chat__msg", "data-role": if message.role == Role::Person { "person" } else { "assistant" },
                        if let Some(text) = message.text.clone() {
                            if message.role == Role::Assistant {
                                span { class: "kit-chat__mark", "aria-hidden": "true" }
                            }
                            p { class: "kit-chat__text", "{text}" }
                        }
                        for (bi, block) in message.blocks.iter().enumerate() {
                            div { key: "{message.id}-b{bi}", class: "kit-chat__block", style: "--kit-block: {bi};",
                            {
                                match block.clone() {
                                    AnswerBlock::Figures(figures) => rsx! {
                                        div { key: "{message.id}-{bi}", class: "kit-chat__figures", role: "group", "aria-label": "Figures",
                                            for figure in figures.iter() {
                                                MetricTile {
                                                    metric_label: figure.label.clone(),
                                                    metric_value: MetricValue::Text(figure.value.clone()),
                                                    change_percent: Some(figure.delta_percent),
                                                    change_note: Some("vs previous period".to_string()),
                                                }
                                            }
                                        }
                                    },
                                    AnswerBlock::Trend { block_id, series_label, comparison_label, points, hint } => {
                                        let x_labels: Vec<String> = points.iter().map(|p| p.x_label.clone()).collect();
                                        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
                                        let comparison: Vec<Option<f64>> = points.iter().map(|p| p.comparison_value).collect();
                                        let chosen = selection().filter(|s| s.block_id == block_id);
                                        rsx! {
                                            TrendBlock {
                                                key: "{block_id}",
                                                block_id: block_id.clone(),
                                                series_label: series_label,
                                                comparison_label: comparison_label,
                                                x_labels: x_labels,
                                                values: values,
                                                comparison: comparison,
                                                hint: hint,
                                                selection: chosen,
                                                on_select: move |s: Option<Selection>| selection.set(s),
                                            }
                                        }
                                    }
                                    AnswerBlock::Breakdown { block_id, groups } => {
                                        let chosen = selection().filter(|s| s.block_id == block_id);
                                        rsx! {
                                            BreakdownBlock {
                                                key: "{block_id}",
                                                block_id: block_id.clone(),
                                                groups: groups,
                                                selection: chosen,
                                                on_select: move |s: Option<Selection>| selection.set(s),
                                            }
                                        }
                                    }
                                    AnswerBlock::Forecast { block_id, points, actual_until_index } => {
                                        let summary = forecast_summary(&points, actual_until_index);
                                        let geometry = forecast_geometry(&points, actual_until_index);
                                        rsx! {
                                            figure { key: "{block_id}", class: "kit-chat__forecast",
                                                svg { class: "kit-chat__forecast-svg", "viewBox": "0 0 600 180", "preserveAspectRatio": "none", "aria-hidden": "true",
                                                    if !geometry.band.is_empty() {
                                                        polygon { class: "kit-chat__band", points: "{geometry.band}" }
                                                    }
                                                    polyline { class: "kit-chat__actual", points: "{geometry.actual}" }
                                                    polyline { class: "kit-chat__forecast-line", points: "{geometry.forecast}" }
                                                }
                                                figcaption { class: "kit-chat__sr", "{summary}" }
                                            }
                                        }
                                    }
                                }
                                }
                            }
                        }
                        if message.failed {
                            p { class: "kit-chat__error", role: "alert", "This answer did not load." }
                            if let Some(retry) = on_retry {
                                button {
                                    r#type: "button",
                                    class: "kit-chat__action",
                                    onclick: {
                                        let id = message.id.clone();
                                        move |_| retry.call(id.clone())
                                    },
                                    "Retry"
                                }
                            }
                        }
                    }
                }
                if !sources.is_empty() {
                    div { class: "kit-chat__source",
                        span { class: "kit-chat__source-label", "From" }
                        for name in sources.iter() {
                            span { class: "kit-chat__source-chip", "{name}" }
                        }
                        span { class: "kit-chat__actions",
                            button { r#type: "button", class: "kit-chat__icon-btn", "aria-label": "Copy answer", "⧉" }
                            button { r#type: "button", class: "kit-chat__icon-btn", "aria-label": "Download answer", "↓" }
                            button { r#type: "button", class: "kit-chat__icon-btn", "aria-label": "Helpful", "+" }
                            button { r#type: "button", class: "kit-chat__icon-btn", "aria-label": "Retry answer", "↻" }
                        }
                    }
                }
                if is_waiting {
                    p { class: "kit-chat__thinking", role: "status", "Working out the answer…" }
                }
                if empty && !is_waiting {
                    div { class: "kit-chat__empty",
                        p { "Ask a question about your numbers, or start with a suggestion below." }
                    }
                }
            }
            form { class: "kit-chat__composer", onsubmit: move |e: FormEvent| {
                    e.prevent_default();
                    send();
                },
                if !suggested_prompts.is_empty() {
                    div { class: "kit-chat__prompts", role: "group", "aria-label": "Suggested questions",
                        for prompt in suggested_prompts.iter() {
                            button {
                                key: "{prompt.id}",
                                r#type: "button",
                                class: "kit-chat__action",
                                onclick: {
                                    let label = prompt.label.clone();
                                    move |_| draft.set(label.clone())
                                },
                                "{prompt.label}"
                            }
                        }
                    }
                }
                if let Some(s) = chip.clone() {
                    div { class: "kit-chat__selection",
                        span { "About {s.label}" }
                        button {
                            r#type: "button",
                            class: "kit-chat__chip-clear",
                            "aria-label": "Clear selection, {s.label}",
                            onclick: move |_| selection.set(None),
                            "Clear"
                        }
                    }
                }
                div { class: "kit-chat__box",
                textarea {
                    class: "kit-chat__input",
                    rows: "2",
                    "aria-label": "Ask a question",
                    placeholder: "{placeholder_text}",
                    value: "{draft}",
                    oninput: move |e: FormEvent| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        match e.key() {
                            Key::Enter if !e.modifiers().shift() => {
                                e.prevent_default();
                                send();
                            }
                            Key::Escape => match escape_action(selection().is_some(), !draft().is_empty()) {
                                EscapeAction::ClearSelection => selection.set(None),
                                EscapeAction::ClearText => draft.set(String::new()),
                                EscapeAction::None => {}
                            },
                            _ => {}
                        }
                    },
                }
                div { class: "kit-chat__bar",
                    div { class: "kit-chat__bar-left",
                        button { r#type: "button", class: "kit-chat__plus", "aria-label": "Add to the question", "+" }
                        span { class: "kit-chat__approval",
                            svg { class: "kit-chat__glyph", "viewBox": "0 0 16 16", "aria-hidden": "true",
                                path { d: "M5 8V3.5a1 1 0 0 1 2 0V7m0-1V2.5a1 1 0 0 1 2 0V7m0-.5V3.5a1 1 0 0 1 2 0V9l.5 2.5L10 14H6.5L4 10.5 3.2 9.2A1 1 0 0 1 5 8" }
                            }
                            "Ask first"
                        }
                    }
                    div { class: "kit-chat__bar-right",
                        span { class: "kit-chat__model-wrap",
                            select { class: "kit-chat__model", "aria-label": "Model",
                                option { "{model_label}" }
                            }
                            svg { class: "kit-chat__chevron", "viewBox": "0 0 16 16", "aria-hidden": "true",
                                path { d: "M4 6l4 4 4-4" }
                            }
                        }
                        button { r#type: "button", class: "kit-chat__mic", "aria-label": "Dictate",
                            svg { class: "kit-chat__glyph", "viewBox": "0 0 16 16", "aria-hidden": "true",
                                rect { x: "6", y: "2", width: "4", height: "8", rx: "2" }
                                path { d: "M3.5 8a4.5 4.5 0 0 0 9 0M8 12.5V14" }
                            }
                        }
                        button { r#type: "submit", class: "kit-chat__send", "aria-label": "Send question", "data-empty": draft().trim().is_empty().then_some("true"),
                            svg { class: "kit-chat__glyph", "viewBox": "0 0 16 16", "aria-hidden": "true",
                                path { d: "M8 13V3M3.5 7.5 8 3l4.5 4.5" }
                            }
                        }
                    }
                }
                }
            }
        }
    }
}
