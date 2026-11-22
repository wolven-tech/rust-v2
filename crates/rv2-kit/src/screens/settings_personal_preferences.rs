//! Settings: Personal Preferences. A person's own defaults on top of what the
//! workspace allows: default models, language, theme, chat tools, custom
//! instructions, and the memories the assistant has kept.
//!
//! Design choice: `memory_on` is the memory tile's state; toggling the memory tile
//! sets it, so the memory card and the tile never disagree.
//! Design choice: the instruction text areas save on blur, as the spec says. The
//! limit is `maxlength`, so typing past 1,500 characters is blocked by the browser.
//! Design choice: the theme radio applies at once through `on_save`; the kit does
//! not switch the page theme itself.
//! Browser checks: models outside the allowed list cannot be chosen, theme saves at
//! once, chat tile checked state, paused instructions disable both areas, Forget and
//! Clear all with Escape, memory-off note, phone width.

use dioxus::prelude::*;

use super::settings_model_access::ModelKind;
use super::settings_profile::{
    SettingsCard, SettingsRow, SettingsShell, SettingsSkel, SettingsStatus,
};
use crate::internal::next_id;
use rv2_affect::dialog::AlertDialog;

const PREFERENCES_CSS: Asset = asset!("/assets/settings-personal-preferences.css");

pub const INSTRUCTION_LIMIT: usize = 1500;
const LANGUAGES: [&str; 5] = ["English", "Deutsch", "Español", "Français", "Română"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }

    pub const ALL: [Theme; 3] = [Self::System, Self::Light, Self::Dark];
}

