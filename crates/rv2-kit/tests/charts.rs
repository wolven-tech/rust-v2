//! Chart lane tests: pure chart maths and SSR renders of each chart component.

use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;
use rv2_kit::components::context_fill_grid::{
    ContextItem, ContextPart, FillLevel, allocate_cells, cell_count, fill_level, part_tokens,
};
use rv2_kit::components::context_treemap::{Block, BlockKind, fill_state, tile_rects};
use rv2_kit::components::daily_usage_bars::{
    DailyUsageBars, DayValue, Figure, bar_fraction, has_usage, live_text, palette_tone,
};
use rv2_kit::components::metric_tile::{
    ChangeTone, MetricTile, MetricValue, change_tone, change_words,
};
use rv2_kit::components::metric_trend_card::{
    MetricSeries, MetricTrendCard, change_chip, ranked_indices,
};
use rv2_kit::components::part_of_total_card::{Part, PartOfTotalCard, ranked_parts, total_amount};
use rv2_kit::components::provider_usage_chart::{
    Measure, ModelUsage, ProviderGroup, ProviderUsageChart, Usage, change_against, format_measure,
    grand_total, segment_mix, task_style, task_totals,
};
use rv2_kit::components::quota_usage_panel::{
    ActionState, CapMeter, DayPoint, QuotaUsagePanel, SpendPart, cap_ratio, countdown_text,
    date_label, meter_state,
};
use rv2_kit::components::run_cost_breakdown::{
    RunCostBreakdown, RunState, Step, StepKind, StepStatus, costliest_index, flatten,
    format_dollars, run_state, step_cost,
};
use rv2_kit::components::run_timing_waterfall::{
    NodeStatus, NodeType, RunTimingWaterfall, TraceNode, axis_end, flatten_trace, node_cost_total,
    node_span, slowest_id,
};
use rv2_kit::components::segmented_bar_explorer::{
    GroupingAxis, MetricDef, SegmentedBarExplorer, TallyRow, bar_breakdown, row_value,
};
use rv2_kit::components::series_trend_chart::{
    ChartVariant, IndexMove, Rect, SeriesTone, SeriesTrendChart, TrendSeries, area_path,
    axis_indices, compact_number, group_digits, line_path, nearest_index, nice_axis, non_negative,
    project_x, project_y, share, share_label, signed_percent, stack_bounds, step_index,
    tick_values, treemap_layout, value_domain, wrap_index,
};
use rv2_kit::components::variant_test_card::{
    Arm, Metric, VariantTestCard, delta_percent, is_favourable, leader_index,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn bracket_arrows_follow_the_feeds_and_stay_in_range() {
    use rv2_kit::components::ranking_bracket::pair_step;
    let sizes = [4, 2, 1];
    assert_eq!(pair_step(&sizes, (0, 3), &Key::ArrowRight), Some((1, 1)));
    assert_eq!(pair_step(&sizes, (1, 1), &Key::ArrowLeft), Some((0, 2)));
    assert_eq!(pair_step(&sizes, (2, 0), &Key::ArrowLeft), Some((1, 0)));
    assert_eq!(pair_step(&sizes, (2, 0), &Key::ArrowRight), None);
    assert_eq!(pair_step(&sizes, (0, 0), &Key::ArrowUp), None);
    assert_eq!(pair_step(&sizes, (0, 0), &Key::ArrowDown), Some((0, 1)));
    assert_eq!(pair_step(&sizes, (1, 1), &Key::ArrowDown), None);
}

#[test]
fn nice_axis_rounds_outwards_to_a_round_step() {
    let (lo, hi, step) = nice_axis(0.0, 87.0, 4);
    assert_eq!(step, 20.0);
    assert_eq!((lo, hi), (0.0, 100.0));
}

#[test]
fn tick_values_span_the_axis_inclusive() {
    assert_eq!(
        tick_values(0.0, 100.0, 25.0),
        vec![0.0, 25.0, 50.0, 75.0, 100.0]
    );
}

#[test]
fn value_domain_anchors_at_zero_only_when_asked() {
    assert_eq!(value_domain([40.0, 42.0], true), (0.0, 42.0));
    assert_eq!(value_domain([40.0, 42.0], false), (40.0, 42.0));
    assert_eq!(value_domain(Vec::<f64>::new(), true), (0.0, 1.0));
    assert_eq!(value_domain([f64::NAN, 3.0], true), (0.0, 3.0));
}

#[test]
fn project_maps_extremes_to_the_edges() {
    assert_eq!(project_y(10.0, 0.0, 10.0, 0.0, 100.0), 0.0);
    assert_eq!(project_y(0.0, 0.0, 10.0, 0.0, 100.0), 100.0);
    assert_eq!(project_x(0, 5, 10.0, 90.0), 10.0);
    assert_eq!(project_x(4, 5, 10.0, 90.0), 90.0);
    assert_eq!(project_x(0, 1, 10.0, 90.0), 50.0);
}

#[test]
fn line_path_lifts_the_pen_at_gaps() {
    let points = [Some((0.0, 0.0)), Some((1.0, 1.0)), None, Some((2.0, 2.0))];
    assert_eq!(line_path(&points), "M0.00 0.00 L1.00 1.00 M2.00 2.00");
}

#[test]
fn area_path_closes_each_run_on_the_baseline() {
    let points = [Some((0.0, 1.0)), Some((2.0, 1.0))];
    assert_eq!(
        area_path(&points, 5.0),
        "M0.00 1.00 L2.00 1.00 L2.00 5.00 L0.00 5.00 Z"
    );
}

#[test]
fn axis_indices_pick_first_middle_last() {
    assert_eq!(axis_indices(7), vec![0, 3, 6]);
    assert_eq!(axis_indices(21), vec![0, 10, 20]);
    assert_eq!(axis_indices(2), vec![0, 1]);
    assert_eq!(axis_indices(1), vec![0]);
    assert!(axis_indices(0).is_empty());
}

#[test]
fn nearest_index_snaps_to_the_closest_point() {
    assert_eq!(nearest_index(0.0, 5, 0.0, 100.0), Some(0));
    assert_eq!(nearest_index(49.0, 5, 0.0, 100.0), Some(2));
    assert_eq!(nearest_index(400.0, 5, 0.0, 100.0), Some(4));
    assert_eq!(nearest_index(10.0, 0, 0.0, 100.0), None);
}

#[test]
fn step_index_clamps_and_jumps() {
    assert_eq!(step_index(Some(2), 5, IndexMove::Next), Some(3));
    assert_eq!(step_index(Some(4), 5, IndexMove::Next), Some(4));
    assert_eq!(step_index(Some(0), 5, IndexMove::Prev), Some(0));
    assert_eq!(step_index(None, 5, IndexMove::Next), Some(0));
    assert_eq!(step_index(None, 5, IndexMove::Prev), Some(4));
    assert_eq!(step_index(Some(1), 5, IndexMove::Last), Some(4));
    assert_eq!(step_index(Some(3), 5, IndexMove::First), Some(0));
    assert_eq!(step_index(Some(0), 0, IndexMove::Next), None);
}

#[test]
fn wrap_index_wraps_at_both_ends() {
    assert_eq!(wrap_index(2, 3, true), 0);
    assert_eq!(wrap_index(0, 3, false), 2);
    assert_eq!(wrap_index(1, 3, true), 2);
}

#[test]
fn stack_bounds_accumulate_non_negative_values() {
    assert_eq!(
        stack_bounds(&[2.0, -1.0, 3.0]),
        vec![(0.0, 2.0), (2.0, 2.0), (2.0, 5.0)]
    );
}

#[test]
fn number_text_is_compact_grouped_and_signed() {
    assert_eq!(compact_number(1234.0), "1.2k");
    assert_eq!(compact_number(640_000.0), "640k");
    assert_eq!(compact_number(8_600_000.0), "8.6M");
    assert_eq!(compact_number(3_100_000_000.0), "3.1B");
    assert_eq!(group_digits(1284.0), "1,284");
    assert_eq!(signed_percent(4.5), "+4.5%");
    assert_eq!(signed_percent(-7.5), "-7.5%");
    assert_eq!(share_label(0.004), "<1%");
    assert_eq!(share_label(0.6), "60%");
}

#[test]
fn treemap_tiles_keep_order_and_fill_the_area() {
    let rects = treemap_layout(
        &[30.0, 50.0, 20.0],
        Rect {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 100.0,
        },
    );
    let area: f64 = rects.iter().map(|r| r.w * r.h).sum();
    assert!((area - 10_000.0).abs() < 1e-6);
    assert_eq!(rects.len(), 3);
    assert!(rects[1].w * rects[1].h > rects[2].w * rects[2].h);
}

#[test]
fn non_negative_and_share_guard_bad_input() {
    assert_eq!(non_negative(f64::NAN), 0.0);
    assert_eq!(non_negative(-3.0), 0.0);
    assert_eq!(share(5.0, 0.0), 0.0);
}

fn trend_app() -> Element {
    rsx! {
        SeriesTrendChart {
            x_labels: (1..=7).map(|d| format!("D{d}")).collect::<Vec<_>>(),
            series: vec![
                TrendSeries { name: "A".into(), values: vec![Some(1.0), Some(2.0), None, Some(4.0), Some(5.0), Some(6.0), Some(7.0)], tone: SeriesTone::Ink, emphasis: true, is_earlier: false },
                TrendSeries { name: "B".into(), values: vec![Some(2.0); 7], tone: SeriesTone::Ok, emphasis: false, is_earlier: true },
            ],
            label: "Requests",
        }
    }
}

#[test]
fn trend_chart_draws_first_middle_last_labels_only() {
    let html = render(trend_app);
    assert!(html.contains("aria-label=\"Requests\""));
    assert!(html.contains(">D1<"));
    assert!(html.contains(">D4<"));
    assert!(html.contains(">D7<"));
    assert!(!html.contains(">D2<"));
    assert!(!html.contains(">D5<"));
}

fn trend_empty_app() -> Element {
    rsx! {
        SeriesTrendChart {
            x_labels: vec!["a".to_string(), "b".to_string()],
            series: vec![TrendSeries { name: "Z".into(), values: vec![Some(0.0), Some(0.0)], tone: SeriesTone::Ink, emphasis: false, is_earlier: false }],
            empty_message: "Nothing yet",
        }
    }
}

#[test]
fn trend_chart_shows_empty_message_for_zeros_only() {
    let html = render(trend_empty_app);
    assert!(html.contains("Nothing yet"));
    assert!(!html.contains("kit-trend__stroke"));
}

fn sparkline_app() -> Element {
    rsx! {
        SeriesTrendChart {
            x_labels: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            series: vec![TrendSeries { name: "S".into(), values: vec![Some(1.0), Some(3.0), Some(2.0)], tone: SeriesTone::Ink, emphasis: false, is_earlier: false }],
            variant: ChartVariant::Sparkline,
        }
    }
}

#[test]
fn sparkline_has_no_axis() {
    let html = render(sparkline_app);
    assert!(!html.contains("kit-trend__date"));
    assert!(!html.contains("kit-trend__tick"));
}

#[test]
fn change_tone_follows_lower_is_better() {
    assert_eq!(change_tone(8.0, false).1, ChangeTone::Favourable);
    assert_eq!(change_tone(8.0, true).1, ChangeTone::Unfavourable);
    assert_eq!(change_tone(-2.0, true).1, ChangeTone::Favourable);
    assert_eq!(change_tone(0.0, false).1, ChangeTone::Neutral);
}

#[test]
fn change_words_read_out_direction_and_magnitude() {
    assert_eq!(
        change_words(16.0, "16.0%", Some("vs last week")),
        "up 16.0 percent vs last week"
    );
    assert_eq!(change_words(0.0, "0.0%", None), "no change");
}

fn tile_link_app() -> Element {
    rsx! {
        MetricTile {
            metric_label: "Revenue",
            metric_value: MetricValue::Number(1234.0),
            change_percent: 8.0,
            change_note: "vs last week",
            target_href: "/revenue",
            corner_control: rsx! { button { "Period" } },
        }
    }
}

#[test]
fn metric_tile_link_names_label_value_and_change() {
    let html = render(tile_link_app);
    assert!(html.contains("href=\"/revenue\""));
    assert!(html.contains("up 8.0 percent vs last week"));
    assert!(html.contains(">Period<"));
}

fn tile_loading_app() -> Element {
    rsx! {
        MetricTile { metric_label: "Signups", metric_value: MetricValue::Text("n/a".into()), is_loading: true }
    }
}

#[test]
fn metric_tile_loading_is_busy_and_hides_the_value() {
    let html = render(tile_loading_app);
    assert!(html.contains("aria-busy=\"true\""));
    assert!(!html.contains("class=\"kit-tile__figure\""));
    assert!(!html.contains("kit-tile__figure--text"));
    assert!(!html.contains("n/a"));
}

fn tile_text_app() -> Element {
    rsx! {
        MetricTile { metric_label: "Status", metric_value: MetricValue::Text("1,000.00".into()) }
    }
}

#[test]
fn metric_tile_shows_text_values_exactly() {
    assert!(render(tile_text_app).contains("1,000.00"));
}

#[test]
fn trend_card_ranks_and_signs() {
    assert_eq!(ranked_indices(&[Some(1.0), None, Some(3.0)]), vec![2, 0, 1]);
    assert_eq!(change_chip(-3.0), ("-", "fault"));
    assert_eq!(change_chip(3.0), ("+", "ok"));
}

fn metric_card_empty_app() -> Element {
    rsx! {
        MetricTrendCard {
            metric_name: "Errors",
            headline_value: "0",
            change_value: 0.0,
            axis_labels: vec!["a".to_string(), "b".to_string()],
            data_series: Vec::<MetricSeries>::new(),
        }
    }
}

#[test]
fn trend_card_empty_shows_message_and_no_lines() {
    let html = render(metric_card_empty_app);
    assert!(html.contains("No data yet"));
    assert!(!html.contains("kit-metric-card__line"));
}

fn metric_card_app() -> Element {
    rsx! {
        MetricTrendCard {
            metric_name: "Traffic",
            headline_value: "1.2k",
            change_value: -4.0,
            axis_labels: (1..=7).map(|d| format!("D{d}")).collect::<Vec<_>>(),
            data_series: vec![MetricSeries { series_name: "Web".into(), series_tone: SeriesTone::Ink, series_values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0] }],
        }
    }
}

