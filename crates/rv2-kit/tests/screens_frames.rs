use dioxus::prelude::*;
use rv2_kit::screens::settings_billing::{
    AddOn, AddOnTier, Allowance, BillInterval, BillingAddress, BillingCycle, BreakdownLine,
    CardInfo, CurrentPlan, Invoice, InvoiceStatus, PlanCard, SettingsBilling, allowance_share,
    allowance_text, plan_action_label, plan_price,
};
use rv2_kit::screens::settings_members_teams::{
    HeaderState, Member, MemberRole, RoleFilter, Seats, SettingsMembersTeams, Team, count_words,
    filter_members, header_state, range_between, usage_percent,
};
use rv2_kit::screens::settings_model_access::{
    AccessLevel, HostingRegion, ModelEntry, ModelKind, ProviderEntry, SettingsModelAccess,
    chat_choices, chat_fallback, image_choices, image_fallback, ordered, provider_matches,
};
use rv2_kit::screens::settings_notification_rules::{
    Delivery, Digest, NotificationType, NotificationValues, QuietHours, SettingsNotificationRules,
    WeeklyReport, channel_label, set_all,
};
use rv2_kit::screens::settings_personal_preferences::{
    AllowedModel, ChatTool, INSTRUCTION_LIMIT, Instructions, SettingsPersonalPreferences, Theme,
    choosable, clip_instruction,
};
use rv2_kit::screens::settings_privacy_controls::{
    CookieChoices, Session, SettingsPrivacyControls, SharedChat, Visibility, empty_scope_message,
    in_scope,
};
use rv2_kit::screens::settings_profile::{
    ProfileRecord, SettingsProfile, WeekStart, at_limit, char_count, clip_to_limit, count_label,
    initials,
};
use rv2_kit::screens::settings_workspace_general::{
    SettingsWorkspaceGeneral, WorkspaceRecord, delete_matches, initial_letter,
};
use rv2_kit::screens::settings_workspace_security::{
    EmailDomain, IdpValues, SecurityValues, SessionLength, SettingsWorkspaceSecurity, any_verified,
    txt_record_for, valid_domain, valid_range,
};
use rv2_kit::screens::workspace_frame_flat::WorkspaceFrameFlat;
use rv2_kit::screens::workspace_frame_inset::{
    Artifact, ArtifactKind, ChatRow, FrameStyle, WidthRange, WorkspaceFrame, clamp_width,
    drag_width, group_chats, reset_width, resize_key,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

fn member(id: &str, name: &str, role: MemberRole, usage: f64, invited: bool) -> Member {
    Member {
        id: id.into(),
        display_name: Some(name.into()),
        email_address: format!("{}@example.com", name.to_lowercase()),
        member_role: role,
        plan_name: "Team".into(),
        usage_ratio: usage,
        is_invited: invited,
    }
}

fn members() -> Vec<Member> {
    vec![
        member("m1", "Ada", MemberRole::Admin, 0.42, false),
        member("m2", "Grace", MemberRole::Member, 0.8, false),
        member("m3", "Linus", MemberRole::Member, 0.15, false),
        member("m4", "Mira", MemberRole::Viewer, 0.0, true),
    ]
}

fn profile() -> ProfileRecord {
    ProfileRecord {
        first_name: "Decebal".into(),
        last_name: "Dobrica".into(),
        account_email: "decebal@example.com".into(),
        job_role: Some("Founder".into()),
        time_zone: "Europe/Bucharest".into(),
        week_start: WeekStart::Monday,
        personal_context: "I run a small product team.".into(),
        avatar_url: None,
        has_password: false,
    }
}

fn plan() -> CurrentPlan {
    CurrentPlan {
        name: "Team".into(),
        price_per_seat: "€18".into(),
        interval: BillInterval::Month,
        seats_used: 6,
        seats_total: 10,
        renews_on: "2026-12-01".into(),
        upcoming_amount: "€108".into(),
        breakdown: vec![BreakdownLine {
            label: "Seats".into(),
            amount: "€108".into(),
        }],
    }
}

fn plans() -> Vec<PlanCard> {
    vec![
        PlanCard {
            id: "team".into(),
            name: "Team".into(),
            monthly_per_seat: "€18".into(),
            yearly_per_seat: "€15".into(),
            features: vec!["Shared chats".into()],
            current: true,
            recommended: false,
        },
        PlanCard {
            id: "scale".into(),
            name: "Scale".into(),
            monthly_per_seat: "€30".into(),
            yearly_per_seat: "€25".into(),
            features: vec!["SSO".into()],
            current: false,
            recommended: true,
        },
    ]
}

fn billing_address() -> BillingAddress {
    BillingAddress {
        company: "Example Ltd".into(),
        line: "1 Market Street".into(),
        email: "billing@example.com".into(),
        vat_number: None,
    }
}

fn billing_app() -> Element {
    rsx! {
        SettingsBilling {
            current_plan: plan(),
            allowances: vec![Allowance { label: "Seats".into(), used: 6, total: 10, unit: "seats".into() }],
            billing_cycle: BillingCycle::Monthly,
            plans: plans(),
            add_ons: vec![AddOn {
                id: "storage".into(),
                name: "Storage".into(),
                description: "Extra files".into(),
                tiers: vec![AddOnTier { id: "s10".into(), label: "10 GB".into(), price: "€5".into(), current: true }],
            }],
            billing_address: billing_address(),
            card: CardInfo { brand: "Visa".into(), last4: "4242".into(), expires: "08/29".into(), is_default: true },
            invoices: vec![Invoice {
                reference: "INV-1001".into(),
                date: "2026-11-01".into(),
                amount: "€108".into(),
                status: InvoiceStatus::Paid,
            }],
        }
    }
}

fn billing_loading_app() -> Element {
    rsx! {
        SettingsBilling {
            current_plan: plan(),
            allowances: vec![],
            plans: plans(),
            add_ons: vec![],
            billing_address: billing_address(),
            card: CardInfo { brand: "Visa".into(), last4: "4242".into(), expires: "08/29".into(), is_default: true },
            invoices: vec![],
            is_loading: true,
        }
    }
}

fn billing_empty_app() -> Element {
    rsx! {
        SettingsBilling {
            current_plan: plan(),
            allowances: vec![],
            plans: plans(),
            add_ons: vec![],
            billing_address: billing_address(),
            card: CardInfo { brand: "Visa".into(), last4: "4242".into(), expires: "08/29".into(), is_default: false },
            invoices: vec![],
            is_empty: true,
        }
    }
}

fn members_app() -> Element {
    rsx! {
        SettingsMembersTeams {
            member_list: members(),
            team_list: vec![Team { id: "t1".into(), display_name: "Core".into(), member_ids: vec!["m1".into(), "m2".into()] }],
            seats: Seats { used: 4, total: 10 },
            current_user_id: "m1".to_string(),
        }
    }
}

fn members_loading_app() -> Element {
    rsx! {
        SettingsMembersTeams {
            member_list: members(),
            team_list: vec![],
            seats: Seats { used: 4, total: 10 },
            current_user_id: "m1".to_string(),
            is_loading: true,
        }
    }
}

fn models() -> Vec<ProviderEntry> {
    vec![
        ProviderEntry {
            id: "custom".into(),
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
            id: "global".into(),
            display_name: "Global Host".into(),
            hosting_region: HostingRegion::Global,
            logo_url: None,
            model_list: vec![],
        },
        ProviderEntry {
            id: "eu".into(),
            display_name: "Eurocloud".into(),
            hosting_region: HostingRegion::Eu,
            logo_url: None,
            model_list: vec![ModelEntry {
                id: "eu-chat".into(),
                display_name: "Euro Chat".into(),
                model_kind: ModelKind::Chat,
                is_enabled: true,
                access_level: AccessLevel::Engineering,
            }],
        },
    ]
}

fn models_app() -> Element {
    rsx! {
        SettingsModelAccess {
            provider_list: models(),
            default_chat: "auto".to_string(),
            default_image: "none".to_string(),
        }
    }
}

fn notification_values() -> (Digest, WeeklyReport, Vec<NotificationType>, QuietHours) {
    (
        Digest {
            is_enabled: false,
            send_time: "08:30".into(),
            time_zone: "Europe/Bucharest".into(),
            includes: vec![],
        },
        WeeklyReport {
            is_enabled: true,
            day: "Monday".into(),
        },
        vec![NotificationType {
            id: "reminders".into(),
            type_name: "Task reminders".into(),
            type_note: "Due today".into(),
            delivery: Delivery::Both,
        }],
        QuietHours {
            is_enabled: false,
            start_time: "22:00".into(),
            end_time: "07:00".into(),
        },
    )
}

fn notifications_app() -> Element {
    let (digest, weekly, types, quiet) = notification_values();
    rsx! {
        SettingsNotificationRules { digest, weekly_report: weekly, types, quiet_hours: quiet }
    }
}

fn preferences_app() -> Element {
    rsx! {
        SettingsPersonalPreferences {
            default_model: "auto".to_string(),
            default_image_model: "img-1".to_string(),
            models: vec![
                AllowedModel { id: "chat-1".into(), name: "Chat One".into(), kind: ModelKind::Chat },
                AllowedModel { id: "img-1".into(), name: "Image One".into(), kind: ModelKind::Image },
            ],
            chat_tools: vec![ChatTool { id: "memory".into(), label: "Memory".into(), description: "Keep notes".into(), is_enabled: false }],
            instructions: Instructions { is_enabled: true, about_text: "Founder".into(), response_style: String::new() },
            memory_on: false,
            is_empty: true,
        }
    }
}

fn privacy_app() -> Element {
    rsx! {
        SettingsPrivacyControls {
            ad_measurement: false,
            product_analytics: true,
            training_on_chats: false,
            cookies: CookieChoices { essential: true, optional: vec![] },
            product_email: true,
            sessions: vec![
                Session { id: "s-here".into(), device_label: "MacBook".into(), place: "Bucharest".into(), last_active: "now".into(), is_current: true },
                Session { id: "s-phone".into(), device_label: "iPhone".into(), place: "Cluj".into(), last_active: "1 day ago".into(), is_current: false },
            ],
            shared_chats: vec![SharedChat {
                id: "c1".into(),
                chat_title: "Roadmap".into(),
                visibility: Visibility::Workspace,
                shared_at: "2026-11-02".into(),
                view_count: 12,
                link: "https://example.com/c1".into(),
            }],
        }
    }
}

fn general_app() -> Element {
    rsx! {
        SettingsWorkspaceGeneral {
            workspace_record: WorkspaceRecord {
                display_name: "Northwind".into(),
                url_slug: "northwind".into(),
                company_context: "We build planning tools.".into(),
                icon_url: None,
            },
        }
    }
}

fn security_app() -> Element {
    rsx! {
        SettingsWorkspaceSecurity {
            email_domains: vec![EmailDomain { name: "pending.example".into(), verified: false, txt_record: "rv2-verification=pending.example".into() }],
            join_by_domain: false,
            require_sso: false,
            two_step_required: true,
            session_length: SessionLength::SevenDays,
            idp: IdpValues {
                audience_uri: "urn:rv2:aud".into(),
                acs_url: "https://rv2.example/acs".into(),
                entity_id: "urn:rv2:sp".into(),
                metadata_url: String::new(),
            },
            ip_ranges: vec![],
        }
    }
}

fn frame_app() -> Element {
    rsx! {
        WorkspaceFrame {
            content: rsx! { p { "Page body" } },
            current_item: "Folders".to_string(),
            workspace_name: "Northwind".to_string(),
            chat_list: vec![ChatRow { id: "c1".into(), title: "Pricing review".into(), pinned: true, unread: false, project_id: None, section: None }],
            side_cards: false,
        }
    }
}

fn flat_app() -> Element {
    rsx! {
        WorkspaceFrameFlat {
            content: rsx! { p { "Flat body" } },
            top_bar: false,
        }
    }
}

fn artifact_of(kind: ArtifactKind) -> Artifact {
    let (name, body) = match kind {
        ArtifactKind::Document => ("Brief.md", "First paragraph.\n\nSecond paragraph."),
        ArtifactKind::Sheet => ("Budget.csv", "Item, Qty\nTea, 2"),
        ArtifactKind::Folder => ("Specs", "alpha.md\nbeta.md"),
        ArtifactKind::Code => ("main.rs", "fn main() {}"),
        ArtifactKind::Image => ("chart.png", "https://example.test/chart.png"),
    };
    Artifact {
        name: name.into(),
        kind,
        body: body.into(),
    }
}

#[component]
fn ArtifactTabHost(kind: ArtifactKind) -> Element {
    rsx! {
        WorkspaceFrame {
            content: rsx! { p { "Page body" } },
            work_area: true,
            initial_tabs: vec![artifact_of(kind)],
            side_cards: false,
        }
    }
}

fn render_artifact_tab(kind: ArtifactKind) -> String {
    let mut dom = VirtualDom::new_with_props(ArtifactTabHost, ArtifactTabHostProps { kind });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn flat_artifact_app() -> Element {
    rsx! {
        WorkspaceFrameFlat {
            content: rsx! { p { "Flat page" } },
            work_area: true,
            initial_tabs: vec![artifact_of(ArtifactKind::Sheet)],
        }
    }
}

fn security_verified_app() -> Element {
    rsx! {
        SettingsWorkspaceSecurity {
            email_domains: vec![EmailDomain { name: "company.example".into(), verified: true, txt_record: String::new() }],
            join_by_domain: true,
            require_sso: false,
            two_step_required: true,
            session_length: SessionLength::SevenDays,
            idp: IdpValues {
                audience_uri: String::new(),
                acs_url: String::new(),
                entity_id: String::new(),
                metadata_url: String::new(),
            },
            ip_ranges: vec![],
        }
    }
}

fn profile_with_docs_app() -> Element {
    rsx! {
        SettingsProfile {
            profile_record: profile(),
            role_options: vec![],
            timezone_options: vec![],
            docs_href: Some("https://docs.example.test/settings/profile".to_string()),
        }
    }
}

fn profile_without_docs_app() -> Element {
    rsx! { SettingsProfile { profile_record: profile(), role_options: vec![], timezone_options: vec![] } }
}

#[test]
fn profile_counts_and_limits_follow_the_spec() {
    assert_eq!(char_count("héllo"), 5);
    assert_eq!(count_label(12, 500), "12 / 500");
    assert!(at_limit(500, 500));
    assert!(!at_limit(499, 500));
    assert_eq!(clip_to_limit("abcdef", 3), "abc");
    assert_eq!(initials("ada", "lovelace"), "AL");
    assert_eq!(initials("ada", ""), "A");
    assert_eq!(WeekStart::Sunday.as_str(), "sunday");
}

#[test]
fn profile_renders_its_fields_and_password_note() {
    fn app() -> Element {
        rsx! { SettingsProfile { profile_record: profile(), role_options: names(&["Founder"]), timezone_options: names(&["Europe/Bucharest"]) } }
    }
    let html = render(app);
    assert!(html.contains("Decebal"));
    assert!(html.contains("Add password"));
    assert!(html.contains("role=\"status\""));
    assert!(html.contains("/ 500"));
}

#[test]
fn profile_loading_shows_shapes_and_no_figures() {
    fn app() -> Element {
        rsx! { SettingsProfile { profile_record: profile(), role_options: vec![], timezone_options: vec![], is_loading: true } }
    }
    let html = render(app);
    assert!(html.contains("aria-busy=\"true\""));
    assert!(html.contains("kit-set__skel"));
    assert!(!html.contains("Decebal"));
}

#[test]
fn billing_allowance_text_and_share_are_plain() {
    let seats = Allowance {
        label: "Seats".into(),
        used: 6,
        total: 10,
        unit: "seats".into(),
    };
    assert_eq!(allowance_text(&seats), "6 of 10 seats");
    assert_eq!(allowance_share(6, 10), 60.0);
    assert_eq!(allowance_share(12, 10), 100.0);
    assert_eq!(allowance_share(1, 0), 0.0);
    assert_eq!(plan_price(&plans()[0], BillingCycle::Yearly), "€15");
    assert_eq!(plan_action_label(&plans()[0]), "Current plan");
    assert_eq!(plan_action_label(&plans()[1]), "Choose Scale");
}

#[test]
fn billing_renders_current_plan_disabled_and_meters_as_text() {
    let html = render(billing_app);
    assert!(html.contains("6 of 10 seats"));
    assert!(html.contains("Current plan"));
    assert!(html.contains("Manage members"));
    assert!(html.contains("INV-1001"));
    assert!(html.contains("Cancel subscription"));
}

#[test]
fn billing_loading_hides_every_figure() {
    let html = render(billing_loading_app);
    assert!(html.contains("aria-busy=\"true\""));
    assert!(!html.contains("Next bill"));
    assert!(!html.contains("Current plan"));
}

#[test]
fn billing_empty_history_offers_the_first_bill_preview() {
    let html = render(billing_empty_app);
    assert!(html.contains("No invoices yet"));
    assert!(html.contains("Preview first bill"));
}

#[test]
fn members_roles_and_filters_match_the_spec() {
    let all = members();
    assert_eq!(filter_members(&all, RoleFilter::Member, "").len(), 2);
    assert_eq!(filter_members(&all, RoleFilter::All, "mira").len(), 1);
    assert_eq!(
        filter_members(&all, RoleFilter::All, "example.com").len(),
        4
    );
    let order = names(&["m1", "m2", "m3", "m4"]);
    assert_eq!(
        range_between(&order, "m2", "m4"),
        names(&["m2", "m3", "m4"])
    );
    assert_eq!(
        range_between(&order, "m4", "m2"),
        names(&["m2", "m3", "m4"])
    );
    assert_eq!(header_state(2, 4), HeaderState::Some);
    assert_eq!(header_state(4, 4), HeaderState::All);
    assert_eq!(header_state(0, 4), HeaderState::None);
    assert_eq!(count_words(2), "two");
    assert_eq!(count_words(12), "12");
    assert_eq!(usage_percent(0.426), 43);
}

#[test]
fn members_renders_you_disabled_and_teams() {
    let html = render(members_app);
    assert!(html.contains("(You)"));
    assert!(html.contains("Role for Ada. Disabled: you cannot change your own role."));
    assert!(html.contains("Invited"));
    assert!(html.contains("role=\"tab\""));
}

#[test]
fn members_loading_shows_shapes() {
    let html = render(members_loading_app);
    assert!(html.contains("aria-busy=\"true\""));
    assert!(!html.contains("Grace"));
}

#[test]
fn model_fallbacks_follow_the_spec() {
    assert_eq!(chat_fallback("lab-1", "lab-1"), "auto");
    assert_eq!(chat_fallback("eu-chat", "lab-1"), "eu-chat");
    let list = models();
    assert_eq!(image_fallback(&list, "none", "none"), "none");
    let mut with_image = list.clone();
    with_image[0].model_list.push(ModelEntry {
        id: "img-a".into(),
        display_name: "Picture A".into(),
        model_kind: ModelKind::Image,
        is_enabled: true,
        access_level: AccessLevel::Everyone,
    });
    assert_eq!(image_fallback(&with_image, "img-b", "img-b"), "img-a");
}

#[test]
fn model_providers_order_eu_first_then_global_then_custom() {
    let sorted: Vec<String> = ordered(&models()).into_iter().map(|p| p.id).collect();
    assert_eq!(sorted, names(&["eu", "global", "custom"]));
    assert!(provider_matches(&models()[0], "lab one"));
    assert!(!provider_matches(&models()[0], "euro"));
    assert_eq!(chat_choices(&models()).len(), 2);
    assert!(image_choices(&models()).is_empty());
}

#[test]
fn model_access_renders_auto_explainer_and_key_masking_markup() {
    let html = render(models_app);
    assert!(html.contains("data-testid=\"auto-explainer\""));
    assert!(html.contains("Lab endpoint"));
    assert!(html.find("Eurocloud").unwrap() < html.find("Lab endpoint").unwrap());
}

#[test]
fn notification_channels_and_set_all_follow_the_spec() {
    assert_eq!(
        channel_label("Task reminders"),
        "Channel for Task reminders"
    );
    assert_eq!(Delivery::parse("email"), Delivery::Email);
    let (digest, weekly, types, quiet) = notification_values();
    let mut values = NotificationValues {
        digest,
        weekly_report: weekly,
        types,
        quiet_hours: quiet,
    };
    set_all(&mut values, Delivery::Inbox);
    assert!(values.types.iter().all(|t| t.delivery == Delivery::Inbox));
}

#[test]
fn notifications_hide_dependent_rows_while_switched_off() {
    let html = render(notifications_app);
    assert!(!html.contains("Send time"));
    assert!(!html.contains("From"));
    assert!(html.contains("Channel for Task reminders"));
}

#[test]
fn preferences_limit_and_choice_rules() {
    assert_eq!(
        clip_instruction(&"x".repeat(INSTRUCTION_LIMIT + 10))
            .chars()
            .count(),
        INSTRUCTION_LIMIT
    );
    let allowed = vec![AllowedModel {
        id: "chat-1".into(),
        name: "Chat One".into(),
        kind: ModelKind::Chat,
    }];
    assert!(choosable(&allowed, "chat-1", ModelKind::Chat));
    assert!(!choosable(&allowed, "chat-1", ModelKind::Image));
    assert!(!choosable(&allowed, "chat-9", ModelKind::Chat));
    assert_eq!(Theme::parse("dark"), Theme::Dark);
}

#[test]
fn preferences_memory_off_and_empty_states_render() {
    let html = render(preferences_app);
    assert!(html.contains("Nothing is remembered yet"));
    assert!(html.contains("Memory is off"));
    assert!(!html.contains("Switch to Auto"));
}

#[test]
fn privacy_scope_and_empty_messages() {
    let chat = SharedChat {
        id: "c".into(),
        chat_title: "t".into(),
        visibility: Visibility::Public,
        shared_at: "d".into(),
        view_count: 0,
        link: "l".into(),
    };
    assert!(in_scope(&chat, Visibility::Public));
    assert!(!in_scope(&chat, Visibility::Workspace));
    assert_eq!(
        empty_scope_message(Visibility::Public),
        "No public links are open."
    );
}

#[test]
fn privacy_marks_the_current_device_and_offers_sign_out_elsewhere() {
    let html = render(privacy_app);
    assert!(html.contains("This device"));
    assert!(html.contains("aria-label=\"Sign out iPhone\""));
    assert!(!html.contains("aria-label=\"Sign out MacBook\""));
    assert!(html.contains("Copy link to Roadmap"));
}

#[test]
fn general_helpers_and_delete_gate() {
    assert_eq!(initial_letter("northwind"), "N");
    assert!(delete_matches("Northwind", "Northwind"));
    assert!(!delete_matches("northwind", "Northwind"));
    assert!(!delete_matches("", ""));
}

#[test]
fn general_renders_initial_letter_and_context_count() {
    let html = render(general_app);
    assert!(html.contains("data-initials=\"N\""));
    assert!(html.contains("24 / 2000"));
}

#[test]
fn security_domain_and_range_validation() {
    assert!(valid_domain("company.com"));
    assert!(!valid_domain("company"));
    assert!(!valid_domain("bad domain.com"));
    assert!(valid_range("10.0.0.0/24"));
    assert!(valid_range("192.168.1.1"));
    assert!(!valid_range("10.0.0/24"));
    assert!(!valid_range("10.0.0.0/33"));
    assert_eq!(
        txt_record_for("Company.com"),
        "rv2-verification=company.com"
    );
    assert!(!any_verified(&[EmailDomain {
        name: "a.b".into(),
        verified: false,
        txt_record: String::new()
    }]));
}

#[test]
fn security_disables_join_and_sso_until_a_domain_is_verified() {
    let html = render(security_app);
    assert!(html.contains("Unavailable until a domain is verified."));
    assert!(html.contains("Add this TXT record"));
    assert!(html.contains("Every network can connect until you add a range."));
    assert!(html.contains("Copy ACS URL"));
}

#[test]
fn security_values_roundtrip_through_its_struct() {
    let value = SecurityValues {
        email_domains: vec![],
        join_by_domain: false,
        require_sso: false,
        two_step_required: false,
        session_length: SessionLength::parse("30d"),
        idp: IdpValues {
            audience_uri: String::new(),
            acs_url: String::new(),
            entity_id: String::new(),
            metadata_url: String::new(),
        },
        ip_ranges: vec![],
    };
    assert_eq!(value.session_length.as_str(), "30d");
}

#[test]
fn frame_groups_pinned_then_recent_then_projects() {
    let chats = vec![
        ChatRow {
            id: "a".into(),
            title: "A".into(),
            pinned: true,
            unread: false,
            project_id: None,
            section: None,
        },
        ChatRow {
            id: "b".into(),
            title: "B".into(),
            pinned: false,
            unread: true,
            project_id: Some("Ops".into()),
            section: None,
        },
        ChatRow {
            id: "c".into(),
            title: "C".into(),
            pinned: false,
            unread: false,
            project_id: None,
            section: None,
        },
    ];
    let groups = group_chats(&chats, &names(&["Ops"]));
    assert_eq!(groups.pinned.len(), 1);
    assert_eq!(groups.recent.len(), 1);
    assert_eq!(groups.by_project[0].1.len(), 1);
}

#[test]
fn frame_widths_clamp_drag_keys_and_reset() {
    let range = WidthRange::new(264.0, 200.0, 420.0);
    assert_eq!(clamp_width(999.0, 200.0, 420.0), 420.0);
    assert_eq!(drag_width(264.0, 100.0, 40.0, range, false), 204.0);
    assert_eq!(drag_width(264.0, 100.0, -200.0, range, false), 200.0);
    assert_eq!(drag_width(264.0, 100.0, 1000.0, range, false), 420.0);
    assert_eq!(
        drag_width(
            340.0,
            500.0,
            400.0,
            WidthRange::new(340.0, 260.0, 560.0),
            true
        ),
        440.0
    );
    assert_eq!(
        resize_key(264.0, "ArrowRight", false, "ArrowRight", range),
        Some(280.0)
    );
    assert_eq!(
        resize_key(264.0, "ArrowRight", true, "ArrowRight", range),
        Some(312.0)
    );
    assert_eq!(
        resize_key(264.0, "Home", false, "ArrowRight", range),
        Some(200.0)
    );
    assert_eq!(
        resize_key(264.0, "End", false, "ArrowRight", range),
        Some(420.0)
    );
    assert_eq!(resize_key(264.0, "a", false, "ArrowRight", range), None);
    assert_eq!(reset_width(range), 264.0);
    assert_eq!(FrameStyle::Flat.as_str(), "flat");
}

#[test]
fn inset_frame_respects_side_cards_and_top_bar_flags() {
    let html = render(frame_app);
    assert!(html.contains("data-style=\"inset\""));
    assert!(!html.contains("kit-frame__card"));
    assert!(html.contains("aria-current=\"true\""));
    assert!(html.contains("Pricing review"));
    assert!(html.contains("Northwind"));
}

#[test]
fn flat_frame_without_top_bar_has_no_header() {
    let html = render(flat_app);
    assert!(html.contains("data-style=\"flat\""));
    assert!(!html.contains("kit-frame__bar"));
    assert!(html.contains("Flat body"));
}

#[test]
fn document_artifact_tab_renders_paragraphs_in_its_viewer() {
    let html = render_artifact_tab(ArtifactKind::Document);
    assert!(html.contains("data-kind=\"document\""));
    assert!(html.contains("First paragraph."));
    assert!(html.contains("Second paragraph."));
    assert!(html.contains("aria-label=\"Brief.md\""));
}

#[test]
fn sheet_artifact_tab_renders_rows_as_table_cells() {
    let html = render_artifact_tab(ArtifactKind::Sheet);
    assert!(html.contains("data-kind=\"sheet\""));
    assert!(html.contains("<td>Tea</td>"));
    assert!(html.contains("<td>2</td>"));
}

#[test]
fn folder_artifact_tab_lists_each_entry() {
    let html = render_artifact_tab(ArtifactKind::Folder);
    assert!(html.contains("data-kind=\"folder\""));
    assert!(html.contains("alpha.md"));
    assert!(html.contains("beta.md"));
}

#[test]
fn code_artifact_tab_renders_source_in_a_code_block() {
    let html = render_artifact_tab(ArtifactKind::Code);
    assert!(html.contains("data-kind=\"code\""));
    assert!(html.contains("<pre class=\"kit-frame__code\""));
    assert!(html.contains("fn main() {}"));
}

#[test]
fn image_artifact_tab_names_its_picture_as_alt_text() {
    let html = render_artifact_tab(ArtifactKind::Image);
    assert!(html.contains("data-kind=\"image\""));
    assert!(html.contains("src=\"https://example.test/chart.png\""));
    assert!(html.contains("alt=\"chart.png\""));
}

#[test]
fn artifact_tab_is_the_active_tab_at_mount() {
    let html = render_artifact_tab(ArtifactKind::Sheet);
    assert!(html.contains("aria-selected=\"true\""));
    assert!(html.contains("aria-label=\"Close Budget.csv\""));
    assert!(html.contains("data-active=\"true\""));
}

#[test]
fn flat_frame_opens_artifact_tabs_it_is_given() {
    let html = render(flat_artifact_app);
    assert!(html.contains("data-style=\"flat\""));
    assert!(html.contains("data-kind=\"sheet\""));
    assert!(html.contains("Flat page"));
}

#[test]
fn flat_frame_header_shows_default_share_and_more_actions() {
    fn app() -> Element {
        rsx! { WorkspaceFrameFlat { content: rsx! { p { "Body" } }, current_item: "Folders".to_string() } }
    }
    let html = render(app);
    assert!(html.contains("aria-label=\"Share\""));
    assert!(html.contains("aria-label=\"More actions for Folders\""));
}

#[test]
fn security_verified_domain_shows_verified_tag_and_no_verify_button() {
    let html = render(security_verified_app);
    assert!(html.contains("data-status=\"verified\""));
    assert!(html.contains("data-verified=\"true\""));
    assert!(!html.contains("aria-label=\"Verify company.example\""));
    assert!(html.contains("aria-label=\"Remove company.example\""));
}

#[test]
fn security_pending_domain_offers_verify_by_name_and_is_not_busy() {
    let html = render(security_app);
    assert!(html.contains("data-status=\"pending\""));
    assert!(html.contains("aria-label=\"Verify pending.example\""));
    assert!(html.contains(">Verify</button>"));
    assert!(!html.contains("aria-busy"));
    assert!(!html.contains("Checking…"));
}

#[test]
fn general_renders_an_icon_upload_input_behind_upload_icon_label() {
    let html = render(general_app);
    assert!(html.contains("Upload icon"));
    assert!(html.contains("type=\"file\""));
    assert!(html.contains("accept=\"image/*\""));
}

#[test]
fn members_row_more_buttons_are_named_for_each_member() {
    let html = render(members_app);
    assert!(html.contains("aria-label=\"Actions for Ada\""));
    assert!(html.contains("aria-label=\"Actions for Grace\""));
    assert!(html.contains("aria-label=\"Actions for Mira\""));
}

#[test]
fn settings_shell_has_no_docs_link_without_a_host_href() {
    let html = render(profile_without_docs_app);
    assert!(!html.contains("Documentation for"));
    assert!(!html.contains("/docs/settings/"));
}

#[test]
fn settings_shell_links_to_the_host_docs_href_when_given() {
    let html = render(profile_with_docs_app);
    assert!(html.contains("href=\"https://docs.example.test/settings/profile\""));
    assert!(html.contains("Documentation for Profile"));
}
