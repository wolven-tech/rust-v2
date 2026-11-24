//! Metric trend card: a headline figure, its change against the previous period, and a
//! multi-line chart of its parts. Hover, touch or the arrow keys move a highlight across
//! the points. The tooltip is the only readout: every series at that point, ranked by value.
//!
//! Design choice: an empty card means no finite value at all. An all-zero series is data
//! and draws a flat line.
//! Design choice: the highlight is the pointer's point, else the keyboard's point, else
//! `resting_index` (default: the last point). Leaving the plot clears both.
//! Design choice: axis labels are HTML, not SVG text, so the SVG's scaling never shrinks
//! them below the kit's 11px floor.
//! Design choice: series colours come from `--kit-series-*` by tone slot: ink 1, ok 2,
//! warn 3, fault 4, muted 5.
//! Not implemented: the guide and marker hop animation; they move without easing.
//! Browser checks: lines draw in from the left on mount, reduced motion skips it, Escape
//! hides the tooltip while the plot keeps focus, the tooltip read-out is spoken politely.

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    IndexMove, SeriesTone, axis_indices, compact_number, line_path, nice_axis, non_negative,
    project_x, project_y, step_index, tick_values, value_domain,
};
use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/metric-trend-card.css");

const WIDTH: f64 = 640.0;
const HEIGHT: f64 = 170.0;
const PLOT_TOP: f64 = 10.0;
const PLOT_BOTTOM: f64 = 160.0;
const PLOT_LEFT: f64 = 40.0;
const PLOT_RIGHT: f64 = 632.0;

#[derive(Clone, Debug, PartialEq)]
pub struct MetricSeries {
    pub series_name: String,
    pub series_tone: SeriesTone,
    pub series_values: Vec<f64>,
}

