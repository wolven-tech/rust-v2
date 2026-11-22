//! Agent spend overview: what the agents cost this month, against the budget,
//! where the money is heading at the current pace, and where it went by project
//! and by model. Items that deserve a look come last.
//!
//! Design choice: the cumulative chart is `SeriesTrendChart`. Actual spend is a
//! solid line; the forecast is an earlier-period series, drawn dashed from the
//! last actual day to month end; the budget is a flat series.
//! Design choice: the selected project's or model's daily bars use
//! `DailyUsageBars`, and the stat tiles use `MetricTile`.
//! Design choice: the ranked lists are this module's own rows. Each row needs a
//! change figure with its sign, which the shared composition card does not carry.
//! Design choice: at 640 px and below a picked project or model opens its panel as a
//! full-width sheet with a Back control.
//! Design choice: the hero total counts up over twelve frames when the daily costs
//! change; reduced motion sets it at once.
//! Browser checks: the forecast dash starts at the last actual day; the sentence
//! names the day the budget is passed; Up/Down move through a list; Enter opens
//! the panel; the chart tooltip shows the day's total; no horizontal scroll at
//! 320 px.

use dioxus::prelude::*;

use crate::components::daily_usage_bars::{DailyUsageBars, DayValue, Figure};
use crate::components::metric_tile::{MetricTile, MetricValue};
use crate::components::series_trend_chart::{
    ChartVariant, SeriesTone, SeriesTrendChart, TrendSeries,
};
use crate::internal::{KIT_CSS, focus_after_frame, sleep};

const CSS: Asset = asset!("/assets/agent-spend-overview.css");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    Project,
    Model,
}

impl EntryKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Model => "model",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttentionKind {
    Spike,
    Jump,
    Budget,
}

