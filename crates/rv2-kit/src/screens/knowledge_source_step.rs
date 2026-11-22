//! Assistant setup, step 1 of 4: the websites the assistant learns from. Each entry lands in a
//! knowledge table in the dashboard preview beside the step.
//!
//! The onboarding shell (top bar, step markers, footer) and the dashboard preview are shared by
//! the four setup steps, so the sibling step modules import them from here.
//!
//! Design choice: the domain field is its own component. `WizardChips` takes any text as a chip, so
//! it cannot refuse an invalid domain or split a pasted list.
//! Design choice: a row's status steps Queued, Learning, Ready on a browser timer. Rows present at
//! mount are Ready.
//! Design choice: the account menu is a label with a Log out button rather than a menu.
//! Not implemented: `demo_entries` (typing for recordings).
//! Browser checks: the preview follows the newest row; Backspace on the empty field removes the last
//! chip; pasted "a.com, b.com" yields two chips; reduced motion makes statuses Ready at once.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};

use crate::internal::{KIT_CSS, next_id, prefers_motion_script, sleep};

const CSS: Asset = asset!("/assets/knowledge-source-step.css");

/// Row status in the knowledge table and on a chip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceStatus {
    Queued,
    Learning,
    Ready,
}

impl SourceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Learning => "learning",
            Self::Ready => "ready",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Learning => "Learning",
            Self::Ready => "Ready",
        }
    }
}

/// A domain the assistant learns from, as the preview's table shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewSource {
    pub domain: String,
    pub status: SourceStatus,
}

/// Picture frame around the dashboard preview.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FrameStyle {
    #[default]
    Inset,
    Flat,
}

impl FrameStyle {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inset => "inset",
            Self::Flat => "flat",
        }
    }
}

/// Cleans what a person typed into a bare domain: no scheme, no `www.`, no path, lowercase.
/// `None` when what remains is not a domain with a dotted, alphabetic top level.
pub fn clean_domain(raw: &str) -> Option<String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    let without_scheme = match trimmed.split_once("://") {
        Some((_, rest)) => rest,
        None => trimmed.as_str(),
    };
    let host_end = without_scheme
        .find(['/', '?', '#'])
        .unwrap_or(without_scheme.len());
    let host = &without_scheme[..host_end];
    let host = host.strip_prefix("www.").unwrap_or(host);
    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() < 2 {
        return None;
    }
    let label_ok = |label: &&str| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    };
    if !labels.iter().all(label_ok) {
        return None;
    }
    let top = labels[labels.len() - 1];
    if top.len() < 2 || !top.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    Some(host.to_string())
}

/// Characters that end an entry. A pasted list can carry any of them: a browser turns a line break
/// pasted into a text field into a space.
const ENTRY_SEPARATORS: [char; 4] = [',', '\n', ' ', '\t'];

/// Splits a pasted or comma-separated list into entries.
pub fn split_entries(text: &str) -> Vec<String> {
    text.split(ENTRY_SEPARATORS)
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

/// Continue is available only when at least one source exists.
pub fn can_continue(sources: &[PreviewSource]) -> bool {
    !sources.is_empty()
}

/// The line under the field, announced politely after each change.
pub fn count_line(count: usize) -> String {
    match count {
        0 => "Add at least one website to continue.".to_string(),
        1 => "1 website added.".to_string(),
        n => format!("{n} websites added."),
    }
}

/// Position of a step marker: done, current or upcoming.
pub fn step_state(step: usize, current: usize) -> &'static str {
    match step.cmp(&current) {
        std::cmp::Ordering::Less => "done",
        std::cmp::Ordering::Equal => "current",
        std::cmp::Ordering::Greater => "upcoming",
    }
}

