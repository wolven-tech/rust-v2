//! Button: primary, secondary, ghost and destructive, three sizes, a loading
//! shimmer, and a width that glides when the label or icon changes.
//!
//! Port losses: the React original keys each icon by its component type, so
//! swapping one icon for another replays the blur-in. Dioxus cannot compare
//! `Element`s by type, so the icon replays only when `loading` flips; pass a
//! different `icon_key` to replay it on a swap. A label cut short gets no
//! automatic `title` — set one.

use dioxus::prelude::*;

use crate::internal::{class_list, next_id, prefers_motion_script};

const CSS: Asset = asset!("/assets/button.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// A dark chip.
    #[default]
    Primary,
    /// White with a hairline shadow.
    Secondary,
    /// Text only; the grey layer appears on hover.
    Ghost,
    /// A light red fill, for deleting.
    Destructive,
}

impl ButtonVariant {
    fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Ghost => "ghost",
            Self::Destructive => "destructive",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// 32px.
    Sm,
    /// 40px.
    #[default]
    Md,
    /// 48px.
    Lg,
}

impl ButtonSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// A native `<button>`. With no `label`, an icon renders square and needs an
/// `aria_label`. While `loading`, clicks are swallowed but the button stays
/// focusable.
#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(default)] loading: bool,
    #[props(default)] disabled: bool,
    #[props(default, into)] label: Option<String>,
    #[props(default)] icon: Option<Element>,
    #[props(default)] icon_right: Option<Element>,
    #[props(default)] icon_key: u32,
    #[props(default = "button".to_string(), into)] r#type: String,
    #[props(default)] class: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] onfocus: Option<EventHandler<FocusEvent>>,
    #[props(default)] onblur: Option<EventHandler<FocusEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(extends = GlobalAttributes, extends = button)] attributes: Vec<Attribute>,
) -> Element {
    let id = use_hook(|| format!("ui-button-{}", next_id()));
    let mut ready = use_signal(|| false);
    use_effect(move || ready.set(true));

    let label = label.filter(|text| !text.is_empty());
    let has_icon = icon.is_some() || icon_right.is_some();
    let icon_only = label.is_none() && has_icon;
    let label_key = label.clone().unwrap_or_default();
    let icon_state = format!("{}-{icon_key}", loading && icon.is_some());
    let content = format!("{label_key}|{icon_state}|{}", icon_right.is_some());

    // The width remembered from the previous render glides to the new one.
    let morph_id = id.clone();
    use_effect(use_reactive((&content,), move |(content,)| {
        let script = format!(
            "const el = document.getElementById({id:?}); if (!el) return;
             const width = el.getBoundingClientRect().width;
             const from = parseFloat(el.dataset.width || '0');
             const changed = el.dataset.content !== undefined && el.dataset.content !== {content:?};
             el.dataset.width = width; el.dataset.content = {content:?};
             if (!changed || !from || Math.abs(from - width) < 0.5 || !({motion})) return;
             el.animate([{{ width: from + 'px' }}, {{ width: width + 'px' }}],
               {{ duration: 480, easing: 'cubic-bezier(0.25, 1, 0.5, 1)' }});",
            id = morph_id,
            motion = prefers_motion_script(),
        );
        document::eval(&script);
    }));

    let class = class_list("ui-buttonc", &class);
    let icon_left = !icon_only && icon.is_some();
    let icon_right_shown = !icon_only && icon_right.is_some();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        button {
            id: "{id}",
            r#type,
            class,
            "data-variant": variant.as_str(),
            "data-size": size.as_str(),
            "data-loading": loading.then_some("true"),
            "data-icon-only": icon_only.then_some("true"),
            "data-icon-left": icon_left.then_some("true"),
            "data-icon-right": icon_right_shown.then_some("true"),
            "data-ready": ready().then_some("true"),
            disabled,
            aria_busy: loading.then_some("true"),
            aria_disabled: loading.then_some("true"),
            onclick: move |event| {
                if loading {
                    event.prevent_default();
                    return;
                }
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
            // A new key remounts the part that changed, which replays its enter animation.
            if let Some(icon) = icon {
                for key in [icon_state.clone()] {
                    span { key: "{key}", class: "ui-buttonc__icon", aria_hidden: "true",
                        if loading { Spinner {} } else { {icon.clone()} }
                    }
                }
            }
            if let Some(text) = label {
                for key in [text.clone()] {
                    span { key: "{key}", class: "ui-buttonc__label", "data-text": "{text}", dir: "auto", "{text}" }
                }
            }
            if let Some(icon) = icon_right {
                span { class: "ui-buttonc__icon", aria_hidden: "true", {icon} }
            }
        }
    }
}

#[component]
fn Spinner() -> Element {
    rsx! {
        svg {
            class: "ui-buttonc__spinner",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            circle { cx: "12", cy: "12", r: "8.5", opacity: "0.2" }
            path { d: "M12 3.5a8.5 8.5 0 0 1 8.5 8.5" }
        }
    }
}
