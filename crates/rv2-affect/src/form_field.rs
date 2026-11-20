//! Form Field: the frame around any control. A label, a required mark or
//! "Optional", a quiet hint, an error that takes the hint's place, and a counter.
//! The control is drawn by `render`, which receives the ids and aria state to
//! spread onto it.
//!
//! Port losses: the React original clones a single child element and wires it
//! automatically; here the control is always rendered from `render`, so the
//! caller spreads `FormFieldControl` onto its element. The control's own
//! `aria-describedby` is not merged in; include it when building the control.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/form-field.css");

/// What the field hands its control. `described_by` names the hint and counter.
/// `label_id` is the label's id, for controls a `<label>` cannot name, such as a
/// radio group: pass it as `aria-labelledby`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FormFieldControl {
    pub id: String,
    pub described_by: Option<String>,
    pub invalid: bool,
    pub required: bool,
    pub label_id: String,
    pub disabled: bool,
}

#[component]
pub fn FormField(
    #[props(default)] label: Option<Element>,
    #[props(default)] hint: Option<Element>,
    #[props(default)] error: Option<Element>,
    #[props(default)] invalid: bool,
    #[props(default)] required: bool,
    #[props(default)] optional: bool,
    #[props(default)] counter: Option<Element>,
    #[props(default)] disabled: bool,
    #[props(default, into)] html_for: Option<String>,
    #[props(default)] class: String,
    render: Callback<FormFieldControl, Element>,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let auto = use_hook(|| format!("ui-form-field-{}", next_id()));
    let control_id = html_for.unwrap_or_else(|| format!("{auto}-control"));
    let label_id = format!("{auto}-label");
    let message_id = format!("{auto}-message");
    let counter_id = format!("{auto}-count");

    let has_error = error.is_some();
    let invalid = invalid || has_error;
    let message_key = if has_error { "error" } else { "hint" };
    let message = error.or(hint);
    let has_message = message.is_some();
    let has_counter = counter.is_some();

    let mut ids: Vec<String> = Vec::new();
    if has_message {
        ids.push(message_id.clone());
    }
    if has_counter {
        ids.push(counter_id.clone());
    }
    let control = FormFieldControl {
        id: control_id.clone(),
        described_by: (!ids.is_empty()).then(|| ids.join(" ")),
        invalid,
        required,
        label_id: label_id.clone(),
        disabled,
    };
    let body = render.call(control);
    let class = class_list("ui-fld", &class);

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-invalid": invalid.then_some("true"),
            "data-disabled": disabled.then_some("true"),
            ..attributes,
            if let Some(text) = label {
                div { class: "ui-fld__head",
                    label { class: "ui-fld__label", id: "{label_id}", r#for: "{control_id}",
                        {text}
                        if required {
                            span { class: "ui-fld__required", aria_hidden: "true", "*" }
                        }
                    }
                    if optional && !required {
                        span { class: "ui-fld__optional", "Optional" }
                    }
                }
            }
            {body}
            if has_message || has_counter {
                div { class: "ui-fld__foot",
                    p {
                        class: "ui-fld__message",
                        id: "{message_id}",
                        key: "{message_key}",
                        dir: "auto",
                        aria_live: has_error.then_some("polite"),
                        {message}
                    }
                    if let Some(count) = counter {
                        span { class: "ui-fld__count", id: "{counter_id}", {count} }
                    }
                }
            }
        }
    }
}
