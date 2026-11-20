//! Tactile controls: buttons that sit on a ledge and press into it, a save
//! toggle that confirms itself, filter chips, a media card and an editorial
//! masthead.
//!
//! Extracted from Ready Play Days (`apps/web` in that repository), which had
//! been writing them as `class: "primary action-button"` overrides on
//! [`crate::Button`] — a colour and radius change through `class`, which UI5
//! forbids. Here they are instructions with operands instead.
//!
//! Geometry, ledge shadows and motion live in `assets/tactile.css`, imported
//! by each app after `theme.css`. Colour comes through `--rv2-tactile-*` hooks
//! that default to the kit's registers (UI3), so an app that sets nothing gets
//! a sober ink-and-rule control; `assets/themes/playful.css` is the warm
//! palette the components were drawn in.

use dioxus::prelude::*;

/// Which of the two tactile treatments a control takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Press {
    /// Ink fill on a deeper ledge. One per view: the action the page is for.
    #[default]
    Primary,
    /// Soft fill with an edge. Everything else that still deserves a ledge.
    Soft,
    /// Square, two-pixel ink border, hard diagonal shadow. The ledger look
    /// Logo Handoff, Talk Run Card, Sponsor Ledger and Solar Quote Check each
    /// drew by hand.
    Offset,
}

impl Press {
    fn class(self) -> &'static str {
        match self {
            Press::Primary => "rv2-press rv2-press--primary",
            Press::Soft => "rv2-press rv2-press--soft",
            Press::Offset => "rv2-press rv2-press--offset",
        }
    }
}

/// A decorative icon at the start of a tactile control.
///
/// Always `alt=""` and `aria-hidden`: the control's text is its name, and an
/// icon announced as well reads the name twice.
fn press_icon(icon: Option<String>) -> Element {
    rsx! {
        if let Some(src) = icon {
            img { class: "rv2-press__icon", src: "{src}", alt: "", aria_hidden: "true" }
        }
    }
}

/// A button that rests on a ledge, lifts on hover and presses down on click.
///
/// The icon tilts on hover and on keyboard focus alike, so the affordance a
/// mouse user sees is not withheld from a keyboard user.
#[component]
pub fn PressButton(
    #[props(default)] press: Press,
    /// Image URL for a decorative leading icon.
    #[props(default)]
    icon: Option<String>,
    #[props(default = false)] disabled: bool,
    #[props(default = "button".to_string())] r#type: String,
    #[props(default)] class: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    rsx! {
        button {
            class: "{press.class()} {class}",
            r#type: "{r#type}",
            disabled,
            onclick: move |event| {
                if !disabled && let Some(handler) = &onclick {
                    handler.call(event);
                }
            },
            {press_icon(icon)}
            {children}
        }
    }
}

/// [`PressButton`]'s look on an anchor, for a call-to-action that navigates.
#[component]
pub fn PressLink(
    href: String,
    #[props(default)] press: Press,
    #[props(default)] icon: Option<String>,
    /// Opens in a new tab with `rel="noopener noreferrer"`.
    #[props(default = false)]
    external: bool,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let target = external.then_some("_blank");
    let rel = external.then_some("noopener noreferrer");
    rsx! {
        a { class: "{press.class()} {class}", href: "{href}", target, rel,
            {press_icon(icon)}
            {children}
        }
    }
}

/// A save/unsave toggle whose gem pulses each time it is pressed.
///
/// The pulse restarts on every press, including two in quick succession: the
/// animation alternates between two identical keyframe names, because a CSS
/// animation that keeps the same name does not replay when its class is
/// re-applied.
///
/// State is the caller's: `saved` in, `ontoggle` with the next value out.
#[component]
pub fn SaveToggle(
    saved: bool,
    ontoggle: EventHandler<bool>,
    /// Image URL for the gem; a heart glyph when absent.
    #[props(default)]
    icon: Option<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    // 0 = never pressed, so the first render does not pulse.
    let mut presses = use_signal(|| 0_u32);
    let pulse = match presses() {
        0 => "",
        n if n % 2 == 1 => "rv2-save--pulse-a",
        _ => "rv2-save--pulse-b",
    };
    rsx! {
        button {
            class: "rv2-save {pulse} {class}",
            r#type: "button",
            aria_pressed: if saved { "true" } else { "false" },
            onclick: move |_| {
                presses += 1;
                ontoggle.call(!saved);
            },
            span { class: "rv2-save__gem", aria_hidden: "true",
                if let Some(src) = icon {
                    img { src: "{src}", alt: "" }
                } else {
                    "♥"
                }
            }
            {children}
        }
    }
}

