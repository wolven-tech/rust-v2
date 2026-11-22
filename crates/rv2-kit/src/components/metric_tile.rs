//! Metric tile: one headline figure with its label, the change against the previous
//! period, an optional chart and a footer. The whole tile can act as a link or button.
//!
//! Design choice: the figure does not roll digit by digit. A new value re-keys the
//! figure, which plays a short CSS rise-in; the settled text is what assistive
//! technology reads.
//! Design choice: the whole-tile link or button is a stretched overlay, so the corner
//! control stays a real control instead of sitting inside an `a`.
//! Design choice: `format_change` receives the magnitude only; the sign comes from
//! the direction badge.
//! Not implemented: the roll-up from zero on first view.
//! Browser checks: rise-in on value change, placeholder sheen while loading,
//! hover and focus emphasis on the overlay, reduced motion, the corner control
//! operating without following the link.

use dioxus::prelude::*;

use crate::components::series_trend_chart::{finite_or_zero, group_digits};
use crate::internal::KIT_CSS;

const CSS: Asset = asset!("/assets/metric-tile.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartPosition {
    /// Chart beneath the figure.
    #[default]
    Stacked,
    /// Chart beside the figure.
    Inline,
}

impl ChartPosition {
    fn as_str(self) -> &'static str {
        match self {
            Self::Stacked => "stacked",
            Self::Inline => "inline",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetricValue {
    /// Formatted with the tile's `format_value`.
    Number(f64),
    /// Shown exactly as given.
    Text(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeDirection {
    Up,
    Down,
    Flat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeTone {
    Favourable,
    Unfavourable,
    Neutral,
}

/// Direction and tone of a change. A rise is unfavourable when lower is better.
pub fn change_tone(change: f64, lower_is_better: bool) -> (ChangeDirection, ChangeTone) {
    let change = finite_or_zero(change);
    if change > 0.0 {
        let tone = if lower_is_better {
            ChangeTone::Unfavourable
        } else {
            ChangeTone::Favourable
        };
        (ChangeDirection::Up, tone)
    } else if change < 0.0 {
        let tone = if lower_is_better {
            ChangeTone::Favourable
        } else {
            ChangeTone::Unfavourable
        };
        (ChangeDirection::Down, tone)
    } else {
        (ChangeDirection::Flat, ChangeTone::Neutral)
    }
}

/// Default magnitude formatter: `16.0%`.
pub fn format_percent(magnitude: f64) -> String {
    format!("{:.1}%", finite_or_zero(magnitude))
}

/// The change as words for assistive technology: `up 16 percent vs last week`.
pub fn change_words(change: f64, magnitude: &str, note: Option<&str>) -> String {
    let direction = match change_tone(change, false).0 {
        ChangeDirection::Up => "up",
        ChangeDirection::Down => "down",
        ChangeDirection::Flat => return "no change".to_string(),
    };
    let magnitude = magnitude.replace('%', " percent");
    match note {
        Some(note) if !note.is_empty() => format!("{direction} {magnitude} {note}"),
        _ => format!("{direction} {magnitude}"),
    }
}

/// Accessible name of an interactive tile: label, value and change.
pub fn accessible_name(label: &str, value: &str, change: Option<&str>) -> String {
    match change {
        Some(change) if !change.is_empty() => format!("{label}, {value}, {change}"),
        _ => format!("{label}, {value}"),
    }
}

/// A labelled group of tiles: a header row with an optional control at its trailing
/// edge, above a two-column grid that drops to one column on narrow widths. The control
/// belongs to the group, not to a tile.
#[component]
pub fn MetricTileGroup(
    #[props(into)] group_label: String,
    #[props(default)] header_control: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-tile-group",
            header { class: "kit-tile-group__head",
                h3 { class: "kit-tile-group__label", "{group_label}" }
                if let Some(control) = header_control {
                    div { class: "kit-tile-group__control", {control} }
                }
            }
            div { class: "kit-tile-group__grid", {children} }
        }
    }
}

/// A headline figure with its change, optional chart and footer. Pass a `target_href`
/// or an `on_activate` to make the whole tile one link or button. While `is_loading`,
/// placeholders hold the value, change and chart positions and the tile is busy.
#[component]
pub fn MetricTile(
    #[props(into)] metric_label: String,
    metric_value: MetricValue,
    #[props(default = group_digits as fn(f64) -> String)] format_value: fn(f64) -> String,
    #[props(default)] change_percent: Option<f64>,
    #[props(default = format_percent as fn(f64) -> String)] format_change: fn(f64) -> String,
    #[props(default)] change_note: Option<String>,
    #[props(default)] lower_is_better: bool,
    #[props(default)] metric_mark: Option<Element>,
    #[props(default)] inline_chart: Option<Element>,
    #[props(default)] chart_position: ChartPosition,
    #[props(default)] footer_content: Option<Element>,
    #[props(default)] is_loading: bool,
    #[props(default)] target_href: Option<String>,
    #[props(default)] on_activate: Option<EventHandler<()>>,
    #[props(default)] corner_control: Option<Element>,
    #[props(default)] is_chosen: Option<bool>,
    #[props(default)] target_attributes: Vec<Attribute>,
) -> Element {
    let value_text = match &metric_value {
        MetricValue::Number(n) => format_value(*n),
        MetricValue::Text(text) => text.clone(),
    };
    let numeric = matches!(metric_value, MetricValue::Number(_));
    let change_value = change_percent.map(finite_or_zero);
    let (direction, tone) = change_tone(change_value.unwrap_or(0.0), lower_is_better);
    let arrow = match direction {
        ChangeDirection::Up => "↑",
        ChangeDirection::Down => "↓",
        ChangeDirection::Flat => "→",
    };
    let tone_name = match tone {
        ChangeTone::Favourable => "favourable",
        ChangeTone::Unfavourable => "unfavourable",
        ChangeTone::Neutral => "neutral",
    };
    let magnitude = format_change(change_value.unwrap_or(0.0).abs());
    let change_text = change_value.map(|_| format!("{arrow} {magnitude}"));
    let words = change_value.map(|value| change_words(value, &magnitude, change_note.as_deref()));
    let interactive = target_href.is_some() || on_activate.is_some();
    let name = accessible_name(&metric_label, &value_text, words.as_deref());
    let chosen_state = match is_chosen {
        Some(true) => "chosen",
        Some(false) => "unchosen",
        None => "none",
    };
    let position = chart_position.as_str();
    let show_chart = inline_chart.is_some() && !is_loading;
    let extra_attributes = target_attributes.clone();
    let label_text = metric_label.clone();
    let rendered_value = value_text.clone();
    let note_text = change_note.clone().unwrap_or_default();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-tile",
            "data-interactive": interactive.then_some("true"),
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            "data-chosen": chosen_state,
            "data-tone": tone_name,
            header { class: "kit-tile__top",
                if let Some(mark) = metric_mark {
                    span { class: "kit-tile__mark", "aria-hidden": "true", {mark} }
                }
                span { class: "kit-tile__label", "{label_text}" }
                if let Some(corner) = corner_control {
                    div { class: "kit-tile__corner", {corner} }
                }
            }
            div { class: "kit-tile__body", "data-position": position,
                div { class: "kit-tile__figure-wrap",
                    if is_loading {
                        span { class: "kit-tile__placeholder kit-tile__placeholder--value", "aria-hidden": "true" }
                    } else if numeric {
                        span { key: "{rendered_value}", class: "kit-tile__figure", "{rendered_value}" }
                    } else {
                        span { class: "kit-tile__figure kit-tile__figure--text", "{rendered_value}" }
                    }
                    if is_loading {
                        div { class: "kit-tile__change", "aria-hidden": "true",
                            span { class: "kit-tile__placeholder kit-tile__placeholder--badge" }
                            span { class: "kit-tile__placeholder kit-tile__placeholder--note" }
                        }
                    } else if let Some(text) = change_text.clone() {
                        div { class: "kit-tile__change",
                            span { class: "kit-tile__badge", "{text}" }
                            if !note_text.is_empty() {
                                span { class: "kit-tile__note", "{note_text}" }
                            }
                        }
                    }
                }
                if is_loading {
                    if inline_chart.is_some() {
                        span { class: "kit-tile__placeholder kit-tile__placeholder--chart", "aria-hidden": "true" }
                    }
                } else if show_chart {
                    div { class: "kit-tile__chart", {inline_chart.clone()} }
                }
            }
            if let Some(footer) = footer_content {
                footer { class: "kit-tile__footer", {footer} }
            }
            if let Some(href) = target_href.clone() {
                a { class: "kit-tile__hit", href: href, "aria-label": name.clone(), ..extra_attributes.clone() }
            } else if let Some(handler) = on_activate {
                button {
                    class: "kit-tile__hit",
                    r#type: "button",
                    "aria-label": name.clone(),
                    onclick: move |_| handler.call(()),
                    ..extra_attributes.clone()
                }
            }
        }
    }
}