/// Adds the valid entries not already present, as Queued. Returns the first entry that is not a
/// domain, which the caller keeps in the field.
pub(crate) fn stage_entries(
    mut sources: Signal<Vec<PreviewSource>>,
    parts: &[String],
) -> Option<String> {
    let mut first_invalid = None;
    for part in parts {
        match clean_domain(part) {
            Some(domain) => {
                if sources.peek().iter().any(|s| s.domain == domain) {
                    continue;
                }
                sources.write().push(PreviewSource {
                    domain: domain.clone(),
                    status: SourceStatus::Queued,
                });
                advance_status(sources, domain);
            }
            None => {
                if first_invalid.is_none() {
                    first_invalid = Some(part.clone());
                }
            }
        }
    }
    first_invalid
}

fn advance_status(mut sources: Signal<Vec<PreviewSource>>, domain: String) {
    spawn(async move {
        let motion = document::eval(&format!("return {};", prefers_motion_script()))
            .await
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(true);
        if motion {
            sleep(450).await;
            set_status(&mut sources, &domain, SourceStatus::Learning);
            sleep(900).await;
        }
        set_status(&mut sources, &domain, SourceStatus::Ready);
    });
}

fn set_status(sources: &mut Signal<Vec<PreviewSource>>, domain: &str, status: SourceStatus) {
    let mut list = sources.write();
    if let Some(row) = list.iter_mut().find(|s| s.domain == domain) {
        row.status = status;
    }
}

/// Top bar, step markers, title and prompt, the step's form, and a preview beside it.
#[component]
pub fn OnboardingShell(
    #[props(default = 1)] step_index: usize,
    #[props(default = 4)] step_count: usize,
    #[props(into)] title: String,
    #[props(into)] prompt: String,
    #[props(default, into)] account_email: String,
    #[props(default = "rv2".to_string(), into)] product_name: String,
    #[props(default)] brand_mark: Option<Element>,
    #[props(default)] frame_style: FrameStyle,
    #[props(default)] on_back: Option<EventHandler<()>>,
    #[props(default)] on_logout: Option<EventHandler<()>>,
    preview: Element,
    children: Element,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "kit-onb", "data-frame": frame_style.as_str(),
            header { class: "kit-onb__top",
                div { class: "kit-onb__brand",
                    if let Some(back) = on_back {
                        Button {
                            variant: ButtonVariant::Ghost,
                            label: "Back",
                            onclick: move |_| back.call(()),
                        }
                    }
                    if let Some(mark) = brand_mark {
                        {mark}
                    } else {
                        span { class: "kit-onb__mark", "aria-hidden": "true" }
                    }
                    span { class: "kit-onb__product", "{product_name}" }
                }
                div { class: "kit-onb__account",
                    span { class: "kit-onb__email", "{account_email}" }
                    if let Some(out) = on_logout {
                        Button {
                            variant: ButtonVariant::Ghost,
                            label: "Log out",
                            onclick: move |_| out.call(()),
                        }
                    }
                }
            }
            div { class: "kit-onb__body",
                section { class: "kit-onb__panel",
                    ol { class: "kit-onb__steps", "aria-label": "Setup progress",
                        for step in 1..=step_count {
                            li {
                                key: "{step}",
                                class: "kit-onb__step",
                                "data-state": step_state(step, step_index),
                                "aria-current": (step == step_index).then_some("step"),
                            }
                        }
                    }
                    h1 { class: "kit-onb__title", "{title}" }
                    p { class: "kit-onb__prompt", "{prompt}" }
                    {children}
                }
                div { class: "kit-onb__preview-wrap", {preview} }
            }
            footer { class: "kit-onb__foot",
                span { "© {product_name}" }
                a { href: "#terms", "Terms" }
                a { href: "#privacy", "Privacy" }
            }
        }
    }
}

