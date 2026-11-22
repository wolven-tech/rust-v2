//! Part-of-total card: one total and what it is made of. A ranked list with values and
//! shares, a stacked bar, and an optional period switch. Hovering or focusing a part
//! highlights its segment and row and swaps the total for that part's amount.
//!
//! Design choice: parts are identified by `part_key` (falling back to the name), so a
//! row keeps its DOM node when the period changes and its rank moves.
//! Design choice: the period switch is a radio group with one tab stop; Left and Right
//! wrap at the ends, Home and End jump.
//! Design choice: the trailing row defaults to "Top part: <name>"; `show_trailing: false`
//! hides it, and `trailing_row` replaces it.
//! Not implemented: the number roll and bar glide between periods. Values re-render at
//! their new numbers and widths.
//! Browser checks: the 880ms period slide, bar width transitions, row reorder, touch
//! targets at narrow widths, and the share column hiding below the narrow breakpoint.

use dioxus::prelude::*;

use crate::components::series_trend_chart::{
    SeriesTone, finite_or_zero, focus_element, group_digits, non_negative, share, signed_percent,
    wrap_index,
};
use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/part-of-total-card.css");

#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub part_key: Option<String>,
    pub part_name: String,
    pub part_amount: f64,
    pub part_color: Option<SeriesTone>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PeriodOption {
    pub period_key: String,
    pub period_name: String,
}

/// Identity of a part: its key, else its name.
pub fn part_identity(part: &Part) -> String {
    part.part_key
        .clone()
        .unwrap_or_else(|| part.part_name.clone())
}

/// Indices into `parts` by amount, highest first. Equal amounts keep input order.
pub fn ranked_parts(parts: &[Part]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..parts.len()).collect();
    order.sort_by(|a, b| {
        let x = finite_or_zero(parts[*a].part_amount);
        let y = finite_or_zero(parts[*b].part_amount);
        y.partial_cmp(&x).unwrap_or(std::cmp::Ordering::Equal)
    });
    order
}

/// Sum of the non-negative amounts.
pub fn total_amount(parts: &[Part]) -> f64 {
    parts.iter().map(|p| non_negative(p.part_amount)).sum()
}

/// Signed change text and whether it takes the negative style.
pub fn change_tag(change: f64) -> (String, bool) {
    (signed_percent(change), finite_or_zero(change) < 0.0)
}

