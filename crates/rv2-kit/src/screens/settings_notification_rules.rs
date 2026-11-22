//! Settings: Notification Rules. A daily digest at a chosen time, a weekly report,
//! a per-type channel grid, and quiet hours for mobile pushes.
//!
//! Design choice: every change calls `on_save` with the full set of values, as the
//! spec says; text inputs save on change, not on blur, because a time field has no
//! blur-only commit.
//! Design choice: the timezone is shown as text, not a picker, since the digest
//! already follows the profile's zone.
//! Browser checks: dependent rows appear and leave with their switch, the set-all
//! control saves once, labels carry the type name, phone width stacks the selects.

use dioxus::prelude::*;
use rv2_affect::switch::Switch;

use super::settings_profile::{SettingsCard, SettingsShell, SettingsSkel, SettingsStatus};

const NOTIFICATIONS_CSS: Asset = asset!("/assets/settings-notification-rules.css");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Delivery {
    #[default]
    Both,
    Email,
    Mobile,
    Inbox,
}

impl Delivery {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Both => "both",
            Self::Email => "email",
            Self::Mobile => "mobile",
            Self::Inbox => "inbox",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Both => "Email and mobile",
            Self::Email => "Email",
            Self::Mobile => "Mobile",
            Self::Inbox => "Inbox only",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "email" => Self::Email,
            "mobile" => Self::Mobile,
            "inbox" => Self::Inbox,
            _ => Self::Both,
        }
    }

    pub const ALL: [Delivery; 4] = [Self::Inbox, Self::Email, Self::Mobile, Self::Both];
}