/// Read-only picture of the assistant's dashboard. Hidden from assistive technology.
#[component]
pub fn DashboardPreview(
    #[props(into)] assistant_name: String,
    #[props(into)] status_label: String,
    #[props(default = false)] live: bool,
    #[props(into)] greeting_text: String,
    #[props(default)] tags: Vec<String>,
    #[props(default)] sources: Vec<PreviewSource>,
    #[props(default)] conversations: usize,
    #[props(default)] test_question: Option<String>,
    #[props(default)] test_reply: Option<String>,
    #[props(default)] handover_line: Option<String>,
    #[props(default = vec!["Overview".to_string(), "Conversations".to_string(), "Knowledge".to_string(), "Learning".to_string()])]
    nav_items: Vec<String>,
) -> Element {
    let knowledge_count = sources.len();
    rsx! {
        div { class: "kit-onb__preview", "aria-hidden": "true", "data-live": live.then_some("true"),
            nav { class: "kit-onb__pv-nav",
                for item in nav_items.iter() {
                    span { key: "{item}", class: "kit-onb__pv-link", "{item}" }
                }
            }
            div { class: "kit-onb__pv-main",
                article { class: "kit-onb__pv-card",
                    div { class: "kit-onb__pv-head",
                        span { class: "kit-onb__pv-name", "{assistant_name}" }
                        span { class: "kit-onb__pv-status", "data-live": live.then_some("true"), "{status_label}" }
                    }
                    p { class: "kit-onb__pv-greeting", "{greeting_text}" }
                    if !tags.is_empty() {
                        ul { class: "kit-onb__pv-tags",
                            for tag in tags.iter() {
                                li { key: "{tag}", class: "kit-onb__pv-tag", "{tag}" }
                            }
                        }
                    }
                }
                div { class: "kit-onb__pv-stats",
                    div { class: "kit-onb__pv-stat",
                        span { class: "kit-onb__pv-figure", "{knowledge_count}" }
                        span { class: "kit-onb__pv-label", "Knowledge sources" }
                    }
                    div { class: "kit-onb__pv-stat",
                        span { class: "kit-onb__pv-figure", "{conversations}" }
                        span { class: "kit-onb__pv-label", "Conversations" }
                    }
                }
                if !sources.is_empty() {
                    table { class: "kit-onb__pv-table",
                        thead {
                            tr {
                                th { "Source" }
                                th { "Status" }
                            }
                        }
                        tbody {
                            for source in sources.iter() {
                                {
                                    let label = source.status.label();
                                    let state = source.status.as_str();
                                    rsx! {
                                        tr { key: "{source.domain}", "data-status": state,
                                            td { class: "kit-onb__pv-domain", "{source.domain}" }
                                            td {
                                                span { class: "kit-onb__pv-badge", "data-status": state, "{label}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(question) = test_question {
                    div { class: "kit-onb__pv-chat",
                        p { class: "kit-onb__pv-bubble kit-onb__pv-bubble--user", "{question}" }
                        if let Some(reply) = test_reply {
                            p { class: "kit-onb__pv-bubble kit-onb__pv-bubble--assistant", "{reply}" }
                        }
                    }
                }
                if let Some(line) = handover_line {
                    p { class: "kit-onb__pv-handover", "{line}" }
                }
            }
        }
    }
}

/// Assistant setup, step 1: the websites to learn from.
#[component]
pub fn KnowledgeSourceStep(
    #[props(default)] default_domains: Vec<String>,
    on_continue: EventHandler<Vec<String>>,
    #[props(default)] on_back: Option<EventHandler<()>>,
    #[props(default)] on_logout: Option<EventHandler<()>>,
    #[props(default, into)] account_email: String,
    #[props(default = "rv2".to_string(), into)] product_name: String,
    #[props(default)] brand_mark: Option<Element>,
    #[props(default)] frame_style: FrameStyle,
    #[props(default)] auto_focus: bool,
    #[props(default = 1)] step_index: usize,
    #[props(default = 4)] step_count: usize,
) -> Element {
    let initial: Vec<PreviewSource> = {
        let mut seen: Vec<String> = Vec::new();
        for domain in default_domains.iter().filter_map(|d| clean_domain(d)) {
            if !seen.contains(&domain) {
                seen.push(domain);
            }
        }
        seen.into_iter()
            .map(|domain| PreviewSource {
                domain,
                status: SourceStatus::Ready,
            })
            .collect()
    };
    let mut sources = use_signal(move || initial);
    let mut draft = use_signal(String::new);
    let mut invalid = use_signal(|| None::<String>);
    let id = use_hook(|| format!("kit-src-{}", next_id()));
    let list = sources();
    let count = list.len();
    let domains: Vec<String> = list.iter().map(|s| s.domain.clone()).collect();
    let error_text = invalid()
        .map(|raw| format!("“{raw}” is not a website address. Try a form such as example.com."));
    let hint_id = format!("{id}-hint");
    let hint = count_line(count);
    let hint_text = error_text.clone().unwrap_or(hint);
    let continue_domains = domains.clone();
    let preview = rsx! {
        DashboardPreview {
            assistant_name: "Support Assistant",
            status_label: "Draft",
            greeting_text: "Hi, I'm your assistant. I'll answer from the sites you add.",
            sources: list.clone(),
            conversations: 0usize,
        }
    };
    rsx! {
        OnboardingShell {
            step_index,
            step_count,
            title: "Where should your assistant learn?",
            prompt: "Add the websites, documentation and help centres it should read.",
            account_email,
            product_name,
            brand_mark,
            frame_style,
            on_back,
            on_logout,
            preview,
            div { class: "kit-src",
                label { class: "kit-src__label", r#for: "{id}", "Websites and help centres" }
                div { class: "kit-src__box", "data-invalid": error_text.is_some().then_some("true"),
                    for domain in domains.iter() {
                        {
                            let remove = domain.clone();
                            rsx! {
                                span { key: "{domain}", class: "kit-src__chip",
                                    "{domain}"
                                    button {
                                        r#type: "button",
                                        class: "kit-src__chip-remove",
                                        "aria-label": "Remove {remove}",
                                        onclick: move |_| {
                                            sources.write().retain(|s| s.domain != remove);
                                        },
                                        "×"
                                    }
                                }
                            }
                        }
                    }
                    input {
                        id: "{id}",
                        class: "kit-src__input",
                        r#type: "text",
                        autocomplete: "off",
                        autofocus: auto_focus,
                        placeholder: "example.com",
                        value: "{draft}",
                        "aria-invalid": error_text.is_some().then_some("true"),
                        "aria-describedby": "{hint_id}",
                        oninput: move |event: FormEvent| {
                            let value = event.value();
                            if value.contains(ENTRY_SEPARATORS) {
                                let parts = split_entries(&value);
                                match stage_entries(sources, &parts) {
                                    Some(raw) => {
                                        draft.set(raw.clone());
                                        invalid.set(Some(raw));
                                    }
                                    None => {
                                        draft.set(String::new());
                                        invalid.set(None);
                                    }
                                }
                            } else {
                                draft.set(value);
                                invalid.set(None);
                            }
                        },
                        onkeydown: move |event: KeyboardEvent| {
                            let typed = draft();
                            let separator = matches!(&event.key(), Key::Character(c) if c == " ");
                            let enter = matches!(event.key(), Key::Enter);
                            if (enter || separator) && !typed.trim().is_empty() {
                                event.prevent_default();
                                match stage_entries(sources, &[typed.trim().to_string()]) {
                                    Some(raw) => invalid.set(Some(raw)),
                                    None => {
                                        draft.set(String::new());
                                        invalid.set(None);
                                    }
                                }
                            } else if enter && count > 0 {
                                event.prevent_default();
                                on_continue.call(domains.clone());
                            } else if matches!(event.key(), Key::Backspace) && typed.is_empty() {
                                sources.write().pop();
                            } else if matches!(event.key(), Key::Escape) {
                                invalid.set(None);
                            }
                        },
                    }
                }
                p { id: "{hint_id}", class: "kit-src__hint", role: "status", "aria-live": "polite",
                    "{hint_text}"
                }
                div { class: "kit-src__actions",
                    Button {
                        variant: ButtonVariant::Primary,
                        label: "Continue",
                        disabled: !can_continue(&list),
                        onclick: move |_| on_continue.call(continue_domains.clone()),
                    }
                }
            }
        }
    }
}
