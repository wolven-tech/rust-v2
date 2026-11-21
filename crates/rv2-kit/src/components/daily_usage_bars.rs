//! Daily usage bars: one bar per day for a metered quantity, with totals above. While
//! today is still filling, its bar takes the live tone with a sheen and the badge spins.
//!
//! Design choice: a day with a missing, non-finite or negative value is an empty slot
//! with zero height. A chart where every value is empty or zero shows `empty_message`
//! and no bars.
//! Design choice: `format_figure` applies to every total; there is no per-figure formatter.
//! Design choice: the bars rise in a staggered CSS sequence (`--kit-i`), not a JS wave.
//! Not implemented: the roll of totals between values. Totals re-render at the new value.
//! Browser checks: staggered rise, live sheen and spinning badge indicator, arrow
//! navigation with the tag read-out, the period text moving below the title when narrow.

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    IndexMove, SeriesTone, axis_indices, compact_number, finite_or_zero, group_digits, nice_axis,
    step_index, tick_values, value_domain,
};
use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/daily-usage-bars.css");

const WIDTH: f64 = 640.0;
const PLOT_LEFT: f64 = 48.0;
const PLOT_RIGHT: f64 = 632.0;
const PLOT_TOP: f64 = 10.0;

#[derive(Clone, Debug, PartialEq)]
pub struct DayValue {
    pub day_label: String,
    pub day_value: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Figure {
    pub figure_label: String,
    pub figure_value: f64,
}

/// A day's value as a usable number: missing, non-finite and negative values are `None`.
pub fn usable_value(value: Option<f64>) -> Option<f64> {
    value.filter(|v| v.is_finite() && *v >= 0.0)
}

/// Fraction of the ceiling a bar fills, from 0 to 1. Invalid values fill nothing.
pub fn bar_fraction(value: Option<f64>, ceiling: f64) -> f64 {
    match usable_value(value) {
        Some(v) if ceiling > 0.0 => (v / ceiling).min(1.0),
        _ => 0.0,
    }
}

/// Tone of bar `index` of `count`, moving through `palette` from left to right.
pub fn palette_tone(palette: &[SeriesTone], index: usize, count: usize) -> SeriesTone {
    if palette.is_empty() {
        return SeriesTone::Ink;
    }
    let slot = if count <= 1 {
        0
    } else {
        index * palette.len() / count
    };
    palette[slot.min(palette.len() - 1)]
}

/// True when the chart has at least one usable, non-zero value.
pub fn has_usage(days: &[DayValue]) -> bool {
    days.iter()
        .any(|d| usable_value(d.day_value).is_some_and(|v| v > 0.0))
}

/// Badge text for a live chart: the given text, or "Real-time" when none is given.
pub fn live_text(is_live: bool, given: &str) -> Option<String> {
    if !is_live {
        None
    } else if given.is_empty() {
        Some("Real-time".to_string())
    } else {
        Some(given.to_string())
    }
}

/// Bars for `days` with a value axis from zero to a round ceiling. Returns
/// `(ceiling, tick values)`.
pub fn axis_for(days: &[DayValue]) -> (f64, Vec<f64>) {
    let values = days.iter().filter_map(|d| usable_value(d.day_value));
    let (lo, hi) = value_domain(values, true);
    let (nlo, nhi, step) = nice_axis(lo, hi, 4);
    (nhi, tick_values(nlo, nhi, step))
}

/// A usage card: totals above, one bar per day, and a tag for the active day.
/// Left and Right step through the days, Home goes to the first, End to the last, and
/// Escape closes the tag.
#[component]
pub fn DailyUsageBars(
    #[props(into)] chart_heading: String,
    day_values: Vec<DayValue>,
    total_figures: Vec<Figure>,
    #[props(default, into)] period_text: String,
    #[props(default)] is_today_live: bool,
    #[props(default, into)] live_badge_text: String,
    #[props(default = group_digits as fn(f64) -> String)] format_figure: fn(f64) -> String,
    #[props(default = compact_number as fn(f64) -> String)] format_axis: fn(f64) -> String,
    #[props(default = vec![SeriesTone::Muted])] bar_palette: Vec<SeriesTone>,
    #[props(default)] live_bar_tone: SeriesTone,
    #[props(default = 180.0)] plot_height: f64,
    #[props(default = "No usage yet".to_string(), into)] empty_message: String,
    #[props(default, into)] extra_class: String,
) -> Element {
    let mut active = use_signal(|| None::<usize>);

    let n = day_values.len();
    let last = n.saturating_sub(1);
    let visible = has_usage(&day_values);
    let badge = live_text(is_today_live, &live_badge_text);
    let live_index = if is_today_live && n > 0 {
        Some(last)
    } else {
        None
    };
    let (ceiling, ticks) = axis_for(&day_values);
    let plot_bottom = plot_height;
    let view_height = plot_height + 24.0;
    let y_of =
        |v: f64| plot_bottom - (v / ceiling.max(f64::EPSILON)).min(1.0) * (plot_bottom - PLOT_TOP);
    let slot = (PLOT_RIGHT - PLOT_LEFT) / n.max(1) as f64;
    let bar_width = (slot * 0.6).min(24.0);

    let current = active().map(|i| i.min(last));
    let mut bars: Vec<(usize, f64, f64, f64, String, &'static str, bool)> = Vec::new();
    let mut hits: Vec<(usize, f64)> = Vec::new();
    if visible {
        for (i, day) in day_values.iter().enumerate() {
            let fraction = bar_fraction(day.day_value, ceiling);
            let height = fraction * (plot_bottom - PLOT_TOP);
            let x = PLOT_LEFT + i as f64 * slot + (slot - bar_width) / 2.0;
            let is_live = live_index == Some(i);
            let tone = if is_live {
                live_bar_tone.as_str()
            } else {
                palette_tone(&bar_palette, i, n).as_str()
            };
            let delay = format!("--kit-i: {i}");
            bars.push((i, x, plot_bottom - height, height, delay, tone, is_live));
            hits.push((i, PLOT_LEFT + i as f64 * slot));
        }
    }
    let axis_labels: Vec<(usize, f64, String, &'static str)> = if visible {
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
                (
                    i,
                    PLOT_LEFT + (i as f64 + 0.5) * slot,
                    day_values[i].day_label.clone(),
                    anchor,
                )
            })
            .collect()
    } else {
        Vec::new()
    };
    let tag = current.and_then(|i| {
        day_values.get(i).map(|day| {
            let value = match usable_value(day.day_value) {
                Some(v) => format_figure(v),
                None => "–".to_string(),
            };
            format!("{}: {value}", day.day_label)
        })
    });
    let class = class_list("kit-usage", &extra_class);
    let view_box = format!("0 0 {WIDTH} {view_height}");
    let tick_labels: Vec<(f64, f64, String)> = ticks
        .iter()
        .map(|t| (*t, y_of(*t), format_axis(*t)))
        .collect();
    let figures: Vec<(usize, String, String)> = total_figures
        .iter()
        .enumerate()
        .map(|(i, f)| {
            (
                i,
                f.figure_label.clone(),
                format_figure(finite_or_zero(f.figure_value)),
            )
        })
        .collect();
    let period = period_text.clone();
    let heading = chart_heading.clone();
    let empty_text = empty_message.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": heading.clone(), "data-live": is_today_live.then_some("true"),
            header { class: "kit-usage__title",
                h3 { class: "kit-usage__heading", "{heading}" }
                if let Some(text) = badge.clone() {
                    span { class: "kit-usage__badge",
                        span { class: "kit-usage__indicator", "aria-hidden": "true" }
                        "{text}"
                    }
                }
                if !period.is_empty() {
                    span { class: "kit-usage__period", "{period}" }
                }
            }
            dl { class: "kit-usage__totals",
                for (i, label, value) in figures.clone() {
                    div { key: "{i}", class: "kit-usage__total",
                        dt { class: "kit-usage__total-label", "{label}" }
                        dd { class: "kit-usage__total-value", "{value}" }
                    }
                }
            }
            div { class: "kit-sr", role: "status", if !visible { "{empty_text}" } }
            if !visible {
                div { class: "kit-usage__empty", "{empty_text}" }
            } else {
                div {
                    class: "kit-usage__plot",
                    tabindex: "0",
                    role: "group",
                    "aria-label": heading.clone(),
                    "aria-roledescription": "chart",
                    onkeydown: move |event: KeyboardEvent| {
                        let movement = match event.key() {
                            Key::ArrowLeft => Some(IndexMove::Prev),
                            Key::ArrowRight => Some(IndexMove::Next),
                            Key::Home => Some(IndexMove::First),
                            Key::End => Some(IndexMove::Last),
                            Key::Escape => {
                                active.set(None);
                                None
                            }
                            _ => None,
                        };
                        if let Some(movement) = movement {
                            event.prevent_default();
                            active.set(step_index(current, n, movement));
                        }
                    },
                    onmouseleave: move |_| active.set(None),
                    svg { class: "kit-usage__svg", view_box: view_box, "aria-hidden": "true",
                        for (idx, (_, y, text)) in tick_labels.clone().into_iter().enumerate() {
                            g { key: "{idx}",
                                line { class: "kit-usage__grid", x1: PLOT_LEFT, x2: PLOT_RIGHT, y1: y, y2: y }
                                text { class: "kit-usage__tick", x: PLOT_LEFT - 6.0, y: y, "text-anchor": "end", "dominant-baseline": "middle", "{text}" }
                            }
                        }
                        for (i, x, y, height, delay, tone, is_live) in bars.clone() {
                            rect {
                                key: "b{i}",
                                class: if is_live { "kit-usage__bar kit-usage__bar--live" } else { "kit-usage__bar" },
                                x: x,
                                y: y,
                                width: bar_width,
                                height: height.max(0.0),
                                "data-tone": tone,
                                "data-active": (current == Some(i)).then_some("true"),
                                style: delay,
                            }
                        }
                        for (i, x, text, anchor) in axis_labels.clone() {
                            text { key: "d{i}", class: "kit-usage__date", x: x, y: view_height - 6.0, "text-anchor": anchor, "{text}" }
                        }
                        for (i, x) in hits.clone() {
                            rect {
                                key: "h{i}",
                                class: "kit-usage__hit",
                                x: x,
                                y: PLOT_TOP,
                                width: slot,
                                height: plot_bottom - PLOT_TOP,
                                onmouseenter: move |_| active.set(Some(i)),
                            }
                        }
                    }
                    div { role: "status", "aria-live": "polite", class: "kit-usage__live",
                        if let Some(text) = tag.clone() {
                            span { class: "kit-usage__tag", "{text}" }
                        }
                    }
                }
            }
        }
    }
}
