//! Assistant setup, step 2 of 4: the business name and the short names customers use for it. The
//! first name is what the assistant calls itself; every other name is recognised as well.
//!
//! Design choice: the chips sit beneath the input, so the field holds only the typed text and its
//! two controls. Enter or comma makes a chip, Backspace on the empty field removes the last, and
//! Enter on the empty field continues.
//! Design choice: the count line is polite and visually hidden; the visible hint states the rule for
//! the first name.
//! Browser checks: the card and greeting follow the primary chip; removing the primary makes the next
//! chip primary.

use dioxus::prelude::*;

use super::knowledge_source_step::{
    DashboardPreview, FrameStyle, OnboardingShell, PreviewSource, SourceStatus,
};
use crate::components::multi_step_wizard_form::{add_chip, split_at_last_comma, split_chips};
use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/assistant-naming-step.css");

/// The business name proposed by a website: its first label, title-cased, hyphens as spaces.
pub fn proposed_name(domain: &str) -> String {
    let first_label = domain.trim().split('.').next().unwrap_or_default();
    let spaced = first_label.replace(['-', '_'], " ");
    let mut chars = spaced.trim().chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Names already known replace the proposal; otherwise the website proposes the first.
pub fn initial_names(website_domain: &str, default_names: &[String]) -> Vec<String> {
    let known: Vec<String> = default_names
        .iter()
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect();
    if !known.is_empty() {
        return known;
    }
    let proposal = proposed_name(website_domain);
    if proposal.is_empty() {
        Vec::new()
    } else {
        vec![proposal]
    }
}

/// Appends each part that is not empty or already present, in order.
pub fn add_names(names: &[String], parts: &[String]) -> Vec<String> {
    parts
        .iter()
        .fold(names.to_vec(), |next, part| add_chip(&next, part))
}

/// Moves `chosen` to the front, keeping the other names in their order. An unknown name leaves the
/// list as it was.
pub fn promote_primary(names: &[String], chosen: &str) -> Vec<String> {
    match names.iter().position(|name| name == chosen) {
        Some(index) => {
            let mut next = names.to_vec();
            let name = next.remove(index);
            next.insert(0, name);
            next
        }
        None => names.to_vec(),
    }
}

/// The assistant introduces itself with the primary name.
pub fn greeting(primary: &str) -> String {
    if primary.is_empty() {
        "Hi, I'm your assistant. How can I help?".to_string()
    } else {
        format!("Hi, I'm {primary}. How can I help?")
    }
}

/// The polite line under the chip field.
pub fn names_line(names: &[String]) -> String {
    match names {
        [] => "No names yet. Add the business name to continue.".to_string(),
        [primary] => format!("1 name. Primary: {primary}."),
        [primary, rest @ ..] => {
            let forms = rest.join(", ");
            let stop = if forms.ends_with('.') { "" } else { "." };
            format!(
                "{} names. Primary: {primary}. Short forms: {forms}{stop}",
                names.len()
            )
        }
    }
}

/// Continue is off until at least one name is present.
pub fn can_continue(names: &[String]) -> bool {
    !names.is_empty()
}

/// Assistant setup, step 2: business name and short forms.
#[component]
pub fn AssistantNamingStep(
    #[props(default, into)] website_domain: String,
    #[props(default)] default_names: Vec<String>,
    on_continue: EventHandler<Vec<String>>,
    #[props(default)] on_back: Option<EventHandler<()>>,
    #[props(default)] on_logout: Option<EventHandler<()>>,
    #[props(default, into)] account_email: String,
    #[props(default = "rv2".to_string(), into)] product_name: String,
    #[props(default)] brand_mark: Option<Element>,
    #[props(default)] frame_style: FrameStyle,
    #[props(default)] auto_focus: bool,
    #[props(default = 2)] step_index: usize,
    #[props(default = 4)] step_count: usize,
) -> Element {
    let source_domain = website_domain.trim().to_string();
    let mut names = use_signal(move || initial_names(&website_domain, &default_names));
    let mut draft = use_signal(String::new);
    let id = use_hook(|| format!("kit-nm-{}", next_id()));
    let input_id = format!("{id}-input");
    let hint_id = format!("{id}-hint");
    let list = names();
    let typed_now = draft();
    let primary = list.first().cloned().unwrap_or_default();
    let short_forms: Vec<String> = list.iter().skip(1).cloned().collect();
    let line = names_line(&list);
    let submit_ready = can_continue(&list) || !typed_now.trim().is_empty();
    let card_name = if primary.is_empty() {
        "Your assistant".to_string()
    } else {
        primary.clone()
    };
    let sources = if source_domain.is_empty() {
        Vec::new()
    } else {
        vec![PreviewSource {
            domain: source_domain,
            status: SourceStatus::Learning,
        }]
    };
    let preview = rsx! {
        DashboardPreview {
            assistant_name: card_name,
            status_label: "Draft",
            sketch: true,
            greeting_text: greeting(&primary),
            tags: short_forms,
            sources,
            source_label: "Knowledge",
            test_question: Some("What are your opening hours?".to_string()),
            test_reply: Some("We are open Monday to Friday, 9 to 5.".to_string()),
        }
    };
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        OnboardingShell {
            step_index,
            step_count,
            title: "What do customers call your business?",
            prompt: "Add the business name and the short names customers use.",
            account_email,
            product_name,
            brand_mark,
            frame_style,
            on_back,
            on_logout,
            preview,
            div { class: "kit-nm", "data-id": "{id}",
                div { class: "kit-nm__field",
                    input {
                        id: "{input_id}",
                        class: "kit-nm__input",
                        r#type: "text",
                        autocomplete: "off",
                        autofocus: auto_focus,
                        placeholder: "Business name",
                        value: "{typed_now}",
                        "aria-label": "Business name and short names",
                        "aria-describedby": "{hint_id}",
                        oninput: move |event: FormEvent| {
                            let value = event.value();
                            match split_at_last_comma(&value) {
                                Some((entries, rest)) => {
                                    let current = names.peek().clone();
                                    let next = add_names(&current, &entries);
                                    draft.set(rest);
                                    if next != current {
                                        names.set(next);
                                    }
                                }
                                None => draft.set(value),
                            }
                        },
                        onkeydown: move |event: KeyboardEvent| {
                            match event.key() {
                                Key::Enter => {
                                    event.prevent_default();
                                    let typed = draft.peek().clone();
                                    if typed.trim().is_empty() {
                                        let current = names.peek().clone();
                                        if can_continue(&current) {
                                            on_continue.call(current);
                                        }
                                    } else {
                                        let next = add_names(&names.peek(), &split_chips(&typed));
                                        draft.set(String::new());
                                        names.set(next);
                                    }
                                }
                                Key::Backspace if draft.peek().is_empty() => {
                                    names.write().pop();
                                }
                                _ => {}
                            }
                        },
                    }
                    button {
                        r#type: "button",
                        class: "kit-nm__plus",
                        "aria-label": "Add name",
                        onclick: move |_| {
                            let typed = draft.peek().clone();
                            if !typed.trim().is_empty() {
                                let next = add_names(&names.peek(), &split_chips(&typed));
                                draft.set(String::new());
                                names.set(next);
                            }
                        },
                        svg { class: "kit-nm__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round", "aria-hidden": "true",
                            path { d: "M8 3v10M3 8h10" }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "kit-nm__submit",
                        "aria-label": "Continue",
                        disabled: !submit_ready,
                        onclick: move |_| {
                            let typed = draft.peek().clone();
                            let next = add_names(&names.peek(), &split_chips(&typed));
                            if can_continue(&next) {
                                on_continue.call(next);
                            }
                        },
                        svg { class: "kit-nm__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                            path { d: "M8 13V3.5M3.9 7.6 8 3.5l4.1 4.1" }
                        }
                    }
                }
                ul { class: "kit-nm__chips", "aria-label": "Business name and short names",
                    for (position, chip) in list.iter().enumerate() {
                        {
                            let remove_at = position;
                            let make_primary = chip.clone();
                            rsx! {
                                li { key: "{chip}", class: "kit-nm__chip", "data-primary": (position == 0).then_some("true"),
                                    if position == 0 {
                                        span { class: "kit-nm__dot", "aria-hidden": "true" }
                                    }
                                    span { class: "kit-nm__name", "{chip}" }
                                    if position > 0 {
                                        button {
                                            r#type: "button",
                                            class: "kit-nm__make",
                                            "aria-label": "Make {make_primary} primary",
                                            onclick: move |_| {
                                                let current = names.peek().clone();
                                                names.set(promote_primary(&current, &make_primary));
                                            },
                                            "Make primary"
                                        }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "kit-nm__remove",
                                        "aria-label": "Remove {chip}",
                                        onclick: move |_| {
                                            let mut next = names.peek().clone();
                                            next.remove(remove_at);
                                            names.set(next);
                                        },
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                }
                p { id: "{hint_id}", class: "kit-nm__hint", "Your assistant calls itself by the first name." }
                p { class: "kit-sr", role: "status", "aria-live": "polite", "{line}" }
            }
        }
    }
}
