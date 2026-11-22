use dioxus::prelude::*;
use rv2_kit::components::connector_settings_card::{ConnectorSettingsCard, ConnectorValues};
use rv2_kit::components::model_failover_flow::{Attempt, AttemptState, End, ModelFailoverFlow};
use rv2_kit::components::traffic_split_router::{
    Fallback, Policy, Route, Strategy, TrafficSplitRouter,
};

use super::Scene;

const MCP_SERVERS: [(&str, &str); 3] = [
    (
        "Calendar",
        "Reads and writes events, asks before creating one",
    ),
    ("Docs search", "Searches the shared Drive folder"),
    ("This computer's files", "Only the Workflows folder"),
];

fn route(id: &str, name: &str, provider: &str, cost: f64, p95: f64) -> Route {
    Route {
        route_id: id.into(),
        route_name: name.into(),
        route_provider: Some(provider.into()),
        route_mark: None,
        route_tone: None,
        cost_per_1k: cost,
        p95_seconds: p95,
    }
}

fn routes() -> Vec<Route> {
    vec![
        route("fast", "Fast tier", "Anthropic", 0.25, 1.1),
        route("balanced", "Balanced tier", "Anthropic", 3.0, 2.8),
        route("quality", "Quality tier", "OpenAI", 15.0, 6.2),
        route(
            "local",
            "Offline model",
            "llama-server, this computer",
            0.0,
            4.0,
        ),
    ]
}

fn strategies() -> Vec<Strategy> {
    vec![
        Strategy {
            strategy_id: "tiers".into(),
            strategy_label: "Tiers by task".into(),
            share_map: vec![
                ("fast".into(), 0.5),
                ("balanced".into(), 0.4),
                ("quality".into(), 0.1),
            ],
        },
        Strategy {
            strategy_id: "cheap".into(),
            strategy_label: "Cheapest first".into(),
            share_map: vec![("fast".into(), 0.8), ("balanced".into(), 0.2)],
        },
    ]
}

fn live_policy() -> Policy {
    Policy {
        active_strategy: "tiers".into(),
        fallback: Fallback {
            enabled: true,
            from_route_id: Some("balanced".into()),
            to_route_id: Some("local".into()),
        },
    }
}

fn attempt(
    id: &str,
    model: &str,
    provider: &str,
    state: AttemptState,
    error: Option<&str>,
    duration_ms: f64,
    cost: f64,
) -> Attempt {
    Attempt {
        attempt_id: id.into(),
        model_name: model.into(),
        provider_name: provider.into(),
        attempt_state: state,
        duration: Some(duration_ms),
        attempt_cost: Some(cost),
        error_text: error.map(str::to_string),
    }
}

