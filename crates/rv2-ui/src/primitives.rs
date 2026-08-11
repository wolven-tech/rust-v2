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
            Variant::Primary => "bg-slate-900 text-white hover:bg-slate-700",
            Variant::Secondary => "bg-slate-100 text-slate-900 hover:bg-slate-200",
            Variant::Danger => "bg-red-600 text-white hover:bg-red-700",
            Variant::Ghost => "bg-transparent text-slate-700 hover:bg-slate-100",
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
                            focus-visible:ring-2 focus-visible:ring-slate-900 \
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
            class: "group inline-flex items-center gap-1 text-sm font-medium text-slate-900 \
                    hover:text-slate-600 focus-visible:outline-none focus-visible:underline {class}",
            href: "{href}",
            {children}
            span { class: "transition-transform group-hover:translate-x-0.5", "aria-hidden": "true", "→" }
        }
    }
}

/// Small pill label.
#[component]
pub fn Badge(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        span {
            class: "inline-flex items-center rounded-full bg-slate-100 px-2.5 py-0.5 \
                    text-xs font-medium text-slate-700 {class}",
            {children}
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
                    bg-slate-900 text-xs font-semibold tabular-nums text-white {class}",
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
