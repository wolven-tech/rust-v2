use dioxus::prelude::*;
use rv2_kit::components::daily_usage_bars::{DailyUsageBars, DayValue, Figure};
use rv2_kit::components::metric_tile::{MetricTile, MetricValue};
use rv2_kit::components::part_of_total_card::{Part, PartOfTotalCard, PeriodOption};
use rv2_kit::components::provider_usage_chart::{
    Measure as UsageMeasure, ModelUsage, ProviderGroup, ProviderUsageChart, Usage,
};
use rv2_kit::components::quota_usage_panel::{
    ActionState, CapMeter, DayPoint, PeriodChoice, QuotaUsagePanel, SpendPart,
};
use rv2_kit::components::run_cost_breakdown::{
    Measure as RunMeasure, RunCostBreakdown, RunState, Step, StepKind, StepStatus,
};
use rv2_kit::components::series_trend_chart::SeriesTone;
use std::collections::BTreeMap;

use super::Scene;
use super::data::percent;
use super::gallery_now;

const PLAN_TOTAL: f64 = 500.0;
const ALERT_RATIO: f64 = 0.8;

fn usage(tokens: f64, cost: f64, requests: f64) -> Usage {
    Usage {
        tokens: Some(tokens),
        cost_usd: Some(cost),
        request_count: Some(requests),
    }
}

fn run_step(
    id: &str,
    name: &str,
    kind: StepKind,
    cost: f64,
    elapsed: f64,
    children: Vec<Step>,
) -> Step {
    Step {
        step_id: id.into(),
        step_name: name.into(),
        step_kind: kind,
        step_status: StepStatus::Done,
        model_name: Some("Balanced tier".into()),
        step_note: None,
        in_tokens: Some(4_200.0),
        out_tokens: Some(640.0),
        step_cost: Some(cost),
        step_elapsed: Some(elapsed),
        prompt_text: None,
        result_text: None,
        error_text: None,
        child_steps: children,
    }
}

fn run_steps() -> Vec<Step> {
    vec![run_step(
        "root",
        "Monday pipeline review, run 12",
        StepKind::Agent,
        0.41,
        41.2,
        vec![
            run_step(
                "pull",
                "Pull open deals",
                StepKind::ToolCall,
                0.02,
                2.4,
                vec![],
            ),
            run_step(
                "rank",
                "Rank by renewal risk",
                StepKind::ModelCall,
                0.31,
                30.6,
                vec![],
            ),
            run_step(
                "write",
                "Write owner summaries",
                StepKind::ModelCall,
                0.08,
                8.2,
                vec![],
            ),
        ],
    )]
}

fn provider_groups() -> Vec<ProviderGroup> {
    vec![
        ProviderGroup {
            group_name: "Anthropic".into(),
            group_tone: Some(SeriesTone::Ink),
            group_logo: None,
            models: vec![ModelUsage {
                model_name: "Balanced tier".into(),
                usage: usage(820_000.0, 12.4, 310.0),
                task_usage: BTreeMap::from([
                    ("Pipeline review".to_string(), usage(500_000.0, 7.9, 180.0)),
                    ("Drafting".to_string(), usage(320_000.0, 4.5, 130.0)),
                ]),
            }],
        },
        ProviderGroup {
            group_name: "OpenAI".into(),
            group_tone: Some(SeriesTone::Ok),
            group_logo: None,
            models: vec![ModelUsage {
                model_name: "Fast tier".into(),
                usage: usage(440_000.0, 1.1, 260.0),
                task_usage: BTreeMap::from([(
                    "Routing and tags".to_string(),
                    usage(440_000.0, 1.1, 260.0),
                )]),
            }],
        },
        ProviderGroup {
            group_name: "Local llama-server".into(),
            group_tone: Some(SeriesTone::Muted),
            group_logo: None,
            models: vec![ModelUsage {
                model_name: "Offline model".into(),
                usage: usage(90_000.0, 0.0, 40.0),
                task_usage: BTreeMap::new(),
            }],
        },
    ]
}

