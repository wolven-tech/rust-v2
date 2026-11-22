use dioxus::prelude::*;
use rv2_kit::screens::settings_billing::{
    AddOn, AddOnTier, Allowance, BillInterval, BillingAddress, BillingCycle, BreakdownLine,
    CardInfo, CurrentPlan, Invoice, InvoiceStatus, PlanCard, SettingsBilling,
};
use rv2_kit::screens::settings_members_teams::{
    Member, MemberRole, Seats, SettingsMembersTeams, Team,
};
use rv2_kit::screens::settings_model_access::{
    AccessLevel, ApiKey, HostingRegion, ModelEntry, ModelKind, ProviderEntry, SettingsModelAccess,
};
use rv2_kit::screens::settings_notification_rules::{
    Delivery, Digest, NotificationType, QuietHours, SettingsNotificationRules, WeeklyReport,
};
use rv2_kit::screens::settings_personal_preferences::{
    AllowedModel, ChatTool, Instructions, Memory, SettingsPersonalPreferences, Theme,
};
use rv2_kit::screens::settings_privacy_controls::{
    CookieChoices, Session, SettingsPrivacyControls, SharedChat, Visibility,
};
use rv2_kit::screens::settings_profile::{ProfileRecord, SettingsProfile, WeekStart};
use rv2_kit::screens::settings_workspace_general::{SettingsWorkspaceGeneral, WorkspaceRecord};
use rv2_kit::screens::settings_workspace_security::{
    EmailDomain, IdpValues, IpRange, SessionLength, SettingsWorkspaceSecurity,
};
use rv2_kit::screens::workspace_frame_flat::WorkspaceFrameFlat;
use rv2_kit::screens::workspace_frame_inset::{
    Artifact, ArtifactKind, ChatRow, FrameAccount, MenuIcon, MenuItem, WorkspaceFrame,
};

use std::future::Future;
use std::pin::Pin;

use super::{Item, Section};

const FRAME_HEIGHT: &str = "height: 720px; padding: 0; overflow: hidden;";

#[component]
pub fn ScreensFrames() -> Element {
    rsx! {
        Section { id: "screens_frames", title: "Screens: frames and settings",
            WorkspaceFrameInsetDemo {}
            WorkspaceFrameFlatDemo {}
            WorkspaceFrameVariantsDemo {}
            SettingsBillingDemo {}
            SettingsMembersTeamsDemo {}
            SettingsModelAccessDemo {}
            SettingsNotificationRulesDemo {}
            SettingsPersonalPreferencesDemo {}
            SettingsPrivacyControlsDemo {}
            SettingsProfileDemo {}
            SettingsWorkspaceGeneralDemo {}
            SettingsWorkspaceSecurityDemo {}
            SettingsStatesDemo {}
            WorkAreaArtifactsDemo {}
        }
    }
}

fn menu() -> Vec<MenuItem> {
    vec![
        MenuItem {
            item_label: "New chat".into(),
            item_icon: MenuIcon::Chat,
        },
        MenuItem {
            item_label: "Folders".into(),
            item_icon: MenuIcon::Folder,
        },
        MenuItem {
            item_label: "Integrations".into(),
            item_icon: MenuIcon::Plug,
        },
    ]
}

fn chats() -> Vec<ChatRow> {
    vec![
        ChatRow {
            id: "c1".into(),
            title: "Pricing review".into(),
            pinned: true,
            unread: false,
            project_id: None,
            section: None,
        },
        ChatRow {
            id: "c2".into(),
            title: "Hiring plan Q1".into(),
            pinned: false,
            unread: true,
            project_id: Some("Growth".into()),
            section: None,
        },
        ChatRow {
            id: "c3".into(),
            title: "Board notes".into(),
            pinned: false,
            unread: false,
            project_id: None,
            section: None,
        },
    ]
}

fn account() -> FrameAccount {
    FrameAccount {
        name: "Decebal Dobrica".into(),
        avatar_url: None,
        email: Some("decebal@example.com".into()),
    }
}

#[component]
fn WorkspaceFrameInsetDemo() -> Element {
    let mut last = use_signal(|| "no navigation yet".to_string());
    rsx! {
        Item { slug: "workspace-frame-inset", name: "WorkspaceFrame",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                WorkspaceFrame {
                    frame_style: rv2_kit::screens::workspace_frame_inset::FrameStyle::Inset,
                    current_item: "New chat".to_string(),
                    workspace_name: "Northwind".to_string(),
                    account: account(),
                    menu_items: menu(),
                    project_list: vec!["Growth".to_string()],
                    chat_list: chats(),
                    work_area: true,
                    side_panel: rsx! { p { class: "gl-tag", "Notes for this page" } },
                    on_navigate: move |label: String| last.set(format!("navigate: {label}")),
                    content: rsx! {
                        div { style: "padding: 16px;",
                            SettingsProfile {
                                profile_record: profile(),
                                role_options: vec!["Founder".to_string()],
                                timezone_options: vec!["Europe/Bucharest".to_string()],
                            }
                        }
                    },
                }
            }
            p { class: "gl-out", "data-out": "workspace-frame-inset", "{last()}" }
        }
    }
}

