//! Card: a white card on a grey tray, with a header (icon, title, quiet line,
//! action), the content, and a footer that sits on the tray below the card.
//! `interactive` makes the whole card a click target through the title's link
//! or button; `selected` darkens the tray.
//!
//! Port losses: a `CardFooter` cannot be detected among the children, so the
//! footer goes in Card's `footer` prop; a `CardFooter` placed among the children
//! renders inside the white card.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/card.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardElement {
    #[default]
    Div,
    Section,
    Article,
    Li,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardTitleTag {
    H2,
    #[default]
    H3,
    H4,
    P,
}

#[component]
pub fn Card(
    #[props(default)] element: CardElement,
    #[props(default)] interactive: bool,
    #[props(default)] selected: bool,
    #[props(default)] children: Option<Element>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] class: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let interactive = interactive.then_some("");
    let selected = selected.then_some("");
    let class = class_list("ui-crd", &class);
    let main = children.map(|content| rsx! { div { class: "ui-crd__card", {content} } });
    let inner = rsx! {
        {main}
        {footer}
    };
    let on_click = move |event: MouseEvent| {
        if let Some(handler) = &onclick {
            handler.call(event);
        }
    };
    let on_key_down = move |event: KeyboardEvent| {
        if let Some(handler) = &onkeydown {
            handler.call(event);
        }
    };
    let node = match element {
        CardElement::Div => rsx! {
            div { class, "data-interactive": interactive, "data-selected": selected, onclick: on_click, onkeydown: on_key_down, ..attributes, {inner} }
        },
        CardElement::Section => rsx! {
            section { class, "data-interactive": interactive, "data-selected": selected, onclick: on_click, onkeydown: on_key_down, ..attributes, {inner} }
        },
        CardElement::Article => rsx! {
            article { class, "data-interactive": interactive, "data-selected": selected, onclick: on_click, onkeydown: on_key_down, ..attributes, {inner} }
        },
        CardElement::Li => rsx! {
            li { class, "data-interactive": interactive, "data-selected": selected, onclick: on_click, onkeydown: on_key_down, ..attributes, {inner} }
        },
    };
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        {node}
    }
}

/// The top of a card: an icon, a title with an optional quiet line, and an action
/// on the title's line.
#[component]
pub fn CardHeader(
    #[props(default)] title: Option<Element>,
    #[props(default)] description: Option<Element>,
    #[props(default)] icon: Option<Element>,
    #[props(default)] action: Option<Element>,
    #[props(default)] title_as: CardTitleTag,
    #[props(default)] children: Option<Element>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let title = title.map(|text| match title_as {
        CardTitleTag::H2 => rsx! { h2 { class: "ui-crd__title", {text} } },
        CardTitleTag::H3 => rsx! { h3 { class: "ui-crd__title", {text} } },
        CardTitleTag::H4 => rsx! { h4 { class: "ui-crd__title", {text} } },
        CardTitleTag::P => rsx! { p { class: "ui-crd__title", {text} } },
    });
    let class = class_list("ui-crd__head", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class, ..attributes,
            if let Some(icon) = icon {
                span { class: "ui-crd__icon", {icon} }
            }
            div { class: "ui-crd__titles",
                {title}
                if let Some(text) = description {
                    p { class: "ui-crd__desc", {text} }
                }
                {children}
            }
            if let Some(action) = action {
                div { class: "ui-crd__action", {action} }
            }
        }
    }
}

#[component]
pub fn CardContent(
    #[props(default)] children: Option<Element>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let class = class_list("ui-crd__body", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class, ..attributes, {children} }
    }
}

/// A row on the tray under the card: status or meta at the start, buttons at the
/// end. Give the first button `margin-inline-start: auto`.
#[component]
pub fn CardFooter(
    #[props(default)] children: Option<Element>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let class = class_list("ui-crd__foot", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class, ..attributes, {children} }
    }
}
