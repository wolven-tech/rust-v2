//! Typographic scale.
//!
//! The scale is fixed here rather than left to per-page Tailwind classes so
//! headings stay consistent across `apps/web` and `apps/app`. Size and
//! semantic level are separate props on purpose: a section heading that must
//! be an `<h2>` for document outline reasons can still be rendered at display
//! size without lying to assistive technology about the structure.

use dioxus::prelude::*;

/// Visual size of a [`Heading`], independent of its semantic level.
///
/// The three larger sizes set `font-display` and `font-heading`, which the
/// kit's theme points at the body family at semibold. An app with a display
/// face sets `--font-display` and `--font-weight-heading` and every heading
/// follows, instead of each call site spelling the family and weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeadingSize {
    /// Hero headline.
    Display,
    #[default]
    /// Section heading.
    Section,
    /// The name of one entry in a list of them, or of a single item's page.
    Title,
    /// Card or sub-section heading.
    Card,
}

impl HeadingSize {
    fn classes(self) -> &'static str {
        match self {
            HeadingSize::Display => "font-display font-heading text-4xl sm:text-5xl tracking-tight",
            HeadingSize::Section => "font-display font-heading text-2xl sm:text-3xl tracking-tight",
            HeadingSize::Title => "font-display font-heading text-xl sm:text-2xl leading-tight",
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
    let class = format!("{size} text-ink {class}");
    match level {
        1 => rsx! { h1 { class, {children} } },
        3 => rsx! { h3 { class, {children} } },
        4 => rsx! { h4 { class, {children} } },
        _ => rsx! { h2 { class, {children} } },
    }
}

/// Body text tone.
///
/// The tone owns the paragraph's colour, and a `class` passed to [`Text`]
/// cannot override it reliably: two colour utilities on one element resolve by
/// stylesheet order, not by which was written last. So a paragraph that needs
/// a different colour needs a tone, not a class (UI5). `OnInk` is the one that
/// was missing — secondary text on an ink-filled plate, which `apps/web` had
/// been writing by hand because `Text` kept painting it dark-on-dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tone {
    #[default]
    Default,
    Muted,
    Small,
    Lead,
    OnInk,
}

impl Tone {
    fn classes(self) -> &'static str {
        match self {
            Tone::Default => "text-base text-ink-muted",
            Tone::Muted => "text-sm text-ink-muted",
            Tone::Small => "text-xs text-ink-muted",
            Tone::Lead => "text-lg text-ink-muted",
            Tone::OnInk => "text-sm text-on-ink-muted",
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
        p { class: "text-xs font-medium uppercase tracking-widest text-ink-muted {class}", {children} }
    }
}
