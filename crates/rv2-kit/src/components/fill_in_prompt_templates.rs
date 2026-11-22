//! Fill-in prompt templates: saved prompts whose `{name}` blanks are filled inside the sentence.
//!
//! Design choice: a blank is a `{name}` whose name uses letters, digits, `-` and `_`; any other
//! brace stays literal text. A repeated name is one value. Values are kept per template for the
//! life of the component. Token counts are a rough estimate of one token per four characters.
//!
//! Browser checks: Enter in a blank moves to the next empty blank; the selection highlight slides
//! between rows; a narrow container puts the list above the detail.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list, copy_to_clipboard, next_id};

const CSS: Asset = asset!("/assets/fill-in-prompt-templates.css");

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlankSpec {
    pub hint_text: Option<String>,
    pub fallback_value: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    pub template_id: String,
    pub template_title: String,
    pub group_label: Option<String>,
    pub group_key: Option<String>,
    pub template_body: String,
    pub blank_specs: BTreeMap<String, BlankSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Text(String),
    Blank(String),
}

fn is_blank_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Splits a template body into literal text and blanks.
pub fn parse_body(body: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut text = String::new();
    let mut rest = body;
    while let Some(open) = rest.find('{') {
        text.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) if is_blank_name(&after[..close]) => {
                if !text.is_empty() {
                    segments.push(Segment::Text(std::mem::take(&mut text)));
                }
                segments.push(Segment::Blank(after[..close].to_string()));
                rest = &after[close + 1..];
            }
            _ => {
                text.push('{');
                rest = after;
            }
        }
    }
    text.push_str(rest);
    if !text.is_empty() {
        segments.push(Segment::Text(text));
    }
    segments
}

