//! SSR and stylesheet checks for the company register at rest (amendment 5).

use dioxus::prelude::*;
use rv2_kit::screens::company_register::{
    Account, CompanyRecord, CompanyRegister, NavItem, NavSection,
};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn record(
    id: &str,
    name: &str,
    category: &str,
    domain: &str,
    stage: &str,
    value: f64,
    contact: &str,
) -> CompanyRecord {
    CompanyRecord {
        id: id.into(),
        company_name: name.into(),
        company_category: category.into(),
        website_domain: domain.into(),
        account_owner: "Ada".into(),
        deal_value: value,
        pipeline_stage: stage.into(),
        primary_contact: "Sam Reyes".into(),
        last_contact: contact.into(),
        logo_url: None,
    }
}

fn nav() -> Vec<NavSection> {
    let item = |label: &str, count: Option<usize>| NavItem {
        label: label.into(),
        count,
    };
    vec![
        NavSection {
            label: String::new(),
            items: vec![
                item("Home", Some(4)),
                item("Companies", Some(10)),
                item("Contacts", Some(23)),
                item("Reports", Some(2)),
            ],
        },
        NavSection {
            label: "Pipelines".into(),
            items: vec![
                item("New business", None),
                item("Expansion", None),
                item("Partners", None),
                item("Renewals", None),
            ],
        },
    ]
}

fn rest_app() -> Element {
    rsx! {
        CompanyRegister {
            account: Account { company_name: "Juniper Works".into(), avatar_url: None },
            sections: nav(),
            company_records: vec![
                record("acme", "Acme Labs", "Software", "acme.example", "Proposal", 12400.0, "Nov 18"),
                record("birch", "Birch & Co", "Retail", "birch.example", "Renewal due", 0.0, "Nov 04"),
                record("cobalt", "Cobalt Health", "Health", "cobalt.example", "Won", 38000.0, "Oct 29"),
                record("delta", "Delta Freight", "Logistics", "delta.example", "Qualified", 7600.0, "Nov 21"),
                record("ember", "Ember Foods", "Food", "ember.example", "Lead", 5200.0, "Oct 14"),
                record("fjord", "Fjord Design", "Creative", "fjord.example", "Proposal", 9800.0, "Nov 12"),
                record("granite", "Granite Mining", "Industrial", "granite.example", "Won", 61000.0, "Sep 30"),
                record("harbor", "Harbor Legal", "Services", "harbor.example", "Renewal due", 0.0, "Nov 09"),
                record("iris", "Iris Analytics", "Software", "iris.example", "Qualified", 14300.0, "Nov 15"),
                record("juno", "Juno Retail", "Retail", "juno.example", "Lead", 22700.0, "Oct 22"),
            ],
        }
    }
}

/// Text of each column header, in order, read from the table's `th` cells.
fn header_labels(html: &str) -> Vec<String> {
    html.split("class=\"kit-sg__label\"")
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

#[test]
fn table_has_seven_headed_columns_in_order_and_ten_rows() {
    let html = render(rest_app);
    assert_eq!(
        header_labels(&html),
        vec![
            "Company",
            "Category",
            "Domain",
            "Owner",
            "Value",
            "Stage",
            "Last contact"
        ],
        "{html}"
    );
    assert_eq!(html.matches("aria-rowindex=").count(), 10, "{html}");
}

#[test]
fn sidebar_lists_four_navigation_items_with_counts_and_four_pipelines() {
    let html = render(rest_app);
    assert_eq!(
        html.matches("class=\"kit-cr__nav-item\"").count(),
        8,
        "{html}"
    );
    assert_eq!(html.matches("class=\"kit-cr__count\"").count(), 4, "{html}");
    assert!(html.contains("Pipelines"), "{html}");
    for label in ["New business", "Expansion", "Partners", "Renewals"] {
        assert!(html.contains(label), "{label}: {html}");
    }
}

#[test]
fn header_is_one_breadcrumb_line_and_the_toolbar_names_both_add_controls() {
    let html = render(rest_app);
    assert_eq!(
        html.matches("class=\"kit-cr__crumbs\"").count(),
        1,
        "{html}"
    );
    assert!(html.contains("Double-click a cell to edit"), "{html}");
    assert!(html.contains("Add row"), "{html}");
    assert!(html.contains("class=\"kit-sg__add\""), "{html}");
}

#[test]
fn view_tabs_show_four_views_and_the_add_control() {
    let html = render(rest_app);
    assert_eq!(html.matches("role=\"tab\"").count(), 4, "{html}");
    assert!(html.contains("aria-label=\"New view\""), "{html}");
    assert!(html.contains("aria-selected=\"true\""), "{html}");
}

#[test]
fn no_cell_is_in_edit_mode_at_rest() {
    let html = render(rest_app);
    assert!(!html.contains("kit-sg__editor"), "{html}");
    assert!(!html.contains("kit-sg__cell\" data-active"), "{html}");
}

#[test]
fn the_grid_fits_its_box_and_the_header_row_stays_at_the_top() {
    let css = include_str!("../assets/company-register.css");
    assert!(css.contains("table-layout: fixed;"), "{css}");
    assert!(css.contains("position: sticky;\n  top: 0;"), "{css}");
    assert!(css.contains("text-overflow: ellipsis;"), "{css}");
    let wrap = css
        .split(".kit-cr__table-wrap {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("table wrap rule");
    assert!(wrap.contains("flex: 1 1 auto;"), "{wrap}");
    assert!(wrap.contains("min-height: 0;"), "{wrap}");
}
