use dioxus::prelude::*;
use rv2_kit::screens::settings_model_access::{
    AccessLevel, ApiKey, HostingRegion, ModelEntry, ModelKind, ProviderEntry, SettingsModelAccess,
    image_choices,
};

const CSS: &str = include_str!("../assets/settings-model-access.css");

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

fn model(id: &str, name: &str, kind: ModelKind, enabled: bool, access: AccessLevel) -> ModelEntry {
    ModelEntry {
        id: id.into(),
        display_name: name.into(),
        model_kind: kind,
        is_enabled: enabled,
        access_level: access,
    }
}

fn rest_providers() -> Vec<ProviderEntry> {
    vec![
        ProviderEntry {
            id: "lab-endpoint".into(),
            display_name: "Lab endpoint".into(),
            hosting_region: HostingRegion::Custom,
            logo_url: None,
            model_list: vec![model(
                "lab-1",
                "Lab One",
                ModelKind::Chat,
                true,
                AccessLevel::Everyone,
            )],
        },
        ProviderEntry {
            id: "global-host".into(),
            display_name: "Global Host".into(),
            hosting_region: HostingRegion::Global,
            logo_url: None,
            model_list: vec![model(
                "gl-chat",
                "Global Chat",
                ModelKind::Chat,
                true,
                AccessLevel::Everyone,
            )],
        },
        ProviderEntry {
            id: "nordhost".into(),
            display_name: "Nordhost".into(),
            hosting_region: HostingRegion::Eu,
            logo_url: None,
            model_list: vec![model(
                "nord-chat",
                "Nord Chat",
                ModelKind::Chat,
                false,
                AccessLevel::Everyone,
            )],
        },
        ProviderEntry {
            id: "eurocloud".into(),
            display_name: "Eurocloud".into(),
            hosting_region: HostingRegion::Eu,
            logo_url: None,
            model_list: vec![
                model(
                    "eu-chat",
                    "Euro Chat",
                    ModelKind::Chat,
                    true,
                    AccessLevel::Everyone,
                ),
                model(
                    "eu-write",
                    "Euro Writer",
                    ModelKind::Chat,
                    true,
                    AccessLevel::Engineering,
                ),
                model(
                    "eu-img",
                    "Euro Picture",
                    ModelKind::Image,
                    true,
                    AccessLevel::Admins,
                ),
            ],
        },
    ]
}

fn rest_app() -> Element {
    rsx! {
        SettingsModelAccess {
            provider_list: rest_providers(),
            default_chat: "auto".to_string(),
            default_image: "eu-img".to_string(),
            open_provider: Some("eurocloud".to_string()),
            api_keys: vec![
                ApiKey { provider_id: "eurocloud".into(), masked_value: "sk-…4c2a".into(), added_at: "3 Nov 2026".into() },
                ApiKey { provider_id: "global-host".into(), masked_value: "sk-…9f1e".into(), added_at: "18 Oct 2026".into() },
            ],
        }
    }
}

#[test]
fn rest_state_expands_one_provider_and_shows_its_three_model_rows() {
    let html = render(rest_app);
    assert_eq!(html.matches("data-open=\"true\"").count(), 1);
    assert_eq!(html.matches("aria-expanded=\"true\"").count(), 1);
    assert_eq!(html.matches("kit-mdl__model\"").count(), 3);
    assert_eq!(html.matches("kit-mdl__access").count(), 3);
    assert!(html.contains(">Everyone</option>"));
    assert!(html.contains(">Engineering</option>"));
    assert!(html.contains(">Admins</option>"));
}

#[test]
fn provider_list_at_rest_is_eu_eu_global_custom() {
    let html = render(rest_app);
    assert_eq!(html.matches("kit-mdl__provider\"").count(), 4);
    let at = |name: &str| html.find(name).unwrap_or_else(|| panic!("no {name}"));
    assert!(at("Eurocloud") < at("Nordhost"));
    assert!(at("Nordhost") < at("Global Host"));
    assert!(at("Global Host") < at("Lab endpoint"));
}

#[test]
fn auto_explainer_is_open_and_the_image_selector_offers_one_model() {
    let html = render(rest_app);
    assert!(html.contains("data-testid=\"auto-explainer\""));
    assert_eq!(image_choices(&rest_providers()).len(), 1);
}

#[test]
fn api_keys_at_rest_are_two_masked_keys_with_added_dates() {
    let html = render(rest_app);
    assert_eq!(html.matches("kit-mdl__key\"").count(), 2);
    assert!(html.contains("sk-…4c2a"));
    assert!(html.contains("Added 3 Nov 2026"));
    assert!(html.contains("Add API key"));
}

#[test]
fn filter_tabs_read_all_chat_image_custom_and_the_custom_model_button_is_present() {
    let html = render(rest_app);
    assert_eq!(html.matches("role=\"tab\"").count(), 4);
    assert!(html.contains(">All</button>"));
    assert!(html.contains(">Chat</button>"));
    assert!(html.contains(">Image</button>"));
    assert!(html.contains(">Custom</button>"));
    assert!(html.contains("Add custom model"));
}

#[test]
fn rolling_list_beside_auto_is_static_until_the_pointer_is_over_it() {
    let track = rule(CSS, ".kit-mdl__rolling-track");
    assert!(!track.contains("animation"));
    let hover = rule(CSS, ".kit-mdl__rolling:hover .kit-mdl__rolling-track");
    assert!(hover.contains("kit-mdl-roll"));
}
