//! Assistant setup, step 3 of 4: the assistant's name, how it sounds, how long its replies run, and
//! whether it hands over to a person when unsure. A test chat in the preview answers under those
//! choices.
//!
//! Design choice: the test reply is chosen from tone and length by `sample_reply`, a fixed set of
//! sentences, so the preview is the same on every render and under reduced motion.
//! Design choice: tone and length are native radio groups, so arrow keys choose and move focus.
//! Browser checks: changing tone re-renders the reply at once; the address field is disabled while
//! the handover switch is off; Enter in the name field continues when the form is valid.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::switch::Switch;

use super::assistant_naming_step::proposed_name;
use super::knowledge_source_step::{DashboardPreview, FrameStyle, OnboardingShell};
use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/assistant-persona-step.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoiceTone {
    Friendly,
    #[default]
    Neutral,
    Formal,
}

impl VoiceTone {
    pub const ALL: [Self; 3] = [Self::Friendly, Self::Neutral, Self::Formal];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Friendly => "friendly",
            Self::Neutral => "neutral",
            Self::Formal => "formal",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Friendly => "Friendly",
            Self::Neutral => "Neutral",
            Self::Formal => "Formal",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReplyLength {
    Short,
    #[default]
    Balanced,
    Detailed,
}

impl ReplyLength {
    pub const ALL: [Self; 3] = [Self::Short, Self::Balanced, Self::Detailed];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Balanced => "balanced",
            Self::Detailed => "detailed",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Short => "Short",
            Self::Balanced => "Balanced",
            Self::Detailed => "Detailed",
        }
    }
}

/// Values handed to `on_continue`.
#[derive(Clone, Debug, PartialEq)]
pub struct PersonaValues {
    pub assistant_name: String,
    pub voice_tone: VoiceTone,
    pub reply_length: ReplyLength,
    pub handover: bool,
    pub handover_email: String,
}

/// The handover address proposed for a website: support at its domain.
pub fn default_handover_email(domain: &str) -> String {
    let domain = domain.trim();
    if domain.is_empty() {
        String::new()
    } else {
        format!("support@{domain}")
    }
}

/// An address with a local part and a dotted domain whose labels are all non-empty.
pub fn handover_email_valid(address: &str) -> bool {
    let Some((local, host)) = address.split_once('@') else {
        return false;
    };
    let spaced = |text: &str| text.chars().any(char::is_whitespace);
    !local.is_empty()
        && !spaced(address)
        && host.contains('.')
        && host.split('.').all(|label| !label.is_empty())
}

/// Continue is blocked without a name, and, while handover is on, without a valid address.
pub fn can_continue(name: &str, handover: bool, email: &str) -> bool {
    !name.trim().is_empty() && (!handover || handover_email_valid(email.trim()))
}

/// The test chat's reply in the chosen tone and length.
pub fn sample_reply(tone: VoiceTone, length: ReplyLength) -> String {
    let opening = match tone {
        VoiceTone::Friendly => "Happy to help!",
        VoiceTone::Neutral => "Here is what we have.",
        VoiceTone::Formal => "Thank you for your question.",
    };
    let body = match length {
        ReplyLength::Short => "We are open 9 to 5, Monday to Friday.",
        ReplyLength::Balanced => {
            "We are open 9 to 5, Monday to Friday, and closed on public holidays."
        }
        ReplyLength::Detailed => {
            "We are open 9 to 5, Monday to Friday, and closed on public holidays. Orders placed after 4 pm are sent the next working day, and you will get a tracking link by email."
        }
    };
    format!("{opening} {body}")
}

/// The line in the preview about who answers when the assistant is unsure.
pub fn handover_line(handover: bool, email: &str) -> String {
    if handover {
        format!("Hands over to {} when unsure.", email.trim())
    } else {
        "Nobody takes over; the assistant answers itself.".to_string()
    }
}

