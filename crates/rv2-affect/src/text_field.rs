//! Text field: a labelled single-line input with an optional hint, a prefix and
//! suffix, a clear button (also on Escape), reveal for passwords, copy, and a
//! character count. An error takes the hint's place.
//!
//! Port losses: a string `prefix` is wrapped in `.ui-tfc__affix-text` by the
//! React original; here the caller wraps long text itself. Clearing does not
//! fire `on_input`, because a Dioxus component cannot synthesise an input
//! event; a controlled field is emptied by its parent in `on_clear`. A click on
//! the box padding does not focus the input (pointer-down routing is not
//! ported). `aria-invalid` and the placeholder-as-name fallback are not read
//! from the spread attributes.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, sleep};

const CSS: Asset = asset!("/assets/text-field.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextFieldSize {
    /// 32px high.
    Sm,
    /// 36px high.
    #[default]
    Md,
    /// 40px high.
    Lg,
}

impl TextFieldSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[component]
pub fn TextField(
    #[props(default, into)] id: Option<String>,
    #[props(default)] label: Option<Element>,
    #[props(default)] hint: Option<Element>,
    #[props(default)] error: Option<Element>,
    #[props(default)] prefix: Option<Element>,
    #[props(default)] suffix: Option<Element>,
    #[props(default)] size: TextFieldSize,
    #[props(default)] optional: bool,
    #[props(default)] clearable: bool,
    #[props(default)] copyable: bool,
    #[props(default)] counter: bool,
    #[props(default)] on_clear: Option<EventHandler<()>>,
    #[props(default = "text".to_string(), into)] r#type: String,
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
    #[props(extends = GlobalAttributes, extends = input)] attributes: Vec<Attribute>,
) -> Element {
    let auto_id = use_hook(|| format!("ui-tf-{}", next_id()));
    let input_id = id.unwrap_or_else(|| format!("{auto_id}-input"));
    let message_id = format!("{auto_id}-message");
    let counter_id = format!("{auto_id}-count");

    let controlled = value.is_some();
    let mut own = use_signal(|| default_value.clone());
    let text = value.clone().unwrap_or(own());
    let mut input_node = use_signal(|| None::<MountedEvent>);

    let is_password = r#type == "password";
    let mut revealed = use_signal(|| false);
    let mut copied = use_signal(|| false);
    let mut copy_generation = use_signal(|| 0u32);

    let invalid = error.is_some();
    let message = error.clone().or(hint.clone());
    let show_count = counter && max_length.is_some();
    let length = text.encode_utf16().count();
    let show_clear = clearable && !disabled && !read_only && !text.is_empty();
    let message_key = if invalid { "error" } else { "hint" };
    let input_type = if is_password && revealed() {
        "text".to_string()
    } else {
        r#type.clone()
    };

    let mut clear = move || {
        if !controlled {
            own.set(String::new());
        }
        if let Some(node) = input_node() {
            spawn(async move {
                let _ = node.set_focus(true).await;
            });
        }
        if let Some(handler) = &on_clear {
            handler.call(());
        }
    };

    let copy_text = text.clone();
    let copy_id = input_id.clone();
    let copy = move |_| {
        let script = format!(
            "try {{ await navigator.clipboard.writeText({copy_text:?}); return true; }} catch {{ document.getElementById({copy_id:?})?.select(); return false; }}"
        );
        spawn(async move {
            let ok = document::eval(&script)
                .await
                .ok()
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if ok {
                let generation = copy_generation() + 1;
                copy_generation.set(generation);
                copied.set(true);
                sleep(1600).await;
                if copy_generation() == generation {
                    copied.set(false);
                }
            }
        });
    };

    let class = class_list("ui-tfc", &class);
    let input_class = class_list("ui-tfc__input", &input_class);
    let described = [
        aria_describedby.clone(),
        message.as_ref().map(|_| message_id.clone()),
        show_count.then(|| counter_id.clone()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ");

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-size": size.as_str(),
            "data-invalid": invalid.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            "data-readonly": read_only.then_some("true"),
            if label.is_some() || optional {
                div { class: "ui-tfc__head",
                    if let Some(label) = label {
                        label { class: "ui-tfc__label", r#for: "{input_id}", {label} }
                    }
                    if optional {
                        span { class: "ui-tfc__optional", "Optional" }
                    }
                }
            }
            div { class: "ui-tfc__box",
                if let Some(prefix) = prefix {
                    span { class: "ui-tfc__affix", "data-side": "start", {prefix} }
                }
                input {
                    id: "{input_id}",
                    class: input_class,
                    r#type: "{input_type}",
                    value: "{text}",
                    disabled,
                    readonly: read_only,
                    maxlength: max_length.map(|m| m.to_string()),
                    aria_invalid: invalid.then_some("true"),
                    aria_describedby: (!described.is_empty()).then_some(described),
                    dir: "auto",
                    "data-revealed": is_password.then_some(if revealed() { "true" } else { "false" }),
                    onmounted: move |node: MountedEvent| input_node.set(Some(node)),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape && show_clear {
                            event.prevent_default();
                            clear();
                        }
                        if let Some(handler) = &onkeydown {
                            handler.call(event);
                        }
                    },
                    oninput: move |event: FormEvent| {
                        if !controlled {
                            own.set(event.value());
                        }
                        if let Some(handler) = &on_input {
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
                if let Some(suffix) = suffix {
                    span { class: "ui-tfc__affix", "data-side": "end", {suffix} }
                }
                if show_clear {
                    button {
                        r#type: "button",
                        class: "ui-tfc__action",
                        aria_label: "Clear",
                        tabindex: "-1",
                        onclick: move |_| clear(),
                        svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "currentColor", "aria-hidden": "true",
                            path { d: "M12 2.25c-5.385 0-9.75 4.365-9.75 9.75s4.365 9.75 9.75 9.75 9.75-4.365 9.75-9.75S17.385 2.25 12 2.25Zm3.53 12.22a.75.75 0 1 1-1.06 1.06L12 13.06l-2.47 2.47a.75.75 0 0 1-1.06-1.06L10.94 12 8.47 9.53a.75.75 0 0 1 1.06-1.06L12 10.94l2.47-2.47a.75.75 0 1 1 1.06 1.06L13.06 12l2.47 2.47Z" }
                        }
                    }
                }
                if is_password && !disabled {
                    button {
                        r#type: "button",
                        class: "ui-tfc__action",
                        aria_label: if revealed() { "Hide password" } else { "Show password" },
                        aria_pressed: if revealed() { "true" } else { "false" },
                        onclick: move |_| revealed.set(!revealed()),
                        span { class: "ui-tfc__swap", key: if revealed() { "hide" } else { "show" },
                            if revealed() { EyeOffIcon {} } else { EyeIcon {} }
                        }
                    }
                }
                if copyable {
                    button {
                        r#type: "button",
                        class: "ui-tfc__action",
                        "data-copied": copied().then_some("true"),
                        aria_label: if copied() { "Copied" } else { "Copy" },
                        onclick: copy,
                        disabled: disabled || text.is_empty(),
                        span { class: "ui-tfc__swap", key: if copied() { "done" } else { "copy" },
                            if copied() { CheckIcon {} } else { CopyIcon {} }
                        }
                    }
                    span { class: "ui-tfc__sr", role: "status", if copied() { "Copied" } }
                }
            }
            if message.is_some() || show_count {
                div { class: "ui-tfc__foot",
                    if let Some(message) = message {
                        p {
                            class: "ui-tfc__message",
                            id: "{message_id}",
                            key: "{message_key}",
                            dir: "auto",
                            aria_live: invalid.then_some("polite"),
                            {message}
                        }
                    }
                    if let Some(max) = max_length.filter(|_| show_count) {
                        span {
                            class: "ui-tfc__count",
                            id: "{counter_id}",
                            "data-full": (length == max).then_some("true"),
                            "data-over": (length > max).then_some("true"),
                            "{length}/{max}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EyeIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M21.544 11.045C21.848 11.471 22 11.685 22 12c0 .316-.152.529-.456.955C20.178 14.871 16.689 19 12 19c-4.69 0-8.178-4.13-9.544-6.045C2.152 12.529 2 12.315 2 12c0-.316.152-.529.456-.955C3.822 9.129 7.311 5 12 5c4.69 0 8.178 4.13 9.544 6.045Z" }
            path { d: "M15 12a3 3 0 1 0-6 0 3 3 0 0 0 6 0Z" }
        }
    }
}

#[component]
fn EyeOffIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M19.439 15.439c.996-.91 1.719-1.82 2.105-2.36.304-.426.456-.64.456-.955 0-.316-.152-.529-.456-.955C20.178 9.253 16.689 5.124 12 5.124c-.908 0-1.77.155-2.582.418M6.74 6.869c-2.09 1.267-3.6 3.083-4.284 4.04-.304.426-.456.64-.456.955 0 .316.152.529.456.955C3.822 14.735 7.311 18.864 12 18.864c2.008 0 3.786-.757 5.26-1.746" }
            path { d: "M9.858 10a2.98 2.98 0 0 0-.858 2.124 3 3 0 0 0 5.124 2.12M3 3l18 18" }
        }
    }
}

#[component]
fn CopyIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M9 15c0-2.828 0-4.243.879-5.121C10.757 9 12.172 9 15 9h1c2.828 0 4.243 0 5.121.879C22 10.757 22 12.172 22 15v1c0 2.828 0 4.243-.879 5.121C20.243 22 18.828 22 16 22h-1c-2.828 0-4.243 0-5.121-.879C9 20.243 9 18.828 9 16v-1Z" }
            path { d: "M17 9c-.003-2.957-.047-4.489-.908-5.538a4 4 0 0 0-.554-.554C14.43 2 12.788 2 9.5 2c-3.287 0-4.931 0-6.038.908a4 4 0 0 0-.554.554C2 4.57 2 6.212 2 9.5c0 3.287 0 4.931.908 6.038a4 4 0 0 0 .554.554c1.05.86 2.58.906 5.538.908" }
        }
    }
}

#[component]
fn CheckIcon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", width: "16", height: "16", fill: "none", stroke: "currentColor", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { class: "ui-tfc__check", d: "M5 12.5 9.5 17 19 7", "pathLength": "1" }
        }
    }
}
