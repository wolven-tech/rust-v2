//! Segmented bar explorer: counts or sums of records across one category axis, optionally
//! stacked by a second axis. The reader switches metric, axis and stack; a legend hides
//! segments; a linked table repeats every number.
//!
//! Design choice: a metric without `source_field` counts rows. Each row contributes its
//! `source_field` value otherwise; missing or non-finite values count as zero.
//! Design choice: the three pickers are buttons over a listbox. Up and Down move the
//! highlight, Enter picks, Escape closes and returns focus to the button.
//! Design choice: the bars are SVG groups with `role="button"`; focusing one shows its
//! breakdown and dims the others, as hovering does.
//! Design choice: the table lists only visible stack values, so its totals match the bars
//! for every combination of hidden segments.
//! Not implemented: the restack and grow tween when metric, axis or stack changes; the
//! bars re-render at their new heights.
//! Browser checks: picker keyboard flow, focus returning from Escape, the table scrolling
//! inside its box at phone width, bar dimming and table row highlight on hover.

use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    SeriesTone, compact_number, focus_element, group_digits, nice_axis, non_negative, tick_values,
};
use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/segmented-bar-explorer.css");

const WIDTH: f64 = 640.0;
const HEIGHT: f64 = 240.0;
const PLOT_LEFT: f64 = 52.0;
const PLOT_TOP: f64 = 12.0;
const PLOT_BOTTOM: f64 = 204.0;

