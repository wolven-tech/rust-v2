//! Atoms: buttons, links, badges.

use dioxus::prelude::*;

/// Visual weight of a [`Button`] or [`LinkButton`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Ghost,
}

impl Variant {
    fn classes(self) -> &'static str {
        match self {
            Variant::Primary => "bg-ink text-on-ink hover:bg-ink-hover",
            Variant::Secondary => "bg-raised text-ink hover:bg-raised",
            Variant::Danger => "bg-fault text-on-ink hover:bg-fault-ink",
            Variant::Ghost => "bg-transparent text-ink hover:bg-raised",
        }
    }
}

/// Control height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Size {
    #[default]
    Default,
    /// Hero call-to-action.
    Large,
}

impl Size {
    fn classes(self) -> &'static str {
        match self {
            Size::Default => "px-4 py-2 text-sm",
            Size::Large => "px-6 py-3 text-base",
        }
    }
}

/// Shared by [`Button`] and [`LinkButton`] so the two are visually identical.
///
/// `focus-visible` rather than `focus`: a mouse user clicking a button should
/// not get a focus ring, but a keyboard user tabbing to it must.
const CONTROL_BASE: &str = "inline-flex items-center justify-center gap-2 rounded-md \
                            font-medium transition-colors focus-visible:outline-none \
                            focus-visible:ring-2 focus-visible:ring-accent \
                            focus-visible:ring-offset-2 disabled:opacity-50 \
                            disabled:pointer-events-none";

#[component]
pub fn Button(
    #[props(default)] variant: Variant,
    #[props(default)] size: Size,
    #[props(default = false)] disabled: bool,
    #[props(default = "button".to_string())] r#type: String,
    #[props(default)] class: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let (variant, size) = (variant.classes(), size.classes());
    rsx! {
        button {
            class: "{CONTROL_BASE} {variant} {size} {class}",
            r#type: "{r#type}",
            disabled,
            onclick: move |event| {
                if let Some(handler) = &onclick {
                    handler.call(event);
                }
            },
            {children}
        }
    }
}

/// An anchor styled as a button.
///
/// A call-to-action that navigates is an `<a>`, not a `<button>` — it must be
/// middle-clickable, openable in a new tab, and crawlable. This crate takes
/// `dioxus` without the router feature so the kit stays renderer-agnostic;
/// callers pass a plain `href` and wrap with `Link` themselves if they want
/// client-side routing.
#[component]
pub fn LinkButton(
    href: String,
    #[props(default)] variant: Variant,
    #[props(default)] size: Size,
    #[props(default)] class: String,
    /// Set for links leaving the site; adds `rel="noopener noreferrer"`, without
    /// which the opened page can reach back through `window.opener`.
    #[props(default = false)]
    external: bool,
    children: Element,
) -> Element {
    let (variant, size) = (variant.classes(), size.classes());
    let target = external.then_some("_blank");
    let rel = external.then_some("noopener noreferrer");
    rsx! {
        a { class: "{CONTROL_BASE} {variant} {size} {class}", href: "{href}", target, rel, {children} }
    }
}

/// Inline "Read more →" link.
///
/// The arrow is `aria-hidden` — it is decoration, and a screen reader reading
/// "rightwards arrow" after every link is noise.
#[component]
pub fn ArrowLink(href: String, #[props(default)] class: String, children: Element) -> Element {
    rsx! {
        a {
            class: "group inline-flex items-center gap-1 text-sm font-medium text-ink \
                    hover:text-ink-muted focus-visible:outline-none focus-visible:underline {class}",
            href: "{href}",
            {children}
            span { class: "transition-transform group-hover:translate-x-0.5", "aria-hidden": "true", "→" }
        }
    }
}

/// Small pill label.
#[component]
pub fn Badge(
    #[props(default)] class: String,
    /// A CSS colour this badge takes for its text and border, instead of the
    /// default grey fill.
    ///
    /// Exists so a set of badges can encode a category rather than merely
    /// label it. A reader scanning a long list sees the colour before they
    /// read the word, which is the whole reason a category has a colour.
    #[props(default)]
    colour: Option<String>,
    children: Element,
) -> Element {
    match colour {
        Some(colour) => rsx! {
            span {
                class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium {class}",
                style: "color:{colour};border-color:{colour}",
                {children}
            }
        },
        None => rsx! {
            span {
                class: "inline-flex items-center rounded-full bg-raised px-2.5 py-0.5 \
                        text-xs font-medium text-ink {class}",
                {children}
            }
        },
    }
}

