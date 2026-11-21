//! Text area: a labelled multi-line field that grows with its text between
//! `min_rows` and `max_rows`, then scrolls. Optional expand button, character
//! count, hint or error line, and Ctrl/Cmd+Enter as a submit shortcut.
//!
//! Port losses: a form reset does not clear an uncontrolled count (the field
//! clears, the count keeps the old text); the Ctrl/Cmd+Enter handler does not
//! skip IME composition; `aria-invalid` and the placeholder-as-name fallback
//! are not read from the spread attributes. The height fit runs in the
//! browser through `document::eval`, so it applies after the DOM update.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/text-area.css");

/// Thousands separators, as `toLocaleString()` in an English locale.
pub fn format_count(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// The count is in UTF-16 code units, as `String.length` in JS.
pub fn text_length(text: &str) -> usize {
    text.encode_utf16().count()
}

#[component]
pub fn TextArea(
    #[props(default, into)] id: Option<String>,
    #[props(default)] label: Option<Element>,
    #[props(default)] hint: Option<Element>,
    #[props(default)] error: Option<Element>,
    #[props(default)] optional: bool,
    #[props(default)] counter: bool,
    #[props(default = 3)] min_rows: u32,
    #[props(default = 10)] max_rows: u32,
    #[props(default)] expandable: bool,
    #[props(default = 16)] expanded_rows: u32,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] on_expanded_change: Option<EventHandler<bool>>,
    #[props(default)] on_submit_shortcut: Option<EventHandler<String>>,
    #[props(default)] value: Option<String>,
    #[props(default)] default_value: String,
    #[props(default)] on_input: Option<EventHandler<FormEvent>>,
    #[props(default)] onchange: Option<EventHandler<FormEvent>>,
    #[props(default)] onfocus: Option<EventHandler<FocusEvent>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(default)] disabled: bool,
    #[props(default)] read_only: bool,
    #[props(default)] max_length: Option<usize>,
    #[props(default, into)] aria_describedby: Option<String>,
    #[props(default)] class: String,
    #[props(default)] input_class: String,
    #[props(extends = GlobalAttributes, extends = textarea)] attributes: Vec<Attribute>,
) -> Element {
    let auto_id = use_hook(|| format!("ui-ta-{}", next_id()));
    let input_id = id.unwrap_or_else(|| format!("{auto_id}-input"));
    let message_id = format!("{auto_id}-message");
    let counter_id = format!("{auto_id}-count");

    let mut own = use_signal(|| default_value.clone());
    let text = value.clone().unwrap_or(own());

    let mut own_expanded = use_signal(|| false);
    let is_expanded = expandable && expanded.unwrap_or(own_expanded());
    let mut gliding = use_signal(|| false);
    let mut glide_generation = use_signal(|| 0u32);

    let invalid = error.is_some();
    let message = error.clone().or(hint.clone());
    let show_count = counter && max_length.is_some();
    let length = text_length(&text);

    let fit_id = input_id.clone();
    use_effect(use_reactive(
        (&text, &is_expanded, &min_rows, &max_rows, &expanded_rows),
        move |(_text, expanded, min_rows, max_rows, expanded_rows)| {
            document::eval(&format!(
                "const el = document.getElementById({fit_id:?}); if (!el) return;
                 const fit = () => {{
                   const cs = getComputedStyle(el);
                   const line = parseFloat(cs.lineHeight) || 20;
                   const pad = parseFloat(cs.paddingTop) + parseFloat(cs.paddingBottom);
                   const min = ({expanded} ? Math.max({expanded_rows}, {min_rows}) : {min_rows}) * line + pad;
                   const max = ({expanded} ? Math.max({expanded_rows}, {max_rows}) : Math.max({max_rows}, {min_rows})) * line + pad;
                   const before = el.style.height;
                   el.style.height = 'auto';
                   const needed = el.scrollHeight;
                   const next = Math.min(Math.max(needed, min), max);
                   el.style.height = before;
                   void el.offsetHeight;
                   el.style.height = next + 'px';
                   el.style.overflowY = needed > max + 1 ? 'auto' : 'hidden';
                 }};
                 el.__uiFit = fit;
                 fit();
                 if (!el.__uiResize && typeof ResizeObserver !== 'undefined') {{
                   let width = el.clientWidth;
                   el.__uiResize = new ResizeObserver(() => {{
                     if (el.clientWidth === width) return;
                     width = el.clientWidth;
                     el.__uiFit();
                   }});
                   el.__uiResize.observe(el);
                 }}"
            ));
        },
    ));

    let class = class_list("ui-ta", &class);
    let input_class = class_list("ui-ta__input", &input_class);
    let described = [
        aria_describedby.clone(),
        message.as_ref().map(|_| message_id.clone()),
        show_count.then(|| counter_id.clone()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");
    let keyshortcuts = on_submit_shortcut
        .is_some()
        .then_some("Meta+Enter Control+Enter");
    let message_key = if invalid { "error" } else { "hint" };

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-invalid": invalid.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            "data-readonly": read_only.then_some("true"),
            "data-expandable": expandable.then_some("true"),
            if label.is_some() || optional {
                div { class: "ui-ta__head",
                    if let Some(label) = label {
                        label { class: "ui-ta__label", r#for: "{input_id}", {label} }
                    }
                    if optional {
                        span { class: "ui-ta__optional", "Optional" }
                    }
                }
            }
            div { class: "ui-ta__box",
                textarea {
                    id: "{input_id}",
                    class: input_class,
                    rows: "{min_rows}",
                    value: "{text}",
                    disabled,
                    readonly: read_only,
                    maxlength: max_length.map(|m| m.to_string()),
                    dir: "auto",
                    aria_invalid: invalid.then_some("true"),
                    aria_describedby: (!described.is_empty()).then_some(described),
                    "aria-keyshortcuts": keyshortcuts,
                    "data-gliding": gliding().then_some("true"),
                    oninput: move |event: FormEvent| {
                        if value.is_none() {
                            own.set(event.value());
                        }
                        if let Some(handler) = &on_input {
                            handler.call(event);
                        }
                    },
                    onkeydown: move |event: KeyboardEvent| {
                        if let Some(handler) = &on_submit_shortcut && event.key() == Key::Enter && (event.modifiers().meta() || event.modifiers().ctrl()) {
                            event.prevent_default();
                            handler.call(text.clone());
                        }
                        if let Some(handler) = &onkeydown {
                            handler.call(event);
                        }
                    },
                    onchange: move |event| {
                        if let Some(handler) = &onchange {
                            handler.call(event);
                        }
                    },
                    onfocus: move |event| {
                        if let Some(handler) = &onfocus {
                            handler.call(event);
                        }
                    },
                    onblur: move |event| {
                        if let Some(handler) = &onblur {
                            handler.call(event);
                        }
                    },
                    ..attributes,
                }
                if expandable && !disabled {
                    button {
                        r#type: "button",
                        class: "ui-ta__expand",
                        aria_label: if is_expanded { "Less room" } else { "More room" },
                        aria_pressed: if is_expanded { "true" } else { "false" },
                        "aria-controls": "{input_id}",
                        onclick: move |_| {
                            let next = !is_expanded;
                            let generation = glide_generation() + 1;
                            glide_generation.set(generation);
                            gliding.set(true);
                            spawn(async move {
                                sleep(320).await;
                                if glide_generation() == generation {
                                    gliding.set(false);
                                }
                            });
                            if expanded.is_none() {
                                own_expanded.set(next);
                            }
                            if let Some(handler) = &on_expanded_change {
                                handler.call(next);
                            }
                        },
                        span { class: "ui-ta__swap", key: if is_expanded { "less" } else { "more" },
                            if is_expanded { CollapseIcon {} } else { ExpandIcon {} }
                        }
                    }
                }
            }
            if message.is_some() || show_count {
                div { class: "ui-ta__foot",
                    if let Some(message) = message {
                        p {
                            class: "ui-ta__message",
                            id: "{message_id}",
                            key: "{message_key}",
                            dir: "auto",
                            aria_live: invalid.then_some("polite"),
                            {message}
                        }
                    }
                    if let Some(max) = max_length.filter(|_| show_count) {
                        span {
                            class: "ui-ta__count",
                            id: "{counter_id}",
                            "data-full": (length == max).then_some("true"),
                            "data-over": (length > max).then_some("true"),
                            "{format_count(length)}/{format_count(max)}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ExpandIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M16.5 3h4.5v4.5M21 3l-6 6M7.5 21H3v-4.5M3 21l6-6" }
        }
    }
}

#[component]
fn CollapseIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M14.5 5v4.5H19M14.5 9.5 20 4M9.5 19v-4.5H5M9.5 14.5 4 20" }
        }
    }
}
