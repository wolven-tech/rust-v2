//! Small inline AI helper: a welcome view with prompt chips, a chat of replies, a field, and dictation.
//!
//! Design choice: a Rust callback cannot return a promise. `on_message` reports each sent message, and
//! the host answers by setting `reply` to the outcome for the latest message. Each new value is appended once.
//! Design choice: `reply` carries `Ok(text)` for an answer and `Err(text)` for a failure. A failure shows as an
//! error line in the conversation and leaves the field usable.
//! Design choice: `speech_mode` `Auto` only marks listening; real recognition is a host concern. `Simulate`
//! types `sample_dictation` into the field on one press of the microphone, then stops listening by itself.
//! Browser checks: the welcome view hands over to the chat with a short rise; the listening ring breathes;
//! the glow and shimmer freeze under reduced motion.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id, sleep};

const CSS: Asset = asset!("/assets/inline-drafting-assistant.css");
/// How long the simulated take listens before the sample text lands in the field.
const SIMULATED_TAKE_MS: u32 = 900;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpeechMode {
    #[default]
    Auto,
    Simulate,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    #[default]
    User,
    Assistant,
    Error,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ChatMessage {
    pub role: Role,
    pub text: String,
}

/// Outcome of a message: the reply text, or an error description.
#[derive(Clone, PartialEq, Debug)]
pub enum ReplyResult {
    Ok(String),
    Err(String),
}

/// Ready-to-send sentence after a dictation; `None` when the text is blank.
pub fn dictation_message(sentence: &str) -> Option<String> {
    let trimmed = sentence.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn role_attr(role: Role) -> &'static str {
    match role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Error => "error",
    }
}

pub fn reply_message(reply: &ReplyResult) -> ChatMessage {
    match reply {
        ReplyResult::Ok(text) => ChatMessage {
            role: Role::Assistant,
            text: text.clone(),
        },
        ReplyResult::Err(text) => ChatMessage {
            role: Role::Error,
            text: text.clone(),
        },
    }
}

/// Assistant card with welcome view, conversation and a prompt field.
#[component]
pub fn InlineDraftingAssistant(
    #[props(default = "Quill".to_string(), into)] assistant_name: String,
    #[props(default, into)] assistant_tagline: String,
    #[props(default, into)] greeting_text: String,
    #[props(default)] prompt_chips: Vec<String>,
    #[props(default = "Ask Quill to rewrite, shorten or translate".to_string(), into)]
    input_hint: String,
    #[props(default)] on_message: Option<EventHandler<String>>,
    #[props(default)] reply: Option<ReplyResult>,
    #[props(default)] speech_mode: SpeechMode,
    #[props(default, into)] sample_dictation: String,
    #[props(default, into)] root_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-ida-{}", next_id()));
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut draft = use_signal(String::new);
    let mut listening = use_signal(|| false);
    let mut waiting = use_signal(|| false);
    let mut seen_reply = use_signal(|| Option::<ReplyResult>::None);

    use_effect(use_reactive((&reply,), move |(incoming,)| {
        if let Some(outcome) = incoming {
            if seen_reply.peek().as_ref() != Some(&outcome) {
                messages.with_mut(|list| list.push(reply_message(&outcome)));
                seen_reply.set(Some(outcome));
                waiting.set(false);
            }
        }
    }));

    let has_messages = !messages.read().is_empty();
    let listening_now = listening();
    let waiting_now = waiting();
    let can_send = !draft().trim().is_empty();
    let root_class = if root_class.is_empty() {
        "kit-ida".to_string()
    } else {
        format!("kit-ida {root_class}")
    };
    let mic_label = if listening_now {
        "Stop dictation"
    } else {
        "Start dictation"
    };
    let tile = assistant_name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    let transcript: Vec<ChatMessage> = messages.read().clone();
    let chips = prompt_chips.clone();

    let mut send_text = {
        let handler = on_message.clone();
        move |text: String| {
            messages.with_mut(|list| {
                list.push(ChatMessage {
                    role: Role::User,
                    text: text.clone(),
                })
            });
            waiting.set(true);
            if let Some(h) = &handler {
                h.call(text);
            }
        }
    };
    let mut send_draft = send_text.clone();
    let send_chip = send_text.clone();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { id: "{id}", class: "{root_class}", aria_label: "{assistant_name}", "data-view": if has_messages { "chat" } else { "welcome" },
            header { class: "kit-ida__head",
                span { class: "kit-ida__tile", aria_hidden: "true", "{tile}" }
                span { class: "kit-ida__name", "{assistant_name}" }
                span { class: "kit-ida__tagline", "{assistant_tagline}" }
                button {
                    r#type: "button",
                    class: "kit-ida__new",
                    onclick: move |_| {
                        messages.set(Vec::new());
                        waiting.set(false);
                        draft.set(String::new());
                    },
                    "New chat"
                }
            }
            if has_messages {
                ol { class: "kit-ida__log", role: "log", aria_live: "polite",
                    for (index, (role, text)) in transcript.into_iter().map(|m| (role_attr(m.role), m.text)).enumerate() {
                        li { key: "{index}", class: "kit-ida__msg", "data-role": role,
                            "{text}"
                        }
                    }
                    if waiting_now {
                        li { class: "kit-ida__msg kit-ida__pending", "Writing…" }
                    }
                }
            } else {
                div { class: "kit-ida__welcome",
                    p { class: "kit-ida__greeting", "{greeting_text}" }
                    div { class: "kit-ida__chips",
                        for (index, chip) in chips.into_iter().enumerate() {
                            button {
                                key: "{index}",
                                r#type: "button",
                                class: "kit-ida__chip",
                                onclick: {
                                    let text = chip.clone();
                                    let mut send = send_chip.clone();
                                    move |_| send(text.clone())
                                },
                                "{chip}"
                            }
                        }
                    }
                }
            }
            div { class: "kit-ida__composer",
                textarea {
                    class: "kit-ida__input",
                    rows: "2",
                    value: "{draft}",
                    placeholder: "{input_hint}",
                    aria_label: "Message",
                    oninput: move |event: FormEvent| draft.set(event.value()),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Enter && !event.modifiers().shift() {
                            event.prevent_default();
                            let text = draft().trim().to_string();
                            if !text.is_empty() {
                                draft.set(String::new());
                                send_draft(text);
                            }
                        }
                    },
                }
                div { class: "kit-ida__tools",
                    button {
                        r#type: "button",
                        class: "kit-ida__mic",
                        "aria-pressed": if listening_now { "true" } else { "false" },
                        aria_label: "{mic_label}",
                        onclick: move |_| {
                            if speech_mode == SpeechMode::Simulate {
                                if let Some(text) = dictation_message(&sample_dictation) {
                                    listening.set(true);
                                    spawn(async move {
                                        sleep(SIMULATED_TAKE_MS).await;
                                        draft.set(text);
                                        listening.set(false);
                                    });
                                }
                                return;
                            }
                            listening.set(!listening());
                        },
                        span { class: "kit-ida__ring", aria_hidden: "true" }
                        "Mic"
                    }
                    button {
                        r#type: "button",
                        class: "kit-ida__send",
                        disabled: !can_send,
                        aria_label: "Send",
                        onclick: move |_| {
                            let text = draft().trim().to_string();
                            if !text.is_empty() {
                                draft.set(String::new());
                                send_text(text);
                            }
                        },
                        "Send"
                    }
                }
            }
        }
    }
}