#[component]
pub fn ModelsAndTools() -> Element {
    let mut retried = use_signal(|| false);
    let mut servers = use_signal(|| [true, true, false]);
    let mut monthly = use_signal(|| "500".to_string());
    let mut warn_at_80 = use_signal(|| true);
    let mut last = use_signal(|| "no change saved".to_string());
    let retry_now = retried();
    let server_now = servers();
    let enabled = server_now.iter().filter(|on| **on).count();

    let attempts = if retry_now {
        vec![attempt(
            "a1",
            "Balanced tier",
            "Anthropic",
            AttemptState::Answered,
            None,
            2_100.0,
            0.018,
        )]
    } else {
        vec![
            attempt(
                "a1",
                "Balanced tier",
                "Anthropic",
                AttemptState::Failed,
                Some("429 rate limited"),
                300.0,
                0.0,
            ),
            attempt(
                "a2",
                "Quality tier",
                "OpenAI",
                AttemptState::Failed,
                Some("Timed out"),
                3_000.0,
                0.0,
            ),
            attempt(
                "a3",
                "Offline model",
                "llama-server",
                AttemptState::Answered,
                None,
                2_800.0,
                0.0,
            ),
        ]
    };

    rsx! {
        Scene {
            id: "settings",
            title: "Models and tools",
            story: "Ravi sets up where Longhand sends work: a split across tiers, an offline fallback when the cloud is down, tool servers he allows, and a monthly budget that warns at 80%.",
            div { class: "lh-stack",
                div { class: "lh-grid",
                    section { class: "lh-card", "aria-label": "Routing",
                        TrafficSplitRouter {
                            route_list: routes(),
                            strategy_list: strategies(),
                            live_policy: live_policy(),
                            environment_label: "Production",
                            heading: "Where work goes",
                            on_deploy: move |p: Policy| last.set(format!("deployed the {} split", p.active_strategy)),
                            on_draft_change: move |p: Policy| last.set(format!("draft: {}", p.active_strategy)),
                        }
                    }
                    section { class: "lh-card", "aria-label": "Failover",
                        h3 { class: "lh-h", "What happens when a provider fails" }
                        ModelFailoverFlow {
                            attempt_list: attempts,
                            heading_text: "Refund reply, run 14",
                            routing_label: Some("Cloud first, then offline".to_string()),
                            origin_node: Some(End { node_name: "Request".into(), node_mark: None, node_text: Some("Refund reply drafted".into()), node_details: vec![] }),
                            result_node: Some(End { node_name: "Reply".into(), node_mark: None, node_text: Some("Delivered to Sam for review".into()), node_details: vec!["Offline model".to_string()] }),
                            elapsed_ms: Some(if retry_now { 1_300.0 } else { 6_100.0 }),
                            on_retry: move |_| {
                                retried.set(true);
                                last.set("retried the reply".to_string());
                            },
                        }
                    }
                }
                div { class: "lh-grid",
                    section { class: "lh-card", "aria-label": "Budget limits",
                        h3 { class: "lh-h", "Budget limits" }
                        label { class: "lh-field", "Monthly limit (credits)",
                            input { class: "lh-input", r#type: "text", value: "{monthly}",
                                oninput: move |e| monthly.set(e.value()),
                            }
                        }
                        label { class: "lh-check",
                            input { r#type: "checkbox", checked: warn_at_80(),
                                onchange: move |e| warn_at_80.set(e.checked()),
                            }
                            "Warn at 80% of the limit"
                        }
                        p { class: "lh-muted", "Workflows pause at 100% and wait for you to top up." }
                    }
                    section { class: "lh-card", "aria-label": "Tool servers",
                        h3 { class: "lh-h", "Tool servers ({enabled} of {MCP_SERVERS.len()} on)" }
                        for (index, (name, note)) in MCP_SERVERS.iter().enumerate() {
                            label { class: "lh-check", key: "{name}",
                                input { r#type: "checkbox", checked: server_now[index],
                                    onchange: move |e| servers.with_mut(|s| s[index] = e.checked()),
                                }
                                span { class: "lh-stack", style: "gap: 0;",
                                    span { "{name}" }
                                    span { class: "lh-muted", "{note}" }
                                }
                            }
                        }
                    }
                }
                section { class: "lh-card", "aria-label": "Google Workspace bridge",
                    ConnectorSettingsCard {
                        service_name: "Google Workspace bridge",
                        item_count: Some(2),
                        subtitle_text: "Reads the Drive folder and the shared calendar for workflows you start",
                        site_url_hint: "https://drive.example.com/bramble",
                        time_zone_options: vec!["Europe/Bucharest".to_string(), "UTC".to_string()],
                        initial_values: ConnectorValues { is_enabled: true, site_url: "https://drive.example.com/bramble".into(), time_zone: "Europe/Bucharest".into(), api_key: String::new() },
                        on_save: move |v: ConnectorValues| last.set(format!("saved the bridge for {}", v.site_url)),
                        on_enabled_change: move |on: bool| last.set(format!("bridge enabled: {on}")),
                    }
                }
                p { class: "gl-out", "data-out": "settings", "{last()}" }
            }
        }
    }
}
