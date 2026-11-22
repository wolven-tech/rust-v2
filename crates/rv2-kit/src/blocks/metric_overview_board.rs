//! Metric overview board: the headline numbers of a dashboard in one glance. Three trend cards
//! (total, change against the previous period, sparkline), an overview chart with a segment
//! filter, and a focus card that follows the day under the pointer or the keyboard.
//!
//! Design choice: each stat card is a `MetricTrendCard`; the overview is a `SeriesTrendChart`.
//! Design choice: the segment filter narrows only the overview. Its total is the sum of the
//! visible series on the chosen day, and the delta is the change from the first to the last day.
//! Design choice: `lower_is_better` cards say good or bad in words in their footer, since the
//! card's own chip only knows the sign.
//! Design choice: the period picker is a `ListPicker`, a button that opens a listbox. Its
//! stylesheet is linked by the picker itself, so a host that uses it alone is themed too.
//! Design choice: when the totals change, each one counts from its old value to the new one over
//! `ROLL_STEPS` frames; under reduced motion it changes at once.
//! Browser checks: hover marks a day and updates the focus card, period change rolls the totals
//! and redraws, segment changes only the overview, arrow keys step through days, a flat series
//! renders zeros, phone focus card below the overview.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonSize, ButtonVariant};

use crate::components::metric_trend_card::{MetricSeries, MetricTrendCard};
use crate::components::series_trend_chart::{
    ChartVariant, SeriesTone, SeriesTrendChart, TrendSeries, compact_number,
};
use crate::internal::{KIT_CSS, class_list, prefers_motion_script, sleep};

const CSS: Asset = asset!("/assets/metric-overview-board.css");

/// Segment id that shows every series.
pub const ALL_SEGMENTS: &str = "all";
/// Frames a changed total takes to count to its new value.
pub const ROLL_STEPS: usize = 12;
const ROLL_FRAME_MS: u32 = 30;
/// Matches the `--kit-fast` token the plot fades with, so the swap lands at zero opacity.
const SWAP_FADE_MS: u32 = 140;

#[derive(Clone, Debug, PartialEq)]
pub struct StatMetric {
    pub id: String,
    pub label: String,
    pub total: f64,
    pub current_series: Vec<f64>,
    pub previous_series: Vec<f64>,
    pub lower_is_better: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverviewSeries {
    pub label: String,
    pub values: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PickOption {
    pub id: String,
    pub label: String,
}

/// Percent change from `previous` to `current`; zero when there is no previous value.
pub fn percent_change(current: f64, previous: f64) -> f64 {
    if previous == 0.0 || !previous.is_finite() || !current.is_finite() {
        0.0
    } else {
        (current - previous) / previous.abs() * 100.0
    }
}

/// `good`, `bad`, or `flat` for a change.
pub fn good_or_bad(change: f64, lower_is_better: bool) -> &'static str {
    if change == 0.0 {
        "flat"
    } else if (change > 0.0) != lower_is_better {
        "good"
    } else {
        "bad"
    }
}

/// Series shown for a segment: all of them, or the one whose label matches.
pub fn visible_series<'a>(series: &'a [OverviewSeries], segment: &str) -> Vec<&'a OverviewSeries> {
    series
        .iter()
        .filter(|s| segment == ALL_SEGMENTS || s.label == segment)
        .collect()
}

/// Sum of the visible series at each index; missing points count as zero.
pub fn overview_totals(series: &[&OverviewSeries]) -> Vec<f64> {
    let len = series.iter().map(|s| s.values.len()).max().unwrap_or(0);
    (0..len)
        .map(|i| {
            series
                .iter()
                .map(|s| s.values.get(i).copied().unwrap_or(0.0))
                .sum()
        })
        .collect()
}

/// The value `step` of `steps` frames into a count from `from` to `to`, with ease-out.
pub fn roll_at(from: f64, to: f64, step: usize, steps: usize) -> f64 {
    let t = (step as f64 / steps.max(1) as f64).clamp(0.0, 1.0);
    from + (to - from) * (1.0 - (1.0 - t).powi(3))
}

