use dioxus::prelude::*;

use super::{Item, NOW_MS, Section, gallery_now};

#[component]
pub fn Charts() -> Element {
    rsx! {
        Section { id: "charts", title: "Charts",
            ContextFillGridDemo {}
            ContextTreemapDemo {}
            DailyUsageBarsDemo {}
            MetricTileDemo {}
            MetricTrendCardDemo {}
            PartOfTotalCardDemo {}
            ProviderUsageChartDemo {}
            QuotaUsagePanelDemo {}
            RankingBracketDemo {}
            CandidateRaceBoardDemo {}
            RunCostBreakdownDemo {}
            SegmentedBarExplorerDemo {}
            SeriesTrendChartDemo {}
            VariantTestCardDemo {}
        }
    }
}

#[component]
fn ContextFillGridDemo() -> Element {
    use rv2_kit::components::context_fill_grid::{
        ContextFillGrid, ContextItem, ContextPart, FooterAction,
    };
    use rv2_kit::components::series_trend_chart::SeriesTone;
    let mut last = use_signal(String::new);
    let parts = vec![
        ContextPart {
            part_key: "sys".into(),
            part_name: "System prompt".into(),
            token_count: Some(3_200.0),
            sub_items: vec![ContextItem {
                item_key: "rules".into(),
                item_name: "Rules".into(),
                item_tokens: 2_000.0,
            }],
            is_locked: true,
            can_compact: false,
            part_color: None,
        },
        ContextPart {
            part_key: "docs".into(),
            part_name: "Documents".into(),
            token_count: Some(48_000.0),
            sub_items: vec![
                ContextItem {
                    item_key: "spec".into(),
                    item_name: "spec.md".into(),
                    item_tokens: 30_000.0,
                },
                ContextItem {
                    item_key: "notes".into(),
                    item_name: "notes.md".into(),
                    item_tokens: 18_000.0,
                },
            ],
            is_locked: false,
            can_compact: true,
            part_color: Some(SeriesTone::Warn),
        },
    ];
    rsx! {
        Item { slug: "context-fill-grid", name: "ContextFillGrid",
            ContextFillGrid {
                part_list: parts,
                window_size: 200_000.0,
                model_name: "Model A",
                panel_title: "Context usage",
                on_item_remove: move |(p, i): (String, String)| last.set(format!("remove {p}/{i}")),
                on_compact_request: move |p: String| last.set(format!("compact {p}")),
                footer_action: Some(FooterAction { action_text: "Manage context".into(), on_activate: EventHandler::new(move |_| last.set("manage".into())) }),
            }
            div { class: "gl-out", "data-out": "context-fill-grid", "{last()}" }
        }
    }
}

#[component]
fn ContextTreemapDemo() -> Element {
    use rv2_kit::components::context_treemap::{
        AgentOption, Block, BlockKind, CompactionRecord, ContextTreemap, View,
    };
    let mut last = use_signal(String::new);
    let mut view = use_signal(|| View::Before);
    let block = |key: &str, label: &str, kind: BlockKind, tokens: f64| Block {
        block_key: key.into(),
        block_label: label.into(),
        block_kind: kind,
        token_count: tokens,
        block_note: None,
        sub_items: vec![],
    };
    let blocks = vec![
        block("ins", "Instructions", BlockKind::Instructions, 4_000.0),
        block("doc", "Spec document", BlockKind::Document, 26_000.0),
        block("conv", "Conversation", BlockKind::Conversation, 18_000.0),
        block("tool", "Tool output", BlockKind::ToolOutput, 9_000.0),
    ];
    let compaction_record = CompactionRecord {
        pre_blocks: vec![
            block("ins", "Instructions", BlockKind::Instructions, 4_000.0),
            block("doc", "Spec document", BlockKind::Document, 26_000.0),
            block("conv", "Conversation", BlockKind::Conversation, 64_000.0),
            block("tool", "Tool output", BlockKind::ToolOutput, 30_000.0),
        ],
        folded_keys: vec!["conv".into()],
        removed_keys: vec!["tool".into()],
        digest_text: Some(
            "Older chat folded into a summary; one large tool output dropped.".into(),
        ),
    };
    rsx! {
        Item { slug: "context-treemap", name: "ContextTreemap",
            ContextTreemap {
                block_list: blocks,
                compaction_record: Some(compaction_record),
                window_size: 200_000.0,
                model_name: "Model A",
                agent_options: vec![AgentOption { agent_id: "a".into(), agent_name: "Writer".into(), window_size: 200_000.0 }],
                active_agent_id: Some("a".to_string()),
                active_view: Some(view()),
                on_view_change: move |v: View| {
                    last.set(format!("view {v:?}"));
                    view.set(v);
                },
                on_block_pick: move |b: Option<Block>| last.set(format!("pick {}", b.map(|x| x.block_key).unwrap_or("none".into()))),
                status_text: Some("Estimated".to_string()),
            }
            div { class: "gl-out", "data-out": "context-treemap", "{last()}" }
        }
    }
}

