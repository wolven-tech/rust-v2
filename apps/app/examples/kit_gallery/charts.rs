use dioxus::prelude::*;

use super::{Item, Section, gallery_now};

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
            part_color: Some(SeriesTone::Muted),
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
        ContextPart {
            part_key: "tools".into(),
            part_name: "Tool results".into(),
            token_count: Some(21_500.0),
            sub_items: vec![
                ContextItem {
                    item_key: "search".into(),
                    item_name: "Web search".into(),
                    item_tokens: 14_000.0,
                },
                ContextItem {
                    item_key: "sql".into(),
                    item_name: "SQL result".into(),
                    item_tokens: 7_500.0,
                },
            ],
            is_locked: false,
            can_compact: true,
            part_color: Some(SeriesTone::Ok),
        },
        ContextPart {
            part_key: "chat".into(),
            part_name: "Conversation".into(),
            token_count: Some(36_000.0),
            sub_items: vec![],
            is_locked: false,
            can_compact: true,
            part_color: Some(SeriesTone::Ink),
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
    let days: Vec<DayValue> = (1..=21)
        .map(|d: u32| DayValue {
            day_label: format!("Nov {d}"),
            day_value: Some((d * 37 % 90 + 10) as f64),
        })
        .collect();
    let total: f64 = days.iter().filter_map(|d| d.day_value).sum();
    let average = total / days.len() as f64;
    rsx! {
        Item { slug: "daily-usage-bars", name: "DailyUsageBars",
            DailyUsageBars {
                chart_heading: "Daily usage",
                day_values: days,
                total_figures: vec![Figure { figure_label: "Total".into(), figure_value: total }, Figure { figure_label: "Average".into(), figure_value: average }],
                period_text: "Nov 1 – Nov 21",
                is_today_live: true,
            }
        }
    }
}