#[component]
pub fn Costs() -> Element {
    let mut period = use_signal(|| Some("month".to_string()));
    let mut warning_dismissed = use_signal(|| false);
    let mut last = use_signal(|| "no action yet".to_string());
    let spent = 420.0_f64;
    let pct = percent(spent as u32, PLAN_TOTAL as u32);
    let days: Vec<DayValue> = (1..=10)
        .map(|d: u32| DayValue {
            day_label: format!("Nov {}", d + 11),
            day_value: Some(f64::from(d * 37 % 90 + 10)),
        })
        .collect();

    rsx! {
        Scene {
            id: "costs",
            title: "Costs and budgets",
            story: "Sam watches the monthly budget. It is at 84% with eleven days left, so the 80% warning shows and spend is split by workflow and provider.",
            div { class: "lh-stack",
                if pct >= 80 && !warning_dismissed() {
                    div { class: "lh-banner", role: "status",
                        span { "Budget at {pct}%. Workflows warn at 80% and stop at 100%." }
                        button { class: "lh-btn", r#type: "button",
                            onclick: move |_| {
                                warning_dismissed.set(true);
                                last.set("warning dismissed for today".to_string());
                            },
                            "Dismiss for today"
                        }
                    }
                }
                div { class: "lh-tiles",
                    MetricTile {
                        metric_label: "Spent this month",
                        metric_value: MetricValue::Text("$420".to_string()),
                        change_note: Some("of $500 plan".to_string()),
                        lower_is_better: true,
                    }
                    MetricTile {
                        metric_label: "Daily average",
                        metric_value: MetricValue::Text("$14".to_string()),
                        change_percent: Some(-4.0),
                        lower_is_better: true,
                    }
                    MetricTile {
                        metric_label: "Days left",
                        metric_value: MetricValue::Number(11.0),
                        change_note: Some("until the plan resets".to_string()),
                    }
                }
                div { class: "lh-grid",
                    QuotaUsagePanel {
                        plan_total: PLAN_TOTAL,
                        plan_label: "Bramble Studio plan",
                        alert_ratio: ALERT_RATIO,
                        spend_breakdown: vec![
                            SpendPart { spend_label: "Monday pipeline review".into(), spend_value: 180.0, spend_emphasis: Some(SeriesTone::Ink) },
                            SpendPart { spend_label: "Weekly newsletter".into(), spend_value: 120.0, spend_emphasis: Some(SeriesTone::Warn) },
                            SpendPart { spend_label: "Refund replies".into(), spend_value: 60.0, spend_emphasis: None },
                            SpendPart { spend_label: "Chat".into(), spend_value: 60.0, spend_emphasis: None },
                        ],
                        cap_list: vec![CapMeter { cap_label: "Monthly".into(), cap_used: spent, cap_total: Some(PLAN_TOTAL), unit: Some("credits".into()), decimals: Some(0) }],
                        period_choices: vec![
                            PeriodChoice { period_id: "m".into(), period_label: "November".into() },
                            PeriodChoice { period_id: "o".into(), period_label: "October".into() },
                        ],
                        shown_period: Some("m".to_string()),
                        resets_at_ms: Some(super::NOW_MS as i64 + 86_400_000 * 11),
                        now_ms: Some(gallery_now() as i64),
                        daily_trend: (1..=12).map(|d: u32| DayPoint { day_label: format!("{d}"), day_value: Some(f64::from(d * 13 % 40)) }).collect(),
                        on_action: move |a: ActionState| last.set(format!("budget action: {a:?}")),
                        on_period_pick: move |p: String| last.set(format!("period {p}")),
                    }
                    PartOfTotalCard {
                        part_entries: vec![
                            Part { part_key: Some("pipeline".into()), part_name: "Monday pipeline".into(), part_amount: 180.0, part_color: Some(SeriesTone::Ink) },
                            Part { part_key: Some("news".into()), part_name: "Newsletter".into(), part_amount: 120.0, part_color: Some(SeriesTone::Warn) },
                            Part { part_key: Some("refunds".into()), part_name: "Refund replies".into(), part_amount: 60.0, part_color: Some(SeriesTone::Ok) },
                            Part { part_key: Some("chat".into()), part_name: "Chat".into(), part_amount: 60.0, part_color: Some(SeriesTone::Muted) },
                        ],
                        card_title: "Spend by workflow",
                        period_options: vec![
                            PeriodOption { period_key: "month".into(), period_name: "This month".into() },
                            PeriodOption { period_key: "quarter".into(), period_name: "This quarter".into() },
                        ],
                        active_period: period(),
                        on_period_change: move |k: String| {
                            last.set(format!("period {k}"));
                            period.set(Some(k));
                        },
                        change_pct: Some(6.4),
                    }
                }
                div { class: "lh-grid",
                    DailyUsageBars {
                        chart_heading: "Daily spend",
                        day_values: days,
                        total_figures: vec![
                            Figure { figure_label: "Total".into(), figure_value: 420.0 },
                            Figure { figure_label: "Average".into(), figure_value: 42.0 },
                        ],
                        period_text: "Last 10 days",
                        is_today_live: true,
                        live_badge_text: "Live",
                    }
                    ProviderUsageChart {
                        provider_groups: provider_groups(),
                        measure_options: vec![UsageMeasure::Cost, UsageMeasure::Tokens, UsageMeasure::Requests],
                        period_label: "This month",
                        prior_totals: Some(usage(900_000.0, 11.0, 300.0)),
                        prior_label: Some("Last month".to_string()),
                    }
                }
                section { class: "lh-card", "aria-label": "Latest run",
                    RunCostBreakdown {
                        step_list: run_steps(),
                        run_label: "Latest run",
                        run_summary: Some("3 steps, $0.41".to_string()),
                        status_override: Some(RunState::Done),
                        starting_measure: RunMeasure::Cost,
                        on_measure_change: move |m: RunMeasure| last.set(format!("measure {m:?}")),
                    }
                }
                section { class: "lh-card", "aria-label": "Budgets per workflow",
                    h3 { class: "lh-h", "Budgets per workflow" }
                    ul { class: "lh-list",
                        for (name, spent_now, budget) in [("Monday pipeline review", 180.0_f64, 200.0_f64), ("Weekly newsletter", 120.0, 150.0), ("Refund replies", 60.0, 200.0), ("Renewal follow-ups", 0.0, 100.0)] {
                            li { key: "{name}",
                                span { "{name}" }
                                span { class: "lh-row",
                                    span { class: "lh-muted", "${spent_now:.0} of ${budget:.0}" }
                                    span { class: "lh-chip", "data-tone": if spent_now / budget >= ALERT_RATIO { "warn" } else { "ok" },
                                        {if spent_now / budget >= ALERT_RATIO { "80% warning" } else { "within budget" }}
                                    }
                                }
                            }
                        }
                    }
                    p { class: "gl-out", "data-out": "costs", "{last()}" }
                }
            }
        }
    }
}
