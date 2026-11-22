use dioxus::prelude::*;
use rv2_kit::components::prompt_input_bar::{
    AttachAction, Model, PromptInputBar, Service, Submission,
};
use rv2_kit::screens::workspace_frame_inset::WorkspaceFrame;

use super::Scene;
use super::data::{account, chats, menu_groups, menu_items};

#[derive(Clone, PartialEq)]
enum Line {
    Me(String),
    Bot(String),
    Proposal,
}

const PROPOSAL_STEPS: [&str; 5] = [
    "Read this week's support threads",
    "Group them by topic",
    "Draft a summary for Sam",
    "Wait for Sam to review the draft",
    "Store it, and email it on Friday",
];

#[component]
pub fn UnifiedChat() -> Element {
    let mut lines = use_signal(|| {
        vec![
            Line::Me(
                "Every Friday, tell me what customers asked for this week, and email it to me."
                    .to_string(),
            ),
            Line::Proposal,
        ]
    });
    let mut created = use_signal(|| false);
    let mut links = use_signal(Vec::<String>::new);
    let mut nav = use_signal(|| "none yet".to_string());
    let thread = lines();
    let is_created = created();

    rsx! {
        Scene {
            id: "chat",
            title: "Unified chat",
            story: "Maya types her goal in plain English. Longhand proposes a workflow with a review point before anything is sent, and she turns it into a real workflow with one click.",
            div { class: "gl-canvas lh-frame",
                WorkspaceFrame {
                    current_item: "New goal".to_string(),
                    workspace_name: "Bramble Studio",
                    account: account(),
                    menu_items: menu_items(),
                    menu_groups: menu_groups(),
                    chat_list: chats(),
                    top_bar_title: rsx! { span { class: "lh-title", "New goal" } },
                    on_navigate: move |label: String| nav.set(label),
                    content: rsx! {
                        div { class: "lh-stack", style: "padding: 16px; height: 100%; box-sizing: border-box; grid-template-rows: minmax(0, 1fr) auto;",
                            div { class: "lh-thread", style: "overflow: auto; align-content: start;",
                                for (index, line) in thread.iter().enumerate() {
                                    {match line {
                                        Line::Me(text) => rsx! {
                                            div { class: "lh-msg lh-msg-me", key: "{index}", "{text}" }
                                        },
                                        Line::Bot(text) => rsx! {
                                            div { class: "lh-msg", key: "{index}", "{text}" }
                                        },
                                        Line::Proposal => rsx! {
                                            section { class: "lh-card", key: "{index}", "aria-label": "Proposed workflow",
                                                h3 { class: "lh-h", "Proposed workflow: Weekly support summary" }
                                                ol { class: "lh-list", style: "list-style: decimal; padding-left: 20px;",
                                                    for step in PROPOSAL_STEPS {
                                                        li { key: "{step}", "{step}" }
                                                    }
                                                }
                                                p { class: "lh-muted", "Runs Fridays at 16:00, uses the balanced tier, and costs about $0.30 a run." }
                                                div { class: "lh-row",
                                                    if is_created {
                                                        span { class: "lh-chip", "data-tone": "ok", "Created" }
                                                    } else {
                                                        button { class: "lh-btn lh-btn--primary", r#type: "button",
                                                            onclick: move |_| {
                                                                created.set(true);
                                                                lines.with_mut(|l| l.push(Line::Bot("Created Weekly support summary. It runs on Fridays and waits for your review.".to_string())));
                                                            },
                                                            "Create workflow"
                                                        }
                                                        button { class: "lh-btn", r#type: "button",
                                                            onclick: move |_| lines.with_mut(|l| l.push(Line::Bot("Which step should change? Name it and I will update the plan.".to_string()))),
                                                            "Change steps"
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                    }}
                                }
                            }
                            PromptInputBar {
                                on_model_pick: move |m: Model| nav.set(format!("model {}", m.model_id)),
                                on_links_change: move |ids: Vec<String>| links.set(ids),
                                linked_service_ids: links(),
                                model_options: vec![
                                    Model { model_id: "balanced".into(), model_name: "Balanced tier".into(), model_mark: None, model_note: Some("Default".into()) },
                                    Model { model_id: "quality".into(), model_name: "Quality tier".into(), model_mark: None, model_note: None },
                                ],
                                folder_options: vec!["Clients".to_string(), "Newsletter".to_string()],
                                selected_folder: Some("Clients".to_string()),
                                service_options: vec![
                                    Service { service_id: "mail".into(), service_name: "Mail".into(), service_mark: None },
                                    Service { service_id: "crm".into(), service_name: "CRM".into(), service_mark: None },
                                ],
                                voice_sample_text: "Every Friday, summarise support",
                                show_context_row: true,
                                on_submit: move |s: Submission| {
                                    let text = s.text;
                                    lines.with_mut(|l| {
                                        l.push(Line::Me(text));
                                        l.push(Line::Bot("Got it. I will add that as a step after the summary.".to_string()));
                                    });
                                },
                                on_attach_pick: move |a: AttachAction| nav.set(format!("attach {a:?}")),
                            }
                        }
                    },
                }
            }
            p { class: "gl-out", "data-out": "chat", "Last navigation: {nav()}" }
        }
    }
}
