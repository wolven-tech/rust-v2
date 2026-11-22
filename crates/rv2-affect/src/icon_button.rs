//! Icon Button: a square button that is only an icon. Ghost, a white tool with a
//! hairline shadow, or a dark chip. It can stay pressed as a toggle, and its label
//! is both the accessible name and the tooltip.
//!
//! Port losses: Hugeicons icon objects are not accepted; pass an inline svg as
//! `icon`.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/icon-button.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonVariant {
    /// Just the icon; a grey layer appears on hover.
    #[default]
    Ghost,
    /// White with a hairline shadow.
    Secondary,
    /// A dark chip.
    Primary,
}

impl IconButtonVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ghost => "ghost",
            Self::Secondary => "secondary",
            Self::Primary => "primary",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonSize {
    /// 24 × 24.
    Sm,
    /// 28 × 28.
    #[default]
    Md,
}

impl IconButtonSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
        }
    }
}

#[component]
pub fn IconButton(
    icon: Element,
    #[props(into)] label: String,
    #[props(default)] variant: IconButtonVariant,
    #[props(default)] size: IconButtonSize,
    #[props(default)] pressed: Option<bool>,
    #[props(default, into)] title: Option<String>,
    #[props(default = "button".to_string(), into)] r#type: String,
    #[props(default)] class: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] onfocus: Option<EventHandler<FocusEvent>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(extends = GlobalAttributes, extends = button)] attributes: Vec<Attribute>,
) -> Element {
    // The label doubles as the tooltip; an empty title leaves it out.
    let title = title.unwrap_or_else(|| label.clone());
    let title = (!title.is_empty()).then_some(title);
    let aria_pressed = pressed.map(|on| if on { "true" } else { "false" });
    let class = class_list("ui-icbc", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        button {
            r#type,
            class,
            "data-variant": variant.as_str(),
            "data-size": size.as_str(),
            aria_label: "{label}",
            title,
            aria_pressed,
            onclick: move |event| {
                if let Some(handler) = &onclick {
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
            span { class: "ui-icbc__icon", aria_hidden: "true", {icon} }
        }
    }
}
