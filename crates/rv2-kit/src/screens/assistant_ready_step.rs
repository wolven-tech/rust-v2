//! Assistant setup, step 4 of 4: a summary of what the assistant learns from and how it behaves, each
//! row with an Edit control back to the step it came from. The preview then goes live.
//!
//! Design choice: "learns from" edits step 1 (the knowledge step), following the step numbers of
//! the setup flow. The spec's acceptance check says 2 for that row, which contradicts its own data
//! table (`on_edit` takes 1 to 3, brand and aliases come from step 2).
//! Design choice: the preview's Live state starts after a fixed glide delay in the browser, and at
//! once under reduced motion. SSR shows Draft.
//! Design choice: the test answer types in after Live; under reduced motion it is shown whole. The
//! preview is hidden from assistive technology, so the status line carries the whole answer once.
//! Browser checks: Edit buttons name their row; the long-value ellipsis shows the full text on
//! hover; Live is announced once.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};

use super::assistant_naming_step::proposed_name;
use super::assistant_persona_step::{ReplyLength, VoiceTone, sample_reply};
use super::knowledge_source_step::{
    DashboardPreview, FrameStyle, OnboardingShell, PreviewSource, SourceStatus,
};
use crate::internal::{KIT_CSS, prefers_motion_script, sleep};

const CSS: Asset = asset!("/assets/assistant-ready-step.css");

/// "example.com", or "example.com and 2 more" for several sources.
pub fn learns_from(domains: &[String]) -> String {
    match domains {
        [] => "No websites yet".to_string(),
        [only] => only.clone(),
        [first, rest @ ..] => format!("{first} and {} more", rest.len()),
    }
}

/// "Friendly, balanced replies".
pub fn voice_summary(tone: VoiceTone, length: ReplyLength) -> String {
    let tone = tone.label();
    let length = length.label().to_ascii_lowercase();
    format!("{tone}, {length} replies")
}

/// The handover row; an empty address means nobody takes over.
pub fn handover_summary(address: &str) -> String {
    let address = address.trim();
    if address.is_empty() {
        "nobody; it answers itself".to_string()
    } else {
        address.to_string()
    }
}

/// The alias row exists only when aliases were given.
pub fn also_answers_to(aliases: &[String]) -> Option<String> {
    let names: Vec<&str> = aliases
        .iter()
        .map(|a| a.trim())
        .filter(|a| !a.is_empty())
        .collect();
    (!names.is_empty()).then(|| names.join(", "))
}

/// The preview's status word.
pub fn status_word(live: bool) -> &'static str {
    if live { "Live" } else { "Draft" }
}

/// One row of the summary: what it is, what it says, and the step its Edit returns to.
#[derive(Clone, Debug, PartialEq)]
struct SummaryRow {
    label: &'static str,
    value: String,
    step: u8,
}

