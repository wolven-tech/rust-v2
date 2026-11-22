//! Settings: Profile. The shared settings frame (navigation, page header, cards,
//! rows, status line, loading shapes) that every settings screen renders inside.
//!
//! Design choice: the settings navigation is part of each settings screen, so a
//! settings page stands alone; a host that wants the workspace frame around it
//! wraps the whole page as content.
//! Design choice: form controls are native inputs. Native `select` and radio
//! groups already carry the arrow, Enter and Escape behaviour the spec describes.
//! Design choice: the documentation link is rendered only when the host passes `docs_href`.
//! Design choice: picture upload hands the chosen file name to the host through
//! `on_upload_picture`; the kit does not read or host image bytes.
//! Design choice: the form keeps its own draft and seeds it from the prop on mount.
//! Browser checks: text fields save once on blur, choices save at once, the
//! context count blocks input at its limit, loading shapes at three widths.

use dioxus::prelude::*;

use crate::internal::{KIT_CSS, next_id};

pub(crate) const SETTINGS_CSS: Asset = asset!("/assets/settings-profile.css");

pub const PERSONAL_NAV: [&str; 4] = ["Profile", "Preferences", "Notifications", "Privacy"];
pub const WORKSPACE_NAV: [&str; 5] = ["General", "Members", "Models", "Security", "Billing"];

/// Characters a text area counts, as the browser does for `maxlength`.
pub fn char_count(text: &str) -> usize {
    text.chars().count()
}

/// The `used / limit` line under a limited text area.
pub fn count_label(used: usize, limit: usize) -> String {
    format!("{used} / {limit}")
}

/// Whether a limited text area has reached its limit (error tone).
pub fn at_limit(used: usize, limit: usize) -> bool {
    used >= limit
}

