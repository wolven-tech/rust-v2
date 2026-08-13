//! Form controls.

use dioxus::prelude::*;

const FIELD_BASE: &str = "w-full rounded-md border border-slate-300 px-3 py-2 text-sm \
                          focus:outline-none focus:ring-2 focus:ring-slate-900";

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
            span { class: "text-sm font-medium text-slate-700", "{label}" }
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
