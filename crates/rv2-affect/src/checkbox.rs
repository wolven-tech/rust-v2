//! Checkbox: a real checkbox with a label and a quiet second line, a third
//! "some of them" state, an error, and a card variant where the whole row is the
//! target. The box fills and its check draws itself.
//!
//! Port losses: `error={true}` in the original is `invalid: true` here; the
//! `ref` forwarding is not available, read state through attributes instead.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/checkbox.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckboxVariant {
    /// The box and its text.
    #[default]
    Plain,
    /// A bordered row that is clickable as a whole and lights up when checked.
    Card,
}

#[component]
pub fn Checkbox(
    #[props(default)] label: Option<Element>,
    #[props(default)] description: Option<Element>,
    #[props(default)] indeterminate: bool,
    #[props(default)] error: Option<Element>,
    #[props(default)] invalid: bool,
    #[props(default)] variant: CheckboxVariant,
    #[props(default, into)] id: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default, into)] aria_describedby: Option<String>,
    #[props(default)] class: String,
    #[props(default)] onchange: Option<EventHandler<FormEvent>>,
    #[props(default)] oninput: Option<EventHandler<FormEvent>>,
    #[props(default)] onfocus: Option<EventHandler<FocusEvent>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(extends = GlobalAttributes, extends = input)] attributes: Vec<Attribute>,
) -> Element {
    let auto = use_hook(|| format!("ui-checkbox-{}", next_id()));
    let input_id = id.unwrap_or_else(|| format!("{auto}-input"));
    let desc_id = format!("{auto}-desc");
    let error_id = format!("{auto}-error");

    let has_description = description.is_some();
    let shows_error = error.is_some();
    let invalid = invalid || shows_error;
    let mut ids: Vec<String> = Vec::new();
    ids.extend(aria_describedby);
    if has_description {
        ids.push(desc_id.clone());
    }
    if shows_error {
        ids.push(error_id.clone());
    }
    let described_by = (!ids.is_empty()).then(|| ids.join(" "));
    let variant_attr = match variant {
        CheckboxVariant::Plain => "plain",
        CheckboxVariant::Card => "card",
    };
    let has_text = label.is_some() || description.is_some();

    // `indeterminate` is a DOM property with no attribute form.
    let indeterminate_id = input_id.clone();
    use_effect(use_reactive((&indeterminate,), move |(value,)| {
        document::eval(&format!(
            "const el = document.getElementById({indeterminate_id:?}); if (el) el.indeterminate = {value};"
        ));
    }));

    let class = class_list("ui-chk", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-variant": variant_attr,
            "data-invalid": invalid.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            label { class: "ui-chk__row", r#for: "{input_id}",
                span { class: "ui-chk__control",
                    input {
                        id: "{input_id}",
                        r#type: "checkbox",
                        class: "ui-chk__input",
                        disabled,
                        aria_invalid: invalid.then_some("true"),
                        aria_describedby: described_by,
                        "data-indeterminate": indeterminate.then_some("true"),
                        onchange: move |event| {
                            if let Some(handler) = &onchange {
                                handler.call(event);
                            }
                        },
                        oninput: move |event| {
                            if let Some(handler) = &oninput {
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
                        onkeydown: move |event| {
                            if let Some(handler) = &onkeydown {
                                handler.call(event);
                            }
                        },
                        ..attributes,
                    }
                    span { class: "ui-chk__box", aria_hidden: "true",
                        svg {
                            class: "ui-chk__mark",
                            view_box: "0 0 16 16",
                            width: "16",
                            height: "16",
                            path { class: "ui-chk__tick", d: "M4 8.5 6.75 11.25 12 5", "pathLength": "1" }
                            path { class: "ui-chk__dash", d: "M4.5 8h7", "pathLength": "1" }
                        }
                    }
                }
                if has_text {
                    span { class: "ui-chk__text",
                        if let Some(text) = label {
                            span { class: "ui-chk__label", {text} }
                        }
                        if let Some(text) = description {
                            span { class: "ui-chk__desc", id: "{desc_id}", {text} }
                        }
                    }
                }
            }
            if let Some(message) = error {
                p { class: "ui-chk__error", id: "{error_id}", aria_live: "polite", {message} }
            }
        }
    }
}
