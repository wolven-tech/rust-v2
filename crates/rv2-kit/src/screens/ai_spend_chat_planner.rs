//! A conversation about what AI usage costs. Answers carry metric cards, a split chart (spend by
//! provider, stacked by model), and a trend or a plan. A plan's levers move the projected savings and
//! a six-month forecast. A person can select part of a chart and ask about only that part.
//!
//! Design choice: answers are `GroundedReply` (caption and sources, with follow-up questions), and
//! figures are `MetricTile`. Trends and forecasts are `SeriesTrendChart`.
//! Design choice: the split chart is local SVG. `SegmentedBarExplorer` brings its own pickers, which
//! the spec does not describe.
//! Design choice: the sidebar is not rendered; it belongs to the shared app frame. `chat_list` is
//! therefore not a prop here.
//! Design choice: plan totals are `NumberTicker`s, which roll digit by digit. Assistive technology
//! reads the settled text from a visually hidden copy, so the rolling figure is not announced.
//! Design choice: a lever announces its new saving when the pointer or focus leaves it, not on each step.
//! Browser checks: arrow keys on a range lever move by 5 percent of its maximum; a switch turns off
//! and its saving drops to zero; Enter on a split bar selects it and a second Enter asks.

use dioxus::prelude::*;
use rv2_affect::motion::number_ticker::NumberTicker;
use rv2_affect::skeleton::Skeleton;

use crate::components::grounded_reply::{GroundedReply, SourceEntry};
use crate::components::metric_tile::{MetricTile, MetricValue};
use crate::components::prompt_input_bar::{Model, PromptInputBar, Submission};
use crate::components::series_trend_chart::{
    ChartVariant, SeriesTone, SeriesTrendChart, TrendSeries, group_digits,
};
use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/ai-spend-chat-planner.css");

/// Months in a plan's forecast.
pub const FORECAST_MONTHS: usize = 6;

