//! Context fill grid: how full a model's context window is, as cells filled part by part.
//! Hover a part to light its cells, expand a part to see its items, remove items, and
//! compact a part, which empties its cells.
//!
//! Design choice: one cell is `window_size / cell_total` tokens. Any part with tokens
//! gets at least one cell. When the parts overflow `cell_total`, later parts lose cells.
//! Design choice: a part's tokens are the sum of its items when it has any; `token_count`
//! is then ignored.
//! Design choice: without `on_compact_request` the component contracts the part itself to
//! a quarter of its tokens. With it, the host owns the change and reports the new
//! `part_list`; `compacting_part` marks the busy part. The promise-based busy state of
//! the source design is replaced by this prop, since an event handler cannot return one.
//! Design choice: without `on_item_remove` removal stays inside the card; with it the card
//! shows exactly the parts given.
//! Design choice: after a compaction, Compact is unavailable until the part's tokens
//! exceed the count it was pressed at.
//! Not implemented: the sweep and shimmer across cells and the removal fold-out; cells
//! change at once. Reduced motion is respected by CSS.
//! Browser checks: the cell sweep on change, the compacting shimmer, disclosure
//! keyboard use, two-column layout at phone width.

use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;

use crate::components::series_trend_chart::{SeriesTone, non_negative, share, share_label};
use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/context-fill-grid.css");

/// Fraction of a part's tokens that remains after a compaction the card performs itself.
const LOCAL_COMPACT_SHARE: f64 = 0.25;

