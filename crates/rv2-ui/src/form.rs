//! Form controls.

use dioxus::prelude::*;

const FIELD_BASE: &str = "w-full rounded-md border border-rule px-3 py-2 text-sm \
                          focus:outline-none focus:ring-2 focus:ring-accent";

#[component]
pub fn TextField(
    label: String,
    value: String,
    #[props(default)] placeholder: String,
    #[props(default = "text".to_string())] r#type: String,
    /// The `name` attribute.
    ///
    /// Optional, but omitting it is half of what made the login screen a form
    /// that could not submit anything: a native form POST serializes fields
    /// **by name**, and a nameless input contributes nothing to the body at
    /// all. Password managers key on it too — without one they cannot offer to
    /// save or fill a credential.
    #[props(default)]
    name: Option<String>,
    /// The `autocomplete` token — `email`, `current-password`, and so on.
    ///
    /// A fixed vocabulary, not free text. Browsers need it to fill credentials
    /// into the right box.
    #[props(default)]
    autocomplete: Option<String>,
    #[props(default = false)] required: bool,
    /// Field-level validation message. Rendered with `role="alert"` so screen
    /// readers announce it — a validation error nobody hears is not a
    /// validation error.
    #[props(default)]
    error: Option<String>,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        label { class: "block space-y-1",
            span { class: "text-sm font-medium text-ink", "{label}" }
            input {
                class: "{FIELD_BASE}",
                r#type: "{r#type}",
                value: "{value}",
                placeholder: "{placeholder}",
                name: name.unwrap_or_default(),
                autocomplete: autocomplete.unwrap_or_default(),
                required,
                oninput: move |event| oninput.call(event),
            }
            if let Some(message) = error {
                span { class: "text-xs text-fault-ink", role: "alert", "{message}" }
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
            span { class: "text-sm font-medium text-ink", "{label}" }
            textarea {
                class: "{FIELD_BASE}",
                rows: "{rows}",
                placeholder: "{placeholder}",
                value: "{value}",
                oninput: move |event| oninput.call(event),
            }
            if let Some(message) = error {
                span { class: "text-xs text-fault-ink", role: "alert", "{message}" }
            }
        }
    }
}

/// A labelled checkbox, optionally carrying a count of what it would match.
///
/// The count is the reason this is a component rather than a bare `input`. A
/// filter rail that shows a facet with no count forces the reader to click it
/// to discover it matches nothing, and a rail whose zero-count options vanish
/// makes the list flicker as selections change. Showing the number, and dimming
/// rather than hiding a zero, keeps the set of options stable while still
/// saying which ones are worth choosing.
#[component]
pub fn Checkbox(
    label: String,
    checked: bool,
    /// How many records this option would match right now. Rendered to the
    /// right, dimmed when nought.
    #[props(default)]
    count: Option<usize>,
    /// Where the value came from, when it was derived rather than recorded —
    /// "headcount band, not revenue". Rendered as a title so the reader can
    /// disagree with a judgement the data did not actually make.
    #[props(default)]
    basis: Option<String>,
    #[props(default = false)] disabled: bool,
    onchange: EventHandler<bool>,
) -> Element {
    let empty = count == Some(0);
    let tone = if empty { "text-ink-faint" } else { "text-ink" };
    rsx! {
        label {
            class: "flex items-center gap-2 py-1 text-sm cursor-pointer {tone}",
            title: basis.clone().unwrap_or_default(),
            input {
                r#type: "checkbox",
                class: "h-4 w-4 rounded border-rule text-ink focus:ring-2 focus:ring-accent",
                checked,
                disabled,
                onchange: move |event| onchange.call(event.checked()),
            }
            span { class: "flex-1", "{label}" }
            if let Some(n) = count {
                span { class: "tabular-nums text-xs text-ink-muted", "{n}" }
            }
            if basis.is_some() {
                span { class: "text-xs text-ink-faint", aria_hidden: "true", "·" }
            }
        }
    }
}

/// A labelled select.
///
/// `options` is `(value, label)` because the value is what goes in a URL and
/// the label is what a person reads, and the two are never the same string once
/// the copy is written in plain English.
#[component]
pub fn Select(
    label: String,
    value: String,
    options: Vec<(String, String)>,
    #[props(default)] name: Option<String>,
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        label { class: "block space-y-1",
            span { class: "text-sm font-medium text-ink", "{label}" }
            select {
                class: "{FIELD_BASE} bg-surface",
                name: name.unwrap_or_default(),
                onchange: move |event| onchange.call(event.value()),
                for (key , text) in options {
                    option { value: "{key}", selected: key == value, "{text}" }
                }
            }
        }
    }
}

/// A group of controls under a legend, with the count of how many are on.
///
/// `native` fieldset and legend rather than a styled div: a screen reader
/// announces the legend with every control inside it, which is the only thing
/// that tells someone tabbing through thirty checkboxes which group they are
/// in.
#[component]
pub fn FieldSet(
    legend: String,
    /// A sentence under the legend saying what the group does, for a group
    /// whose meaning is not obvious from its title alone.
    #[props(default)]
    note: Option<String>,
    /// How many controls in this group are switched on.
    #[props(default)]
    active: usize,
    children: Element,
) -> Element {
    rsx! {
        fieldset { class: "border-t border-rule-soft pt-3",
            legend { class: "flex items-baseline gap-2 pr-2 text-xs font-semibold uppercase tracking-wide text-ink-muted",
                "{legend}"
                if active > 0 {
                    span { class: "font-normal normal-case tracking-normal text-ink-muted",
                        "({active} on)"
                    }
                }
            }
            if let Some(text) = note {
                p { class: "pb-1 text-xs text-ink-muted", "{text}" }
            }
            {children}
        }
    }
}
