use std::collections::BTreeMap;

use dioxus::prelude::*;
use rv2_kit::components::sheet_grid::{Column, ColumnKind, GridRow, SheetGrid, TagTone, Tone};

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn columns() -> Vec<Column> {
    vec![
        Column::new("title", "Tool", ColumnKind::Title),
        Column {
            tag_choices: ["Core", "Trial"].map(String::from).to_vec(),
            ..Column::new("tag", "Status", ColumnKind::Tag)
        },
        Column::new("link", "Site", ColumnKind::Link),
        Column::new("renews", "Renews", ColumnKind::Text),
    ]
}

fn rows() -> Vec<GridRow> {
    [
        (
            "ledger",
            "Ledger Pro",
            "Core",
            "ledgerpro.example",
            "2027-03-01",
        ),
        (
            "mailroom",
            "Mailroom",
            "Trial",
            "mailroom.example",
            "2027-01-15",
        ),
        (
            "pixel",
            "Pixel Tools",
            "Core",
            "pixeltools.example",
            "2026-12-09",
        ),
        (
            "atlas",
            "Atlas Notes",
            "Core",
            "atlas.example",
            "2027-06-30",
        ),
        (
            "forms",
            "Old Forms",
            "Trial",
            "oldforms.example",
            "2026-11-30",
        ),
    ]
    .into_iter()
    .map(|(id, title, tag, site, renews)| GridRow {
        row_id: id.into(),
        row_logo: None,
        values: BTreeMap::from([
            ("title".to_string(), title.to_string()),
            ("tag".to_string(), tag.to_string()),
            ("link".to_string(), site.to_string()),
            ("renews".to_string(), renews.to_string()),
        ]),
    })
    .collect()
}

fn tones() -> Vec<TagTone> {
    vec![
        TagTone {
            tag: "Core".into(),
            tone: Tone::Positive,
        },
        TagTone {
            tag: "Trial".into(),
            tone: Tone::Caution,
        },
    ]
}

fn card(read_only: bool) -> String {
    let app: fn() -> Element = if read_only {
        || {
            rsx! {
                SheetGrid {
                    grid_heading: "Tool renewals",
                    grid_columns: columns(),
                    grid_rows: rows(),
                    tag_tones: tones(),
                    read_only: true,
                }
            }
        }
    } else {
        || {
            rsx! {
                SheetGrid {
                    grid_heading: "Tool renewals",
                    grid_columns: columns(),
                    grid_rows: rows(),
                    tag_tones: tones(),
                }
            }
        }
    };
    render(app)
}

#[test]
fn card_stacks_title_row_then_grid_then_footer() {
    let html = card(false);
    let title = html.find("kit-sg__title").expect("title row");
    let scroll = html.find("kit-sg__scroll").expect("grid");
    let footer = html.find("kit-sg__foot").expect("footer");
    assert!(title < scroll && scroll < footer);
}

#[test]
fn header_row_leads_with_a_checkbox_then_sorts_title_tag_link_text() {
    let html = card(false);
    assert_eq!(html.matches("class=\"kit-sg__head-cell\"").count(), 4);
    let first_head = html.find("class=\"kit-sg__head-cell\"").expect("header");
    let heads = &html[first_head..];
    let order: Vec<usize> = ["Tool", "Status", "Site", "Renews"]
        .iter()
        .map(|label| heads.find(label).expect("head label"))
        .collect();
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(html.find("Select all rows").expect("select all") < first_head);
}

#[test]
fn body_shows_five_rows_with_a_logo_tile_chip_host_and_date_text() {
    let html = card(false);
    assert_eq!(html.matches("aria-rowindex").count(), 5);
    assert_eq!(html.matches("class=\"kit-sg__tile\"").count(), 5);
    assert_eq!(html.matches("class=\"kit-sg__chip\"").count(), 5);
    assert!(html.contains("data-tone=\"positive\""));
    assert!(html.contains("ledgerpro.example"));
    assert!(html.contains("2027-03-01"));
}

#[test]
fn footer_holds_add_row_then_count_and_editing_hint() {
    let html = card(false);
    let add = html.find("Add row").expect("add row");
    let count = html.find("5 rows").expect("count");
    let hint = html.find("Double-click a cell to edit").expect("hint");
    assert!(add < count && count < hint);
}

#[test]
fn rest_state_selects_nothing_edits_nothing_and_sorts_nothing() {
    let html = card(false);
    assert!(!html.contains("data-selected"));
    assert!(!html.contains("data-active"));
    assert!(!html.contains("kit-sg__editor"));
    assert!(!html.contains("aria-sort=\"ascending\""));
    assert!(!html.contains("aria-sort=\"descending\""));
}

#[test]
fn read_only_grid_drops_add_row_and_the_editing_hint_but_keeps_checkboxes() {
    let html = card(true);
    assert!(!html.contains("Add row"));
    assert!(!html.contains("Double-click a cell to edit"));
    assert!(html.contains("class=\"kit-sg__check\""));
    assert!(html.contains("5 rows"));
}

#[test]
fn sheet_grid_stylesheet_names_no_literal_colour_in_its_rules() {
    let css = include_str!("../assets/sheet-grid.css");
    assert!(!css.contains('#'));
    assert!(!css.contains("rgb("));
}