#[component]
fn DailyUsageBarsDemo() -> Element {
    use rv2_kit::components::daily_usage_bars::{DailyUsageBars, DayValue, Figure};
    let days: Vec<DayValue> = (1..=10)
        .map(|d: u32| DayValue {
            day_label: format!("Nov {d}"),
            day_value: Some((d * 37 % 90 + 10) as f64),
        })
        .collect();
    rsx! {
        Item { slug: "daily-usage-bars", name: "DailyUsageBars",
            DailyUsageBars {
                chart_heading: "Daily usage",
                day_values: days,
                total_figures: vec![Figure { figure_label: "Total".into(), figure_value: 1_284.0 }, Figure { figure_label: "Average".into(), figure_value: 128.4 }],
                period_text: "Last 10 days",
                is_today_live: true,
                live_badge_text: "Live",
            }
        }
    }
}

#[component]
fn MetricTileDemo() -> Element {
    use rv2_kit::components::metric_tile::{MetricTile, MetricValue};
    let mut last = use_signal(String::new);
    let mut chosen = use_signal(|| false);
    rsx! {
        Item { slug: "metric-tile", name: "MetricTile",
            div { class: "gl-row", style: "align-items: stretch;",
                MetricTile {
                    metric_label: "Open tasks",
                    metric_value: MetricValue::Number(42.0),
                    change_percent: Some(12.5),
                    change_note: Some("vs last week".to_string()),
                    is_chosen: Some(chosen()),
                    on_activate: move |_| {
                        chosen.set(!chosen());
                        last.set(format!("tile chosen {}", chosen()));
                    },
                }
                MetricTile {
                    metric_label: "Median latency",
                    metric_value: MetricValue::Text("1.2 s".to_string()),
                    change_percent: Some(-4.0),
                    lower_is_better: true,
                }
                MetricTile {
                    metric_label: "Cost",
                    metric_value: MetricValue::Number(0.0),
                    is_loading: true,
                }
            }
            div { class: "gl-out", "data-out": "metric-tile", "{last()}" }
        }
    }
}

#[component]
fn MetricTrendCardDemo() -> Element {
    use rv2_kit::components::metric_trend_card::{MetricSeries, MetricTrendCard};
    use rv2_kit::components::series_trend_chart::SeriesTone;
    let labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .map(|s| s.to_string())
        .to_vec();
    rsx! {
        Item { slug: "metric-trend-card", name: "MetricTrendCard",
            MetricTrendCard {
                metric_name: "Signups",
                headline_value: "1,284",
                change_value: 8.2,
                axis_labels: labels,
                data_series: vec![MetricSeries {
                    series_name: "This week".into(),
                    series_tone: SeriesTone::Ok,
                    series_values: vec![120.0, 180.0, 160.0, 210.0, 190.0, 240.0, 260.0],
                }],
                footer_text: Some("Updated just now".to_string()),
            }
        }
    }
}

