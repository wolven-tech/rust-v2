//! Settings: Workspace Security. Which email domains the organisation owns, who may
//! join by domain, single sign-on and two-step rules, session length, the identity
//! provider values to copy, the network allowlist, and sign-out-everyone.
//!
//! Design choice: `on_verify_domain` returns a future that resolves once the TXT
//! lookup finishes; Verify shows a busy state until then. A false answer keeps the
//! domain pending and reports it in the status line (the kit has no toast).
//! Design choice: join by domain and SSO stay disabled until a domain is verified,
//! and each switch's label carries the reason.
//! Design choice: the copy control shows "Copied" text as well as the check icon.
//! Browser checks: disabled reasons, verify true and false paths, copy confirmation,
//! Enter adds a range, the no-ranges line, sign-out confirm with Escape, remove
//! buttons reachable by keyboard focus, phone width.

use std::future::Future;
use std::pin::Pin;

use dioxus::prelude::*;
use rv2_affect::dialog::AlertDialog;
use rv2_affect::switch::Switch;

use super::settings_profile::{
    SettingsCard, SettingsRow, SettingsShell, SettingsSkel, SettingsStatus, focus_later,
};
use crate::internal::{copy_to_clipboard, next_id, sleep};

const SECURITY_CSS: Asset = asset!("/assets/settings-workspace-security.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SessionLength {
    OneDay,
    #[default]
    SevenDays,
    ThirtyDays,
    Never,
}

impl SessionLength {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneDay => "1d",
            Self::SevenDays => "7d",
            Self::ThirtyDays => "30d",
            Self::Never => "never",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::OneDay => "1 day",
            Self::SevenDays => "7 days",
            Self::ThirtyDays => "30 days",
            Self::Never => "Never",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "1d" => Self::OneDay,
            "30d" => Self::ThirtyDays,
            "never" => Self::Never,
            _ => Self::SevenDays,
        }
    }

    pub const ALL: [SessionLength; 4] =
        [Self::OneDay, Self::SevenDays, Self::ThirtyDays, Self::Never];
}

