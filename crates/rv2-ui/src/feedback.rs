//! Surfaces and state feedback: cards, headers, empty/loading/error.

use dioxus::prelude::*;

use crate::primitives::{Button, Variant};

/// Inner spacing of a [`Card`].
///
/// Two values, because two is what the pages need: a list of cards that should
/// read as a list, and a single card that is the whole of its section. Hand-made
/// surfaces had been using four (`p-3`, `p-4`, `p-6`, `p-8`), which is the
/// drift and not a design. `None` is a card whose content brings its own edge —
/// a table running to the border.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Pad {
    None,
    #[default]
    Default,
    Roomy,
}

impl Pad {
    fn classes(self) -> &'static str {
        match self {
            Pad::None => "",
            Pad::Default => "p-4",
            Pad::Roomy => "p-6",
        }
    }
}

/// A bordered surface.
///
/// Flat, with no shadow. Every hand-made surface in `apps/web` and
/// `apps/hiring` was flat, and the hiring ledger spends its one shadow on a
/// verdict — "nothing else on the page casts a shadow at all" — which a shadow
/// on every card would quietly break.
///
/// `class` is for layout inside the card: flex direction, gaps, alignment.
#[component]
pub fn Card(
    #[props(default)] pad: Pad,
    #[props(default)] class: String,
    children: Element,
) -> Element {
    let pad = pad.classes();
    rsx! {
        div { class: "rounded-lg border border-rule-soft bg-surface {pad} {class}", {children} }
    }
}

#[component]
pub fn PageHeader(
    title: String,
    #[props(default)] subtitle: Option<String>,
    #[props(default)] actions: Option<Element>,
) -> Element {
    rsx! {
        header { class: "mb-6 flex items-start justify-between gap-4",
            div {
                h1 { class: "text-2xl font-semibold text-ink", "{title}" }
                if let Some(text) = subtitle {
                    p { class: "mt-1 text-sm text-ink-muted", "{text}" }
                }
            }
            div { class: "flex gap-2", {actions} }
        }
    }
}

/// Shown when a collection is legitimately empty — as distinct from loading
/// (use [`Skeleton`]) or failed (use [`ErrorBanner`]). Conflating the three is
/// the most common way a list UI lies to the user.
#[component]
pub fn EmptyState(
    title: String,
    #[props(default)] description: Option<String>,
    #[props(default)] action: Option<Element>,
) -> Element {
    rsx! {
        div { class: "rounded-lg border border-dashed border-rule p-10 text-center",
            p { class: "text-sm font-medium text-ink", "{title}" }
            if let Some(text) = description {
                p { class: "mt-1 text-sm text-ink-muted", "{text}" }
            }
            div { class: "mt-4 flex justify-center", {action} }
        }
    }
}

#[component]
pub fn Skeleton(#[props(default = 3)] lines: u32) -> Element {
    rsx! {
        div { class: "space-y-2", role: "status", "aria-label": "Loading",
            for index in 0..lines {
                div { key: "{index}", class: "h-4 w-full animate-pulse rounded bg-raised" }
            }
        }
    }
}

#[component]
pub fn ErrorBanner(
    message: String,
    #[props(default)] onretry: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between gap-4 rounded-md border border-fault bg-surface px-4 py-3",
            role: "alert",
            p { class: "text-sm text-fault-ink", "{message}" }
            if let Some(handler) = onretry {
                Button { variant: Variant::Secondary, onclick: handler, "Retry" }
            }
        }
    }
}