#[component]
fn PartOfTotalCardDemo() -> Element {
    use rv2_kit::components::part_of_total_card::{Part, PartOfTotalCard, PeriodOption};
    use rv2_kit::components::series_trend_chart::SeriesTone;
    let mut last = use_signal(String::new);
    let mut period = use_signal(|| Some("month".to_string()));
    let part = |key: &str, name: &str, amount: f64, tone: SeriesTone| Part {
        part_key: Some(key.into()),
        part_name: name.into(),
        part_amount: amount,
        part_color: Some(tone),
    };
    rsx! {
        Item { slug: "part-of-total-card", name: "PartOfTotalCard",
            PartOfTotalCard {
                part_entries: vec![part("api", "API", 4_200.0, SeriesTone::Ink), part("seats", "Seats", 1_800.0, SeriesTone::Ok), part("storage", "Storage", 600.0, SeriesTone::Warn)],
                card_title: "Spend by line",
                period_options: vec![PeriodOption { period_key: "month".into(), period_name: "This month".into() }, PeriodOption { period_key: "quarter".into(), period_name: "This quarter".into() }],
                active_period: period(),
                on_period_change: move |k: String| {
                    last.set(format!("period {k}"));
                    period.set(Some(k));
                },
                change_pct: Some(6.4),
            }
            div { class: "gl-out", "data-out": "part-of-total-card", "{last()}" }
        }
    }
}

#[component]
fn ProviderUsageChartDemo() -> Element {
    use rv2_kit::components::provider_usage_chart::{
        Measure, ModelUsage, ProviderGroup, ProviderUsageChart, Usage,
    };
    use rv2_kit::components::series_trend_chart::SeriesTone;
    use std::collections::BTreeMap;
    let usage = |t: f64, c: f64, r: f64| Usage {
        tokens: Some(t),
        cost_usd: Some(c),
        request_count: Some(r),
    };
    let groups = vec![
        ProviderGroup {
            group_name: "Provider A".into(),
            group_tone: Some(SeriesTone::Ink),
            group_logo: None,
            models: vec![ModelUsage {
                model_name: "Model A1".into(),
                usage: usage(820_000.0, 12.4, 310.0),
                task_usage: BTreeMap::from([
                    ("Summaries".to_string(), usage(500_000.0, 7.9, 180.0)),
                    ("Drafting".to_string(), usage(320_000.0, 4.5, 130.0)),
                ]),
            }],
        },
        ProviderGroup {
            group_name: "Provider B".into(),
            group_tone: Some(SeriesTone::Ok),
            group_logo: None,
            models: vec![
                ModelUsage {
                    model_name: "Model B1".into(),
                    usage: usage(440_000.0, 5.1, 120.0),
                    task_usage: BTreeMap::from([(
                        "Search".to_string(),
                        usage(440_000.0, 5.1, 120.0),
                    )]),
                },
                ModelUsage {
                    model_name: "Model B2".into(),
                    usage: usage(90_000.0, 0.7, 40.0),
                    task_usage: BTreeMap::new(),
                },
            ],
        },
    ];
    rsx! {
        Item { slug: "provider-usage-chart", name: "ProviderUsageChart",
            ProviderUsageChart {
                provider_groups: groups,
                measure_options: vec![Measure::Tokens, Measure::Cost, Measure::Requests],
                period_label: "This week",
                prior_totals: Some(usage(900_000.0, 11.0, 300.0)),
                prior_label: Some("Last week".to_string()),
            }
        }
    }
}

