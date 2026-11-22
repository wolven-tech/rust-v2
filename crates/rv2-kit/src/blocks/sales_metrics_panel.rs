//! Sales metrics panel: a sales overview for one period. Headline metric cards, one trend chart
//! whose metric and style the person can change, and ranked tables of top products and channels.
//!
//! Design choice: metric cards are `MetricTile`s with a sparkline; the chosen card is marked and
//! the trend redraws for its series.
//! Design choice: the trend is a `SeriesTrendChart`, an area or bars; the compare switch adds the
//! previous period as a second series with its tooltip value.
//! Design choice: breakdowns are tables with a name and a value column; each row carries a bar
//! scaled to the largest value, which grows from nothing when the row first appears.
//! Design choice: values arrive as formatted text, so a change counts the leading number up or
//! down and keeps the host's prefix, decimals and suffix; text without a number changes at once.
//! Design choice: the period picker is a `ListPicker` and export is a menu of three formats.
//! Browser checks: metric selection marks and redraws, bars and area share tooltip values,
//! compare adds a series, hover marker and value, breakdown order and bar widths, export menu
//! fires the format, phone two-column metric grid.

use dioxus::prelude::*;
use rv2_affect::segmented_control::{SegmentOption, SegmentedControl};
use rv2_affect::switch::Switch;

use crate::blocks::metric_overview_board::{
    ListPicker, PickOption, ROLL_STEPS, fade_swap, motion_allowed, roll_at,
};

use crate::components::metric_tile::{MetricTile, MetricValue};
use crate::components::series_trend_chart::{
    AxisMode, ChartVariant, SeriesTone, SeriesTrendChart, TrendSeries, compact_number,
};
use crate::internal::{KIT_CSS, class_list, sleep};

const CSS: Asset = asset!("/assets/sales-metrics-panel.css");
const ROLL_FRAME_MS: u32 = 30;

