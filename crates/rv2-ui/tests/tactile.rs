//! `rv2_ui::tactile`, rendered. Each test pins an accessibility or behaviour
//! contract the CSS cannot carry on its own.

use dioxus::prelude::*;
use rv2_ui::{
    Chip, ChipGroup, Masthead, MediaCard, NavLink, Press, PressButton, PressLink, SaveToggle,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn a_press_icon_is_hidden_from_assistive_technology() {
    let html = render(|| {
        rsx! {
            PressButton { icon: "/star.png", "Find an activity" }
        }
    });
    assert!(html.contains(r#"alt="""#), "{html}");
    assert!(html.contains(r#"aria-hidden="true""#), "{html}");
}

#[test]
fn the_two_press_treatments_render_different_classes() {
    let primary = render(|| rsx! { PressButton { "Go" } });
    let soft = render(|| rsx! { PressButton { press: Press::Soft, "Go" } });
    assert!(primary.contains("rv2-press--primary"), "{primary}");
    assert!(soft.contains("rv2-press--soft"), "{soft}");
}

#[test]
fn an_external_press_link_cannot_reach_back_through_window_opener() {
    let html = render(|| {
        rsx! {
            PressLink { href: "https://example.com", external: true, "Elsewhere" }
        }
    });
    assert!(html.contains(r#"target="_blank""#), "{html}");
    assert!(html.contains(r#"rel="noopener noreferrer""#), "{html}");
}

#[test]
fn a_save_toggle_reports_its_state_as_aria_pressed_and_does_not_pulse_on_first_render() {
    let saved = render(|| rsx! { SaveToggle { saved: true, ontoggle: |_| {}, "Saved" } });
    let unsaved = render(|| rsx! { SaveToggle { saved: false, ontoggle: |_| {}, "Save" } });
    assert!(saved.contains(r#"aria-pressed="true""#), "{saved}");
    assert!(unsaved.contains(r#"aria-pressed="false""#), "{unsaved}");
    assert!(!saved.contains("rv2-save--pulse"), "{saved}");
}

#[test]
fn chips_sit_in_a_named_group_and_expose_selection_as_aria_pressed() {
    let html = render(|| {
        rsx! {
            ChipGroup { label: "Filter activities",
                Chip { active: true, "All" }
                Chip { "Indoor" }
            }
        }
    });
    assert!(html.contains(r#"role="group""#), "{html}");
    assert!(html.contains(r#"aria-label="Filter activities""#), "{html}");
    assert_eq!(html.matches(r#"aria-pressed="true""#).count(), 1, "{html}");
    assert_eq!(html.matches(r#"aria-pressed="false""#).count(), 1, "{html}");
}

/// Two links per card would double every keyboard user's tab stops.
#[test]
fn a_media_card_puts_one_link_in_the_tab_order() {
    let html = render(|| {
        rsx! {
            MediaCard { href: "/a", image: "/a.svg", title: "Sun catcher", "Body" }
        }
    });
    assert_eq!(html.matches(r#"href="/a""#).count(), 2, "{html}");
    assert_eq!(html.matches(r#"tabindex="-1""#).count(), 1, "{html}");
    assert!(html.contains("<h3"), "{html}");
}

#[test]
fn a_masthead_marks_only_the_current_route() {
    let html = render(|| {
        rsx! {
            Masthead {
                brand: "Desk",
                items: vec![NavLink::route("a", "A", "/a"), NavLink::route("b", "B", "/b")],
                current: "b",
            }
        }
    });
    assert_eq!(html.matches(r#"aria-current="page""#).count(), 1, "{html}");
}