#[derive(Clone, Debug, PartialEq)]
pub struct Metric {
    pub metric_label: String,
    pub value: f64,
    pub change_percent: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SplitPart {
    pub entry_name: String,
    pub value_dollars: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SplitGroup {
    pub entry_name: String,
    pub parts: Vec<SplitPart>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeverKind {
    Range,
    Switch,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lever {
    pub id: String,
    pub entry_label: String,
    pub detail_text: String,
    pub lever_kind: LeverKind,
    pub monthly_base_dollars: f64,
    pub max_saving_share: f64,
    pub max_percent: f64,
    pub entry_value: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Chart {
    Split {
        title_text: String,
        previous_total: Option<f64>,
        groups: Vec<SplitGroup>,
    },
    Trend {
        title_text: String,
        x_labels: Vec<String>,
        series: Vec<TrendSeries>,
    },
    Plan {
        current_monthly: f64,
        months: usize,
        growth_rate: f64,
        lever_list: Vec<Lever>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    pub id: String,
    pub metric_cards: Vec<Metric>,
    pub chart_block: Option<Chart>,
    pub caption_text: String,
    pub source_list: Vec<SourceEntry>,
    pub follow_up_list: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Turn {
    Question { id: String, text: String },
    Answer(Answer),
}

/// Indices a question concerns: groups for a split chart, points for a trend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AskRequest {
    pub question: String,
    pub focus_range: Option<FocusRange>,
    pub plan_snapshot: Option<String>,
}

/// Dollars saved per month by one lever: base spend, times the share saved at full engagement, times
/// how far the lever is engaged (0 to 1). A range is engaged by value over its maximum; a switch by
/// being on.
pub fn lever_saving(lever: &Lever, value: f64) -> f64 {
    let intensity = match lever.lever_kind {
        LeverKind::Range if lever.max_percent > 0.0 => value / lever.max_percent,
        LeverKind::Range => 0.0,
        LeverKind::Switch => value,
    };
    lever.monthly_base_dollars * lever.max_saving_share * intensity.clamp(0.0, 1.0)
}

/// Total monthly saving across all levers at their current values.
pub fn total_savings(levers: &[Lever], values: &[f64]) -> f64 {
    levers
        .iter()
        .zip(values)
        .map(|(lever, value)| lever_saving(lever, *value))
        .sum()
}

/// The baseline spend for months 1 to `months`, growing by `growth` per month from `current`.
pub fn forecast(current: f64, growth: f64, months: usize) -> Vec<f64> {
    (1..=months)
        .map(|k| current * (1.0 + growth).powi(k as i32))
        .collect()
}

/// The planned path: the baseline scaled down by the share of current spend the plan saves.
pub fn planned(baseline: &[f64], current: f64, savings: f64) -> Vec<f64> {
    let ratio = if current > 0.0 {
        1.0 - savings / current
    } else {
        1.0
    };
    baseline.iter().map(|value| value * ratio).collect()
}

/// A split bar's total is the sum of its segments.
pub fn group_total(group: &SplitGroup) -> f64 {
    group.parts.iter().map(|p| p.value_dollars).sum()
}

/// Step for a range lever's arrow keys: 5 percent of its maximum.
pub fn range_step(max_percent: f64) -> f64 {
    max_percent * 0.05
}

/// Sets the live region to the total saving of the levers at `values`.
fn announce_saving(mut live: Signal<String>, levers: &[Lever], values: &[f64]) {
    let total = total_savings(levers, values);
    live.set(format!(
        "Monthly saving {} dollars.",
        group_digits(total.round())
    ));
}

/// "Batch jobs: 120 dollars a month saved".
pub fn saving_words(label: &str, saving: f64) -> String {
    format!(
        "{label}: {} dollars a month saved",
        group_digits(saving.round())
    )
}

/// Text sent to the assistant when the plan is turned into rules, carrying the plan's state.
pub fn plan_snapshot(levers: &[Lever], values: &[f64], current: f64) -> String {
    let savings = total_savings(levers, values);
    let parts: Vec<String> = levers
        .iter()
        .zip(values)
        .map(|(lever, value)| match lever.lever_kind {
            LeverKind::Range => format!(
                "{} at {:.0}%",
                lever.entry_label,
                value / lever.max_percent.max(1.0) * 100.0
            ),
            LeverKind::Switch => format!(
                "{} {}",
                lever.entry_label,
                if *value > 0.0 { "on" } else { "off" }
            ),
        })
        .collect();
    format!(
        "{}; saves {} dollars a month; new monthly spend {} dollars.",
        parts.join(", "),
        group_digits(savings.round()),
        group_digits((current - savings).round())
    )
}

/// The planner's conversation.
#[component]
pub fn AiSpendChatPlanner(
    #[props(default)] turn_list: Vec<Turn>,
    #[props(default)] is_working: bool,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] starter_questions: Vec<String>,
    on_ask: EventHandler<AskRequest>,
) -> Element {
    let id = use_hook(|| format!("kit-spend-{}", next_id()));
    let models = vec![Model {
        model_id: "standard".to_string(),
        model_name: "Standard".to_string(),
        model_mark: None,
        model_note: None,
    }];

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-spend", "aria-busy": is_loading.then_some("true"),
            div { class: "kit-spend__thread", role: "log", "aria-live": "polite",
                if is_loading {
                    div { class: "kit-spend__skeleton",
                        Skeleton { width: "65%", height: "40px" }
                        div { class: "kit-spend__cards",
                            Skeleton { width: "100%", height: "96px" }
                            Skeleton { width: "100%", height: "96px" }
                        }
                        Skeleton { width: "100%", height: "180px" }
                    }
                } else if is_empty {
                    div { class: "kit-spend__empty",
                        p { class: "kit-spend__note", "Ask what AI usage costs, by provider, model or week." }
                        div { class: "kit-spend__starters",
                            for question in starter_questions.iter() {
                                {
                                    let key = question.clone();
                                    let text = question.clone();
                                    rsx! {
                                        button {
                                            key: "{key}",
                                            r#type: "button",
                                            class: "kit-spend__starter",
                                            onclick: move |_| on_ask.call(AskRequest { question: text.clone(), focus_range: None, plan_snapshot: None }),
                                            "{key}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    for turn in turn_list.iter() {
                        {
                            match turn {
                                Turn::Question { id: key, text } => rsx! {
                                    div { key: "{key}", class: "kit-spend__question", p { "{text}" } }
                                },
                                Turn::Answer(answer) => rsx! {
                                    AnswerView { answer: answer.clone(), on_ask }
                                },
                            }
                        }
                    }
                }
                if is_working {
                    p { class: "kit-spend__working", role: "status",
                        span { class: "kit-spend__mark", "aria-hidden": "true" }
                        "Reading your usage…"
                    }
                }
            }
            div { class: "kit-spend__composer",
                PromptInputBar {
                    root_class: "kit-spend__input",
                    empty_hint: "Ask about your AI spend",
                    model_options: models,
                    on_submit: move |submission: Submission| {
                        on_ask.call(AskRequest { question: submission.text, focus_range: None, plan_snapshot: None });
                    },
                }
            }
            span { class: "kit-sr", id: "{id}-live" }
        }
    }
}

#[component]
fn AnswerView(answer: Answer, on_ask: EventHandler<AskRequest>) -> Element {
    let caption = answer.caption_text.clone();
    let follow_ups = answer.follow_up_list.clone();
    rsx! {
        article { class: "kit-spend__answer", "aria-label": "Answer",
            if !answer.metric_cards.is_empty() {
                div { class: "kit-spend__cards",
                    for metric in answer.metric_cards.iter() {
                        {
                            let label = metric.metric_label.clone();
                            let value = metric.value;
                            let change = metric.change_percent;
                            rsx! {
                                MetricTile {
                                    metric_label: label,
                                    metric_value: MetricValue::Number(value),
                                    change_percent: change,
                                }
                            }
                        }
                    }
                }
            }
            {
                match answer.chart_block.clone() {
                    Some(Chart::Split { title_text, previous_total, groups }) => rsx! {
                        SplitChart { title_text, previous_total, groups, on_ask }
                    },
                    Some(Chart::Trend { title_text, x_labels, series }) => rsx! {
                        TrendChart { title_text, x_labels, series, on_ask }
                    },
                    Some(Chart::Plan { current_monthly, months, growth_rate, lever_list }) => rsx! {
                        PlanCard { current_monthly, months, growth_rate, lever_list, on_ask }
                    },
                    None => rsx! {},
                }
            }
            GroundedReply {
                answer_text: caption,
                source_entries: answer.source_list.clone(),
                next_questions: follow_ups,
                reveal_animated: false,
                raised_surface: false,
                on_next_question_chosen: move |question: String| {
                    on_ask.call(AskRequest { question, focus_range: None, plan_snapshot: None });
                },
            }
        }
    }
}

#[component]
fn SplitChart(
    title_text: String,
    previous_total: Option<f64>,
    groups: Vec<SplitGroup>,
    on_ask: EventHandler<AskRequest>,
) -> Element {
    let id = use_hook(|| format!("kit-split-{}", next_id()));
    let mut selected = use_signal(|| None::<usize>);
    let totals: Vec<f64> = groups.iter().map(group_total).collect();
    let max = totals.iter().cloned().fold(0.0_f64, f64::max).max(1.0);
    let grand: f64 = totals.iter().sum();
    let title_id = format!("{id}-title");
    let previous_text =
        previous_total.map(|p| format!("Previous period: {} dollars", group_digits(p)));
    let grand_text = format!("Total {} dollars", group_digits(grand));
    let selected_index = selected();
    let ask_name = selected_index
        .and_then(|i| groups.get(i))
        .map(|g| g.entry_name.clone());
    rsx! {
        figure { class: "kit-spend__chart", "aria-labelledby": "{title_id}",
            figcaption { id: "{title_id}", class: "kit-spend__chart-title", "{title_text}" }
            div { class: "kit-spend__bars-wrap",
            svg { class: "kit-spend__bars", view_box: "0 0 640 200", role: "group", "aria-label": "Spend by provider and model",
                for (index, group) in groups.iter().enumerate() {
                    {
                        let total = totals[index];
                        let x = 24.0 + index as f64 * (592.0 / groups.len().max(1) as f64);
                        let width = (592.0 / groups.len().max(1) as f64) * 0.6;
                        let mut y = 180.0;
                        let is_selected = selected() == Some(index);
                        let label = format!("{}: {} dollars", group.entry_name, group_digits(total.round()));
                        let ask_name = group.entry_name.clone();
                        let segments: Vec<(f64, f64, usize)> = group.parts.iter().enumerate().map(|(part_index, part)| {
                            let h = part.value_dollars / max * 160.0;
                            y -= h;
                            (y, h, part_index)
                        }).collect();
                        rsx! {
                            g { key: "{index}", class: "kit-spend__bar", role: "button", tabindex: "0",
                                "aria-label": "{label}",
                                "aria-pressed": is_selected.then_some("true"),
                                "data-selected": is_selected.then_some("true"),
                                onclick: move |_| selected.set(Some(index)),
                                onkeydown: move |event: KeyboardEvent| {
                                    if event.key() == Key::Enter {
                                        event.prevent_default();
                                        if selected() == Some(index) {
                                            on_ask.call(AskRequest {
                                                question: format!("What is driving the spend on {ask_name}?"),
                                                focus_range: Some(FocusRange { start: index, end: index + 1 }),
                                                plan_snapshot: None,
                                            });
                                        } else {
                                            selected.set(Some(index));
                                        }
                                    }
                                },
                                for (sy, sh, part_index) in segments {
                                    rect { class: "kit-spend__segment", "data-part": "{part_index}",
                                        x: "{x}", y: "{sy}", width: "{width}", height: "{sh}" }
                                }
                                text { x: "{x}", y: "196", class: "kit-spend__axis", "{group.entry_name}" }
                            }
                        }
                    }
                }
            }
            }
            table { class: "kit-spend__table",
                thead { tr { th { "Provider" } th { "Model" } th { "Dollars" } } }
                tbody {
                    for (index, group) in groups.iter().enumerate() {
                        {
                            let total = totals[index];
                            let group_name = group.entry_name.clone();
                            let total_text = group_digits(total);
                            let rows: Vec<(String, String)> = group.parts.iter().map(|p| (p.entry_name.clone(), group_digits(p.value_dollars))).collect();
                            rsx! {
                                for (model, amount) in rows {
                                    tr { key: "{group_name}-{model}",
                                        td { "{group_name}" }
                                        td { "{model}" }
                                        td { class: "kit-spend__num", "{amount}" }
                                    }
                                }
                                tr { class: "kit-spend__total",
                                    td { "{group_name}" }
                                    td { "Total" }
                                    td { class: "kit-spend__num", "{total_text}" }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "kit-spend__chart-foot", "{grand_text}" }
            if let Some(text) = previous_text {
                p { class: "kit-spend__chart-foot", "{text}" }
            }
            if let (Some(index), Some(name)) = (selected_index, ask_name) {
                button {
                    r#type: "button",
                    class: "kit-spend__ask",
                    onclick: move |_| on_ask.call(AskRequest {
                        question: format!("What is driving the spend on {name}?"),
                        focus_range: Some(FocusRange { start: index, end: index + 1 }),
                        plan_snapshot: None,
                    }),
                    "Ask about this bar"
                }
            }
        }
    }
}

#[component]
fn TrendChart(
    title_text: String,
    x_labels: Vec<String>,
    series: Vec<TrendSeries>,
    on_ask: EventHandler<AskRequest>,
) -> Element {
    let mut active = use_signal(|| None::<usize>);
    let shown = active();
    let key_labels = x_labels.clone();
    rsx! {
        div { class: "kit-spend__trend", onkeydown: move |event: KeyboardEvent| {
            if event.key() == Key::Enter {
                if let Some(index) = active() {
                    on_ask.call(AskRequest {
                        question: format!("What changed around {}?", key_labels.get(index).cloned().unwrap_or_default()),
                        focus_range: Some(FocusRange { start: index, end: index }),
                        plan_snapshot: None,
                    });
                }
            }
        },
            SeriesTrendChart {
                x_labels: x_labels.clone(),
                series,
                variant: ChartVariant::Line,
                label: title_text,
                active: Some(shown),
                on_active_change: move |next: Option<usize>| active.set(next),
            }
        }
    }
}

#[component]
fn PlanCard(
    current_monthly: f64,
    months: usize,
    growth_rate: f64,
    lever_list: Vec<Lever>,
    on_ask: EventHandler<AskRequest>,
) -> Element {
    let id = use_hook(|| format!("kit-plan-{}", next_id()));
    let seed: Vec<f64> = lever_list.iter().map(|l| l.entry_value).collect();
    let mut values = use_signal(move || seed);
    let live = use_signal(String::new);
    let current = values();
    let savings = total_savings(&lever_list, &current);
    let new_spend = current_monthly - savings;
    let baseline = forecast(current_monthly, growth_rate, months);
    let path = planned(&baseline, current_monthly, savings);
    let x_labels: Vec<String> = (1..=months).map(|m| format!("M{m}")).collect();
    let series = vec![
        TrendSeries {
            name: "Baseline".to_string(),
            values: baseline.iter().map(|v| Some(*v)).collect(),
            tone: SeriesTone::Muted,
            emphasis: false,
            is_earlier: false,
        },
        TrendSeries {
            name: "Planned".to_string(),
            values: path.iter().map(|v| Some(*v)).collect(),
            tone: SeriesTone::Ink,
            emphasis: true,
            is_earlier: false,
        },
    ];
    let snapshot = plan_snapshot(&lever_list, &current, current_monthly);
    let savings_text = format!("{} dollars a month", group_digits(savings.round()));
    let new_text = format!("{} dollars a month", group_digits(new_spend.round()));
    let live_id = format!("{id}-live");
    rsx! {
        section { class: "kit-spend__plan", "aria-label": "Plan",
            ul { class: "kit-spend__levers",
                for (index, lever) in lever_list.iter().enumerate() {
                    {
                        let key = lever.id.clone();
                        let label = lever.entry_label.clone();
                        let detail = lever.detail_text.clone();
                        let saving = lever_saving(lever, current[index]);
                        let words = saving_words(&label, saving);
                        let slider_id = format!("{id}-lever-{index}");
                        let snapshot_levers = lever_list.clone();
                        let settled_levers = lever_list.clone();
                        let kind = lever.lever_kind;
                        let max = lever.max_percent;
                        let step_text = range_step(max).to_string();
                        let value = current[index];
                        let value_text = format!("{value:.0}%");
                        rsx! {
                            li { key: "{key}", class: "kit-spend__lever",
                                if kind == LeverKind::Range {
                                    label { class: "kit-spend__lever-head", r#for: "{slider_id}",
                                        span { class: "kit-spend__lever-name", "{label}" }
                                        span { class: "kit-spend__lever-value", "{value_text}" }
                                    }
                                    input {
                                        id: "{slider_id}",
                                        r#type: "range",
                                        min: "0",
                                        max: "{max}",
                                        step: "{step_text}",
                                        value: "{value}",
                                        "aria-valuetext": "{words}",
                                        oninput: move |event: FormEvent| {
                                            let parsed = event.value().parse::<f64>().unwrap_or(0.0);
                                            values.with_mut(|v| v[index] = parsed);
                                        },
                                        onpointerup: move |_| {
                                            let now = values.peek().clone();
                                            announce_saving(live, &settled_levers, &now);
                                        },
                                        onblur: move |_| {
                                            let now = values.peek().clone();
                                            announce_saving(live, &snapshot_levers, &now);
                                        },
                                    }
                                } else {
                                    label { class: "kit-spend__lever-head", r#for: "{slider_id}",
                                        span { class: "kit-spend__lever-name", "{label}" }
                                        span { class: "kit-spend__lever-value", if value > 0.0 { "On" } else { "Off" } }
                                    }
                                    input {
                                        id: "{slider_id}",
                                        class: "kit-spend__switch",
                                        r#type: "checkbox",
                                        role: "switch",
                                        "aria-label": "{label}",
                                        checked: value > 0.0,
                                        onchange: move |event: FormEvent| {
                                            let next = if event.checked() { 1.0 } else { 0.0 };
                                            values.with_mut(|v| v[index] = next);
                                            let now = values.peek().clone();
                                            announce_saving(live, &snapshot_levers, &now);
                                        },
                                    }
                                }
                                p { class: "kit-spend__lever-detail", "{detail}" }
                                p { class: "kit-spend__lever-saving", "{words}" }
                            }
                        }
                    }
                }
            }
            div { class: "kit-spend__summary",
                div { class: "kit-spend__figure",
                    span { class: "kit-spend__figure-label", "Monthly savings" }
                    span { class: "kit-spend__figure-value", "aria-hidden": "true",
                        NumberTicker { value: savings.round(), suffix: " dollars a month" }
                    }
                    span { class: "kit-sr", "{savings_text}" }
                }
                div { class: "kit-spend__figure",
                    span { class: "kit-spend__figure-label", "New monthly spend" }
                    span { class: "kit-spend__figure-value", "aria-hidden": "true",
                        NumberTicker { value: new_spend.round(), suffix: " dollars a month" }
                    }
                    span { class: "kit-sr", "{new_text}" }
                }
            }
            SeriesTrendChart {
                x_labels,
                series,
                variant: ChartVariant::Line,
                label: "Six-month forecast against baseline",
                show_tooltip: true,
            }
            p { id: "{live_id}", class: "kit-sr", role: "status", "aria-live": "polite", "{live()}" }
            button {
                r#type: "button",
                class: "kit-spend__rules",
                onclick: move |_| on_ask.call(AskRequest {
                    question: format!("Turn this plan into rules: {snapshot}"),
                    focus_range: None,
                    plan_snapshot: Some(snapshot.clone()),
                }),
                "Turn into rules"
            }
        }
    }
}
