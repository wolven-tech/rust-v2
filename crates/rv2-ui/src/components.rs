//! The component kit.
//!
//! Every component is a plain function taking props, so it can be rendered by
//! any Dioxus renderer. None of them fetch: data comes in as props, so the same
//! component works under CSR (`apps/app`) and SSG (`apps/web`).

use dioxus::prelude::*;
use rv2_api_types::PostView;

/// Visual weight of a [`Button`].
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

const BUTTON_BASE: &str = "inline-flex items-center justify-center rounded-md px-4 py-2 \
                           text-sm font-medium transition-colors disabled:opacity-50 \
                           disabled:pointer-events-none";

#[component]
pub fn Button(
    #[props(default)] variant: Variant,
    #[props(default = false)] disabled: bool,
    #[props(default = "button".to_string())] r#type: String,
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    rsx! {
        button {
            class: "{BUTTON_BASE} {variant.classes()}",
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

const FIELD_BASE: &str = "w-full rounded-md border border-slate-300 px-3 py-2 text-sm \
                          focus:outline-none focus:ring-2 focus:ring-slate-900";

#[component]
pub fn TextField(
    label: String,
    value: String,
    #[props(default)] placeholder: String,
    #[props(default = "text".to_string())] r#type: String,
    /// Field-level validation message. Rendered with `role="alert"` so screen
    /// readers announce it — a validation error nobody hears is not a
    /// validation error.
    #[props(default)]
    error: Option<String>,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        label { class: "block space-y-1",
            span { class: "text-sm font-medium text-slate-700", "{label}" }
            input {
                class: "{FIELD_BASE}",
                r#type: "{r#type}",
                value: "{value}",
                placeholder: "{placeholder}",
                oninput: move |event| oninput.call(event),
            }
            if let Some(message) = error {
                span { class: "text-xs text-red-600", role: "alert", "{message}" }
            }
        }
    }
}

#[component]
pub fn TextArea(
    label: String,
    value: String,
    #[props(default = 6)] rows: u32,
    #[props(default)] placeholder: String,
    #[props(default)] error: Option<String>,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        label { class: "block space-y-1",
            span { class: "text-sm font-medium text-slate-700", "{label}" }
            textarea {
                class: "{FIELD_BASE}",
                rows: "{rows}",
                placeholder: "{placeholder}",
                value: "{value}",
                oninput: move |event| oninput.call(event),
            }
            if let Some(message) = error {
                span { class: "text-xs text-red-600", role: "alert", "{message}" }
            }
        }
    }
}

#[component]
pub fn Card(children: Element) -> Element {
    rsx! {
        div { class: "rounded-lg border border-slate-200 bg-white p-4 shadow-sm", {children} }
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

/// A single post in the list. Takes a [`PostView`] — the *same* type the API
/// serves and the folders produce — so there is no view-model to keep in sync.
#[component]
pub fn PostCard(post: PostView, #[props(default)] actions: Option<Element>) -> Element {
    // Formatted outside `rsx!`: the macro's interpolation parser does not
    // accept a call carrying a quoted argument.
    let iso = post.created_at.to_rfc3339();
    let human = post.created_at.format("%Y-%m-%d %H:%M UTC").to_string();
    rsx! {
        Card {
            article {
                h2 { class: "text-lg font-medium text-slate-900", "{post.title}" }
                p { class: "mt-1 whitespace-pre-wrap text-sm text-slate-600", "{post.content}" }
                footer { class: "mt-3 flex items-center justify-between",
                    time { class: "text-xs text-slate-400", datetime: "{iso}", "{human}" }
                    div { class: "flex gap-2", {actions} }
                }
            }
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
}