impl AttentionKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Spike => "Spike",
            Self::Jump => "Jump",
            Self::Budget => "Budget",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopTask {
    pub entry_title: String,
    pub run_count: u32,
    pub cost: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Breakdown {
    pub id: String,
    pub entry_name: String,
    pub amount: f64,
    /// Percent against last month; the sign is shown in words.
    pub change_vs_last_month: Option<f64>,
    pub daily_costs: Vec<f64>,
    pub top_tasks: Vec<TopTask>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttentionItem {
    pub id: String,
    pub item_kind: AttentionKind,
    pub text: String,
    pub detail_text: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UsageStats {
    pub task_count: Option<u32>,
    pub run_count: Option<u32>,
    pub token_count: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BudgetState {
    /// No budget set: no budget line and no budget sentence.
    None,
    Under,
    Over,
}

/// Where the month is heading at the pace of the days so far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Forecast {
    pub spent: f64,
    pub pace_per_day: f64,
    pub projected: f64,
    /// 1-based day of month on which the budget is passed, if it is.
    pub budget_day: Option<usize>,
    pub budget_state: BudgetState,
}

/// Formats an amount with the currency's symbol when known, else with its code.
pub fn format_money(amount: f64, code: &str) -> String {
    let symbol = match code {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        _ => "",
    };
    let sign = if amount < 0.0 { "-" } else { "" };
    let fixed = format!("{:.2}", amount.abs());
    let (whole, cents) = fixed.split_once('.').unwrap_or((fixed.as_str(), "00"));
    let grouped = group_thousands(whole);
    if symbol.is_empty() {
        format!("{sign}{grouped}.{cents} {code}")
    } else {
        format!("{sign}{symbol}{grouped}.{cents}")
    }
}

fn group_thousands(digits: &str) -> String {
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}

/// Running total by day.
pub fn cumulative(daily: &[f64]) -> Vec<f64> {
    daily
        .iter()
        .scan(0.0, |total, value| {
            *total += value;
            Some(*total)
        })
        .collect()
}

/// Share of `total` in percent; zero when there is no total.
pub fn share_pct(amount: f64, total: f64) -> f64 {
    if total > 0.0 {
        amount / total * 100.0
    } else {
        0.0
    }
}

/// Forecast for the month from the days elapsed so far.
pub fn forecast(daily: &[f64], days_in_month: usize, budget: Option<f64>) -> Forecast {
    let elapsed = daily.len();
    let spent: f64 = daily.iter().sum();
    let pace = if elapsed > 0 {
        spent / elapsed as f64
    } else {
        0.0
    };
    let remaining = days_in_month.saturating_sub(elapsed) as f64;
    let projected = spent + pace * remaining;
    let Some(limit) = budget else {
        return Forecast {
            spent,
            pace_per_day: pace,
            projected,
            budget_day: None,
            budget_state: BudgetState::None,
        };
    };
    if spent > limit {
        let day = cumulative(daily)
            .iter()
            .position(|total| *total > limit)
            .map(|index| index + 1);
        return Forecast {
            spent,
            pace_per_day: pace,
            projected,
            budget_day: day,
            budget_state: BudgetState::Over,
        };
    }
    let budget_day = if pace > 0.0 && projected > limit {
        let days_to_limit = ((limit - spent) / pace).ceil() as usize;
        let day = elapsed + days_to_limit;
        (day <= days_in_month).then_some(day)
    } else {
        None
    };
    Forecast {
        spent,
        pace_per_day: pace,
        projected,
        budget_day,
        budget_state: BudgetState::Under,
    }
}

/// Actual cumulative spend, one value per day of the month; empty after today.
pub fn actual_series(daily: &[f64], days_in_month: usize) -> Vec<Option<f64>> {
    let totals = cumulative(daily);
    (0..days_in_month)
        .map(|index| totals.get(index).copied())
        .collect()
}

/// The dashed forecast: starts at the last actual day and extends at the average pace.
pub fn forecast_series(daily: &[f64], days_in_month: usize) -> Vec<Option<f64>> {
    let elapsed = daily.len();
    if elapsed == 0 {
        return vec![None; days_in_month];
    }
    let totals = cumulative(daily);
    let last = totals[elapsed - 1];
    let pace = last / elapsed as f64;
    (0..days_in_month)
        .map(|index| {
            if index + 1 < elapsed {
                None
            } else {
                Some(last + pace * (index + 1 - elapsed) as f64)
            }
        })
        .collect()
}

/// Sentence stating where the month lands, and the day the budget is passed.
pub fn hero_sentence(forecast: &Forecast, budget: Option<f64>, code: &str) -> String {
    match (forecast.budget_state, budget) {
        (BudgetState::None, _) | (_, None) => format!(
            "On pace for {} by month end.",
            format_money(forecast.projected, code)
        ),
        (BudgetState::Over, Some(limit)) => match forecast.budget_day {
            Some(day) => format!(
                "Budget of {} passed on day {day}. On pace for {}.",
                format_money(limit, code),
                format_money(forecast.projected, code)
            ),
            None => format!("Budget of {} passed.", format_money(limit, code)),
        },
        (BudgetState::Under, Some(limit)) => match forecast.budget_day {
            Some(day) => format!(
                "At this pace the budget of {} is passed on day {day}.",
                format_money(limit, code)
            ),
            None => format!(
                "On pace for {}, under the {} budget.",
                format_money(forecast.projected, code),
                format_money(limit, code)
            ),
        },
    }
}

/// Change against last month with its direction in words.
pub fn change_words(change: f64) -> String {
    if change > 0.0 {
        format!("up {change:.0}%")
    } else if change < 0.0 {
        format!("down {:.0}%", -change)
    } else {
        "no change".to_string()
    }
}

/// Rows sorted by amount, largest first.
pub fn sorted_by_amount(rows: &[Breakdown]) -> Vec<Breakdown> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| b.amount.total_cmp(&a.amount));
    sorted
}

fn format_axis(value: f64) -> String {
    format!("{value:.0}")
}

/// Steps the hero total from `from` to `to` over twelve frames; reduced motion sets it at once.
async fn count_up(mut shown: Signal<f64>, from: f64, to: f64) {
    let reduce =
        document::eval("return window.matchMedia('(prefers-reduced-motion: reduce)').matches;")
            .await
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
    if reduce {
        shown.set(to);
        return;
    }
    for step in 1..=12 {
        sleep(40).await;
        shown.set(from + (to - from) * f64::from(step) / 12.0);
    }
    shown.set(to);
}

/// The symbol `format_money` puts before the amount for `code`.
fn currency_symbol(code: &str) -> String {
    format_money(0.0, code)
        .chars()
        .take_while(|c| !c.is_ascii_digit() && *c != '-')
        .collect()
}

fn format_count(value: u64) -> String {
    group_thousands(&value.to_string())
}

fn direction_of(change: Option<f64>) -> &'static str {
    match change {
        Some(c) if c > 0.0 => "up",
        Some(c) if c < 0.0 => "down",
        _ => "flat",
    }
}

