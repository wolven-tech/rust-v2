//! Tag input: several values in one field, as chips. Enter, comma, semicolon or
//! Tab adds what was typed; a value that fails `validate` stays in the field
//! with a message; a duplicate flashes the chip it repeats. Backspace on an
//! empty field marks the last chip, a second Backspace removes it.
//!
//! Port losses: a paste is not intercepted. The pasted text lands in the field
//! and the change handler splits it, so the last piece stays in the field until
//! Enter, Tab or blur. Clicking the box background does not focus the field.
//! ArrowLeft moves to the last chip only when the field is empty, because the
//! caret position is not readable here. IME composition is not checked.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/tag-input.css");

/// What a commit adds, what stays in the field, and what to say about it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitPlan {
    pub added: Vec<String>,
    /// Values that failed `validate` or hit `max`; they stay in the field.
    pub left: Vec<String>,
    /// The first problem, shown under the field.
    pub problem: String,
    /// The last duplicate seen; its chip flashes.
    pub duplicate: Option<String>,
}

/// `parts` are already normalised and non-empty. `validate` returns a message
/// for a refused value.
pub fn plan_commit(
    parts: &[String],
    existing: &[String],
    max: Option<usize>,
    validate: impl Fn(&str) -> Option<String>,
) -> CommitPlan {
    let mut plan = CommitPlan::default();
    for part in parts {
        if let Some(why) = validate(part) {
            plan.left.push(part.clone());
            if plan.problem.is_empty() {
                plan.problem = why;
            }
        } else if existing.contains(part) || plan.added.contains(part) {
            plan.duplicate = Some(part.clone());
        } else if let Some(limit) = max.filter(|limit| existing.len() + plan.added.len() >= *limit)
        {
            plan.left.push(part.clone());
            if plan.problem.is_empty() {
                plan.problem = format!("Up to {limit} at a time.");
            }
        } else {
            plan.added.push(part.clone());
        }
    }
    plan
}

/// Splits on `,` `;` and newlines, and on any whitespace when `split_on_space`.
pub fn split_tags(text: &str, split_on_space: bool) -> Vec<&str> {
    text.split(|c: char| is_separator(c, split_on_space))
        .filter(|piece| !piece.is_empty())
        .collect()
}

fn is_separator(c: char, split_on_space: bool) -> bool {
    c == ',' || c == ';' || c == '\n' || (split_on_space && c.is_whitespace())
}

/// The chip's circle: the first character, upper-cased.
pub fn avatar_letter(tag: &str) -> String {
    tag.chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_default()
}

/// Focuses chip `index` (negative counts from the end), or the field when there is none.
fn focus_script(box_id: &str, input_id: &str, index: i64) -> String {
    format!(
        "const chips = document.querySelectorAll({:?}); const el = chips.at({index}) ?? document.getElementById({:?}); el?.focus();",
        format!("#{box_id} .ui-tag__chip"),
        input_id
    )
}

fn focus_input_script(input_id: &str) -> String {
    format!("document.getElementById({input_id:?})?.focus();")
}