#[derive(Clone, Debug, PartialEq)]
pub struct AllowedModel {
    pub id: String,
    pub name: String,
    pub kind: ModelKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatTool {
    pub id: String,
    pub label: String,
    pub description: String,
    pub is_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instructions {
    pub is_enabled: bool,
    pub about_text: String,
    pub response_style: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Memory {
    pub id: String,
    pub text: String,
    pub saved_at: String,
}

/// Every preference, as `on_save` receives it.
#[derive(Clone, Debug, PartialEq)]
pub struct Preferences {
    pub default_model: String,
    pub default_image_model: String,
    pub language: String,
    pub theme: Theme,
    pub app_actions: bool,
    pub chat_tools: Vec<ChatTool>,
    pub instructions: Instructions,
    pub memories: Vec<Memory>,
    pub memory_on: bool,
}

/// `text` clipped to `limit` characters, the way the field blocks typing past it.
pub fn clip_instruction(text: &str) -> String {
    text.chars().take(INSTRUCTION_LIMIT).collect()
}

/// Whether a model may be chosen: the workspace allows it and its kind fits.
pub fn choosable(models: &[AllowedModel], id: &str, kind: ModelKind) -> bool {
    models.iter().any(|m| m.id == id && m.kind == kind)
}

/// Personal preferences page.
#[component]
pub fn SettingsPersonalPreferences(
    default_model: String,
    default_image_model: String,
    models: Vec<AllowedModel>,
    #[props(default = "English".to_string(), into)] language: String,
    #[props(default)] theme: Theme,
    #[props(default)] app_actions: bool,
    chat_tools: Vec<ChatTool>,
    instructions: Instructions,
    #[props(default)] memories: Vec<Memory>,
    #[props(default)] memory_on: bool,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_save: Option<EventHandler<Preferences>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] docs_href: Option<String>,
) -> Element {
    let uid = use_hook(|| next_id());
    let allowed = use_signal(|| models.clone());
    let mut prefs = use_signal(|| Preferences {
        default_model: default_model.clone(),
        default_image_model: default_image_model.clone(),
        language: language.clone(),
        theme,
        app_actions,
        chat_tools: chat_tools.clone(),
        instructions: instructions.clone(),
        memories: memories.clone(),
        memory_on,
    });
    let mut status = use_signal(String::new);
    let mut confirm_clear = use_signal(|| false);
    let mut last_saved = use_signal(|| prefs());

    let mut apply = move |edit: &dyn Fn(&mut Preferences)| {
        prefs.with_mut(|p| edit(p));
        if let Some(h) = &on_save {
            h.call(prefs());
        }
        status.set("Preferences saved.".to_string());
    };
    let mut commit_text = move || {
        let next = prefs();
        if next != last_saved() {
            last_saved.set(next.clone());
            if let Some(h) = &on_save {
                h.call(next);
            }
            status.set("Preferences saved.".to_string());
        }
    };

    let snapshot = prefs();
    let chat_models: Vec<AllowedModel> = models
        .iter()
        .filter(|m| m.kind == ModelKind::Chat)
        .cloned()
        .collect();
    let image_models: Vec<AllowedModel> = models
        .iter()
        .filter(|m| m.kind == ModelKind::Image)
        .cloned()
        .collect();
    let about_used = snapshot.instructions.about_text.chars().count();
    let style_used = snapshot.instructions.response_style.chars().count();
    let paused = !snapshot.instructions.is_enabled;
    let memory_note = !snapshot.memory_on;

    if is_loading {
        return rsx! {
            SettingsShell { current: "Preferences", title: "Preferences", description: "Your defaults, within what the workspace allows.", on_navigate, docs_href: docs_href.clone(),
                document::Link { rel: "stylesheet", href: PREFERENCES_CSS }
                div { class: "kit-set__loading", "aria-busy": "true",
                    SettingsCard { heading: "Models".to_string(), SettingsSkel { kind: "field" } }
                    SettingsCard { heading: "Language and appearance".to_string(), SettingsSkel { kind: "field" } }
                    SettingsCard { heading: "Chat tools".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Custom instructions".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Memory".to_string(), SettingsSkel { kind: "block" } }
                }
            }
        };
    }

    rsx! {
        SettingsShell { current: "Preferences", title: "Preferences", description: "Your defaults, within what the workspace allows.", on_navigate, docs_href: docs_href.clone(),
            document::Link { rel: "stylesheet", href: PREFERENCES_CSS }
            SettingsCard { heading: "Models".to_string(),
                SettingsRow { label: "Default chat model".to_string(), note: "New chats start with this model.".to_string(),
                    select {
                        class: "kit-set__input",
                        "aria-label": "Default chat model",
                        onchange: move |e: FormEvent| {
                            let value = e.value();
                            if choosable(&allowed(), &value, ModelKind::Chat) || value == "auto" {
                                apply(&|p| p.default_model = value.clone());
                            }
                        },
                        option { value: "auto", selected: snapshot.default_model == "auto", "Auto" }
                        for model in chat_models.clone() {
                            option { key: "{model.id}", value: "{model.id}", selected: snapshot.default_model == model.id, "{model.name}" }
                        }
                    }
                    if snapshot.default_model != "auto" {
                        button {
                            r#type: "button",
                            class: "kit-set__btn kit-set__btn--quiet",
                            onclick: move |_| apply(&|p| p.default_model = "auto".to_string()),
                            "Switch to Auto"
                        }
                    }
                }
                SettingsRow { label: "Default image model".to_string(), note: "Used for picture requests.".to_string(),
                    select {
                        class: "kit-set__input",
                        "aria-label": "Default image model",
                        onchange: move |e: FormEvent| {
                            let value = e.value();
                            if choosable(&allowed(), &value, ModelKind::Image) {
                                apply(&|p| p.default_image_model = value.clone());
                            }
                        },
                        for model in image_models.clone() {
                            option { key: "{model.id}", value: "{model.id}", selected: snapshot.default_image_model == model.id, "{model.name}" }
                        }
                    }
                }
            }

            SettingsCard { heading: "Language and appearance".to_string(),
                SettingsRow { label: "Language".to_string(), note: "Menus and answers.".to_string(),
                    select {
                        class: "kit-set__input",
                        "aria-label": "Language",
                        onchange: move |e: FormEvent| {
                            let value = e.value();
                            apply(&|p| p.language = value.clone());
                        },
                        for lang in LANGUAGES {
                            option { key: "{lang}", value: "{lang}", selected: snapshot.language == lang, "{lang}" }
                        }
                    }
                }
                SettingsRow { label: "Appearance".to_string(), note: "Applies at once.".to_string(),
                    div { class: "kit-set__radios", role: "radiogroup", aria_label: "Appearance",
                        for choice in Theme::ALL {
                            label { key: "{choice.as_str()}", class: "kit-set__radio",
                                input {
                                    r#type: "radio",
                                    name: "theme-{uid}",
                                    value: "{choice.as_str()}",
                                    checked: snapshot.theme == choice,
                                    onchange: move |_| apply(&|p| p.theme = choice),
                                }
                                "{choice.label()}"
                            }
                        }
                    }
                }
                SettingsRow { label: "App actions (beta)".to_string(), note: "Find app actions without explicit tags.".to_string(),
                    label { class: "kit-pref__toggle",
                        input {
                            r#type: "checkbox",
                            role: "switch",
                            checked: snapshot.app_actions,
                            onchange: move |e: FormEvent| {
                                let on = e.checked();
                                apply(&|p| p.app_actions = on);
                            },
                        }
                        span { if snapshot.app_actions { "On" } else { "Off" } }
                    }
                }
            }

            SettingsCard { heading: "Chat tools".to_string(),
                note: "Turn on the tools a chat may use.".to_string(),
                div { class: "kit-pref__tiles",
                    for tool in snapshot.chat_tools.clone() {
                        {
                            let tool_id = tool.id.clone();
                            rsx! {
                                label { key: "{tool.id}", class: "kit-pref__tile", "data-enabled": tool.is_enabled.then_some("true"),
                                    input {
                                        r#type: "checkbox",
                                        checked: tool.is_enabled,
                                        onchange: move |e: FormEvent| {
                                            let on = e.checked();
                                            let id = tool_id.clone();
                                            apply(&|p| {
                                                if let Some(t) = p.chat_tools.iter_mut().find(|t| t.id == id) {
                                                    t.is_enabled = on;
                                                }
                                                if id == "memory" {
                                                    p.memory_on = on;
                                                }
                                            });
                                        },
                                    }
                                    span { class: "kit-pref__tile-name", "{tool.label}" }
                                    span { class: "kit-set__info", "{tool.description}" }
                                }
                            }
                        }
                    }
                }
            }

            SettingsCard { heading: "Custom instructions".to_string(),
                note: "Sent with every message.".to_string(),
                SettingsRow { label: "Use custom instructions".to_string(), note: if paused { "Paused. Your text is kept.".to_string() } else { "On.".to_string() },
                    label { class: "kit-pref__toggle",
                        input {
                            r#type: "checkbox",
                            role: "switch",
                            checked: snapshot.instructions.is_enabled,
                            onchange: move |e: FormEvent| {
                                let on = e.checked();
                                apply(&|p| p.instructions.is_enabled = on);
                            },
                        }
                        span { if paused { "Paused" } else { "Active" } }
                    }
                }
                div { class: "kit-pref__instructions", "data-paused": paused.then_some("true"),
                    div { class: "kit-set__field",
                        label { r#for: "{uid}-about", "About you" }
                        textarea {
                            id: "{uid}-about",
                            class: "kit-set__input kit-set__area",
                            disabled: paused,
                            maxlength: "{INSTRUCTION_LIMIT}",
                            value: "{snapshot.instructions.about_text}",
                            oninput: move |e: FormEvent| {
                                let value = clip_instruction(&e.value());
                                prefs.with_mut(|p| p.instructions.about_text = value);
                            },
                            onblur: move |_| commit_text(),
                        }
                        span { class: "kit-set__count", "data-limit": (about_used >= INSTRUCTION_LIMIT).then_some("true"), "{about_used} / {INSTRUCTION_LIMIT}" }
                    }
                    div { class: "kit-set__field",
                        label { r#for: "{uid}-style", "How it should answer" }
                        textarea {
                            id: "{uid}-style",
                            class: "kit-set__input kit-set__area",
                            disabled: paused,
                            maxlength: "{INSTRUCTION_LIMIT}",
                            value: "{snapshot.instructions.response_style}",
                            oninput: move |e: FormEvent| {
                                let value = clip_instruction(&e.value());
                                prefs.with_mut(|p| p.instructions.response_style = value);
                            },
                            onblur: move |_| commit_text(),
                        }
                        span { class: "kit-set__count", "data-limit": (style_used >= INSTRUCTION_LIMIT).then_some("true"), "{style_used} / {INSTRUCTION_LIMIT}" }
                    }
                }
            }

            SettingsCard { heading: "Memory".to_string(),
                if memory_note {
                    p { class: "kit-set__info", "Memory is off, so nothing new is kept. Saved items stay listed. "
                        button {
                            r#type: "button",
                            class: "kit-set__link-btn",
                            onclick: move |_| {
                                document::eval(&format!("document.getElementById('{uid}-tool-memory')?.focus();"));
                            },
                            "Turn memory on in chat tools"
                        }
                    }
                }
                if snapshot.memories.is_empty() || is_empty {
                    div { class: "kit-pref__empty",
                        p { class: "kit-set__info", "Nothing is remembered yet." }
                        div { class: "kit-set__inline",
                            button {
                                r#type: "button",
                                class: "kit-set__btn",
                                onclick: move |_| {
                                    if let Some(h) = &on_navigate {
                                        h.call("Start a chat".to_string());
                                    }
                                },
                                "Start a chat"
                            }
                            if memory_note {
                                button {
                                    r#type: "button",
                                    class: "kit-set__btn",
                                    onclick: move |_| apply(&|p| {
                                        p.memory_on = true;
                                        if let Some(t) = p.chat_tools.iter_mut().find(|t| t.id == "memory") {
                                            t.is_enabled = true;
                                        }
                                    }),
                                    "Turn memory on"
                                }
                            }
                        }
                    }
                } else {
                    ul { class: "kit-pref__memories",
                        for memory in snapshot.memories.clone() {
                            {
                                let memory_id = memory.id.clone();
                                let short = memory.text.chars().take(40).collect::<String>();
                                let forget_label = format!("Forget: {short}");
                                rsx! {
                                    li { key: "{memory.id}", class: "kit-pref__memory",
                                        div { class: "kit-pref__memory-text",
                                            span { "{memory.text}" }
                                            span { class: "kit-set__info", "Saved {memory.saved_at}" }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "kit-set__btn",
                                            "aria-label": "{forget_label}",
                                            onclick: move |_| {
                                                let id = memory_id.clone();
                                                apply(&|p| p.memories.retain(|m| m.id != id));
                                            },
                                            "Forget"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "kit-set__inline",
                        button {
                            r#type: "button",
                            class: "kit-set__btn",
                            onclick: move |_| confirm_clear.set(true),
                            "Clear all"
                        }
                    }
                }
            }

            AlertDialog {
                open: confirm_clear(),
                onopenchange: move |open: bool| confirm_clear.set(open),
                title: rsx! { "Clear all memories?" },
                description: rsx! { "The assistant forgets everything it has kept. This cannot be undone." },
                confirm_label: "Clear all",
                cancel_label: "Keep memories",
                danger: true,
                onconfirm: move |_| {
                    confirm_clear.set(false);
                    apply(&|p| p.memories.clear());
                },
                oncancel: move |_| confirm_clear.set(false),
            }
            SettingsStatus { message: status() }
        }
    }
}
