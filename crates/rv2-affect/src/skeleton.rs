//! Skeleton: a grey shape that stands in for text, an avatar or a button while
//! data loads, with a sheen that sweeps across it. `Loading` wraps the
//! skeletons of one area as `role="status"` with `aria-busy`, fading them in
//! after 150ms so a fast load never flashes them.
//!
//! Port losses: `width`, `height`, `radius` and `line_height` take CSS strings
//! ("24px", "50%", "1lh"), not bare numbers.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/skeleton.css");

/// One grey bar. With `line` it sits in a box one text line tall (`line_height`),
/// centred, so a row keeps its height when the words land.
#[component]
pub fn Skeleton(
    #[props(default = "100%".to_string(), into)] width: String,
    #[props(default = "12px".to_string(), into)] height: String,
    #[props(default, into)] radius: Option<String>,
    #[props(default)] line: bool,
    #[props(default = "1lh".to_string(), into)] line_height: String,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = span)] attributes: Vec<Attribute>,
) -> Element {
    let radius_css = radius
        .map(|r| format!(" border-radius: {r};"))
        .unwrap_or_default();
    let bar_style = format!("width: {width}; height: {height};{radius_css}");
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        if line {
            span {
                class: class_list("ui-skl__line", &class),
                style: "height: {line_height};",
                aria_hidden: "true",
                ..attributes,
                span { class: "ui-skl", style: "{bar_style}", aria_hidden: "true" }
            }
        } else {
            span {
                class: class_list("ui-skl", &class),
                style: "{bar_style}",
                aria_hidden: "true",
                ..attributes,
            }
        }
    }
}

/// The skeletons that stand in for one area. Give it the real layout's class,
/// so the grid or list it replaces is the one the data fills.
#[component]
pub fn Loading(
    #[props(default = "Loading".to_string(), into)] label: String,
    #[props(default)] class: String,
    #[props(default)] children: Option<Element>,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class: class_list("ui-skl-loading", &class),
            role: "status",
            aria_busy: "true",
            aria_label: "{label}",
            ..attributes,
            {children}
        }
    }
}
