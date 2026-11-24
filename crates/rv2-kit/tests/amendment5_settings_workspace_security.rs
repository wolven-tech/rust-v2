use dioxus::prelude::*;
use rv2_kit::screens::settings_workspace_security::{
    EmailDomain, IdpValues, IpRange, SessionLength, SettingsWorkspaceSecurity, any_verified,
};

const CSS: &str = include_str!("../assets/settings-workspace-security.css");

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The CSS block for one selector, from its opening brace to the closing brace.
fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
    let start = css
        .find(&format!("{selector} {{"))
        .unwrap_or_else(|| panic!("no rule for {selector}"));
    let end = css[start..]
        .find('}')
        .map(|i| start + i)
        .unwrap_or(css.len());
    &css[start..end]
}

fn rest_domains() -> Vec<EmailDomain> {
    vec![
        EmailDomain {
            name: "northwind.example".into(),
            verified: true,
            txt_record: "rv2-verification=northwind.example".into(),
        },
        EmailDomain {
            name: "agency.example".into(),
            verified: false,
            txt_record: "rv2-verification=agency.example".into(),
        },
    ]
}

fn rest_app() -> Element {
    rsx! {
        SettingsWorkspaceSecurity {
            email_domains: rest_domains(),
            join_by_domain: true,
            require_sso: true,
            two_step_required: true,
            session_length: SessionLength::SevenDays,
            idp: IdpValues {
                audience_uri: "urn:rv2:northwind".into(),
                acs_url: "https://rv2.example/sso/acs".into(),
                entity_id: "urn:rv2:northwind:sp".into(),
                metadata_url: String::new(),
            },
            ip_ranges: vec![
                IpRange { id: "r1".into(), range: "10.0.0.0/24".into(), label: None },
                IpRange { id: "r2".into(), range: "192.168.10.0/24".into(), label: None },
            ],
        }
    }
}

#[test]
fn rest_page_stacks_five_cards_in_the_specified_order() {
    let html = render(rest_app);
    let at = |heading: &str| {
        html.find(heading)
            .unwrap_or_else(|| panic!("no card {heading}"))
    };
    assert!(at("Email domains") < at("Sign-in"));
    assert!(at("Sign-in") < at("Identity provider"));
    assert!(at("Identity provider") < at("IP allowlist"));
    assert!(at("IP allowlist") < at("Danger zone"));
}

#[test]
fn sign_in_card_shows_three_switches_and_the_session_length_value() {
    let html = render(rest_app);
    assert_eq!(html.matches("role=\"switch\"").count(), 3);
    assert!(html.contains("aria-label=\"Session length\""));
    assert!(html.contains(">7 days</option>"));
}

#[test]
fn sso_is_enabled_at_rest_because_a_domain_is_verified() {
    assert!(any_verified(&rest_domains()));
    let html = render(rest_app);
    assert!(!html.contains("Unavailable until a domain is verified."));
}

#[test]
fn identity_provider_card_shows_three_copy_rows_and_an_empty_metadata_field() {
    let html = render(rest_app);
    assert_eq!(html.matches("kit-sec__copy\"").count(), 4);
    assert!(html.contains("aria-label=\"Copy Audience URI\""));
    assert!(html.contains("aria-label=\"Copy SP entity ID\""));
    assert!(html.contains("aria-label=\"Copy ACS URL\""));
    assert!(html.contains("placeholder=\"https://login.example.com/metadata\""));
}

#[test]
fn email_domains_show_one_verified_and_one_pending_with_its_verify_control_and_record() {
    let html = render(rest_app);
    assert_eq!(html.matches("data-status=\"verified\"").count(), 1);
    assert_eq!(html.matches("data-status=\"pending\"").count(), 1);
    assert!(html.contains("aria-label=\"Verify agency.example\""));
    assert!(html.contains("rv2-verification=agency.example"));
    assert!(!html.contains("aria-label=\"Verify northwind.example\""));
}

#[test]
fn ip_allowlist_shows_both_ranges_with_their_remove_controls() {
    let html = render(rest_app);
    assert_eq!(html.matches("kit-sec__range\"").count(), 2);
    assert!(html.contains("aria-label=\"Remove 10.0.0.0/24\""));
    assert!(html.contains("aria-label=\"Remove 192.168.10.0/24\""));
}

#[test]
fn range_remove_controls_are_hidden_until_hover_or_keyboard_focus() {
    let remove = rule(CSS, "\n.kit-sec__remove");
    assert!(remove.contains("opacity: 0"));
    let shown = rule(
        CSS,
        ".kit-sec__range:hover .kit-sec__remove,\n.kit-sec__remove:focus",
    );
    assert!(shown.contains("opacity: 1"));
    assert!(CSS.contains(".kit-sec__range:hover .kit-sec__remove"));
    assert!(CSS.contains(".kit-sec__remove:focus {"));
}
