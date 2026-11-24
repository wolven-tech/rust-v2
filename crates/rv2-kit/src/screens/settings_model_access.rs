//! Settings: Model Access. Which AI models people may use: the default chat and
//! image models, each provider's models switched on or off with their access, and
//! the workspace's own provider keys.
//!
//! Design choice: provider cards use the native disclosure pattern of a button with
//! `aria-expanded` over a region, not a `details` element, so a search match can
//! open a card from outside.
//! Design choice: the rolling model names beside Auto are decorative, `aria-hidden`,
//! and stop under reduced motion; one sentence describes them for screen readers.
//! Design choice: an API key is added through a password field; the kit stores
//! nothing and reports the key to the host through `on_add_key`.
//! Browser checks: default fallback when a default is switched off, search opens
//! matching providers, custom model with Enter, key masking, EU-first ordering,
//! reduced motion, phone width.

use dioxus::prelude::*;
use rv2_affect::switch::Switch;

use super::settings_profile::{
    SettingsCard, SettingsRow, SettingsShell, SettingsSkel, SettingsStatus, focus_later,
};
use crate::internal::next_id;

const MODELS_CSS: Asset = asset!("/assets/settings-model-access.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HostingRegion {
    #[default]
    Eu,
    Global,
    Custom,
}

impl HostingRegion {
    pub fn label(self) -> &'static str {
        match self {
            Self::Eu => "EU-hosted",
            Self::Global => "Global",
            Self::Custom => "Custom endpoint",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Eu => "Requests stay in EU data centres.",
            Self::Global => "Requests may be served from any region.",
            Self::Custom => "Your own endpoint; the provider sees what you send.",
        }
    }

    /// Order the provider list shows: EU-hosted, then global, then custom.
    pub fn rank(self) -> u8 {
        match self {
            Self::Eu => 0,
            Self::Global => 1,
            Self::Custom => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModelKind {
    #[default]
    Chat,
    Image,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccessLevel {
    #[default]
    Everyone,
    Engineering,
    Admins,
}

impl AccessLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Everyone => "everyone",
            Self::Engineering => "engineering",
            Self::Admins => "admins",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Everyone => "Everyone",
            Self::Engineering => "Engineering",
            Self::Admins => "Admins",
        }
    }

    pub fn from_str_value(value: &str) -> Self {
        match value {
            "engineering" => Self::Engineering,
            "admins" => Self::Admins,
            _ => Self::Everyone,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModelFilter {
    #[default]
    All,
    Chat,
    Image,
    Custom,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelEntry {
    pub id: String,
    pub display_name: String,
    pub model_kind: ModelKind,
    pub is_enabled: bool,
    pub access_level: AccessLevel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProviderEntry {
    pub id: String,
    pub display_name: String,
    pub hosting_region: HostingRegion,
    pub logo_url: Option<String>,
    pub model_list: Vec<ModelEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApiKey {
    pub provider_id: String,
    pub masked_value: String,
    pub added_at: String,
}

/// The whole model configuration, as `on_save` receives it.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelAccessState {
    pub providers: Vec<ProviderEntry>,
    pub default_chat: String,
    pub default_image: String,
}

/// The chat default after `removed` was switched off: Auto.
pub fn chat_fallback(default_chat: &str, removed: &str) -> String {
    if default_chat == removed {
        "auto".to_string()
    } else {
        default_chat.to_string()
    }
}

/// The image default after `removed` was switched off: the first image model still
/// on, else `none`.
pub fn image_fallback(providers: &[ProviderEntry], default_image: &str, removed: &str) -> String {
    if default_image != removed {
        return default_image.to_string();
    }
    providers
        .iter()
        .flat_map(|p| p.model_list.iter())
        .find(|m| m.model_kind == ModelKind::Image && m.is_enabled && m.id != removed)
        .map(|m| m.id.clone())
        .unwrap_or_else(|| "none".to_string())
}

/// Whether a provider or any of its models matches the search text.
pub fn provider_matches(provider: &ProviderEntry, query: &str) -> bool {
    let needle = query.trim().to_lowercase();
    needle.is_empty()
        || provider.display_name.to_lowercase().contains(&needle)
        || provider
            .model_list
            .iter()
            .any(|m| m.display_name.to_lowercase().contains(&needle))
}

/// Providers in display order: by region rank, then by name.
pub fn ordered(providers: &[ProviderEntry]) -> Vec<ProviderEntry> {
    let mut out = providers.to_vec();
    out.sort_by(|a, b| {
        a.hosting_region
            .rank()
            .cmp(&b.hosting_region.rank())
            .then_with(|| a.display_name.cmp(&b.display_name))
    });
    out
}

fn kind_matches(filter: ModelFilter, provider: &ProviderEntry) -> bool {
    match filter {
        ModelFilter::All => true,
        ModelFilter::Custom => provider.hosting_region == HostingRegion::Custom,
        ModelFilter::Chat => provider
            .model_list
            .iter()
            .any(|m| m.model_kind == ModelKind::Chat),
        ModelFilter::Image => provider
            .model_list
            .iter()
            .any(|m| m.model_kind == ModelKind::Image),
    }
}

/// Models the chat selector offers: enabled chat models, after Auto.
pub fn chat_choices(providers: &[ProviderEntry]) -> Vec<(String, String)> {
    providers
        .iter()
        .flat_map(|p| p.model_list.iter())
        .filter(|m| m.model_kind == ModelKind::Chat && m.is_enabled)
        .map(|m| (m.id.clone(), m.display_name.clone()))
        .collect()
}

/// Models the image selector offers: enabled image models.
pub fn image_choices(providers: &[ProviderEntry]) -> Vec<(String, String)> {
    providers
        .iter()
        .flat_map(|p| p.model_list.iter())
        .filter(|m| m.model_kind == ModelKind::Image && m.is_enabled)
        .map(|m| (m.id.clone(), m.display_name.clone()))
        .collect()
}

/// Model access settings page.
#[component]
pub fn SettingsModelAccess(
    provider_list: Vec<ProviderEntry>,
    default_chat: String,
    default_image: String,
    #[props(default)] api_keys: Vec<ApiKey>,
    #[props(default)] filter: ModelFilter,
    #[props(default)] query: String,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_save: Option<EventHandler<ModelAccessState>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] docs_href: Option<String>,
    #[props(default)] on_add_key: Option<EventHandler<(String, String)>>,
    #[props(default)] open_provider: Option<String>,
) -> Element {
    let uid = use_hook(|| next_id());
    let mut state = use_signal(|| ModelAccessState {
        providers: provider_list.clone(),
        default_chat: default_chat.clone(),
        default_image: default_image.clone(),
    });
    let mut kind_filter = use_signal(|| filter);
    let mut search = use_signal(|| query.clone());
    let mut toggled = use_signal({
        let seed = open_provider.clone();
        move || seed.iter().cloned().collect::<Vec<String>>()
    });
    let mut custom_open = use_signal(|| false);
    let mut key_open = use_signal(|| false);
    let mut custom_name = use_signal(String::new);
    let mut custom_endpoint = use_signal(String::new);
    let mut key_provider = use_signal(String::new);
    let mut key_value = use_signal(String::new);
    let mut status = use_signal(String::new);

    let mut apply = move |edit: &dyn Fn(&mut ModelAccessState)| {
        state.with_mut(|s| edit(s));
        if let Some(h) = &on_save {
            h.call(state());
        }
    };

    let snapshot = state();
    let query_now = search();
    let searching = !query_now.trim().is_empty();
    let visible: Vec<ProviderEntry> = ordered(&snapshot.providers)
        .into_iter()
        .filter(|p| kind_matches(kind_filter(), p))
        .filter(|p| provider_matches(p, &query_now))
        .collect();
    let match_count = visible.len();
    let chat_opts = chat_choices(&snapshot.providers);
    let image_opts = image_choices(&snapshot.providers);
    let auto_chosen = snapshot.default_chat == "auto";
    let filters = [
        (ModelFilter::All, "All"),
        (ModelFilter::Chat, "Chat"),
        (ModelFilter::Image, "Image"),
        (ModelFilter::Custom, "Custom"),
    ];

    if is_loading {
        return rsx! {
            SettingsShell { current: "Models", title: "Models", description: "Which AI models your people may use, and the keys that pay for them.", on_navigate, docs_href: docs_href.clone(),
                document::Link { rel: "stylesheet", href: MODELS_CSS }
                div { class: "kit-set__loading", "aria-busy": "true",
                    SettingsCard { heading: "Defaults".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Providers".to_string(),
                        for slot in 0..3 { span { key: "{slot}", class: "kit-set__skel", "data-kind": "block", aria_hidden: "true" } }
                    }
                    SettingsCard { heading: "API keys".to_string(), SettingsSkel { kind: "block" } }
                }
            }
        };
    }

    rsx! {
        SettingsShell { current: "Models", title: "Models", description: "Which AI models your people may use, and the keys that pay for them.", on_navigate, docs_href: docs_href.clone(),
            document::Link { rel: "stylesheet", href: MODELS_CSS }
            SettingsCard { heading: "Defaults".to_string(),
                SettingsRow { label: "Chat model".to_string(), note: "Used for new chats unless someone picks another.".to_string(),
                    select {
                        class: "kit-set__input",
                        "aria-label": "Default chat model",
                        onchange: move |e: FormEvent| {
                            let value = e.value();
                            apply(&|s| s.default_chat = value.clone());
                        },
                        option { value: "auto", selected: auto_chosen, "Auto" }
                        for (id, name) in chat_opts.clone() {
                            option { key: "{id}", value: "{id}", selected: snapshot.default_chat == id, "{name}" }
                        }
                    }
                    if auto_chosen {
                        p { class: "kit-mdl__auto", "data-testid": "auto-explainer",
                            "Auto picks the model for each message from the enabled chat models, and reasons longer on harder questions."
                        }
                        div { class: "kit-mdl__rolling", aria_hidden: "true",
                            span { class: "kit-mdl__rolling-track",
                                for name in ["Fast chat", "Deep reasoning", "Long context"] {
                                    span { key: "{name}", "{name}" }
                                }
                            }
                        }
                        span { class: "kit-sr", "Auto chooses among the enabled chat models for each message." }
                    }
                }
                SettingsRow { label: "Image model".to_string(), note: "Used when someone asks for a picture.".to_string(),
                    select {
                        class: "kit-set__input",
                        "aria-label": "Default image model",
                        onchange: move |e: FormEvent| {
                            let value = e.value();
                            apply(&|s| s.default_image = value.clone());
                        },
                        if image_opts.is_empty() {
                            option { value: "none", "None" }
                        }
                        for (id, name) in image_opts.clone() {
                            option { key: "{id}", value: "{id}", selected: snapshot.default_image == id, "{name}" }
                        }
                    }
                }
            }

            SettingsCard { heading: "Providers".to_string(),
                div { class: "kit-mdl__toolbar",
                    div { class: "kit-mdl__tabs", role: "tablist", aria_label: "Filter models",
                        onkeydown: move |e: KeyboardEvent| {
                            let order = [ModelFilter::All, ModelFilter::Chat, ModelFilter::Image, ModelFilter::Custom];
                            let at = order.iter().position(|f| *f == kind_filter()).unwrap_or(0) as isize;
                            let step = match e.key() { Key::ArrowRight => 1, Key::ArrowLeft => -1, _ => 0 };
                            if step != 0 {
                                let next = (at + step).rem_euclid(order.len() as isize) as usize;
                                kind_filter.set(order[next]);
                                document::eval(&format!("requestAnimationFrame(() => document.getElementById('{uid}-filter-{next}')?.focus());"));
                            }
                        },
                        for (index, (value, label)) in filters.iter().copied().enumerate() {
                            button {
                                key: "{label}",
                                id: "{uid}-filter-{index}",
                                r#type: "button",
                                role: "tab",
                                class: "kit-mdl__tab",
                                aria_selected: (kind_filter() == value).to_string(),
                                tabindex: if kind_filter() == value { "0" } else { "-1" },
                                onclick: move |_| kind_filter.set(value),
                                "{label}"
                            }
                        }
                    }
                    input {
                        class: "kit-set__input kit-mdl__search",
                        r#type: "search",
                        placeholder: "Search providers and models",
                        "aria-label": "Search providers and models",
                        value: "{query_now}",
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape {
                                search.set(String::new());
                            }
                        },
                        oninput: move |e: FormEvent| search.set(e.value()),
                    }
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        aria_expanded: custom_open().to_string(),
                        onclick: move |_| {
                            custom_open.set(!custom_open());
                            if custom_open() {
                                focus_later(&format!("{uid}-custom-name"));
                            }
                        },
                        "Add custom model"
                    }
                }
                if searching {
                    p { class: "kit-set__info", role: "status", aria_live: "polite", "{match_count} matching providers" }
                }
                if custom_open() {
                    form { class: "kit-mdl__form", aria_label: "Add custom model",
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape {
                                custom_open.set(false);
                                custom_name.set(String::new());
                                custom_endpoint.set(String::new());
                            }
                        },
                        onsubmit: move |e: FormEvent| {
                            e.prevent_default();
                            let name = custom_name().trim().to_string();
                            if name.is_empty() {
                                status.set("Give the model a name.".to_string());
                                return;
                            }
                            let endpoint = custom_endpoint().trim().to_string();
                            let model = ModelEntry {
                                id: format!("custom-{}", name.to_lowercase().replace(' ', "-")),
                                display_name: name.clone(),
                                model_kind: ModelKind::Chat,
                                is_enabled: true,
                                access_level: AccessLevel::Everyone,
                            };
                            let provider_name = if endpoint.is_empty() { "Custom endpoint".to_string() } else { endpoint.clone() };
                            apply(&|s| {
                                if let Some(p) = s.providers.iter_mut().find(|p| p.id == "custom") {
                                    p.model_list.push(model.clone());
                                } else {
                                    s.providers.push(ProviderEntry {
                                        id: "custom".to_string(),
                                        display_name: provider_name.clone(),
                                        hosting_region: HostingRegion::Custom,
                                        logo_url: None,
                                        model_list: vec![model.clone()],
                                    });
                                }
                            });
                            status.set(format!("Added custom model {name}."));
                            custom_name.set(String::new());
                            custom_endpoint.set(String::new());
                            custom_open.set(false);
                        },
                        label { r#for: "{uid}-custom-name", "Model name" }
                        input {
                            id: "{uid}-custom-name", class: "kit-set__input", r#type: "text",
                            value: "{custom_name()}", oninput: move |e: FormEvent| custom_name.set(e.value()),
                        }
                        label { r#for: "{uid}-custom-endpoint", "Endpoint" }
                        input {
                            id: "{uid}-custom-endpoint", class: "kit-set__input", r#type: "url",
                            value: "{custom_endpoint()}", oninput: move |e: FormEvent| custom_endpoint.set(e.value()),
                        }
                        p { class: "kit-set__info", "Press Enter to add. Escape cancels." }
                    }
                }

                if visible.is_empty() {
                    div { class: "kit-mdl__empty",
                        if is_empty || snapshot.providers.is_empty() {
                            p { class: "kit-set__info", "No models are configured. The defaults stay on Auto until you add one." }
                            button {
                                r#type: "button",
                                class: "kit-set__btn",
                                onclick: move |_| {
                                    custom_open.set(true);
                                    focus_later(&format!("{uid}-custom-name"));
                                },
                                "Add custom model"
                            }
                        } else {
                            p { class: "kit-set__info", "No provider matches this search." }
                        }
                    }
                }

                ul { class: "kit-mdl__providers",
                    for provider in visible.clone() {
                        {
                            let open = searching || toggled().contains(&provider.id);
                            let enabled_count = provider.model_list.iter().filter(|m| m.is_enabled).count();
                            let panel_id = format!("{uid}-panel-{}", provider.id);
                            let toggle_id = provider.id.clone();
                            let region_note = provider.hosting_region.description();
                            let provider_for_rows = provider.clone();
                            let rows: Vec<ModelEntry> = provider.model_list.iter()
                                .filter(|m| match kind_filter() {
                                    ModelFilter::Chat => m.model_kind == ModelKind::Chat,
                                    ModelFilter::Image => m.model_kind == ModelKind::Image,
                                    _ => true,
                                })
                                .cloned()
                                .collect();
                            rsx! {
                                li { key: "{provider.id}", class: "kit-mdl__provider", "data-open": open.then_some("true"),
                                    button {
                                        r#type: "button",
                                        class: "kit-mdl__provider-head",
                                        aria_expanded: open.to_string(),
                                        aria_controls: "{panel_id}",
                                        onclick: move |_| {
                                            let id = toggle_id.clone();
                                            toggled.with_mut(|list| {
                                                if let Some(pos) = list.iter().position(|x| *x == id) {
                                                    list.remove(pos);
                                                } else {
                                                    list.push(id);
                                                }
                                            });
                                        },
                                        span { class: "kit-mdl__logo", aria_hidden: "true", "{provider.display_name.chars().next().unwrap_or('?')}" }
                                        span { class: "kit-mdl__provider-name", "{provider.display_name}" }
                                        span { class: "kit-mdl__region", "{provider.hosting_region.label()}" }
                                        span { class: "kit-set__info", "{enabled_count} enabled" }
                                    }
                                    if open {
                                        div { id: "{panel_id}", class: "kit-mdl__models",
                                            p { class: "kit-set__info", "{region_note}" }
                                            for model in rows.clone() {
                                                {
                                                    let model_id = model.id.clone();
                                                    let model_name = model.display_name.clone();
                                                    let access_label = format!("Access for {model_name}");
                                                    let hosting = provider_for_rows.hosting_region.label();
                                                    let access = model.access_level;
                                                    let kind_word = match model.model_kind { ModelKind::Chat => "Chat", ModelKind::Image => "Image" };
                                                    rsx! {
                                                        div { key: "{model.id}", class: "kit-mdl__model", "data-enabled": model.is_enabled.then_some("true"),
                                                            div { class: "kit-mdl__model-text",
                                                                Switch {
                                                                    label: rsx! { "{model_name}" },
                                                                    description: rsx! { "{hosting} · {kind_word}" },
                                                                    placement: rv2_affect::switch::SwitchPlacement::End,
                                                                    checked: model.is_enabled,
                                                                    onchange: {
                                                                        let model_id = model_id.clone();
                                                                        move |e: FormEvent| {
                                                                            let on = e.checked();
                                                                            let id = model_id.clone();
                                                                            apply(&|s| {
                                                                                let removed = (!on).then(|| id.clone());
                                                                                for p in s.providers.iter_mut() {
                                                                                    for m in p.model_list.iter_mut() {
                                                                                        if m.id == id {
                                                                                            m.is_enabled = on;
                                                                                        }
                                                                                    }
                                                                                }
                                                                                if let Some(removed) = removed {
                                                                                    s.default_chat = chat_fallback(&s.default_chat, &removed);
                                                                                    s.default_image = image_fallback(&s.providers, &s.default_image, &removed);
                                                                                }
                                                                            });
                                                                        }
                                                                    },
                                                                }
                                                            }
                                                            select {
                                                                class: "kit-set__input kit-mdl__access",
                                                                "aria-label": "{access_label}",
                                                                onchange: {
                                                                    let model_id = model_id.clone();
                                                                    move |e: FormEvent| {
                                                                        let level = AccessLevel::from_str_value(&e.value());
                                                                        let id = model_id.clone();
                                                                        apply(&|s| {
                                                                            for p in s.providers.iter_mut() {
                                                                                for m in p.model_list.iter_mut() {
                                                                                    if m.id == id {
                                                                                        m.access_level = level;
                                                                                    }
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                option { value: "everyone", selected: access == AccessLevel::Everyone, "Everyone" }
                                                                option { value: "engineering", selected: access == AccessLevel::Engineering, "Engineering" }
                                                                option { value: "admins", selected: access == AccessLevel::Admins, "Admins" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            SettingsCard { heading: "API keys".to_string(),
                note: "A provider key you add here bills that provider directly, not this workspace.".to_string(),
                if api_keys.is_empty() {
                    p { class: "kit-set__info kit-mdl__no-keys", "No API keys yet. Models use the platform's own keys until you add one." }
                } else {
                    ul { class: "kit-mdl__keys",
                        for key in api_keys.clone() {
                            li { key: "{key.provider_id}-{key.masked_value}", class: "kit-mdl__key",
                                span { class: "kit-mdl__key-provider", "{key.provider_id}" }
                                span { class: "kit-mdl__key-value", "{key.masked_value}" }
                                span { class: "kit-set__info", "Added {key.added_at}" }
                            }
                        }
                    }
                }
                div { class: "kit-mdl__key-actions",
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        aria_expanded: key_open().to_string(),
                        onclick: move |_| {
                            key_open.set(!key_open());
                            if key_open() {
                                focus_later(&format!("{uid}-key-provider"));
                            }
                        },
                        "Add API key"
                    }
                }
                if key_open() {
                    form { class: "kit-mdl__form", aria_label: "Add API key",
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape {
                                key_open.set(false);
                                key_value.set(String::new());
                            }
                        },
                        onsubmit: move |e: FormEvent| {
                            e.prevent_default();
                            let value = key_value();
                            if value.trim().is_empty() {
                                status.set("Enter the key before adding it.".to_string());
                                return;
                            }
                            let provider = if key_provider().is_empty() { "custom".to_string() } else { key_provider() };
                            if let Some(h) = &on_add_key {
                                h.call((provider, value));
                            }
                            key_value.set(String::new());
                            key_open.set(false);
                            status.set("API key added.".to_string());
                        },
                        label { r#for: "{uid}-key-provider", "Provider" }
                        select {
                            id: "{uid}-key-provider", class: "kit-set__input",
                            onchange: move |e: FormEvent| key_provider.set(e.value()),
                            for provider in snapshot.providers.clone() {
                                option { key: "{provider.id}", value: "{provider.id}", "{provider.display_name}" }
                            }
                        }
                        label { r#for: "{uid}-key-value", "API key" }
                        input {
                            id: "{uid}-key-value", class: "kit-set__input", r#type: "password", autocomplete: "off",
                            value: "{key_value()}", oninput: move |e: FormEvent| key_value.set(e.value()),
                        }
                        p { class: "kit-set__info", "Press Enter to add. Escape cancels." }
                    }
                }
            }
            SettingsStatus { message: status() }
        }
    }
}
