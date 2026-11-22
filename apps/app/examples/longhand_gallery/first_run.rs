use dioxus::prelude::*;
use rv2_kit::components::onboarding_checklist_panel::{
    CheckGroup, CheckItem, OnboardingChecklistPanel,
};

use super::Scene;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Path {
    Cloud,
    Offline,
}

const PROVIDERS: [&str; 3] = ["Anthropic", "OpenAI", "Google AI"];

fn check(id: &str, text: &str, done: bool, locked: bool) -> CheckItem {
    CheckItem {
        item_id: id.into(),
        item_text: text.into(),
        is_done: done,
        is_locked: locked,
    }
}

fn checklist() -> Vec<CheckGroup> {
    vec![
        CheckGroup {
            group_id: "setup".into(),
            group_title: "Set up".into(),
            task_list: vec![
                check(
                    "model",
                    "Connect a model, or load an offline one",
                    false,
                    false,
                ),
                check("skill", "Try one skill on a real file", false, false),
                check("team", "Invite a teammate", false, false),
            ],
        },
        CheckGroup {
            group_id: "safety".into(),
            group_title: "Safety".into(),
            task_list: vec![
                check("review", "Choose which steps wait for you", true, false),
                check("budget", "Set a monthly budget", false, true),
            ],
        },
    ]
}

#[component]
pub fn FirstRun() -> Element {
    let mut path = use_signal(|| None::<Path>);
    let mut provider = use_signal(|| 0usize);
    let mut key_text = use_signal(String::new);
    let mut connected = use_signal(|| None::<String>);
    let mut offline_ready = use_signal(|| false);
    let mut learning = use_signal(|| true);
    let mut last = use_signal(|| "nothing checked yet".to_string());
    let chosen = path();
    let is_cloud = chosen == Some(Path::Cloud);
    let is_offline = chosen == Some(Path::Offline);
    let provider_now = provider();
    let learning_now = learning();

    rsx! {
        Scene {
            id: "first_run",
            title: "First run and Learning Mode",
            story: "Mohammed has never used an AI tool. The first screen asks one question, connects a provider or loads an offline model, and keeps models, tokens and tool servers hidden until he asks for them.",
            div { class: "lh-grid",
                section { class: "lh-card", "aria-label": "Choose how to start",
                    h3 { class: "lh-h", "How should Longhand think?" }
                    p { class: "lh-muted", "Pick one. You can change it later in Models and tools." }
                    div { class: "lh-row",
                        button { class: "lh-btn", r#type: "button", aria_pressed: "{is_cloud}",
                            onclick: move |_| path.set(Some(Path::Cloud)),
                            "Use a cloud provider"
                        }
                        button { class: "lh-btn", r#type: "button", aria_pressed: "{is_offline}",
                            onclick: move |_| path.set(Some(Path::Offline)),
                            "Work offline on this computer"
                        }
                    }
                    if is_cloud {
                        div { class: "lh-stack", style: "gap: 10px;",
                            div { class: "lh-row",
                                for (index, name) in PROVIDERS.iter().enumerate() {
                                    button { class: "lh-btn", r#type: "button", key: "{name}",
                                        aria_pressed: "{index == provider_now}",
                                        onclick: move |_| provider.set(index),
                                        "{name}"
                                    }
                                }
                            }
                            label { class: "lh-field", "API key for {PROVIDERS[provider_now]}",
                                input { class: "lh-input", r#type: "password", autocomplete: "off",
                                    value: "{key_text}",
                                    oninput: move |e| key_text.set(e.value()),
                                }
                            }
                            div { class: "lh-row",
                                button { class: "lh-btn lh-btn--primary", r#type: "button",
                                    disabled: key_text().trim().is_empty(),
                                    onclick: move |_| {
                                        let name = PROVIDERS[provider()];
                                        connected.set(Some(name.to_string()));
                                        last.set(format!("connected {name}"));
                                    },
                                    "Save key"
                                }
                                if let Some(name) = connected() {
                                    span { class: "lh-chip", "data-tone": "ok", "Connected to {name}" }
                                }
                            }
                        }
                    }
                    if is_offline {
                        div { class: "lh-stack", style: "gap: 10px;",
                            p { class: "lh-muted", "llama-server found on this computer at 127.0.0.1:8080, with a 3B model." }
                            div { class: "lh-row",
                                button { class: "lh-btn lh-btn--primary", r#type: "button",
                                    onclick: move |_| {
                                        offline_ready.set(true);
                                        last.set("loaded the offline model".to_string());
                                    },
                                    "Load the offline model"
                                }
                                if offline_ready() {
                                    span { class: "lh-chip", "data-tone": "ok", "Ready, no internet needed" }
                                }
                            }
                        }
                    }
                }
                section { class: "lh-card", "aria-label": "Learning Mode",
                    h3 { class: "lh-h", "Learning Mode" }
                    label { class: "lh-check",
                        input { r#type: "checkbox", checked: learning_now,
                            onchange: move |e| learning.set(e.checked()),
                        }
                        "Hide models, tokens and tool servers until I ask"
                    }
                    if learning_now {
                        p { class: "lh-muted", "Longhand shows the job, the draft and the review. Ask \"which model wrote this?\" to see the details." }
                    } else {
                        ul { class: "lh-list",
                            li { span { "Model" } span { class: "lh-muted", "Balanced tier, Anthropic" } }
                            li { span { "Tokens this run" } span { class: "lh-muted", "5,040 in, 640 out" } }
                            li { span { "Tool servers" } span { class: "lh-muted", "Mail, Docs search" } }
                        }
                    }
                }
            }
            section { class: "lh-card", "aria-label": "Setup checklist", style: "margin-top: 16px;",
                OnboardingChecklistPanel {
                    group_list: checklist(),
                    on_change: move |ids: Vec<String>| last.set(format!("checked {}", ids.join(", "))),
                    on_dismiss: move |_| last.set("checklist hidden for now".to_string()),
                }
            }
            p { class: "gl-out", "data-out": "first-run", "{last()}" }
        }
    }
}
