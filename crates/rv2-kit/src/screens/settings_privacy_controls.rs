//! Settings: Privacy Controls. What data about a person and their chats is used,
//! which cookies this browser keeps, product email, signed-in devices, shared chat
//! links, and the export and delete-all actions.
//!
//! Design choice: `on_save` receives the whole privacy state, sessions and shared
//! links included, after every change, as the spec says.
//! Design choice: a revoked device or link leaves the list at once; the fade is a
//! CSS exit on the row and the list closes the gap after it.
//! Design choice: deleting every chat asks twice. The first dialog states the
//! consequence; the second, marked destructive, is the one that calls
//! `on_delete_chats`.
//! Browser checks: switch states, cookie disclosure, the current device badge,
//! sign-out and stop-share removing rows, shared-link scope tabs, two-step delete,
//! export busy state, phone width.

use dioxus::prelude::*;
use rv2_affect::dialog::AlertDialog;
use rv2_affect::switch::Switch;

use super::settings_profile::{
    SettingsCard, SettingsRow, SettingsShell, SettingsSkel, SettingsStatus,
};
use crate::internal::{copy_to_clipboard, next_id};

const PRIVACY_CSS: Asset = asset!("/assets/settings-privacy-controls.css");

/// Optional cookie categories this browser may keep. Essential cookies are always on.
pub const COOKIE_CATEGORIES: [&str; 2] = ["Preferences", "Performance"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visibility {
    #[default]
    Workspace,
    Public,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: String,
    pub device_label: String,
    pub place: String,
    pub last_active: String,
    pub is_current: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SharedChat {
    pub id: String,
    pub chat_title: String,
    pub visibility: Visibility,
    pub shared_at: String,
    pub view_count: u32,
    pub link: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CookieChoices {
    pub essential: bool,
    pub optional: Vec<String>,
}

/// Every privacy value, as `on_save` receives it.
#[derive(Clone, Debug, PartialEq)]
pub struct PrivacyValues {
    pub ad_measurement: bool,
    pub product_analytics: bool,
    pub training_on_chats: bool,
    pub cookies: CookieChoices,
    pub product_email: bool,
    pub sessions: Vec<Session>,
    pub shared_chats: Vec<SharedChat>,
}

/// Shared-link tab a row is listed under.
pub fn in_scope(chat: &SharedChat, scope: Visibility) -> bool {
    chat.visibility == scope
}

/// The no-match line for an empty shared-link tab.
pub fn empty_scope_message(scope: Visibility) -> &'static str {
    match scope {
        Visibility::Workspace => "No chats are shared with the workspace.",
        Visibility::Public => "No public links are open.",
    }
}

/// Privacy settings page.
#[component]
pub fn SettingsPrivacyControls(
    ad_measurement: bool,
    product_analytics: bool,
    training_on_chats: bool,
    cookies: CookieChoices,
    product_email: bool,
    sessions: Vec<Session>,
    shared_chats: Vec<SharedChat>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] export_busy: bool,
    #[props(default)] on_save: Option<EventHandler<PrivacyValues>>,
    #[props(default)] on_sign_out_session: Option<EventHandler<String>>,
    #[props(default)] on_sign_out_others: Option<EventHandler<()>>,
    #[props(default)] on_stop_share: Option<EventHandler<String>>,
    #[props(default)] on_copy_link: Option<EventHandler<String>>,
    #[props(default)] on_export: Option<EventHandler<()>>,
    #[props(default)] on_delete_chats: Option<EventHandler<()>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] docs_href: Option<String>,
) -> Element {
    let uid = use_hook(|| next_id());
    let mut values = use_signal(|| PrivacyValues {
        ad_measurement,
        product_analytics,
        training_on_chats,
        cookies: cookies.clone(),
        product_email,
        sessions: sessions.clone(),
        shared_chats: shared_chats.clone(),
    });
    let mut cookies_open = use_signal(|| false);
    let mut scope = use_signal(|| Visibility::Workspace);
    let mut status = use_signal(String::new);
    let mut copied = use_signal(|| None::<String>);
    let mut delete_step = use_signal(|| 0u8);

    let mut apply = move |edit: &dyn Fn(&mut PrivacyValues)| {
        values.with_mut(|v| edit(v));
        if let Some(h) = &on_save {
            h.call(values());
        }
    };

    let snapshot = values();
    let only_this_device = snapshot.sessions.iter().all(|s| s.is_current);
    let workspace_count = snapshot
        .shared_chats
        .iter()
        .filter(|c| in_scope(c, Visibility::Workspace))
        .count();
    let public_count = snapshot
        .shared_chats
        .iter()
        .filter(|c| in_scope(c, Visibility::Public))
        .count();
    let others = snapshot.sessions.iter().filter(|s| !s.is_current).count();
    let visible_chats: Vec<SharedChat> = snapshot
        .shared_chats
        .iter()
        .filter(|c| in_scope(c, scope()))
        .cloned()
        .collect();

    if is_loading {
        return rsx! {
            SettingsShell { current: "Privacy", title: "Privacy", description: "What is used, what is shared, and what stays on your devices.", on_navigate, docs_href: docs_href.clone(),
                document::Link { rel: "stylesheet", href: PRIVACY_CSS }
                div { class: "kit-set__loading", "aria-busy": "true",
                    SettingsCard { heading: "Data and cookies".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Email".to_string(), SettingsSkel { kind: "field" } }
                    SettingsCard { heading: "Active sessions".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Shared chats".to_string(), SettingsSkel { kind: "block" } }
                }
            }
        };
    }

    rsx! {
        SettingsShell { current: "Privacy", title: "Privacy", description: "What is used, what is shared, and what stays on your devices.", on_navigate, docs_href: docs_href.clone(),
            document::Link { rel: "stylesheet", href: PRIVACY_CSS }
            SettingsCard { heading: "Data and cookies".to_string(),
                div { class: "kit-prv__switch",
                    Switch {
                        label: rsx! { "Ad measurement" },
                        description: rsx! { "Share hashed contact and sign-up events with ad partners." },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        checked: snapshot.ad_measurement,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.ad_measurement = on);
                        },
                    }
                }
                div { class: "kit-prv__switch",
                    Switch {
                        label: rsx! { "Product analytics" },
                        description: rsx! { "Count which features are used. Never the content." },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        checked: snapshot.product_analytics,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.product_analytics = on);
                        },
                    }
                }
                div { class: "kit-prv__switch",
                    Switch {
                        label: rsx! { "Training on chats" },
                        description: rsx! { "Let anonymised chat content help train later models. Off by default." },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        checked: snapshot.training_on_chats,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.training_on_chats = on);
                        },
                    }
                }
                div { class: "kit-prv__cookies",
                    div { class: "kit-set__row",
                        div { class: "kit-set__row-text",
                            span { class: "kit-set__row-label", "Cookies" }
                            span { class: "kit-set__row-note", "Essential cookies are always on. Choose the optional ones this browser keeps." }
                        }
                        div { class: "kit-set__row-control",
                            button {
                                r#type: "button",
                                class: "kit-set__btn",
                                aria_expanded: cookies_open().to_string(),
                                aria_controls: "{uid}-cookies",
                                onclick: move |_| cookies_open.set(!cookies_open()),
                                "Manage"
                            }
                        }
                    }
                    if cookies_open() {
                        fieldset { id: "{uid}-cookies", class: "kit-prv__choices",
                            legend { class: "kit-sr", "Optional cookies" }
                            label { class: "kit-set__radio",
                                input { r#type: "checkbox", checked: true, disabled: true }
                                "Essential (always on)"
                            }
                            for category in COOKIE_CATEGORIES {
                                {
                                    let on = snapshot.cookies.optional.iter().any(|c| c == category);
                                    rsx! {
                                        label { key: "{category}", class: "kit-set__radio",
                                            input {
                                                r#type: "checkbox",
                                                checked: on,
                                                onchange: move |e: FormEvent| {
                                                    let checked = e.checked();
                                                    apply(&|v| {
                                                        v.cookies.optional.retain(|c| c != category);
                                                        if checked {
                                                            v.cookies.optional.push(category.to_string());
                                                        }
                                                    });
                                                },
                                            }
                                            "{category}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                p { class: "kit-set__info kit-prv__scope", "These settings apply to all of your devices." }
            }

            SettingsCard { heading: "Email".to_string(),
                div { class: "kit-prv__switch",
                    Switch {
                        label: rsx! { "Product updates" },
                        description: rsx! { "New features and changes, at most once a week." },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        checked: snapshot.product_email,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.product_email = on);
                        },
                    }
                }
            }

            SettingsCard { heading: "Active sessions".to_string(),
                note: "Devices signed in to your account.".to_string(),
                div { class: "kit-prv__card-action",
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        disabled: others == 0,
                        onclick: move |_| {
                            if let Some(h) = &on_sign_out_others {
                                h.call(());
                            }
                            apply(&|v| v.sessions.retain(|s| s.is_current));
                            status.set("Signed out of every other device.".to_string());
                        },
                        "Sign out other devices"
                    }
                }
                if only_this_device {
                    p { class: "kit-set__info kit-prv__only", "Only this device is signed in." }
                }
                ul { class: "kit-prv__rows",
                    for session in snapshot.sessions.clone() {
                        {
                            let session_id = session.id.clone();
                            let label = format!("Sign out {}", session.device_label);
                            rsx! {
                                li { key: "{session.id}", class: "kit-prv__row",
                                    div { class: "kit-prv__row-text",
                                        span { class: "kit-set__row-label", "{session.device_label}" }
                                        span { class: "kit-set__row-note", "{session.place} · last active {session.last_active}" }
                                    }
                                    if session.is_current {
                                        span { class: "kit-set__tag", "This device" }
                                    } else {
                                        button {
                                            r#type: "button",
                                            class: "kit-set__btn",
                                            "aria-label": "{label}",
                                            onclick: move |_| {
                                                let id = session_id.clone();
                                                if let Some(h) = &on_sign_out_session {
                                                    h.call(id.clone());
                                                }
                                                apply(&|v| v.sessions.retain(|s| s.id != id));
                                                status.set("Device signed out.".to_string());
                                            },
                                            "Sign out"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            SettingsCard { heading: "Shared chats".to_string(),
                note: "Chats you have shared by link.".to_string(),
                div { class: "kit-prv__tabs", role: "tablist", aria_label: "Shared chats scope",
                    onkeydown: move |e: KeyboardEvent| {
                        let next = match e.key() {
                            Key::ArrowRight | Key::ArrowLeft => Some(match scope() { Visibility::Workspace => Visibility::Public, Visibility::Public => Visibility::Workspace }),
                            _ => None,
                        };
                        if let Some(next) = next {
                            scope.set(next);
                            let target = if next == Visibility::Workspace { format!("{uid}-scope-workspace") } else { format!("{uid}-scope-public") };
                            document::eval(&format!("requestAnimationFrame(() => document.getElementById({target:?})?.focus());"));
                        }
                    },
                    button {
                        id: "{uid}-scope-workspace",
                        r#type: "button",
                        role: "tab",
                        class: "kit-prv__tab",
                        aria_selected: (scope() == Visibility::Workspace).to_string(),
                        tabindex: if scope() == Visibility::Workspace { "0" } else { "-1" },
                        onclick: move |_| scope.set(Visibility::Workspace),
                        "In the workspace "
                        span { class: "kit-prv__count", "{workspace_count}" }
                    }
                    button {
                        id: "{uid}-scope-public",
                        r#type: "button",
                        role: "tab",
                        class: "kit-prv__tab",
                        aria_selected: (scope() == Visibility::Public).to_string(),
                        tabindex: if scope() == Visibility::Public { "0" } else { "-1" },
                        onclick: move |_| scope.set(Visibility::Public),
                        "Public links "
                        span { class: "kit-prv__count", "{public_count}" }
                    }
                }
                if visible_chats.is_empty() {
                    div { class: "kit-prv__empty",
                        p { class: "kit-set__info", "{empty_scope_message(scope())}" }
                        button {
                            r#type: "button",
                            class: "kit-set__btn",
                            onclick: move |_| {
                                if let Some(h) = &on_navigate {
                                    h.call("Chats".to_string());
                                }
                            },
                            "Open chats"
                        }
                    }
                } else {
                    ul { class: "kit-prv__rows",
                        for chat in visible_chats.clone() {
                            {
                                let chat_id = chat.id.clone();
                                let stop_id = chat.id.clone();
                                let link = chat.link.clone();
                                let copy_label = format!("Copy link to {}", chat.chat_title);
                                let stop_label = format!("Stop sharing {}", chat.chat_title);
                                let copied_here = copied() == Some(chat.id.clone());
                                rsx! {
                                    li { key: "{chat.id}", class: "kit-prv__row",
                                        div { class: "kit-prv__row-text",
                                            span { class: "kit-set__row-label", "{chat.chat_title}" }
                                            span { class: "kit-set__row-note", "Shared {chat.shared_at} · {chat.view_count} views" }
                                        }
                                        div { class: "kit-set__inline",
                                            button {
                                                r#type: "button",
                                                class: "kit-set__btn",
                                                "aria-label": "{copy_label}",
                                                onclick: move |_| {
                                                    copy_to_clipboard(&link);
                                                    copied.set(Some(chat_id.clone()));
                                                    if let Some(h) = &on_copy_link {
                                                        h.call(chat_id.clone());
                                                    }
                                                },
                                                if copied_here { "Copied" } else { "Copy link" }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-set__btn",
                                                "aria-label": "{stop_label}",
                                                onclick: move |_| {
                                                    let id = stop_id.clone();
                                                    if let Some(h) = &on_stop_share {
                                                        h.call(id.clone());
                                                    }
                                                    apply(&|v| v.shared_chats.retain(|c| c.id != id));
                                                    status.set("Link stopped.".to_string());
                                                },
                                                "Stop sharing"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            SettingsCard { heading: "Danger zone".to_string(), danger: true,
                SettingsRow { label: "Export your data".to_string(), note: "Every chat and file, as one archive.".to_string(),
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        aria_busy: export_busy.then_some("true"),
                        disabled: export_busy,
                        onclick: move |_| {
                            if let Some(h) = &on_export {
                                h.call(());
                            }
                            status.set("Preparing your export.".to_string());
                        },
                        if export_busy { "Preparing…" } else { "Export" }
                    }
                }
                SettingsRow { label: "Delete all chats".to_string(), note: "Removes every chat from this workspace. It cannot be undone.".to_string(),
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        onclick: move |_| delete_step.set(1),
                        "Delete all chats"
                    }
                }
            }

            AlertDialog {
                open: delete_step() == 1,
                onopenchange: move |open: bool| {
                    if !open {
                        delete_step.set(0);
                    }
                },
                title: rsx! { "Delete all chats?" },
                description: rsx! { "Every chat in this workspace is removed for everyone who uses it." },
                confirm_label: "Continue",
                cancel_label: "Keep chats",
                onconfirm: move |_| delete_step.set(2),
                oncancel: move |_| delete_step.set(0),
            }
            AlertDialog {
                open: delete_step() == 2,
                onopenchange: move |open: bool| {
                    if !open {
                        delete_step.set(0);
                    }
                },
                title: rsx! { "This cannot be undone" },
                description: rsx! { "Confirm once more to delete every chat." },
                confirm_label: "Delete all chats",
                cancel_label: "Keep chats",
                danger: true,
                onconfirm: move |_| {
                    delete_step.set(0);
                    status.set("All chats deleted.".to_string());
                    if let Some(h) = &on_delete_chats {
                        h.call(());
                    }
                },
                oncancel: move |_| delete_step.set(0),
            }
            SettingsStatus { message: status() }
        }
    }
}
