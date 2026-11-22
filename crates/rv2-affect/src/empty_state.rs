//! Empty State: what a list, table or panel shows while it holds nothing. An icon
//! on a small tile, a title, one line on what will show up, and an action.
//!
//! Port losses: Hugeicons icon objects are not accepted; pass an inline svg as
//! `icon`. `icon={null}` in the original is `no_icon: true` here, because an
//! `Option<Element>` cannot tell "unset" from "null" once a default is set.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/empty-state.css");

#[component]
pub fn EmptyState(
    title: Element,
    #[props(default)] text: Option<Element>,
    #[props(default)] action: Option<Element>,
    #[props(default)] icon: Option<Element>,
    #[props(default)] no_icon: bool,
    #[props(default)] bare: bool,
    #[props(default)] tall: bool,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let icon = if no_icon {
        None
    } else {
        Some(icon.unwrap_or_else(file_icon))
    };
    let class = class_list("ui-estc", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-bare": bare.then_some("true"),
            "data-tall": tall.then_some("true"),
            ..attributes,
            if let Some(glyph) = icon {
                span { class: "ui-estc__icon", aria_hidden: "true", {glyph} }
            }
            p { class: "ui-estc__title", {title} }
            if let Some(line) = text {
                p { class: "ui-estc__text", {line} }
            }
            if let Some(step) = action {
                div { class: "ui-estc__action", {step} }
            }
        }
    }
}

/// The default glyph: a file. Hugeicons Free (MIT, © Hugeicons, hugeicons.com).
fn file_icon() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", fill: "none", "focusable": "false",
            path { d: "M8 17H16", stroke: "currentColor", stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "1.9" }
            path { d: "M8 13H12", stroke: "currentColor", stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "1.9" }
            path {
                d: "M13 2.5V3C13 5.82843 13 7.24264 13.8787 8.12132C14.7574 9 16.1716 9 19 9H19.5M20 10.6569V14C20 17.7712 20 19.6569 18.8284 20.8284C17.6569 22 15.7712 22 12 22C8.22876 22 6.34315 22 5.17157 20.8284C4 19.6569 4 17.7712 4 14V9.45584C4 6.21082 4 4.58831 4.88607 3.48933C5.06508 3.26731 5.26731 3.06508 5.48933 2.88607C6.58831 2 8.21082 2 11.4558 2C12.1614 2 12.5141 2 12.8372 2.11401C12.9044 2.13772 12.9702 2.165 13.0345 2.19575C13.3436 2.34355 13.593 2.593 14.0919 3.09188L18.8284 7.82843C19.4065 8.40649 19.6955 8.69552 19.8478 9.06306C20 9.4306 20 9.83935 20 10.6569Z",
                stroke: "currentColor",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                stroke_width: "1.9",
            }
        }
    }
}
