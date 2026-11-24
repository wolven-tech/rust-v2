//! SSR and pure checks for the connected apps catalogue at rest (amendment 5).

use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::screens::connected_apps_catalogue::{
    App, ConnectedAppsCatalogue, Tab, tab_count, visible_apps,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn strip_comments(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => {
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

fn app(id: &str, name: &str, category: &str, summary: &str, connected: bool, popular: bool) -> App {
    App {
        id: id.into(),
        name: name.into(),
        summary: summary.into(),
        category: category.into(),
        connected,
        popular,
        logo_url: None,
        bundle: Vec::new(),
    }
}

const DOCS: &str = "Documents and design";
const COMMS: &str = "Communication";
const SCHED: &str = "Scheduling and analytics";
const DATA: &str = "Data and payments";

fn rest_apps() -> Vec<App> {
    let mut office = app(
        "office",
        "Office suite",
        DOCS,
        "Documents, sheets and slides.",
        false,
        false,
    );
    office.bundle = vec!["Word".into(), "Excel".into(), "PowerPoint".into()];
    let mut calendars = app(
        "calendars",
        "Calendars",
        SCHED,
        "Check availability across calendars.",
        false,
        false,
    );
    calendars.bundle = vec!["Google Calendar".into(), "Outlook Calendar".into()];
    vec![
        app(
            "slack",
            "Slack",
            COMMS,
            "Post updates and read team threads.",
            true,
            true,
        ),
        app(
            "drive",
            "Google Drive",
            DOCS,
            "Open and share team files.",
            true,
            true,
        ),
        app(
            "gmail",
            "Gmail",
            COMMS,
            "Read and send email from the assistant.",
            false,
            true,
        ),
        app(
            "stripe",
            "Stripe",
            DATA,
            "Read payments, refunds and payouts.",
            false,
            true,
        ),
        app(
            "notion",
            "Notion",
            DOCS,
            "Search and update pages and databases.",
            true,
            false,
        ),
        office,
        app(
            "figma",
            "Figma",
            DOCS,
            "Read frames and design comments.",
            false,
            false,
        ),
        app(
            "dropbox",
            "Dropbox",
            DOCS,
            "Find and open shared folders.",
            false,
            false,
        ),
        app(
            "zoom",
            "Zoom",
            COMMS,
            "Start meetings and read recordings.",
            true,
            false,
        ),
        app(
            "teams",
            "Microsoft Teams",
            COMMS,
            "Post to channels and read messages.",
            false,
            false,
        ),
        app(
            "intercom",
            "Intercom",
            COMMS,
            "Read support conversations.",
            false,
            false,
        ),
        app(
            "mailchimp",
            "Mailchimp",
            COMMS,
            "Read campaign results.",
            false,
            false,
        ),
        calendars,
        app(
            "looker",
            "Looker Studio",
            SCHED,
            "Read dashboards and report figures.",
            true,
            false,
        ),
        app(
            "xero",
            "Xero",
            DATA,
            "Read invoices and bank lines.",
            false,
            false,
        ),
        app(
            "plaid",
            "Plaid",
            DATA,
            "Read linked bank balances.",
            false,
            false,
        ),
    ]
}

fn categories() -> Vec<String> {
    [DOCS, COMMS, SCHED, DATA]
        .iter()
        .map(|c| c.to_string())
        .collect()
}

fn rest_app() -> Element {
    rsx! {
        ConnectedAppsCatalogue {
            apps: rest_apps(),
            categories: categories(),
        }
    }
}

/// The text of each section heading, in document order, without its count.
fn heading_texts(html: &str) -> Vec<String> {
    let clean = strip_comments(html);
    clean
        .split("class=\"kit-cac__section-title\"")
        .skip(1)
        .map(|piece| {
            let body = piece.trim_start_matches('>');
            body.split('<')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        })
        .collect()
}

/// The number each section heading carries, in document order.
fn section_counts(html: &str) -> Vec<usize> {
    let clean = strip_comments(html);
    clean
        .split("class=\"kit-cac__section-count\"")
        .skip(1)
        .map(|piece| {
            let body = piece.trim_start_matches('>');
            body.split('<')
                .next()
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0)
        })
        .collect()
}

/// The number each tab label carries, in document order; the Request tab has none.
fn tab_counts(html: &str) -> Vec<usize> {
    let clean = strip_comments(html);
    clean
        .split("class=\"kit-cac__count\"")
        .skip(1)
        .map(|piece| {
            let body = piece.trim_start_matches('>');
            body.split('<')
                .next()
                .unwrap_or_default()
                .trim()
                .parse()
                .unwrap_or(0)
        })
        .collect()
}

#[test]
fn rest_state_holds_sixteen_tiles_in_five_sections_with_popular_first() {
    let html = render(rest_app);
    assert_eq!(
        html.matches("class=\"kit-cac__tile\"").count(),
        16,
        "{html}"
    );
    assert_eq!(
        html.matches("class=\"kit-cac__section\"").count(),
        5,
        "{html}"
    );
    assert_eq!(
        heading_texts(&html),
        vec!["Popular", DOCS, COMMS, SCHED, DATA],
        "{html}"
    );
}

#[test]
fn each_section_count_matches_the_tiles_it_holds() {
    let html = render(rest_app);
    assert_eq!(section_counts(&html), vec![4, 4, 4, 2, 2], "{html}");
    let pieces: Vec<&str> = html.split("class=\"kit-cac__section\"").skip(1).collect();
    let tiles: Vec<usize> = pieces
        .iter()
        .map(|p| p.matches("class=\"kit-cac__tile\"").count())
        .collect();
    assert_eq!(tiles, vec![4, 4, 4, 2, 2], "{html}");
}

#[test]
fn popular_section_holds_only_popular_apps() {
    let html = render(rest_app);
    let rest = html
        .split("class=\"kit-cac__section\"")
        .nth(1)
        .expect("popular section");
    let first = rest
        .split("class=\"kit-cac__section\"")
        .next()
        .expect("popular section body");
    for name in ["Slack", "Google Drive", "Gmail", "Stripe"] {
        assert!(first.contains(name), "{name}: {html}");
    }
    assert!(!first.contains("Notion"), "{html}");
    assert!(!first.contains("Figma"), "{html}");
}

#[test]
fn tab_counts_read_all_connected_available_and_popular() {
    let html = render(rest_app);
    assert_eq!(tab_counts(&html), vec![16, 5, 11, 4], "{html}");
    assert!(html.contains(">All<"), "{html}");
    assert!(html.contains(">Request<"), "{html}");
}

#[test]
fn five_tiles_show_connected_and_each_has_a_menu() {
    let html = render(rest_app);
    assert_eq!(
        html.matches("class=\"kit-cac__badge\"").count(),
        5,
        "{html}"
    );
    assert_eq!(html.matches("aria-haspopup=\"menu\"").count(), 5, "{html}");
}

#[test]
fn bundle_tiles_list_their_services_after_the_summary() {
    let html = render(rest_app);
    assert_eq!(
        html.matches("class=\"kit-cac__bundle\"").count(),
        2,
        "{html}"
    );
    let summary = html.find("Documents, sheets and slides.").expect("summary");
    let bundle = html
        .find("Includes Word, Excel, PowerPoint")
        .expect("bundle");
    assert!(summary < bundle, "{html}");
}

#[test]
fn rest_state_has_no_request_form_and_no_search_text() {
    let html = render(rest_app);
    assert!(!html.contains("kit-cac__request"), "{html}");
    assert!(html.contains("placeholder=\"Search apps\""), "{html}");
}

#[test]
fn tab_counts_equal_the_tiles_each_tab_shows() {
    let apps = rest_apps();
    let none = BTreeMap::new();
    for tab in [Tab::All, Tab::Connected, Tab::Available, Tab::Popular] {
        assert_eq!(
            tab_count(&apps, tab, &none),
            visible_apps(&apps, tab, "", &none).len(),
            "{tab:?}"
        );
    }
}

#[test]
fn the_grid_is_two_columns_on_desktop_and_one_below_phone_width() {
    let css = include_str!("../assets/connected-apps-catalogue.css");
    assert!(css.contains("grid-template-columns: repeat(2, minmax(0, 1fr));"));
    let phone = css
        .split("@media (max-width: 480px)")
        .nth(1)
        .expect("phone breakpoint");
    assert!(phone.contains("grid-template-columns: minmax(0, 1fr);"));
}

#[test]
fn the_header_with_tabs_and_search_is_sticky() {
    let css = include_str!("../assets/connected-apps-catalogue.css");
    let head = css
        .split(".kit-cac__head {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("header rule");
    assert!(head.contains("position: sticky;"), "{head}");
    assert!(head.contains("top: 0;"), "{head}");
}