#[derive(Clone, Debug, PartialEq)]
pub struct SalesMetric {
    pub id: String,
    pub label: String,
    pub value: String,
    pub delta_percent: f64,
    pub series: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PeriodOption {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrendPoint {
    pub date: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ranked {
    pub id: String,
    pub name: String,
    pub value: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartStyle {
    #[default]
    Area,
    Bars,
}

impl ChartStyle {
    fn as_str(self) -> &'static str {
        match self {
            Self::Area => "area",
            Self::Bars => "bars",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Xlsx,
    Pdf,
}

impl ExportFormat {
    pub const ALL: [ExportFormat; 3] = [Self::Csv, Self::Xlsx, Self::Pdf];

    pub fn label(self) -> &'static str {
        match self {
            Self::Csv => "CSV",
            Self::Xlsx => "Excel",
            Self::Pdf => "PDF",
        }
    }
}

/// Ranked rows, largest value first; ties keep their input order.
pub fn sort_ranked(list: &[Ranked]) -> Vec<Ranked> {
    let mut out = list.to_vec();
    out.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// Width of each row as a percent of the largest value.
pub fn scaled_widths(rows: &[Ranked]) -> Vec<f64> {
    let max = rows.iter().map(|r| r.value).fold(0.0_f64, f64::max);
    rows.iter()
        .map(|r| {
            if max > 0.0 {
                (r.value / max * 100.0).clamp(0.0, 100.0)
            } else {
                0.0
            }
        })
        .collect()
}

/// A formatted amount split around its number: `$48.2k` is prefix `$`, value 48.2, one decimal
/// and suffix `k`.
#[derive(Clone, Debug, PartialEq)]
pub struct Amount {
    pub prefix: String,
    pub value: f64,
    pub decimals: usize,
    pub suffix: String,
}

/// The first number in `text`, with the text around it; `None` when there is no number.
pub fn parse_amount(text: &str) -> Option<Amount> {
    let start = text.find(|c: char| c.is_ascii_digit() || c == '-')?;
    let rest = &text[start..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',' || c == '-'))
        .unwrap_or(rest.len());
    let number: String = rest[..end].chars().filter(|c| *c != ',').collect();
    let value: f64 = number.parse().ok()?;
    let decimals = number.split_once('.').map_or(0, |(_, d)| d.len());
    Some(Amount {
        prefix: text[..start].to_string(),
        value,
        decimals,
        suffix: rest[end..].to_string(),
    })
}

/// The text `step` of `steps` frames into a count from `from` to `to`. Both must parse; the
/// final frame is `to` itself, so the host's own formatting is what finally shows.
pub fn roll_text(from: &str, to: &str, step: usize, steps: usize) -> String {
    match (parse_amount(from), parse_amount(to)) {
        (Some(a), Some(b)) if step < steps => format!(
            "{}{:.*}{}",
            b.prefix,
            b.decimals,
            roll_at(a.value, b.value, step, steps),
            b.suffix
        ),
        _ => to.to_string(),
    }
}

fn breakdown_rows(title: String, rows: Vec<Ranked>, format_amount: fn(f64) -> String) -> Element {
    let widths = scaled_widths(&rows);
    rsx! {
        section { class: "kit-sales__list",
            h3 { class: "kit-sales__list-title", "{title}" }
            if rows.is_empty() {
                p { class: "kit-sales__empty", "No sales in this period." }
            } else {
                table { class: "kit-sales__table", "aria-label": "{title}",
                    thead {
                        tr {
                            th { "scope": "col", "Name" }
                            th { "scope": "col", class: "kit-sales__num", "Value" }
                        }
                    }
                    tbody {
                        for (row, width) in rows.iter().zip(widths.iter()) {
                            tr { key: "{row.id}-{row.value}",
                                th { "scope": "row", class: "kit-sales__name", "{row.name}" }
                                td { class: "kit-sales__num",
                                    span { class: "kit-sales__amount", "{format_amount(row.value)}" }
                                    span { class: "kit-sales__bar", "aria-hidden": "true",
                                        span { class: "kit-sales__bar-fill", style: "width: {width:.2}%" }
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

#[component]
pub fn SalesMetricsPanel(
    #[props(default = "Sales".to_string(), into)] title: String,
    #[props(default, into)] subtitle: String,
    metric_cards: Vec<SalesMetric>,
    #[props(default)] selected_metric: Option<String>,
    #[props(default)] on_metric_change: Option<EventHandler<String>>,
    periods: Vec<PeriodOption>,
    #[props(default)] selected_period: Option<String>,
    #[props(default)] on_period_change: Option<EventHandler<String>>,
    trend_series: Vec<TrendPoint>,
    #[props(default)] previous_series: Vec<f64>,
    #[props(default)] chart_style: ChartStyle,
    #[props(default)] on_chart_style_change: Option<EventHandler<ChartStyle>>,
    #[props(default)] compare_enabled: bool,
    #[props(default)] on_compare_change: Option<EventHandler<bool>>,
    #[props(default)] top_products: Vec<Ranked>,
    #[props(default)] channels: Vec<Ranked>,
    #[props(default)] on_export: Option<EventHandler<ExportFormat>>,
    #[props(default = compact_number as fn(f64) -> String)] format_amount: fn(f64) -> String,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let mut local_metric = use_signal(|| None::<String>);
    let mut export_open = use_signal(|| false);
    let active_metric = selected_metric
        .clone()
        .or_else(|| local_metric())
        .or_else(|| metric_cards.first().map(|m| m.id.clone()))
        .unwrap_or_default();
    let metric = metric_cards.iter().find(|m| m.id == active_metric).cloned();
    let x_labels: Vec<String> = trend_series.iter().map(|p| p.date.clone()).collect();
    let values: Vec<Option<f64>> = trend_series.iter().map(|p| Some(p.value)).collect();
    let mut series = vec![TrendSeries {
        name: metric.as_ref().map(|m| m.label.clone()).unwrap_or_default(),
        values,
        tone: SeriesTone::Ink,
        emphasis: true,
        is_earlier: false,
    }];
    if compare_enabled && !previous_series.is_empty() {
        series.push(TrendSeries {
            name: "Previous period".to_string(),
            values: previous_series.iter().map(|v| Some(*v)).collect(),
            tone: SeriesTone::Muted,
            emphasis: false,
            is_earlier: true,
        });
    }
    let mut chosen_style = use_signal(|| chart_style);
    let shown_style = use_signal(|| chart_style);
    let style_fading = use_signal(|| false);
    let style_run = use_signal(|| 0_u32);
    let variant = match shown_style() {
        ChartStyle::Area => ChartVariant::Area,
        ChartStyle::Bars => ChartVariant::Bar,
    };
    let root_class = class_list("kit-sales", &extra_classes);
    let products = sort_ranked(&top_products);
    let channel_rows = sort_ranked(&channels);
    let trend_title = metric.as_ref().map(|m| m.label.clone()).unwrap_or_default();
    let style_value = chosen_style().as_str().to_string();

    let values_target: Vec<String> = metric_cards.iter().map(|c| c.value.clone()).collect();
    let mut shown_values = use_signal(|| values_target.clone());
    let mut roll_run = use_signal(|| 0_u32);
    use_effect(use_reactive(
        (&values_target,),
        move |(to,): (Vec<String>,)| {
            let from = shown_values.peek().clone();
            if from == to || from.len() != to.len() {
                shown_values.set(to);
                return;
            }
            let run = *roll_run.peek() + 1;
            roll_run.set(run);
            spawn(async move {
                let mut shown_values = shown_values;
                if !motion_allowed().await {
                    shown_values.set(to);
                    return;
                }
                for step in 1..=ROLL_STEPS {
                    if *roll_run.peek() != run {
                        return;
                    }
                    shown_values.set(
                        from.iter()
                            .zip(&to)
                            .map(|(a, b)| roll_text(a, b, step, ROLL_STEPS))
                            .collect(),
                    );
                    sleep(ROLL_FRAME_MS).await;
                }
                shown_values.set(to);
            });
        },
    ));
    let shown_now = shown_values();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-labelledby": "kit-sales-title",
            header { class: "kit-sales__head",
                div {
                    h2 { id: "kit-sales-title", class: "kit-sales__title", "{title}" }
                    if !subtitle.is_empty() {
                        p { class: "kit-sales__subtitle", "{subtitle}" }
                    }
                }
                div { class: "kit-sales__tools",
                    if !periods.is_empty() {
                        ListPicker {
                            options: periods.iter().map(|p| PickOption { id: p.id.clone(), label: p.label.clone() }).collect::<Vec<_>>(),
                            selected: selected_period.clone(),
                            label: "Period",
                            on_change: move |p: String| {
                                if let Some(handler) = on_period_change { handler.call(p); }
                            },
                        }
                    }
                    div { class: "kit-sales__export",
                        button {
                            r#type: "button",
                            class: "kit-sales__btn",
                            "aria-haspopup": "menu",
                            "aria-expanded": if export_open() { "true" } else { "false" },
                            disabled: on_export.is_none(),
                            onclick: move |_| export_open.set(!export_open()),
                            "Export"
                        }
                        if export_open() {
                            ul { class: "kit-sales__menu", role: "menu",
                                for format in ExportFormat::ALL {
                                    li { key: "{format.label()}", role: "none",
                                        button {
                                            r#type: "button",
                                            role: "menuitem",
                                            class: "kit-sales__menu-item",
                                            onclick: move |_| {
                                                export_open.set(false);
                                                if let Some(handler) = on_export { handler.call(format); }
                                            },
                                            "{format.label()}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "kit-sales__metrics", role: "group", "aria-label": "Metrics",
                for (idx, card) in metric_cards.iter().enumerate() {
                    {
                        let chosen = card.id == active_metric;
                        let id = card.id.clone();
                        let headline = shown_now.get(idx).cloned().unwrap_or_else(|| card.value.clone());
                        let x_labels_card: Vec<String> = (0..card.series.len()).map(|i| i.to_string()).collect();
                        let spark = TrendSeries { name: card.label.clone(), values: card.series.iter().map(|v| Some(*v)).collect(), tone: SeriesTone::Ink, emphasis: chosen, is_earlier: false };
                        rsx! {
                            div { key: "{card.id}", class: "kit-sales__metric", "data-dimmed": (!chosen).then_some("true"),
                                MetricTile {
                                    metric_label: card.label.clone(),
                                    metric_value: MetricValue::Text(headline),
                                    change_percent: Some(card.delta_percent.abs()),
                                    change_note: Some(if card.delta_percent >= 0.0 { "up vs last period".to_string() } else { "down vs last period".to_string() }),
                                    is_chosen: Some(chosen),
                                    target_attributes: vec![Attribute::new("aria-pressed", if chosen { "true" } else { "false" }, None, false)],
                                    on_activate: move |_| {
                                        local_metric.set(Some(id.clone()));
                                        if let Some(handler) = on_metric_change { handler.call(id.clone()); }
                                    },
                                    inline_chart: rsx! {
                                        SeriesTrendChart {
                                            x_labels: x_labels_card,
                                            series: vec![spark],
                                            variant: ChartVariant::Sparkline,
                                            axis: AxisMode::Off,
                                            show_tooltip: false,
                                            height: 34.0,
                                            label: card.label.clone(),
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
            section { class: "kit-sales__trend", "aria-label": "{trend_title} trend",
                header { class: "kit-sales__trend-head",
                    h3 { class: "kit-sales__trend-title", "{trend_title}" }
                    div { class: "kit-sales__trend-tools",
                        SegmentedControl {
                            aria_label: "Chart style",
                            value: style_value,
                            on_change: move |value: String| {
                                let style = if value == "bars" { ChartStyle::Bars } else { ChartStyle::Area };
                                chosen_style.set(style);
                                fade_swap(shown_style, style_fading, style_run, style);
                                if let Some(handler) = on_chart_style_change { handler.call(style); }
                            },
                            options: vec![
                                SegmentOption { value: "area".to_string(), label: Some(rsx! { "Area" }), ..Default::default() },
                                SegmentOption { value: "bars".to_string(), label: Some(rsx! { "Bars" }), ..Default::default() },
                            ],
                        }
                        Switch {
                            label: rsx! { "Compare" },
                            checked: compare_enabled,
                            onchange: move |e: FormEvent| {
                                if let Some(handler) = on_compare_change { handler.call(e.checked()); }
                            },
                        }
                    }
                }
                div { class: "kit-sales__chart", "data-fading": style_fading().then_some("true"),
                    SeriesTrendChart {
                        x_labels: x_labels,
                        series: series,
                        variant: variant,
                        label: trend_title.clone(),
                        height: 240.0,
                    }
                }
            }
            div { class: "kit-sales__breakdowns",
                {breakdown_rows("Top products".to_string(), products, format_amount)}
                {breakdown_rows("Channels".to_string(), channel_rows, format_amount)}
            }
        }
    }
}