#[component]
fn WorkspaceFrameFlatDemo() -> Element {
    let mut last = use_signal(|| "no navigation yet".to_string());
    rsx! {
        Item { slug: "workspace-frame-flat", name: "WorkspaceFrameFlat",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                WorkspaceFrameFlat {
                    current_item: "Folders".to_string(),
                    workspace_name: "Northwind".to_string(),
                    account: account(),
                    menu_items: menu(),
                    menu_groups: vec![rv2_kit::screens::workspace_frame_inset::MenuGroup {
                        title: "Pipelines".to_string(),
                        items: vec![MenuItem { item_label: "Sales".into(), item_icon: MenuIcon::Folder }],
                    }],
                    project_list: vec!["Growth".to_string()],
                    chat_list: chats(),
                    top_bar_title: rsx! { "Folders" },
                    on_navigate: move |label: String| last.set(format!("navigate: {label}")),
                    content: rsx! {
                        div { style: "padding: 24px;",
                            h2 { style: "margin: 0 0 8px; font-size: 18px;", "Folders" }
                            p { class: "gl-tag", "The page area fills the space beside the sidebar." }
                        }
                    },
                }
            }
            p { class: "gl-out", "data-out": "workspace-frame-flat", "{last()}" }
        }
    }
}

#[component]
fn WorkspaceFrameVariantsDemo() -> Element {
    rsx! {
        Item { slug: "workspace-frame-inset-no-cards", name: "WorkspaceFrame (side_cards false)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                WorkspaceFrame {
                    frame_style: rv2_kit::screens::workspace_frame_inset::FrameStyle::Inset,
                    current_item: "New chat".to_string(),
                    workspace_name: "Northwind".to_string(),
                    account: account(),
                    menu_items: menu(),
                    chat_list: chats(),
                    side_cards: false,
                    content: rsx! { div { style: "padding: 16px;", "Page without side cards." } },
                }
            }
            p { class: "gl-out", "data-out": "workspace-frame-inset-no-cards", "no cards" }
        }
        Item { slug: "workspace-frame-flat-no-header", name: "WorkspaceFrameFlat (top_bar false)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                WorkspaceFrameFlat {
                    workspace_name: "Northwind".to_string(),
                    account: account(),
                    menu_items: menu(),
                    top_bar: false,
                    content: rsx! { div { style: "padding: 16px;", "Page starts at the top, no header." } },
                }
            }
            p { class: "gl-out", "data-out": "workspace-frame-flat-no-header", "no header" }
        }
    }
}

fn profile() -> ProfileRecord {
    ProfileRecord {
        first_name: "Decebal".into(),
        last_name: "Dobrica".into(),
        account_email: "decebal@example.com".into(),
        job_role: Some("Founder".into()),
        time_zone: "Europe/Bucharest".into(),
        week_start: WeekStart::Monday,
        personal_context: "I run a small product team building planning tools.".into(),
        avatar_url: Some("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='56' height='56'><rect width='56' height='56' fill='%23475569'/></svg>".into()),
        has_password: false,
    }
}

#[component]
fn SettingsProfileDemo() -> Element {
    let mut last = use_signal(|| "no save yet".to_string());
    rsx! {
        Item { slug: "settings-profile", name: "SettingsProfile",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsProfile {
                    profile_record: profile(),
                    role_options: vec!["Founder".to_string(), "Engineer".to_string()],
                    timezone_options: vec!["Europe/Bucharest".to_string(), "UTC".to_string()],
                    docs_href: "/docs/settings/profile".to_string(),
                    on_save: move |value: ProfileRecord| last.set(format!("on_save first_name={}", value.first_name)),
                }
            }
            p { class: "gl-out", "data-out": "settings-profile", "{last()}" }
        }
    }
}

fn plan() -> CurrentPlan {
    CurrentPlan {
        name: "Team".into(),
        price_per_seat: "€18".into(),
        interval: BillInterval::Month,
        seats_used: 6,
        seats_total: 10,
        renews_on: "1 Dec 2026".into(),
        upcoming_amount: "€108".into(),
        breakdown: vec![
            BreakdownLine {
                label: "Seats (6 × €18)".into(),
                amount: "€108".into(),
            },
            BreakdownLine {
                label: "Storage add-on".into(),
                amount: "€0".into(),
            },
        ],
    }
}

