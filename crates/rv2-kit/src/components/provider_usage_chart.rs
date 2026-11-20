//! Provider usage chart: a stacked column per AI provider, one segment per model, with a
//! switch between tokens, cost and requests. Aiming at a column opens its model card and
//! narrows the task list below to that provider.
//!
//! Design choice: within a column the heaviest model sits at the base with the most
//! emphasis; later models take lighter shades of the group tone (CSS `color-mix`).
//! Design choice: non-numeric and negative usage counts as zero. A provider with no
//! models stands empty.
//! Design choice: the card sits in flow under the chart; on wide containers CSS places
//! it beside the chart.
//! Design choice: a task kind not in `task_kinds` takes the built-in label and tone, or a
//! neutral tone with its raw name.
//! Not implemented: the column rise sequence and resize tweening; values re-render.
//! Browser checks: column rise on first view, measure resize, card placement on narrow
//! widths, the task list collapsing to one column.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::components::series_trend_chart::non_negative;
use crate::components::series_trend_chart::{
    IndexMove, SeriesTone, compact_number, nice_axis, share, share_label, signed_percent,
    stack_bounds, step_index, tick_values, value_domain,
};
use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/provider-usage-chart.css");

const WIDTH: f64 = 640.0;
const HEIGHT: f64 = 220.0;
const PLOT_LEFT: f64 = 48.0;
const PLOT_TOP: f64 = 10.0;
const PLOT_BOTTOM: f64 = 200.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Measure {
    #[default]
    Tokens,
    Cost,
    Requests,
}

