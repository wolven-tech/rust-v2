//! Parallel work grid: one agent fans a job out to many workers, one cell per task, filling in as they complete.
//!
//! Design choice: remaining time needs `concurrency` and at least three finished items. The estimate is the mean
//! duration of finished items times the items still to do, divided by `concurrency`.
//! Design choice: `start_offset`/`end_offset` are job-clock offsets, used only for their difference.
//! Design choice: the grid is a roving-focus group; focus moves with the arrow keys, and details follow focus.
//! Browser checks: the wave on first render, sweep highlight on in-progress cells, pop on landing, details window scale.

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/parallel-work-grid.css");

/// Finished items needed before the remaining-time estimate appears.
pub const ESTIMATE_MIN_DONE: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemState {
    Queued,
    InProgress,
    Retrying,
    Done,
    Failed,
}

impl ItemState {
    pub fn word(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::InProgress => "In progress",
            Self::Retrying => "Retrying",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateFilter {
    Done,
    InProgress,
    Retrying,
    Failed,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Item {
    pub item_id: String,
    pub item_label: String,
    pub item_state: ItemState,
    pub worker_name: Option<String>,
    pub start_offset: Option<f64>,
    pub end_offset: Option<f64>,
    pub item_cost: Option<f64>,
    pub attempt_count: u32,
    pub output_value: Option<String>,
    pub error_text: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub done: usize,
    pub in_progress: usize,
    pub retrying: usize,
    pub failed: usize,
    pub queued: usize,
}

pub fn counts(items: &[Item]) -> Counts {
    let mut c = Counts::default();
    for item in items {
        match item.item_state {
            ItemState::Queued => c.queued += 1,
            ItemState::InProgress => c.in_progress += 1,
            ItemState::Retrying => c.retrying += 1,
            ItemState::Done => c.done += 1,
            ItemState::Failed => c.failed += 1,
        }
    }
    c
}

pub fn total_cost(items: &[Item]) -> f64 {
    items.iter().filter_map(|i| i.item_cost).sum()
}

/// Seconds left, or `None` while hidden: no concurrency, too few finished, or nothing left to do.
pub fn remaining_secs(items: &[Item], concurrency: Option<u32>) -> Option<f64> {
    let workers = concurrency.filter(|c| *c > 0)? as f64;
    let durations: Vec<f64> = items
        .iter()
        .filter(|i| i.item_state == ItemState::Done)
        .filter_map(|i| Some(i.end_offset? - i.start_offset?))
        .collect();
    if durations.len() < ESTIMATE_MIN_DONE {
        return None;
    }
    let left = items
        .iter()
        .filter(|i| {
            matches!(
                i.item_state,
                ItemState::Queued | ItemState::InProgress | ItemState::Retrying
            )
        })
        .count();
    if left == 0 {
        return None;
    }
    let mean = durations.iter().sum::<f64>() / durations.len() as f64;
    Some(mean * left as f64 / workers)
}

/// Ids of failed items, in cell order, for "Retry failed".
pub fn retry_ids(items: &[Item]) -> Vec<String> {
    items
        .iter()
        .filter(|i| i.item_state == ItemState::Failed)
        .map(|i| i.item_id.clone())
        .collect()
}

/// Progress tenths reached: finished plus failed, out of the total.
pub fn progress_tenths(items: &[Item]) -> usize {
    if items.is_empty() {
        return 0;
    }
    let settled = items
        .iter()
        .filter(|i| matches!(i.item_state, ItemState::Done | ItemState::Failed))
        .count();
    settled * 10 / items.len()
}

pub fn matches_filter(item: &Item, filter: Option<StateFilter>) -> bool {
    match filter {
        None => true,
        Some(StateFilter::Done) => item.item_state == ItemState::Done,
        Some(StateFilter::InProgress) => item.item_state == ItemState::InProgress,
        Some(StateFilter::Retrying) => item.item_state == ItemState::Retrying,
        Some(StateFilter::Failed) => item.item_state == ItemState::Failed,
    }
}

/// Duration of a finished or running item, in seconds, when both offsets are known.
pub fn item_duration(item: &Item) -> Option<f64> {
    Some(item.end_offset? - item.start_offset?)
}

fn filter_key(filter: StateFilter) -> &'static str {
    match filter {
        StateFilter::Done => "done",
        StateFilter::InProgress => "in progress",
        StateFilter::Retrying => "retrying",
        StateFilter::Failed => "failed",
    }
}

/// Fan-out grid of job items. Hovering or focusing a cell shows its details; a click pins them.
#[component]
pub fn ParallelWorkGrid(
    job_items: Vec<Item>,
    #[props(default)] job_name: String,
    #[props(default)] concurrency: Option<u32>,
    #[props(default)] clock_offset: f64,
    #[props(default)] on_retry: Option<EventHandler<Vec<String>>>,
    #[props(default)] on_pick: Option<EventHandler<Option<Item>>>,
    #[props(default)] format_money: Option<Callback<f64, String>>,
    #[props(default = "Fan-out".to_string(), into)] heading_text: String,
    #[props(default)] heading_icon: Option<String>,
    #[props(default)] extra_class: String,
) -> Element {
    let base = use_hook(|| format!("kit-pwg-{}", next_id()));
    let mut picked = use_signal(|| None::<usize>);
    let mut focused = use_signal(|| 0usize);
    let mut filter = use_signal(|| None::<StateFilter>);
    let mut dismissed = use_signal(|| false);

    let c = counts(&job_items);
    let total = job_items.len();
    let complete = c.done + c.failed == total && total > 0;
    let cost = total_cost(&job_items);
    let remaining = remaining_secs(&job_items, concurrency).filter(|_| !complete);
    let retry = retry_ids(&job_items);
    let class = class_list("kit-pwg", &extra_class);
    let money = |v: f64| match format_money {
        Some(f) => f.call(v),
        None => format!("${v:.2}"),
    };
    let cost_text = money(cost);
    let remaining_text = remaining.map(|s| format!("~{:.0}s left", s));
    let counts_line = format!("{} of {total} done", c.done);
    let progress = progress_tenths(&job_items);
    let announce = format!("{progress} of 10 tenths complete");
    let detail_index = if dismissed() {
        None
    } else {
        picked().or(Some(focused()))
    }
    .filter(|i| *i < total);
    let detail = detail_index.and_then(|i| job_items.get(i).cloned());
    let width_hint = format!("{} cells", total);
    let icon = heading_icon.clone();

    if total == 0 {
        return rsx! {
            document::Link { rel: "stylesheet", href: KIT_CSS }
            document::Link { rel: "stylesheet", href: CSS }
            section { class: "{class}", "aria-label": "{heading_text}", h2 { class: "kit-pwg__title", "{heading_text}" } }
        };
    }

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{class}", "aria-labelledby": "{base}-title",
            header { class: "kit-pwg__head",
                if let Some(i) = icon { span { class: "kit-pwg__icon", "aria-hidden": "true", "{i}" } }
                h2 { id: "{base}-title", class: "kit-pwg__title", "{heading_text}" }
                if !job_name.is_empty() { p { class: "kit-pwg__job", "{job_name}" } }
            }
            dl { class: "kit-pwg__stats",
                div { dt { "Done" } dd { "{c.done}" } }
                div { dt { "In progress" } dd { "{c.in_progress}" } }
                div { dt { "Retrying" } dd { "{c.retrying}" } }
                if c.failed > 0 { div { dt { "Failed" } dd { "{c.failed}" } } }
                div { dt { "Cost" } dd { "{cost_text}" } }
                if let Some(r) = remaining_text.clone() { div { dt { "Remaining" } dd { "{r}" } } }
            }
            div { class: "kit-pwg__grid", role: "grid", "aria-label": "{heading_text} cells, {width_hint}",
                for (i, item) in job_items.iter().enumerate() {
                    {
                        let dim = filter().is_some_and(|f| !matches_filter(item, Some(f)));
                        let is_pick = picked() == Some(i);
                        let label = format!("{}, {}", item.item_label, item.item_state.word());
                        let dom_id = format!("{base}-cell-{i}");
                        let tab = if focused() == i { "0" } else { "-1" };
                        let delay = format!("--pwg-wave: {}ms;", (i % 40) * 12);
                        let state_attr = format!("{:?}", item.item_state);
                        let key_base = base.clone();
                        let item_click = item.clone();
                        let item_enter = item.clone();
                        rsx! {
                            button {
                                key: "{item.item_id}", id: "{dom_id}", r#type: "button", role: "gridcell",
                                class: "kit-pwg__cell", style: "{delay}", "data-state": "{state_attr}",
                                "data-dim": dim.then_some("true"), "aria-pressed": "{is_pick}", "aria-label": "{label}",
                                tabindex: "{tab}",
                                onfocus: move |_| {
                                    focused.set(i);
                                    dismissed.set(false);
                                },
                                onmouseenter: move |_| {
                                    focused.set(i);
                                    dismissed.set(false);
                                },
                                onclick: move |_| {
                                    let next = if picked() == Some(i) { None } else { Some(i) };
                                    picked.set(next);
                                    focused.set(i);
                                    dismissed.set(false);
                                    if let Some(h) = on_pick {
                                        h.call(next.map(|_| item_click.clone()));
                                    }
                                },
                                onkeydown: move |event| {
                                    let last = total.saturating_sub(1);
                                    let next = match event.key() {
                                        Key::ArrowRight => (i + 1).min(last),
                                        Key::ArrowLeft => i.saturating_sub(1),
                                        Key::ArrowDown => (i + 8).min(last),
                                        Key::ArrowUp => i.saturating_sub(8),
                                        Key::Home => 0,
                                        Key::End => last,
                                        Key::Enter => {
                                            event.prevent_default();
                                            let next = if picked() == Some(i) { None } else { Some(i) };
                                            picked.set(next);
                                            dismissed.set(false);
                                            if let Some(h) = on_pick {
                                                h.call(next.map(|_| item_enter.clone()));
                                            }
                                            return;
                                        }
                                        Key::Escape => {
                                            picked.set(None);
                                            dismissed.set(true);
                                            return;
                                        }
                                        _ => return,
                                    };
                                    event.prevent_default();
                                    focused.set(next);
                                    dismissed.set(false);
                                    document::eval(&format!("document.getElementById('{key_base}-cell-{next}')?.focus();"));
                                },
                                span { class: "kit-pwg__dot", "aria-hidden": "true" }
                            }
                        }
                    }
                }
            }
            if let Some(item) = detail {
                {
                    let duration = item_duration(&item).map(|d| format!("{d:.0}s")).unwrap_or_else(|| "–".to_string());
                    let item_cost_text = item.item_cost.map(money).unwrap_or_else(|| "–".to_string());
                    let worker = item.worker_name.clone().unwrap_or_else(|| "–".to_string());
                    let attempts = (item.attempt_count >= 2).then(|| format!("Attempts: {}", item.attempt_count));
                    let outcome = item.error_text.clone().or(item.output_value.clone()).unwrap_or_default();
                    let state = item.item_state.word();
                    rsx! {
                        aside { class: "kit-pwg__details", "aria-label": "Details", "aria-live": "polite",
                            h3 { class: "kit-pwg__item", "{item.item_label}" }
                            p { class: "kit-pwg__line", "{state} · worker {worker}" }
                            p { class: "kit-pwg__line", "{duration} · {item_cost_text}" }
                            if let Some(a) = attempts { p { class: "kit-pwg__line", "{a}" } }
                            if !outcome.is_empty() { pre { class: "kit-pwg__outcome", "{outcome}" } }
                        }
                    }
                }
            }
            footer { class: "kit-pwg__foot",
                div { class: "kit-pwg__filters", role: "group", "aria-label": "Filter by state",
                    for f in [StateFilter::Done, StateFilter::InProgress, StateFilter::Retrying, StateFilter::Failed] {
                        {
                            let active = filter() == Some(f);
                            let label = filter_key(f);
                            rsx! {
                                button { key: "{label}", r#type: "button", class: "kit-pwg__filter", "aria-pressed": "{active}",
                                    onclick: move |_| filter.set(if active { None } else { Some(f) }),
                                    "{label}"
                                }
                            }
                        }
                    }
                }
                p { class: "kit-pwg__count", role: "status", "aria-live": "polite", "{counts_line}" }
                if !retry.is_empty() {
                    button {
                        r#type: "button", class: "kit-pwg__retry",
                        onclick: move |_| {
                            if let Some(h) = on_retry {
                                h.call(retry.clone());
                            }
                        },
                        "Retry failed"
                    }
                }
            }
            p { class: "kit-pwg__live", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
