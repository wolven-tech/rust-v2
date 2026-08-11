//! Layout primitives.
//!
//! These exist so pages never hand-roll `mx-auto max-w-… px-…` strings. A
//! marketing page is mostly layout, and inconsistent container widths are the
//! most visible way a site looks unfinished.

use dioxus::prelude::*;

/// Horizontal content width.
///
/// `Prose` is deliberately narrower than `Default`: long-form reading breaks
/// down past roughly 75 characters per line, so guide pages want a tighter
/// measure than a three-column card grid does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Width {
    Prose,
    #[default]
    Default,
    Wide,
}

impl Width {
    fn classes(self) -> &'static str {
        match self {
            Width::Prose => "max-w-2xl",
            Width::Default => "max-w-5xl",
            Width::Wide => "max-w-7xl",
        }
    }
}

/// Centred, padded content column.
#[component]
pub fn Container(#[props(default)] width: Width, children: Element) -> Element {
    let width = width.classes();
    rsx! {
        div { class: "mx-auto w-full {width} px-6", {children} }
    }
}

/// Vertical rhythm between page sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Space {
    Tight,
    #[default]
    Default,
    Loose,
}

impl Space {
    fn classes(self) -> &'static str {
        match self {
            Space::Tight => "py-10 sm:py-12",
            Space::Default => "py-16 sm:py-20",
            Space::Loose => "py-24 sm:py-32",
        }
    }
}

/// A page section with consistent vertical padding.
///
/// Takes an optional `id` so a nav link can target it — an in-page anchor is
/// the cheapest navigation a marketing site has, and it needs no router.
#[component]
pub fn Section(
    #[props(default)] space: Space,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let space = space.classes();
    rsx! {
        section { class: "{space} {class}", id, {children} }
    }
}

/// Responsive column grid.
///
/// Collapses to a single column on small screens in every case; `columns` is
/// the widest arrangement, not a fixed one.
#[component]
pub fn Grid(
    #[props(default = 3)] columns: u8,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let cols = match columns {
        1 => "grid-cols-1",
        2 => "grid-cols-1 sm:grid-cols-2",
        4 => "grid-cols-1 sm:grid-cols-2 lg:grid-cols-4",
        _ => "grid-cols-1 sm:grid-cols-2 lg:grid-cols-3",
    };
    rsx! {
        div { class: "grid gap-6 {cols} {class}", {children} }
    }
}

/// Vertical flex stack with a uniform gap.
#[component]
pub fn Stack(
    #[props(default = 4)] gap: u8,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let gap = match gap {
        1 => "gap-1",
        2 => "gap-2",
        3 => "gap-3",
        6 => "gap-6",
        8 => "gap-8",
        _ => "gap-4",
    };
    rsx! {
        div { class: "flex flex-col {gap} {class}", {children} }
    }
}

/// Horizontal flex row that wraps rather than overflowing on narrow screens.
#[component]
pub fn Row(
    #[props(default = 3)] gap: u8,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let gap = match gap {
        1 => "gap-1",
        2 => "gap-2",
        4 => "gap-4",
        6 => "gap-6",
        _ => "gap-3",
    };
    rsx! {
        div { class: "flex flex-wrap items-center {gap} {class}", {children} }
    }
}

/// Horizontal rule.
///
/// `aria-hidden` because it is decoration; a screen reader announcing
/// "separator" between every section is noise.
#[component]
pub fn Divider(#[props(default)] class: String) -> Element {
    rsx! {
        hr { class: "border-slate-200 {class}", "aria-hidden": "true" }
    }
}