#[component]
fn QuotaUsagePanelDemo() -> Element {
    use rv2_kit::components::quota_usage_panel::{
        ActionState, CapMeter, DayPoint, PeriodChoice, QuotaUsagePanel, SpendPart,
    };
    use rv2_kit::components::series_trend_chart::SeriesTone;
    let mut last = use_signal(String::new);
    rsx! {
        Item { slug: "quota-usage-panel", name: "QuotaUsagePanel",
            QuotaUsagePanel {
                plan_total: 500.0,
                plan_label: "Team plan",
                spend_breakdown: vec![
                    SpendPart { spend_label: "Chat".into(), spend_value: 180.0, spend_emphasis: Some(SeriesTone::Ink) },
                    SpendPart { spend_label: "Agents".into(), spend_value: 240.0, spend_emphasis: Some(SeriesTone::Warn) },
                ],
                cap_list: vec![CapMeter { cap_label: "Monthly".into(), cap_used: 420.0, cap_total: Some(500.0), unit: Some("credits".into()), decimals: Some(0) }],
                period_choices: vec![PeriodChoice { period_id: "m".into(), period_label: "November".into() }, PeriodChoice { period_id: "o".into(), period_label: "October".into() }],
                shown_period: Some("m".to_string()),
                resets_at_ms: Some(NOW_MS as i64 + 86_400_000 * 9),
                now_ms: Some(gallery_now() as i64),
                daily_trend: (1..=12).map(|d: u32| DayPoint { day_label: format!("{d}"), day_value: Some((d * 13 % 40) as f64) }).collect(),
                on_action: move |a: ActionState| last.set(format!("action {a:?}")),
                on_period_pick: move |p: String| last.set(format!("period {p}")),
            }
            div { class: "gl-out", "data-out": "quota-usage-panel", "{last()}" }
        }
    }
}

#[component]
fn RankingBracketDemo() -> Element {
    use rv2_kit::components::ranking_bracket::{Entrant, RankingBracket, Verdict};
    let mut last = use_signal(String::new);
    let e = |id: &str, name: &str| Entrant {
        entrant_id: id.into(),
        entrant_name: name.into(),
        entrant_mark: None,
        entrant_meta: Some("model A".into()),
        entrant_text: Some(format!("Draft from {name}")),
    };
    rsx! {
        Item { slug: "ranking-bracket", name: "RankingBracket",
            RankingBracket {
                entrants: vec![e("a", "Draft A"), e("b", "Draft B"), e("c", "Draft C"), e("d", "Draft D")],
                verdicts: vec![
                    ("0:0".to_string(), Verdict { winner_id: "a".into(), rationale: Some("Clearer structure".into()), scores: Some((8.0, 6.5)) }),
                    ("0:1".to_string(), Verdict { winner_id: "d".into(), rationale: Some("Tighter argument".into()), scores: Some((5.0, 7.0)) }),
                    ("1:0".to_string(), Verdict { winner_id: "d".into(), rationale: Some("Best of the four".into()), scores: Some((6.0, 8.5)) }),
                ],
                pairing_in_flight: None,
                judge_label: "Judge",
                brief: "Pick the clearest draft",
                on_adopt: move |en: Entrant| last.set(format!("adopt {}", en.entrant_id)),
            }
            div { class: "gl-out", "data-out": "ranking-bracket", "{last()}" }
        }
    }
}

#[component]
fn CandidateRaceBoardDemo() -> Element {
    use rv2_kit::components::candidate_race_board::{Candidate, CandidateRaceBoard, RunState};
    let mut last = use_signal(String::new);
    let c = |key: &str, name: &str, state: RunState, frac: Option<f64>| Candidate {
        quality_score: (state == RunState::Finished).then_some(72.0),
        candidate_key: key.into(),
        display_name: name.into(),
        method_summary: "Sampled at low temperature".into(),
        model_id: Some("model-a".into()),
        mark: None,
        run_state: state,
        fraction_done: frac,
        current_action: Some("Scoring".into()),
        outcome_note: None,
        elapsed_secs: Some(12.4),
        run_cost: Some(0.04),
    };
    rsx! {
        Item { slug: "candidate-race-board", name: "CandidateRaceBoard",
            CandidateRaceBoard {
                candidate_list: vec![c("x", "Approach X", RunState::Finished, Some(1.0)), c("y", "Approach Y", RunState::Running, Some(0.6)), c("z", "Approach Z", RunState::Halted, Some(0.2))],
                task_text: "Refactor the parser",
                board_title: Some("Candidates".to_string()),
                on_choose: move |cand: Candidate| last.set(format!("choose {}", cand.candidate_key)),
            }
            div { class: "gl-out", "data-out": "candidate-race-board", "{last()}" }
        }
    }
}