/// A labelled row of [`Chip`]s.
///
/// `role="group"` with a name, so a screen reader announces "Filter
/// activities, group" before the first chip rather than a run of unexplained
/// toggles.
#[component]
pub fn ChipGroup(label: String, #[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "rv2-chips {class}", role: "group", aria_label: "{label}", {children} }
    }
}

/// One selectable filter chip. `active` is exposed as `aria-pressed`, not as
/// colour alone.
#[component]
pub fn Chip(
    #[props(default = false)] active: bool,
    #[props(default)] icon: Option<String>,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    rsx! {
        button {
            class: "rv2-chip",
            r#type: "button",
            aria_pressed: if active { "true" } else { "false" },
            onclick: move |event| {
                if let Some(handler) = &onclick {
                    handler.call(event);
                }
            },
            {press_icon(icon)}
            {children}
        }
    }
}

/// A browsing card: a picture that eases forward on hover, a category line, a
/// linked title, a body and actions pinned to the bottom.
///
/// The picture and the title are two links to one place. The picture link is
/// named by `title` and taken out of the tab order, so a keyboard user tabs
/// once per card rather than twice.
#[component]
pub fn MediaCard(
    href: String,
    image: String,
    title: String,
    #[props(default)] label: Option<String>,
    #[props(default)] actions: Option<Element>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    rsx! {
        article { class: "rv2-media-card {class}",
            a { class: "rv2-media-card__image", href: "{href}", tabindex: "-1", aria_label: "{title}",
                img { src: "{image}", alt: "", loading: "lazy" }
            }
            if let Some(label) = label {
                p { class: "rv2-media-card__label", "{label}" }
            }
            crate::Heading { level: 3, size: crate::HeadingSize::Title,
                a { class: "rv2-media-card__title", href: "{href}", "{title}" }
            }
            div { class: "rv2-media-card__body", {children} }
            if let Some(actions) = actions {
                div { class: "rv2-media-card__actions", {actions} }
            }
        }
    }
}

/// A newspaper masthead: a strap line, the wordmark, a subtitle, and a nav
/// between a double rule and a single one.
///
/// For a site whose identity is editorial — a gazette, a desk, a journal —
/// where [`crate::ProductHeader`]'s sticky app bar would read as software.
#[component]
pub fn Masthead(
    brand: String,
    #[props(default = "/".to_string())] brand_href: String,
    /// Image URL for a mark set beside the wordmark.
    #[props(default)]
    mark: Option<String>,
    #[props(default)] subtitle: Option<String>,
    /// Left and right of the strap above the wordmark; the right one is
    /// dropped on narrow screens.
    #[props(default)]
    strap: Option<(String, String)>,
    #[props(default)] items: Vec<crate::NavLink>,
    #[props(default)] current: Option<String>,
    #[props(default)] class: String,
) -> Element {
    let current = current.as_deref();
    rsx! {
        header { class: "rv2-masthead {class}",
            if let Some((left, right)) = strap {
                div { class: "rv2-masthead__strap",
                    span { "{left}" }
                    span { "{right}" }
                }
            }
            a { class: "rv2-masthead__brand", href: "{brand_href}",
                if let Some(src) = mark {
                    img { src: "{src}", alt: "", aria_hidden: "true" }
                }
                "{brand}"
            }
            if let Some(subtitle) = subtitle {
                p { class: "rv2-masthead__subtitle", "{subtitle}" }
            }
            if !items.is_empty() {
                nav { class: "rv2-masthead__nav", aria_label: "Main navigation",
                    for item in items.iter() {
                        a {
                            href: "{item.href}",
                            aria_current: (item.key.is_some() && item.key.as_deref() == current).then_some("page"),
                            target: item.external.then_some("_blank"),
                            rel: item.external.then_some("noopener noreferrer"),
                            "{item.label}"
                        }
                    }
                }
            }
        }
    }
}

/// A four-point star that unfolds once when it mounts. Decoration only; place
/// it inside a `position: relative` parent.
#[component]
pub fn Sparkle(#[props(default)] class: String) -> Element {
    rsx! {
        span { class: "rv2-sparkle {class}", aria_hidden: "true" }
    }
}