#[test]
fn trend_card_negative_change_is_signed_and_toned() {
    let html = render(metric_card_app);
    assert!(html.contains("-4"));
    assert!(html.contains("data-tone=\"fault\""));
    assert!(html.contains("down 4 vs previous period"));
}

#[test]
fn parts_rank_descending_and_keep_ties_in_input_order() {
    let parts = vec![
        Part {
            part_key: None,
            part_name: "A".into(),
            part_amount: 5.0,
            part_color: None,
        },
        Part {
            part_key: None,
            part_name: "B".into(),
            part_amount: 9.0,
            part_color: None,
        },
        Part {
            part_key: None,
            part_name: "C".into(),
            part_amount: 5.0,
            part_color: None,
        },
    ];
    assert_eq!(ranked_parts(&parts), vec![1, 0, 2]);
    assert_eq!(total_amount(&parts), 19.0);
}

fn part_total_app() -> Element {
    rsx! {
        PartOfTotalCard {
            part_entries: vec![
                Part { part_key: Some("x".into()), part_name: "Search".into(), part_amount: 300.0, part_color: None },
                Part { part_key: Some("y".into()), part_name: "Code".into(), part_amount: 700.0, part_color: None },
            ],
            change_pct: -3.0,
        }
    }
}

#[test]
fn part_card_names_top_part_and_negative_change() {
    let html = render(part_total_app);
    assert!(html.contains("Top part: Code"));
    assert!(html.contains("data-negative=\"true\""));
    assert!(html.contains("-3.0%"));
}

