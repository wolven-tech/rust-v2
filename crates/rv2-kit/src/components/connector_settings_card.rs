//! Connector settings card: enable a linked service, store its key, checked site address and time
//! zone. Edits wait in a draft until saved or discarded.
//!
//! Design choice: `saving` is driven by the host (Save shows busy and ignores repeat presses while
//! it is true), rather than awaiting a future from `on_save`. The key is masked by default; Reveal
//! toggles it. Generate without a host callback makes a demo key.
//!
//! Browser checks: the save row rises from the card's lower edge; Space toggles the switch; Escape
//! closes the time-zone list and keeps the chosen value.

use dioxus::prelude::*;
use rv2_affect::button::{Button, ButtonVariant};
use rv2_affect::select::{Select, SelectOption};
use rv2_affect::switch::Switch;

use crate::internal::{KIT_CSS, class_list, next_id};

const CSS: Asset = asset!("/assets/connector-settings-card.css");

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConnectorValues {
    pub is_enabled: bool,
    pub site_url: String,
    pub time_zone: String,
    pub api_key: String,
}

/// `https://shop.example` and `http://localhost:8080` are valid; anything without a scheme of
/// `http` or `https` and a host is not.
pub fn validate_site_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    let (scheme, rest) = trimmed
        .split_once("://")
        .ok_or_else(|| "Add http:// or https:// before the address".to_string())?;
    if scheme != "http" && scheme != "https" {
        return Err("Use an http or https address".to_string());
    }
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if host.is_empty() || host.chars().any(char::is_whitespace) {
        return Err("Enter a host name after the scheme".to_string());
    }
    Ok(())
}

pub fn is_dirty(saved: &ConnectorValues, draft: &ConnectorValues) -> bool {
    saved != draft
}

/// Bullets with the last four characters visible.
pub fn mask_key(key: &str) -> String {
    let count = key.chars().count();
    if count <= 4 {
        return "•".repeat(count);
    }
    let tail: String = key.chars().skip(count - 4).collect();
    format!("{}{tail}", "•".repeat(12))
}

pub fn demo_key(seed: usize) -> String {
    format!("rv2_demo_{seed:016x}")
}

