//! Provider usage chart: the structure amendment's acceptance checks, as SSR renders and pure logic.

use dioxus::prelude::*;
use rv2_kit::components::provider_usage_chart::{
    Measure, ModelUsage, ProviderGroup, ProviderUsageChart, Usage, change_against, column_segments,
    grand_total, task_totals,
};
use rv2_kit::components::series_trend_chart::{SeriesTone, share, share_label};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn usage(tokens: f64) -> Usage {
    Usage {
        tokens: Some(tokens),
        cost_usd: Some(tokens / 100_000.0),
        request_count: Some(tokens / 1_000.0),
    }
}

fn model(name: &str, tokens: f64, tasks: &[(&str, f64)]) -> ModelUsage {
    ModelUsage {
        model_name: name.into(),
        usage: usage(tokens),
        task_usage: tasks
            .iter()
            .map(|(kind, t)| (kind.to_string(), usage(*t)))
            .collect(),
    }
}

fn group(name: &str, models: Vec<ModelUsage>) -> ProviderGroup {
    ProviderGroup {
        group_name: name.into(),
        group_tone: Some(SeriesTone::Ink),
        group_logo: None,
        models,
    }
}

/// Four providers, ten models, four task kinds; mirrors the gallery demo.
fn demo_groups() -> Vec<ProviderGroup> {
    vec![
        group(
            "Alder",
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
            vec![
                model(
                    "Dune Core",
                    450_000.0,
                    &[("analyse", 250_000.0), ("code", 200_000.0)],
                ),
                model("Dune Fast", 50_000.0, &[("search", 50_000.0)]),
            ],
        ),
    ]
}

fn at_rest_app() -> Element {
    rsx! {
        ProviderUsageChart {
            provider_groups: demo_groups(),
            prior_totals: Some(usage(4_600_000.0)),
            prior_label: Some("last week".to_string()),
        }
    }
}

fn engaged_app() -> Element {
    rsx! {
        ProviderUsageChart {
            provider_groups: demo_groups(),
            starting_group: Some(1),
            prior_totals: Some(usage(4_600_000.0)),
            prior_label: Some("last week".to_string()),
        }
    }
}

fn single_measure_app() -> Element {
    rsx! {
        ProviderUsageChart {
            provider_groups: demo_groups(),
            measure_options: vec![Measure::Tokens],
        }
    }
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

#[test]
fn provider_total_is_the_sum_of_every_model_under_the_measure() {
    assert_eq!(grand_total(&demo_groups(), Measure::Tokens), 5_200_000.0);
}

#[test]
fn provider_task_totals_add_up_to_the_summary_total() {
    let groups = demo_groups();
    let tasks: f64 = task_totals(&groups, Measure::Tokens, None)
        .iter()
        .map(|(_, v)| v)
        .sum();
    assert_eq!(tasks, grand_total(&groups, Measure::Tokens));
}

#[test]
fn engaged_provider_narrows_task_totals_to_its_own_models() {
    let groups = demo_groups();
    let birch: f64 = task_totals(&groups, Measure::Tokens, Some(1))
        .iter()
        .map(|(_, v)| v)
        .sum();
    assert_eq!(birch, 1_200_000.0);
}

#[test]
fn provider_column_shares_round_to_one_hundred_percent() {
    for group in demo_groups() {
        let total: f64 = column_segments(&group, Measure::Tokens)
            .iter()
            .map(|(_, v)| *v)
            .sum();
        let percent: u32 = column_segments(&group, Measure::Tokens)
            .iter()
            .map(|(_, v)| {
                share_label(share(*v, total))
                    .trim_end_matches('%')
                    .parse::<u32>()
                    .expect("whole percent")
            })
            .sum();
        assert_eq!(percent, 100, "shares of {}", group.group_name);
    }
}

#[test]
fn provider_model_under_one_percent_reads_less_than_one() {
    assert_eq!(share_label(share(50_000.0, 10_000_000.0)), "<1%");
}

#[test]
fn provider_negative_or_missing_usage_counts_as_zero_in_every_measure() {
    let broken = Usage {
        tokens: Some(-5.0),
        cost_usd: None,
        request_count: Some(f64::NAN),
    };
    for measure in [Measure::Tokens, Measure::Cost, Measure::Requests] {
        assert_eq!(broken.value(measure), 0.0);
    }
}

#[test]
fn provider_column_puts_the_heaviest_model_at_the_base() {
    let segments = column_segments(&demo_groups()[0], Measure::Tokens);
    assert_eq!(segments[0].0, "Alder Large");
    assert_eq!(segments[2].0, "Alder Mini");
}

#[test]
fn provider_change_points_up_only_when_total_exceeds_prior() {
    let (pct, up) = change_against(5_200_000.0, Some(4_600_000.0)).expect("prior given");
    assert!(up);
    assert!((pct - 13.04).abs() < 0.01);
    assert_eq!(
        change_against(4_000_000.0, Some(4_600_000.0)).map(|c| c.1),
        Some(false)
    );
}

#[test]
fn provider_chart_at_rest_shows_every_column_and_the_full_task_list() {
    let html = render(at_rest_app);
    assert_eq!(count(&html, "class=\"kit-pu__label\""), 4);
    assert_eq!(count(&html, "class=\"kit-pu__segment\""), 10);
    assert_eq!(count(&html, "data-dim=\"true\""), 0);
    assert!(html.contains("kit-pu__card kit-pu__card--idle"));
    assert!(html.contains("Tasks across all providers"));
    assert!(html.contains("data-up=\"true\""));
}

#[test]
fn provider_engaged_column_opens_its_card_and_dims_the_others() {
    let html = render(engaged_app);
    assert_eq!(count(&html, "data-active=\"true\""), 1);
    assert_eq!(count(&html, "data-dim=\"true\""), 8);
    assert!(html.contains("data-card=\"true\""));
    assert!(html.contains("Tasks for Birch"));
    assert!(html.contains(">Birch Pro<"));
}

#[test]
fn provider_chart_with_one_measure_renders_no_switch() {
    let html = render(single_measure_app);
    assert!(!html.contains("kit-pu__measures"));
}