fn part_total_hidden_app() -> Element {
    rsx! {
        PartOfTotalCard {
            part_entries: vec![Part { part_key: None, part_name: "Only".into(), part_amount: 1.0, part_color: None }],
            show_trailing: false,
        }
    }
}

#[test]
fn part_card_trailing_row_can_be_hidden() {
    assert!(!render(part_total_hidden_app).contains("Top part:"));
}

fn usage_app() -> Element {
    rsx! {
        DailyUsageBars {
            chart_heading: "API calls",
            day_values: (1..=21)
                .map(|d| DayValue { day_label: format!("Sep {d}"), day_value: if d == 5 { None } else { Some(d as f64 * 10.0) } })
                .collect::<Vec<_>>(),
            total_figures: vec![Figure { figure_label: "Total".into(), figure_value: 1284.0 }],
            is_today_live: true,
        }
    }
}

#[test]
fn daily_usage_draws_one_bar_per_day_and_the_live_badge() {
    let html = render(usage_app);
    assert_eq!(html.matches("class=\"kit-usage__bar").count(), 21);
    assert!(html.contains("Real-time"));
    assert!(html.contains("kit-usage__bar--live"));
    assert!(html.contains("1,284"));
}

fn usage_empty_app() -> Element {
    rsx! { DailyUsageBars { chart_heading: "Empty", day_values: vec![DayValue { day_label: "a".into(), day_value: None }], total_figures: vec![] } }
}

