//! Typographic scale.
//!
//! The scale is fixed here rather than left to per-page Tailwind classes so
//! headings stay consistent across `apps/web` and `apps/app`. Size and
//! semantic level are separate props on purpose: a section heading that must
//! be an `<h2>` for document outline reasons can still be rendered at display
//! size without lying to assistive technology about the structure.

use dioxus::prelude::*;

/// Visual size of a [`Heading`], independent of its semantic level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadingSize {
    /// Hero headline.
    Display,
    #[default]
    /// Section heading.
    Section,
    /// Card or sub-section heading.
    Card,
}

impl HeadingSize {
    fn classes(self) -> &'static str {
        match self {
            HeadingSize::Display => "text-4xl sm:text-5xl font-semibold tracking-tight",
            HeadingSize::Section => "text-2xl sm:text-3xl font-semibold tracking-tight",
            HeadingSize::Card => "text-base font-semibold",
        }
    }
}

/// A heading at an explicit semantic level.
///
/// `level` drives the tag, `size` drives the appearance. Defaulting `level` to
/// 2 is deliberate: a page has exactly one `<h1>`, and it is almost always the
/// hero, so every other heading opting into 2 is the safer default.
#[component]
pub fn Heading(
    #[props(default = 2)] level: u8,
    #[props(default)] size: HeadingSize,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let size = size.classes();
    let class = format!("{size} text-slate-900 {class}");
    match level {
        1 => rsx! { h1 { class, {children} } },
        3 => rsx! { h3 { class, {children} } },
        4 => rsx! { h4 { class, {children} } },
        _ => rsx! { h2 { class, {children} } },
    }
}

/// Body text tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tone {
    #[default]
    Default,
    Muted,
    Small,
    Lead,
}

impl Tone {
    fn classes(self) -> &'static str {
        match self {
            Tone::Default => "text-base text-slate-700",
            Tone::Muted => "text-sm text-slate-600",
            Tone::Small => "text-xs text-slate-500",
            Tone::Lead => "text-lg text-slate-600",
        }
    }
}

/// A paragraph of body copy.
#[component]
pub fn Text(
    #[props(default)] tone: Tone,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let tone = tone.classes();
    rsx! {
        p { class: "{tone} {class}", {children} }
    }
}

/// Small uppercase label above a heading.
///
/// Rendered as a `<p>`, not a heading: it is a visual lead-in, and promoting it
/// to `<h3>` would put a meaningless entry in the document outline.
#[component]
pub fn Eyebrow(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        p { class: "text-xs font-medium uppercase tracking-widest text-slate-500 {class}", {children} }
    }
}
