//! Settings: Workspace General. The workspace's icon, name and address, the company
//! context every chat reads, and deleting the workspace behind a typed name.
//!
//! Design choice: the icon is chosen by the host (`on_upload_icon` receives the
//! file name); the kit does not read or host image bytes, and the icon preview
//! shows whatever `icon_url` the host supplies.
//! Design choice: the name saves on blur, and Enter saves and leaves the field.
//! Design choice: the address change warning shows as soon as the draft differs
//! from the saved address, before it saves on blur.
//! Browser checks: blur saves once, the context limit blocks input and turns the
//! count to the error tone, the delete confirm stays disabled until the name
//! matches exactly, Escape closes the dialog, phone width.

use dioxus::prelude::*;
use rv2_affect::dialog::Dialog;

use super::settings_profile::{
    SettingsCard, SettingsRow, SettingsShell, SettingsSkel, SettingsStatus, at_limit, char_count,
    clip_to_limit, count_label,
};
use crate::internal::next_id;

const GENERAL_CSS: Asset = asset!("/assets/settings-workspace-general.css");

#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceRecord {
    pub display_name: String,
    pub url_slug: String,
    pub company_context: String,
    pub icon_url: Option<String>,
}

/// The first letter of the workspace name: the preview fallback without an icon.
pub fn initial_letter(name: &str) -> String {
    name.trim()
        .chars()
        .next()
        .map(|c| c.to_uppercase().collect())
        .unwrap_or_default()
}

/// Whether the typed confirmation matches the workspace name exactly.
pub fn delete_matches(typed: &str, name: &str) -> bool {
    !name.is_empty() && typed == name
}

