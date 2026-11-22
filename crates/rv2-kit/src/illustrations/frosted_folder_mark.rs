//! Frosted folder mark: a decorative folder whose frosted front pocket lifts to show the papers
//! inside. It stands for a collection of documents on a card.
//!
//! Design choice: the pocket hinges on its bottom edge and the papers rise by transform only, so
//! the open state is one short movement with no layout change.
//! Design choice: the mark is never focusable. Focus opens it when an ancestor carries
//! `kit-folder-scope` (the focusable card), which is the caller's job to add.
//! Design choice: `size_token` sets the mark's width; the aspect ratio is fixed and the papers
//! stay inside the box at every step.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/frosted-folder-mark.css");

/// The design system's size steps for the mark.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SizeStep {
    Sm,
    #[default]
    Md,
    Lg,
}

impl SizeStep {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

/// Rendered width in CSS px for a size step.
pub fn folder_px(size: SizeStep) -> u32 {
    match size {
        SizeStep::Sm => 48,
        SizeStep::Md => 72,
        SizeStep::Lg => 96,
    }
}

#[component]
pub fn FrostedFolderMark(
    #[props(default, into)] label: Option<String>,
    #[props(default = true)] open_on_hover: bool,
    #[props(default = true)] open_when_focused: bool,
    #[props(default)] size_token: SizeStep,
    #[props(default)] disabled: bool,
    #[props(default)] class: String,
) -> Element {
    let class = class_list("kit-folder", &class);
    let style = format!("--kit-folder-size: {}px;", folder_px(size_token));
    let named = label.is_some();
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        span {
            class,
            style: "{style}",
            role: named.then_some("img"),
            "aria-label": label,
            "aria-hidden": (!named).then_some("true"),
            "data-size": size_token.as_str(),
            "data-open-hover": open_on_hover.to_string(),
            "data-open-focus": open_when_focused.to_string(),
            "data-disabled": disabled.then_some("true"),
            span { class: "kit-folder__back" }
            span { class: "kit-folder__tab" }
            span { class: "kit-folder__papers",
                span { class: "kit-folder__sheet" }
                span { class: "kit-folder__sheet" }
                span { class: "kit-folder__sheet" }
            }
            span { class: "kit-folder__pocket" }
        }
    }
}