/// A card with one total, a ranked list of its parts, a stacked bar and an optional
/// period switch. The host supplies matching `part_entries` and `change_pct` for each
/// period it reports through `on_period_change`.
#[component]
pub fn PartOfTotalCard(
    part_entries: Vec<Part>,
    #[props(default = "Composition".to_string(), into)] card_title: String,
    #[props(default)] period_options: Vec<PeriodOption>,
    #[props(default)] active_period: Option<String>,
    #[props(default)] initial_period: Option<String>,
    #[props(default)] on_period_change: Option<EventHandler<String>>,
    #[props(default)] change_pct: Option<f64>,
    #[props(default = group_digits as fn(f64) -> String)] format_amount: fn(f64) -> String,
    #[props(default = "Overall".to_string(), into)] total_caption: String,
    #[props(default = true)] show_trailing: bool,
    #[props(default)] trailing_row: Option<Element>,
    #[props(default, into)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-pot-{}", next_id()));
    let mut hovered = use_signal(|| None::<String>);
    let mut local_period = use_signal(|| None::<String>);

    let period_keys: Vec<String> = period_options
        .iter()
        .map(|o| o.period_key.clone())
        .collect();
    let selected_index = active_period
        .clone()
        .or_else(|| local_period())
        .or_else(|| initial_period.clone())
        .and_then(|key| period_keys.iter().position(|k| *k == key))
        .unwrap_or(0);
    let selected_key = period_keys.get(selected_index).cloned();

    let order = ranked_parts(&part_entries);
    let total = total_amount(&part_entries);
    let hovered_key = hovered();
    let focused = hovered_key
        .as_ref()
        .and_then(|key| part_entries.iter().find(|p| part_identity(p) == *key));
    let (total_text, caption) = match focused {
        Some(part) => (
            format_amount(non_negative(part.part_amount)),
            format!(
                "{} of total",
                share_label_text(share(part.part_amount, total))
            ),
        ),
        None => (format_amount(total), total_caption.clone()),
    };
    let change = change_pct.map(change_tag);
    let change_words = match change_pct {
        Some(value) if finite_or_zero(value) > 0.0 => format!(
            "up {} vs previous period",
            signed_percent(value).trim_start_matches('+')
        ),
        Some(value) if finite_or_zero(value) < 0.0 => format!(
            "down {} vs previous period",
            signed_percent(value).trim_start_matches('-')
        ),
        Some(_) => "no change vs previous period".to_string(),
        None => String::new(),
    };
    let summary = if change_words.is_empty() {
        format!("Total {total_text}")
    } else {
        format!("Total {total_text}, {change_words}")
    };
    let top_part = order.first().map(|i| part_entries[*i].part_name.clone());
    let trailing = if !show_trailing {
        None
    } else if let Some(element) = trailing_row {
        Some(TrailingKind::Custom(element))
    } else {
        top_part.map(TrailingKind::TopPart)
    };
    let class = class_list("kit-pot", &extra_class);
    let has_focus = hovered_key.is_some();
    let key_base = base.clone();
    let period_count = period_keys.len();

    let controlled = active_period.is_some();
    let mut pick = move |key: String| {
        if !controlled {
            local_period.set(Some(key.clone()));
        }
        if let Some(handler) = &on_period_change {
            handler.call(key);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": card_title.clone(), "data-focus": has_focus.then_some("true"),
            header { class: "kit-pot__head",
                h3 { class: "kit-pot__title", "{card_title}" }
                if period_count > 0 {
                    div {
                        class: "kit-pot__periods",
                        role: "radiogroup",
                        "aria-label": "Period",
                        onkeydown: {
                            let key_base = key_base.clone();
                            let keys = period_keys.clone();
                            let current = selected_index;
                            move |event: KeyboardEvent| {
                                let next = match event.key() {
                                    Key::ArrowRight => Some(wrap_index(current, period_count, true)),
                                    Key::ArrowLeft => Some(wrap_index(current, period_count, false)),
                                    Key::Home => Some(0),
                                    Key::End => Some(period_count - 1),
                                    _ => None,
                                };
                                if let Some(index) = next {
                                    event.prevent_default();
                                    pick(keys[index].clone());
                                    focus_element(&format!("{key_base}-p-{index}"));
                                }
                            }
                        },
                        for (index, option) in period_options.iter().enumerate() {
                            button {
                                key: "{option.period_key}",
                                id: "{key_base}-p-{index}",
                                r#type: "button",
                                role: "radio",
                                "aria-checked": if Some(&option.period_key) == selected_key.as_ref() { "true" } else { "false" },
                                tabindex: if index == selected_index { "0" } else { "-1" },
                                class: "kit-pot__period",
                                onclick: {
                                    let key = option.period_key.clone();
                                    move |_| pick(key.clone())
                                },
                                "{option.period_name}"
                            }
                        }
                    }
                }
            }
            div { class: "kit-pot__total",
                span { class: "kit-pot__figure", "{total_text}" }
                if let Some((text, negative)) = change.clone() {
                    span { class: "kit-pot__change", "data-negative": negative.then_some("true"), "{text}" }
                }
            }
            p { class: "kit-pot__caption", "{caption}" }
            p { class: "kit-pot__sr", role: "status", "{summary}" }
            if part_entries.is_empty() {
                p { class: "kit-pot__empty", "No parts" }
            } else {
                div { class: "kit-pot__bar", "aria-hidden": "true",
                    for index in order.iter().copied() {
                        {
                            let part = part_entries[index].clone();
                            let key = part_identity(&part);
                            let width = format!("{:.2}%", share(part.part_amount, total) * 100.0);
                            let tone = part.part_color.unwrap_or_default().as_str();
                            let lit = hovered_key.as_deref() == Some(key.as_str());
                            let enter_key = key.clone();
                            rsx! {
                                span {
                                    key: "{key}",
                                    class: "kit-pot__segment",
                                    "data-tone": tone,
                                    "data-lit": lit.then_some("true"),
                                    style: "width: {width}",
                                    onmouseenter: move |_| hovered.set(Some(enter_key.clone())),
                                    onmouseleave: move |_| hovered.set(None),
                                }
                            }
                        }
                    }
                }
                ol { class: "kit-pot__list",
                    for index in order.iter().copied() {
                        {
                            let part = part_entries[index].clone();
                            let key = part_identity(&part);
                            let amount = format_amount(non_negative(part.part_amount));
                            let pct = share_label_text(share(part.part_amount, total));
                            let tone = part.part_color.unwrap_or_default().as_str();
                            let lit = hovered_key.as_deref() == Some(key.as_str());
                            let enter_key = key.clone();
                            let focus_key = key.clone();
                            let blur_key = key.clone();
                            let name = part.part_name.clone();
                            rsx! {
                                li {
                                    key: "{key}",
                                    class: "kit-pot__row",
                                    tabindex: "0",
                                    "data-tone": tone,
                                    "data-lit": lit.then_some("true"),
                                    "aria-label": "{name}, {amount}, {pct}",
                                    onmouseenter: move |_| hovered.set(Some(enter_key.clone())),
                                    onmouseleave: move |_| hovered.set(None),
                                    onfocus: move |_| hovered.set(Some(focus_key.clone())),
                                    onblur: move |_| {
                                        if hovered().as_deref() == Some(blur_key.as_str()) {
                                            hovered.set(None);
                                        }
                                    },
                                    span { class: "kit-pot__swatch", "aria-hidden": "true" }
                                    span { class: "kit-pot__name", "{name}" }
                                    span { class: "kit-pot__amount", "{amount}" }
                                    span { class: "kit-pot__share", "{pct}" }
                                }
                            }
                        }
                    }
                }
                match trailing {
                    Some(TrailingKind::TopPart(name)) => rsx! {
                        p { class: "kit-pot__trailing", "Top part: {name}" }
                    },
                    Some(TrailingKind::Custom(element)) => rsx! {
                        div { class: "kit-pot__trailing", {element} }
                    },
                    None => rsx! {},
                }
            }
        }
    }
}

enum TrailingKind {
    TopPart(String),
    Custom(Element),
}

fn share_label_text(ratio: f64) -> String {
    crate::components::series_trend_chart::share_label(ratio)
}
