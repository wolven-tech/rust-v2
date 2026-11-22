use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::components::fill_in_prompt_templates::{BlankSpec, FillInPromptTemplates, Template};
use rv2_kit::components::model_selector::{ModelOption, ModelSelector};

use super::Scene;

const EXAMPLES: [(&str, &str, &str); 3] = [
    (
        "Juniper Labs renewal call",
        "Renewal in March, wants CSV export, budget is tight.",
        "three bullets, one decision",
    ),
    (
        "Harbor Dental onboarding",
        "Kickoff done, two admins need training next week.",
        "three bullets, one owner",
    ),
    (
        "Field Co. pricing question",
        "Asked for a discount on seats, no decision yet.",
        "three bullets, one open question",
    ),
];

const TEMPLATE_BODY: &str =
    "Summarise the sales call with {client_name} in three bullets. Call notes: {call_notes}";

fn tier_options() -> Vec<ModelOption> {
    vec![
        ModelOption {
            option_id: "fast".into(),
            option_name: "Fast".into(),
            option_note: Some("Short answers, lowest cost".into()),
            option_icon: None,
            speed_rank: Some(5),
            quality_rank: Some(2),
            cost_rank: Some(1),
            first_token_time: Some(0.4),
            price_per_million: Some(0.25),
        },
        ModelOption {
            option_id: "balanced".into(),
            option_name: "Balanced".into(),
            option_note: Some("Default for most skills".into()),
            option_icon: None,
            speed_rank: Some(3),
            quality_rank: Some(4),
            cost_rank: Some(3),
            first_token_time: Some(0.9),
            price_per_million: Some(3.0),
        },
        ModelOption {
            option_id: "quality".into(),
            option_name: "Quality".into(),
            option_note: Some("Longest reasoning, highest cost".into()),
            option_icon: None,
            speed_rank: Some(1),
            quality_rank: Some(5),
            cost_rank: Some(5),
            first_token_time: Some(2.1),
            price_per_million: Some(15.0),
        },
    ]
}

fn example_passes(tier: &str, index: usize) -> bool {
    match tier {
        "fast" => index != 1,
        _ => true,
    }
}

fn template() -> Template {
    let blank = |hint: &str| BlankSpec {
        hint_text: Some(hint.into()),
        fallback_value: None,
    };
    Template {
        template_id: "summarise-call".into(),
        template_title: "Summarise a sales call".into(),
        group_label: Some("Sales".into()),
        group_key: Some("sales".into()),
        template_body: TEMPLATE_BODY.into(),
        blank_specs: BTreeMap::from([
            ("client_name".to_string(), blank("Client name")),
            ("call_notes".to_string(), blank("Paste the call notes")),
        ]),
    }
}

#[component]
pub fn SkillEditor() -> Element {
    let mut tier = use_signal(|| "balanced".to_string());
    let mut results = use_signal(|| None::<Vec<bool>>);
    let mut body = use_signal(|| TEMPLATE_BODY.to_string());
    let mut tried = use_signal(|| "Fill in the variables and try the skill.".to_string());
    let tier_now = tier();
    let tier_label = match tier_now.as_str() {
        "fast" => "Fast",
        "quality" => "Quality",
        _ => "Balanced",
    };
    let passed = results().map(|r| r.iter().filter(|ok| **ok).count());

    rsx! {
        Scene {
            id: "skill_editor",
            title: "Skill editor",
            story: "Priya writes a reusable skill for sales calls: two variables, three worked examples, a preferred tier, and a test run before it goes into a workflow.",
            div { class: "lh-grid",
                section { class: "lh-card", "aria-label": "Skill",
                    div { class: "lh-between",
                        h3 { class: "lh-h", "Summarise a sales call" }
                        span { class: "lh-chip", "Used in 2 workflows" }
                    }
                    label { class: "lh-field", "Instructions with variables"
                        textarea { class: "lh-input",
                            value: "{body}",
                            oninput: move |e| body.set(e.value()),
                        }
                    }
                    div { class: "lh-row",
                        span { class: "lh-muted", "Variables:" }
                        span { class: "lh-chip", "{{client_name}}" }
                        span { class: "lh-chip", "{{call_notes}}" }
                    }
                    h3 { class: "lh-h", "Examples" }
                    ul { class: "lh-list",
                        for (name, _, expected) in EXAMPLES {
                            li { key: "{name}",
                                span { "{name}" }
                                span { class: "lh-muted", "expects {expected}" }
                            }
                        }
                    }
                }
                div { class: "lh-stack",
                    section { class: "lh-card", "aria-label": "Preferred model tier",
                        h3 { class: "lh-h", "Preferred tier" }
                        ModelSelector {
                            model_options: tier_options(),
                            selected_model_id: Some(tier_now.clone()),
                            on_model_change: move |id: String| {
                                tier.set(id);
                                results.set(None);
                            },
                        }
                        p { class: "lh-muted", "Routing uses this tier first and falls back to a local model when the provider is offline." }
                    }
                    section { class: "lh-card", "aria-label": "Test runs",
                        div { class: "lh-between",
                            h3 { class: "lh-h", "Test runs" }
                            button { class: "lh-btn lh-btn--primary", r#type: "button",
                                onclick: move |_| {
                                    let current = tier();
                                    results.set(Some((0..EXAMPLES.len()).map(|i| example_passes(&current, i)).collect()));
                                },
                                "Run examples"
                            }
                        }
                        ul { class: "lh-list",
                            for (index, (name, _, expected)) in EXAMPLES.iter().enumerate() {
                                li { key: "{name}",
                                    span { "{name}" }
                                    {match results() {
                                        Some(r) if r[index] => rsx! { span { class: "lh-chip", "data-tone": "ok", "passed" } },
                                        Some(_) => rsx! { span { class: "lh-chip", "data-tone": "fault", "missed, expected {expected}" } },
                                        None => rsx! { span { class: "lh-chip", "not run" } },
                                    }}
                                }
                            }
                        }
                        p { class: "lh-muted",
                            {match passed {
                                Some(n) => format!("{n} of {} examples passed on the {tier_label} tier.", EXAMPLES.len()),
                                None => "Run the examples to check this tier.".to_string(),
                            }}
                        }
                    }
                }
            }
            section { class: "lh-card", "aria-label": "Try the skill", style: "margin-top: 16px;",
                h3 { class: "lh-h", "Try it with a call" }
                FillInPromptTemplates {
                    template_list: vec![template()],
                    selected_id: Some("summarise-call".to_string()),
                    on_select: move |_id: String| {},
                    on_submit: move |(_, _, fills): (String, Template, BTreeMap<String, String>)| {
                        let client = fills.get("client_name").cloned().unwrap_or_default();
                        tried.set(format!("Ran the skill for {client} on the {tier_label} tier."));
                    },
                }
                p { class: "gl-out", "data-out": "skill-editor", "{tried()}" }
            }
        }
    }
}