#[derive(Clone, Debug, PartialEq)]
pub struct Digest {
    pub is_enabled: bool,
    pub send_time: String,
    pub time_zone: String,
    pub includes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeeklyReport {
    pub is_enabled: bool,
    pub day: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NotificationType {
    pub id: String,
    pub type_name: String,
    pub type_note: String,
    pub delivery: Delivery,
}

#[derive(Clone, Debug, PartialEq)]
pub struct QuietHours {
    pub is_enabled: bool,
    pub start_time: String,
    pub end_time: String,
}

/// Every notification setting, as `on_save` receives it.
#[derive(Clone, Debug, PartialEq)]
pub struct NotificationValues {
    pub digest: Digest,
    pub weekly_report: WeeklyReport,
    pub types: Vec<NotificationType>,
    pub quiet_hours: QuietHours,
}

/// The channel menu's accessible name for one notification type.
pub fn channel_label(type_name: &str) -> String {
    format!("Channel for {type_name}")
}

/// Sets every type to `delivery`, leaving the rest of the values as they are.
pub fn set_all(values: &mut NotificationValues, delivery: Delivery) {
    for t in values.types.iter_mut() {
        t.delivery = delivery;
    }
}

/// Notifications settings page.
#[component]
pub fn SettingsNotificationRules(
    digest: Digest,
    weekly_report: WeeklyReport,
    types: Vec<NotificationType>,
    quiet_hours: QuietHours,
    #[props(default)] is_loading: bool,
    #[props(default)] on_save: Option<EventHandler<NotificationValues>>,
    #[props(default)] on_navigate: Option<EventHandler<String>>,
    #[props(default)] docs_href: Option<String>,
) -> Element {
    let mut values = use_signal(|| NotificationValues {
        digest: digest.clone(),
        weekly_report: weekly_report.clone(),
        types: types.clone(),
        quiet_hours: quiet_hours.clone(),
    });
    let mut status = use_signal(String::new);

    let mut apply = move |edit: &dyn Fn(&mut NotificationValues)| {
        values.with_mut(|v| edit(v));
        if let Some(h) = &on_save {
            h.call(values());
        }
        status.set("Notification settings saved.".to_string());
    };

    let snapshot = values();

    if is_loading {
        return rsx! {
            SettingsShell { current: "Notifications", title: "Notifications", description: "What you hear about, when, and where it arrives.", on_navigate, docs_href: docs_href.clone(),
                document::Link { rel: "stylesheet", href: NOTIFICATIONS_CSS }
                div { class: "kit-set__loading", "aria-busy": "true",
                    SettingsCard { heading: "Digest".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Channels".to_string(), SettingsSkel { kind: "block" } }
                    SettingsCard { heading: "Quiet hours".to_string(), SettingsSkel { kind: "field" } }
                }
            }
        };
    }

    rsx! {
        SettingsShell { current: "Notifications", title: "Notifications", description: "What you hear about, when, and where it arrives.", on_navigate, docs_href: docs_href.clone(),
            document::Link { rel: "stylesheet", href: NOTIFICATIONS_CSS }
            SettingsCard { heading: "Digest".to_string(),
                div { class: "kit-nrules__switch",
                    Switch {
                        label: rsx! { "Daily digest" },
                        description: rsx! { "One summary of what waited and what finished." },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        checked: snapshot.digest.is_enabled,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.digest.is_enabled = on);
                        },
                    }
                }
                if snapshot.digest.is_enabled {
                    div { class: "kit-nrules__dependent",
                        label { class: "kit-nrules__field",
                            span { "Send time" }
                            input {
                                class: "kit-set__input kit-nrules__time",
                                r#type: "time",
                                "aria-label": "Digest send time",
                                value: "{snapshot.digest.send_time}",
                                onchange: move |e: FormEvent| {
                                    let value = e.value();
                                    apply(&|v| v.digest.send_time = value.clone());
                                },
                            }
                        }
                        p { class: "kit-set__info kit-nrules__zone", "Time zone: {snapshot.digest.time_zone}" }
                    }
                }
                div { class: "kit-nrules__switch",
                    Switch {
                        label: rsx! { "Weekly usage report" },
                        description: rsx! { "Usage across the workspace, once a week." },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        checked: snapshot.weekly_report.is_enabled,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.weekly_report.is_enabled = on);
                        },
                    }
                }
                if snapshot.weekly_report.is_enabled {
                    p { class: "kit-set__info kit-nrules__dependent", "Sent on {snapshot.weekly_report.day}." }
                }
            }

            SettingsCard { heading: "Channels".to_string(),
                note: "Choose where each kind of alert arrives.".to_string(),
                div { class: "kit-nrules__table-wrap",
                    table { class: "kit-nrules__table",
                        thead {
                            tr {
                                th { scope: "col", "Type" }
                                th { scope: "col",
                                    label { class: "kit-nrules__set-all",
                                        span { "Channel" }
                                        select {
                                            class: "kit-set__input kit-nrules__select",
                                            "aria-label": "Set every type to one channel",
                                            value: "",
                                            onchange: move |e: FormEvent| {
                                                let next = Delivery::parse(&e.value());
                                                if e.value().is_empty() {
                                                    return;
                                                }
                                                apply(&|v| set_all(v, next));
                                            },
                                            option { value: "", "Set all…" }
                                            for choice in Delivery::ALL {
                                                option { key: "{choice.as_str()}", value: "{choice.as_str()}", "{choice.label()}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        tbody {
                            for notification in snapshot.types.clone() {
                                {
                                    let label = channel_label(&notification.type_name);
                                    let type_id = notification.id.clone();
                                    let delivery = notification.delivery;
                                    rsx! {
                                        tr { key: "{notification.id}",
                                            td { class: "kit-nrules__type",
                                                span { class: "kit-nrules__type-name", "{notification.type_name}" }
                                                span { class: "kit-set__info", "{notification.type_note}" }
                                            }
                                            td {
                                                select {
                                                    class: "kit-set__input kit-nrules__select",
                                                    "aria-label": "{label}",
                                                    onchange: move |e: FormEvent| {
                                                        let next = Delivery::parse(&e.value());
                                                        let id = type_id.clone();
                                                        apply(&|v| {
                                                            if let Some(t) = v.types.iter_mut().find(|t| t.id == id) {
                                                                t.delivery = next;
                                                            }
                                                        });
                                                    },
                                                    for choice in Delivery::ALL {
                                                        option { key: "{choice.as_str()}", value: "{choice.as_str()}", selected: delivery == choice, "{choice.label()}" }
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

            SettingsCard { heading: "Quiet hours".to_string(),
                note: "Mobile pushes wait until the window ends.".to_string(),
                div { class: "kit-nrules__switch",
                    Switch {
                        label: rsx! { "Quiet hours" },
                        description: rsx! { "Hold mobile pushes during this window." },
                        placement: rv2_affect::switch::SwitchPlacement::End,
                        checked: snapshot.quiet_hours.is_enabled,
                        onchange: move |e: FormEvent| {
                            let on = e.checked();
                            apply(&|v| v.quiet_hours.is_enabled = on);
                        },
                    }
                }
                if snapshot.quiet_hours.is_enabled {
                    div { class: "kit-nrules__dependent kit-nrules__window",
                        label { class: "kit-nrules__field",
                            span { "From" }
                            input {
                                class: "kit-set__input kit-nrules__time",
                                r#type: "time",
                                value: "{snapshot.quiet_hours.start_time}",
                                onchange: move |e: FormEvent| {
                                    let value = e.value();
                                    apply(&|v| v.quiet_hours.start_time = value.clone());
                                },
                            }
                        }
                        label { class: "kit-nrules__field",
                            span { "To" }
                            input {
                                class: "kit-set__input kit-nrules__time",
                                r#type: "time",
                                value: "{snapshot.quiet_hours.end_time}",
                                onchange: move |e: FormEvent| {
                                    let value = e.value();
                                    apply(&|v| v.quiet_hours.end_time = value.clone());
                                },
                            }
                        }
                    }
                }
            }
            SettingsStatus { message: status() }
        }
    }
}