#[test]
fn daily_usage_empty_shows_message_and_no_bars() {
    let html = render(usage_empty_app);
    assert!(html.contains("No usage yet"));
    assert!(!html.contains("kit-usage__bar"));
}

#[test]
fn daily_usage_helpers_handle_invalid_days_and_live_text() {
    assert_eq!(bar_fraction(Some(-1.0), 10.0), 0.0);
    assert_eq!(bar_fraction(None, 10.0), 0.0);
    assert_eq!(bar_fraction(Some(5.0), 10.0), 0.5);
    assert!(!has_usage(&[DayValue {
        day_label: "a".into(),
        day_value: Some(0.0)
    }]));
    assert_eq!(live_text(true, ""), Some("Real-time".to_string()));
    assert_eq!(live_text(false, "x"), None);
    assert_eq!(
        palette_tone(&[SeriesTone::Ink, SeriesTone::Ok], 0, 4),
        SeriesTone::Ink
    );
    assert_eq!(
        palette_tone(&[SeriesTone::Ink, SeriesTone::Ok], 3, 4),
        SeriesTone::Ok
    );
}

fn provider_groups() -> Vec<ProviderGroup> {
    let usage = |t: f64, c: f64, r: f64| Usage {
        tokens: Some(t),
        cost_usd: Some(c),
        request_count: Some(r),
    };
    let mut tasks = BTreeMap::new();
    tasks.insert("code".to_string(), usage(100.0, 1.0, 2.0));
    vec![
        ProviderGroup {
            group_name: "Anthropic".into(),
            group_tone: Some(SeriesTone::Ink),
            group_logo: None,
            models: vec![
                ModelUsage {
                    model_name: "big".into(),
                    usage: usage(900.0, 8.0, 10.0),
                    task_usage: tasks.clone(),
                },
                ModelUsage {
                    model_name: "small".into(),
                    usage: usage(100.0, 0.5, 4.0),
                    task_usage: BTreeMap::new(),
                },
            ],
        },
        ProviderGroup {
            group_name: "Other".into(),
            group_tone: None,
            group_logo: None,
            models: vec![ModelUsage {
                model_name: "neg".into(),
                usage: Usage {
                    tokens: Some(-5.0),
                    cost_usd: None,
                    request_count: None,
                },
                task_usage: BTreeMap::new(),
            }],
        },
    ]
}