#[component]
fn MetricTileDemo() -> Element {
    use rv2_kit::components::metric_tile::{MetricTile, MetricTileGroup, MetricValue};
    let mut last = use_signal(String::new);
    let mut period = use_signal(|| 0usize);
    let line = || {
        rsx! {
            svg { view_box: "0 0 240 56", preserve_aspect_ratio: "none", fill: "none", stroke: "currentColor", "stroke-width": "1.6", "aria-hidden": "true",
                path { d: "M0 44 L40 36 L80 40 L120 24 L160 28 L200 14 L240 16" }
            }
        }
    };
    let columns = || {
        rsx! {
            svg { view_box: "0 0 240 56", preserve_aspect_ratio: "none", fill: "currentColor", "aria-hidden": "true",
                rect { x: "4", y: "30", width: "22", height: "26" }
                rect { x: "38", y: "22", width: "22", height: "34" }
                rect { x: "72", y: "34", width: "22", height: "22" }
                rect { x: "106", y: "18", width: "22", height: "38" }
                rect { x: "140", y: "26", width: "22", height: "30" }
                rect { x: "174", y: "12", width: "22", height: "44" }
                rect { x: "208", y: "20", width: "22", height: "36" }
            }
        }
    };
    rsx! {
        Item { slug: "metric-tile", name: "MetricTile",
            MetricTileGroup {
                group_label: "Summary",
                header_control: Some(rsx! {
                    div { class: "kit-tile__segments", role: "group", "aria-label": "Period",
                        button { r#type: "button", class: "kit-tile__segment", "aria-pressed": if period() == 0 { "true" } else { "false" }, onclick: move |_| { period.set(0); last.set("7 days".into()); }, "7 days" }
                        button { r#type: "button", class: "kit-tile__segment", "aria-pressed": if period() == 1 { "true" } else { "false" }, onclick: move |_| { period.set(1); last.set("30 days".into()); }, "30 days" }
                    }
                }),
                MetricTile {
                    metric_label: "Open tasks",
                    metric_mark: Some(rsx! {
                        svg { width: "14", height: "14", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", "stroke-width": "1.6", "stroke-linecap": "round",
                            path { d: "M2.5 4.5l1.5 1.5 2.5-2.5M2.5 11.5l1.5 1.5 2.5-2.5M9 4.5h4.5M9 11.5h4.5" }
                        }
                    }),
                    metric_value: MetricValue::Number(42.0),
                    change_percent: Some(12.5),
                    change_note: Some("against the prior seven days".to_string()),
                    inline_chart: Some(line()),
                    footer_content: Some(rsx! { "Read 23 Nov 2026" }),
                    on_activate: move |_| last.set("open tasks".into()),
                }
                MetricTile {
                    metric_label: "Median latency",
                    metric_mark: Some(rsx! {
                        svg { width: "14", height: "14", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", "stroke-width": "1.6",
                            circle { cx: "8", cy: "8", r: "5.5" }
                            path { d: "M8 5v3.2l2 1.3" }
                        }
                    }),
                    metric_value: MetricValue::Text("1.2 s".to_string()),
                    change_percent: Some(-4.0),
                    change_note: Some("against the prior seven days".to_string()),
                    lower_is_better: true,
                    inline_chart: Some(line()),
                    footer_content: Some(rsx! { "Read 23 Nov 2026" }),
                }
                MetricTile {
                    metric_label: "Cost",
                    metric_value: MetricValue::Text("$18.40".to_string()),
                    change_percent: Some(7.8),
                    change_note: Some("against the prior seven days".to_string()),
                    lower_is_better: true,
                    inline_chart: Some(columns()),
                    footer_content: Some(rsx! { "Read 23 Nov 2026" }),
                }
                MetricTile {
                    metric_label: "Failed runs",
                    metric_value: MetricValue::Number(3.0),
                    change_percent: Some(-25.0),
                    change_note: Some("against the prior seven days".to_string()),
                    lower_is_better: true,
                    inline_chart: Some(line()),
                    footer_content: Some(rsx! { "Read 23 Nov 2026" }),
                }
            }
            MetricTileGroup { group_label: "Loading example",
                MetricTile {
                    metric_label: "Cost",
                    metric_value: MetricValue::Number(0.0),
                    is_loading: true,
                    inline_chart: Some(line()),
                }
            }
            div { class: "gl-out", "data-out": "metric-tile", "{last()}" }
        }
    }
}

#[component]
fn MetricTrendCardDemo() -> Element {
    use rv2_kit::components::metric_trend_card::{MetricSeries, MetricTrendCard};
    use rv2_kit::components::series_trend_chart::{SeriesTone, compact_number};
    fn figure(n: f64) -> String {
        if n.fract() == 0.0 {
            compact_number(n)
        } else {
            format!("{n:.1}")
        }
    }
    let labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        .map(|s| s.to_string())
        .to_vec();
    let series = || {
        vec![
            MetricSeries {
                series_name: "Trials".into(),
                series_tone: SeriesTone::Ink,
                series_values: vec![120.0, 150.0, 140.0, 180.0, 170.0, 200.0, 210.0],
            },
            MetricSeries {
                series_name: "Upgrades".into(),
                series_tone: SeriesTone::Ok,
                series_values: vec![40.0, 52.0, 44.0, 60.0, 58.0, 66.0, 72.0],
            },
            MetricSeries {
                series_name: "Renewals".into(),
                series_tone: SeriesTone::Warn,
                series_values: vec![20.0, 24.0, 30.0, 28.0, 33.0, 35.0, 41.0],
            },
        ]
    };
    rsx! {
        Item { slug: "metric-trend-card", name: "MetricTrendCard",
            MetricTrendCard {
                metric_name: "New accounts",
                headline_value: "1,284",
                change_value: 8.1,
                change_suffix: "%",
                axis_labels: labels.clone(),
                data_series: series(),
                format_value: figure as fn(f64) -> String,
                footer_text: Some("Refreshed from the nightly load.".to_string()),
            }
        }
        Item { slug: "metric-trend-card-negative", name: "MetricTrendCard (negative change)",
            MetricTrendCard {
                metric_name: "New accounts",
                headline_value: "1,284",
                change_value: -3.2,
                change_suffix: "%",
                axis_labels: labels.clone(),
                data_series: series(),
                format_value: figure as fn(f64) -> String,
                footer_text: Some("Refreshed from the nightly load.".to_string()),
            }
        }
        Item { slug: "metric-trend-card-empty", name: "MetricTrendCard (empty)",
            MetricTrendCard {
                metric_name: "New accounts",
                headline_value: "0",
                change_value: 0.0,
                change_suffix: "%",
                axis_labels: labels,
                data_series: Vec::<MetricSeries>::new(),
                format_value: figure as fn(f64) -> String,
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
                part_entries: vec![part("api", "API", 4_200.0, SeriesTone::Ink), part("seats", "Seats", 1_800.0, SeriesTone::Ok), part("storage", "Storage", 600.0, SeriesTone::Warn), part("support", "Support", 340.0, SeriesTone::Fault)],
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
        Measure, ModelUsage, ProviderGroup, ProviderUsageChart, TaskKind, Usage,
    };
    use rv2_kit::components::series_trend_chart::SeriesTone;
    use std::collections::BTreeMap;
    let usage = |tokens: f64| Usage {
        tokens: Some(tokens),
        cost_usd: Some(tokens / 100_000.0),
        request_count: Some(tokens / 1_000.0),
    };
    let model = |name: &str, tokens: f64, tasks: &[(&str, f64)]| ModelUsage {
        model_name: name.into(),
        usage: usage(tokens),
        task_usage: tasks
            .iter()
            .map(|(kind, t)| (kind.to_string(), usage(*t)))
            .collect(),
    };
    let group = |name: &str, tone: SeriesTone, models: Vec<ModelUsage>| ProviderGroup {
        group_name: name.into(),
        group_tone: Some(tone),
        group_logo: None,
        models,
    };
    let groups = vec![
        group(
            "Alder",
            SeriesTone::Ink,
            vec![
                model(
                    "Alder Large",
                    1_400_000.0,
                    &[
                        ("search", 600_000.0),
                        ("analyse", 500_000.0),
                        ("code", 300_000.0),
                    ],
                ),
                model(
                    "Alder Small",
                    600_000.0,
                    &[("code", 400_000.0), ("test", 200_000.0)],
                ),
                model("Alder Mini", 200_000.0, &[("search", 200_000.0)]),
            ],
        ),
        group(
            "Birch",
            SeriesTone::Ok,
            vec![
                model(
                    "Birch Pro",
                    900_000.0,
                    &[
                        ("analyse", 400_000.0),
                        ("code", 300_000.0),
                        ("test", 200_000.0),
                    ],
                ),
                model("Birch Lite", 300_000.0, &[("search", 300_000.0)]),
            ],
        ),
        group(
            "Cedar",
            SeriesTone::Warn,
            vec![
                model(
                    "Cedar One",
                    700_000.0,
                    &[
                        ("search", 250_000.0),
                        ("analyse", 250_000.0),
                        ("test", 200_000.0),
                    ],
                ),
                model("Cedar Two", 500_000.0, &[("code", 500_000.0)]),
                model("Cedar Edge", 100_000.0, &[("test", 100_000.0)]),
            ],
        ),
        group(
            "Dune",
            SeriesTone::Fault,
            vec![
                model(
                    "Dune Core",
                    450_000.0,
                    &[("analyse", 250_000.0), ("code", 200_000.0)],
                ),
                model("Dune Fast", 50_000.0, &[("search", 50_000.0)]),
            ],
        ),
    ];
    let kind = |label: &str, tone: SeriesTone| TaskKind {
        kind_label: label.into(),
        kind_tone: tone,
        kind_icon: None,
    };
    let task_kinds = BTreeMap::from([
        ("search".to_string(), kind("Search", SeriesTone::Ink)),
        ("analyse".to_string(), kind("Analyse", SeriesTone::Ok)),
        ("code".to_string(), kind("Code", SeriesTone::Warn)),
        ("test".to_string(), kind("Test", SeriesTone::Fault)),
    ]);
    rsx! {
        Item { slug: "provider-usage-chart", name: "ProviderUsageChart",
            ProviderUsageChart {
                provider_groups: groups.clone(),
                measure_options: vec![Measure::Tokens, Measure::Cost, Measure::Requests],
                starting_measure: Measure::Tokens,
                period_label: "This week",
                prior_totals: Some(usage(4_600_000.0)),
                prior_label: Some("last week".to_string()),
                task_kinds: task_kinds.clone(),
            }
        }
        Item { slug: "provider-usage-chart-engaged", name: "ProviderUsageChart (provider engaged)",
            ProviderUsageChart {
                provider_groups: groups,
                measure_options: vec![Measure::Tokens, Measure::Cost, Measure::Requests],
                starting_measure: Measure::Tokens,
                starting_group: Some(1),
                period_label: "This week",
                prior_totals: Some(usage(4_600_000.0)),
                prior_label: Some("last week".to_string()),
                task_kinds,
            }
        }
    }
}

#[component]
fn QuotaUsagePanelDemo() -> Element {
    use rv2_kit::components::quota_usage_panel::{
        ActionState, CapMeter, DayPoint, PeriodChoice, QuotaUsagePanel, SpendPart,
    };
    let mut last = use_signal(String::new);
    let now = gallery_now() as i64;
    let day_values: [Option<f64>; 14] = [
        Some(18.0),
        Some(22.0),
        Some(15.0),
        Some(27.0),
        Some(31.0),
        Some(19.0),
        Some(24.0),
        Some(33.0),
        Some(17.0),
        Some(29.0),
        Some(21.0),
        None,
        None,
        None,
    ];
    let daily: Vec<DayPoint> = day_values
        .iter()
        .enumerate()
        .map(|(i, day_value)| DayPoint {
            day_label: format!("{}", i + 1),
            day_value: *day_value,
        })
        .collect();
    rsx! {
        Item { slug: "quota-usage-panel", name: "QuotaUsagePanel",
            QuotaUsagePanel {
                panel_heading: "Studio usage",
                plan_label: "Studio plan",
                meter_unit: "jobs",
                plan_total: 500.0,
                headroom_action: "Raise limit",
                spend_breakdown: vec![
                    SpendPart { spend_label: "Batch extraction".into(), spend_value: 150.0, spend_emphasis: None },
                    SpendPart { spend_label: "Live answers".into(), spend_value: 120.0, spend_emphasis: None },
                    SpendPart { spend_label: "Index rebuilds".into(), spend_value: 90.0, spend_emphasis: None },
                    SpendPart { spend_label: "Exports".into(), spend_value: 60.0, spend_emphasis: None },
                ],
                cap_list: vec![
                    CapMeter { cap_label: "Members".into(), cap_used: 7.0, cap_total: Some(12.0), unit: None, decimals: Some(0) },
                    CapMeter { cap_label: "Archive".into(), cap_used: 2.4, cap_total: Some(8.0), unit: Some("GB".into()), decimals: Some(1) },
                    CapMeter { cap_label: "Shared spaces".into(), cap_used: 31.0, cap_total: None, unit: None, decimals: Some(0) },
                ],
                period_choices: vec![PeriodChoice { period_id: "cycle".into(), period_label: "Cycle to date".into() }, PeriodChoice { period_id: "previous".into(), period_label: "Previous cycle".into() }],
                shown_period: Some("cycle".to_string()),
                resets_at_ms: Some(now + 774_000_000),
                now_ms: Some(now),
                daily_trend: daily,
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
    let e = |id: &str, name: &str, meta: &str, text: &str| Entrant {
        entrant_id: id.into(),
        entrant_name: name.into(),
        entrant_mark: None,
        entrant_meta: Some(meta.into()),
        entrant_text: Some(text.into()),
    };
    let entrants = || {
        vec![
            e(
                "a",
                "Dedupe table",
                "42 lines · 3 files",
                "INSERT INTO charges (key) ON CONFLICT DO NOTHING",
            ),
            e(
                "b",
                "Outbox",
                "88 lines · 5 files",
                "Write the intent, publish it from a relay",
            ),
            e(
                "c",
                "Queue retry",
                "31 lines · 2 files",
                "Re-enqueue the charge with backoff",
            ),
            e(
                "d",
                "Token guard",
                "27 lines · 2 files",
                "Reject a charge whose token was used",
            ),
            e(
                "e",
                "Client retry",
                "54 lines · 4 files",
                "Retry the gateway call with the same key",
            ),
            e(
                "f",
                "Manual review",
                "12 lines · 1 file",
                "Hold retried payments for a person",
            ),
            e(
                "g",
                "Saga",
                "120 lines · 7 files",
                "Compensate the charge when a step fails",
            ),
            e(
                "h",
                "Lock per order",
                "38 lines · 3 files",
                "Take a row lock on the order before charging",
            ),
        ]
    };
    let v = |winner: &str, rationale: &str, upper: f64, lower: f64| Verdict {
        winner_id: winner.into(),
        rationale: Some(rationale.into()),
        scores: Some((upper, lower)),
    };
    let quarterfinals = vec![
        (
            "0:0".to_string(),
            v("a", "Checks the key before the charge call.", 82.0, 74.0),
        ),
        (
            "0:1".to_string(),
            v(
                "d",
                "Smallest change that stops the second charge.",
                61.0,
                77.0,
            ),
        ),
        (
            "0:2".to_string(),
            v(
                "e",
                "Automatic, and covered by the existing retry test.",
                70.0,
                45.0,
            ),
        ),
        (
            "0:3".to_string(),
            v(
                "h",
                "The second attempt waits on the order row.",
                58.0,
                80.0,
            ),
        ),
    ];
    let mut in_progress = quarterfinals.clone();
    in_progress.push((
        "1:0".to_string(),
        v(
            "a",
            "Dedupe covers the retry window the guard misses.",
            85.0,
            79.0,
        ),
    ));
    let mut complete = in_progress.clone();
    complete.push((
        "1:1".to_string(),
        v("h", "Serializes the charge per order.", 71.0, 68.0),
    ));
    complete.push((
        "2:0".to_string(),
        v("a", "Dedupe stops duplicates before any write.", 90.0, 76.0),
    ));
    let brief = "Stop duplicate charges when a payment is retried.";
    rsx! {
        Item { slug: "ranking-bracket", name: "RankingBracket",
            RankingBracket {
                entrants: entrants(),
                verdicts: in_progress,
                pairing_in_flight: Some("1:1".to_string()),
                open_pairing: Some("0:1".to_string()),
                judge_label: "Judge model B".to_string(),
                brief: brief.to_string(),
            }
        }
        Item { slug: "ranking-bracket-complete", name: "RankingBracket",
            RankingBracket {
                entrants: entrants(),
                verdicts: complete,
                judge_label: "Judge model B".to_string(),
                brief: brief.to_string(),
                on_adopt: move |en: Entrant| last.set(format!("adopt {}", en.entrant_id)),
            }
            div { class: "gl-out", "data-out": "ranking-bracket-complete", "{last()}" }
        }
    }
}

#[component]
fn CandidateRaceBoardDemo() -> Element {
    use rv2_kit::components::candidate_race_board::{Candidate, CandidateRaceBoard, RunState};
    let mut last = use_signal(String::new);
    let c = |key: &str,
             name: &str,
             model: &str,
             method: &str,
             state: RunState,
             frac: f64,
             score: Option<f64>,
             note: &str,
             cost: f64,
             secs: f64| Candidate {
        candidate_key: key.into(),
        display_name: name.into(),
        method_summary: method.into(),
        model_id: Some(model.into()),
        mark: None,
        run_state: state,
        fraction_done: Some(frac),
        current_action: Some(note.into()),
        quality_score: score,
        outcome_note: (state != RunState::Running).then(|| note.to_string()),
        elapsed_secs: Some(secs),
        run_cost: Some(cost),
    };
    rsx! {
        Item { slug: "candidate-race-board", name: "CandidateRaceBoard",
            CandidateRaceBoard {
                candidate_list: vec![
                    c("retry", "Retry queue", "model A", "Replays failed charges in order", RunState::Running, 0.2, None, "Calling the gateway", 0.03, 8.0),
                    c("mock", "Mock the gateway", "model B", "Stubs the payment API", RunState::Running, 0.45, None, "Writing fixtures", 0.05, 14.0),
                    c("settle", "Wait for settle", "model C", "Polls until the charge settles", RunState::Running, 0.7, None, "Checking status", 0.02, 21.0),
                ],
                task_text: "Stop duplicate charges when a payment is retried",
                board_title: Some("Best of 3".to_string()),
                on_choose: move |cand: Candidate| last.set(format!("choose {}", cand.candidate_key)),
            }
        }
        Item { slug: "candidate-race-board-mixed", name: "CandidateRaceBoard",
            CandidateRaceBoard {
                candidate_list: vec![
                    c("retry", "Retry queue", "model A", "Replays failed charges in order", RunState::Finished, 1.0, Some(72.0), "Passes all 14 checks", 0.07, 42.0),
                    c("settle", "Wait for settle", "model C", "Polls until the charge settles", RunState::Finished, 1.0, Some(64.0), "Passes 12 of 14 checks", 0.04, 38.0),
                    c("mock", "Mock the gateway", "model B", "Stubs the payment API", RunState::Halted, 0.35, None, "Error: fixture server refused connection", 0.02, 11.0),
                    c("clock", "Stub the clock", "model D", "Advances time instead of sleeping", RunState::Running, 0.6, None, "Advancing time", 0.03, 25.0),
                ],
                task_text: "Stop duplicate charges when a payment is retried",
                board_title: Some("Best of 4".to_string()),
                on_choose: move |cand: Candidate| last.set(format!("choose {}", cand.candidate_key)),
            }
            div { class: "gl-out", "data-out": "candidate-race-board-mixed", "{last()}" }
        }
    }
}

#[component]
fn RunCostBreakdownDemo() -> Element {
    use rv2_kit::components::run_cost_breakdown::{
        Measure, RunCostBreakdown, Step, StepKind, StepStatus,
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
    let plan_steps = vec![
        step(
            "plan",
            "Plan the report",
            StepKind::Agent,
            StepStatus::Done,
            0.04,
            vec![],
        ),
        step(
            "research",
            "Researcher",
            StepKind::Agent,
            StepStatus::Done,
            0.03,
            vec![step(
                "search",
                "Web search",
                StepKind::ToolCall,
                StepStatus::Done,
                0.03,
                vec![],
            )],
        ),
    ];
    let thread_steps = vec![
        step(
            "root",
            "Summarise thread",
            StepKind::Agent,
            StepStatus::Done,
            0.0,
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
        ),
        step(
            "root2",
            "Index documents",
            StepKind::Agent,
            StepStatus::Done,
            0.0,
            vec![
                step(
                    "i1",
                    "Fetch files",
                    StepKind::ToolCall,
                    StepStatus::Done,
                    0.0,
                    vec![],
                ),
                step(
                    "i2",
                    "Embed chunks",
                    StepKind::ModelCall,
                    StepStatus::Done,
                    0.03,
                    vec![],
                ),
            ],
        ),
    ];
    rsx! {
        Item { slug: "run-cost-breakdown", name: "RunCostBreakdown",
            RunCostBreakdown {
                step_list: plan_steps,
                run_label: "Plan run",
                run_summary: Some("Plan the report, then research it".to_string()),
                starting_measure: Measure::Cost,
                on_measure_change: move |m: Measure| last.set(format!("measure {m:?}")),
                on_fold_change: move |ids: Vec<String>| last.set(format!("folded {}", ids.len())),
                on_detail_change: move |(id, _): (Option<String>, Option<Step>)| last.set(format!("detail {id:?}")),
            }
            div { class: "gl-out", "data-out": "run-cost-breakdown", "{last()}" }
        }
        Item { slug: "run-cost-breakdown-failed", name: "RunCostBreakdown",
            RunCostBreakdown {
                step_list: thread_steps,
                run_label: "Summary run",
                run_summary: Some("2 steps, 1 failed".to_string()),
                starting_folded_step_ids: vec!["root2".to_string()],
                starting_detail_step_id: Some("m2".to_string()),
                on_measure_change: move |m: Measure| last.set(format!("measure {m:?}")),
                on_fold_change: move |ids: Vec<String>| last.set(format!("folded {}", ids.len())),
                on_detail_change: move |(id, _): (Option<String>, Option<Step>)| last.set(format!("detail {id:?}")),
            }
            div { class: "gl-out", "data-out": "run-cost-breakdown-failed", "{last()}" }
        }
    }
}

#[component]
fn SegmentedBarExplorerDemo() -> Element {
    use rv2_kit::components::segmented_bar_explorer::{
        GroupingAxis, MetricDef, SegmentedBarExplorer, TallyRow,
    };
    use std::collections::BTreeMap;
    const REGIONS: [&str; 4] = ["North", "South", "East", "West"];
    const STATUSES: [&str; 3] = ["Open", "In review", "Closed"];
    const PRIORITIES: [&str; 3] = ["High", "Medium", "Low"];
    let rows = || -> Vec<TallyRow> {
        (0..118usize)
            .map(|i| TallyRow {
                axis: BTreeMap::from([
                    (
                        "region".to_string(),
                        REGIONS[(i * 3 + i / 7) % 4].to_string(),
                    ),
                    ("status".to_string(), STATUSES[(i + i / 5) % 3].to_string()),
                    (
                        "priority".to_string(),
                        PRIORITIES[(i * 2 + i / 9) % 3].to_string(),
                    ),
                ]),
                metrics: BTreeMap::from([("hours".to_string(), (1 + (i * 37) % 23) as f64)]),
            })
            .collect()
    };
    let axes = || {
        vec![
            GroupingAxis {
                axis_key: "region".into(),
                axis_label: "Region".into(),
                axis_values: REGIONS.map(String::from).to_vec(),
            },
            GroupingAxis {
                axis_key: "status".into(),
                axis_label: "Status".into(),
                axis_values: STATUSES.map(String::from).to_vec(),
            },
            GroupingAxis {
                axis_key: "priority".into(),
                axis_label: "Priority".into(),
                axis_values: PRIORITIES.map(String::from).to_vec(),
            },
        ]
    };
    let metrics = || {
        vec![
            MetricDef {
                metric_key: "tickets".into(),
                metric_label: "Tickets".into(),
                unit: "tickets".into(),
                source_field: None,
            },
            MetricDef {
                metric_key: "hours".into(),
                metric_label: "Hours".into(),
                unit: "h".into(),
                source_field: Some("hours".into()),
            },
        ]
    };
    rsx! {
        Item { slug: "segmented-bar-explorer", name: "SegmentedBarExplorer",
            SegmentedBarExplorer {
                tally_rows: rows(),
                grouping_axes: axes(),
                metric_defs: metrics(),
                heading_text: "Tickets by region",
                start_metric: "tickets",
                start_axis: "region",
                start_stack: "status",
            }
        }
        Item { slug: "segmented-bar-explorer-focus", name: "SegmentedBarExplorer (East focused, Low hidden)",
            SegmentedBarExplorer {
                tally_rows: rows(),
                grouping_axes: axes(),
                metric_defs: metrics(),
                heading_text: "Hours by region",
                start_metric: "hours",
                start_axis: "region",
                start_stack: "priority",
                start_focus: "East",
                start_hidden: vec!["Low".to_string()],
            }
        }
    }
}

#[component]
fn SeriesTrendChartDemo() -> Element {
    use rv2_kit::components::series_trend_chart::{
        SeriesTone, SeriesTrendBlock, SparkTile, TrendSeries,
    };
    let labels: Vec<String> = (1..=7).map(|w| format!("W{w}")).collect();
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
                Some(31.0),
            ],
            tone: SeriesTone::Series1,
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
                Some(19.0),
            ],
            tone: SeriesTone::Muted,
            emphasis: false,
            is_earlier: true,
        },
    ];
    let sparklines = vec![
        SparkTile {
            label: "Open items".into(),
            values: vec![
                Some(9.0),
                Some(11.0),
                Some(10.0),
                Some(13.0),
                Some(12.0),
                Some(14.0),
                Some(16.0),
            ],
            tone: SeriesTone::Series2,
        },
        SparkTile {
            label: "Reopened".into(),
            values: vec![
                Some(2.0),
                Some(1.0),
                Some(3.0),
                Some(2.0),
                Some(4.0),
                Some(3.0),
                Some(2.0),
            ],
            tone: SeriesTone::Series3,
        },
        SparkTile {
            label: "Average age".into(),
            values: vec![
                Some(6.0),
                Some(7.0),
                Some(5.0),
                Some(8.0),
                Some(7.0),
                Some(9.0),
                Some(8.0),
            ],
            tone: SeriesTone::Series4,
        },
    ];
    rsx! {
        Item { slug: "series-trend-chart", name: "SeriesTrendChart",
            SeriesTrendBlock {
                title: "Tickets per week",
                label: "Tickets per week",
                x_labels: labels,
                series,
                sparklines,
            }
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
    let arm = |id: &str,
               letter: char,
               name: &str,
               control: bool,
               rate: f64,
               revenue: f64,
               refunds: f64| Arm {
        arm_id: id.into(),
        arm_letter: letter,
        arm_name: name.into(),
        readings: BTreeMap::from([
            ("conversion".to_string(), rate),
            ("revenue".to_string(), revenue),
            ("refunds".to_string(), refunds),
        ]),
        is_control: control,
        sample_size: Some(1_200),
        confidence: BTreeMap::from([("conversion".to_string(), 0.93)]),
        badge_tone: Some(SeriesTone::Ok),
    };
    rsx! {
        Item { slug: "variant-test-card", name: "VariantTestCard",
            VariantTestCard {
                test_name: "Pricing page copy",
                arms: vec![
                    arm("a", 'A', "Current copy", true, 0.042, 4.10, 2.4),
                    arm("b", 'B', "Short copy", false, 0.051, 4.85, 2.1),
                    arm("c", 'C', "Benefit-led copy", false, 0.047, 4.40, 2.9),
                ],
                metric_options: vec![
                    Metric { metric_id: "conversion".into(), metric_label: "Conversion".into(), format: Some(|v| format!("{:.1}%", v * 100.0)), lower_wins: false },
                    Metric { metric_id: "revenue".into(), metric_label: "Revenue".into(), format: Some(|v| format!("${v:.2}")), lower_wins: false },
                    Metric { metric_id: "refunds".into(), metric_label: "Refund rate".into(), format: Some(|v| format!("{v:.1}%")), lower_wins: true },
                ],
                starting_metric: Some("conversion".to_string()),
                chosen_arm: Some(choice()),
                on_choose: move |c: Option<String>| {
                    last.set(format!("choose {c:?}"));
                    choice.set(c);
                },
                status_chip: Some(StatusChip { chip_text: "Significant".into(), chip_tone: SeriesTone::Ok }),
                status_aside: Some("Day 9 of 14, 93% confidence".to_string()),
                result_action: Some(ResultAction { action_text: ActionLabel::Static("Ship winner".into()), on_run: EventHandler::new(move |a: Arm| last.set(format!("ship {}", a.arm_id))) }),
                reveal: Reveal::Tap,
                starting_open: true,
            }
            div { class: "gl-out", "data-out": "variant-test-card", "{last()}" }
        }
    }
}