#[component]
fn RunCostBreakdownDemo() -> Element {
    use rv2_kit::components::run_cost_breakdown::{
        Measure, RunCostBreakdown, RunState, Step, StepKind, StepStatus,
    };
    let mut last = use_signal(String::new);
    let step = |id: &str,
                name: &str,
                kind: StepKind,
                status: StepStatus,
                cost: f64,
                children: Vec<Step>| Step {
        step_id: id.into(),
        step_name: name.into(),
        step_kind: kind,
        step_status: status,
        model_name: Some("model A".into()),
        step_note: None,
        in_tokens: Some(4_200.0),
        out_tokens: Some(860.0),
        step_cost: Some(cost),
        step_elapsed: Some(3.2),
        prompt_text: Some("Summarise the thread".into()),
        result_text: Some("Three points.".into()),
        error_text: None,
        child_steps: children,
    };
    let steps = vec![step(
        "root",
        "Summarise thread",
        StepKind::Agent,
        StepStatus::Done,
        0.09,
        vec![
            step(
                "m1",
                "Read messages",
                StepKind::ToolCall,
                StepStatus::Done,
                0.01,
                vec![],
            ),
            step(
                "m2",
                "Write summary",
                StepKind::ModelCall,
                StepStatus::Failed,
                0.08,
                vec![],
            ),
        ],
    )];
    rsx! {
        Item { slug: "run-cost-breakdown", name: "RunCostBreakdown",
            RunCostBreakdown {
                step_list: steps,
                run_label: "Summary run",
                run_summary: Some("2 steps, 1 failed".to_string()),
                status_override: Some(RunState::Failed),
                starting_measure: Measure::Cost,
                on_measure_change: move |m: Measure| last.set(format!("measure {m:?}")),
                on_fold_change: move |ids: Vec<String>| last.set(format!("folded {}", ids.len())),
                on_detail_change: move |(id, _): (Option<String>, Option<Step>)| last.set(format!("detail {id:?}")),
                starting_detail_step_id: Some("m2".to_string()),
            }
            div { class: "gl-out", "data-out": "run-cost-breakdown", "{last()}" }
        }
    }
}

#[component]
fn SegmentedBarExplorerDemo() -> Element {
    use rv2_kit::components::segmented_bar_explorer::{
        GroupingAxis, MetricDef, SegmentedBarExplorer, TallyRow,
    };
    use std::collections::BTreeMap;
    let row = |team: &str, status: &str, tickets: f64, hours: f64| TallyRow {
        axis: BTreeMap::from([
            ("team".to_string(), team.to_string()),
            ("status".to_string(), status.to_string()),
        ]),
        metrics: BTreeMap::from([
            ("tickets".to_string(), tickets),
            ("hours".to_string(), hours),
        ]),
    };
    let rows = vec![
        row("Growth", "open", 14.0, 6.0),
        row("Growth", "closed", 22.0, 9.5),
        row("Platform", "open", 8.0, 12.0),
        row("Platform", "closed", 31.0, 15.0),
        row("Support", "open", 19.0, 4.0),
    ];
    rsx! {
        Item { slug: "segmented-bar-explorer", name: "SegmentedBarExplorer",
            SegmentedBarExplorer {
                tally_rows: rows,
                grouping_axes: vec![
                    GroupingAxis { axis_key: "team".into(), axis_label: "Team".into(), axis_values: vec!["Growth".into(), "Platform".into(), "Support".into()] },
                    GroupingAxis { axis_key: "status".into(), axis_label: "Status".into(), axis_values: vec!["open".into(), "closed".into()] },
                ],
                metric_defs: vec![
                    MetricDef { metric_key: "tickets".into(), metric_label: "Tickets".into(), unit: "".into(), source_field: Some("tickets".into()) },
                    MetricDef { metric_key: "hours".into(), metric_label: "Hours".into(), unit: "h".into(), source_field: Some("hours".into()) },
                ],
                heading_text: "Work by team",
                start_metric: "tickets",
                start_axis: "team",
                start_stack: "status",
            }
        }
    }
}

