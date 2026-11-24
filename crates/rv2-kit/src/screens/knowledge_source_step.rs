//! Assistant setup, step 1 of 4: the websites the assistant learns from. Each entry lands in a
//! knowledge table in the dashboard preview beside the step.
//!
//! The onboarding shell (top bar, step markers, footer) and the dashboard preview are shared by
//! the four setup steps, so the sibling step modules import them from here.
//!
//! Design choice: the domain field is its own component. `WizardChips` takes any text as a chip, so
//! it cannot refuse an invalid domain or split a pasted list.
//! Design choice: the table lists pages, three per source, from the one source. Pages from a source
//! entered during the session step Queued, Learning, Ready on a browser timer; pages present at mount
//! hold their rest statuses (Ready, Learning, Queued) and do not move.
//! Design choice: the page paths are fixed placeholders; the kit has no crawler.
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

/// The status a source moves to next: Queued, Learning, Ready, then Queued again.
pub fn next_status(status: SourceStatus) -> SourceStatus {
    match status {
        SourceStatus::Queued => SourceStatus::Learning,
        SourceStatus::Learning => SourceStatus::Ready,
        SourceStatus::Ready => SourceStatus::Queued,
    }
}

/// One row of the preview's knowledge table: a page the assistant reads, by its address.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewSource {
    pub domain: String,
    pub status: SourceStatus,
}

/// Paths the assistant reads under each entered source, in reading order. The step shows these
/// three pages per source; a host with a real crawl passes its own rows to `DashboardPreview`.
const PAGE_PATHS: [&str; 3] = ["", "/about", "/help"];

/// Statuses of a source's pages at rest, in reading order.
const REST_STATUSES: [SourceStatus; 3] = [
    SourceStatus::Ready,
    SourceStatus::Learning,
    SourceStatus::Queued,
];

/// A chip and the pages read from it.
#[derive(Clone, Debug, PartialEq)]
struct Entry {
    domain: String,
    pages: Vec<PreviewSource>,
}