impl Measure {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tokens => "Tokens",
            Self::Cost => "Cost",
            Self::Requests => "Requests",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Usage {
    pub tokens: Option<f64>,
    pub cost_usd: Option<f64>,
    pub request_count: Option<f64>,
}

impl Usage {
    /// The usage under `measure`; missing, non-finite and negative values count as zero.
    pub fn value(&self, measure: Measure) -> f64 {
        let raw = match measure {
            Measure::Tokens => self.tokens,
            Measure::Cost => self.cost_usd,
            Measure::Requests => self.request_count,
        };
        non_negative(raw.unwrap_or(0.0))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelUsage {
    pub model_name: String,
    pub usage: Usage,
    pub task_usage: BTreeMap<String, Usage>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderGroup {
    pub group_name: String,
    pub group_tone: Option<SeriesTone>,
    pub group_logo: Option<Element>,
    pub models: Vec<ModelUsage>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TaskKind {
    pub kind_label: String,
    pub kind_tone: SeriesTone,
    pub kind_icon: Option<Element>,
}

/// Label and tone of a task kind. Kinds not in `kinds` fall back to the built-in set,
/// then to a neutral tone with the raw kind name.
pub fn task_style(kinds: &BTreeMap<String, TaskKind>, kind: &str) -> (String, SeriesTone) {
    if let Some(found) = kinds.get(kind) {
        return (found.kind_label.clone(), found.kind_tone);
    }
    match kind {
        "search" => ("Search".to_string(), SeriesTone::Ink),
        "analysis" => ("Analysis".to_string(), SeriesTone::Ok),
        "code" => ("Code".to_string(), SeriesTone::Warn),
        "tests" => ("Tests".to_string(), SeriesTone::Fault),
        other => (other.to_string(), SeriesTone::Muted),
    }
}

/// Figures in the measure's own unit: `3.1B tokens`, `$8.6M`, `640k requests`.
pub fn format_measure(measure: Measure, value: f64) -> String {
    let value = non_negative(value);
    match measure {
        Measure::Tokens => format!("{} tokens", compact_number(value)),
        Measure::Requests => format!("{} requests", compact_number(value)),
        Measure::Cost => {
            if value < 1000.0 {
                format!("${value:.2}")
            } else {
                format!("${}", compact_number(value))
            }
        }
    }
}

/// Model segments in column order: heaviest first, so it sits at the base.
pub fn column_segments(group: &ProviderGroup, measure: Measure) -> Vec<(String, f64)> {
    let mut segments: Vec<(String, f64)> = group
        .models
        .iter()
        .map(|m| (m.model_name.clone(), m.usage.value(measure)))
        .collect();
    segments.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    segments
}

/// Total of a provider's models under `measure`.
pub fn group_total(group: &ProviderGroup, measure: Measure) -> f64 {
    group.models.iter().map(|m| m.usage.value(measure)).sum()
}

/// Total across every provider under `measure`.
pub fn grand_total(groups: &[ProviderGroup], measure: Measure) -> f64 {
    groups.iter().map(|g| group_total(g, measure)).sum()
}

/// Task totals across every provider, or only `only`, highest first.
pub fn task_totals(
    groups: &[ProviderGroup],
    measure: Measure,
    only: Option<usize>,
) -> Vec<(String, f64)> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for (index, group) in groups.iter().enumerate() {
        if only.is_some_and(|wanted| wanted != index) {
            continue;
        }
        for model in &group.models {
            for (kind, usage) in &model.task_usage {
                *totals.entry(kind.clone()).or_insert(0.0) += usage.value(measure);
            }
        }
    }
    let mut ordered: Vec<(String, f64)> = totals.into_iter().collect();
    ordered.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ordered
}

/// Percent change of `total` against `prior`, and whether the total rose. `None`
/// without a prior.
pub fn change_against(total: f64, prior: Option<f64>) -> Option<(f64, bool)> {
    let prior = prior.map(non_negative)?;
    let pct = if prior > 0.0 {
        (total - prior) / prior * 100.0
    } else {
        0.0
    };
    Some((pct, total > prior))
}

/// Shade of segment `index`: the base is full strength; each lighter model moves toward
/// the surface, to at least 35%.
pub fn segment_mix(index: usize) -> u32 {
    100usize.saturating_sub(index * 18).max(35) as u32
}

/// A stacked-column chart of provider usage with a model card and a task breakdown.
/// Left and Right step across providers, Home and End jump, Escape closes the card.
#[component]
pub fn ProviderUsageChart(
    provider_groups: Vec<ProviderGroup>,
    #[props(default = vec![Measure::Tokens, Measure::Cost, Measure::Requests])]
    measure_options: Vec<Measure>,
    #[props(default)] starting_measure: Measure,
    #[props(default = "This week".to_string(), into)] period_label: String,
    #[props(default)] prior_totals: Option<Usage>,
    #[props(default)] prior_label: Option<String>,
    #[props(default)] task_kinds: BTreeMap<String, TaskKind>,
    #[props(default, into)] extra_class: String,
) -> Element {
    let mut measure = use_signal(|| starting_measure);
    let mut engaged = use_signal(|| None::<usize>);

    let current = measure();
    let groups = &provider_groups;
    let total = grand_total(groups, current);
    let usable = total > 0.0;
    let prior_value = prior_totals.as_ref().map(|p| p.value(current));
    let change = change_against(total, prior_value);
    let (domain_lo, domain_hi) = value_domain(groups.iter().map(|g| group_total(g, current)), true);
    let (nlo, nhi, step) = nice_axis(domain_lo, domain_hi, 4);
    let y_of =
        |v: f64| PLOT_BOTTOM - (v - nlo) / (nhi - nlo).max(f64::EPSILON) * (PLOT_BOTTOM - PLOT_TOP);
    let n = groups.len();
    let last = n.saturating_sub(1);
    let slot = (WIDTH - PLOT_LEFT) / n.max(1) as f64;
    let column_w = (slot * 0.55).min(72.0);
    let active = engaged().map(|i| i.min(last));

    let mut columns: Vec<(String, f64, f64, f64, f64, &'static str, u32, String, bool)> =
        Vec::new();
    let mut hits: Vec<(usize, f64, f64)> = Vec::new();
    let mut labels: Vec<(usize, String, Option<Element>, String)> = Vec::new();
    if usable {
        for (g, group) in groups.iter().enumerate() {
            let center = PLOT_LEFT + (g as f64 + 0.5) * slot;
            let tone = group.group_tone.unwrap_or_default().as_str();
            let segments = column_segments(group, current);
            let values: Vec<f64> = segments.iter().map(|(_, v)| *v).collect();
            for (k, ((model, value), (from, to))) in
                segments.iter().zip(stack_bounds(&values)).enumerate()
            {
                let top = y_of(to);
                let bottom = y_of(from);
                columns.push((
                    format!("{g}-{k}"),
                    center - column_w / 2.0,
                    top,
                    column_w,
                    (bottom - top).max(0.0),
                    tone,
                    segment_mix(k),
                    format!("{model}: {}", format_measure(current, *value)),
                    active.is_some_and(|a| a != g),
                ));
            }
            hits.push((g, PLOT_LEFT + g as f64 * slot, slot));
            labels.push((
                g,
                group.group_name.clone(),
                group.group_logo.clone(),
                format_measure(current, group_total(group, current)),
            ));
        }
    }
    let ticks: Vec<(usize, f64, String)> = if usable {
        tick_values(nlo, nhi, step)
            .into_iter()
            .enumerate()
            .map(|(i, t)| (i, y_of(t), compact_number(t)))
            .collect()
    } else {
        Vec::new()
    };

    let card = active.and_then(|g| groups.get(g)).cloned();
    let card_models: Vec<(String, String, String)> = card
        .as_ref()
        .map(|group| {
            let group_sum = group_total(group, current);
            column_segments(group, current)
                .into_iter()
                .map(|(name, value)| {
                    (
                        name,
                        format_measure(current, value),
                        share_label(share(value, group_sum)),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let card_title = card
        .as_ref()
        .map(|g| g.group_name.clone())
        .unwrap_or_default();
    let card_total = card
        .as_ref()
        .map(|g| format_measure(current, group_total(g, current)))
        .unwrap_or_default();

    let task_rows = task_totals(groups, current, active);
    let task_max = task_rows.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max);
    let task_items: Vec<(String, String, SeriesTone, String, String)> = task_rows
        .iter()
        .map(|(kind, value)| {
            let (label, tone) = task_style(&task_kinds, kind);
            let width = if task_max > 0.0 {
                value / task_max * 100.0
            } else {
                0.0
            };
            (
                kind.clone(),
                label,
                tone,
                format!("width: {width:.1}%"),
                format_measure(current, *value),
            )
        })
        .collect();
    let tasks_heading = match active.and_then(|g| groups.get(g)) {
        Some(group) => format!("Tasks for {}", group.group_name),
        None => "Tasks across all providers".to_string(),
    };
    let change_text = change.map(|(pct, _)| signed_percent(pct));
    let change_up = change.map(|(_, up)| up).unwrap_or(false);
    let change_note = prior_label.clone().unwrap_or_default();
    let class = class_list("kit-pu", &extra_class);
    let view_box = format!("0 0 {WIDTH} {HEIGHT}");
    let total_text = format_measure(current, total);
    let empty_text = format!("No usage {}", period_label.to_lowercase());
    let period = period_label.clone();
    let measure_list = measure_options.clone();
    let single_measure = measure_list.len() <= 1;
    let has_card = card.is_some();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": "Provider usage",
            header { class: "kit-pu__summary",
                div { class: "kit-pu__period", "{period}" }
                div { class: "kit-pu__total",
                    span { class: "kit-pu__figure", "{total_text}" }
                    if let Some(text) = change_text.clone() {
                        span { class: "kit-pu__change", "data-up": change_up.then_some("true"),
                            span { "aria-hidden": "true", if change_up { "↑" } else { "↓" } }
                            " {text}"
                            if !change_note.is_empty() {
                                span { class: "kit-pu__change-note", " vs {change_note}" }
                            }
                        }
                    }
                }
                if !single_measure {
                    div { class: "kit-pu__measures", role: "group", "aria-label": "Measure",
                        for option in measure_list.clone() {
                            button {
                                key: "{option.label()}",
                                r#type: "button",
                                class: "kit-pu__measure",
                                "aria-pressed": if option == current { "true" } else { "false" },
                                onclick: move |_| measure.set(option),
                                "{option.label()}"
                            }
                        }
                    }
                }
            }
            div { class: "kit-sr", role: "status", if !usable { "{empty_text}" } }
            if !usable {
                p { class: "kit-pu__empty", "{empty_text}" }
            } else {
                div { class: "kit-pu__chart", "data-card": has_card.then_some("true"),
                    div {
                        class: "kit-pu__plot",
                        tabindex: "0",
                        role: "group",
                        "aria-label": "Usage by provider",
                        "aria-roledescription": "chart",
                        onkeydown: move |event: KeyboardEvent| {
                            let movement = match event.key() {
                                Key::ArrowLeft => Some(IndexMove::Prev),
                                Key::ArrowRight => Some(IndexMove::Next),
                                Key::Home => Some(IndexMove::First),
                                Key::End => Some(IndexMove::Last),
                                Key::Escape => {
                                    engaged.set(None);
                                    None
                                }
                                _ => None,
                            };
                            if let Some(movement) = movement {
                                event.prevent_default();
                                engaged.set(step_index(active, n, movement));
                            }
                        },
                        onmouseleave: move |_| engaged.set(None),
                        svg { class: "kit-pu__svg", view_box: view_box, "aria-hidden": "true",
                            for (i, y, text) in ticks.clone() {
                                g { key: "{i}",
                                    line { class: "kit-pu__grid", x1: PLOT_LEFT, x2: WIDTH, y1: y, y2: y }
                                    text { class: "kit-pu__tick", x: PLOT_LEFT - 6.0, y: y, "text-anchor": "end", "dominant-baseline": "middle", "{text}" }
                                }
                            }
                            for (key, x, y, w, h, tone, mix, title, dim) in columns.clone() {
                                rect {
                                    key: "{key}",
                                    class: "kit-pu__segment",
                                    x: x,
                                    y: y,
                                    width: w,
                                    height: h,
                                    "data-tone": tone,
                                    "data-dim": dim.then_some("true"),
                                    style: "--kit-seg-mix: {mix}%",
                                    title { "{title}" }
                                }
                            }
                            for (g, x, w) in hits.clone() {
                                rect {
                                    key: "h{g}",
                                    class: "kit-pu__hit",
                                    x: x,
                                    y: PLOT_TOP,
                                    width: w,
                                    height: PLOT_BOTTOM - PLOT_TOP,
                                    onmouseenter: move |_| engaged.set(Some(g)),
                                }
                            }
                        }
                        div { class: "kit-pu__labels", onmouseleave: move |_| engaged.set(None),
                            for (g, name, logo, sum) in labels.clone() {
                                button {
                                    key: "{g}",
                                    r#type: "button",
                                    class: "kit-pu__label",
                                    "data-active": (active == Some(g)).then_some("true"),
                                    "aria-label": "{name}, {sum}",
                                    onmouseenter: move |_| engaged.set(Some(g)),
                                    onfocus: move |_| engaged.set(Some(g)),
                                    onblur: move |_| engaged.set(None),
                                    if let Some(logo) = logo {
                                        span { class: "kit-pu__logo", aria_hidden: "true", {logo} }
                                    }
                                    span { class: "kit-pu__name", "{name}" }
                                    span { class: "kit-pu__sum", "{sum}" }
                                }
                            }
                        }
                    }
                    if has_card {
                        div { class: "kit-pu__card", role: "region", "aria-label": "{card_title}",
                            div { class: "kit-pu__card-head",
                                span { class: "kit-pu__card-title", "{card_title}" }
                                span { class: "kit-pu__card-total", "{card_total}" }
                            }
                            ul { class: "kit-pu__models",
                                for (idx, (name, value, pct)) in card_models.clone().into_iter().enumerate() {
                                    li { key: "{idx}", class: "kit-pu__model",
                                        span { class: "kit-pu__model-name", "{name}" }
                                        span { class: "kit-pu__model-value", "{value}" }
                                        span { class: "kit-pu__model-share", "{pct}" }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "kit-pu__card kit-pu__card--idle", "aria-hidden": "true" }
                    }
                }
                section { class: "kit-pu__tasks", "aria-label": "{tasks_heading}",
                    h4 { class: "kit-pu__tasks-title", "{tasks_heading}" }
                    ul { class: "kit-pu__task-list",
                        for (kind, label, tone, width, value) in task_items.clone() {
                            li { key: "{kind}", class: "kit-pu__task", "data-tone": tone.as_str(),
                                span { class: "kit-pu__task-name", "{label}" }
                                span { class: "kit-pu__task-track",
                                    span { class: "kit-pu__task-bar", style: width }
                                }
                                span { class: "kit-pu__task-value", "{value}" }
                            }
                        }
                    }
                    if task_items.is_empty() {
                        p { class: "kit-pu__task-empty", "{empty_text}" }
                    }
                }
            }
        }
    }
}
