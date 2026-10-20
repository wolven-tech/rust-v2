//! The instructions added by `docs/architecture/003-ui-semantic-isa.md`,
//! rendered.
//!
//! Each of these replaced markup that apps had been writing by hand, and each
//! hand-written copy had a defect the component exists to remove. These tests
//! pin the removal, so the next edit cannot quietly put the defect back.

use dioxus::prelude::*;
use rv2_ui::{Input, LinkButton, NavLink, ProductHeader, TextField, TextLink, Variant};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The opening tag carrying `attribute`, from `<` to `>`.
fn tag_with<'a>(html: &'a str, attribute: &str) -> &'a str {
    let at = html
        .find(attribute)
        .unwrap_or_else(|| panic!("no element carries {attribute}: {html}"));
    let start = html[..at]
        .rfind('<')
        .expect("an attribute sits inside a tag");
    let end = at + html[at..].find('>').expect("a tag closes");
    &html[start..=end]
}

#[test]
fn an_external_text_link_cannot_reach_back_through_window_opener() {
    let html = render(|| {
        rsx! {
            TextLink { href: "https://example.com", external: true, "Elsewhere" }
        }
    });
    assert!(html.contains(r#"target="_blank""#), "{html}");
    assert!(html.contains(r#"rel="noopener noreferrer""#), "{html}");
}

#[test]
fn an_internal_text_link_opens_in_place() {
    let html = render(|| {
        rsx! {
            TextLink { href: "/about", "About" }
        }
    });
    assert!(!html.contains("target="), "{html}");
}

/// The export link this replaced was `pointer-events: none`, which stops a
/// mouse and nothing else — it stayed in the tab order and Enter followed it.
#[test]
fn a_disabled_link_button_has_no_href_to_follow() {
    let html = render(|| {
        rsx! {
            LinkButton { href: "data:text/csv,a", disabled: true, download: "x.csv", "Export" }
        }
    });
    let tag = tag_with(&html, "aria-disabled");
    assert!(!tag.contains("href="), "{tag}");
    assert!(tag.contains(r#"aria-disabled="true""#), "{tag}");
}

#[test]
fn an_enabled_link_button_downloads_what_it_names() {
    let html = render(|| {
        rsx! {
            LinkButton {
                href: "data:text/csv,a",
                variant: Variant::Outline,
                download: "x.csv",
                "Export"
            }
        }
    });
    assert!(html.contains(r#"href="data:text/csv,a""#), "{html}");
    assert!(html.contains(r#"download="x.csv""#), "{html}");
    assert!(!html.contains("aria-disabled"), "{html}");
}

/// Two of the four hand-made inputs this replaced had no focus ring.
#[test]
fn a_bare_input_and_a_text_field_both_carry_the_focus_ring() {
    let html = render(|| {
        rsx! {
            Input { id: "q", value: "", oninput: |_| {} }
            TextField { label: "Email", value: "", oninput: |_| {} }
        }
    });
    assert_eq!(
        html.matches("focus:ring-2 focus:ring-accent").count(),
        2,
        "{html}"
    );
}

/// A nameless input contributes nothing to a native form submission, which is
/// half of how the login screen once shipped unable to submit.
#[test]
fn a_named_field_renders_its_name_and_an_unnamed_one_renders_none() {
    let named = render(|| {
        rsx! {
            TextField { label: "Email", name: "email", value: "", oninput: |_| {} }
        }
    });
    assert!(named.contains(r#"name="email""#), "{named}");

    let unnamed = render(|| {
        rsx! {
            Input { value: "", oninput: |_| {} }
        }
    });
    assert!(!unnamed.contains("name="), "{unnamed}");
}

#[test]
fn only_the_route_whose_key_is_current_is_marked_as_the_page() {
    let html = render(|| {
        rsx! {
            ProductHeader {
                brand: "Brand",
                items: vec![NavLink::route("home", "Home", "/"), NavLink::route("guide", "Guide", "/guide")],
                current: "guide",
                action: NavLink::new("Open app", "https://app.example.com").external(),
            }
        }
    });
    // Desktop and mobile navigation each mark it once.
    assert_eq!(html.matches(r#"aria-current="page""#).count(), 2, "{html}");
    assert!(
        tag_with(&html, r#"href="/guide""#).contains("aria-current"),
        "{html}"
    );
}
