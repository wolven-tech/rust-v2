//! Spinner: a small arc turning on a faint ring, drawn in `currentColor`.
//!
//! With `label` it is announced as a status; without one the svg is hidden from
//! screen readers.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/spinner.css");

/// A turning arc. `size` is in pixels.
#[component]
pub fn Spinner(
    #[props(default = 14)] size: u32,
    #[props(default, into)] label: Option<String>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = svg)] attributes: Vec<Attribute>,
) -> Element {
    let announced = label.is_some();
    let class = class_list("ui-spn", &class);
    let size = size.to_string();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        svg {
            class,
            view_box: "0 0 16 16",
            width: "{size}",
            height: "{size}",
            role: announced.then_some("status"),
            "aria-label": label,
            "aria-hidden": (!announced).then_some("true"),
            ..attributes,
            circle { cx: "8", cy: "8", r: "6", fill: "none", stroke: "currentColor", "stroke-opacity": "0.2", stroke_width: "2" }
            path { d: "M8 2a6 6 0 0 1 6 6", fill: "none", stroke: "currentColor", stroke_width: "2", stroke_linecap: "round" }
        }
    }
}