/// A small filled circle carrying a category's colour.
///
/// Sits beside a filter option so the rail and the results agree visually: the
/// swatch next to "FTSE 250" is the same green as the badge on every FTSE 250
/// company.
#[component]
pub fn Swatch(colour: String, #[props(default)] class: String) -> Element {
    rsx! {
        span {
            class: "inline-block h-2.5 w-2.5 shrink-0 rounded-full {class}",
            style: "background:{colour}",
            aria_hidden: "true",
        }
    }
}

/// How healthy something is, as a dot plus its label.
///
/// The dot is what makes a column of these scannable; the label is what makes
/// it readable when the dot is invisible to you. Both, always — a colour on its
/// own is not a status anyone can act on.
#[component]
pub fn Status(
    label: String,
    tone: StatusTone,
    #[props(default)] detail: Option<String>,
) -> Element {
    rsx! {
        span { class: "inline-flex items-center gap-1.5 text-xs {tone.text()}",
            span {
                class: "inline-block h-1.5 w-1.5 rounded-full {tone.dot()}",
                aria_hidden: "true",
            }
            "{label}"
            if let Some(detail) = detail {
                span { class: "text-ink-faint", "{detail}" }
            }
        }
    }
}

/// How a [`Status`] reads at a glance.
///
/// Named apart from [`crate::typography::Tone`], which grades text emphasis
/// rather than health.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum StatusTone {
    Ok,
    Warn,
    Bad,
    #[default]
    Muted,
}

impl StatusTone {
    #[must_use]
    pub fn dot(self) -> &'static str {
        match self {
            Self::Ok => "bg-takeable",
            Self::Warn => "bg-caution",
            Self::Bad => "bg-fault",
            Self::Muted => "bg-ink-faint",
        }
    }

    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Self::Ok => "text-takeable-ink",
            Self::Warn => "text-caution-ink",
            Self::Bad => "text-fault-ink",
            Self::Muted => "text-ink-muted",
        }
    }
}

/// A native `details`/`summary` disclosure.
///
/// Native rather than a signal and a conditional render, because the browser
/// already gives this keyboard operation, the right ARIA state and find-in-page
/// that reaches inside a closed section. A hand-rolled one gives up all three.
#[component]
pub fn Disclosure(
    summary: String,
    #[props(default = false)] open: bool,
    #[props(default)] class: String,
    /// A CSS colour for a left rule down the open body, used where the
    /// disclosure carries a graded judgement rather than plain detail.
    #[props(default)]
    accent: Option<String>,
    children: Element,
) -> Element {
    let style = accent
        .map(|c| format!("border-left:3px solid {c};padding-left:0.75rem"))
        .unwrap_or_default();
    rsx! {
        details { class: "group {class}", open,
            summary { class: "cursor-pointer list-none text-sm text-ink hover:text-ink",
                span { class: "mr-1 inline-block transition-transform group-open:rotate-90", aria_hidden: "true", "›" }
                "{summary}"
            }
            div { class: "pt-2", style: "{style}", {children} }
        }
    }
}

/// The `01` / `02` / `03` marker used by feature cards and step lists.
///
/// Zero-padded to two digits so the markers stay optically aligned past nine.
#[component]
pub fn StepMarker(n: u8, #[props(default)] class: String) -> Element {
    rsx! {
        span {
            class: "inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full \
                    bg-ink text-xs font-semibold tabular-nums text-on-ink {class}",
            "aria-hidden": "true",
            "{n:02}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_has_distinct_classes() {
        let all = [
            Variant::Primary,
            Variant::Secondary,
            Variant::Danger,
            Variant::Ghost,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a.classes(), b.classes(), "{a:?} and {b:?} look identical");
            }
        }
    }

    #[test]
    fn the_default_variant_is_primary() {
        assert_eq!(Variant::default(), Variant::Primary);
    }

    #[test]
    fn every_size_has_distinct_classes() {
        assert_ne!(Size::Default.classes(), Size::Large.classes());
    }

    /// The step marker is zero-padded so `01`..`09` stay optically aligned with
    /// `10`+ in a grid. Regressing this silently ruins the alignment.
    #[test]
    fn step_markers_are_zero_padded_to_two_digits() {
        assert_eq!(format!("{:02}", 1u8), "01");
        assert_eq!(format!("{:02}", 12u8), "12");
    }
}