#[test]
fn provider_totals_and_change_follow_the_measure() {
    let groups = provider_groups();
    assert_eq!(grand_total(&groups, Measure::Tokens), 1000.0);
    assert_eq!(change_against(1000.0, Some(800.0)), Some((25.0, true)));
    assert_eq!(change_against(500.0, Some(800.0)).map(|c| c.1), Some(false));
    assert_eq!(change_against(500.0, None), None);
}

#[test]
fn provider_helpers_format_task_and_shade() {
    assert_eq!(format_measure(Measure::Cost, 8_600_000.0), "$8.6M");
    assert_eq!(
        format_measure(Measure::Requests, 640_000.0),
        "640k requests"
    );
    assert_eq!(task_style(&BTreeMap::new(), "search").0, "Search");
    assert_eq!(task_style(&BTreeMap::new(), "mystery").1, SeriesTone::Muted);
    assert_eq!(segment_mix(0), 100);
    assert_eq!(segment_mix(10), 35);
    let tasks = task_totals(&provider_groups(), Measure::Tokens, Some(0));
    assert_eq!(tasks, vec![("code".to_string(), 100.0)]);
}

fn provider_app() -> Element {
    rsx! { ProviderUsageChart { provider_groups: provider_groups() } }
}

#[test]
fn provider_chart_renders_columns_and_measure_switch() {
    let html = render(provider_app);
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("kit-pu__segment"));
}

fn provider_empty_app() -> Element {
    rsx! { ProviderUsageChart { provider_groups: Vec::<ProviderGroup>::new() } }
}

#[test]
fn provider_chart_without_usage_says_so() {
    let html = render(provider_empty_app);
    assert!(html.contains("No usage this week"));
    assert!(!html.contains("kit-pu__segment"));
}

#[test]
fn quota_state_and_cap_ratio() {
    assert_eq!(meter_state(3015.0, 5000.0, 0.85), ActionState::Ok);
    assert_eq!(meter_state(4300.0, 5000.0, 0.85), ActionState::Warn);
    assert_eq!(meter_state(5000.0, 5000.0, 0.85), ActionState::Limit);
    assert_eq!(cap_ratio(3.0, None), None);
    assert_eq!(cap_ratio(3.0, Some(4.0)), Some(0.75));
}