#[component]
pub fn TagInput(
    #[props(default, into)] id: Option<String>,
    #[props(default)] value: Option<Vec<String>>,
    #[props(default)] default_value: Vec<String>,
    #[props(default)] on_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] label: Option<Element>,
    #[props(default)] hint: Option<Element>,
    #[props(default)] error: Option<Element>,
    #[props(default)] optional: bool,
    #[props(default = "Add…".to_string(), into)] placeholder: String,
    /// Returns the message for a refused value, `None` to accept it.
    #[props(default)]
    validate: Option<Callback<String, Option<String>>>,
    /// Cleans a value before the check. Default: trim.
    #[props(default)]
    normalize: Option<Callback<String, String>>,
    #[props(default)] split_on_space: bool,
    #[props(default)] max: Option<usize>,
    #[props(default)] avatar: bool,
    #[props(default)] disabled: bool,
    #[props(default, into)] name: String,
    #[props(default, into)] aria_label: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let auto_id = use_hook(|| format!("ui-tag-{}", next_id()));
    let input_id = id.unwrap_or_else(|| format!("{auto_id}-input"));
    let box_id = format!("{auto_id}-box");
    let label_id = format!("{auto_id}-label");
    let message_id = format!("{auto_id}-message");
    let live_id = format!("{auto_id}-live");

    let controlled = value.is_some();
    let mut own = use_signal(|| default_value.clone());
    let tags: Vec<String> = value.unwrap_or(own());
    let mut draft = use_signal(String::new);
    let mut problem = use_signal(String::new);
    let mut armed = use_signal(|| false);
    let mut flash = use_signal(|| None::<(String, u32)>);
    let mut flash_count = use_signal(|| 0u32);
    let mut said = use_signal(String::new);

    let full = max.is_some_and(|m| tags.len() >= m);
    let shows_error = error.is_some() || !problem().is_empty();
    let invalid = shows_error;
    let message_key = if shows_error { "error" } else { "hint" };
    let message: Option<Element> = if error.is_some() {
        error.clone()
    } else if !problem().is_empty() {
        Some(rsx! { "{problem}" })
    } else {
        hint.clone()
    };
    let separator: &'static str = if split_on_space { " " } else { ", " };

    let set_tags = move |next: Vec<String>| {
        if !controlled {
            own.set(next.clone());
        }
        if let Some(handler) = &on_change {
            handler.call(next);
        }
    };

    let normalize_one = move |raw: String| match normalize {
        Some(cb) => cb.call(raw),
        None => raw.trim().to_string(),
    };

    let make_commit = move |tags: Vec<String>| {
        let mut set_tags = set_tags;
        move |text: String, rest: String| {
            let parts: Vec<String> = split_tags(&text, split_on_space)
                .into_iter()
                .map(|piece| normalize_one(piece.to_string()))
                .filter(|part| !part.is_empty())
                .collect();
            if parts.is_empty() {
                draft.set(rest);
                return;
            }
            let plan = plan_commit(&parts, &tags, max, |part| {
                validate.and_then(|cb| cb.call(part.to_string()))
            });
            if !plan.added.is_empty() {
                let mut next = tags.clone();
                next.extend(plan.added.iter().cloned());
                said.set(match plan.added.as_slice() {
                    [only] => format!("{only} added"),
                    many => format!("{} added", many.len()),
                });
                set_tags(next);
            }
            let left = plan.left.join(separator);
            draft.set(
                [left, rest]
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(separator),
            );
            problem.set(plan.problem);
            if let Some(tag) = plan.duplicate {
                let generation = flash_count() + 1;
                flash_count.set(generation);
                flash.set(Some((tag, generation)));
            }
        }
    };
    let commit_for_render = make_commit(tags.clone());

    let class = class_list("ui-tag", &class);
    let shown_message_id = message.is_some().then(|| message_id.clone());
    let described = [shown_message_id, Some(live_id.clone())]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    let placeholder_now = if full {
        String::new()
    } else if tags.is_empty() {
        placeholder.clone()
    } else {
        "Add more…".to_string()
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-invalid": invalid.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            if label.is_some() || optional {
                div { class: "ui-tag__head",
                    if let Some(label) = label {
                        label { class: "ui-tag__label", id: "{label_id}", r#for: "{input_id}", {label} }
                    }
                    if optional {
                        span { class: "ui-tag__optional", "Optional" }
                    }
                }
            }
            div { class: "ui-tag__box", id: "{box_id}",
                for (index, tag) in tags.iter().enumerate() {
                    {
                        let is_last = index + 1 == tags.len();
                        let mut set_tags = set_tags;
                        let flash_value = flash().filter(|(t, _)| t == tag).map(|(_, n)| n.to_string());
                        let key_tags = tags.clone();
                        let key_box = box_id.clone();
                        let key_input = input_id.clone();
                        let button_tags = tags.clone();
                        let button_input = input_id.clone();
                        let remove_tag = tag.clone();
                        let button_remove = remove_tag.clone();
                        rsx! {
                            span {
                                key: "{tag}",
                                class: "ui-tag__chip",
                                tabindex: "-1",
                                title: "{tag}",
                                "data-armed": (armed() && is_last).then_some("true"),
                                "data-flash": flash_value,
                                onanimationend: move |event: AnimationEvent| {
                                    if event.animation_name() == "ui-tag-flash" {
                                        flash.set(None);
                                    }
                                },
                                onkeydown: move |event: KeyboardEvent| {
                                    match event.key() {
                                        Key::ArrowLeft => {
                                            event.prevent_default();
                                            document::eval(&focus_script(&key_box, &key_input, index.saturating_sub(1) as i64));
                                        }
                                        Key::ArrowRight => {
                                            event.prevent_default();
                                            document::eval(&focus_script(&key_box, &key_input, index as i64 + 1));
                                        }
                                        Key::Backspace | Key::Delete => {
                                            event.prevent_default();
                                            let mut next = key_tags.clone();
                                            let removed = next.remove(index);
                                            said.set(format!("{removed} removed"));
                                            set_tags(next);
                                            document::eval(&focus_script(&key_box, &key_input, index as i64));
                                        }
                                        _ => {}
                                    }
                                },
                                if avatar {
                                    span { class: "ui-tag__avatar", "aria-hidden": "true", "{avatar_letter(tag)}" }
                                }
                                span { class: "ui-tag__text", "{tag}" }
                                if !disabled {
                                    button {
                                        r#type: "button",
                                        class: "ui-tag__remove",
                                        tabindex: "-1",
                                        aria_label: "Remove {tag}",
                                        onpointerdown: move |event: PointerEvent| event.prevent_default(),
                                        onclick: move |_| {
                                            let next: Vec<String> = button_tags.iter().filter(|t| **t != button_remove).cloned().collect();
                                            said.set(format!("{remove_tag} removed"));
                                            set_tags(next);
                                            document::eval(&focus_input_script(&button_input));
                                        },
                                        svg { view_box: "0 0 12 12", width: "10", height: "10", "aria-hidden": "true",
                                            path { d: "M3 3l6 6M9 3l-6 6" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                input {
                    id: "{input_id}",
                    class: "ui-tag__input",
                    r#type: "text",
                    value: "{draft}",
                    placeholder: "{placeholder_now}",
                    disabled: disabled || full,
                    dir: "auto",
                    autocomplete: "off",
                    spellcheck: "false",
                    aria_label,
                    aria_invalid: invalid.then_some("true"),
                    aria_describedby: described,
                    oninput: {
                        let mut commit = commit_for_render.clone();
                        move |event: FormEvent| {
                            let text = event.value();
                            armed.set(false);
                            if !problem().is_empty() {
                                problem.set(String::new());
                            }
                            let pieces: Vec<&str> = text.split(|c: char| is_separator(c, split_on_space)).collect();
                            match pieces.split_last() {
                                Some((last, head)) if !head.is_empty() => commit(head.join(","), last.to_string()),
                                _ => draft.set(text.clone()),
                            }
                        }
                    },
                    onkeydown: {
                        let mut commit = commit_for_render.clone();
                        let mut set_tags = set_tags;
                        let tags = tags.clone();
                        let box_id = box_id.clone();
                        let input_id = input_id.clone();
                        move |event: KeyboardEvent| {
                            let key = event.key();
                            let separator_key = match &key {
                                Key::Character(c) => is_separator(c.chars().next().unwrap_or_default(), split_on_space) && c.chars().count() == 1,
                                _ => false,
                            };
                            if (matches!(key, Key::Enter | Key::Tab) || separator_key) && !draft().trim().is_empty() {
                                event.prevent_default();
                                commit(draft(), String::new());
                                return;
                            }
                            if separator_key {
                                event.prevent_default();
                                return;
                            }
                            if key == Key::Backspace && draft().is_empty() && !tags.is_empty() {
                                event.prevent_default();
                                if armed() {
                                    let mut next = tags.clone();
                                    let removed = next.pop().unwrap_or_default();
                                    said.set(format!("{removed} removed"));
                                    set_tags(next);
                                    armed.set(false);
                                } else {
                                    armed.set(true);
                                }
                                return;
                            }
                            if key == Key::ArrowLeft && draft().is_empty() && !tags.is_empty() {
                                event.prevent_default();
                                document::eval(&focus_script(&box_id, &input_id, -1));
                                return;
                            }
                            if key == Key::Escape && !draft().is_empty() {
                                event.prevent_default();
                                draft.set(String::new());
                                problem.set(String::new());
                            }
                            armed.set(false);
                        }
                    },
                    onblur: {
                        let mut commit = commit_for_render.clone();
                        move |_| {
                            armed.set(false);
                            if !draft().trim().is_empty() {
                                commit(draft(), String::new());
                            }
                        }
                    },
                }
            }
            if !name.is_empty() {
                for tag in tags.iter() {
                    input { key: "{tag}", r#type: "hidden", name: "{name}", value: "{tag}" }
                }
            }
            if message.is_some() || max.is_some() {
                div { class: "ui-tag__foot",
                    if let Some(message) = message {
                        p {
                            class: "ui-tag__message",
                            id: "{message_id}",
                            key: "{message_key}",
                            aria_live: shows_error.then_some("polite"),
                            {message}
                        }
                    }
                    if let Some(max) = max {
                        span { class: "ui-tag__count", "data-full": full.then_some("true"), "{tags.len()}/{max}" }
                    }
                }
            }
            span { class: "ui-tag__sr", id: "{live_id}", role: "status", "{said}" }
        }
    }
}
