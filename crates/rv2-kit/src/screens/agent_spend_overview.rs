//! Agent spend overview: what the agents cost this month, against the budget,
//! where the money is heading at the current pace, and where it went by project
//! and by model. Items that deserve a look come last.
//!
//! Design choice: the hero chart is drawn here, not by `SeriesTrendChart`. Its
//! y axis carries no labels and its x labels sit at fixed days, so it needs its own
//! geometry. Actual spend is a solid line over an area; the forecast is dashed from
//! the last actual day to month end; the budget is a flat line at the top.
//! Design choice: from 761 px the main column and the right panel take fixed
//! fractions of the content area; below that they stack in one column.
//! Design choice: the ranked lists are this module's own rows. Each row needs a
//! change figure with its sign, which the shared composition card does not carry.
//! Design choice: the hero total counts up over twelve frames when the daily costs
//! change; reduced motion sets it at once.
//! Browser checks: the forecast dash starts at the last actual day; the sentence
//! names the day the budget is passed; Up/Down move through a list; Enter opens
//! the panel; no horizontal scroll at 320 px.

use dioxus::prelude::*;

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

/// Token counts in words of magnitude: `48.2M`, `1.5K`, or the plain count below a thousand.
pub fn format_tokens(value: u64) -> String {
    match value {
        v if v >= 1_000_000 => format!("{:.1}M", v as f64 / 1_000_000.0),
        v if v >= 1_000 => format!("{:.1}K", v as f64 / 1_000.0),
        v => v.to_string(),
    }
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
/// Day labels under the hero chart: days 1, 8, 15 and 23, then the last day of the month.
pub fn axis_days(days_in_month: usize) -> Vec<usize> {
    let last = days_in_month.max(1);
    let mut days: Vec<usize> = [1, 8, 15, 23]
        .into_iter()
        .filter(|day| *day < last)
        .collect();
    days.push(last);
    days
}

/// Horizontal position of a day across the plot: 0 at day 1, 100 at the last day.
pub fn day_x_percent(day: usize, days_in_month: usize) -> f64 {
    let span = days_in_month.saturating_sub(1).max(1) as f64;
    day.saturating_sub(1) as f64 / span * 100.0
}

/// Vertical position from the plot top: 0 at `top_value`, 100 at zero.
pub fn value_y_percent(value: f64, top_value: f64) -> f64 {
    if top_value <= 0.0 {
        return 100.0;
    }
    (100.0 - value / top_value * 100.0).clamp(0.0, 100.0)
}

/// Value at the top of the hero chart: the budget when set, otherwise the
/// larger of today's total and the month-end forecast, with headroom.
pub fn chart_top(daily: &[f64], days_in_month: usize, budget: Option<f64>) -> f64 {
    if let Some(limit) = budget.filter(|limit| *limit > 0.0) {
        return limit;
    }
    let spent = cumulative(daily).last().copied().unwrap_or(0.0);
    let forecast_peak = forecast_series(daily, days_in_month)
        .into_iter()
        .flatten()
        .fold(0.0_f64, f64::max);
    (spent.max(forecast_peak) * 1.1).max(1.0)
}

/// Path data for the hero chart's three shapes, in a 1000 by 100 box.
struct ChartPaths {
    spent: String,
    area: String,
    forecast: String,
    dot: Option<(f64, f64)>,
}

fn polyline(points: &[(f64, f64)]) -> String {
    points
        .iter()
        .enumerate()
        .map(|(index, (x, y))| {
            let command = if index == 0 { "M" } else { "L" };
            format!("{command} {:.1} {:.1}", x * 10.0, y)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn chart_paths(daily: &[f64], days_in_month: usize, top: f64) -> ChartPaths {
    let running = cumulative(daily);
    let spent_points: Vec<(f64, f64)> = running
        .iter()
        .enumerate()
        .map(|(index, value)| {
            (
                day_x_percent(index + 1, days_in_month),
                value_y_percent(*value, top),
            )
        })
        .collect();
    let forecast_points: Vec<(f64, f64)> = forecast_series(daily, days_in_month)
        .into_iter()
        .enumerate()
        .filter_map(|(index, value)| {
            value.map(|v| {
                (
                    day_x_percent(index + 1, days_in_month),
                    value_y_percent(v, top),
                )
            })
        })
        .collect();
    let spent = polyline(&spent_points);
    let area = match (spent_points.first(), spent_points.last()) {
        (Some((first_x, _)), Some((last_x, _))) => format!(
            "{} L {:.1} 100 L {:.1} 100 Z",
            polyline(&spent_points),
            last_x * 10.0,
            first_x * 10.0
        ),
        _ => String::new(),
    };
    let dot = spent_points.last().copied();
    ChartPaths {
        spent,
        area,
        forecast: polyline(&forecast_points),
        dot,
    }
}

/// Series token for a row's marker, cycling through the six series tokens.
fn row_series(index: usize) -> usize {
    index % 6 + 1
}

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
    let mut panel_open = use_signal(|| true);
    let mut tab = use_signal(|| EntryKind::Project);
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
    let budget_text = budget_amount.map_or_else(String::new, |limit| {
        format!("of {}", format_money(limit, &currency_code))
    });
    let budget_share_text = budget_amount.map_or_else(String::new, |limit| {
        format!(
            "{:.0}% of budget",
            (total / limit * 100.0).clamp(0.0, 100.0)
        )
    });
    let top = chart_top(&daily_costs, days_in_month, budget_amount);
    let paths = chart_paths(&daily_costs, days_in_month, top);
    let dot_style = paths.dot.map_or_else(String::new, |(x, y)| {
        format!("left: {x:.2}%; top: {y:.2}%;")
    });
    let budget_line_y = budget_amount.map(|limit| value_y_percent(limit, top));
    let budget_label = budget_amount
        .map(|limit| format!("Budget {}", format_money(limit, &currency_code)))
        .unwrap_or_default();
    let x_labels: Vec<(String, String)> = axis_days(days_in_month)
        .into_iter()
        .map(|day| {
            let label = if day == days_in_month.max(1) {
                format!("Day {day}")
            } else {
                day.to_string()
            };
            let style = format!("left: {:.2}%;", day_x_percent(day, days_in_month));
            (label, style)
        })
        .collect();
    let chart_label = format!(
        "Cumulative spend by day: {total_text} spent across {} days of {days_in_month}",
        daily_costs.len()
    );
    let stats = usage_stats.unwrap_or_default();
    let tasks_text = stats
        .task_count
        .map_or_else(|| "—".to_string(), |n| n.to_string());
    let runs_text = stats
        .run_count
        .map_or_else(|| "—".to_string(), |n| n.to_string());
    let tokens_text = format_tokens(stats.token_count.unwrap_or(0));
    let cost_per_task_text = match stats.task_count {
        Some(tasks) if tasks > 0 => format_money(total / f64::from(tasks), &currency_code),
        _ => "—".to_string(),
    };
    let projects = sorted_by_amount(&project_breakdown);
    let models = sorted_by_amount(&model_breakdown);
    let project_total: f64 = projects.iter().map(|p| p.amount).sum();
    let model_total: f64 = models.iter().map(|m| m.amount).sum();
    let default_pick = projects
        .first()
        .map(|row| (EntryKind::Project, row.id.clone()));
    let picked = selected().or(default_pick);
    let panel_source = picked.clone().and_then(|(kind, id)| {
        let rows = match kind {
            EntryKind::Project => &project_breakdown,
            EntryKind::Model => &model_breakdown,
        };
        rows.iter().find(|row| row.id == id).cloned()
    });
    let panel_bars: Vec<(String, String)> = panel_source.as_ref().map_or_else(Vec::new, |row| {
        let peak = row.daily_costs.iter().copied().fold(0.0_f64, f64::max);
        row.daily_costs
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let height = if peak > 0.0 {
                    value / peak * 100.0
                } else {
                    0.0
                };
                let title = format!(
                    "Day {}: {}",
                    index + 1,
                    format_money(*value, &currency_code)
                );
                (title, format!("height: {height:.1}%;"))
            })
            .collect()
    });
    let panel_name = panel_source
        .as_ref()
        .map_or_else(String::new, |row| row.entry_name.clone());
    let panel_amount = panel_source
        .as_ref()
        .map_or_else(String::new, |row| format_money(row.amount, &currency_code));
    let panel_facts = panel_source.as_ref().map_or_else(String::new, |row| {
        format!("{:.0}% of the month", share_pct(row.amount, total))
    });
    let panel_change =
        panel_source
            .as_ref()
            .map_or_else(String::new, |row| match row.change_vs_last_month {
                Some(change) => format!("{} vs last month", change_words(change)),
                None => "no comparison".to_string(),
            });
    let panel_direction = panel_source
        .as_ref()
        .map_or("flat", |row| direction_of(row.change_vs_last_month));
    let panel_tasks: Vec<TopTask> = panel_source
        .as_ref()
        .map_or_else(Vec::new, |row| row.top_tasks.clone());
    let panel_day_span = format!("Day 1 to day {}", panel_bars.len());
    let attention_list = attention_items.clone();
    let currency = currency_code.clone();
    let projects_currency = currency_code.clone();
    let models_currency = currency_code.clone();
    let spent_label = format!("Spent in {month_label}");
    let week_label = format!("Usage, {month_label}");

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: "kit-aso",
            "data-loading": is_loading.then_some("true"),
            "aria-busy": is_loading.then_some("true"),
            div { class: "kit-aso__main", "aria-label": "{week_label}",
                div { class: "kit-aso__canvas",
                    header { class: "kit-aso__head",
                        h2 { class: "kit-aso__title", "Usage" }
                        p { class: "kit-aso__month", "{month_label}" }
                    }

                    if is_loading {
                        section { class: "kit-aso__hero kit-aso__hero--loading",
                            span { class: "kit-aso__ph kit-aso__ph--big", "aria-hidden": "true" }
                            span { class: "kit-aso__ph", "aria-hidden": "true" }
                        }
                    } else if is_empty || daily_costs.is_empty() {
                        section { class: "kit-aso__hero",
                            p { class: "kit-aso__total", "{zero_text}" }
                            p { class: "kit-aso__pace", "No spend yet this month. Costs appear here once a run finishes." }
                        }
                    } else {
                        section { class: "kit-aso__hero", "aria-label": "Month so far",
                            p { class: "kit-aso__spent-label", "{spent_label}" }
                            p { class: "kit-aso__total-row",
                                span { class: "kit-aso__total", "{total_text}" }
                                if !budget_text.is_empty() {
                                    span { class: "kit-aso__of", "{budget_text}" }
                                }
                                if !budget_share_text.is_empty() {
                                    span { class: "kit-aso__of", "· {budget_share_text}" }
                                }
                            }
                            p { class: "kit-aso__pace", "{sentence}" }
                            div { class: "kit-aso__chart", role: "img",
                                "aria-label": "{chart_label}",
                                div { class: "kit-aso__plot",
                                    svg { class: "kit-aso__svg", view_box: "0 0 1000 100", preserve_aspect_ratio: "none", "aria-hidden": "true",
                                        if let Some(y) = budget_line_y {
                                            line { class: "kit-aso__budget-line", x1: "0", x2: "1000", y1: "{y:.2}", y2: "{y:.2}" }
                                        }
                                        path { class: "kit-aso__area", d: "{paths.area}" }
                                        path { class: "kit-aso__forecast", d: "{paths.forecast}" }
                                        path { class: "kit-aso__spent", d: "{paths.spent}" }
                                    }
                                    if !budget_label.is_empty() {
                                        span { class: "kit-aso__budget-label", "{budget_label}" }
                                    }
                                    if !dot_style.is_empty() {
                                        span { class: "kit-aso__dot", "aria-hidden": "true", style: "{dot_style}" }
                                    }
                                }
                                div { class: "kit-aso__xaxis", "aria-hidden": "true",
                                    for (label, position) in x_labels.iter().cloned() {
                                        span {
                                            key: "{label}",
                                            class: "kit-aso__xlabel",
                                            style: "{position}",
                                            "{label}"
                                        }
                                    }
                                }
                            }
                            div { class: "kit-aso__divider", "aria-hidden": "true" }
                            section { class: "kit-aso__stats", "aria-label": "Usage",
                                div { class: "kit-aso__stat",
                                    span { class: "kit-aso__stat-label", "Tasks" }
                                    span { class: "kit-aso__stat-value", "{tasks_text}" }
                                }
                                div { class: "kit-aso__stat",
                                    span { class: "kit-aso__stat-label", "Runs" }
                                    span { class: "kit-aso__stat-value", "{runs_text}" }
                                }
                                div { class: "kit-aso__stat",
                                    span { class: "kit-aso__stat-label", "Cost per task" }
                                    span { class: "kit-aso__stat-value", "{cost_per_task_text}" }
                                }
                                div { class: "kit-aso__stat",
                                    span { class: "kit-aso__stat-label", "Tokens" }
                                    span { class: "kit-aso__stat-value", "{tokens_text}" }
                                }
                            }
                        }
                        section { class: "kit-aso__ranked", "aria-label": "Ranked",
                            div { class: "kit-aso__ranked-head",
                                h3 { class: "kit-aso__section-title", "Where it went" }
                                div { class: "kit-aso__switch", role: "group", "aria-label": "Ranking",
                                    button {
                                        r#type: "button",
                                        class: "kit-aso__seg-btn",
                                        "aria-pressed": if tab() == EntryKind::Project { "true" } else { "false" },
                                        onclick: move |_| tab.set(EntryKind::Project),
                                        "Projects"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "kit-aso__seg-btn",
                                        "aria-pressed": if tab() == EntryKind::Model { "true" } else { "false" },
                                        onclick: move |_| tab.set(EntryKind::Model),
                                        "Models"
                                    }
                                }
                            }
                            if tab() == EntryKind::Project {
                                BreakdownList {
                                    heading: "Projects",
                                    rows: projects,
                                    kind: EntryKind::Project,
                                    total: project_total,
                                    code: projects_currency,
                                    selected: picked.clone(),
                                    on_pick: move |(kind, id): (EntryKind, String)| {
                                        selected.set(Some((kind, id.clone())));
                                        panel_open.set(true);
                                        if let Some(handler) = on_select {
                                            handler.call((kind, id));
                                        }
                                    },
                                }
                            } else {
                                BreakdownList {
                                    heading: "Models",
                                    rows: models,
                                    kind: EntryKind::Model,
                                    total: model_total,
                                    code: models_currency,
                                    selected: picked.clone(),
                                    on_pick: move |(kind, id): (EntryKind, String)| {
                                        selected.set(Some((kind, id.clone())));
                                        panel_open.set(true);
                                        if let Some(handler) = on_select {
                                            handler.call((kind, id));
                                        }
                                    },
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
                                                    span { class: "kit-aso__icon", "aria-hidden": "true",
                                                        svg { class: "kit-aso__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round",
                                                            circle { cx: "8", cy: "8", r: "5.5" }
                                                            path { d: "M8 5.25v3.25M8 10.75v.01" }
                                                        }
                                                    }
                                                    span { class: "kit-aso__item-body",
                                                        span { class: "kit-aso__item-text", "{item_text}" }
                                                        if !item_detail.is_empty() {
                                                            span { class: "kit-aso__meta", "{item_detail}" }
                                                        }
                                                    }
                                                    span { class: "kit-aso__kind", "{kind_label}" }
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

            if panel_open() && panel_source.is_some() && !is_loading && !is_empty && !daily_costs.is_empty() {
                section { class: "kit-aso__panel", "aria-label": "{panel_name}",
                    div { class: "kit-aso__tab",
                        span { class: "kit-aso__chip", "{panel_name}" }
                        button {
                            r#type: "button",
                            class: "kit-aso__close",
                            "aria-label": "Close panel",
                            onclick: move |_| panel_open.set(false),
                            svg { class: "kit-aso__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", "aria-hidden": "true",
                                path { d: "M4 4l8 8M12 4l-8 8" }
                            }
                        }
                    }
                    p { class: "kit-aso__panel-name",
                        span { class: "kit-aso__swatch", "aria-hidden": "true" }
                        "{panel_name}"
                    }
                    p { class: "kit-aso__panel-amount", "{panel_amount} · {panel_facts}" }
                    p { class: "kit-aso__panel-change", "data-direction": "{panel_direction}", "{panel_change}" }
                    p { class: "kit-aso__panel-label kit-aso__panel-label--day", "Day by day" }
                    div { class: "kit-aso__bars", role: "img", "aria-label": "Daily spend for {panel_name}",
                        for (title, style) in panel_bars.iter().cloned() {
                            span { class: "kit-aso__bar-day", title: "{title}", style: "{style}" }
                        }
                    }
                    p { class: "kit-aso__axis", "{panel_day_span}" }
                    p { class: "kit-aso__panel-label kit-aso__panel-label--cost", "What cost the most" }
                    div { class: "kit-aso__task-card",
                        if !panel_tasks.is_empty() {
                            ol { class: "kit-aso__tasks",
                                for task in panel_tasks.iter() {
                                    {
                                        let task_title = task.entry_title.clone();
                                        let task_runs = format!("{} runs", task.run_count);
                                        let task_cost = format_money(task.cost, &currency);
                                        rsx! {
                                            li { key: "{task_title}",
                                                span { class: "kit-aso__task-title", "{task_title}" }
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
        ol { class: "kit-aso__rows", "aria-label": "{heading}",
            for (index, row) in rows.iter().enumerate() {
                {
                    let row_key = row.id.clone();
                    let row_id = format!("{base}-{index}");
                    let row_name = row.entry_name.clone();
                    let row_amount = format_money(row.amount, &code);
                    let row_share = share_pct(row.amount, total);
                    let share_text = format!("{row_share:.0}%");
                    let share_width = format!("width: {row_share:.1}%;");
                    let series = format!("{}", row_series(index));
                    let share_label = format!(
                        "{row_name}, {row_amount}, {row_share:.0} percent of {heading}"
                    );
                    let pressed = selected
                        .as_ref()
                        .is_some_and(|(k, id)| *k == kind && *id == row.id)
                        .to_string();
                    let direction = direction_of(row.change_vs_last_month);
                    let change_text = match row.change_vs_last_month {
                        Some(change) => change_words(change),
                        None => "no comparison".to_string(),
                    };
                    let pick_id = row.id.clone();
                    let focus_base = base.clone();
                    rsx! {
                        li { key: "{row_key}", class: "kit-aso__cell",
                            button {
                                id: "{row_id}",
                                r#type: "button",
                                class: "kit-aso__row",
                                "data-series": "{series}",
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
                                span { class: "kit-aso__marker", "aria-hidden": "true" }
                                span { class: "kit-aso__row-name", "{row_name}" }
                                span { class: "kit-aso__bar", "aria-hidden": "true",
                                    span { class: "kit-aso__bar-fill", style: "{share_width}" }
                                }
                                span { class: "kit-aso__num", "{row_amount}" }
                                span { class: "kit-aso__share", "{share_text}" }
                                span { class: "kit-aso__change", "data-direction": "{direction}", "{change_text}" }
                                svg { class: "kit-aso__chevron", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                                    path { d: "M6 3.5 10.5 8 6 12.5" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