#[test]
fn quota_countdown_and_date() {
    assert_eq!(countdown_text(3 * 86_400_000 + 4 * 3_600_000), "3d 4h");
    assert_eq!(countdown_text(5 * 3_600_000 + 12 * 60_000), "5h 12m");
    assert_eq!(countdown_text(42 * 60_000), "42m");
    assert_eq!(countdown_text(0), "now");
    assert_eq!(date_label(0), "Jan 1");
    // Day 11017 since the epoch is 2000-03-01.
    assert_eq!(date_label(11_017 * 86_400_000), "Mar 1");
}

fn quota_app() -> Element {
    rsx! {
        QuotaUsagePanel {
            plan_total: 5000.0,
            spend_breakdown: vec![
                SpendPart { spend_label: "Search".into(), spend_value: 2015.0, spend_emphasis: None },
                SpendPart { spend_label: "Code".into(), spend_value: 1000.0, spend_emphasis: None },
            ],
            cap_list: vec![CapMeter { cap_label: "Seats".into(), cap_used: 3.0, cap_total: None, unit: None, decimals: None }],
            daily_trend: vec![DayPoint { day_label: "Mon".into(), day_value: None }],
        }
    }
}

#[test]
fn quota_meter_reads_consumed_of_total_and_open_cap() {
    let html = render(quota_app);
    assert!(html.contains("3,015 of 5,000 credits"));
    assert!(html.contains("Open-ended"));
    assert!(html.contains("data-stub=\"true\""));
}

fn explorer_rows() -> (Vec<TallyRow>, GroupingAxis, GroupingAxis, MetricDef) {
    let row = |model: &str, status: &str, tokens: f64| TallyRow {
        axis: BTreeMap::from([
            ("model".to_string(), model.to_string()),
            ("status".to_string(), status.to_string()),
        ]),
        metrics: BTreeMap::from([("tokens".to_string(), tokens)]),
    };
    let rows = vec![
        row("a", "ok", 10.0),
        row("a", "err", 5.0),
        row("b", "ok", 20.0),
    ];
    let model = GroupingAxis {
        axis_key: "model".into(),
        axis_label: "Model".into(),
        axis_values: vec!["a".into(), "b".into()],
    };
    let status = GroupingAxis {
        axis_key: "status".into(),
        axis_label: "Status".into(),
        axis_values: vec!["ok".into(), "err".into()],
    };
    let metric = MetricDef {
        metric_key: "tokens".into(),
        metric_label: "Tokens".into(),
        unit: "tok".into(),
        source_field: Some("tokens".into()),
    };
    (rows, model, status, metric)
}

#[test]
fn segmented_breakdown_totals_match_for_visible_segments() {
    let (rows, model, status, metric) = explorer_rows();
    let all = bar_breakdown(&rows, &model, Some(&status), &metric, &BTreeSet::new());
    assert_eq!(all[0].total, 15.0);
    assert_eq!(all[1].total, 20.0);
    let hidden = BTreeSet::from(["err".to_string()]);
    let some = bar_breakdown(&rows, &model, Some(&status), &metric, &hidden);
    assert_eq!(some[0].total, 10.0);
    let counted = MetricDef {
        metric_key: "count".into(),
        metric_label: "Rows".into(),
        unit: "rows".into(),
        source_field: None,
    };
    assert_eq!(row_value(&rows[0], &counted), 1.0);
}

fn explorer_app() -> Element {
    let (rows, model, status, metric) = explorer_rows();
    rsx! {
        SegmentedBarExplorer {
            tally_rows: rows,
            grouping_axes: vec![model, status],
            metric_defs: vec![metric],
            heading_text: "Tokens by model",
            start_metric: "tokens",
            start_axis: "model",
            start_stack: "status",
        }
    }
}

#[test]
fn segmented_explorer_renders_legend_toggles_and_table() {
    let html = render(explorer_app);
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("<table"));
    assert!(html.contains("Total"));
}

fn leaf(id: &str, cost: Option<f64>) -> Step {
    Step {
        step_id: id.into(),
        step_name: id.into(),
        step_kind: StepKind::ModelCall,
        step_status: StepStatus::Done,
        model_name: None,
        step_note: None,
        in_tokens: Some(10.0),
        out_tokens: Some(5.0),
        step_cost: cost,
        step_elapsed: Some(1.0),
        prompt_text: None,
        result_text: None,
        error_text: None,
        child_steps: vec![],
    }
}

