//! Series trend chart: lines, areas, columns or a sparkline over shared x-labels,
//! with a tooltip, keyboard stepping and a highlight that hosts can link.
//!
//! The chart maths for this lane lives here as `pub fn` (scales, ticks, paths,
//! nearest point, keyboard steps, number text, treemap layout) so every chart
//! component shares one implementation.
//!
//! Design choice: `active` is `Option<Option<usize>>`. `None` leaves the
//! highlight with the component; `Some(v)` makes the host the owner, and
//! `Some(None)` means nothing is highlighted. Charts sharing one value move
//! together through `on_active_change`.
//! Design choice: the plot keeps a fixed viewBox width and scales with its
//! container; strokes use non-scaling-stroke so line weight stays constant.
//! Design choice: an earlier-period series is dashed in line mode and faded in
//! column mode, both by CSS.
//! Design choice: touch reads a point through mouseenter; a touch drag across
//! the plot is not mapped to pointer moves.
//! Not implemented: tweening between data sets. Changed values re-render at
//! their new positions; the sweep and rise motion runs on first render only.
//! Browser checks: line and area sweep on mount, column rise, guide hop, the
//! polite tooltip read-out, Left/Right/Home/End/Escape on the focused plot.

use dioxus::prelude::*;

use crate::components::metric_tile::{ChangeDirection, ChangeTone, change_tone, format_percent};
use crate::internal::KIT_CSS;
use rv2_affect::motion::number_ticker::format_value;

const CSS: Asset = asset!("/assets/series-trend-chart.css");

/// Fixed drawing width in SVG user units; the plot scales to its container.
pub const PLOT_WIDTH: f64 = 640.0;

/// Tone of a series, a segment or a tag. Colours come from the kit tokens in CSS.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeriesTone {
    #[default]
    Ink,
    Ok,
    Warn,
    Fault,
    Muted,
    /// Chart series tokens, `--kit-series-1` to `-6`; separate from status tones.
    Series1,
    Series2,
    Series3,
    Series4,
    Series5,
    Series6,
}

impl SeriesTone {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ink => "ink",
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Fault => "fault",
            Self::Muted => "muted",
            Self::Series1 => "series-1",
            Self::Series2 => "series-2",
            Self::Series3 => "series-3",
            Self::Series4 => "series-4",
            Self::Series5 => "series-5",
            Self::Series6 => "series-6",
        }
    }
}

/// An axis-aligned rectangle in plot units.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// A keyboard move across indexed items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexMove {
    Prev,
    Next,
    First,
    Last,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartVariant {
    #[default]
    Line,
    Area,
    Bar,
    Sparkline,
}

impl ChartVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Area => "area",
            Self::Bar => "bar",
            Self::Sparkline => "sparkline",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AxisMode {
    /// Value ticks on the left and dates below.
    #[default]
    On,
    /// Dates only.
    XOnly,
    Off,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrendSeries {
    pub name: String,
    /// One value per label; `None` breaks the line at that point.
    pub values: Vec<Option<f64>>,
    pub tone: SeriesTone,
    pub emphasis: bool,
    pub is_earlier: bool,
}

/// Replaces NaN and infinities with zero.
pub fn finite_or_zero(value: f64) -> f64 {
    if value.is_finite() { value } else { 0.0 }
}

/// Usage-style value: non-finite and negative inputs count as zero.
pub fn non_negative(value: f64) -> f64 {
    finite_or_zero(value).max(0.0)
}

/// Lower and upper bound of the finite values. Anchored at zero when `from_zero`.
/// Empty or all-zero input gives `(0.0, 1.0)`; a flat non-zero value gets a unit pad.
pub fn value_domain(values: impl IntoIterator<Item = f64>, from_zero: bool) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for value in values.into_iter().filter(|v| v.is_finite()) {
        lo = lo.min(value);
        hi = hi.max(value);
    }
    if !lo.is_finite() {
        return (0.0, 1.0);
    }
    if from_zero {
        lo = lo.min(0.0);
        hi = hi.max(0.0);
    }
    if (hi - lo).abs() < f64::EPSILON {
        return if lo == 0.0 {
            (0.0, 1.0)
        } else {
            (lo - 1.0, hi + 1.0)
        };
    }
    (lo, hi)
}

