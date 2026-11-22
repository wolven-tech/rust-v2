use dioxus::prelude::*;
use rv2_kit::components::automation_flow_editor::{
    AutomationFlowEditor, Step, StepIcon, StepKind, Template,
};

use super::Scene;

const DISPOSITIONS: [&str; 5] = [
    "Store",
    "Send by email",
    "Publish via webhook",
    "Pipe to next step",
    "Trigger another workflow",
];

const OUTPUTS: [&str; 3] = ["Pipeline summary", "Renewal risk list", "Owner emails"];

fn step(id: &str, kind: StepKind, title: &str, detail: &str, icon: StepIcon) -> Step {
    Step {
        step_id: id.into(),
        step_kind: kind,
        step_title: title.into(),
        step_detail: Some(detail.into()),
        step_note: None,
        step_icon: Some(icon),
        branch_paths: vec![],
    }
}

fn template(kind: StepKind, title: &str, icon: StepIcon) -> Template {
    Template {
        step_kind: kind,
        step_title: title.into(),
        step_detail: Some("Added from the step library".into()),
        step_icon: Some(icon),
        branch_paths: vec![],
    }
}

fn flow_steps() -> Vec<Step> {
    vec![
        step(
            "s1",
            StepKind::Trigger,
            "Every Monday at 08:00",
            "Starts with last week's deals",
            StepIcon::Schedule,
        ),
        step(
            "s2",
            StepKind::Agent,
            "Pull open deals from the CRM",
            "Skill: List open deals by owner",
            StepIcon::Database,
        ),
        step(
            "s3",
            StepKind::Branch,
            "Any deal over 10,000?",
            "Large deals get a closer look",
            StepIcon::Flash,
        ),
        step(
            "s4",
            StepKind::Action,
            "Review point: Sam checks the list",
            "Waits here until Sam approves",
            StepIcon::Form,
        ),
        step(
            "s5",
            StepKind::Agent,
            "Summarise by owner",
            "Skill: Summarise pipeline",
            StepIcon::Chat,
        ),
        step(
            "s6",
            StepKind::Action,
            "Publish the summary via webhook",
            "Posts to the team board",
            StepIcon::Webhook,
        ),
    ]
}

fn step_templates() -> Vec<Template> {
    vec![
        template(StepKind::Action, "Add a review point", StepIcon::Form),
        template(StepKind::Action, "Send by email", StepIcon::Mail),
        template(StepKind::Wait, "Wait for a reply", StepIcon::Message),
        template(
            StepKind::Action,
            "Trigger another workflow",
            StepIcon::Flash,
        ),
    ]
}

#[component]
pub fn WorkflowBuilder() -> Element {
    let mut routes = use_signal(|| [0usize, 0, 2]);
    let mut review_first = use_signal(|| true);
    let mut chain_renewals = use_signal(|| false);
    let mut budget = use_signal(|| "2.00".to_string());
    let mut saved = use_signal(|| "Draft. Not running yet.".to_string());
    let review_now = review_first();
    let chain_now = chain_renewals();
    let route_now = routes();

    rsx! {
        Scene {
            id: "workflow_builder",
            title: "Workflow builder",
            story: "Ana builds Monday pipeline review: a schedule, two skills, one review point, three outputs routed to different places, and a hand-off to the renewal workflow.",
            div { class: "lh-box", style: "height: 620px;",
                AutomationFlowEditor {
                    flow_name: "Monday pipeline review",
                    default_flow_steps: flow_steps(),
                    insert_templates: step_templates(),
                }
            }
            div { class: "lh-grid", style: "margin-top: 16px;",
                section { class: "lh-card", "aria-label": "Where each output goes",
                    h3 { class: "lh-h", "Where each output goes" }
                    for (index, output) in OUTPUTS.iter().enumerate() {
                        label { class: "lh-field", key: "{output}",
                            "{output}"
                            select { class: "lh-input",
                                onchange: move |e| {
                                    let picked = DISPOSITIONS
                                        .iter()
                                        .position(|d| *d == e.value())
                                        .unwrap_or(0);
                                    routes.with_mut(|r| r[index] = picked);
                                },
                                for (choice, name) in DISPOSITIONS.iter().enumerate() {
                                    option { key: "{name}", value: "{name}", selected: choice == route_now[index], "{name}" }
                                }
                            }
                        }
                    }
                }
                section { class: "lh-card", "aria-label": "Review and triggers",
                    h3 { class: "lh-h", "Review and triggers" }
                    label { class: "lh-check",
                        input { r#type: "checkbox", checked: review_now,
                            onchange: move |e| review_first.set(e.checked()),
                        }
                        "Wait for Sam before anything is published"
                    }
                    label { class: "lh-check",
                        input { r#type: "checkbox", checked: chain_now,
                            onchange: move |e| chain_renewals.set(e.checked()),
                        }
                        "When this finishes, start Renewal follow-ups"
                    }
                    label { class: "lh-field",
                        "Budget per run (USD)"
                        input { class: "lh-input", r#type: "text", value: "{budget}",
                            oninput: move |e| budget.set(e.value()),
                        }
                    }
                    p { class: "lh-muted", "A run warns at 80% of this budget and stops at 100%." }
                    div { class: "lh-row",
                        button { class: "lh-btn lh-btn--primary", r#type: "button",
                            onclick: move |_| {
                                let status = if review_first() { "Saved. Publishing waits for Sam." } else { "Saved. Publishing runs without review." };
                                saved.set(status.to_string());
                            },
                            "Save workflow"
                        }
                    }
                    p { class: "gl-out", "data-out": "workflow-builder",
                        if chain_now { "Chains into Renewal follow-ups. " }
                        "{saved()}"
                    }
                }
            }
        }
    }
}