fn entry_pages(domain: &str, statuses: &[SourceStatus; 3]) -> Vec<PreviewSource> {
    PAGE_PATHS
        .iter()
        .zip(statuses)
        .map(|(path, status)| PreviewSource {
            domain: format!("{domain}{path}"),
            status: *status,
        })
        .collect()
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
pub fn can_continue(domains: &[String]) -> bool {
    !domains.is_empty()
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

/// Adds the valid entries not already present, with their pages Queued. Returns the first entry
/// that is not a domain, which the caller keeps in the field.
fn stage_entries(mut entries: Signal<Vec<Entry>>, parts: &[String]) -> Option<String> {
    let mut first_invalid = None;
    for part in parts {
        match clean_domain(part) {
            Some(domain) => {
                if entries.peek().iter().any(|e| e.domain == domain) {
                    continue;
                }
                entries.write().push(Entry {
                    domain: domain.clone(),
                    pages: entry_pages(&domain, &[SourceStatus::Queued; 3]),
                });
                advance_pages(entries, domain);
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

/// Reads the entry's pages in order: each goes Queued, Learning, Ready. Without motion every page
/// is Ready at once.
fn advance_pages(mut entries: Signal<Vec<Entry>>, domain: String) {
    spawn(async move {
        let motion = document::eval(&format!("return {};", prefers_motion_script()))
            .await
            .ok()
            .and_then(|value| value.as_bool())
            .unwrap_or(true);
        for path in PAGE_PATHS {
            let page = format!("{domain}{path}");
            if motion {
                sleep(450).await;
                set_status(&mut entries, &domain, &page, SourceStatus::Learning);
                sleep(900).await;
            }
            set_status(&mut entries, &domain, &page, SourceStatus::Ready);
        }
    });
}

fn set_status(entries: &mut Signal<Vec<Entry>>, domain: &str, page: &str, status: SourceStatus) {
    let mut list = entries.write();
    let row = list
        .iter_mut()
        .filter(|e| e.domain == domain)
        .flat_map(|e| e.pages.iter_mut())
        .find(|p| p.domain == page);
    if let Some(row) = row {
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
///
/// `sketch` draws the unfinished dashboard of the setup steps: skeleton sidebar, tiles and chart,
/// a "Knowledge" table and a "Test chat" heading, with no figures. The other callers keep the
/// figures and the nav labels they pass.
#[component]
pub fn DashboardPreview(
    #[props(into)] assistant_name: String,
    #[props(into)] status_label: String,
    #[props(default = false)] live: bool,
    #[props(default = false)] sketch: bool,
    #[props(into)] greeting_text: String,
    #[props(default)] tags: Vec<String>,
    #[props(default)] sources: Vec<PreviewSource>,
    #[props(default = "Knowledge sources".to_string(), into)] source_label: String,
    #[props(default)] loading: bool,
    #[props(default)] conversations: usize,
    #[props(default)] test_question: Option<String>,
    #[props(default)] test_reply: Option<String>,
    #[props(default)] handover_line: Option<String>,
    #[props(default = vec!["Overview".to_string(), "Conversations".to_string(), "Knowledge".to_string(), "Learning".to_string()])]
    nav_items: Vec<String>,
) -> Element {
    let knowledge_count = sources.len();
    let sketch_attr = sketch.then_some("true");
    rsx! {
        div { class: "kit-onb__preview", "aria-hidden": "true", "data-live": live.then_some("true"), "data-sketch": sketch_attr,
            if sketch {
                div { class: "kit-onb__pv-side",
                    for index in 0..6usize {
                        span { key: "{index}", class: "kit-onb__pv-bar" }
                    }
                }
            } else {
                nav { class: "kit-onb__pv-nav",
                    for item in nav_items.iter() {
                        span { key: "{item}", class: "kit-onb__pv-link", "{item}" }
                    }
                }
            }
            div { class: "kit-onb__pv-main",
                if sketch {
                    div { class: "kit-onb__pv-band",
                        for index in 0..3usize {
                            span { key: "{index}", class: "kit-onb__pv-tile kit-onb__pv-skel" }
                        }
                    }
                }
                article { class: "kit-onb__pv-card", "data-loading": loading.then_some("true"),
                    div { class: "kit-onb__pv-head",
                        span { class: "kit-onb__pv-name", "{assistant_name}" }
                        span { class: "kit-onb__pv-status", "data-live": live.then_some("true"), "{status_label}" }
                    }
                    p { class: "kit-onb__pv-greeting", "{greeting_text}" }
                    if loading {
                        div { class: "kit-onb__pv-loading",
                            span { class: "kit-onb__pv-skel" }
                            span { class: "kit-onb__pv-skel kit-onb__pv-skel--short" }
                        }
                    }
                    if !tags.is_empty() {
                        if sketch {
                            span { class: "kit-onb__pv-also", "Also answers to" }
                        }
                        ul { class: "kit-onb__pv-tags",
                            for tag in tags.iter() {
                                li { key: "{tag}", class: "kit-onb__pv-tag", "{tag}" }
                            }
                        }
                    }
                }
                if sketch {
                    div { class: "kit-onb__pv-chart",
                        for index in 0..7usize {
                            span { key: "{index}", class: "kit-onb__pv-col" }
                        }
                    }
                } else {
                    div { class: "kit-onb__pv-stats",
                        div { class: "kit-onb__pv-stat",
                            span { class: "kit-onb__pv-figure", "{knowledge_count}" }
                            span { class: "kit-onb__pv-label", "{source_label}" }
                        }
                        div { class: "kit-onb__pv-stat",
                            span { class: "kit-onb__pv-figure", "{conversations}" }
                            span { class: "kit-onb__pv-label", "Conversations" }
                        }
                    }
                }
                if !sources.is_empty() {
                    if sketch {
                        span { class: "kit-onb__pv-heading", "{source_label}" }
                    }
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
                            for index in 0..3usize {
                                tr { key: "skel-{index}", class: "kit-onb__pv-skel-row",
                                    td { span { class: "kit-onb__pv-skel" } }
                                    td { span { class: "kit-onb__pv-skel kit-onb__pv-skel--short" } }
                                }
                            }
                        }
                    }
                }
                if let Some(question) = test_question {
                    if sketch {
                        span { class: "kit-onb__pv-heading", "Test chat" }
                    }
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
    let initial: Vec<Entry> = {
        let mut seen: Vec<String> = Vec::new();
        for domain in default_domains.iter().filter_map(|d| clean_domain(d)) {
            if !seen.contains(&domain) {
                seen.push(domain);
            }
        }
        seen.into_iter()
            .map(|domain| Entry {
                pages: entry_pages(&domain, &REST_STATUSES),
                domain,
            })
            .collect()
    };
    let mut entries = use_signal(move || initial);
    let mut draft = use_signal(String::new);
    let mut invalid = use_signal(|| None::<String>);
    let id = use_hook(|| format!("kit-src-{}", next_id()));
    let list = entries();
    let count = list.len();
    let domains: Vec<String> = list.iter().map(|e| e.domain.clone()).collect();
    let pages: Vec<PreviewSource> = list.iter().flat_map(|e| e.pages.clone()).collect();
    let domain_row: Vec<PreviewSource> = pages.iter().take(1).cloned().collect();
    let error_text = invalid()
        .map(|raw| format!("“{raw}” is not a website address. Try a form such as example.com."));
    let hint_id = format!("{id}-hint");
    let hint = count_line(count);
    let hint_text = error_text.clone().unwrap_or(hint);
    let continue_domains = domains.clone();
    let submit_enabled = can_continue(&domains);
    let preview = rsx! {
        DashboardPreview {
            assistant_name: "Support Assistant",
            status_label: "Draft",
            greeting_text: "Hi, I'm your assistant. I'll answer from the sites you add.",
            sketch: true,
            sources: domain_row,
            source_label: "Knowledge",
            test_question: Some("What are your opening hours?".to_string()),
            test_reply: Some("We are open Monday to Friday, 9 to 5.".to_string()),
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
                                            entries.write().retain(|e| e.domain != remove);
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
                        "aria-label": "Websites and help centres",
                        "aria-invalid": error_text.is_some().then_some("true"),
                        "aria-describedby": "{hint_id}",
                        oninput: move |event: FormEvent| {
                            let value = event.value();
                            if value.contains(ENTRY_SEPARATORS) {
                                let parts = split_entries(&value);
                                match stage_entries(entries, &parts) {
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
                                match stage_entries(entries, &[typed.trim().to_string()]) {
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
                                entries.write().pop();
                            } else if matches!(event.key(), Key::Escape) {
                                invalid.set(None);
                            }
                        },
                    }
                    button {
                        r#type: "button",
                        class: "kit-src__plus",
                        "aria-label": "Add website",
                        onclick: move |_| {
                            let typed = draft().trim().to_string();
                            if typed.is_empty() {
                                return;
                            }
                            match stage_entries(entries, &[typed]) {
                                Some(raw) => invalid.set(Some(raw)),
                                None => {
                                    draft.set(String::new());
                                    invalid.set(None);
                                }
                            }
                        },
                        svg { class: "kit-src__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round", "aria-hidden": "true",
                            path { d: "M8 3v10M3 8h10" }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "kit-src__submit",
                        "aria-label": "Continue",
                        disabled: !submit_enabled,
                        onclick: move |_| on_continue.call(continue_domains.clone()),
                        svg { class: "kit-src__glyph", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.75", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
                            path { d: "M8 13V3.5M3.9 7.6 8 3.5l4.1 4.1" }
                        }
                    }
                }
                p {
                    id: "{hint_id}",
                    class: if error_text.is_some() { "kit-src__hint" } else { "kit-sr" },
                    role: "status",
                    "aria-live": "polite",
                    "{hint_text}"
                }
            }
        }
    }
}