/// Picks the 1-2-5 step nearest to `(hi - lo) / intervals`, then rounds `[lo, hi]` outwards to it.
/// Returns `(nice_lo, nice_hi, step)`.
pub fn nice_axis(lo: f64, hi: f64, intervals: usize) -> (f64, f64, f64) {
    let intervals = intervals.max(1);
    let span = hi - lo;
    let raw = if span > 0.0 {
        span / intervals as f64
    } else {
        1.0
    };
    let magnitude = 10f64.powf(raw.log10().floor());
    let normal = raw / magnitude;
    let factor = if normal < 1.5 {
        1.0
    } else if normal < 3.0 {
        2.0
    } else if normal < 7.0 {
        5.0
    } else {
        10.0
    };
    let step = magnitude * factor;
    ((lo / step).floor() * step, (hi / step).ceil() * step, step)
}

/// Tick values from `lo` to `hi` inclusive, one per `step`.
pub fn tick_values(lo: f64, hi: f64, step: f64) -> Vec<f64> {
    if step.is_nan() || step <= 0.0 || !step.is_finite() || hi < lo {
        return vec![lo];
    }
    let count = ((hi - lo) / step).round() as usize;
    (0..=count).map(|i| lo + i as f64 * step).collect()
}

/// Maps a value to a y coordinate; the top of the range is `hi`.
pub fn project_y(value: f64, lo: f64, hi: f64, top: f64, bottom: f64) -> f64 {
    let span = hi - lo;
    if span <= 0.0 {
        return bottom;
    }
    top + (hi - value) / span * (bottom - top)
}

/// Maps point `index` of `count` across `[left, right]`. A single point sits mid-span.
pub fn project_x(index: usize, count: usize, left: f64, right: f64) -> f64 {
    if count <= 1 {
        (left + right) / 2.0
    } else {
        left + index as f64 / (count - 1) as f64 * (right - left)
    }
}

/// Centre of column slot `index` when `count` slots share `[left, right]`.
pub fn slot_center(index: usize, count: usize, left: f64, right: f64) -> f64 {
    let width = (right - left) / count.max(1) as f64;
    left + (index as f64 + 0.5) * width
}