/// Assistant setup, step 3: name, tone, reply length and handover.
#[component]
pub fn AssistantPersonaStep(
    #[props(default, into)] website_domain: String,
    #[props(default, into)] brand_name: String,
    #[props(default)] short_forms: Vec<String>,
    #[props(default)] default_name: Option<String>,
    #[props(default)] default_tone: VoiceTone,
    #[props(default)] default_length: ReplyLength,
    #[props(default = true)] default_handover: bool,
    #[props(default)] default_email: Option<String>,
    on_continue: EventHandler<PersonaValues>,
    #[props(default)] on_back: Option<EventHandler<()>>,
    #[props(default)] on_logout: Option<EventHandler<()>>,
    #[props(default, into)] account_email: String,
    #[props(default = "rv2".to_string(), into)] product_name: String,
    #[props(default)] brand_mark: Option<Element>,
    #[props(default)] frame_style: FrameStyle,
    #[props(default)] auto_focus: bool,
    #[props(default = 3)] step_index: usize,
    #[props(default = 4)] step_count: usize,
) -> Element {
    let brand = if brand_name.trim().is_empty() {
        proposed_name(&website_domain)
    } else {
        brand_name.trim().to_string()
    };
    let mut name = use_signal(move || default_name.unwrap_or_else(|| format!("{brand} Assistant")));
    let mut tone = use_signal(move || default_tone);
    let mut length = use_signal(move || default_length);
    let mut handover = use_signal(move || default_handover);
    let mut email = use_signal(move || {
        default_email.unwrap_or_else(|| default_handover_email(&website_domain))
    });
    let id = use_hook(|| format!("kit-ps-{}", next_id()));
    let name_id = format!("{id}-name");
    let address_id = format!("{id}-address");
    let switch_id = format!("{id}-handover");
    let address_hint_id = format!("{id}-hint");

    let name_now = name();
    let handover_now = handover();
    let email_now = email();
    let valid_now = can_continue(&name_now, handover_now, &email_now);
    let address_invalid = handover_now && !handover_email_valid(email_now.trim());
    let hint = if address_invalid {
        "Enter an address with a domain, such as support@example.com.".to_string()
    } else {
        String::new()
    };
    let preview = rsx! {
        DashboardPreview {
            assistant_name: name_now.clone(),
            status_label: "Draft",
            greeting_text: format!("Hi, I'm {name_now}. How can I help?"),
            tags: short_forms.clone(),
            test_question: "Are you open on public holidays?".to_string(),
            test_reply: sample_reply(tone(), length()),
            handover_line: handover_line(handover_now, &email_now),
        }
    };
    let submit_name = name_now.clone();
    let submit_email = email_now.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        OnboardingShell {
            step_index,
            step_count,
            title: "Shape how your assistant speaks",
            prompt: "Choose its name, its tone and how much it says. Then decide who it passes questions to.",
            account_email,
            product_name,
            brand_mark,
            frame_style,
            on_back,
            on_logout,
            preview,
            form { class: "kit-ps", onsubmit: move |event: FormEvent| event.prevent_default(),
                div { class: "kit-ps__field",
                    label { class: "kit-ps__label", r#for: "{name_id}", "Assistant name" }
                    input {
                        id: "{name_id}",
                        class: "kit-ps__input",
                        r#type: "text",
                        autocomplete: "off",
                        autofocus: auto_focus,
                        value: "{name}",
                        oninput: move |event: FormEvent| name.set(event.value()),
                        onkeydown: move |event: KeyboardEvent| {
                            if event.key() == Key::Enter && can_continue(&name.peek(), handover(), &email.peek()) {
                                event.prevent_default();
                                on_continue.call(PersonaValues {
                                    assistant_name: name.peek().trim().to_string(),
                                    voice_tone: tone(),
                                    reply_length: length(),
                                    handover: handover(),
                                    handover_email: email.peek().trim().to_string(),
                                });
                            }
                        },
                    }
                }
                fieldset { class: "kit-ps__field kit-ps__choice",
                    legend { class: "kit-ps__label", "Tone" }
                    div { class: "kit-ps__options",
                        for option in VoiceTone::ALL {
                            {
                                let value = option.as_str();
                                let text = option.label();
                                rsx! {
                                    label { key: "{value}", class: "kit-ps__option",
                                        input {
                                            r#type: "radio",
                                            name: "{id}-tone",
                                            value: "{value}",
                                            checked: tone() == option,
                                            onchange: move |_| tone.set(option),
                                        }
                                        span { "{text}" }
                                    }
                                }
                            }
                        }
                    }
                }
                fieldset { class: "kit-ps__field kit-ps__choice",
                    legend { class: "kit-ps__label", "Reply length" }
                    div { class: "kit-ps__options",
                        for option in ReplyLength::ALL {
                            {
                                let value = option.as_str();
                                let text = option.label();
                                rsx! {
                                    label { key: "{value}", class: "kit-ps__option",
                                        input {
                                            r#type: "radio",
                                            name: "{id}-length",
                                            value: "{value}",
                                            checked: length() == option,
                                            onchange: move |_| length.set(option),
                                        }
                                        span { "{text}" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-ps__field kit-ps__handover",
                    Switch {
                        id: switch_id.clone(),
                        label: rsx! { "Hand over to a person when unsure" },
                        description: rsx! { "Turn off to let the assistant answer every question itself." },
                        checked: handover_now,
                        aria_describedby: "{address_hint_id}",
                        onchange: move |event: FormEvent| handover.set(event.checked()),
                    }
                    div { class: "kit-ps__address",
                        label { class: "kit-ps__label", r#for: "{address_id}", "Hand over to" }
                        input {
                            id: "{address_id}",
                            class: "kit-ps__input",
                            r#type: "email",
                            autocomplete: "off",
                            value: "{email}",
                            disabled: !handover_now,
                            "aria-invalid": address_invalid.then_some("true"),
                            oninput: move |event: FormEvent| email.set(event.value()),
                            onkeydown: move |event: KeyboardEvent| {
                                if event.key() == Key::Enter && can_continue(&name.peek(), handover(), &email.peek()) {
                                    event.prevent_default();
                                    on_continue.call(PersonaValues {
                                        assistant_name: name.peek().trim().to_string(),
                                        voice_tone: tone(),
                                        reply_length: length(),
                                        handover: handover(),
                                        handover_email: email.peek().trim().to_string(),
                                    });
                                }
                            },
                        }
                    }
                    p { id: "{address_hint_id}", class: "kit-ps__hint", role: "status", "aria-live": "polite", "{hint}" }
                }
                div { class: "kit-ps__actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        label: "Continue",
                        disabled: !valid_now,
                        onclick: move |_| on_continue.call(PersonaValues {
                            assistant_name: submit_name.trim().to_string(),
                            voice_tone: tone(),
                            reply_length: length(),
                            handover: handover(),
                            handover_email: submit_email.trim().to_string(),
                        }),
                    }
                }
            }
        }
    }
}