fn plans() -> Vec<PlanCard> {
    vec![
        PlanCard {
            id: "team".into(),
            name: "Team".into(),
            monthly_per_seat: "€18".into(),
            yearly_per_seat: "€15".into(),
            features: vec!["Shared chats".into(), "Team folders".into()],
            current: true,
            recommended: false,
        },
        PlanCard {
            id: "scale".into(),
            name: "Scale".into(),
            monthly_per_seat: "€30".into(),
            yearly_per_seat: "€25".into(),
            features: vec!["Single sign-on".into(), "Audit log".into()],
            current: false,
            recommended: true,
        },
    ]
}

#[component]
fn SettingsBillingDemo() -> Element {
    let mut last = use_signal(|| "no action yet".to_string());
    rsx! {
        Item { slug: "settings-billing", name: "SettingsBilling",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsBilling {
                    current_plan: plan(),
                    allowances: vec![
                        Allowance { label: "Seats".into(), used: 6, total: 10, unit: "seats".into() },
                        Allowance { label: "Credits".into(), used: 1200, total: 5000, unit: "credits".into() },
                        Allowance { label: "Storage".into(), used: 3, total: 20, unit: "GB".into() },
                        Allowance { label: "Runs".into(), used: 88, total: 100, unit: "runs".into() },
                    ],
                    billing_cycle: BillingCycle::Monthly,
                    plans: plans(),
                    add_ons: vec![AddOn {
                        id: "storage".into(),
                        name: "Storage".into(),
                        description: "More space for files and exports.".into(),
                        tiers: vec![
                            AddOnTier { id: "s20".into(), label: "20 GB".into(), price: "€5".into(), current: true },
                            AddOnTier { id: "s100".into(), label: "100 GB".into(), price: "€15".into(), current: false },
                        ],
                    }],
                    billing_address: BillingAddress { company: "Northwind Ltd".into(), line: "1 Market Street, Cluj".into(), email: "billing@example.com".into(), vat_number: Some("RO123".into()) },
                    card: CardInfo { brand: "Visa".into(), last4: "4242".into(), expires: "08/29".into(), is_default: true },
                    invoices: vec![
                        Invoice { reference: "INV-1002".into(), date: "1 Nov 2026".into(), amount: "€108".into(), status: InvoiceStatus::Paid },
                        Invoice { reference: "INV-1003".into(), date: "1 Dec 2026".into(), amount: "€108".into(), status: InvoiceStatus::Upcoming },
                    ],
                    account: Some(rv2_kit::screens::settings_billing::AccountBlock { name: "Decebal".into(), avatar_url: None }),
                    pending_plan: None,
                    on_change_plan: move |id: String| last.set(format!("on_change_plan {id}")),
                    on_add_add_on: move |(id, tier): (String, String)| last.set(format!("on_add_add_on {id} {tier}")),
                    on_edit_address: move |_| last.set("on_edit_address".to_string()),
                    on_edit_card: move |_| last.set("on_edit_card".to_string()),
                    on_cancel: move |_| last.set("on_cancel".to_string()),
                    on_preview_bill: move |_| last.set("on_preview_bill".to_string()),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-billing", "{last()}" }
        }
    }
}

fn member(id: &str, name: &str, role: MemberRole, usage: f64, invited: bool) -> Member {
    Member {
        id: id.into(),
        display_name: Some(name.into()),
        email_address: format!("{}@northwind.example", name.to_lowercase()),
        member_role: role,
        plan_name: "Team".into(),
        usage_ratio: usage,
        is_invited: invited,
    }
}

#[component]
fn SettingsMembersTeamsDemo() -> Element {
    let mut last = use_signal(|| "no action yet".to_string());
    rsx! {
        Item { slug: "settings-members-teams", name: "SettingsMembersTeams",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsMembersTeams {
                    member_list: vec![
                        member("m1", "Ada", MemberRole::Admin, 0.42, false),
                        member("m2", "Grace", MemberRole::Member, 0.8, false),
                        member("m3", "Linus", MemberRole::Member, 0.15, false),
                        member("m4", "Mira", MemberRole::Viewer, 0.0, true),
                        member("m5", "Tove", MemberRole::Viewer, 0.31, false),
                    ],
                    team_list: vec![Team { id: "t1".into(), display_name: "Core".into(), member_ids: vec!["m1".into(), "m2".into()] }],
                    seats: Seats { used: 5, total: 10 },
                    current_user_id: "m1".to_string(),
                    on_invite: move |(emails, role): (Vec<String>, MemberRole)| last.set(format!("on_invite {} {}", emails.len(), role.as_str())),
                    on_change_role: move |(id, role): (String, MemberRole)| last.set(format!("on_change_role {id} {}", role.as_str())),
                    on_remove: move |id: String| last.set(format!("on_remove {id}")),
                    on_bulk: move |(ids, action, _value): (Vec<String>, rv2_kit::screens::settings_members_teams::BulkAction, Option<String>)| last.set(format!("on_bulk {} {:?}", ids.len(), action)),
                    on_export: move |_| last.set("on_export".to_string()),
                    on_create_team: move |_| last.set("on_create_team".to_string()),
                    on_edit_team: move |id: String| last.set(format!("on_edit_team {id}")),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-members-teams", "{last()}" }
        }
    }
}

fn providers() -> Vec<ProviderEntry> {
    vec![
        ProviderEntry {
            id: "lab".into(),
            display_name: "Lab endpoint".into(),
            hosting_region: HostingRegion::Custom,
            logo_url: None,
            model_list: vec![ModelEntry {
                id: "lab-1".into(),
                display_name: "Lab One".into(),
                model_kind: ModelKind::Chat,
                is_enabled: true,
                access_level: AccessLevel::Everyone,
            }],
        },
        ProviderEntry {
            id: "eu".into(),
            display_name: "Eurocloud".into(),
            hosting_region: HostingRegion::Eu,
            logo_url: None,
            model_list: vec![
                ModelEntry {
                    id: "eu-chat".into(),
                    display_name: "Euro Chat".into(),
                    model_kind: ModelKind::Chat,
                    is_enabled: true,
                    access_level: AccessLevel::Engineering,
                },
                ModelEntry {
                    id: "eu-img".into(),
                    display_name: "Euro Picture".into(),
                    model_kind: ModelKind::Image,
                    is_enabled: true,
                    access_level: AccessLevel::Admins,
                },
            ],
        },
        ProviderEntry {
            id: "global".into(),
            display_name: "Global Host".into(),
            hosting_region: HostingRegion::Global,
            logo_url: None,
            model_list: vec![ModelEntry {
                id: "gl-chat".into(),
                display_name: "Global Chat".into(),
                model_kind: ModelKind::Chat,
                is_enabled: false,
                access_level: AccessLevel::Everyone,
            }],
        },
    ]
}

#[component]
fn SettingsModelAccessDemo() -> Element {
    let mut last = use_signal(|| "no save yet".to_string());
    rsx! {
        Item { slug: "settings-model-access", name: "SettingsModelAccess",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsModelAccess {
                    provider_list: providers(),
                    default_chat: "eu-chat".to_string(),
                    default_image: "eu-img".to_string(),
                    api_keys: vec![ApiKey { provider_id: "eu".into(), masked_value: "sk-…4c2a".into(), added_at: "3 Nov 2026".into() }],
                    on_save: move |state: rv2_kit::screens::settings_model_access::ModelAccessState| last.set(format!("on_save chat={} image={}", state.default_chat, state.default_image)),
                    on_add_key: move |(provider, _value): (String, String)| last.set(format!("on_add_key {provider}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-model-access", "{last()}" }
        }
    }
}

#[component]
fn SettingsNotificationRulesDemo() -> Element {
    let mut last = use_signal(|| "no change yet".to_string());
    rsx! {
        Item { slug: "settings-notification-rules", name: "SettingsNotificationRules",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsNotificationRules {
                    digest: Digest { is_enabled: true, send_time: "08:30".into(), time_zone: "Europe/Bucharest".into(), includes: vec!["Tasks".into()] },
                    weekly_report: WeeklyReport { is_enabled: true, day: "Monday".into() },
                    types: vec![
                        NotificationType { id: "reminders".into(), type_name: "Task reminders".into(), type_note: "Due today or overdue".into(), delivery: Delivery::Both },
                        NotificationType { id: "mentions".into(), type_name: "Mentions".into(), type_note: "When someone names you".into(), delivery: Delivery::Inbox },
                        NotificationType { id: "runs".into(), type_name: "Run finished".into(), type_note: "An agent run completes".into(), delivery: Delivery::Email },
                    ],
                    quiet_hours: QuietHours { is_enabled: true, start_time: "22:00".into(), end_time: "07:00".into() },
                    on_save: move |values: rv2_kit::screens::settings_notification_rules::NotificationValues| last.set(format!("on_save types={}", values.types.len())),
                }
            }
            p { class: "gl-out", "data-out": "settings-notification-rules", "{last()}" }
        }
    }
}

#[component]
fn SettingsPersonalPreferencesDemo() -> Element {
    let mut last = use_signal(|| "no save yet".to_string());
    rsx! {
        Item { slug: "settings-personal-preferences", name: "SettingsPersonalPreferences",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsPersonalPreferences {
                    default_model: "auto".to_string(),
                    default_image_model: "img-1".to_string(),
                    models: vec![
                        AllowedModel { id: "chat-1".into(), name: "Chat One".into(), kind: ModelKind::Chat },
                        AllowedModel { id: "img-1".into(), name: "Image One".into(), kind: ModelKind::Image },
                    ],
                    language: "English".to_string(),
                    theme: Theme::System,
                    chat_tools: vec![
                        ChatTool { id: "search".into(), label: "Web search".into(), description: "Look things up".into(), is_enabled: true },
                        ChatTool { id: "memory".into(), label: "Memory".into(), description: "Keep useful notes".into(), is_enabled: true },
                        ChatTool { id: "images".into(), label: "Image generation".into(), description: "Make pictures".into(), is_enabled: false },
                    ],
                    instructions: Instructions { is_enabled: true, about_text: "I run a small product team.".into(), response_style: "Short, plain sentences.".into() },
                    memories: vec![Memory { id: "k1".into(), text: "Prefers weekly reviews on Monday.".into(), saved_at: "2 Nov 2026".into() }],
                    memory_on: true,
                    on_save: move |prefs: rv2_kit::screens::settings_personal_preferences::Preferences| last.set(format!("on_save theme={} memories={}", prefs.theme.as_str(), prefs.memories.len())),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-personal-preferences", "{last()}" }
        }
    }
}

#[component]
fn SettingsPrivacyControlsDemo() -> Element {
    let mut last = use_signal(|| "no action yet".to_string());
    let mut saved = use_signal(|| "no save yet".to_string());
    rsx! {
        Item { slug: "settings-privacy-controls", name: "SettingsPrivacyControls",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsPrivacyControls {
                    ad_measurement: false,
                    product_analytics: true,
                    training_on_chats: false,
                    cookies: CookieChoices { essential: true, optional: vec!["Preferences".into()] },
                    product_email: true,
                    sessions: vec![
                        Session { id: "s1".into(), device_label: "MacBook Pro".into(), place: "Cluj-Napoca".into(), last_active: "now".into(), is_current: true },
                        Session { id: "s2".into(), device_label: "iPhone".into(), place: "Bucharest".into(), last_active: "yesterday".into(), is_current: false },
                    ],
                    shared_chats: vec![
                        SharedChat { id: "c1".into(), chat_title: "Q4 roadmap".into(), visibility: Visibility::Workspace, shared_at: "5 Nov 2026".into(), view_count: 4, link: "https://example.com/c1".into() },
                        SharedChat { id: "c2".into(), chat_title: "Launch notes".into(), visibility: Visibility::Public, shared_at: "9 Nov 2026".into(), view_count: 31, link: "https://example.com/c2".into() },
                    ],
                    on_save: move |values: rv2_kit::screens::settings_privacy_controls::PrivacyValues| saved.set(format!("on_save sessions={} links={}", values.sessions.len(), values.shared_chats.len())),
                    on_sign_out_session: move |id: String| last.set(format!("on_sign_out_session {id}")),
                    on_sign_out_others: move |_| last.set("on_sign_out_others".to_string()),
                    on_stop_share: move |id: String| last.set(format!("on_stop_share {id}")),
                    on_copy_link: move |id: String| last.set(format!("on_copy_link {id}")),
                    on_export: move |_| last.set("on_export".to_string()),
                    on_delete_chats: move |_| last.set("on_delete_chats".to_string()),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-privacy-controls", "{last()} / {saved()}" }
        }
    }
}

#[component]
fn SettingsWorkspaceGeneralDemo() -> Element {
    let mut last = use_signal(|| "no save yet".to_string());
    rsx! {
        Item { slug: "settings-workspace-general", name: "SettingsWorkspaceGeneral",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsWorkspaceGeneral {
                    workspace_record: WorkspaceRecord {
                        display_name: "Northwind".into(),
                        url_slug: "northwind".into(),
                        company_context: "Northwind builds planning tools for small product teams. We sell to founders and ops leads.".into(),
                        icon_url: Some("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='56' height='56'><rect width='56' height='56' rx='12' fill='%23334155'/></svg>".into()),
                    },
                    account_name: Some("Decebal Dobrica".to_string()),
                    on_save: move |ws: WorkspaceRecord| last.set(format!("on_save name={}", ws.display_name)),
                    on_delete: move |_| last.set("on_delete".to_string()),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-workspace-general", "{last()}" }
        }
    }
}

#[component]
fn SettingsWorkspaceSecurityDemo() -> Element {
    let mut last = use_signal(|| "no action yet".to_string());
    rsx! {
        Item { slug: "settings-workspace-security", name: "SettingsWorkspaceSecurity",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsWorkspaceSecurity {
                    email_domains: vec![
                        EmailDomain { name: "northwind.example".into(), verified: true, txt_record: "rv2-verification=northwind.example".into() },
                        EmailDomain { name: "agency.example".into(), verified: false, txt_record: "rv2-verification=agency.example".into() },
                        EmailDomain { name: "partners.example".into(), verified: false, txt_record: "rv2-verification=partners.example".into() },
                    ],
                    join_by_domain: true,
                    require_sso: false,
                    two_step_required: true,
                    session_length: SessionLength::SevenDays,
                    idp: IdpValues {
                        audience_uri: "urn:rv2:northwind".into(),
                        acs_url: "https://rv2.example/sso/acs".into(),
                        entity_id: "urn:rv2:northwind:sp".into(),
                        metadata_url: String::new(),
                    },
                    ip_ranges: vec![IpRange { id: "r1".into(), range: "10.0.0.0/24".into(), label: None }],
                    on_verify_domain: Callback::new(|domain: String| -> Pin<Box<dyn Future<Output = bool>>> {
                        Box::pin(async move { domain == "partners.example" })
                    }),
                    on_save: move |values: rv2_kit::screens::settings_workspace_security::SecurityValues| last.set(format!("on_save session={}", values.session_length.as_str())),
                    on_sign_out_all: move |_| last.set("on_sign_out_all".to_string()),
                    on_navigate: move |label: String| last.set(format!("navigate {label}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-workspace-security", "{last()}" }
        }
    }
}

#[component]
fn SettingsStatesDemo() -> Element {
    let mut last = use_signal(|| "no action yet".to_string());
    let mut last_members = use_signal(|| "no action yet".to_string());
    let mut last_models = use_signal(|| "no action yet".to_string());
    let mut last_prefs = use_signal(|| "no action yet".to_string());
    let mut last_privacy = use_signal(|| "no action yet".to_string());
    let mut last_security = use_signal(|| "no action yet".to_string());
    rsx! {
        Item { slug: "settings-profile-loading", name: "SettingsProfile (loading)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsProfile {
                    profile_record: profile(),
                    role_options: vec!["Founder".to_string()],
                    timezone_options: vec!["Europe/Bucharest".to_string()],
                    is_loading: true,
                }
            }
            p { class: "gl-out", "data-out": "settings-profile-loading", "loading" }
        }
        Item { slug: "settings-billing-loading", name: "SettingsBilling (loading)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsBilling {
                    current_plan: plan(),
                    allowances: vec![Allowance { label: "Seats".into(), used: 6, total: 10, unit: "seats".into() }],
                    billing_cycle: BillingCycle::Monthly,
                    plans: plans(),
                    add_ons: vec![],
                    billing_address: BillingAddress { company: "Northwind Ltd".into(), line: "1 Market Street, Cluj".into(), email: "billing@example.com".into(), vat_number: None },
                    card: CardInfo { brand: "Visa".into(), last4: "4242".into(), expires: "08/29".into(), is_default: true },
                    invoices: vec![],
                    is_loading: true,
                }
            }
            p { class: "gl-out", "data-out": "settings-billing-loading", "loading" }
        }
        Item { slug: "settings-billing-empty", name: "SettingsBilling (new subscription)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsBilling {
                    current_plan: plan(),
                    allowances: vec![Allowance { label: "Seats".into(), used: 1, total: 10, unit: "seats".into() }],
                    billing_cycle: BillingCycle::Monthly,
                    plans: plans(),
                    add_ons: vec![],
                    billing_address: BillingAddress { company: "Northwind Ltd".into(), line: "1 Market Street, Cluj".into(), email: "billing@example.com".into(), vat_number: None },
                    card: CardInfo { brand: "Visa".into(), last4: "4242".into(), expires: "08/29".into(), is_default: true },
                    invoices: vec![],
                    is_empty: true,
                    on_preview_bill: move |_| last.set("on_preview_bill".to_string()),
                }
            }
            p { class: "gl-out", "data-out": "settings-billing-empty", "{last()}" }
        }
        Item { slug: "settings-members-teams-loading", name: "SettingsMembersTeams (loading)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsMembersTeams {
                    member_list: vec![],
                    team_list: vec![],
                    seats: Seats { used: 0, total: 10 },
                    current_user_id: "m1".to_string(),
                    is_loading: true,
                }
            }
            p { class: "gl-out", "data-out": "settings-members-teams-loading", "loading" }
        }
        Item { slug: "settings-members-teams-empty", name: "SettingsMembersTeams (empty)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsMembersTeams {
                    member_list: vec![],
                    team_list: vec![],
                    seats: Seats { used: 0, total: 10 },
                    current_user_id: "m1".to_string(),
                    is_empty: true,
                    on_invite: move |(emails, role): (Vec<String>, MemberRole)| last_members.set(format!("on_invite {} {}", emails.len(), role.as_str())),
                    on_create_team: move |_| last_members.set("on_create_team".to_string()),
                }
            }
            p { class: "gl-out", "data-out": "settings-members-teams-empty", "{last_members()}" }
        }
        Item { slug: "settings-model-access-loading", name: "SettingsModelAccess (loading)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsModelAccess {
                    provider_list: providers(),
                    default_chat: "auto".to_string(),
                    default_image: "eu-img".to_string(),
                    api_keys: vec![],
                    is_loading: true,
                }
            }
            p { class: "gl-out", "data-out": "settings-model-access-loading", "loading" }
        }
        Item { slug: "settings-model-access-empty", name: "SettingsModelAccess (empty)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsModelAccess {
                    provider_list: vec![],
                    default_chat: "auto".to_string(),
                    default_image: String::new(),
                    api_keys: vec![],
                    is_empty: true,
                    on_add_key: move |(provider, _value): (String, String)| last_models.set(format!("on_add_key {provider}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-model-access-empty", "{last_models()}" }
        }
        Item { slug: "settings-notification-rules-loading", name: "SettingsNotificationRules (loading)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsNotificationRules {
                    digest: Digest { is_enabled: true, send_time: "08:30".into(), time_zone: "Europe/Bucharest".into(), includes: vec![] },
                    weekly_report: WeeklyReport { is_enabled: true, day: "Monday".into() },
                    types: vec![],
                    quiet_hours: QuietHours { is_enabled: false, start_time: "22:00".into(), end_time: "07:00".into() },
                    is_loading: true,
                }
            }
            p { class: "gl-out", "data-out": "settings-notification-rules-loading", "loading" }
        }
        Item { slug: "settings-personal-preferences-loading", name: "SettingsPersonalPreferences (loading)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsPersonalPreferences {
                    default_model: "auto".to_string(),
                    default_image_model: "img-1".to_string(),
                    models: vec![],
                    language: "English".to_string(),
                    theme: Theme::System,
                    chat_tools: vec![],
                    instructions: Instructions { is_enabled: true, about_text: String::new(), response_style: String::new() },
                    memories: vec![],
                    memory_on: true,
                    is_loading: true,
                }
            }
            p { class: "gl-out", "data-out": "settings-personal-preferences-loading", "loading" }
        }
        Item { slug: "settings-personal-preferences-empty", name: "SettingsPersonalPreferences (empty)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsPersonalPreferences {
                    default_model: "auto".to_string(),
                    default_image_model: "img-1".to_string(),
                    models: vec![AllowedModel { id: "chat-1".into(), name: "Chat One".into(), kind: ModelKind::Chat }],
                    language: "English".to_string(),
                    theme: Theme::System,
                    chat_tools: vec![ChatTool { id: "memory".into(), label: "Memory".into(), description: "Keep useful notes".into(), is_enabled: true }],
                    instructions: Instructions { is_enabled: true, about_text: String::new(), response_style: String::new() },
                    memories: vec![],
                    memory_on: true,
                    is_empty: true,
                    on_save: move |_: rv2_kit::screens::settings_personal_preferences::Preferences| last_prefs.set("on_save".to_string()),
                }
            }
            p { class: "gl-out", "data-out": "settings-personal-preferences-empty", "{last_prefs()}" }
        }
        Item { slug: "settings-privacy-controls-loading", name: "SettingsPrivacyControls (loading)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsPrivacyControls {
                    ad_measurement: false,
                    product_analytics: true,
                    training_on_chats: false,
                    cookies: CookieChoices { essential: true, optional: vec![] },
                    product_email: true,
                    sessions: vec![],
                    shared_chats: vec![],
                    is_loading: true,
                }
            }
            p { class: "gl-out", "data-out": "settings-privacy-controls-loading", "loading" }
        }
        Item { slug: "settings-privacy-controls-empty", name: "SettingsPrivacyControls (nothing shared)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsPrivacyControls {
                    ad_measurement: false,
                    product_analytics: true,
                    training_on_chats: false,
                    cookies: CookieChoices { essential: true, optional: vec![] },
                    product_email: false,
                    sessions: vec![Session { id: "s1".into(), device_label: "MacBook Pro".into(), place: "Cluj-Napoca".into(), last_active: "now".into(), is_current: true }],
                    shared_chats: vec![],
                    is_empty: true,
                    on_navigate: move |label: String| last_privacy.set(format!("navigate {label}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-privacy-controls-empty", "{last_privacy()}" }
        }
        Item { slug: "settings-personal-preferences-memory-off", name: "SettingsPersonalPreferences (memory off)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsPersonalPreferences {
                    default_model: "auto".to_string(),
                    default_image_model: "img-1".to_string(),
                    models: vec![AllowedModel { id: "chat-1".into(), name: "Chat One".into(), kind: ModelKind::Chat }],
                    language: "English".to_string(),
                    theme: Theme::Light,
                    chat_tools: vec![ChatTool { id: "memory".into(), label: "Memory".into(), description: "Keep useful notes".into(), is_enabled: false }],
                    instructions: Instructions { is_enabled: true, about_text: "I run a small product team.".into(), response_style: "Short.".into() },
                    memories: vec![Memory { id: "k1".into(), text: "Prefers weekly reviews on Monday.".into(), saved_at: "2 Nov 2026".into() }],
                    memory_on: false,
                    on_navigate: move |label: String| last_prefs.set(format!("navigate {label}")),
                }
            }
            p { class: "gl-out", "data-out": "settings-personal-preferences-memory-off", "{last_prefs()}" }
        }
        Item { slug: "settings-workspace-general-loading", name: "SettingsWorkspaceGeneral (loading)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsWorkspaceGeneral {
                    workspace_record: WorkspaceRecord { display_name: "Northwind".into(), url_slug: "northwind".into(), company_context: String::new(), icon_url: None },
                    is_loading: true,
                }
            }
            p { class: "gl-out", "data-out": "settings-workspace-general-loading", "loading" }
        }
        Item { slug: "settings-workspace-security-loading", name: "SettingsWorkspaceSecurity (loading)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsWorkspaceSecurity {
                    email_domains: vec![],
                    join_by_domain: false,
                    require_sso: false,
                    two_step_required: false,
                    session_length: SessionLength::SevenDays,
                    idp: IdpValues { audience_uri: String::new(), acs_url: String::new(), entity_id: String::new(), metadata_url: String::new() },
                    ip_ranges: vec![],
                    is_loading: true,
                }
            }
            p { class: "gl-out", "data-out": "settings-workspace-security-loading", "loading" }
        }
        Item { slug: "settings-workspace-security-empty", name: "SettingsWorkspaceSecurity (empty)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                SettingsWorkspaceSecurity {
                    email_domains: vec![],
                    join_by_domain: false,
                    require_sso: false,
                    two_step_required: false,
                    session_length: SessionLength::SevenDays,
                    idp: IdpValues { audience_uri: "urn:rv2:northwind".into(), acs_url: "https://rv2.example/sso/acs".into(), entity_id: "urn:rv2:northwind:sp".into(), metadata_url: String::new() },
                    ip_ranges: vec![],
                    is_empty: true,
                    on_sign_out_all: move |_| last_security.set("on_sign_out_all".to_string()),
                }
            }
            p { class: "gl-out", "data-out": "settings-workspace-security-empty", "{last_security()}" }
        }
    }
}