/// The month's spend against the budget, its forecast, the breakdowns, and the
/// items to look at. `on_select` fires with the kind and id of a row;
/// `on_open_item` with the attention item.
#[component]
pub fn AgentSpendOverview(
    #[props(into)] month_label: String,
    #[props(into)] currency_code: String,
    #[props(default)] daily_costs: Vec<f64>,
    #[props(default = 30)] days_in_month: usize,
    #[props(default)] budget_amount: Option<f64>,
    #[props(default)] project_breakdown: Vec<Breakdown>,
    #[props(default)] model_breakdown: Vec<Breakdown>,
    #[props(default)] attention_items: Vec<AttentionItem>,
    #[props(default)] usage_stats: Option<UsageStats>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_select: Option<EventHandler<(EntryKind, String)>>,
    #[props(default)] on_open_item: Option<EventHandler<AttentionItem>>,
) -> Element {
    let mut selected = use_signal(|| None::<(EntryKind, String)>);
    let mut sheet_open = use_signal(|| false);
    let sheet_now = sheet_open();
    let total: f64 = daily_costs.iter().sum();
    let fc = forecast(&daily_costs, days_in_month, budget_amount);
    let sentence = hero_sentence(&fc, budget_amount, &currency_code);
    let shown_total = use_signal(|| total);
    let mut seen_total = use_signal(|| total);
    if *seen_total.peek() != total {
        seen_total.set(total);
        spawn(count_up(shown_total, *shown_total.peek(), total));
    }
    let total_text = format_money(shown_total(), &currency_code);
    let zero_text = format_money(0.0, &currency_code);
    let budget_fraction = budget_amount.map(|limit| (total / limit * 100.0).clamp(0.0, 100.0));
    let budget_text = budget_amount.map_or_else(String::new, |limit| {
        format!(
            "{:.0}% of {}",
            budget_fraction.unwrap_or(0.0),
            format_money(limit, &currency_code)
        )
    });
    let budget_width = format!("width: {:.1}%;", budget_fraction.unwrap_or(0.0));
    let budget_fraction_value = format!("{:.0}", budget_fraction.unwrap_or(0.0));
    let x_labels: Vec<String> = (1..=days_in_month).map(|d| d.to_string()).collect();
    let mut series = vec![
        TrendSeries {
            name: "Spent".to_string(),
            values: actual_series(&daily_costs, days_in_month),
            tone: SeriesTone::Ink,
            emphasis: true,
            is_earlier: false,
        },
        TrendSeries {
            name: "Forecast".to_string(),
            values: forecast_series(&daily_costs, days_in_month),
            tone: SeriesTone::Muted,
            emphasis: false,
            is_earlier: true,
        },
    ];
    if let Some(limit) = budget_amount {
        series.push(TrendSeries {
            name: "Budget".to_string(),
            values: vec![Some(limit); days_in_month],
            tone: SeriesTone::Warn,
            emphasis: false,
            is_earlier: false,
        });
    }
    let stats = usage_stats.unwrap_or_default();
    let tasks_value = f64::from(stats.task_count.unwrap_or(0));
    let runs_value = f64::from(stats.run_count.unwrap_or(0));
    let tokens_text = format_count(stats.token_count.unwrap_or(0));
    let cost_per_task_text = match stats.task_count {
        Some(tasks) if tasks > 0 => format_money(total / f64::from(tasks), &currency_code),
        _ => "—".to_string(),
    };
    let projects = sorted_by_amount(&project_breakdown);
    let models = sorted_by_amount(&model_breakdown);
    let project_total: f64 = projects.iter().map(|p| p.amount).sum();
    let model_total: f64 = models.iter().map(|m| m.amount).sum();
    let panel_source = selected().and_then(|(kind, id)| {
        let rows = match kind {
            EntryKind::Project => &project_breakdown,
            EntryKind::Model => &model_breakdown,
        };
        rows.iter().find(|row| row.id == id).cloned()
    });
    let is_today_live = !daily_costs.is_empty() && daily_costs.len() < days_in_month;
    let panel_bars: Vec<DayValue> = panel_source.as_ref().map_or_else(Vec::new, |row| {
        (0..days_in_month)
            .map(|index| DayValue {
                day_label: (index + 1).to_string(),
                day_value: row.daily_costs.get(index).copied(),
            })
            .collect()
    });
    let panel_name = panel_source
        .as_ref()
        .map_or_else(String::new, |row| row.entry_name.clone());
    let panel_figure = panel_source.as_ref().map_or(0.0, |row| row.amount);
    let panel_tasks: Vec<TopTask> = panel_source
        .as_ref()
        .map_or_else(Vec::new, |row| row.top_tasks.clone());
    let attention_list = attention_items.clone();
    let currency = currency_code.clone();
    let projects_currency = currency_code.clone();
    let models_currency = currency_code.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-aso",
            "data-sheet": sheet_now.then_some("open"),
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            header { class: "kit-aso__head",
                h2 { class: "kit-aso__title", "{month_label}" }
            }

            if is_loading {
                section { class: "kit-aso__hero",
                    span { class: "kit-aso__ph kit-aso__ph--big", "aria-hidden": "true" }
                    span { class: "kit-aso__ph", "aria-hidden": "true" }
                }
                section { class: "kit-aso__chart kit-aso__ph kit-aso__ph--chart", "aria-hidden": "true" }
            } else if is_empty || daily_costs.is_empty() {
                section { class: "kit-aso__hero",
                    p { class: "kit-aso__total", "{zero_text}" }
                    p { class: "kit-aso__sentence", "No spend yet this month. Costs appear here once a run finishes." }
                }
            } else {
                section { class: "kit-aso__hero", "aria-label": "Month so far",
                    p { class: "kit-aso__total", "{total_text}" }
                    if budget_amount.is_some() {
                        div { class: "kit-aso__budget",
                            span {
                                class: "kit-aso__bar",
                                role: "progressbar",
                                "aria-valuemin": "0",
                                "aria-valuemax": "100",
                                "aria-valuenow": "{budget_fraction_value}",
                                "aria-label": "Share of budget spent",
                                span { class: "kit-aso__bar-fill", style: "{budget_width}" }
                            }
                            span { class: "kit-aso__meta", "{budget_text}" }
                        }
                    }
                    p { class: "kit-aso__sentence", "{sentence}" }
                }
                section { class: "kit-aso__chart", "aria-label": "Cumulative spend by day",
                    SeriesTrendChart {
                        x_labels,
                        series,
                        variant: ChartVariant::Line,
                        fmt_value: format_axis,
                        value_prefix: currency_symbol(&currency_code),
                        height: 220.0,
                        label: "Cumulative spend",
                        empty_message: "No spend yet",
                    }
                }
                section { class: "kit-aso__stats", "aria-label": "Usage",
                    MetricTile { metric_label: "Tasks", metric_value: MetricValue::Number(tasks_value) }
                    MetricTile { metric_label: "Runs", metric_value: MetricValue::Number(runs_value) }
                    MetricTile { metric_label: "Tokens", metric_value: MetricValue::Text(tokens_text) }
                    MetricTile { metric_label: "Cost per task", metric_value: MetricValue::Text(cost_per_task_text) }
                }
                div { class: "kit-aso__lists",
                    BreakdownList {
                        heading: "Projects",
                        rows: projects,
                        kind: EntryKind::Project,
                        total: project_total,
                        code: projects_currency,
                        selected: selected(),
                        on_pick: move |(kind, id): (EntryKind, String)| {
                            selected.set(Some((kind, id.clone())));
                            sheet_open.set(true);
                            if let Some(handler) = on_select {
                                handler.call((kind, id));
                            }
                        },
                    }
                    BreakdownList {
                        heading: "Models",
                        rows: models,
                        kind: EntryKind::Model,
                        total: model_total,
                        code: models_currency,
                        selected: selected(),
                        on_pick: move |(kind, id): (EntryKind, String)| {
                            selected.set(Some((kind, id.clone())));
                            sheet_open.set(true);
                            if let Some(handler) = on_select {
                                handler.call((kind, id));
                            }
                        },
                    }
                }
                if panel_source.is_some() {
                    section { class: "kit-aso__panel", "aria-label": "{panel_name}",
                        button {
                            r#type: "button",
                            class: "kit-aso__back",
                            onclick: move |_| sheet_open.set(false),
                            "Back to spend"
                        }
                        h3 { class: "kit-aso__panel-title", "{panel_name}" }
                        DailyUsageBars {
                            chart_heading: "Daily spend",
                            day_values: panel_bars,
                            total_figures: vec![Figure { figure_label: "Spent".to_string(), figure_value: panel_figure }],
                            is_today_live,
                            live_badge_text: "Today",
                            empty_message: "No spend yet",
                            plot_height: 140.0,
                        }
                        if !panel_tasks.is_empty() {
                            ol { class: "kit-aso__tasks",
                                for task in panel_tasks.iter() {
                                    {
                                        let task_title = task.entry_title.clone();
                                        let task_runs = format!("{} runs", task.run_count);
                                        let task_cost = format_money(task.cost, &currency);
                                        rsx! {
                                            li { key: "{task_title}",
                                                span { "{task_title}" }
                                                span { class: "kit-aso__meta", "{task_runs}" }
                                                span { class: "kit-aso__num", "{task_cost}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !attention_list.is_empty() {
                    section { class: "kit-aso__attention", "aria-label": "Worth a look",
                        h3 { class: "kit-aso__section-title", "Worth a look" }
                        ul { class: "kit-aso__items",
                            for item in attention_list.iter() {
                                {
                                    let item_key = item.id.clone();
                                    let kind_label = item.item_kind.label();
                                    let item_text = item.text.clone();
                                    let item_detail = item.detail_text.clone().unwrap_or_default();
                                    let open_label = format!("Open: {item_text}");
                                    let target = item.clone();
                                    rsx! {
                                        li { key: "{item_key}", class: "kit-aso__item",
                                            span { class: "kit-aso__kind", "{kind_label}" }
                                            span { class: "kit-aso__item-body",
                                                span { class: "kit-aso__item-text", "{item_text}" }
                                                if !item_detail.is_empty() {
                                                    span { class: "kit-aso__meta", "{item_detail}" }
                                                }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-aso__open",
                                                "aria-label": "{open_label}",
                                                onclick: move |_| {
                                                    if let Some(handler) = on_open_item {
                                                        handler.call(target.clone());
                                                    }
                                                },
                                                "Open"
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
    }
}

#[component]
fn BreakdownList(
    heading: String,
    rows: Vec<Breakdown>,
    kind: EntryKind,
    total: f64,
    code: String,
    selected: Option<(EntryKind, String)>,
    on_pick: EventHandler<(EntryKind, String)>,
) -> Element {
    let base = format!("kit-aso-{}", kind.as_str());
    let count = rows.len();
    rsx! {
        section { class: "kit-aso__list", "aria-label": "{heading}",
            h3 { class: "kit-aso__section-title", "{heading}" }
            ol { class: "kit-aso__rows",
                for (index, row) in rows.iter().enumerate() {
                    {
                        let row_key = row.id.clone();
                        let row_id = format!("{base}-{index}");
                        let row_name = row.entry_name.clone();
                        let row_amount = format_money(row.amount, &code);
                        let row_share = share_pct(row.amount, total);
                        let share_text = format!("{row_share:.0}%");
                        let share_width = format!("width: {row_share:.1}%;");
                        let share_label = format!(
                            "{row_name}, {row_amount}, {row_share:.0} percent of {heading}"
                        );
                        let pressed = selected
                            .as_ref()
                            .is_some_and(|(k, id)| *k == kind && *id == row.id)
                            .to_string();
                        let direction = direction_of(row.change_vs_last_month);
                        let change_text = match row.change_vs_last_month {
                            Some(change) => format!("{} vs last month", change_words(change)),
                            None => "no comparison".to_string(),
                        };
                        let pick_id = row.id.clone();
                        let focus_base = base.clone();
                        rsx! {
                            li { key: "{row_key}",
                                button {
                                    id: "{row_id}",
                                    r#type: "button",
                                    class: "kit-aso__row",
                                    "aria-pressed": "{pressed}",
                                    "aria-label": "{share_label}",
                                    onclick: move |_| on_pick.call((kind, pick_id.clone())),
                                    onkeydown: move |event: KeyboardEvent| {
                                        let delta: isize = match event.key() {
                                            Key::ArrowDown => 1,
                                            Key::ArrowUp => -1,
                                            _ => return,
                                        };
                                        let next = index as isize + delta;
                                        if next >= 0 && (next as usize) < count {
                                            focus_after_frame(&format!("{focus_base}-{next}"));
                                        }
                                    },
                                    span { class: "kit-aso__row-name", "{row_name}" }
                                    span { class: "kit-aso__num", "{row_amount}" }
                                    span { class: "kit-aso__share", "{share_text}" }
                                    span { class: "kit-aso__change", "data-direction": "{direction}", "{change_text}" }
                                    span { class: "kit-aso__bar kit-aso__bar--share", "aria-hidden": "true",
                                        span { class: "kit-aso__bar-fill", style: "{share_width}" }
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