/// Clips `text` to `limit` characters, the way typing past the limit is blocked.
pub fn clip_to_limit(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

/// Moves focus to the element `id` once the form it opens has rendered.
pub fn focus_later(id: &str) {
    document::eval(&format!(
        "requestAnimationFrame(() => document.getElementById({id:?})?.focus());"
    ));
}

/// Initials from a first and last name.
pub fn initials(first: &str, last: &str) -> String {
    let mut out = String::new();
    for word in [first, last] {
        if let Some(c) = word.trim().chars().next() {
            out.extend(c.to_uppercase());
        }
    }
    out
}

/// Settings page frame: navigation on the left (stacked above on narrow widths),
/// then the page header and its content. `current` is the nav label marked as
/// the page the viewer is on.
#[component]
pub fn SettingsShell(
    current: &'static str,
    title: &'static str,
    description: &'static str,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] account_name: Option<String>,
    #[props(default)] docs_href: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: SETTINGS_CSS }
        div { class: "kit-set-wrap",
          div { class: "kit-set",
            nav { class: "kit-set__nav", aria_label: "Settings",
                button {
                    r#type: "button",
                    class: "kit-set__back",
                    onclick: move |_| {
                        if let Some(h) = &on_navigate {
                            h.call("Back to app".to_string());
                        }
                    },
                    "Back to app"
                }
                for group in [("Personal", &PERSONAL_NAV[..]), ("Workspace", &WORKSPACE_NAV[..])] {
                    div { class: "kit-set__group", key: "{group.0}",
                        h2 { class: "kit-set__group-title", "{group.0}" }
                        ul { class: "kit-set__list",
                            for label in group.1.iter().copied() {
                                li { key: "{label}",
                                    button {
                                        r#type: "button",
                                        class: "kit-set__link",
                                        aria_current: (label == current).then_some("page"),
                                        onclick: move |_| {
                                            if let Some(h) = &on_navigate {
                                                h.call(label.to_string());
                                            }
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(name) = account_name {
                    p { class: "kit-set__account", "Signed in as {name}" }
                }
            }
            main { class: "kit-set__main",
                header { class: "kit-set__head",
                    h1 { class: "kit-set__title", "{title}" }
                    p { class: "kit-set__desc", "{description}" }
                    if let Some(href) = docs_href {
                        a { class: "kit-set__docs", href: "{href}", "Documentation for {title}" }
                    }
                }
                {children}
            }
          }
        }
    }
}

/// A titled card of settings rows.
#[component]
pub fn SettingsCard(
    heading: String,
    #[props(default)] note: Option<String>,
    #[props(default)] danger: bool,
    children: Element,
) -> Element {
    let id = use_hook(|| format!("kit-set-card-{}", next_id()));
    rsx! {
        section {
            class: "kit-set__card",
            "data-danger": danger.then_some("true"),
            aria_labelledby: "{id}",
            div { class: "kit-set__card-head",
                h2 { class: "kit-set__card-title", id: "{id}", "{heading}" }
                if let Some(note) = note {
                    p { class: "kit-set__card-note", "{note}" }
                }
            }
            {children}
        }
    }
}

/// One row: label and note on the left, its control on the right. Stacks on narrow
/// widths.
#[component]
pub fn SettingsRow(
    label: String,
    #[props(default)] note: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "kit-set__row",
            div { class: "kit-set__row-text",
                span { class: "kit-set__row-label", "{label}" }
                if let Some(note) = note {
                    span { class: "kit-set__row-note", "{note}" }
                }
            }
            div { class: "kit-set__row-control", {children} }
        }
    }
}

/// Polite status line for save outcomes. Always mounted so screen readers
/// register changes; only the text changes.
#[component]
pub fn SettingsStatus(message: String) -> Element {
    rsx! {
        p { class: "kit-set__status kit-sr", role: "status", aria_live: "polite", "{message}" }
    }
}

/// Grey placeholder shape of a real size, used while a screen loads.
#[component]
pub fn SettingsSkel(#[props(default = "line")] kind: &'static str) -> Element {
    rsx! {
        span { class: "kit-set__skel", "data-kind": "{kind}", aria_hidden: "true" }
    }
}

/// Profile: picture, name, role, email, time preferences, personal context and
/// how the person signs in. Text fields save on blur; choices save at once.
#[component]
pub fn SettingsProfile(
    profile_record: ProfileRecord,
    role_options: Vec<String>,
    timezone_options: Vec<String>,
    #[props(default = 500)] context_limit: usize,
    #[props(default)] is_loading: bool,
    #[props(default)] on_save: Option<EventHandler<ProfileRecord>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] docs_href: Option<String>,
    #[props(default)] on_change_email: Option<EventHandler<()>>,
    #[props(default)] on_set_password: Option<EventHandler<()>>,
    #[props(default)] on_upload_picture: Option<EventHandler<String>>,
) -> Element {
    let uid = use_hook(|| next_id());
    let mut saved = use_signal(|| profile_record.clone());
    let mut current = use_signal(|| profile_record.clone());
    let mut status = use_signal(String::new);
    let mut email_draft = use_signal(|| None::<String>);
    let mut password_open = use_signal(|| false);
    let mut password_new = use_signal(String::new);
    let mut password_repeat = use_signal(String::new);

    let mut commit = move || {
        let next = current();
        if next != saved() {
            saved.set(next.clone());
            if let Some(h) = &on_save {
                h.call(next);
            }
            status.set("Profile saved.".to_string());
        }
    };

    let record = current();
    let limit = context_limit;
    let used = char_count(&record.personal_context);
    let shown_initials = initials(&record.first_name, &record.last_name);

    if is_loading {
        return rsx! {
            SettingsShell { current: "Profile", title: "Profile", description: "Your name, picture, time preferences and how you sign in.", on_navigate, docs_href: docs_href.clone(),
                div { class: "kit-set__loading", "aria-busy": "true",
                    SettingsCard { heading: "Loading profile".to_string(),
                        SettingsRow { label: "Picture".to_string(), SettingsSkel { kind: "avatar" } }
                        SettingsRow { label: "First name".to_string(), SettingsSkel { kind: "field" } }
                        SettingsRow { label: "Last name".to_string(), SettingsSkel { kind: "field" } }
                        SettingsRow { label: "Role".to_string(), SettingsSkel { kind: "field" } }
                        SettingsRow { label: "Email".to_string(), SettingsSkel { kind: "field" } }
                    }
                    SettingsCard { heading: "Time preferences".to_string(),
                        SettingsRow { label: "Time zone".to_string(), SettingsSkel { kind: "field" } }
                        SettingsRow { label: "Week starts on".to_string(), SettingsSkel { kind: "field" } }
                    }
                    SettingsCard { heading: "Personal context".to_string(),
                        SettingsSkel { kind: "block" }
                    }
                }
            }
        };
    }

    rsx! {
        SettingsShell { current: "Profile", title: "Profile", description: "Your name, picture, time preferences and how you sign in.", on_navigate, docs_href: docs_href.clone(),
            p { class: "kit-set__info", "Changes apply to every workspace you belong to." }
            SettingsCard { heading: "Identity".to_string(),
                SettingsRow { label: "Picture".to_string(), note: "Shown beside your name in every chat.".to_string(),
                    div { class: "kit-set__picture",
                        span { class: "kit-set__avatar", "data-initials": "{shown_initials}", aria_hidden: "true",
                            if let Some(url) = record.avatar_url.clone() {
                                img { src: "{url}", alt: "" }
                            } else {
                                "{shown_initials}"
                            }
                        }
                        label { class: "kit-set__btn", r#for: "{uid}-picture",
                            "Upload"
                            input {
                                id: "{uid}-picture",
                                class: "kit-sr",
                                r#type: "file",
                                accept: "image/*",
                                onchange: move |e: FormEvent| {
                                    let value = e.value();
                                    let name = value.rsplit(['\\', '/']).next().unwrap_or_default().to_string();
                                    if let Some(h) = &on_upload_picture {
                                        h.call(name.clone());
                                    }
                                    status.set(format!("Picture selected: {name}"));
                                },
                            }
                        }
                        button {
                            r#type: "button",
                            class: "kit-set__btn kit-set__btn--quiet",
                            disabled: record.avatar_url.is_none(),
                            onclick: move |_| {
                                current.with_mut(|p| p.avatar_url = None);
                                commit();
                            },
                            "Remove picture"
                        }
                    }
                }
                SettingsRow { label: "First name".to_string(),
                    input {
                        id: "{uid}-first",
                        class: "kit-set__input",
                        r#type: "text",
                        value: "{record.first_name}",
                        oninput: move |e: FormEvent| current.with_mut(|p| p.first_name = e.value()),
                        onblur: move |_| commit(),
                    }
                }
                SettingsRow { label: "Last name".to_string(),
                    input {
                        id: "{uid}-last",
                        class: "kit-set__input",
                        r#type: "text",
                        value: "{record.last_name}",
                        oninput: move |e: FormEvent| current.with_mut(|p| p.last_name = e.value()),
                        onblur: move |_| commit(),
                    }
                }
                SettingsRow { label: "Role".to_string(),
                    select {
                        id: "{uid}-role",
                        class: "kit-set__input",
                        "aria-label": "Role",
                        onchange: move |e: FormEvent| {
                            let value = e.value();
                            current.with_mut(|p| p.job_role = (!value.is_empty()).then_some(value));
                            commit();
                        },
                        option { value: "", selected: record.job_role.is_none(), "Not set" }
                        for role in role_options.clone() {
                            option { key: "{role}", value: "{role}", selected: record.job_role.as_deref() == Some(role.as_str()), "{role}" }
                        }
                    }
                }
                SettingsRow { label: "Email".to_string(), note: "Used to sign in.".to_string(),
                    div { class: "kit-set__inline",
                        span { class: "kit-set__value", "{record.account_email}" }
                        button {
                            r#type: "button",
                            class: "kit-set__btn",
                            aria_expanded: email_draft().is_some().to_string(),
                            onclick: move |_| {
                                if email_draft().is_none() {
                                    email_draft.set(Some(String::new()));
                                    focus_later(&format!("{uid}-email"));
                                }
                            },
                            "Change"
                        }
                    }
                }
                if let Some(draft) = email_draft() {
                    form { class: "kit-set__form", aria_label: "Change email",
                        onsubmit: move |e: FormEvent| {
                            e.prevent_default();
                            if let Some(h) = &on_change_email {
                                h.call(());
                            }
                            status.set(format!("Confirmation sent to {}.", email_draft().unwrap_or_default()));
                            email_draft.set(None);
                        },
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape {
                                email_draft.set(None);
                                status.set("Email change cancelled.".to_string());
                            }
                        },
                        label { r#for: "{uid}-email", "New email" }
                        input {
                            id: "{uid}-email",
                            class: "kit-set__input",
                            r#type: "email",
                            value: "{draft}",
                            oninput: move |e: FormEvent| email_draft.set(Some(e.value())),
                        }
                        div { class: "kit-set__inline",
                            button { r#type: "submit", class: "kit-set__btn kit-set__btn--primary", "Send confirmation" }
                            button {
                                r#type: "button",
                                class: "kit-set__btn",
                                onclick: move |_| email_draft.set(None),
                                "Cancel"
                            }
                        }
                    }
                }
            }
            SettingsCard { heading: "Time preferences".to_string(),
                SettingsRow { label: "Time zone".to_string(), note: "Used by reminders and reports.".to_string(),
                    select {
                        id: "{uid}-tz",
                        class: "kit-set__input",
                        "aria-label": "Time zone",
                        onchange: move |e: FormEvent| {
                            let value = e.value();
                            current.with_mut(|p| p.time_zone = value);
                            commit();
                        },
                        for zone in timezone_options.clone() {
                            option { key: "{zone}", value: "{zone}", selected: record.time_zone == zone, "{zone}" }
                        }
                    }
                }
                SettingsRow { label: "Week starts on".to_string(),
                    div { class: "kit-set__radios", role: "radiogroup", aria_label: "Week starts on",
                        label { class: "kit-set__radio",
                            input {
                                r#type: "radio",
                                name: "week-start-{uid}",
                                value: "monday",
                                checked: record.week_start == WeekStart::Monday,
                                onchange: move |_| {
                                    current.with_mut(|p| p.week_start = WeekStart::Monday);
                                    commit();
                                },
                            }
                            "Monday"
                        }
                        label { class: "kit-set__radio",
                            input {
                                r#type: "radio",
                                name: "week-start-{uid}",
                                value: "sunday",
                                checked: record.week_start == WeekStart::Sunday,
                                onchange: move |_| {
                                    current.with_mut(|p| p.week_start = WeekStart::Sunday);
                                    commit();
                                },
                            }
                            "Sunday"
                        }
                    }
                }
            }
            SettingsCard { heading: "Personal context".to_string(),
                note: "The assistant reads this with every chat.".to_string(),
                div { class: "kit-set__field",
                    label { r#for: "{uid}-context", "About you" }
                    textarea {
                        id: "{uid}-context",
                        class: "kit-set__input kit-set__area",
                        maxlength: "{limit}",
                        value: "{record.personal_context}",
                        oninput: move |e: FormEvent| {
                            let value = clip_to_limit(&e.value(), limit);
                            current.with_mut(|p| p.personal_context = value);
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
            SettingsCard { heading: "Sign-in".to_string(),
                SettingsRow { label: "Password".to_string(),
                    note: if record.has_password {
                        "Sign-in uses an emailed link or your password.".to_string()
                    } else {
                        "Sign-in uses an emailed link. Add a password to sign in with one too.".to_string()
                    },
                    if record.has_password {
                        span { class: "kit-set__value", "Password set" }
                    } else {
                        button {
                            r#type: "button",
                            class: "kit-set__btn",
                            aria_expanded: password_open().to_string(),
                            onclick: move |_| {
                                password_open.set(!password_open());
                                if password_open() {
                                    focus_later(&format!("{uid}-pw"));
                                }
                            },
                            "Add password"
                        }
                    }
                }
                if password_open() && !record.has_password {
                    form { class: "kit-set__form", aria_label: "Add password",
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape {
                                password_open.set(false);
                                password_new.set(String::new());
                                password_repeat.set(String::new());
                            }
                        },
                        onsubmit: move |e: FormEvent| {
                            e.prevent_default();
                            if password_new().len() < 8 {
                                status.set("Use at least 8 characters.".to_string());
                                return;
                            }
                            if password_new() != password_repeat() {
                                status.set("The two passwords do not match.".to_string());
                                return;
                            }
                            if let Some(h) = &on_set_password {
                                h.call(());
                            }
                            password_open.set(false);
                            password_new.set(String::new());
                            password_repeat.set(String::new());
                            status.set("Password added.".to_string());
                        },
                        label { r#for: "{uid}-pw", "New password" }
                        input {
                            id: "{uid}-pw", class: "kit-set__input", r#type: "password", autocomplete: "new-password",
                            value: "{password_new()}", oninput: move |e: FormEvent| password_new.set(e.value()),
                        }
                        label { r#for: "{uid}-pw2", "Repeat password" }
                        input {
                            id: "{uid}-pw2", class: "kit-set__input", r#type: "password", autocomplete: "new-password",
                            value: "{password_repeat()}", oninput: move |e: FormEvent| password_repeat.set(e.value()),
                        }
                        div { class: "kit-set__inline",
                            button { r#type: "submit", class: "kit-set__btn kit-set__btn--primary", "Save password" }
                            button {
                                r#type: "button",
                                class: "kit-set__btn",
                                onclick: move |_| password_open.set(false),
                                "Cancel"
                            }
                        }
                    }
                }
            }
            SettingsStatus { message: status() }
        }
    }
}

/// The profile's data, as the settings screen reads and writes it.
#[derive(Clone, Debug, PartialEq)]
pub struct ProfileRecord {
    pub first_name: String,
    pub last_name: String,
    pub account_email: String,
    pub job_role: Option<String>,
    pub time_zone: String,
    pub week_start: WeekStart,
    pub personal_context: String,
    pub avatar_url: Option<String>,
    pub has_password: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeekStart {
    #[default]
    Monday,
    Sunday,
}

impl WeekStart {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Monday => "monday",
            Self::Sunday => "sunday",
        }
    }
}