#[component]
fn WorkAreaArtifactsDemo() -> Element {
    let mut last = use_signal(|| "no artifact opened".to_string());
    let artifacts = vec![
        Artifact {
            name: "Plan.md".into(),
            kind: ArtifactKind::Document,
            body: "Q1 plan.\n\nHire two engineers.".into(),
        },
        Artifact {
            name: "Budget.csv".into(),
            kind: ArtifactKind::Sheet,
            body: "item, amount\nHosting, 120\nTools, 80".into(),
        },
        Artifact {
            name: "Build.rs".into(),
            kind: ArtifactKind::Code,
            body: "fn main() {}".into(),
        },
        Artifact {
            name: "Contracts".into(),
            kind: ArtifactKind::Folder,
            body: "Hosting agreement\nDesign retainer\nHiring letters".into(),
        },
        Artifact {
            name: "Hero.svg".into(),
            kind: ArtifactKind::Image,
            body: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='320' height='200'><rect width='320' height='200' fill='%23475569'/><circle cx='160' cy='100' r='56' fill='%2394a3b8'/></svg>".into(),
        },
    ];
    let open_inset = vec![
        artifacts[0].clone(),
        artifacts[1].clone(),
        artifacts[3].clone(),
    ];
    let open_flat = vec![artifacts[2].clone(), artifacts[4].clone()];
    rsx! {
        Item { slug: "workspace-frame-inset-artifacts", name: "WorkspaceFrame (work area with artifacts)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                WorkspaceFrame {
                    frame_style: rv2_kit::screens::workspace_frame_inset::FrameStyle::Inset,
                    current_item: "Folders".to_string(),
                    workspace_name: "Northwind".to_string(),
                    account: account(),
                    menu_items: menu(),
                    chat_list: chats(),
                    work_area: true,
                    artifacts: artifacts.clone(),
                    initial_tabs: open_inset,
                    on_open_artifact: move |name: String| last.set(format!("opened {name}")),
                    content: rsx! { div { style: "padding: 16px;", "Page beside the work area." } },
                }
            }
            p { class: "gl-out", "data-out": "workspace-frame-inset-artifacts", "{last()}" }
        }
        Item { slug: "workspace-frame-flat-work-area", name: "WorkspaceFrameFlat (work area)",
            div { class: "gl-canvas", style: FRAME_HEIGHT,
                WorkspaceFrameFlat {
                    workspace_name: "Northwind".to_string(),
                    account: account(),
                    menu_items: menu(),
                    chat_list: chats(),
                    work_area: true,
                    side_panel: rsx! { p { class: "gl-tag", "Notes for this page" } },
                    artifacts: artifacts,
                    initial_tabs: open_flat,
                    content: rsx! { div { style: "padding: 16px;", "Flat page with a work area." } },
                }
            }
            p { class: "gl-out", "data-out": "workspace-frame-flat-work-area", "flat" }
        }
    }
}