/// Workspace general settings page.
#[component]
pub fn SettingsWorkspaceGeneral(
    workspace_record: WorkspaceRecord,
    #[props(default = 2000)] context_limit: usize,
    #[props(default)] is_loading: bool,
    #[props(default)] on_save: Option<EventHandler<WorkspaceRecord>>,
    #[props(default)] on_delete: Option<EventHandler<()>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] docs_href: Option<String>,
    #[props(default)] on_upload_icon: Option<EventHandler<String>>,
    #[props(default)] account_name: Option<String>,
) -> Element {
    let uid = use_hook(|| next_id());
    let mut saved = use_signal(|| workspace_record.clone());
    let mut current = use_signal(|| workspace_record.clone());
    let mut status = use_signal(String::new);
    let mut delete_open = use_signal(|| false);
    let mut typed = use_signal(String::new);

    let mut commit = move || {
        let next = current();
        if next != saved() {
            saved.set(next.clone());
            if let Some(h) = &on_save {
                h.call(next);
            }
            status.set("Workspace saved.".to_string());
        }
    };

    let mut commit_now = move || {
        let next = current();
        saved.set(next.clone());
        if let Some(h) = &on_save {
            h.call(next);
        }
        status.set("Workspace saved.".to_string());
    };

    let record = current();
    let limit = context_limit;
    let used = char_count(&record.company_context);
    let address_changed = record.url_slug != saved().url_slug;
    let letter = initial_letter(&record.display_name);
    let name_for_delete = saved().display_name.clone();
    let confirm_label = format!("Delete {}", name_for_delete);

    if is_loading {
        return rsx! {
            SettingsShell { current: "General", title: "General", description: "Your workspace's name, icon, address and company context.", on_navigate, account_name, docs_href: docs_href.clone(),
                document::Link { rel: "stylesheet", href: GENERAL_CSS }
                div { class: "kit-set__loading", "aria-busy": "true",
                    SettingsCard { heading: "Workspace".to_string(),
                        SettingsRow { label: "Icon".to_string(), SettingsSkel { kind: "avatar" } }
                        SettingsRow { label: "Name".to_string(), SettingsSkel { kind: "field" } }
                        SettingsRow { label: "Address".to_string(), SettingsSkel { kind: "field" } }
                    }
                    SettingsCard { heading: "Company context".to_string(), SettingsSkel { kind: "block" } }
                }
            }
        };
    }

    rsx! {
        SettingsShell { current: "General", title: "General", description: "Your workspace's name, icon, address and company context.", on_navigate, account_name, docs_href: docs_href.clone(),
            document::Link { rel: "stylesheet", href: GENERAL_CSS }
            SettingsCard { heading: "Workspace".to_string(),
                SettingsRow { label: "Icon".to_string(), note: "Shown in the sidebar and on shared chats.".to_string(),
                    div { class: "kit-set__picture",
                        span { class: "kit-set__avatar", "data-initials": "{letter}", aria_hidden: "true",
                            if let Some(url) = record.icon_url.clone() {
                                img { src: "{url}", alt: "" }
                            } else {
                                "{letter}"
                            }
                        }
                        label { class: "kit-set__btn", r#for: "{uid}-icon",
                            "Upload icon"
                            input {
                                id: "{uid}-icon",
                                class: "kit-sr",
                                r#type: "file",
                                accept: "image/*",
                                onchange: move |e: FormEvent| {
                                    let value = e.value();
                                    let name = value.rsplit(['\\', '/']).next().unwrap_or_default().to_string();
                                    if let Some(h) = &on_upload_icon {
                                        h.call(name.clone());
                                    }
                                    let mut picked = document::eval(&format!(
                                        "const f = document.getElementById({:?})?.files?.[0]; dioxus.send(f ? URL.createObjectURL(f) : '');",
                                        format!("{uid}-icon")
                                    ));
                                    spawn(async move {
                                        if let Ok(url) = picked.recv::<String>().await {
                                            if url.is_empty() {
                                                return;
                                            }
                                            current.with_mut(|r| r.icon_url = Some(url));
                                            commit_now();
                                            status.set(format!("Icon selected: {name}"));
                                        }
                                    });
                                },
                            }
                        }
                        button {
                            r#type: "button",
                            class: "kit-set__btn kit-set__btn--quiet",
                            disabled: record.icon_url.is_none(),
                            "aria-label": "Remove workspace icon",
                            onclick: move |_| {
                                current.with_mut(|r| r.icon_url = None);
                                commit();
                            },
                            "Remove"
                        }
                    }
                }
                SettingsRow { label: "Name".to_string(), note: "The name members see.".to_string(),
                    input {
                        id: "{uid}-name",
                        class: "kit-set__input",
                        r#type: "text",
                        "aria-label": "Workspace name",
                        value: "{record.display_name}",
                        oninput: move |e: FormEvent| current.with_mut(|r| r.display_name = e.value()),
                        onblur: move |_| commit(),
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter {
                                e.prevent_default();
                                commit();
                                document::eval(&format!("document.getElementById({:?})?.blur();", format!("{uid}-name")));
                            }
                        },
                    }
                }
                SettingsRow { label: "Address".to_string(), note: "Your workspace's web address.".to_string(),
                    div { class: "kit-gen__slug",
                        span { class: "kit-gen__prefix", "aria-hidden": "true", "rv2.app/" }
                        input {
                            id: "{uid}-slug",
                            class: "kit-set__input kit-gen__slug-input",
                            r#type: "text",
                            "aria-label": "Workspace address",
                            "aria-describedby": "{uid}-slug-warning",
                            value: "{record.url_slug}",
                            oninput: move |e: FormEvent| current.with_mut(|r| r.url_slug = e.value()),
                            onblur: move |_| commit(),
                        }
                        if address_changed {
                            p { id: "{uid}-slug-warning", class: "kit-gen__warning", role: "alert",
                                "Changing the address breaks links that members have already saved."
                            }
                        }
                    }
                }
            }

            SettingsCard { heading: "Company context".to_string(),
                note: "Every chat in this workspace reads this. Two or three short paragraphs work best.".to_string(),
                div { class: "kit-set__field",
                    label { r#for: "{uid}-context", "About the company" }
                    textarea {
                        id: "{uid}-context",
                        class: "kit-set__input kit-set__area",
                        maxlength: "{limit}",
                        value: "{record.company_context}",
                        oninput: move |e: FormEvent| {
                            let value = clip_to_limit(&e.value(), limit);
                            current.with_mut(|r| r.company_context = value);
                        },
                        onblur: move |_| commit(),
                    }
                    span {
                        class: "kit-set__count",
                        "data-limit": at_limit(used, limit).then_some("true"),
                        "{count_label(used, limit)}"
                    }
                }
            }

            SettingsCard { heading: "Danger zone".to_string(), danger: true,
                SettingsRow { label: "Delete workspace".to_string(), note: "Removes the workspace, its chats and its files for every member.".to_string(),
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        onclick: move |_| {
                            typed.set(String::new());
                            delete_open.set(true);
                        },
                        "Delete workspace"
                    }
                }
            }

            Dialog {
                open: delete_open(),
                onopenchange: move |open: bool| delete_open.set(open),
                title: rsx! { "Delete {name_for_delete}?" },
                description: rsx! { "Type the workspace name to confirm. This cannot be undone." },
                footer: rsx! {
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        onclick: move |_| delete_open.set(false),
                        "Cancel"
                    }
                    button {
                        r#type: "button",
                        class: "kit-set__btn kit-gen__danger",
                        disabled: !delete_matches(&typed(), &name_for_delete),
                        "aria-label": "{confirm_label}",
                        onclick: move |_| {
                            delete_open.set(false);
                            status.set("Workspace deletion requested.".to_string());
                            if let Some(h) = &on_delete {
                                h.call(());
                            }
                        },
                        "Delete workspace"
                    }
                },
                div { class: "kit-set__field",
                    label { r#for: "{uid}-confirm-name", "Workspace name" }
                    input {
                        id: "{uid}-confirm-name",
                        class: "kit-set__input",
                        r#type: "text",
                        autocomplete: "off",
                        value: "{typed()}",
                        oninput: move |e: FormEvent| typed.set(e.value()),
                    }
                }
            }
            SettingsStatus { message: status() }
        }
    }
}