#[derive(Clone, Debug, PartialEq)]
pub struct EmailDomain {
    pub name: String,
    pub verified: bool,
    pub txt_record: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IdpValues {
    pub audience_uri: String,
    pub acs_url: String,
    pub entity_id: String,
    pub metadata_url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IpRange {
    pub id: String,
    pub range: String,
    pub label: Option<String>,
}

/// Every security value, as `on_save` receives it.
#[derive(Clone, Debug, PartialEq)]
pub struct SecurityValues {
    pub email_domains: Vec<EmailDomain>,
    pub join_by_domain: bool,
    pub require_sso: bool,
    pub two_step_required: bool,
    pub session_length: SessionLength,
    pub idp: IdpValues,
    pub ip_ranges: Vec<IpRange>,
}

/// Whether a domain name is plausible: labels separated by dots, no spaces.
pub fn valid_domain(name: &str) -> bool {
    let name = name.trim();
    let labels: Vec<&str> = name.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|l| {
            !l.is_empty()
                && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                && !l.starts_with('-')
                && !l.ends_with('-')
        })
}

/// Whether a range is an IPv4 address or CIDR block: four octets, optional `/0-32`.
pub fn valid_range(range: &str) -> bool {
    let (addr, prefix) = match range.trim().split_once('/') {
        Some((a, p)) => (a, Some(p)),
        None => (range.trim(), None),
    };
    let octets: Vec<&str> = addr.split('.').collect();
    let octets_ok = octets.len() == 4
        && octets
            .iter()
            .all(|o| o.parse::<u8>().is_ok() && !o.is_empty());
    let prefix_ok = match prefix {
        None => true,
        Some(p) => p.parse::<u8>().is_ok_and(|n| n <= 32),
    };
    octets_ok && prefix_ok
}

/// The TXT record the admin adds to prove a domain: a fixed key and the domain.
pub fn txt_record_for(domain: &str) -> String {
    format!("rv2-verification={}", domain.trim().to_lowercase())
}

/// Whether any domain is verified: the precondition for joining by domain and SSO.
pub fn any_verified(domains: &[EmailDomain]) -> bool {
    domains.iter().any(|d| d.verified)
}

/// Checks a domain's TXT record; the future resolves true once the record is found.
pub type VerifyCheck = Callback<String, Pin<Box<dyn Future<Output = bool>>>>;

/// Workspace security settings page.
#[component]
pub fn SettingsWorkspaceSecurity(
    email_domains: Vec<EmailDomain>,
    join_by_domain: bool,
    require_sso: bool,
    two_step_required: bool,
    session_length: SessionLength,
    idp: IdpValues,
    ip_ranges: Vec<IpRange>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_verify_domain: Option<VerifyCheck>,
    #[props(default)] on_save: Option<EventHandler<SecurityValues>>,
    #[props(default)] on_sign_out_all: Option<EventHandler<()>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] docs_href: Option<String>,
) -> Element {
    let uid = use_hook(|| next_id());
    let mut values = use_signal(|| SecurityValues {
        email_domains: email_domains.clone(),
        join_by_domain,
        require_sso,
        two_step_required,
        session_length,
        idp: idp.clone(),
        ip_ranges: ip_ranges.clone(),
    });
    let mut domain_draft = use_signal(|| None::<String>);
    let mut range_draft = use_signal(|| None::<String>);
    let mut sign_out_open = use_signal(|| false);
    let mut copied = use_signal(|| None::<&'static str>);
    let mut status = use_signal(String::new);
    let mut next_range = use_signal(|| 1usize);
    let mut verifying = use_signal(|| None::<String>);

    let mut apply = move |edit: &dyn Fn(&mut SecurityValues)| {
        values.with_mut(|v| edit(v));
        if let Some(h) = &on_save {
            h.call(values());
        }
    };

    let snapshot = values();
    let verified_any = any_verified(&snapshot.email_domains);
    let sso_reason = if verified_any {
        "Members at verified domains sign in through the provider.".to_string()
    } else {
        "Unavailable until a domain is verified.".to_string()
    };
    let join_reason = if verified_any {
        "Members with verified addresses join without an invite.".to_string()
    } else {
        "Unavailable until a domain is verified.".to_string()
    };

    if is_loading {
        return rsx! {
            SettingsShell { current: "Security", title: "Security", description: "Who may sign in, and how.", on_navigate, docs_href: docs_href.clone(),
                document::Link { rel: "stylesheet", href: SECURITY_CSS }
                div { class: "kit-set__loading", "aria-busy": "true",
                    SettingsCard { heading: "Email domains".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Sign-in".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Identity provider".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "IP allowlist".to_string(), SettingsSkel { kind: "block" } }
                }
            }
        };
    }

    rsx! {
        SettingsShell { current: "Security", title: "Security", description: "Who may sign in, and how.", on_navigate, docs_href: docs_href.clone(),
            document::Link { rel: "stylesheet", href: SECURITY_CSS }
            SettingsCard { heading: "Email domains".to_string(),
                note: "Addresses at a verified domain belong to your organisation.".to_string(),
                div { class: "kit-sec__toolbar",
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        aria_expanded: domain_draft().is_some().to_string(),
                        onclick: move |_| {
                            if domain_draft().is_none() {
                                domain_draft.set(Some(String::new()));
                                focus_later(&format!("{uid}-domain"));
                            }
                        },
                        "Add domain"
                    }
                }
                if let Some(draft) = domain_draft() {
                    form { class: "kit-set__form", aria_label: "Add domain",
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape {
                                domain_draft.set(None);
                            }
                        },
                        onsubmit: move |e: FormEvent| {
                            e.prevent_default();
                            let name = domain_draft().unwrap_or_default().trim().to_lowercase();
                            if !valid_domain(&name) {
                                status.set("Enter a domain such as company.com.".to_string());
                                return;
                            }
                            let record = txt_record_for(&name);
                            apply(&|v| v.email_domains.push(EmailDomain { name: name.clone(), verified: false, txt_record: record.clone() }));
                            domain_draft.set(None);
                            status.set(format!("Added {name}. Add the TXT record, then verify."));
                        },
                        label { r#for: "{uid}-domain", "Domain" }
                        input {
                            id: "{uid}-domain", class: "kit-set__input", r#type: "text", placeholder: "company.com",
                            value: "{draft}", oninput: move |e: FormEvent| domain_draft.set(Some(e.value())),
                        }
                        p { class: "kit-set__info", "Press Enter to add. Escape cancels." }
                    }
                }
                if snapshot.email_domains.is_empty() || is_empty {
                    div { class: "kit-sec__empty",
                        p { class: "kit-set__info", "No domains yet. Add one to let members join by domain." }
                    }
                } else {
                    ul { class: "kit-sec__rows",
                        for domain in snapshot.email_domains.clone() {
                            {
                                let name = domain.name.clone();
                                let verify_name = domain.name.clone();
                                let remove_name = domain.name.clone();
                                let record_line = domain.txt_record.clone();
                                let verify_label = format!("Verify {}", domain.name);
                                let is_busy = verifying().as_deref() == Some(domain.name.as_str());
                                let remove_label = format!("Remove {}", domain.name);
                                rsx! {
                                    li { key: "{domain.name}", class: "kit-sec__row", "data-verified": domain.verified.then_some("true"),
                                        div { class: "kit-sec__row-main",
                                            span { class: "kit-set__row-label", "{name}" }
                                            span { class: "kit-set__tag", "data-status": if domain.verified { "verified" } else { "pending" },
                                                if domain.verified { "Verified" } else { "Pending" }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "kit-sec__remove",
                                                "aria-label": "{remove_label}",
                                                onclick: move |_| {
                                                    let target = remove_name.clone();
                                                    apply(&|v| v.email_domains.retain(|d| d.name != target));
                                                    status.set("Domain removed.".to_string());
                                                },
                                                "Remove"
                                            }
                                        }
                                        if !domain.verified {
                                            div { class: "kit-set__inline",
                                                button {
                                                    r#type: "button",
                                                    class: "kit-set__btn",
                                                    "aria-label": "{verify_label}",
                                                    "aria-busy": is_busy.then_some("true"),
                                                    disabled: verifying().is_some(),
                                                    onclick: move |_| {
                                                        let target = verify_name.clone();
                                                        let Some(check) = on_verify_domain else {
                                                            status.set(format!("The TXT record for {target} was not found yet. It stays pending."));
                                                            return;
                                                        };
                                                        verifying.set(Some(target.clone()));
                                                        status.set(format!("Checking the TXT record for {target}."));
                                                        let pending = check.call(target.clone());
                                                        let mut apply = apply;
                                                        spawn(async move {
                                                            let found = pending.await;
                                                            verifying.set(None);
                                                            if found {
                                                                apply(&|v| {
                                                                    if let Some(d) = v.email_domains.iter_mut().find(|d| d.name == target) {
                                                                        d.verified = true;
                                                                    }
                                                                });
                                                                status.set(format!("{target} is verified."));
                                                            } else {
                                                                status.set(format!("The TXT record for {target} was not found yet. It stays pending."));
                                                            }
                                                        });
                                                    },
                                                    if is_busy { "Checking…" } else { "Verify" }
                                                }
                                            }
                                            p { class: "kit-sec__txt",
                                                span { class: "kit-set__info", "Add this TXT record: " }
                                                code { "{record_line}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            SettingsCard { heading: "Sign-in".to_string(),
                SettingsRow { label: "Join by domain".to_string(), note: join_reason.clone(),
                    Switch {
                        label: rsx! { span { class: "kit-sr", "Join by domain" } },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        disabled: !verified_any,
                        checked: snapshot.join_by_domain && verified_any,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.join_by_domain = on);
                        },
                    }
                }
                SettingsRow { label: "Require single sign-on".to_string(), note: sso_reason.clone(),
                    Switch {
                        label: rsx! { span { class: "kit-sr", "Require single sign-on" } },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        disabled: !verified_any,
                        checked: snapshot.require_sso && verified_any,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.require_sso = on);
                        },
                    }
                }
                SettingsRow { label: "Two-step verification".to_string(), note: "Members who do not use single sign-on enter a code from an authenticator app.".to_string(),
                    Switch {
                        label: rsx! { span { class: "kit-sr", "Require two-step verification" } },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        checked: snapshot.two_step_required,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.two_step_required = on);
                        },
                    }
                }
                SettingsRow { label: "Session length".to_string(), note: "How long a signed-in session lasts without activity.".to_string(),
                    select {
                        class: "kit-set__input",
                        "aria-label": "Session length",
                        onchange: move |e: FormEvent| {
                            let next = SessionLength::parse(&e.value());
                            apply(&|v| v.session_length = next);
                        },
                        for choice in SessionLength::ALL {
                            option { key: "{choice.as_str()}", value: "{choice.as_str()}", selected: snapshot.session_length == choice, "{choice.label()}" }
                        }
                    }
                }
            }

            SettingsCard { heading: "Identity provider".to_string(),
                note: "Copy these values into your provider.".to_string(),
                for (key, label, value) in [
                    ("audience", "Audience URI", snapshot.idp.audience_uri.clone()),
                    ("entity", "SP entity ID", snapshot.idp.entity_id.clone()),
                    ("acs", "ACS URL", snapshot.idp.acs_url.clone()),
                ] {
                    {
                        let copy_label = format!("Copy {label}");
                        let value_for_copy = value.clone();
                        rsx! {
                            div { key: "{key}", class: "kit-sec__copy",
                                label { r#for: "{uid}-{key}", class: "kit-set__row-label", "{label}" }
                                div { class: "kit-set__inline",
                                    input {
                                        id: "{uid}-{key}",
                                        class: "kit-set__input kit-sec__readonly",
                                        r#type: "text",
                                        readonly: true,
                                        value: "{value}",
                                        onfocus: move |_| {
                                            document::eval(&format!("document.getElementById('{uid}-{key}')?.select();"));
                                        },
                                    }
                                    button {
                                        r#type: "button",
                                        class: "kit-set__btn",
                                        "aria-label": "{copy_label}",
                                        onclick: move |_| {
                                            copy_to_clipboard(&value_for_copy);
                                            copied.set(Some(key));
                                            spawn(async move {
                                                sleep(1500).await;
                                                copied.set(None);
                                            });
                                        },
                                        if copied() == Some(key) { "✓ Copied" } else { "Copy" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-sec__copy",
                    label { r#for: "{uid}-metadata", class: "kit-set__row-label", "Metadata URL" }
                    input {
                        id: "{uid}-metadata",
                        class: "kit-set__input",
                        r#type: "url",
                        placeholder: "https://login.example.com/metadata",
                        value: "{snapshot.idp.metadata_url}",
                        oninput: move |e: FormEvent| {
                            let value = e.value();
                            values.with_mut(|v| v.idp.metadata_url = value);
                        },
                        onblur: move |_| {
                            if let Some(h) = &on_save {
                                h.call(values());
                            }
                        },
                    }
                }
            }

            SettingsCard { heading: "IP allowlist".to_string(),
                note: "Networks allowed to connect. Use an IPv4 address or a CIDR block such as 10.0.0.0/24.".to_string(),
                div { class: "kit-sec__toolbar",
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        aria_expanded: range_draft().is_some().to_string(),
                        onclick: move |_| {
                            if range_draft().is_none() {
                                range_draft.set(Some(String::new()));
                                focus_later(&format!("{uid}-range"));
                            }
                        },
                        "Add range"
                    }
                }
                if let Some(draft) = range_draft() {
                    form { class: "kit-set__form", aria_label: "Add IP range",
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Escape {
                                range_draft.set(None);
                            }
                        },
                        onsubmit: move |e: FormEvent| {
                            e.prevent_default();
                            let range = range_draft().unwrap_or_default().trim().to_string();
                            if !valid_range(&range) {
                                status.set("Enter an address such as 10.0.0.0/24.".to_string());
                                return;
                            }
                            let id = format!("{uid}-range-{}", next_range());
                            next_range.set(next_range() + 1);
                            apply(&|v| v.ip_ranges.push(IpRange { id: id.clone(), range: range.clone(), label: None }));
                            range_draft.set(None);
                            status.set(format!("Added {range}."));
                        },
                        label { r#for: "{uid}-range", "Network range" }
                        input {
                            id: "{uid}-range", class: "kit-set__input", r#type: "text", placeholder: "10.0.0.0/24",
                            value: "{draft}", oninput: move |e: FormEvent| range_draft.set(Some(e.value())),
                        }
                        p { class: "kit-set__info", "Press Enter to add. Escape cancels." }
                    }
                }
                if snapshot.ip_ranges.is_empty() {
                    p { class: "kit-sec__open", "Every network can connect until you add a range." }
                } else {
                    ul { class: "kit-sec__ranges",
                        for range in snapshot.ip_ranges.clone() {
                            {
                                let remove_id = range.id.clone();
                                let remove_label = format!("Remove {}", range.range);
                                rsx! {
                                    li { key: "{range.id}", class: "kit-sec__range",
                                        code { "{range.range}" }
                                        button {
                                            r#type: "button",
                                            class: "kit-sec__remove",
                                            "aria-label": "{remove_label}",
                                            onclick: move |_| {
                                                let id = remove_id.clone();
                                                apply(&|v| v.ip_ranges.retain(|r| r.id != id));
                                                status.set("Range removed.".to_string());
                                            },
                                            "Remove"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            SettingsCard { heading: "Danger zone".to_string(), danger: true,
                SettingsRow { label: "Sign out everyone".to_string(), note: "Ends every session in this workspace, including yours.".to_string(),
                    button {
                        r#type: "button",
                        class: "kit-set__btn",
                        onclick: move |_| sign_out_open.set(true),
                        "Sign out everyone"
                    }
                }
            }

            AlertDialog {
                open: sign_out_open(),
                onopenchange: move |open: bool| sign_out_open.set(open),
                title: rsx! { "Sign out everyone?" },
                description: rsx! { "Every member is signed out at once and must sign in again." },
                confirm_label: "Sign out everyone",
                cancel_label: "Keep sessions",
                danger: true,
                onconfirm: move |_| {
                    sign_out_open.set(false);
                    status.set("Every session was ended.".to_string());
                    if let Some(h) = &on_sign_out_all {
                        h.call(());
                    }
                },
                oncancel: move |_| sign_out_open.set(false),
            }
            SettingsStatus { message: status() }
        }
    }
}