/// Indices of `values` ordered from highest to lowest. `None` ranks last; ties keep input order.
pub fn ranked_indices(values: &[Option<f64>]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|a, b| match (values[*a], values[*b]) {
        (Some(x), Some(y)) => y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    order
}

/// Sign and tone of the change chip: a positive change uses the success tone, a negative
/// one the error tone, and zero is neutral.
pub fn change_chip(change: f64) -> (&'static str, &'static str) {
    if change > 0.0 {
        ("+", "ok")
    } else if change < 0.0 {
        ("-", "fault")
    } else {
        ("", "muted")
    }
}

/// A headline figure, its signed change, and a multi-series line chart with a hover and
/// keyboard tooltip. `headline_value` is already formatted; `format_value` formats the
/// tooltip and change chip. `change_suffix` follows the change figure, for example `%`.
#[component]
pub fn MetricTrendCard(
    #[props(into)] metric_name: String,
    #[props(into)] headline_value: String,
    change_value: f64,
    axis_labels: Vec<String>,
    #[props(default)] tooltip_labels: Vec<String>,
    data_series: Vec<MetricSeries>,
    #[props(default = compact_number as fn(f64) -> String)] format_value: fn(f64) -> String,
    #[props(default, into)] change_suffix: String,
    #[props(default)] footer_text: Option<String>,
    #[props(default)] resting_index: Option<usize>,
    #[props(default = "No data yet".to_string(), into)] empty_message: String,
    /// Visible words after the change chip, for example "vs last period"; empty draws none.
    #[props(default, into)]
    comparison_text: String,
    /// Whether the tooltip for the resting point is drawn before any pointer or key use.
    #[props(default = true)]
    tooltip_at_rest: bool,
    #[props(default, into)] extra_class: String,
) -> Element {
    let mut hovered = use_signal(|| None::<usize>);
    let mut keyed = use_signal(|| None::<usize>);
    let mut tip_hidden = use_signal(|| false);

    let n = axis_labels.len();
    let last = n.saturating_sub(1);
    let has_values = data_series
        .iter()
        .any(|s| s.series_values.iter().any(|v| v.is_finite()));
    let visible = has_values && n > 0;
    let resting = resting_index.unwrap_or(last).min(last);
    let current = hovered().or(keyed()).unwrap_or(resting).min(last);

    let domain_values = data_series
        .iter()
        .flat_map(|s| s.series_values.iter().copied().filter(|v| v.is_finite()));
    let (lo, hi) = value_domain(domain_values, true);
    let (nlo, nhi, step) = nice_axis(lo, hi, 3);
    let y_of = |v: f64| project_y(v, nlo, nhi, PLOT_TOP, PLOT_BOTTOM);
    let x_of = |i: usize| project_x(i, n, PLOT_LEFT, PLOT_RIGHT);

    let mut lines: Vec<(String, &'static str, String)> = Vec::new();
    let mut markers: Vec<(String, f64, f64, &'static str)> = Vec::new();
    let mut rows: Vec<(String, String, &'static str)> = Vec::new();
    if visible {
        for (j, series) in data_series.iter().enumerate() {
            let tone = series.series_tone.as_str();
            let points: Vec<Option<(f64, f64)>> = series
                .series_values
                .iter()
                .enumerate()
                .map(|(i, v)| v.is_finite().then(|| (x_of(i), y_of(*v))))
                .collect();
            lines.push((format!("line-{j}"), tone, line_path(&points)));
            if let Some(value) = series
                .series_values
                .get(current)
                .copied()
                .filter(|v| v.is_finite())
            {
                markers.push((format!("marker-{j}"), x_of(current), y_of(value), tone));
            }
        }
        let at_point: Vec<Option<f64>> = data_series
            .iter()
            .map(|s| {
                s.series_values
                    .get(current)
                    .copied()
                    .filter(|v| v.is_finite())
            })
            .collect();
        let value_text = |index: usize| {
            at_point[index]
                .map(format_value)
                .unwrap_or_else(|| "–".to_string())
        };
        for index in ranked_indices(&at_point) {
            let series = &data_series[index];
            rows.push((
                series.series_name.clone(),
                value_text(index),
                series.series_tone.as_str(),
            ));
        }
    }

    let heading = tooltip_labels
        .get(current)
        .or_else(|| axis_labels.get(current))
        .cloned()
        .unwrap_or_default();
    let guide_x = x_of(current);
    let gridlines: Vec<(usize, f64)> = if visible {
        tick_values(nlo, nhi, step)
            .into_iter()
            .map(y_of)
            .enumerate()
            .collect()
    } else {
        Vec::new()
    };
    let date_labels: Vec<(usize, String, String, &'static str)> = if visible {
        axis_indices(n)
            .into_iter()
            .map(|i| {
                let anchor = if i == 0 {
                    "start"
                } else if i == last {
                    "end"
                } else {
                    "middle"
                };
                let left = format!("left: {:.2}%", x_of(i) / WIDTH * 100.0);
                (i, left, axis_labels[i].clone(), anchor)
            })
            .collect()
    } else {
        Vec::new()
    };
    let hits: Vec<(usize, f64, f64)> = (0..n)
        .map(|i| {
            let span = if n > 1 {
                (PLOT_RIGHT - PLOT_LEFT) / (n - 1) as f64
            } else {
                PLOT_RIGHT - PLOT_LEFT
            };
            (i, x_of(i) - span / 2.0, span)
        })
        .collect();

    let (sign, chip_tone) = change_chip(change_value);
    let chip = format!(
        "{sign}{}{change_suffix}",
        format_value(non_negative(change_value.abs()))
    );
    let chip_words = if change_value > 0.0 {
        format!(
            "up {}{change_suffix} vs previous period",
            format_value(change_value)
        )
    } else if change_value < 0.0 {
        format!(
            "down {}{change_suffix} vs previous period",
            format_value(change_value.abs())
        )
    } else {
        "no change vs previous period".to_string()
    };
    let class = class_list("kit-metric-card", &extra_class);
    let view_box = format!("0 0 {WIDTH} {HEIGHT}");
    let engaged = hovered().is_some() || keyed().is_some();
    let tooltip_on = visible && !tip_hidden() && (tooltip_at_rest || engaged);
    let name_label = metric_name.clone();
    let footer = footer_text.clone();
    let empty_text = empty_message.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": name_label.clone(), "data-empty": (!visible).then_some("true"),
            div { class: "kit-metric-card__name", "{name_label}" }
            div { class: "kit-metric-card__headline",
                span { class: "kit-metric-card__value", "{headline_value}" }
                span { class: "kit-metric-card__chip", "data-tone": chip_tone, "aria-hidden": "true", "{chip}" }
                if !comparison_text.is_empty() {
                    span { class: "kit-metric-card__vs", "{comparison_text}" }
                }
                span { class: "kit-metric-card__sr", "{chip_words}" }
            }
            div { class: "kit-sr", role: "status", if !visible { "{empty_text}" } }
            if !visible {
                div { class: "kit-metric-card__empty", "{empty_text}" }
            } else {
                div { class: "kit-metric-card__body",
                    div {
                        class: "kit-metric-card__plot",
                        tabindex: "0",
                        role: "group",
                        "aria-label": name_label.clone(),
                        "aria-roledescription": "chart",
                        onkeydown: move |event: KeyboardEvent| {
                            let movement = match event.key() {
                                Key::ArrowLeft => Some(IndexMove::Prev),
                                Key::ArrowRight => Some(IndexMove::Next),
                                Key::Escape => {
                                    tip_hidden.set(true);
                                    None
                                }
                                _ => None,
                            };
                            if let Some(movement) = movement {
                                event.prevent_default();
                                tip_hidden.set(false);
                                keyed.set(step_index(Some(current), n, movement));
                            }
                        },
                        onmouseleave: move |_| {
                            hovered.set(None);
                            keyed.set(None);
                        },
                        svg { class: "kit-metric-card__svg", view_box: view_box, "aria-hidden": "true",
                            for (idx, y) in gridlines.clone() {
                                line { key: "g{idx}", class: "kit-metric-card__grid", x1: PLOT_LEFT, x2: PLOT_RIGHT, y1: y, y2: y }
                            }
                            for (key, tone, d) in lines.clone() {
                                path { key: "{key}", class: "kit-metric-card__line", d: d, "data-tone": tone }
                            }
                            line { class: "kit-metric-card__guide", x1: guide_x, x2: guide_x, y1: PLOT_TOP, y2: PLOT_BOTTOM }
                            for (key, cx, cy, tone) in markers.clone() {
                                circle { key: "{key}", class: "kit-metric-card__marker", cx: cx, cy: cy, r: 4.0, "data-tone": tone }
                            }
                            for (i, x, width) in hits.clone() {
                                rect {
                                    key: "h{i}",
                                    class: "kit-metric-card__hit",
                                    x: x,
                                    y: PLOT_TOP,
                                    width: width,
                                    height: PLOT_BOTTOM - PLOT_TOP,
                                    onmouseenter: move |_| {
                                        tip_hidden.set(false);
                                        hovered.set(Some(i));
                                    },
                                }
                            }
                        }
                        div { class: "kit-metric-card__axis", "aria-hidden": "true",
                            for (i, left, text, anchor) in date_labels.clone() {
                                span { key: "d{i}", class: "kit-metric-card__date", "data-anchor": anchor, style: left, "{text}" }
                            }
                        }
                    }
                    div { class: "kit-metric-card__panel",
                        div { role: "status", "aria-live": "polite", class: "kit-metric-card__live",
                            if tooltip_on {
                                div { class: "kit-metric-card__tooltip",
                                    div { class: "kit-metric-card__heading", "{heading}" }
                                    for (idx, (name, value, tone)) in rows.clone().into_iter().enumerate() {
                                        div { key: "{idx}", class: "kit-metric-card__row",
                                            span { class: "kit-metric-card__swatch", "data-tone": tone }
                                            span { class: "kit-metric-card__name-cell", "{name}" }
                                            span { class: "kit-metric-card__value-cell", "{value}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(text) = footer {
                p { class: "kit-metric-card__footer", "{text}" }
            }
        }
    }
}
