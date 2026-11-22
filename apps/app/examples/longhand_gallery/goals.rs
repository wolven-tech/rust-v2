use dioxus::prelude::*;
use rv2_kit::components::metric_trend_card::{MetricSeries, MetricTrendCard};
use rv2_kit::components::series_trend_chart::{
    ChartVariant, SeriesTone, SeriesTrendChart, TrendSeries,
};

use super::Scene;

const WEEKS: [&str; 8] = ["W40", "W41", "W42", "W43", "W44", "W45", "W46", "W47"];
const RENEWAL_HISTORY: [f64; 7] = [85.1, 85.9, 86.2, 86.8, 87.0, 87.6, 88.0];

struct KeyResult {
    name: &'static str,
    unit: &'static str,
    target: f64,
    lower_is_better: bool,
    step: f64,
}

const KEY_RESULTS: [KeyResult; 3] = [
    KeyResult {
        name: "Renewal rate",
        unit: "%",
        target: 92.0,
        lower_is_better: false,
        step: 0.6,
    },
    KeyResult {
        name: "Median reply time",
        unit: "h",
        target: 4.0,
        lower_is_better: true,
        step: -0.2,
    },
    KeyResult {
        name: "Newsletter open rate",
        unit: "%",
        target: 45.0,
        lower_is_better: false,
        step: 1.0,
    },
];

fn progress(kr: &KeyResult, current: f64) -> u32 {
    let ratio = if kr.lower_is_better {
        kr.target / current.max(0.01)
    } else {
        current / kr.target
    };
    (ratio * 100.0).clamp(0.0, 100.0) as u32
}

#[component]
pub fn Goals() -> Element {
    let mut current = use_signal(|| [88.4_f64, 3.1, 41.0]);
    let mut last = use_signal(|| "no update logged yet".to_string());
    let values = current();
    let renewal = values[0];
    let mut renewal_series = RENEWAL_HISTORY.to_vec();
    renewal_series.push(renewal);
    let weeks: Vec<String> = WEEKS.iter().map(|w| w.to_string()).collect();
    let target_line = vec![92.0; WEEKS.len()];
    let change = renewal - RENEWAL_HISTORY[RENEWAL_HISTORY.len() - 1];

    rsx! {
        Scene {
            id: "goals",
            title: "Goals and results",
            story: "Priya tracks the Q4 goal: renewals above 92%, support replies under four hours, and newsletter opens. Each key result is fed by the workflows that move it.",
            div { class: "lh-stack",
                section { class: "lh-card", "aria-label": "Q4 goal",
                    div { class: "lh-between",
                        h3 { class: "lh-h", "Q4: keep renewals above 92% and answer support fast" }
                        span { class: "lh-chip", "data-tone": "ok", "2 of 3 on track" }
                    }
                    div { class: "lh-grid-3",
                        for (index, kr) in KEY_RESULTS.iter().enumerate() {
                            section { class: "lh-card", key: "{kr.name}", "aria-label": "{kr.name}",
                                span { class: "lh-muted", "Key result {index + 1}" }
                                span { class: "lh-title", "{kr.name}" }
                                span { class: "lh-kv",
                                    span { "{values[index]:.1}{kr.unit}" }
                                    span { class: "lh-muted", "target {kr.target:.0}{kr.unit}" }
                                }
                                div { class: "lh-bar",
                                    div { class: "lh-bar-fill", "data-tone": if progress(kr, values[index]) >= 100 { "ok" } else { "" }, style: "width: {progress(kr, values[index])}%;" }
                                }
                                button { class: "lh-btn", r#type: "button",
                                    onclick: move |_| {
                                        current.with_mut(|v| v[index] += KEY_RESULTS[index].step);
                                        last.set(format!("logged an update for {}", KEY_RESULTS[index].name));
                                    },
                                    "Log this week's number"
                                }
                            }
                        }
                    }
                }
                div { class: "lh-grid",
                    MetricTrendCard {
                        metric_name: "Renewal rate, weekly",
                        headline_value: format!("{renewal:.1}%"),
                        change_value: change,
                        axis_labels: weeks.clone(),
                        data_series: vec![MetricSeries {
                            series_name: "Renewal rate".into(),
                            series_tone: SeriesTone::Ok,
                            series_values: renewal_series.clone(),
                        }],
                        footer_text: Some("Updated from the Monday pipeline review".to_string()),
                    }
                    section { class: "lh-card", "aria-label": "Linked workflows",
                        h3 { class: "lh-h", "Workflows that move these" }
                        ul { class: "lh-list",
                            li { span { "Monday pipeline review" } span { class: "lh-chip", "data-tone": "ok", "running" } }
                            li { span { "Renewal follow-ups" } span { class: "lh-chip", "idle" } }
                            li { span { "Weekly customer newsletter" } span { class: "lh-chip", "data-tone": "warn", "needs you" } }
                        }
                        p { class: "lh-muted", "Each update comes from the workflow that produced the number, with its run link." }
                    }
                }
                section { class: "lh-card", "aria-label": "Trend",
                    h3 { class: "lh-h", "Renewals against target, last eight weeks" }
                    SeriesTrendChart {
                        x_labels: weeks,
                        series: vec![
                            TrendSeries {
                                name: "Renewal rate".into(),
                                values: renewal_series.iter().map(|v| Some(*v)).collect(),
                                tone: SeriesTone::Ok,
                                emphasis: true,
                                is_earlier: false,
                            },
                            TrendSeries {
                                name: "Target".into(),
                                values: target_line.iter().map(|v| Some(*v)).collect(),
                                tone: SeriesTone::Muted,
                                emphasis: false,
                                is_earlier: true,
                            },
                        ],
                        variant: ChartVariant::Line,
                        label: "Renewal rate against the 92% target",
                        height: Some(220.0),
                    }
                    p { class: "lh-muted", "Renewal rate is {progress(&KEY_RESULTS[0], renewal)}% of the way to target." }
                    p { class: "gl-out", "data-out": "goals", "{last()}" }
                }
            }
        }
    }
}
