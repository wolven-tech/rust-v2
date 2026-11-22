use dioxus::prelude::*;
use rv2_kit::components::grounded_reply::{GroundedReply, SourceEntry};
use rv2_kit::components::inline_edit_review::{InlineEditReview, Segment};

use super::Scene;
use super::data::newsletter_item;

const DISPOSITIONS: [&str; 5] = [
    "Store",
    "Send by email",
    "Publish via webhook",
    "Pipe to next step",
    "Trigger another workflow",
];

#[component]
pub fn ReviewJob() -> Element {
    let mut verdict = use_signal(|| None::<&'static str>);
    let mut revise_note = use_signal(String::new);
    let mut disposition = use_signal(|| 0usize);
    let mut accepted = use_signal(|| None::<String>);
    let mut last = use_signal(|| "no answer chosen yet".to_string());
    let item = newsletter_item();
    let chosen = DISPOSITIONS[disposition()];
    let verdict_now = verdict();
    let is_approved = verdict_now == Some("Approved");
    let is_revise = verdict_now == Some("Revise");
    let is_rejected = verdict_now == Some("Rejected");
    let disposition_now = disposition();

    rsx! {
        Scene {
            id: "review_job",
            title: "Review a Job",
            story: "Sam reviews the Friday newsletter draft: fixes one sentence inline, checks the grounded answer, then approves and picks where the result goes.",
            div { class: "lh-stack",
                div { class: "lh-between",
                    div { class: "lh-stack", style: "gap: 4px;",
                        h3 { class: "lh-h", "Job: {item.item_title}" }
                        p { class: "lh-muted", "Newsletter workflow, run 47, Balanced tier, 2 of 4 steps reviewed" }
                    }
                    div { class: "lh-row",
                        span { class: "lh-chip", "data-tone": "warn", "Needs your review" }
                        span { class: "lh-chip", "Confidence 78%" }
                        span { class: "lh-chip", "Medium risk" }
                    }
                }
                div { class: "lh-grid",
                    section { class: "lh-card", "aria-label": "Draft",
                        h3 { class: "lh-h", "Draft" }
                        InlineEditReview {
                            before_text: "Hi friends, this week the Juniper team got the new export reports. We will ship the update next week. Thanks for reading.",
                            after_text: "Hi friends, this week the Juniper team got the new export reports. We ship the update on Tuesday. Thanks for reading.",
                            segments: vec![
                                Segment::Same("Hi friends, this week the Juniper team got the new export reports. ".to_string()),
                                Segment::Change {
                                    removed: "We will ship the update next week.".to_string(),
                                    inserted: "We ship the update on Tuesday.".to_string(),
                                },
                                Segment::Same(" Thanks for reading.".to_string()),
                            ],
                            caption_text: "Suggested edit: the release notes say Tuesday",
                            on_complete: move |text: String| {
                                last.set("accepted the inline edit".to_string());
                                accepted.set(Some(text));
                            },
                        }
                        if let Some(text) = accepted() {
                            p { class: "lh-muted", "Saved to the draft: {text.len()} characters" }
                        }
                    }
                    section { class: "lh-card", "aria-label": "Plan that made the draft",
                        h3 { class: "lh-h", "How the draft was made" }
                        ul { class: "lh-list",
                            for step in item.plan_steps.iter() {
                                li { key: "{step.step_title}",
                                    span { "{step.step_title}" }
                                    span { class: "lh-muted", "{step.tool_name.clone().unwrap_or_default()}" }
                                }
                            }
                        }
                        p { class: "lh-muted", "Flagged: \"next week\" disagrees with the release notes." }
                    }
                }
                section { class: "lh-card", "aria-label": "Ask about this draft",
                    h3 { class: "lh-h", "Ask about this draft" }
                    GroundedReply {
                        heading_text: "Answer from your sources",
                        source_entries: vec![
                            SourceEntry {
                                entry_id: "s1".into(),
                                chip_text: "Support thread 412".into(),
                                link_target: None,
                                leading_mark: None,
                                snippet: Some("Three customers asked for CSV export on Oct 14.".into()),
                            },
                            SourceEntry {
                                entry_id: "s2".into(),
                                chip_text: "Release notes 4.2".into(),
                                link_target: None,
                                leading_mark: None,
                                snippet: Some("Export reports ship Tuesday 24 November.".into()),
                            },
                        ],
                        answer_text: "Three customers asked for CSV export in October [1], and the release is set for Tuesday 24 November [2].",
                        reveal_animated: false,
                        next_questions: vec!["Who asked for the export?".to_string()],
                        on_next_question_chosen: move |q: String| last.set(format!("next question: {q}")),
                        on_copy_requested: move |t: String| last.set(format!("copied {} characters", t.len())),
                        on_retry_requested: move |_| last.set("asked to retry the answer".to_string()),
                    }
                }
                div { class: "lh-grid",
                    section { class: "lh-card", "aria-label": "Decision",
                        h3 { class: "lh-h", "Decide" }
                        div { class: "lh-row",
                            button { class: "lh-btn", r#type: "button",
                                aria_pressed: "{is_approved}",
                                onclick: move |_| verdict.set(Some("Approved")),
                                "Approve"
                            }
                            button { class: "lh-btn", r#type: "button",
                                aria_pressed: "{is_revise}",
                                onclick: move |_| verdict.set(Some("Revise")),
                                "Revise"
                            }
                            button { class: "lh-btn", r#type: "button",
                                aria_pressed: "{is_rejected}",
                                onclick: move |_| verdict.set(Some("Rejected")),
                                "Reject"
                            }
                        }
                        if is_revise {
                            label { class: "lh-field",
                                "What should change?"
                                textarea { class: "lh-input",
                                    value: "{revise_note}",
                                    oninput: move |e| revise_note.set(e.value()),
                                }
                            }
                        }
                        p { class: "lh-muted",
                            {match verdict_now {
                                Some("Approved") => format!("Approved. It goes to: {chosen}."),
                                Some("Revise") => "Sent back to the workflow with your note.".to_string(),
                                Some(_) => "Rejected. The workflow will not store this draft.".to_string(),
                                None => "Nothing decided yet.".to_string(),
                            }}
                        }
                    }
                    section { class: "lh-card", "aria-label": "Where the result goes",
                        h3 { class: "lh-h", "Where the result goes" }
                        div { class: "lh-tabs", role: "group", "aria-label": "Disposition",
                            for (index, name) in DISPOSITIONS.iter().enumerate() {
                                button { class: "lh-btn", r#type: "button", key: "{name}",
                                    aria_pressed: "{index == disposition_now}",
                                    onclick: move |_| disposition.set(index),
                                    "{name}"
                                }
                            }
                        }
                        p { class: "gl-out", "data-out": "review-job", "{last()}" }
                    }
                }
            }
        }
    }
}