#[derive(Clone, Debug, PartialEq)]
pub struct ContextItem {
    pub item_key: String,
    pub item_name: String,
    pub item_tokens: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextPart {
    pub part_key: String,
    pub part_name: String,
    pub token_count: Option<f64>,
    pub sub_items: Vec<ContextItem>,
    pub is_locked: bool,
    pub can_compact: bool,
    pub part_color: Option<SeriesTone>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FooterAction {
    pub action_text: String,
    pub on_activate: EventHandler<()>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillLevel {
    Normal,
    Caution,
    Full,
}

/// Tokens of a part: the sum of its items when it has any, else its `token_count`.
pub fn part_tokens(part: &ContextPart) -> f64 {
    if part.sub_items.is_empty() {
        non_negative(part.token_count.unwrap_or(0.0))
    } else {
        part.sub_items
            .iter()
            .map(|i| non_negative(i.item_tokens))
            .sum()
    }
}

/// Cells for a part: none without tokens, otherwise `tokens / (window / cells)` rounded,
/// and at least one.
pub fn cell_count(tokens: f64, window: f64, cell_total: usize) -> usize {
    if tokens <= 0.0 || window <= 0.0 || cell_total == 0 {
        return 0;
    }
    let unit = window / cell_total as f64;
    ((tokens / unit).round() as usize).max(1)
}

/// Cells per part in part order, clipped so the total never exceeds `cell_total`.
pub fn allocate_cells(tokens: &[f64], window: f64, cell_total: usize) -> Vec<usize> {
    let mut remaining = cell_total;
    tokens
        .iter()
        .map(|t| {
            let wanted = cell_count(*t, window, cell_total).min(remaining);
            remaining -= wanted;
            wanted
        })
        .collect()
}

/// Caution at or above `caution_share`, full at or above `full_share`.
pub fn fill_level(used_share: f64, caution_share: f64, full_share: f64) -> FillLevel {
    if used_share >= full_share {
        FillLevel::Full
    } else if used_share >= caution_share {
        FillLevel::Caution
    } else {
        FillLevel::Normal
    }
}

fn default_tone(index: usize) -> SeriesTone {
    const PALETTE: [SeriesTone; 4] = [
        SeriesTone::Ink,
        SeriesTone::Ok,
        SeriesTone::Warn,
        SeriesTone::Muted,
    ];
    PALETTE[index % PALETTE.len()]
}

/// A context window as a grid of cells, with a part list and compaction controls.
#[component]
pub fn ContextFillGrid(
    part_list: Vec<ContextPart>,
    window_size: f64,
    #[props(default = "Context usage".to_string(), into)] panel_title: String,
    #[props(default, into)] model_name: String,
    #[props(default)] model_mark: Option<Element>,
    #[props(default)] on_item_remove: Option<EventHandler<(String, String)>>,
    #[props(default)] on_compact_request: Option<EventHandler<String>>,
    #[props(default)] compacting_part: Option<String>,
    #[props(default)] footer_action: Option<FooterAction>,
    #[props(default = 0.8)] caution_share: f64,
    #[props(default = 0.95)] full_share: f64,
    #[props(default = 100)] cell_total: usize,
    #[props(default = 20)] cells_per_row: usize,
    #[props(default, into)] root_class: String,
) -> Element {
    let mut removed = use_signal(BTreeSet::<(String, String)>::new);
    let mut compacted = use_signal(BTreeMap::<String, f64>::new);
    let mut expanded = use_signal(BTreeSet::<String>::new);
    let mut hovered = use_signal(|| None::<String>);

    let has_remove_callback = on_item_remove.is_some();
    let has_compact_callback = on_compact_request.is_some();
    let removed_now = removed();
    let compacted_now = compacted();

    let parts: Vec<(ContextPart, f64, bool)> = part_list
        .iter()
        .map(|part| {
            let mut effective = part.clone();
            if !has_remove_callback {
                effective.sub_items.retain(|i| {
                    !removed_now.contains(&(part.part_key.clone(), i.item_key.clone()))
                });
            }
            let original = part_tokens(&effective);
            let local_compacted =
                !has_compact_callback && compacted_now.contains_key(&part.part_key);
            let tokens = if local_compacted {
                original * LOCAL_COMPACT_SHARE
            } else {
                original
            };
            let cooling = compacted_now
                .get(&part.part_key)
                .is_some_and(|recorded| original <= *recorded);
            (effective, tokens, cooling)
        })
        .collect();

    let window = non_negative(window_size).max(f64::EPSILON);
    let token_values: Vec<f64> = parts.iter().map(|(_, t, _)| *t).collect();
    let used: f64 = token_values.iter().sum();
    let used_share = share(used, window);
    let level = fill_level(used_share, caution_share, full_share);
    let level_name = match level {
        FillLevel::Normal => "normal",
        FillLevel::Caution => "caution",
        FillLevel::Full => "full",
    };
    let cells = allocate_cells(&token_values, window, cell_total);
    let filled_total: usize = cells.iter().sum();
    let free_cells = cell_total.saturating_sub(filled_total);
    let percent = share_label(used_share);
    let used_text = format!("{:.0} of {:.0} tokens used", used, window);
    let left = (window - used).max(0.0);
    let figure_sr = format!("{used_text}, {percent}");
    let hovered_key = hovered();
    let is_compacting = compacting_part.is_some();
    let caption = if let Some(key) = compacting_part.as_ref() {
        let name = part_list
            .iter()
            .find(|p| p.part_key == *key)
            .map(|p| p.part_name.clone())
            .unwrap_or_else(|| "Part".to_string());
        format!("Compacting {name}…")
    } else if level == FillLevel::Full {
        "Window full".to_string()
    } else if level == FillLevel::Caution {
        format!("{left:.0} tokens left")
    } else {
        format!("{left:.0} tokens free")
    };
    let class = class_list("kit-ctx", &root_class);
    let cols = format!("--kit-cols: {}", cells_per_row.max(1));
    let title = panel_title.clone();
    let model = model_name.clone();
    let cell_list: Vec<(String, &'static str, bool)> = {
        let mut out = Vec::new();
        for (index, ((part, _, _), count)) in parts.iter().zip(cells.iter()).enumerate() {
            let tone = part
                .part_color
                .unwrap_or_else(|| default_tone(index))
                .as_str();
            let dim = hovered_key.as_ref().is_some_and(|k| *k != part.part_key);
            for c in 0..*count {
                out.push((format!("{}-{c}", part.part_key), tone, dim));
            }
        }
        for c in 0..free_cells {
            out.push((format!("free-{c}"), "free", false));
        }
        out
    };
    let rows: Vec<(
        usize,
        String,
        String,
        String,
        String,
        bool,
        bool,
        bool,
        Option<String>,
        Vec<(String, String, String)>,
        bool,
        &'static str,
    )> = parts
        .iter()
        .enumerate()
        .map(|(index, (part, tokens, cooling))| {
            let key = part.part_key.clone();
            let is_open = expanded().contains(&key);
            let tone = part
                .part_color
                .unwrap_or_else(|| default_tone(index))
                .as_str();
            let token_text = format!("{:.0}", tokens);
            let items: Vec<(String, String, String)> = part
                .sub_items
                .iter()
                .map(|item| {
                    (
                        item.item_key.clone(),
                        item.item_name.clone(),
                        format!("{:.0}", non_negative(item.item_tokens)),
                    )
                })
                .collect();
            let compact_busy = compacting_part.as_deref() == Some(key.as_str());
            let can_compact = part.can_compact && !part.is_locked && !is_compacting;
            let compact_disabled = !can_compact || *cooling;
            let compacted_label = compacted_now.contains_key(&key) && *cooling;
            (
                index,
                key.clone(),
                part.part_name.clone(),
                token_text,
                tone.to_string(),
                is_open,
                part.sub_items.is_empty(),
                compact_disabled,
                if compacted_label {
                    Some("Compacted".to_string())
                } else {
                    None
                },
                items,
                compact_busy,
                tone,
            )
        })
        .collect();
    let footer = footer_action.clone();
    let model_mark_el = model_mark.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class, "aria-label": title.clone(), "data-level": level_name,
            header { class: "kit-ctx__head",
                h3 { class: "kit-ctx__title", "{title}" }
                if !model.is_empty() {
                    span { class: "kit-ctx__model",
                        if let Some(mark) = model_mark_el {
                            span { class: "kit-ctx__mark", "aria-hidden": "true", {mark} }
                        }
                        "{model}"
                    }
                }
            }
            div { class: "kit-ctx__used",
                span { class: "kit-ctx__figure", "aria-hidden": "true", "{used_text}" }
                span { class: "kit-ctx__pct", "data-level": level_name, "{percent}" }
                span { class: "kit-ctx__sr", "{figure_sr}" }
            }
            div { class: "kit-ctx__grid", "aria-hidden": "true", style: cols,
                for (idx, (key, tone, dim)) in cell_list.iter().cloned().enumerate() {
                    span {
                        key: "{idx}-{key}",
                        class: "kit-ctx__cell",
                        "data-tone": tone,
                        "data-dim": dim.then_some("true"),
                        "data-free": (tone == "free").then_some("true"),
                    }
                }
            }
            ul { class: "kit-ctx__parts",
                for (index, key, name, token_text, tone, is_open, leaf, _disabled, _label, items, _busy, _t) in rows.clone() {
                    li { key: "{index}", class: "kit-ctx__part", "data-tone": tone, "data-lit": (hovered_key.as_deref() == Some(key.as_str())).then_some("true"),
                        onmouseenter: {
                            let key = key.clone();
                            move |_| hovered.set(Some(key.clone()))
                        },
                        onmouseleave: move |_| hovered.set(None),
                        button {
                            r#type: "button",
                            class: "kit-ctx__row",
                            "aria-expanded": (!leaf).then_some(if is_open { "true" } else { "false" }),
                            disabled: leaf,
                            onclick: {
                                let key = key.clone();
                                move |_| {
                                    let mut next = expanded();
                                    if next.contains(&key) {
                                        next.remove(&key);
                                    } else {
                                        next.insert(key.clone());
                                    }
                                    expanded.set(next);
                                }
                            },
                            span { class: "kit-ctx__swatch", "aria-hidden": "true" }
                            span { class: "kit-ctx__name", "{name}" }
                            span { class: "kit-ctx__tokens", "{token_text}" }
                        }
                        if is_open && !leaf {
                            ul { class: "kit-ctx__items",
                                for (item_key, item_name, item_text) in items.clone() {
                                    li { key: "{item_key}", class: "kit-ctx__item",
                                        span { class: "kit-ctx__item-name", "{item_name}" }
                                        span { class: "kit-ctx__item-tokens", "{item_text}" }
                                        button {
                                            r#type: "button",
                                            class: "kit-ctx__remove",
                                            "aria-label": "Remove {item_name}",
                                            onclick: {
                                                let part_key = key.clone();
                                                let item_key = item_key.clone();
                                                move |_| {
                                                    if let Some(handler) = &on_item_remove {
                                                        handler.call((part_key.clone(), item_key.clone()));
                                                    } else {
                                                        let mut next = removed();
                                                        next.insert((part_key.clone(), item_key.clone()));
                                                        removed.set(next);
                                                    }
                                                }
                                            },
                                            "Remove"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            footer { class: "kit-ctx__foot",
                p { class: "kit-ctx__caption", role: "status", "aria-live": "polite", "data-level": level_name, "{caption}" }
                div { class: "kit-ctx__actions",
                    for (index, (part, _, _)) in parts.iter().enumerate() {
                        if part.can_compact && !part.is_locked {
                            {
                                let key = part.part_key.clone();
                                let name = part.part_name.clone();
                                let (_, _, cooling) = parts[index].clone();
                                let busy = compacting_part.as_deref() == Some(key.as_str());
                                let original = part_tokens(part);
                                let disabled = cooling || is_compacting;
                                let label = if busy {
                                    "Compacting…".to_string()
                                } else if cooling {
                                    "Compacted".to_string()
                                } else {
                                    format!("Compact {name}")
                                };
                                let click_key = key.clone();
                                rsx! {
                                    button {
                                        key: "{index}",
                                        r#type: "button",
                                        class: "kit-ctx__compact",
                                        disabled: disabled,
                                        "aria-busy": busy.then_some("true"),
                                        "data-original": format!("{original:.0}"),
                                        onclick: move |_| {
                                            if let Some(handler) = &on_compact_request {
                                                handler.call(click_key.clone());
                                            } else {
                                                let mut next = compacted();
                                                next.insert(click_key.clone(), original);
                                                compacted.set(next);
                                            }
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                    if let Some(action) = footer {
                        button {
                            r#type: "button",
                            class: "kit-ctx__footer-action",
                            onclick: move |_| action.on_activate.call(()),
                            "{action.action_text}"
                        }
                    }
                }
            }
        }
    }
}