/// Distinct blank names in order of first appearance.
pub fn blank_names(body: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for segment in parse_body(body) {
        if let Segment::Blank(name) = segment {
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

/// The prompt with each blank replaced by its value. A blank with no value keeps its `{name}`,
/// so the raw prompt shows what is still missing.
pub fn render_prompt(body: &str, values: &BTreeMap<String, String>) -> String {
    parse_body(body)
        .into_iter()
        .map(|segment| match segment {
            Segment::Text(text) => text,
            Segment::Blank(name) => match values.get(&name).filter(|v| !v.trim().is_empty()) {
                Some(value) => value.clone(),
                None => format!("{{{name}}}"),
            },
        })
        .collect()
}

/// Blank names with no value, or only whitespace.
pub fn missing_blanks(names: &[String], values: &BTreeMap<String, String>) -> Vec<String> {
    names
        .iter()
        .filter(|name| {
            values
                .get(*name)
                .is_none_or(|value| value.trim().is_empty())
        })
        .cloned()
        .collect()
}

/// About one token per four characters, rounded up.
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// The count of blanks still empty, shown in the footer and the live status.
pub fn blanks_remaining_text(missing: usize) -> String {
    match missing {
        1 => "1 blank still empty".to_string(),
        n => format!("{n} blanks still empty"),
    }
}

/// Values for a template: its fallbacks, overlaid with what the person typed.
fn merged_values(
    template: &Template,
    typed: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut merged: BTreeMap<String, String> = template
        .blank_specs
        .iter()
        .filter_map(|(name, spec)| {
            spec.fallback_value
                .clone()
                .map(|value| (name.clone(), value))
        })
        .collect();
    merged.extend(
        typed
            .iter()
            .map(|(name, value)| (name.clone(), value.clone())),
    );
    merged
}

#[component]
pub fn FillInPromptTemplates(
    template_list: Vec<Template>,
    #[props(default)] selected_id: Option<String>,
    #[props(default)] default_selected_id: Option<String>,
    #[props(default)] on_select: EventHandler<String>,
    #[props(default)] on_submit: EventHandler<(String, Template, BTreeMap<String, String>)>,
    #[props(default)] on_copy: EventHandler<String>,
    #[props(default = "Prompts".to_string(), into)] list_title: String,
    #[props(default = "Use prompt".to_string(), into)] submit_label: String,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-fpt-{}", next_id()));
    let mut internal_selected = use_signal(|| default_selected_id.clone());
    let mut typed = use_signal(BTreeMap::<String, BTreeMap<String, String>>::new);
    let mut query = use_signal(String::new);
    let mut copied = use_signal(|| false);
    let mut status = use_signal(String::new);

    let needle = query().trim().to_lowercase();
    let visible: Vec<Template> = template_list
        .iter()
        .filter(|t| needle.is_empty() || t.template_title.to_lowercase().contains(&needle))
        .cloned()
        .collect();
    let chosen_id = selected_id
        .clone()
        .or(internal_selected())
        .or_else(|| template_list.first().map(|t| t.template_id.clone()));
    let selected: Option<Template> = chosen_id.as_ref().and_then(|sid| {
        template_list
            .iter()
            .find(|t| &t.template_id == sid)
            .cloned()
    });

    let (values, prompt, missing, tokens) = match &selected {
        Some(template) => {
            let stored = typed()
                .get(&template.template_id)
                .cloned()
                .unwrap_or_default();
            let values = merged_values(template, &stored);
            let names = blank_names(&template.template_body);
            let missing = missing_blanks(&names, &values);
            let prompt = render_prompt(&template.template_body, &values);
            let tokens = estimate_tokens(&prompt);
            (values, prompt, missing, tokens)
        }
        None => (BTreeMap::new(), String::new(), Vec::new(), 0),
    };
    let ready = selected.is_some() && missing.is_empty();
    let blank_total = selected
        .as_ref()
        .map_or(0, |t| blank_names(&t.template_body).len());
    let footer_status = if ready {
        "All blanks filled".to_string()
    } else {
        format!("{} of {blank_total} filled", blank_total - missing.len())
    };
    let segments = selected
        .as_ref()
        .map(|t| parse_body(&t.template_body))
        .unwrap_or_default();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { id: "{id}", class: class_list("kit-fpt", &extra_class), "aria-label": "{list_title}",
            if template_list.is_empty() {
                p { class: "kit-fpt__empty", "No templates yet." }
            } else {
                div { class: "kit-fpt__head",
                    h3 { class: "kit-fpt__title", "{list_title}" }
                    input {
                        class: "kit-fpt__search",
                        r#type: "search",
                        placeholder: "Search prompts",
                        "aria-label": "Search prompts",
                        value: "{query}",
                        oninput: move |event| query.set(event.value()),
                        onkeydown: {
                            let scope = id.clone();
                            move |event: KeyboardEvent| {
                                if event.key() == Key::Escape {
                                    query.set(String::new());
                                } else if event.key() == Key::ArrowDown {
                                    event.prevent_default();
                                    document::eval(&format!(
                                        "(document.querySelector('#{scope} .kit-fpt__row[aria-selected=\"true\"]') || document.querySelector('#{scope} .kit-fpt__row'))?.focus();"
                                    ));
                                }
                            }
                        },
                    }
                }
                div { class: "kit-fpt__body",
                    div { class: "kit-fpt__list", role: "listbox", "aria-label": "{list_title}",
                        if visible.is_empty() {
                            p { class: "kit-fpt__none", "No prompts match." }
                        }
                        for template in visible.iter() {
                            {
                                let is_selected = chosen_id.as_deref() == Some(template.template_id.as_str());
                                let key = template.template_id.clone();
                                let pick_id = template.template_id.clone();
                                let group = template.group_key.clone();
                                let title = template.template_title.clone();
                                let group_label = template.group_label.clone();
                                rsx! {
                                    button {
                                        key: "{key}",
                                        r#type: "button",
                                        role: "option",
                                        class: "kit-fpt__row",
                                        "aria-selected": "{is_selected}",
                                        tabindex: if is_selected { "0" } else { "-1" },
                                        "data-group": group,
                                        onclick: move |_| {
                                            internal_selected.set(Some(pick_id.clone()));
                                            on_select.call(pick_id.clone());
                                        },
                                        span { class: "kit-fpt__row-title", "{title}" }
                                        if let Some(label) = group_label {
                                            span { class: "kit-fpt__badge", "{label}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(template) = selected.clone() {
                        div { class: "kit-fpt__detail", "aria-live": "polite",
                            h4 { class: "kit-fpt__detail-title", "{template.template_title}" }
                            p { class: "kit-fpt__sentence",
                                for (position, segment) in segments.iter().enumerate() {
                                    {
                                        match segment {
                                            Segment::Text(text) => {
                                                let text = text.clone();
                                                rsx! { span { key: "{position}", "{text}" } }
                                            }
                                            Segment::Blank(name) => {
                                                let name = name.clone();
                                                let value = values.get(&name).cloned().unwrap_or_default();
                                                let hint = template.blank_specs.get(&name).and_then(|s| s.hint_text.clone()).unwrap_or_else(|| name.clone());
                                                let input_id = format!("{id}-{name}");
                                                let label = format!("{name}: {hint}");
                                                let owner = template.template_id.clone();
                                                let stored_name = name.clone();
                                                rsx! {
                                                    input {
                                                        key: "{position}",
                                                        id: "{input_id}",
                                                        class: "kit-fpt__blank",
                                                        r#type: "text",
                                                        "aria-label": "{label}",
                                                        placeholder: "{hint}",
                                                        value: "{value}",
                                                        onkeydown: {
                                                            let scope = id.clone();
                                                            move |event: KeyboardEvent| {
                                                                if event.key() == Key::Enter {
                                                                    event.prevent_default();
                                                                    document::eval(&format!(
                                                                        "const blanks = [...document.querySelectorAll('#{scope} .kit-fpt__blank')]; const at = blanks.indexOf(document.activeElement); const next = blanks.slice(at + 1).find((el) => el.value.trim() === ''); if (next) {{ next.focus(); }} else if (blanks.every((el) => el.value.trim() !== '')) {{ document.querySelector('#{scope} .kit-fpt__use')?.click(); }} else {{ blanks.find((el) => el.value.trim() === '')?.focus(); }}"
                                                                    ));
                                                                }
                                                            }
                                                        },
                                                        oninput: move |event| {
                                                            let text = event.value();
                                                            let owner = owner.clone();
                                                            let stored_name = stored_name.clone();
                                                            typed.with_mut(|all| {
                                                                all.entry(owner).or_default().insert(stored_name, text);
                                                            });
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            pre { class: "kit-fpt__preview", "{prompt}" }
                            div { class: "kit-fpt__foot",
                                span { class: "kit-fpt__status", "{footer_status}" }
                                span { class: "kit-fpt__tokens", "≈ {tokens} tokens (estimate)" }
                                button {
                                    r#type: "button",
                                    class: "kit-fpt__copy",
                                    onclick: {
                                        let text = prompt.clone();
                                        move |_| {
                                            copy_to_clipboard(&text);
                                            copied.set(true);
                                            status.set("Prompt copied".to_string());
                                            on_copy.call(text.clone());
                                        }
                                    },
                                    if copied() { "Copied" } else { "Copy" }
                                }
                                button {
                                    r#type: "button",
                                    class: "kit-fpt__use",
                                    "aria-disabled": if ready { "false" } else { "true" },
                                    "data-ready": ready.then_some("true"),
                                    onclick: {
                                        let owner = template.clone();
                                        let filled_values = values.clone();
                                        let missing_count = missing.len();
                                        let prompt_text = prompt.clone();
                                        move |_| {
                                            if missing_count > 0 {
                                                status.set(blanks_remaining_text(missing_count));
                                                return;
                                            }
                                            on_submit.call((prompt_text.clone(), owner.clone(), filled_values.clone()));
                                        }
                                    },
                                    "{submit_label}"
                                }
                            }
                            p { class: "kit-fpt__live", role: "status", "{status}" }
                        }
                    }
                }
            }
        }
    }
}
