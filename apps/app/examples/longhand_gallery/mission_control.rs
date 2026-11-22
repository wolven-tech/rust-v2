use dioxus::prelude::*;
use rv2_kit::components::metric_tile::{MetricTile, MetricValue};

use super::Scene;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Running,
    NeedsYou,
    Idle,
}

impl Status {
    fn key(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::NeedsYou => "needs",
            Self::Idle => "idle",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::NeedsYou => "needs you",
            Self::Idle => "idle",
        }
    }
}

const WORKFLOWS: [(&str, &str, &str); 4] = [
    (
        "Monday pipeline review",
        "List open deals, Summarise pipeline",
        "Renewal risk list",
    ),
    (
        "Weekly customer newsletter",
        "Draft newsletter, Check release notes",
        "Newsletter draft",
    ),
    ("Refund replies", "Write refund reply", "Reply queue"),
    (
        "Renewal follow-ups",
        "Draft follow-up, Summarise pipeline",
        "Follow-up emails",
    ),
];

const SKILLS: [(&str, &str); 4] = [
    ("List open deals", "Pipeline review, 1 run today"),
    ("Summarise pipeline", "2 workflows"),
    ("Draft newsletter", "Newsletter, last ran Friday"),
    ("Write refund reply", "Refunds, waiting on review"),
];

#[component]
pub fn MissionControl() -> Element {
    let mut statuses = use_signal(|| {
        [
            Status::Running,
            Status::NeedsYou,
            Status::Idle,
            Status::Idle,
        ]
    });
    let mut last = use_signal(|| "nothing started from here yet".to_string());
    let status_now = statuses();
    let running = status_now.iter().filter(|s| **s == Status::Running).count();
    let waiting = status_now
        .iter()
        .filter(|s| **s == Status::NeedsYou)
        .count();

    rsx! {
        Scene {
            id: "mission_control",
            title: "Mission Control",
            story: "Ravi checks the whole operation in one view: which goal each workflow serves, which skills it calls, and what is running right now. He starts and stops work from the same screen.",
            div { class: "lh-tiles", style: "margin-bottom: 16px;",
                MetricTile {
                    metric_label: "Key results on track",
                    metric_value: MetricValue::Text("2 of 3".to_string()),
                    change_note: Some("Q4 key results".to_string()),
                }
                MetricTile {
                    metric_label: "Workflows running",
                    metric_value: MetricValue::Number(running as f64),
                    change_note: Some("in the background".to_string()),
                }
                MetricTile {
                    metric_label: "Waiting for you",
                    metric_value: MetricValue::Number(waiting as f64),
                    change_note: Some("review points".to_string()),
                }
            }
            div { class: "lh-graph",
                section { class: "lh-stack", "aria-label": "Goals",
                    h3 { class: "lh-h", "Goals" }
                    div { class: "lh-node", "data-state": "running",
                        span { class: "lh-title", "Q4: renewals above 92%" }
                        span { class: "lh-muted", "Renewal rate 88.4%, on track" }
                        span { class: "lh-chip", "data-tone": "ok", "on track" }
                    }
                    div { class: "lh-node",
                        span { class: "lh-title", "Q4: answer support in 4 hours" }
                        span { class: "lh-muted", "Median 3.1 hours" }
                        span { class: "lh-chip", "data-tone": "ok", "on track" }
                    }
                }
                div { class: "lh-arrow", "aria-hidden": "true", "→" }
                section { class: "lh-stack", "aria-label": "Workflows",
                    h3 { class: "lh-h", "Workflows" }
                    for (index, (name, skills, output)) in WORKFLOWS.iter().enumerate() {
                        div { class: "lh-node", key: "{name}", "data-state": "{status_now[index].key()}",
                            div { class: "lh-between",
                                span { class: "lh-title",
                                    span { class: "lh-dot", "data-state": "{status_now[index].key()}" }
                                    "{name}"
                                }
                                span { class: "lh-chip", "{status_now[index].label()}" }
                            }
                            span { class: "lh-muted", "Skills: {skills}" }
                            span { class: "lh-muted", "Output: {output}" }
                            div { class: "lh-row",
                                if status_now[index] == Status::Idle {
                                    button { class: "lh-btn", r#type: "button",
                                        onclick: move |_| {
                                            statuses.with_mut(|s| s[index] = Status::Running);
                                            last.set(format!("started {name}"));
                                        },
                                        "Run now"
                                    }
                                }
                                if status_now[index] == Status::Running {
                                    button { class: "lh-btn", r#type: "button",
                                        onclick: move |_| {
                                            statuses.with_mut(|s| s[index] = Status::Idle);
                                            last.set(format!("stopped {name}"));
                                        },
                                        "Stop"
                                    }
                                }
                                if status_now[index] == Status::NeedsYou {
                                    button { class: "lh-btn lh-btn--primary", r#type: "button",
                                        onclick: move |_| {
                                            statuses.with_mut(|s| s[index] = Status::Idle);
                                            last.set(format!("reviewed {name}"));
                                        },
                                        "Open review"
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "lh-arrow", "aria-hidden": "true", "→" }
                section { class: "lh-stack", "aria-label": "Skills",
                    h3 { class: "lh-h", "Skills" }
                    for (name, note) in SKILLS {
                        div { class: "lh-node", key: "{name}",
                            span { class: "lh-title", "{name}" }
                            span { class: "lh-muted", "{note}" }
                        }
                    }
                }
            }
            p { class: "gl-out", "data-out": "mission-control", "{last()}" }
        }
    }
}
