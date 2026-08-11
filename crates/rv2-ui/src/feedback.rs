//! Surfaces and state feedback: cards, headers, empty/loading/error.

use dioxus::prelude::*;

use crate::primitives::{Button, Variant};

#[component]
pub fn Card(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "rounded-lg border border-slate-200 bg-white p-4 shadow-sm {class}", {children} }
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
                h1 { class: "text-2xl font-semibold text-slate-900", "{title}" }
                if let Some(text) = subtitle {
                    p { class: "mt-1 text-sm text-slate-500", "{text}" }
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
        div { class: "rounded-lg border border-dashed border-slate-300 p-10 text-center",
            p { class: "text-sm font-medium text-slate-900", "{title}" }
            if let Some(text) = description {
                p { class: "mt-1 text-sm text-slate-500", "{text}" }
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
                div { key: "{index}", class: "h-4 w-full animate-pulse rounded bg-slate-200" }
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
            class: "flex items-center justify-between gap-4 rounded-md border border-red-200 bg-red-50 px-4 py-3",
            role: "alert",
            p { class: "text-sm text-red-800", "{message}" }
            if let Some(handler) = onretry {
                Button { variant: Variant::Secondary, onclick: handler, "Retry" }
            }
        }
    }
}