/// Assistant setup, step 4: summary and go-live.
#[component]
pub fn AssistantReadyStep(
    #[props(default)] source_domains: Vec<String>,
    #[props(default, into)] brand: String,
    #[props(default)] aliases: Vec<String>,
    #[props(default, into)] assistant_name: String,
    #[props(default)] voice_tone: VoiceTone,
    #[props(default)] reply_length: ReplyLength,
    #[props(default, into)] handover_to: String,
    on_finish: EventHandler<()>,
    on_edit: EventHandler<u8>,
    #[props(default)] on_back: Option<EventHandler<()>>,
    #[props(default)] on_logout: Option<EventHandler<()>>,
    #[props(default, into)] account_email: String,
    #[props(default = "rv2".to_string(), into)] product_name: String,
    #[props(default)] brand_mark: Option<Element>,
    #[props(default)] frame_style: FrameStyle,
    #[props(default = 4)] step_index: usize,
    #[props(default = 4)] step_count: usize,
) -> Element {
    let brand_name = if brand.trim().is_empty() {
        source_domains
            .first()
            .map(|d| proposed_name(d))
            .unwrap_or_else(|| "Your business".to_string())
    } else {
        brand.trim().to_string()
    };
    let name = if assistant_name.trim().is_empty() {
        format!("{brand_name} Assistant")
    } else {
        assistant_name.trim().to_string()
    };
    let mut live = use_signal(|| false);
    // Characters of the test answer shown so far; `usize::MAX` once it is all shown.
    let mut typed = use_signal(|| 0usize);
    let reply_chars = sample_reply(voice_tone, reply_length).chars().count();
    use_hook(move || {
        spawn(async move {
            let motion = document::eval(&format!("return {};", prefers_motion_script()))
                .await
                .ok()
                .and_then(|value| value.as_bool())
                .unwrap_or(true);
            if motion {
                sleep(900).await;
            }
            live.set(true);
            if !motion {
                typed.set(usize::MAX);
                return;
            }
            let mut count = 0usize;
            while count < reply_chars {
                count = (count + 2).min(reply_chars);
                typed.set(count);
                sleep(30).await;
            }
            typed.set(usize::MAX);
        });
    });
    let id = use_hook(|| format!("kit-rd-{}", crate::internal::next_id()));
    let is_live = live();
    let reply = sample_reply(voice_tone, reply_length);
    let shown_reply: String = reply.chars().take(typed()).collect();
    let mut rows = vec![SummaryRow {
        label: "learns from",
        value: learns_from(&source_domains),
        step: 1,
    }];
    rows.push(SummaryRow {
        label: "name",
        value: name.clone(),
        step: 2,
    });
    if let Some(also) = also_answers_to(&aliases) {
        rows.push(SummaryRow {
            label: "also answers to",
            value: also,
            step: 2,
        });
    }
    rows.push(SummaryRow {
        label: "voice",
        value: voice_summary(voice_tone, reply_length),
        step: 3,
    });
    rows.push(SummaryRow {
        label: "hands over to",
        value: handover_summary(&handover_to),
        step: 3,
    });
    let sources: Vec<PreviewSource> = source_domains
        .iter()
        .map(|domain| PreviewSource {
            domain: domain.clone(),
            status: SourceStatus::Ready,
        })
        .collect();
    let handover_line = if handover_to.trim().is_empty() {
        "Nobody takes over; the assistant answers itself.".to_string()
    } else {
        format!("Hands over to {} when unsure.", handover_to.trim())
    };
    let preview = rsx! {
        DashboardPreview {
            assistant_name: name.clone(),
            status_label: status_word(is_live),
            live: is_live,
            greeting_text: format!("Hi, I'm {name}. How can I help?"),
            tags: aliases.clone(),
            sources,
            conversations: 0usize,
            test_question: "Can I get a refund?".to_string(),
            test_reply: (!shown_reply.is_empty()).then(|| shown_reply.clone()),
            handover_line,
            nav_items: vec!["Overview".to_string(), "Conversations".to_string(), "Knowledge".to_string(), "Settings".to_string()],
        }
    };
    let status_text = if is_live {
        format!("Your assistant is live. Test answer: {reply}")
    } else {
        String::new()
    };
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        OnboardingShell {
            step_index,
            step_count,
            title: "Your assistant is ready",
            prompt: "Check the summary. Edit anything that is wrong, then open your dashboard.",
            account_email,
            product_name,
            brand_mark,
            frame_style,
            on_back,
            on_logout,
            preview,
            div { class: "kit-rd", "data-id": "{id}",
                dl { class: "kit-rd__summary",
                    for row in rows.iter() {
                        {
                            let label = row.label;
                            let value = row.value.clone();
                            let step = row.step;
                            let title_text = value.clone();
                            rsx! {
                                div { key: "{label}", class: "kit-rd__row",
                                    dt { class: "kit-rd__label", "{label}" }
                                    dd { class: "kit-rd__value", title: "{title_text}", "{value}" }
                                    button {
                                        r#type: "button",
                                        class: "kit-rd__edit",
                                        "aria-label": "Edit {label}",
                                        onclick: move |_| on_edit.call(step),
                                        "Edit"
                                    }
                                }
                            }
                        }
                    }
                }
                p { class: "kit-sr", role: "status", "aria-live": "polite", "{status_text}" }
                div { class: "kit-rd__actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        label: "Open dashboard",
                        onclick: move |_| on_finish.call(()),
                    }
                }
            }
        }
    }
}