/// Whether the viewer accepts motion; a failed probe counts as no motion.
pub(crate) async fn motion_allowed() -> bool {
    document::eval(&format!("return {};", prefers_motion_script()))
        .await
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Sets `shown` to `next` once the old content has faded out. `fading` holds the content at zero
/// opacity during the fade. A newer call supersedes an unfinished one; reduced motion swaps at once.
pub(crate) fn fade_swap<T: 'static>(
    mut shown: Signal<T>,
    mut fading: Signal<bool>,
    mut run: Signal<u32>,
    next: T,
) {
    let this = *run.peek() + 1;
    run.set(this);
    spawn(async move {
        if !motion_allowed().await {
            shown.set(next);
            return;
        }
        fading.set(true);
        sleep(SWAP_FADE_MS).await;
        if *run.peek() != this {
            return;
        }
        shown.set(next);
        fading.set(false);
    });
}

/// A single-choice listbox: a button that opens an `ul role="listbox"`. Arrow keys move the
/// active option, Enter picks, Escape closes and returns focus to the trigger.
#[component]
pub fn ListPicker(
    options: Vec<PickOption>,
    selected: Option<String>,
    #[props(into)] label: String,
    on_change: EventHandler<String>,
    #[props(default)] disabled: bool,
) -> Element {
    let uid = use_signal(|| format!("kit-lp-{}", crate::internal::next_id()));
    let mut open = use_signal(|| false);
    let mut active = use_signal(|| 0_usize);
    let shown = selected
        .as_ref()
        .and_then(|s| options.iter().find(|o| o.id == *s))
        .or_else(|| options.first())
        .map(|o| o.label.clone())
        .unwrap_or_default();
    let count = options.len();
    let pick_ids: Vec<String> = options.iter().map(|o| o.id.clone()).collect();
    let selected_index = selected
        .as_ref()
        .and_then(|s| options.iter().position(|o| o.id == *s))
        .unwrap_or(0);

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-pick", "data-disabled": disabled.then_some("true"),
            button {
                id: "{uid()}-trigger",
                r#type: "button",
                class: "kit-pick__trigger",
                "aria-label": "{label}",
                "aria-haspopup": "listbox",
                "aria-expanded": if open() { "true" } else { "false" },
                disabled: disabled,
                onclick: move |_| {
                    let next = !open();
                    open.set(next);
                    active.set(selected_index);
                    if next {
                        document::eval(&format!(
                            "requestAnimationFrame(() => document.getElementById('{}-list')?.focus());",
                            uid()
                        ));
                    }
                },
                "{shown}"
            }
            if open() {
                ul { id: "{uid()}-list", class: "kit-pick__list", role: "listbox", tabindex: "-1", "aria-label": "{label}",
                    "aria-activedescendant": "{uid()}-opt-{active()}",
                    onkeydown: move |e: KeyboardEvent| {
                        match e.key() {
                            Key::ArrowDown => { e.prevent_default(); active.set((active() + 1).min(count.saturating_sub(1))); }
                            Key::ArrowUp => { e.prevent_default(); active.set(active().saturating_sub(1)); }
                            Key::Home => { e.prevent_default(); active.set(0); }
                            Key::End => { e.prevent_default(); active.set(count.saturating_sub(1)); }
                            Key::Enter => {
                                e.prevent_default();
                                pick_option(open, on_change, pick_ids.get(active()).cloned(), uid());
                            }
                            Key::Escape => {
                                e.prevent_default();
                                open.set(false);
                                crate::internal::focus_after_frame(&format!("{}-trigger", uid()));
                            }
                            _ => {}
                        }
                    },
                    for (i, option) in options.iter().enumerate() {
                        li { key: "{option.id}", id: "{uid()}-opt-{i}", role: "option",
                            "aria-selected": if selected.as_deref() == Some(option.id.as_str()) { "true" } else { "false" },
                            "data-active": (active() == i).then_some("true"),
                            onclick: {
                                let id = option.id.clone();
                                move |_| pick_option(open, on_change, Some(id.clone()), uid())
                            },
                            "{option.label}"
                        }
                    }
                }
            }
        }
    }
}

