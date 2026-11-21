//! Switch: on or off for something that takes effect at once. A real checkbox
//! with `role="switch"`, a label, and a quiet second line. Placed at the start of
//! the row or, for settings rows, at the end.
//!
//! Port losses: the `ref` forwarding of the original is not available; the
//! checkbox state is read through its attributes (`checked`, `onchange`).

use dioxus::prelude::*;

use crate::internal::{class_list, next_id};

const CSS: Asset = asset!("/assets/switch.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchPlacement {
    /// The switch before the text.
    #[default]
    Start,
    /// The switch after the text, across the row: the settings look.
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchSize {
    /// 16px high.
    Sm,
    /// 20px high.
    #[default]
    Md,
}

#[component]
pub fn Switch(
    #[props(default)] label: Option<Element>,
    #[props(default)] description: Option<Element>,
    #[props(default)] placement: SwitchPlacement,
    #[props(default)] size: SwitchSize,
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
    let auto = use_hook(|| format!("ui-switch-{}", next_id()));
    let input_id = id.unwrap_or_else(|| format!("{auto}-input"));
    let desc_id = format!("{auto}-desc");
    let has_description = description.is_some();
    let described_by = match (aria_describedby, has_description) {
        (Some(ids), true) => Some(format!("{ids} {desc_id}")),
        (Some(ids), false) => Some(ids),
        (None, true) => Some(desc_id.clone()),
        (None, false) => None,
    };
    let placement_attr = match placement {
        SwitchPlacement::Start => "start",
        SwitchPlacement::End => "end",
    };
    let size_attr = match size {
        SwitchSize::Sm => "sm",
        SwitchSize::Md => "md",
    };
    let has_text = label.is_some() || description.is_some();
    let class = class_list("ui-sw", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-placement": placement_attr,
            "data-size": size_attr,
            "data-disabled": disabled.then_some("true"),
            label { class: "ui-sw__row", r#for: "{input_id}",
                span { class: "ui-sw__control",
                    input {
                        id: "{input_id}",
                        r#type: "checkbox",
                        role: "switch",
                        class: "ui-sw__input",
                        disabled,
                        aria_describedby: described_by,
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
                    span { class: "ui-sw__track", aria_hidden: "true",
                        span { class: "ui-sw__thumb" }
                    }
                }
                if has_text {
                    span { class: "ui-sw__text",
                        if let Some(text) = label {
                            span { class: "ui-sw__label", {text} }
                        }
                        if let Some(text) = description {
                            span { class: "ui-sw__desc", id: "{desc_id}", {text} }
                        }
                    }
                }
            }
        }
    }
}