#[test]
fn run_cost_sums_children_and_names_costliest() {
    let mut parent = leaf("agent", None);
    parent.step_kind = StepKind::Agent;
    parent.child_steps = vec![leaf("c1", Some(0.02)), leaf("c2", Some(0.03))];
    assert!((step_cost(&parent) - 0.05).abs() < 1e-12);
    let steps = vec![
        leaf("x", Some(0.04)),
        leaf("y", Some(0.01)),
        leaf("z", Some(0.0)),
    ];
    assert_eq!(costliest_index(&steps), Some(0));
    assert_eq!(format_dollars(f64::NAN), "–");
    assert_eq!(format_dollars(0.04), "$0.040");
}

#[test]
fn run_state_and_folding() {
    let mut failing = leaf("f", None);
    failing.step_status = StepStatus::Failed;
    assert_eq!(
        run_state(std::slice::from_ref(&failing), None),
        RunState::Failed
    );
    assert_eq!(
        run_state(std::slice::from_ref(&failing), Some(RunState::Done)),
        RunState::Done
    );
    let mut parent = leaf("p", None);
    parent.child_steps = vec![leaf("k", Some(0.1))];
    assert_eq!(
        flatten(std::slice::from_ref(&parent), &BTreeSet::new()).len(),
        2
    );
    assert_eq!(
        flatten(
            std::slice::from_ref(&parent),
            &BTreeSet::from(["p".to_string()])
        )
        .len(),
        1
    );
}

fn cost_app() -> Element {
    rsx! {
        RunCostBreakdown {
            step_list: vec![leaf("one", Some(0.04)), leaf("two", Some(0.01))],
            run_summary: "Summarise the spec",
        }
    }
}

#[test]
fn run_cost_renders_tree_items() {
    let html = render(cost_app);
    assert_eq!(html.matches("role=\"treeitem\"").count(), 2);
    assert!(html.contains("aria-pressed=\"true\""));
    assert!(html.contains("Summarise the spec"));
}

fn trace_node(id: &str, kind: NodeType, begin: f64, finish: Option<f64>) -> TraceNode {
    TraceNode {
        node_id: id.into(),
        node_label: id.into(),
        node_type: kind,
        node_status: if finish.is_some() {
            NodeStatus::Done
        } else {
            NodeStatus::Running
        },
        begin_offset: begin,
        finish_offset: finish,
        node_note: None,
        node_tag: None,
        model_label: None,
        prompt_tokens: None,
        completion_tokens: None,
        node_cost: Some(0.01),
        input_excerpt: None,
        output_excerpt: None,
        failure_text: None,
        children: vec![],
    }
}

fn trace() -> TraceNode {
    let mut root = trace_node("root", NodeType::Agent, 0.0, Some(10.0));
    let mut sub = trace_node("sub", NodeType::Subagent, 1.0, Some(8.0));
    sub.children = vec![
        trace_node("m1", NodeType::Model, 1.0, Some(3.0)),
        trace_node("t1", NodeType::Tool, 3.0, Some(8.0)),
    ];
    root.children = vec![sub];
    root
}

#[test]
fn waterfall_parent_spans_children_and_costs_sum() {
    let root = trace();
    assert_eq!(node_span(&root.children[0], 10.0), (1.0, 8.0));
    assert_eq!(node_cost_total(&root), 0.02);
    assert_eq!(slowest_id(&root), Some("t1".to_string()));
    assert_eq!(axis_end(8.0, 1.0), 10.0);
    let collapsed = BTreeSet::from(["sub".to_string()]);
    assert_eq!(flatten_trace(&root, &collapsed).len(), 2);
}

fn waterfall_app() -> Element {
    rsx! { RunTimingWaterfall { trace_tree: trace() } }
}

#[test]
fn waterfall_renders_heading_and_slowest_tag() {
    let html = render(waterfall_app);
    assert!(html.contains("Slowest"));
    assert!(html.contains("role=\"tree\""));
    assert!(html.contains("Done"));
}

fn arm(id: &str, letter: char, value: f64, control: bool) -> Arm {
    Arm {
        arm_id: id.into(),
        arm_letter: letter,
        arm_name: id.into(),
        readings: BTreeMap::from([("ctr".to_string(), value)]),
        is_control: control,
        sample_size: None,
        confidence: BTreeMap::new(),
        badge_tone: None,
    }
}

