use dioxus::prelude::*;
use rv2_kit::components::connector_settings_card::{ConnectorSettingsCard, ConnectorValues};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn card_without_key() -> Element {
    rsx! {
        ConnectorSettingsCard {
            service_name: "Storefront",
            initial_values: ConnectorValues { is_enabled: true, ..Default::default() },
        }
    }
}

fn card_with_key() -> Element {
    rsx! {
        ConnectorSettingsCard {
            service_name: "Storefront",
            initial_values: ConnectorValues {
                is_enabled: true,
                api_key: "rv2_demo_0000000000000abc".into(),
                ..Default::default()
            },
        }
    }
}

#[test]
fn key_field_without_a_key_says_no_key_yet_and_hints_how_to_get_one() {
    let html = render(card_without_key);
    assert!(html.contains("placeholder=\"No key yet\""));
    assert!(html.contains("Generate a key to link this service."));
}

#[test]
fn key_field_with_a_key_drops_the_empty_state_hint() {
    let html = render(card_with_key);
    assert!(!html.contains("Generate a key to link this service."));
}
