//! Separator: a 0.5px hairline between groups, or an upright line between tools. A
//! horizontal separator can carry a short label in its middle.
//!
//! Port losses: `label` is an `Element`, so the separator cannot take its
//! accessible name from a text label as the React original does; pass
//! `aria_label` when `decorative` is false.

use dioxus::prelude::*;

use crate::internal::class_list;

const CSS: Asset = asset!("/assets/separator.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
    /// A line across, as wide as its container.
    #[default]
    Horizontal,
    /// An upright line, 16px high in a row.
    Vertical,
}

impl SeparatorOrientation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// `decorative` (the default) hides the line from screen readers; `false` makes
/// it a `role="separator"` that splits content into announced sections.
#[component]
pub fn Separator(
    #[props(default)] orientation: SeparatorOrientation,
    #[props(default = true)] decorative: bool,
    #[props(default)] label: Option<Element>,
    #[props(default)] class: String,
    #[props(extends = GlobalAttributes, extends = div)] attributes: Vec<Attribute>,
) -> Element {
    let vertical = orientation == SeparatorOrientation::Vertical;
    let label = if vertical { None } else { label };
    let has_label = label.is_some();
    let role = match (decorative, has_label) {
        (true, false) => Some("none"),
        (true, true) => None,
        (false, _) => Some("separator"),
    };
    let aria_orientation = (!decorative && vertical).then_some("vertical");
    let class = class_list("ui-sepc", &class);
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div {
            class,
            "data-orientation": orientation.as_str(),
            "data-label": has_label.then_some(""),
            role,
            aria_orientation,
            ..attributes,
            if let Some(text) = label {
                span { class: "ui-sepc__line", aria_hidden: "true" }
                span { class: "ui-sepc__label", {text} }
                span { class: "ui-sepc__line", aria_hidden: "true" }
            }
        }
    }
}
