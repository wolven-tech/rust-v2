//! Assistant setup, step 2 of 4: the business name and the short names customers use for it. The
//! first name is what the assistant calls itself; every other name is recognised as well.
//!
//! Design choice: entry is `WizardChips` (Enter or comma makes a chip, Backspace on the empty field
//! removes the last, Enter on the empty field continues), so the chip rules are not restated here.
//! Design choice: `WizardChips` does not mark the primary chip, so the line under the field names it.
//! Browser checks: the card and greeting follow the primary chip; removing the primary makes the next
//! chip primary.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};

use super::knowledge_source_step::{DashboardPreview, FrameStyle, OnboardingShell};
use crate::components::multi_step_wizard_form::WizardChips;
use crate::internal::{KIT_CSS, next_id};

const CSS: Asset = asset!("/assets/assistant-naming-step.css");
const CHIPS_CSS: Asset = asset!("/assets/multi-step-wizard-form.css");

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
        [primary, rest @ ..] => format!(
            "{} names. Primary: {primary}. Short forms: {}.",
            names.len(),
            rest.join(", ")
        ),
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
    let mut names = use_signal(move || initial_names(&website_domain, &default_names));
    let id = use_hook(|| format!("kit-nm-{}", next_id()));
    let list = names();
    let primary = list.first().cloned().unwrap_or_default();
    let short_forms: Vec<String> = list.iter().skip(1).cloned().collect();
    let line = names_line(&list);
    let card_name = if primary.is_empty() {
        "Your assistant".to_string()
    } else {
        primary.clone()
    };
    let preview = rsx! {
        DashboardPreview {
            assistant_name: card_name,
            status_label: "Draft",
            greeting_text: greeting(&primary),
            tags: short_forms,
        }
    };
    let submit = names.peek().clone();
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CHIPS_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        OnboardingShell {
            step_index,
            step_count,
            title: "What do customers call your business?",
            prompt: "Add the business name, then any short names customers use. The first name is what your assistant calls itself.",
            account_email,
            product_name,
            brand_mark,
            frame_style,
            on_back,
            on_logout,
            preview,
            div { class: "kit-nm", "data-id": "{id}",
                WizardChips {
                    values: list.clone(),
                    label: "Business name and short names",
                    on_change: move |next: Vec<String>| names.set(next),
                    on_make_primary: move |chosen: String| {
                        let current = names.peek().clone();
                        names.set(promote_primary(&current, &chosen));
                    },
                    on_empty_enter: Callback::new(move |_: ()| {
                        let current = names.peek().clone();
                        if can_continue(&current) {
                            on_continue.call(current);
                        }
                    }),
                }
                p { class: "kit-nm__line", role: "status", "aria-live": "polite", "{line}" }
                div { class: "kit-nm__actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        label: "Continue",
                        disabled: !can_continue(&list),
                        onclick: move |_| on_continue.call(submit.clone()),
                    }
                }
            }
        }
    }
}