#[test]
fn variant_delta_and_leader() {
    assert!((delta_percent(41.8, 40.0).unwrap() - 4.5).abs() < 1e-9);
    assert_eq!(delta_percent(1.0, 0.0), None);
    assert!(is_favourable(-7.5, true));
    let arms = vec![arm("a", 'A', 40.0, true), arm("b", 'B', 37.0, false)];
    let metric = Metric {
        metric_id: "ctr".into(),
        metric_label: "CTR".into(),
        format: None,
        lower_wins: true,
    };
    assert_eq!(leader_index(&arms, &metric), Some(1));
}

fn variant_app() -> Element {
    rsx! {
        VariantTestCard {
            test_name: "Headline",
            arms: vec![arm("a", 'A', 40.0, true), arm("b", 'B', 41.8, false)],
            metric_options: vec![Metric { metric_id: "ctr".into(), metric_label: "CTR".into(), format: None, lower_wins: false }],
            is_open: true,
        }
    }
}

#[test]
fn variant_card_open_shows_delta_against_control() {
    let html = render(variant_app);
    assert!(html.contains("+4.5%"));
    assert!(html.contains("Base"));
    assert!(html.contains("role=\"radio\""));
    assert!(html.contains("Pick a winner"));
}

#[test]
fn context_cells_follow_tokens_and_levels() {
    assert_eq!(cell_count(1.0, 1000.0, 100), 1);
    assert_eq!(cell_count(0.0, 1000.0, 100), 0);
    assert_eq!(
        allocate_cells(&[500.0, 300.0, 400.0], 1000.0, 100),
        vec![50, 30, 20]
    );
    assert_eq!(fill_level(0.85, 0.8, 0.95), FillLevel::Caution);
    assert_eq!(fill_level(0.96, 0.8, 0.95), FillLevel::Full);
}

#[test]
fn context_part_with_items_sums_them() {
    let part = ContextPart {
        part_key: "files".into(),
        part_name: "Files".into(),
        token_count: Some(999.0),
        sub_items: vec![
            ContextItem {
                item_key: "a".into(),
                item_name: "a.rs".into(),
                item_tokens: 10.0,
            },
            ContextItem {
                item_key: "b".into(),
                item_name: "b.rs".into(),
                item_tokens: 5.0,
            },
        ],
        is_locked: false,
        can_compact: true,
        part_color: None,
    };
    assert_eq!(part_tokens(&part), 15.0);
}

fn context_app() -> Element {
    rsx! {
        rv2_kit::components::context_fill_grid::ContextFillGrid {
            part_list: vec![ContextPart {
                part_key: "sys".into(),
                part_name: "System".into(),
                token_count: Some(2000.0),
                sub_items: vec![],
                is_locked: true,
                can_compact: false,
                part_color: None,
            }],
            window_size: 10000.0,
            model_name: "model-x",
        }
    }
}

#[test]
fn context_grid_reports_used_tokens() {
    let html = render(context_app);
    assert!(html.contains("2000 of 10000 tokens used"));
    assert!(html.contains("20%"));
}

#[test]
fn treemap_tiles_plus_free_fill_the_window() {
    let rects = tile_rects(&[300.0, 200.0], 1000.0);
    assert_eq!(rects.len(), 3);
    let area: f64 = rects.iter().map(|r| r.w * r.h).sum();
    assert!((area - 10_000.0).abs() < 1e-6);
    assert_eq!(fill_state(1200.0, 1000.0), ("full".to_string(), true));
}

fn treemap_app() -> Element {
    rsx! {
        rv2_kit::components::context_treemap::ContextTreemap {
            block_list: vec![
                Block { block_key: "sys".into(), block_label: "Instructions".into(), block_kind: BlockKind::Instructions, token_count: 300.0, block_note: None, sub_items: vec![] },
                Block { block_key: "conv".into(), block_label: "Conversation".into(), block_kind: BlockKind::Conversation, token_count: 200.0, block_note: None, sub_items: vec!["turn 1".into()] },
            ],
            window_size: 1000.0,
            model_name: "model-x",
        }
    }
}

#[test]
fn treemap_renders_tiles_and_no_compaction_note() {
    let html = render(treemap_app);
    assert!(html.contains("No compaction has run"));
    assert!(html.contains("data-kind=\"conversation\""));
}