/// Closes a listbox, reports the pick, and returns focus to its trigger.
fn pick_option(
    mut open: Signal<bool>,
    on_change: EventHandler<String>,
    id: Option<String>,
    uid: String,
) {
    open.set(false);
    if let Some(id) = id {
        on_change.call(id);
    }
    crate::internal::focus_after_frame(&format!("{uid}-trigger"));
}

#[component]
pub fn MetricOverviewBoard(
    stat_cards: Vec<StatMetric>,
    overview_series: Vec<OverviewSeries>,
    axis_labels: Vec<String>,
    #[props(default)] period_options: Vec<PickOption>,
    #[props(default)] selected_period: Option<String>,
    #[props(default)] on_period_change: Option<EventHandler<String>>,
    #[props(default)] segment_options: Vec<PickOption>,
    #[props(default = ALL_SEGMENTS.to_string(), into)] selected_segment: String,
    #[props(default)] on_segment_change: Option<EventHandler<String>>,
    #[props(default = "This period".to_string(), into)] current_value_label: String,
    #[props(default = "Previous period".to_string(), into)] previous_value_label: String,
    #[props(default = compact_number as fn(f64) -> String)] format_amount: fn(f64) -> String,
    #[props(default)] heading: Option<String>,
    #[props(default)] setup_action: Option<(String, EventHandler<()>)>,
    #[props(default)] footer_action: Option<(String, EventHandler<()>)>,
    #[props(default)] disabled: bool,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let mut hovered = use_signal(|| None::<usize>);
    let mut segment = use_signal(|| selected_segment.clone());
    let shown_segment = use_signal(|| selected_segment.clone());
    let segment_fading = use_signal(|| false);
    let segment_run = use_signal(|| 0_u32);
    let active_segment = shown_segment();
    let visible = visible_series(&overview_series, &active_segment);
    let totals = overview_totals(&visible);
    let last = totals.len().saturating_sub(1);
    let day = hovered().unwrap_or(last).min(last);
    let day_label = axis_labels.get(day).cloned().unwrap_or_default();
    let day_value = totals.get(day).copied().unwrap_or(0.0);
    let total_now = totals.last().copied().unwrap_or(0.0);
    let overview_change = percent_change(total_now, totals.first().copied().unwrap_or(0.0));
    let root_class = class_list("kit-mob", &extra_classes);

    let mut targets: Vec<f64> = stat_cards.iter().map(|c| c.total).collect();
    targets.push(total_now);
    let mut shown = use_signal(|| targets.clone());
    let mut roll_run = use_signal(|| 0_u32);
    use_effect(use_reactive((&targets,), move |(to,): (Vec<f64>,)| {
        let from = shown.peek().clone();
        if from == to || from.len() != to.len() {
            shown.set(to);
            return;
        }
        let run = *roll_run.peek() + 1;
        roll_run.set(run);
        spawn(async move {
            let mut shown = shown;
            if !motion_allowed().await {
                shown.set(to);
                return;
            }
            for step in 1..=ROLL_STEPS {
                if *roll_run.peek() != run {
                    return;
                }
                shown.set(
                    from.iter()
                        .zip(&to)
                        .map(|(a, b)| roll_at(*a, *b, step, ROLL_STEPS))
                        .collect(),
                );
                sleep(ROLL_FRAME_MS).await;
            }
            shown.set(to);
        });
    }));
    let shown_now = shown();

    let chart_series: Vec<TrendSeries> = visible
        .iter()
        .enumerate()
        .map(|(i, s)| TrendSeries {
            name: s.label.clone(),
            values: s.values.iter().map(|v| Some(*v)).collect(),
            tone: if i == 0 {
                SeriesTone::Ink
            } else {
                SeriesTone::Ok
            },
            emphasis: i == 0,
            is_earlier: false,
        })
        .collect();
    let focus_previous = visible
        .first()
        .and_then(|s| s.values.get(day).copied())
        .unwrap_or(0.0);
    let cards: Vec<(StatMetric, f64)> = stat_cards
        .iter()
        .map(|c| {
            let change = percent_change(
                c.current_series.last().copied().unwrap_or(c.total),
                c.previous_series.last().copied().unwrap_or(0.0),
            );
            (c.clone(), change)
        })
        .collect();
    let overview_word = if overview_change > 0.0 {
        "up"
    } else if overview_change < 0.0 {
        "down"
    } else {
        "flat"
    };
    let overview_delta = format!("{overview_word} {:.1}%", overview_change.abs());
    let overview_shown = shown_now
        .get(stat_cards.len())
        .copied()
        .unwrap_or(total_now);

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-label": "Metric overview", "data-disabled": disabled.then_some("true"),
            header { class: "kit-mob__head",
                if let Some(title) = heading.clone() {
                    h2 { class: "kit-mob__title", "{title}" }
                }
                if !period_options.is_empty() {
                    ListPicker {
                        options: period_options.clone(),
                        selected: selected_period.clone(),
                        label: "Period",
                        disabled: disabled,
                        on_change: move |p: String| {
                            if let Some(handler) = on_period_change { handler.call(p); }
                        },
                    }
                }
            }
            div { class: "kit-mob__stats", role: "group", "aria-label": "Headline trends",
                for (idx, (card, change)) in cards.iter().enumerate() {
                    {
                        let series = vec![
                            MetricSeries { series_name: current_value_label.clone(), series_tone: SeriesTone::Ink, series_values: card.current_series.clone() },
                            MetricSeries { series_name: previous_value_label.clone(), series_tone: SeriesTone::Muted, series_values: card.previous_series.clone() },
                        ];
                        let axis: Vec<String> = (0..card.current_series.len()).map(|i| axis_labels.get(i).cloned().unwrap_or_else(|| (i + 1).to_string())).collect();
                        let verdict = good_or_bad(*change, card.lower_is_better);
                        let footer = format!("{verdict} vs {previous_value_label}");
                        let headline = format_amount(shown_now.get(idx).copied().unwrap_or(card.total));
                        rsx! {
                            div { key: "{card.id}", class: "kit-mob__stat", "data-verdict": verdict,
                                MetricTrendCard {
                                    metric_name: card.label.clone(),
                                    headline_value: headline,
                                    change_value: *change,
                                    axis_labels: axis,
                                    data_series: series,
                                    format_value: format_amount,
                                    footer_text: footer,
                                }
                            }
                        }
                    }
                }
            }
            div { class: "kit-mob__overview",
                section { class: "kit-mob__chart", "aria-label": "Overview",
                    div { class: "kit-mob__chart-head",
                        p { class: "kit-mob__total",
                            span { class: "kit-mob__total-figure", "{format_amount(overview_shown)}" }
                            span { class: "kit-mob__delta", "{overview_delta}" }
                        }
                        if segment_options.len() > 1 {
                            ListPicker {
                                options: segment_options.clone(),
                                selected: Some(segment()),
                                label: "Segment",
                                disabled: disabled,
                                on_change: move |s: String| {
                                    segment.set(s.clone());
                                    fade_swap(shown_segment, segment_fading, segment_run, s.clone());
                                    if let Some(handler) = on_segment_change { handler.call(s); }
                                },
                            }
                        }
                    }
                    div { class: "kit-mob__plot", "data-fading": segment_fading().then_some("true"),
                        SeriesTrendChart {
                            x_labels: axis_labels.clone(),
                            series: chart_series,
                            variant: ChartVariant::Line,
                            on_active_change: move |index: Option<usize>| hovered.set(index),
                            label: "Overview".to_string(),
                            height: 220.0,
                        }
                    }
                }
                aside { class: "kit-mob__focus", "aria-live": "polite", "aria-label": "Focused day",
                    p { class: "kit-mob__focus-day", "{day_label}" }
                    dl { class: "kit-mob__focus-rows",
                        dt { "{current_value_label}" }
                        dd { "{format_amount(day_value)}" }
                        dt { "{previous_value_label}" }
                        dd { "{format_amount(focus_previous)}" }
                    }
                    if let Some((label, on_activate)) = setup_action.clone() {
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Sm,
                            label: label,
                            onclick: move |_| on_activate.call(()),
                        }
                    }
                }
            }
            if let Some((label, on_activate)) = footer_action.clone() {
                footer { class: "kit-mob__footer",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        label: label,
                        onclick: move |_| on_activate.call(()),
                    }
                }
            }
        }
    }
}