/// One item counted: a value per axis key, and numeric fields for metrics.
#[derive(Clone, Debug, PartialEq)]
pub struct TallyRow {
    pub axis: BTreeMap<String, String>,
    pub metrics: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupingAxis {
    pub axis_key: String,
    pub axis_label: String,
    pub axis_values: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MetricDef {
    pub metric_key: String,
    pub metric_label: String,
    pub unit: String,
    pub source_field: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bar {
    pub axis_value: String,
    /// Visible stack values with their amounts, in stack-axis order.
    pub segments: Vec<(String, f64)>,
    pub total: f64,
}

/// What one row contributes under a metric.
pub fn row_value(row: &TallyRow, metric: &MetricDef) -> f64 {
    match &metric.source_field {
        None => 1.0,
        Some(field) => non_negative(row.metrics.get(field).copied().unwrap_or(0.0)),
    }
}

/// Bars for each axis value, with one segment per visible stack value. `stack_axis` of
/// `None` gives a single segment per bar.
pub fn bar_breakdown(
    rows: &[TallyRow],
    axis: &GroupingAxis,
    stack_axis: Option<&GroupingAxis>,
    metric: &MetricDef,
    hidden: &BTreeSet<String>,
) -> Vec<Bar> {
    axis.axis_values
        .iter()
        .map(|axis_value| {
            let segments: Vec<(String, f64)> = match stack_axis {
                None => {
                    let total: f64 = rows
                        .iter()
                        .filter(|r| r.axis.get(&axis.axis_key) == Some(axis_value))
                        .map(|r| row_value(r, metric))
                        .sum();
                    vec![(String::new(), total)]
                }
                Some(stack) => visible_stack_values(stack, hidden)
                    .into_iter()
                    .map(|stack_value| {
                        let total: f64 = rows
                            .iter()
                            .filter(|r| {
                                r.axis.get(&axis.axis_key) == Some(axis_value)
                                    && r.axis.get(&stack.axis_key) == Some(&stack_value)
                            })
                            .map(|r| row_value(r, metric))
                            .sum();
                        (stack_value, total)
                    })
                    .collect(),
            };
            let total = segments.iter().map(|(_, v)| v).sum();
            Bar {
                axis_value: axis_value.clone(),
                segments,
                total,
            }
        })
        .collect()
}

/// Stack values that are not hidden, in axis order.
pub fn visible_stack_values(stack: &GroupingAxis, hidden: &BTreeSet<String>) -> Vec<String> {
    stack
        .axis_values
        .iter()
        .filter(|v| !hidden.contains(*v))
        .cloned()
        .collect()
}

/// Tone of the stack value at `index`, cycling through the palette.
pub fn stack_tone(index: usize) -> SeriesTone {
    const PALETTE: [SeriesTone; 5] = [
        SeriesTone::Ink,
        SeriesTone::Ok,
        SeriesTone::Warn,
        SeriesTone::Fault,
        SeriesTone::Muted,
    ];
    PALETTE[index % PALETTE.len()]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Picker {
    Metric,
    Axis,
    Stack,
}

/// A card that shows how records split across one axis, optionally stacked by a second.
/// `start_stack` of `"none"` gives unstacked bars.
#[component]
pub fn SegmentedBarExplorer(
    tally_rows: Vec<TallyRow>,
    grouping_axes: Vec<GroupingAxis>,
    metric_defs: Vec<MetricDef>,
    #[props(into)] heading_text: String,
    #[props(into)] start_metric: String,
    #[props(into)] start_axis: String,
    #[props(default = "none".to_string(), into)] start_stack: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-seg-{}", next_id()));
    let mut metric_key = use_signal(|| start_metric.clone());
    let mut axis_key = use_signal(|| start_axis.clone());
    let mut stack_key = use_signal(|| start_stack.clone());
    let mut hidden = use_signal(BTreeSet::<String>::new);
    let mut open = use_signal(|| None::<Picker>);
    let mut highlight = use_signal(|| 0usize);
    let mut focused_bar = use_signal(|| None::<usize>);

    let metric = metric_defs
        .iter()
        .find(|m| m.metric_key == metric_key())
        .or_else(|| metric_defs.first())
        .cloned();
    let axis = grouping_axes
        .iter()
        .find(|a| a.axis_key == axis_key())
        .or_else(|| grouping_axes.first())
        .cloned();
    let stack = if stack_key() == "none" {
        None
    } else {
        grouping_axes
            .iter()
            .find(|a| a.axis_key == stack_key())
            .cloned()
    };
    let class = class_list("kit-seg", &root_class);

    let (Some(metric), Some(axis)) = (metric, axis) else {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            section { class: class, "aria-label": heading_text.clone(), p { class: "kit-seg__empty", "No axes to show" } }
        };
    };

    let hidden_set = hidden();
    let bars = bar_breakdown(&tally_rows, &axis, stack.as_ref(), &metric, &hidden_set);
    let total: f64 = bars.iter().map(|b| b.total).sum();
    let max_total = bars.iter().map(|b| b.total).fold(0.0_f64, f64::max);
    let (_, nhi, step) = nice_axis(0.0, max_total.max(1.0), 4);
    let ticks = tick_values(0.0, nhi, step);
    let y_of =
        |v: f64| PLOT_BOTTOM - (v / nhi.max(f64::EPSILON)).min(1.0) * (PLOT_BOTTOM - PLOT_TOP);
    let slot = (WIDTH - PLOT_LEFT) / bars.len().max(1) as f64;
    let bar_w = (slot * 0.56).min(64.0);
    let stack_values: Vec<String> = match &stack {
        Some(s) => visible_stack_values(s, &hidden_set),
        None => vec![String::new()],
    };
    let stack_all: Vec<String> = stack
        .as_ref()
        .map(|s| s.axis_values.clone())
        .unwrap_or_default();
    let focus = focused_bar().filter(|i| *i < bars.len());
    let unit = metric.unit.clone();
    let heading = heading_text.clone();
    let total_text = format!("{} {unit}", group_digits(total));
    let empty = tally_rows.is_empty();
    let key_base = base.clone();

    let metric_options: Vec<(String, String)> = metric_defs
        .iter()
        .map(|m| (m.metric_key.clone(), m.metric_label.clone()))
        .collect();
    let axis_options: Vec<(String, String)> = grouping_axes
        .iter()
        .map(|a| (a.axis_key.clone(), a.axis_label.clone()))
        .collect();
    let mut stack_options: Vec<(String, String)> = vec![("none".to_string(), "None".to_string())];
    stack_options.extend(
        grouping_axes
            .iter()
            .filter(|a| a.axis_key != axis.axis_key)
            .map(|a| (a.axis_key.clone(), a.axis_label.clone())),
    );
    let picker_labels = [
        ("Metric", metric.metric_label.clone(), Picker::Metric),
        ("Axis", axis.axis_label.clone(), Picker::Axis),
        (
            "Stack",
            stack
                .as_ref()
                .map(|s| s.axis_label.clone())
                .unwrap_or_else(|| "None".to_string()),
            Picker::Stack,
        ),
    ];
    let open_now = open();
    let open_options: Vec<(String, String)> = match open_now {
        Some(Picker::Metric) => metric_options.clone(),
        Some(Picker::Axis) => axis_options.clone(),
        Some(Picker::Stack) => stack_options.clone(),
        None => Vec::new(),
    };

    let tick_decimals = if step >= 1.0 {
        0
    } else {
        (-step.log10().floor()) as usize
    };
    let grid_lines: Vec<(f64, String)> = ticks
        .iter()
        .map(|t| {
            let text = if step >= 1.0 {
                compact_number(*t)
            } else {
                format!("{t:.tick_decimals$}")
            };
            (y_of(*t), text)
        })
        .collect();
    let shapes: Vec<(
        usize,
        f64,
        f64,
        Vec<(String, f64, f64, f64, &'static str)>,
        String,
        bool,
    )> = bars
        .iter()
        .enumerate()
        .map(|(i, bar)| {
            let x = PLOT_LEFT + (i as f64 + 0.5) * slot - bar_w / 2.0;
            let mut stacked = 0.0;
            let mut parts = Vec::new();
            for (k, (name, value)) in bar.segments.iter().enumerate() {
                let from = stacked;
                stacked += non_negative(*value);
                let index = stack_all.iter().position(|v| v == name).unwrap_or(k);
                parts.push((
                    name.clone(),
                    y_of(stacked),
                    y_of(from) - y_of(stacked),
                    bar_w,
                    stack_tone(index).as_str(),
                ));
            }
            let label = format!(
                "{}: total {} {unit}",
                bar.axis_value,
                group_digits(bar.total)
            );
            (i, x, bar.total, parts, label, focus.is_some_and(|f| f != i))
        })
        .collect();
    let axis_labels: Vec<(usize, f64, String)> = bars
        .iter()
        .enumerate()
        .map(|(i, bar)| {
            (
                i,
                PLOT_LEFT + (i as f64 + 0.5) * slot,
                bar.axis_value.clone(),
            )
        })
        .collect();
    let detail: Option<Vec<(String, String)>> = focus.map(|i| {
        bars[i]
            .segments
            .iter()
            .map(|(name, value)| {
                let label = if name.is_empty() {
                    bars[i].axis_value.clone()
                } else {
                    name.clone()
                };
                (label, format!("{} {unit}", group_digits(*value)))
            })
            .collect()
    });
    let legend: Vec<(String, String, bool, &'static str)> = stack_all
        .iter()
        .enumerate()
        .map(|(i, name)| {
            (
                name.clone(),
                name.clone(),
                !hidden_set.contains(name),
                stack_tone(i).as_str(),
            )
        })
        .collect();
    let stack_name = stack
        .as_ref()
        .map(|s| s.axis_label.clone())
        .unwrap_or_default();
    let table_rows: Vec<(String, Vec<String>, String, bool)> = bars
        .iter()
        .enumerate()
        .map(|(i, bar)| {
            let cells: Vec<String> = stack_values
                .iter()
                .map(|sv| {
                    let v = bar
                        .segments
                        .iter()
                        .find(|(n, _)| n == sv)
                        .map(|(_, v)| *v)
                        .unwrap_or(0.0);
                    group_digits(v)
                })
                .collect();
            (
                bar.axis_value.clone(),
                cells,
                group_digits(bar.total),
                focus == Some(i),
            )
        })
        .collect();

    let chart_label = format!("Bars by {}", axis.axis_label);

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": heading.clone(), "data-empty": empty.then_some("true"),
            header { class: "kit-seg__head",
                h3 { class: "kit-seg__title", "{heading}" }
                p { class: "kit-seg__total", "{total_text}" }
            }
            div { class: "kit-seg__pickers",
                for (label, value, which) in picker_labels.iter().cloned() {
                    div { key: "{label}", class: "kit-seg__picker",
                        span { class: "kit-seg__picker-label", "{label}" }
                        button {
                            id: "{key_base}-{label}",
                            r#type: "button",
                            class: "kit-seg__picker-button",
                            "aria-haspopup": "listbox",
                            "aria-expanded": (open_now == Some(which)).then_some("true"),
                            onclick: move |_| {
                                if open() == Some(which) {
                                    open.set(None);
                                } else {
                                    open.set(Some(which));
                                    highlight.set(0);
                                }
                            },
                            "{value}"
                        }
                        if open_now == Some(which) {
                            ul {
                                class: "kit-seg__listbox",
                                role: "listbox",
                                "aria-label": "{label}",
                                tabindex: "-1",
                                onkeydown: {
                                    let options = open_options.clone();
                                    let button_id = format!("{key_base}-{label}");
                                    move |event: KeyboardEvent| {
                                        let count = options.len();
                                        match event.key() {
                                            Key::ArrowDown => {
                                                event.prevent_default();
                                                highlight.set((highlight() + 1) % count.max(1));
                                            }
                                            Key::ArrowUp => {
                                                event.prevent_default();
                                                highlight.set((highlight() + count.max(1) - 1) % count.max(1));
                                            }
                                            Key::Enter => {
                                                event.prevent_default();
                                                if let Some((key, _)) = options.get(highlight()).cloned() {
                                                    match which {
                                                        Picker::Metric => metric_key.set(key),
                                                        Picker::Axis => axis_key.set(key),
                                                        Picker::Stack => stack_key.set(key),
                                                    }
                                                }
                                                open.set(None);
                                                focus_element(&button_id);
                                            }
                                            Key::Escape => {
                                                event.prevent_default();
                                                open.set(None);
                                                focus_element(&button_id);
                                            }
                                            _ => {}
                                        }
                                    }
                                },
                                for (idx, (key, text)) in open_options.clone().into_iter().enumerate() {
                                    li {
                                        key: "{key}",
                                        role: "option",
                                        "aria-selected": (idx == highlight()).then_some("true"),
                                        class: "kit-seg__option",
                                        "data-highlight": (idx == highlight()).then_some("true"),
                                        onclick: {
                                            let key = key.clone();
                                            move |_| {
                                                match which {
                                                    Picker::Metric => metric_key.set(key.clone()),
                                                    Picker::Axis => axis_key.set(key.clone()),
                                                    Picker::Stack => stack_key.set(key.clone()),
                                                }
                                                open.set(None);
                                            }
                                        },
                                        "{text}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if empty {
                p { class: "kit-seg__empty", role: "status", "No rows yet" }
            } else {
                div { class: "kit-seg__chart", role: "group", "aria-label": chart_label,
                    div { class: "kit-seg__bars",
                        svg { class: "kit-seg__svg", view_box: format!("0 0 {WIDTH} {HEIGHT}"), "aria-hidden": "true",
                            for (idx, (y, text)) in grid_lines.iter().cloned().enumerate() {
                                g { key: "{idx}",
                                    line { class: "kit-seg__grid", x1: PLOT_LEFT, x2: WIDTH, y1: y, y2: y }
                                    text { class: "kit-seg__tick", x: PLOT_LEFT - 6.0, y: y, "text-anchor": "end", "dominant-baseline": "middle", "{text}" }
                                }
                            }
                        }
                        svg { class: "kit-seg__svg kit-seg__svg--bars", view_box: format!("0 0 {WIDTH} {HEIGHT}"), "aria-hidden": "true",
                            for (i, x, _total, parts, label, dim) in shapes.clone() {
                                g {
                                    key: "{i}",
                                    class: "kit-seg__bar",
                                    role: "button",
                                    tabindex: "0",
                                    "aria-label": label,
                                    "data-dim": dim.then_some("true"),
                                    "data-active": (focus == Some(i)).then_some("true"),
                                    onmouseenter: move |_| focused_bar.set(Some(i)),
                                    onmouseleave: move |_| focused_bar.set(None),
                                    onfocus: move |_| focused_bar.set(Some(i)),
                                    onblur: move |_| focused_bar.set(None),
                                    for (k, (name, y, h, w, tone)) in parts.clone().into_iter().enumerate() {
                                        rect { key: "{k}", class: "kit-seg__segment", x: x, y: y, width: w, height: h.max(0.0), "data-tone": tone, "data-name": name }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-seg__axis",
                        for (i, _x, text) in axis_labels.clone() {
                            span { key: "{i}", class: "kit-seg__axis-label", "{text}" }
                        }
                    }
                    dl { class: "kit-seg__detail", role: "status", "aria-live": "polite",
                        if let Some(rows) = detail.clone() {
                            for (idx, (name, value)) in rows.into_iter().enumerate() {
                                div { key: "{idx}", class: "kit-seg__detail-row",
                                    dt { "{name}" }
                                    dd { "{value}" }
                                }
                            }
                        }
                    }
                }
                if stack.is_some() {
                    ul { class: "kit-seg__legend", "aria-label": "{stack_name}",
                        for (idx, (key, text, pressed, tone)) in legend.iter().cloned().enumerate() {
                            li { key: "{idx}",
                                button {
                                    r#type: "button",
                                    class: "kit-seg__toggle",
                                    "aria-pressed": if pressed { "true" } else { "false" },
                                    "data-tone": tone,
                                    onclick: {
                                        let key = key.clone();
                                        move |_| {
                                            let mut next = hidden();
                                            if next.contains(&key) {
                                                next.remove(&key);
                                            } else {
                                                next.insert(key.clone());
                                            }
                                            hidden.set(next);
                                        }
                                    },
                                    "{text}"
                                }
                            }
                        }
                    }
                    div { class: "kit-seg__table-box",
                        table { class: "kit-seg__table",
                            caption { class: "kit-seg__sr", "{stack_name} by {axis.axis_label}" }
                            thead {
                                tr {
                                    th { scope: "col", "{axis.axis_label}" }
                                    for sv in stack_values.clone() {
                                        th { key: "{sv}", scope: "col", "{sv}" }
                                    }
                                    th { scope: "col", "Total" }
                                }
                            }
                            tbody {
                                for (idx, (label, cells, sum, active)) in table_rows.clone().into_iter().enumerate() {
                                    tr {
                                        key: "{idx}",
                                        "data-active": active.then_some("true"),
                                        onmouseenter: move |_| focused_bar.set(Some(idx)),
                                        onmouseleave: move |_| focused_bar.set(None),
                                        th { scope: "row", "{label}" }
                                        for (c, cell) in cells.into_iter().enumerate() {
                                            td { key: "{c}", "{cell}" }
                                        }
                                        td { class: "kit-seg__sum", "{sum}" }
                                    }
                                }
                            }
                        }
                    }
                }
                if stack.is_none() {
                    div { class: "kit-seg__table-box",
                        table { class: "kit-seg__table",
                            caption { class: "kit-seg__sr", "{axis.axis_label}" }
                            thead { tr { th { scope: "col", "{axis.axis_label}" } th { scope: "col", "Total" } } }
                            tbody {
                                for (idx, (label, _cells, sum, active)) in table_rows.clone().into_iter().enumerate() {
                                    tr {
                                        key: "{idx}",
                                        "data-active": active.then_some("true"),
                                        onmouseenter: move |_| focused_bar.set(Some(idx)),
                                        onmouseleave: move |_| focused_bar.set(None),
                                        th { scope: "row", "{label}" }
                                        td { class: "kit-seg__sum", "{sum}" }
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