#[component]
fn SeriesTrendChartDemo() -> Element {
    use rv2_kit::components::series_trend_chart::{
        ChartVariant, SeriesTone, SeriesTrendChart, TrendSeries,
    };
    let mut last = use_signal(String::new);
    let labels = ["W1", "W2", "W3", "W4", "W5", "W6"]
        .map(|s| s.to_string())
        .to_vec();
    let series = vec![
        TrendSeries {
            name: "Completed".into(),
            values: vec![
                Some(12.0),
                Some(18.0),
                Some(15.0),
                None,
                Some(24.0),
                Some(28.0),
            ],
            tone: SeriesTone::Ok,
            emphasis: true,
            is_earlier: false,
        },
        TrendSeries {
            name: "Opened".into(),
            values: vec![
                Some(14.0),
                Some(11.0),
                Some(17.0),
                Some(16.0),
                Some(13.0),
                Some(15.0),
            ],
            tone: SeriesTone::Muted,
            emphasis: false,
            is_earlier: true,
        },
    ];
    rsx! {
        Item { slug: "series-trend-chart", name: "SeriesTrendChart",
            SeriesTrendChart {
                x_labels: labels,
                series,
                variant: ChartVariant::Line,
                label: "Tickets per week",
                height: Some(220.0),
                on_active_change: move |i: Option<usize>| last.set(format!("active {i:?}")),
            }
            div { class: "gl-out", "data-out": "series-trend-chart", "{last()}" }
        }
    }
}

#[component]
fn VariantTestCardDemo() -> Element {
    use rv2_kit::components::series_trend_chart::SeriesTone;
    use rv2_kit::components::variant_test_card::{
        ActionLabel, Arm, Metric, ResultAction, Reveal, StatusChip, VariantTestCard,
    };
    use std::collections::BTreeMap;
    let mut last = use_signal(String::new);
    let mut choice = use_signal(|| None::<String>);
    let arm = |id: &str, letter: char, name: &str, control: bool, rate: f64| Arm {
        arm_id: id.into(),
        arm_letter: letter,
        arm_name: name.into(),
        readings: BTreeMap::from([("conversion".to_string(), rate)]),
        is_control: control,
        sample_size: Some(1_200),
        confidence: BTreeMap::from([("conversion".to_string(), 0.93)]),
        badge_tone: Some(SeriesTone::Ok),
    };
    rsx! {
        Item { slug: "variant-test-card", name: "VariantTestCard",
            VariantTestCard {
                test_name: "Pricing page copy",
                arms: vec![arm("a", 'A', "Current copy", true, 0.042), arm("b", 'B', "Short copy", false, 0.051)],
                metric_options: vec![Metric { metric_id: "conversion".into(), metric_label: "Conversion".into(), format: Some(|v| format!("{:.1}%", v * 100.0)), lower_wins: false }],
                starting_metric: Some("conversion".to_string()),
                chosen_arm: Some(choice()),
                on_choose: move |c: Option<String>| {
                    last.set(format!("choose {c:?}"));
                    choice.set(c);
                },
                status_chip: Some(StatusChip { chip_text: "Significant".into(), chip_tone: SeriesTone::Ok }),
                status_aside: Some("93% confidence".to_string()),
                result_action: Some(ResultAction { action_text: ActionLabel::Static("Ship winner".into()), on_run: EventHandler::new(move |a: Arm| last.set(format!("ship {}", a.arm_id))) }),
                reveal: Reveal::Tap,
            }
            div { class: "gl-out", "data-out": "variant-test-card", "{last()}" }
        }
    }
}