/// Contiguous runs of present points; a `None` ends a run.
fn runs(points: &[Option<(f64, f64)>]) -> Vec<Vec<(f64, f64)>> {
    let mut out: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut current: Vec<(f64, f64)> = Vec::new();
    for point in points {
        match point {
            Some(p) => current.push(*p),
            None => {
                if !current.is_empty() {
                    out.push(std::mem::take(&mut current));
                }
            }
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// SVG path through the points, lifting the pen at each `None`.
pub fn line_path(points: &[Option<(f64, f64)>]) -> String {
    let mut d = String::new();
    for run in runs(points) {
        for (i, (x, y)) in run.iter().enumerate() {
            let cmd = if i == 0 { "M" } else { "L" };
            d.push_str(&format!("{cmd}{x:.2} {y:.2} "));
        }
    }
    d.trim_end().to_string()
}

/// Filled area under each contiguous run, closed on `baseline`.
pub fn area_path(points: &[Option<(f64, f64)>], baseline: f64) -> String {
    let mut d = String::new();
    for run in runs(points) {
        let (first_x, _) = run[0];
        let (last_x, _) = run[run.len() - 1];
        for (i, (x, y)) in run.iter().enumerate() {
            let cmd = if i == 0 { "M" } else { "L" };
            d.push_str(&format!("{cmd}{x:.2} {y:.2} "));
        }
        d.push_str(&format!(
            "L{last_x:.2} {baseline:.2} L{first_x:.2} {baseline:.2} Z "
        ));
    }
    d.trim_end().to_string()
}

/// Labels drawn on the date axis: first, middle and last. Fewer points draw fewer.
pub fn axis_indices(count: usize) -> Vec<usize> {
    match count {
        0 => Vec::new(),
        1 => vec![0],
        2 => vec![0, 1],
        _ => vec![0, (count - 1) / 2, count - 1],
    }
}

/// Index of the point nearest to `x` among `count` points across `[left, right]`.
pub fn nearest_index(x: f64, count: usize, left: f64, right: f64) -> Option<usize> {
    if count == 0 {
        return None;
    }
    if count == 1 || right <= left {
        return Some(0);
    }
    let raw = ((x - left) / (right - left) * (count - 1) as f64).round();
    Some(raw.clamp(0.0, (count - 1) as f64) as usize)
}

/// Clamped keyboard move. With nothing highlighted, Next and First give the first item,
/// Prev and Last give the last.
pub fn step_index(current: Option<usize>, count: usize, movement: IndexMove) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let last = count - 1;
    let current = current.map(|i| i.min(last));
    Some(match (movement, current) {
        (IndexMove::First, _) => 0,
        (IndexMove::Last, _) => last,
        (IndexMove::Next, Some(i)) => (i + 1).min(last),
        (IndexMove::Prev, Some(i)) => i.saturating_sub(1),
        (IndexMove::Next, None) => 0,
        (IndexMove::Prev, None) => last,
    })
}

/// Wrapping move for radio-style groups. `count` must be non-zero.
pub fn wrap_index(current: usize, count: usize, forward: bool) -> usize {
    if count == 0 {
        return 0;
    }
    if forward {
        (current + 1) % count
    } else {
        (current + count - 1) % count
    }
}

/// Start and end of each value when the values are stacked in order.
pub fn stack_bounds(values: &[f64]) -> Vec<(f64, f64)> {
    let mut acc = 0.0;
    values
        .iter()
        .map(|value| {
            let start = acc;
            acc += non_negative(*value);
            (start, acc)
        })
        .collect()
}

/// `part / total` as a ratio; zero when the total is not positive.
pub fn share(part: f64, total: f64) -> f64 {
    if total > 0.0 {
        non_negative(part) / total
    } else {
        0.0
    }
}

/// Whole-percent label; a present but tiny share reads `<1%`.
pub fn share_label(ratio: f64) -> String {
    let pct = finite_or_zero(ratio) * 100.0;
    if ratio > 0.0 && pct < 1.0 {
        "<1%".to_string()
    } else {
        format!("{pct:.0}%")
    }
}

/// Percent change with an explicit sign: `+4.5%`, `-7.5%`, `0.0%`.
pub fn signed_percent(value: f64) -> String {
    let value = finite_or_zero(value);
    let sign = if value > 0.0 {
        "+"
    } else if value < 0.0 {
        "-"
    } else {
        ""
    };
    format!("{sign}{:.1}%", value.abs())
}

/// Short magnitude: `999`, `1.2k`, `640k`, `8.6M`, `3.1B`.
pub fn compact_number(value: f64) -> String {
    let value = finite_or_zero(value);
    let magnitude = value.abs();
    if magnitude < 1_000.0 {
        return format!("{value:.0}");
    }
    let (scaled, unit) = if magnitude >= 1e9 {
        (value / 1e9, "B")
    } else if magnitude >= 1e6 {
        (value / 1e6, "M")
    } else {
        (value / 1e3, "k")
    };
    let digits = if scaled.abs() >= 100.0 { 0 } else { 1 };
    let text = format!("{scaled:.digits$}");
    let text = if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text
    };
    format!("{text}{unit}")
}

/// Whole number with thousands separators: `1,284`.
pub fn group_digits(value: f64) -> String {
    format_value(finite_or_zero(value), 0, true)
}

/// Splits each range where its weight is closest to half, alternating the cut
/// direction with the longer side. Keeps the input order; zero weights get zero rects.
pub fn treemap_layout(sizes: &[f64], area: Rect) -> Vec<Rect> {
    let weights: Vec<f64> = sizes.iter().map(|s| non_negative(*s)).collect();
    let mut out = vec![Rect::default(); weights.len()];
    split_range(&weights, 0, weights.len(), area, &mut out);
    out
}

fn split_range(weights: &[f64], lo: usize, hi: usize, area: Rect, out: &mut [Rect]) {
    match hi - lo {
        0 => return,
        1 => {
            out[lo] = area;
            return;
        }
        _ => {}
    }
    let total: f64 = weights[lo..hi].iter().sum();
    if total <= 0.0 {
        return;
    }
    let mut cut = lo + 1;
    let mut best_gap = f64::INFINITY;
    let mut acc = 0.0;
    for k in lo + 1..hi {
        acc += weights[k - 1];
        let gap = (acc - total / 2.0).abs();
        if gap < best_gap {
            best_gap = gap;
            cut = k;
        }
    }
    let left: f64 = weights[lo..cut].iter().sum();
    let frac = left / total;
    let (first, second) = if area.w >= area.h {
        let w1 = area.w * frac;
        (
            Rect { w: w1, ..area },
            Rect {
                x: area.x + w1,
                w: area.w - w1,
                ..area
            },
        )
    } else {
        let h1 = area.h * frac;
        (
            Rect { h: h1, ..area },
            Rect {
                y: area.y + h1,
                h: area.h - h1,
                ..area
            },
        )
    };
    split_range(weights, lo, cut, first, out);
    split_range(weights, cut, hi, second, out);
}

/// Moves keyboard focus to the element with `id`. Call from an event handler.
pub fn focus_element(id: &str) {
    document::eval(&format!("document.getElementById({id:?})?.focus();"));
}

/// One sparkline tile under a `SeriesTrendBlock`: its label, last value and sparkline.
#[derive(Clone, Debug, PartialEq)]
pub struct SparkTile {
    pub label: String,
    /// One value per block label; `None` breaks the line at that point.
    pub values: Vec<Option<f64>>,
    pub tone: SeriesTone,
}

const MODE_OPTIONS: [(ChartVariant, &str); 3] = [
    (ChartVariant::Line, "Line"),
    (ChartVariant::Area, "Area"),
    (ChartVariant::Bar, "Column"),
];

fn last_value(values: &[Option<f64>]) -> Option<f64> {
    values.last().copied().flatten().filter(|v| v.is_finite())
}

/// A headline block: a mode control, the latest current-period value with its change
/// against the previous period, the trend plot and a row of sparkline tiles.
///
/// The plot rests on the last label with no tooltip open. The headline states both
/// periods as text, so the resting view never depends on a popover.
#[component]
pub fn SeriesTrendBlock(
    #[props(into)] title: String,
    #[props(into)] label: String,
    x_labels: Vec<String>,
    series: Vec<TrendSeries>,
    #[props(default = ChartVariant::Area)] initial_variant: ChartVariant,
    #[props(default = group_digits as fn(f64) -> String)] fmt_value: fn(f64) -> String,
    sparklines: Vec<SparkTile>,
) -> Element {
    let mut mode = use_signal(|| initial_variant);
    let last = x_labels.len().checked_sub(1);
    let last_label = last.map(|i| x_labels[i].clone()).unwrap_or_default();
    let current = series.iter().find(|s| !s.is_earlier);
    let earlier = series.iter().find(|s| s.is_earlier);
    let current_name = current.map(|s| s.name.clone()).unwrap_or_default();
    let earlier_name = earlier.map(|s| s.name.clone()).unwrap_or_default();
    let now = current.and_then(|s| last_value(&s.values));
    let before = earlier.and_then(|s| last_value(&s.values));
    let figure = now.map(fmt_value).unwrap_or_else(|| "–".to_string());
    let previous_text = before.map(fmt_value).unwrap_or_else(|| "–".to_string());
    let change = match (now, before) {
        (Some(now), Some(before)) if before != 0.0 => Some((now - before) / before * 100.0),
        _ => None,
    };
    let (direction, tone) = change_tone(change.unwrap_or(0.0), false);
    let arrow = match direction {
        ChangeDirection::Up => "↑",
        ChangeDirection::Down => "↓",
        ChangeDirection::Flat => "→",
    };
    let badge = change.map(|c| format!("{arrow} {}", format_percent(c.abs())));
    let tone_name = match tone {
        ChangeTone::Favourable => "favourable",
        ChangeTone::Unfavourable => "unfavourable",
        ChangeTone::Neutral => "neutral",
    };
    let caption = format!("{current_name} at {last_label}");
    let previous = format!("{earlier_name} {previous_text} at {last_label}");
    let spark_views: Vec<(String, String, TrendSeries)> = sparklines
        .into_iter()
        .map(|tile| {
            let headline = last_value(&tile.values)
                .map(fmt_value)
                .unwrap_or_else(|| "–".to_string());
            let series = TrendSeries {
                name: tile.label.clone(),
                values: tile.values,
                tone: tile.tone,
                emphasis: true,
                is_earlier: false,
            };
            (tile.label, headline, series)
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-series-block",
            header { class: "kit-series-block__head",
                h3 { class: "kit-series-block__title", "{title}" }
                div { class: "kit-series-block__modes", role: "group", "aria-label": "Chart mode",
                    for (variant, text) in MODE_OPTIONS {
                        button {
                            key: "{text}",
                            r#type: "button",
                            class: "kit-series-block__mode",
                            "aria-pressed": if mode() == variant { "true" } else { "false" },
                            onclick: move |_| mode.set(variant),
                            "{text}"
                        }
                    }
                }
            }
            div { class: "kit-series-block__headline",
                div { class: "kit-series-block__figure-wrap",
                    span { class: "kit-series-block__caption", "{caption}" }
                    span { class: "kit-series-block__figure", "{figure}" }
                }
                if let Some(text) = badge {
                    span { class: "kit-series-block__badge", "data-tone": tone_name, "{text}" }
                }
                span { class: "kit-series-block__previous", "{previous}" }
            }
            SeriesTrendChart {
                x_labels: x_labels.clone(),
                series: series.clone(),
                variant: mode(),
                label: label,
                height: Some(220.0),
                initial_active: last,
                fmt_value: fmt_value,
            }
            div { class: "kit-series-block__sparks",
                for (idx, (name, headline, spark)) in spark_views.into_iter().enumerate() {
                    div { key: "{idx}", class: "kit-series-block__spark",
                        div { class: "kit-series-block__spark-head",
                            span { class: "kit-series-block__spark-label", "{name}" }
                            span { class: "kit-series-block__spark-value", "{headline}" }
                        }
                        SeriesTrendChart {
                            x_labels: x_labels.clone(),
                            series: vec![spark],
                            variant: ChartVariant::Sparkline,
                            label: name.clone(),
                            initial_active: last,
                            fmt_value: fmt_value,
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct PathShape {
    key: String,
    d: String,
    tone: &'static str,
    class: String,
}

#[derive(Clone, PartialEq, Debug)]
struct Column {
    key: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    tone: &'static str,
    class: String,
}

#[derive(Clone, PartialEq, Debug)]
struct Marker {
    key: String,
    cx: f64,
    cy: f64,
    tone: &'static str,
}

#[derive(Clone, PartialEq, Debug)]
struct Hit {
    index: usize,
    x: f64,
    w: f64,
}

/// A line, area, column or sparkline over shared labels.
///
/// Missing or non-finite values break the line. Nothing but zeros (or nothing at
/// all) shows `empty_message` and no axis. Keyboard: Tab enters the plot, Left and
/// Right step, Home and End jump, Escape hides the tooltip.
///
/// A locally owned highlight (`initial_active`) rests without a tooltip; the tooltip
/// opens on hover, focus or a key step. A host-owned `active` shows its tooltip at once.
#[component]
pub fn SeriesTrendChart(
    x_labels: Vec<String>,
    #[props(default)] tooltip_headings: Vec<String>,
    series: Vec<TrendSeries>,
    #[props(default)] variant: ChartVariant,
    #[props(default = group_digits as fn(f64) -> String)] fmt_value: fn(f64) -> String,
    #[props(default)] fmt_axis: Option<fn(f64) -> String>,
    /// Text placed before each tooltip value, such as a currency symbol.
    #[props(default, into)]
    value_prefix: String,
    #[props(default)] height: Option<f64>,
    #[props(default)] axis: Option<AxisMode>,
    #[props(default = true)] show_tooltip: bool,
    #[props(default = true)] from_zero: bool,
    #[props(default)] active: Option<Option<usize>>,
    #[props(default)] initial_active: Option<usize>,
    #[props(default)] on_active_change: Option<EventHandler<Option<usize>>>,
    #[props(default = "Chart".to_string(), into)] label: String,
    #[props(default = "No data".to_string(), into)] empty_message: String,
) -> Element {
    let mut local = use_signal(|| None::<usize>);
    let mut tip_hidden = use_signal(|| false);
    let mut engaged = use_signal(|| false);

    let host_owned = active.is_some();
    let current: Option<usize> = match active {
        Some(value) => value,
        None => local().or(initial_active),
    };
    let n = x_labels.len();
    let sparkline = variant == ChartVariant::Sparkline;
    let is_bar = variant == ChartVariant::Bar;
    let axis_mode = axis.unwrap_or(if sparkline {
        AxisMode::Off
    } else {
        AxisMode::On
    });
    let show_axis_y = axis_mode == AxisMode::On && !sparkline;
    let show_axis_x = axis_mode != AxisMode::Off && !sparkline;
    let has_data = series.iter().any(|s| {
        s.values
            .iter()
            .flatten()
            .any(|v| v.is_finite() && *v != 0.0)
    });
    let visible = has_data && n > 0;

    let height = height.unwrap_or(if sparkline { 56.0 } else { 240.0 });
    let left = 4.0;
    let right = PLOT_WIDTH - 8.0;
    let top = 8.0;
    let bottom = if show_axis_x {
        height - 22.0
    } else {
        height - 4.0
    };

    let values = series
        .iter()
        .flat_map(|s| s.values.iter().flatten().copied());
    let (lo, hi) = value_domain(values, from_zero || is_bar);
    let (nlo, nhi, step) = nice_axis(lo, hi, 4);
    let y_of = |v: f64| project_y(v, nlo, nhi, top, bottom);
    let x_of = |i: usize| project_x(i, n, left, right);
    let slot_w = (right - left) / n.max(1) as f64;
    let baseline = y_of(0.0_f64.clamp(nlo, nhi));

    let mut shapes: Vec<PathShape> = Vec::new();
    let mut columns: Vec<Column> = Vec::new();
    let mut markers: Vec<Marker> = Vec::new();
    let mut rows: Vec<(String, String, &'static str)> = Vec::new();
    let column_count = series.len().max(1) as f64;
    if visible {
        for (j, s) in series.iter().enumerate() {
            let tone = s.tone.as_str();
            let points: Vec<Option<(f64, f64)>> = s
                .values
                .iter()
                .enumerate()
                .map(|(i, v)| (*v).filter(|v| v.is_finite()).map(|v| (x_of(i), y_of(v))))
                .collect();
            if is_bar {
                let width = slot_w * 0.7 / column_count;
                for (i, v) in s.values.iter().enumerate() {
                    if let Some(value) = (*v).filter(|v| v.is_finite()) {
                        let top_y = y_of(value);
                        columns.push(Column {
                            key: format!("{j}-{i}"),
                            x: left + i as f64 * slot_w + slot_w * 0.15 + j as f64 * width,
                            y: top_y.min(baseline),
                            w: (width - 1.0).max(1.0),
                            h: (baseline - top_y).abs(),
                            tone,
                            class: if s.is_earlier {
                                "kit-trend__column kit-trend__column--earlier".to_string()
                            } else {
                                "kit-trend__column".to_string()
                            },
                        });
                    }
                }
            } else {
                let earlier = if s.is_earlier {
                    " kit-trend__stroke--earlier"
                } else {
                    ""
                };
                let emphasis = if s.emphasis {
                    " kit-trend__stroke--emphasis"
                } else {
                    ""
                };
                if variant == ChartVariant::Area {
                    shapes.push(PathShape {
                        key: format!("{j}-fill"),
                        d: area_path(&points, baseline),
                        tone,
                        class: "kit-trend__area".to_string(),
                    });
                }
                shapes.push(PathShape {
                    key: format!("{j}-line"),
                    d: line_path(&points),
                    tone,
                    class: format!("kit-trend__stroke{earlier}{emphasis}"),
                });
            }
            if let Some(i) = current.filter(|i| *i < n) {
                if let Some(value) = s.values.get(i).copied().flatten().filter(|v| v.is_finite()) {
                    let cx = if is_bar {
                        slot_center(i, n, left, right)
                    } else {
                        x_of(i)
                    };
                    markers.push(Marker {
                        key: format!("{j}-marker"),
                        cx,
                        cy: y_of(value),
                        tone,
                    });
                }
            }
            let value_text = current
                .and_then(|i| s.values.get(i).copied().flatten())
                .filter(|v| v.is_finite())
                .map(|v| format!("{value_prefix}{}", fmt_value(v)))
                .unwrap_or_else(|| "–".to_string());
            rows.push((s.name.clone(), value_text, tone));
        }
    }

    let heading = current
        .filter(|i| *i < n)
        .map(|i| {
            tooltip_headings
                .get(i)
                .cloned()
                .unwrap_or_else(|| x_labels[i].clone())
        })
        .unwrap_or_default();
    let tag_text = match rows.first() {
        Some((_, value, _)) => format!("{heading}: {value}"),
        None => heading.clone(),
    };
    let hits: Vec<Hit> = (0..n)
        .map(|i| {
            if is_bar {
                Hit {
                    index: i,
                    x: left + i as f64 * slot_w,
                    w: slot_w,
                }
            } else {
                let span = if n > 1 {
                    (right - left) / (n - 1) as f64
                } else {
                    right - left
                };
                Hit {
                    index: i,
                    x: x_of(i) - span / 2.0,
                    w: span,
                }
            }
        })
        .collect();
    let guide_x = current.filter(|i| *i < n).map(|i| {
        if is_bar {
            slot_center(i, n, left, right)
        } else {
            x_of(i)
        }
    });
    let y_labels: Vec<(f64, String)> = if show_axis_y {
        let fmt: fn(f64) -> String = fmt_axis.unwrap_or(fmt_value);
        tick_values(nlo, nhi, step)
            .into_iter()
            .map(|t| (y_of(t), fmt(t)))
            .collect()
    } else {
        Vec::new()
    };
    let x_ticks: Vec<(usize, f64, String, &'static str)> = if show_axis_x && visible {
        axis_indices(n)
            .into_iter()
            .map(|i| {
                let anchor = if i == 0 {
                    "start"
                } else if i == n - 1 {
                    "end"
                } else {
                    "middle"
                };
                let x = if is_bar {
                    slot_center(i, n, left, right)
                } else {
                    x_of(i)
                };
                (i, x, x_labels[i].clone(), anchor)
            })
            .collect()
    } else {
        Vec::new()
    };

    let y_tick_styles: Vec<(usize, String, String)> = y_labels
        .iter()
        .enumerate()
        .map(|(k, (y, text))| (k, format!("top: {:.3}%", y / height * 100.0), text.clone()))
        .collect();
    let x_tick_styles: Vec<(usize, String, String, &'static str)> = x_ticks
        .iter()
        .map(|(i, x, text, anchor)| {
            (
                *i,
                format!("left: {:.3}%; top: {:.3}%", x / PLOT_WIDTH * 100.0, 100.0),
                text.clone(),
                *anchor,
            )
        })
        .collect();
    let gap_styles: Vec<(usize, String)> = if visible && !sparkline {
        (0..n)
            .filter(|&i| {
                series.iter().any(|s| {
                    !s.values
                        .get(i)
                        .copied()
                        .flatten()
                        .is_some_and(f64::is_finite)
                })
            })
            .enumerate()
            .map(|(k, i)| (k, format!("left: {:.3}%", x_of(i) / PLOT_WIDTH * 100.0)))
            .collect()
    } else {
        Vec::new()
    };
    let tooltip_on =
        show_tooltip && visible && !tip_hidden() && current.is_some() && (host_owned || engaged());
    let view_box = format!("0 0 {PLOT_WIDTH} {height}");
    let variant_name = variant.as_str();
    let label_text = label.clone();
    let mid_index = if n > 2 { Some((n - 1) / 2) } else { None };
    let mut report = move |next: Option<usize>| {
        if !host_owned {
            local.set(next);
        }
        if let Some(handler) = &on_active_change {
            handler.call(next);
        }
    };

    let legend: Vec<(String, &'static str)> = series
        .iter()
        .map(|s| (s.name.clone(), s.tone.as_str()))
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-trend", "data-variant": variant_name, "data-empty": (!visible).then_some("true"),
            div { class: "kit-sr", role: "status", if !visible { "{empty_message}" } }
            if !visible {
                div { class: "kit-trend__empty", "{empty_message}" }
            } else {
                div {
                    class: "kit-trend__plot",
                    role: "group",
                    tabindex: "0",
                    "aria-label": label_text,
                    "aria-roledescription": "chart",
                    onfocus: move |_| engaged.set(true),
                    onblur: move |_| engaged.set(false),
                    onkeydown: move |event: KeyboardEvent| {
                        let movement = match event.key() {
                            Key::ArrowLeft => Some(IndexMove::Prev),
                            Key::ArrowRight => Some(IndexMove::Next),
                            Key::Home => Some(IndexMove::First),
                            Key::End => Some(IndexMove::Last),
                            Key::Escape => {
                                tip_hidden.set(true);
                                None
                            }
                            _ => None,
                        };
                        if let Some(movement) = movement {
                            event.prevent_default();
                            tip_hidden.set(false);
                            engaged.set(true);
                            report(step_index(current, n, movement));
                        }
                    },
                    div { class: if show_axis_y { "kit-trend__canvas kit-trend__canvas--axis" } else { "kit-trend__canvas" },
                    if show_axis_y {
                        div { class: "kit-trend__ticks", "aria-hidden": "true",
                            for (idx, style, text) in y_tick_styles.clone() {
                                span { key: "{idx}", class: "kit-trend__tick", style: style, "{text}" }
                            }
                        }
                    }
                    svg {
                        class: "kit-trend__svg",
                        view_box: view_box,
                        "aria-hidden": "true",
                        onmouseleave: move |_| {
                            engaged.set(false);
                            report(None);
                        },
                        for (idx, (y, _)) in y_labels.clone().into_iter().enumerate() {
                            line { key: "{idx}", class: "kit-trend__grid", x1: left, x2: right, y1: y, y2: y }
                        }
                        if show_axis_x {
                            line { class: "kit-trend__baseline", x1: left, x2: right, y1: bottom, y2: bottom }
                        }
                        for shape in shapes.clone() {
                            path { key: "{shape.key}", class: shape.class.clone(), d: shape.d.clone(), "data-tone": shape.tone }
                        }
                        for column in columns.clone() {
                            rect { key: "{column.key}", class: column.class.clone(), x: column.x, y: column.y, width: column.w, height: column.h, "data-tone": column.tone }
                        }
                        if let Some(gx) = guide_x {
                            line { class: "kit-trend__guide", x1: gx, x2: gx, y1: top, y2: bottom }
                        }
                        for marker in markers.clone() {
                            circle { key: "{marker.key}", class: "kit-trend__marker", cx: marker.cx, cy: marker.cy, r: 3.5, "data-tone": marker.tone }
                        }
                        for hit in hits.clone() {
                            rect {
                                key: "h{hit.index}",
                                class: "kit-trend__hit",
                                x: hit.x,
                                y: top,
                                width: hit.w,
                                height: bottom - top,
                                onmouseenter: move |_| {
                                    tip_hidden.set(false);
                                    engaged.set(true);
                                    report(Some(hit.index));
                                },
                            }
                        }
                    }
                    if show_axis_x {
                        div { class: "kit-trend__dates", "aria-hidden": "true",
                            for (i, style, text, anchor) in x_tick_styles.clone() {
                                span {
                                    key: "x{i}",
                                    class: if Some(i) == mid_index { "kit-trend__date kit-trend__date--mid" } else { "kit-trend__date" },
                                    style: style,
                                    "data-anchor": anchor,
                                    "{text}"
                                }
                            }
                        }
                    }
                    for (idx, style) in gap_styles.clone() {
                        span { key: "gap{idx}", class: "kit-trend__gap", style: style, "No data" }
                    }
                    }
                    if legend.len() > 1 {
                        ul { class: "kit-trend__legend",
                            for (idx, (name, tone)) in legend.clone().into_iter().enumerate() {
                                li { key: "{idx}", class: "kit-trend__legend-item",
                                    span { class: "kit-trend__swatch", "data-tone": tone }
                                    "{name}"
                                }
                            }
                        }
                    }
                    div { role: "status", "aria-live": "polite", class: "kit-trend__live",
                        if tooltip_on {
                            if sparkline {
                                div { class: "kit-trend__tag", "{tag_text}" }
                            } else {
                                div { class: "kit-trend__tooltip",
                                    div { class: "kit-trend__heading", "{heading}" }
                                    for (idx, (name, value, tone)) in rows.clone().into_iter().enumerate() {
                                        div { key: "{idx}", class: "kit-trend__row",
                                            span { class: "kit-trend__swatch", "data-tone": tone }
                                            span { class: "kit-trend__name", "{name}" }
                                            span { class: "kit-trend__value", "{value}" }
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