#[component]
pub fn ConnectorSettingsCard(
    service_name: String,
    #[props(default)] service_mark: Option<Element>,
    #[props(default)] item_count: Option<usize>,
    #[props(default, into)] subtitle_text: String,
    #[props(default)] initial_values: ConnectorValues,
    #[props(default, into)] site_url_hint: String,
    #[props(default)] time_zone_options: Vec<String>,
    #[props(default)] on_generate_key: Option<Callback<(), String>>,
    #[props(default)] on_save: EventHandler<ConnectorValues>,
    #[props(default)] saving: bool,
    #[props(default)] on_enabled_change: EventHandler<bool>,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-csc-{}", next_id()));
    let mut saved = use_signal(|| initial_values.clone());
    let mut draft = use_signal(|| initial_values.clone());
    let mut revealed = use_signal(|| false);
    let mut generated = use_signal(|| false);
    let mut url_error = use_signal(|| None::<String>);

    let current = draft();
    let dirty = is_dirty(&saved(), &current);
    let enabled = current.is_enabled;
    let paused = if enabled {
        subtitle_text.clone()
    } else {
        "Paused".to_string()
    };
    let key_shown = if revealed() {
        current.api_key.clone()
    } else {
        mask_key(&current.api_key)
    };
    let key_type = if revealed() { "text" } else { "password" };
    let zone_options: Vec<SelectOption> = time_zone_options
        .iter()
        .map(|zone| SelectOption {
            value: zone.clone(),
            label: zone.clone(),
            description: None,
            icon: None,
            disabled: false,
        })
        .collect();
    let count_text = item_count.map(|n| n.to_string());
    let field_id = format!("{id}-url");
    let hint_id = format!("{id}-url-hint");
    let url_message = url_error();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: class_list("kit-csc", &extra_class), "data-enabled": enabled.then_some("true"),
            header { class: "kit-csc__head",
                span { class: "kit-csc__mark", "aria-hidden": "true",
                    if let Some(mark) = service_mark { {mark} }
                }
                div { class: "kit-csc__titles",
                    h3 { class: "kit-csc__name",
                        "{service_name}"
                        if let Some(count) = count_text {
                            span { class: "kit-csc__count", "{count}" }
                        }
                    }
                    p { class: "kit-csc__subtitle", "{paused}" }
                }
                Switch {
                    label: rsx! { "Enabled" },
                    checked: enabled,
                    onchange: move |_| {
                        let next = !enabled;
                        draft.with_mut(|d| d.is_enabled = next);
                        on_enabled_change.call(next);
                    },
                }
            }
            if enabled {
                div { class: "kit-csc__body",
                    div { class: "kit-csc__field",
                        label { class: "kit-csc__label", r#for: "{id}-key", "Access key" }
                        div { class: "kit-csc__row",
                            input {
                                id: "{id}-key",
                                class: "kit-csc__input",
                                r#type: key_type,
                                value: "{key_shown}",
                                readonly: true,
                                autocomplete: "off",
                            }
                            button {
                                r#type: "button",
                                class: "kit-csc__action",
                                "aria-pressed": if revealed() { "true" } else { "false" },
                                onclick: move |_| revealed.set(!revealed()),
                                if revealed() { "Hide" } else { "Reveal" }
                            }
                            button {
                                r#type: "button",
                                class: "kit-csc__action",
                                onclick: move |_| {
                                    let key = match on_generate_key {
                                        Some(generate) => generate.call(()),
                                        None => demo_key(next_id()),
                                    };
                                    draft.with_mut(|d| d.api_key = key.clone());
                                    generated.set(true);
                                },
                                "Generate new"
                            }
                        }
                        p { class: "kit-csc__notice", role: "status", "aria-live": "polite",
                            if generated() { "Copy this key now. It will not be shown again." }
                        }
                    }
                    div { class: "kit-csc__field",
                        label { class: "kit-csc__label", r#for: "{field_id}", "Site address" }
                        input {
                            id: "{field_id}",
                            class: "kit-csc__input",
                            r#type: "url",
                            value: "{current.site_url}",
                            "aria-invalid": if url_message.is_some() { "true" } else { "false" },
                            "aria-describedby": "{hint_id}",
                            oninput: move |event| {
                                let text = event.value();
                                url_error.set(None);
                                draft.with_mut(|d| d.site_url = text);
                            },
                        }
                        if let Some(message) = url_message.clone() {
                            p { class: "kit-csc__error", role: "alert", "{message}" }
                        }
                        p { id: "{hint_id}", class: "kit-csc__hint", "{site_url_hint}" }
                    }
                    div { class: "kit-csc__field",
                        span { class: "kit-csc__label", "Time zone" }
                        Select {
                            options: zone_options,
                            value: current.time_zone.clone(),
                            name: "time-zone",
                            onchange: move |option: SelectOption| draft.with_mut(|d| d.time_zone = option.value),
                        }
                    }
                }
            }
            if dirty {
                footer { class: "kit-csc__save",
                    span { class: "kit-csc__dirty", "Unsaved changes" }
                    Button {
                        variant: ButtonVariant::Secondary,
                        label: "Discard",
                        onclick: move |_| {
                            draft.set(saved());
                            url_error.set(None);
                        },
                    }
                    Button {
                        label: if saving { "Saving…".to_string() } else { "Save".to_string() },
                        loading: saving,
                        onclick: move |_| {
                            let values = draft();
                            if let Err(message) = validate_site_url(&values.site_url) {
                                url_error.set(Some(message));
                                return;
                            }
                            saved.set(values.clone());
                            on_save.call(values);
                        },
                    }
                }
            }
        }
    }
}
