//! Company register: one editable table of companies, read through four views (company list, deals, contacts,
//! renewals) with a tab strip, a sidebar of navigation and pipelines, and an account block.
//!
//! Design choice: the table is `SheetGrid`, which owns keyboard cell moves, the in-place editor, copy and paste,
//! and undo. A view switch swaps rows in place; "Add row" focuses the new company cell through `focus_request`.
//! Design choice: `on_cell_change` receives `(row id, column key, new value)`, not only deal values, because the
//! spec's edits cover every column; the host persists the record.
//! Design choice: the "+" tab control calls `on_new_view`, a prop the spec does not list.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::components::sheet_grid::{Cell, Column, ColumnKind, GridRow, SheetGrid};
use crate::internal::*;

const CSS: Asset = asset!("/assets/company-register.css");

/// The four views the register offers, in tab order.
pub const DEFAULT_VIEWS: [&str; 4] = ["Company CRM", "Deals", "Contacts", "Renewals"];

/// Stage choices for the stage column when the host gives none.
pub const DEFAULT_STAGES: [&str; 6] = [
    "Lead",
    "Qualified",
    "Proposal",
    "Won",
    "Renewal due",
    "Lost",
];

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub company_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompanyRecord {
    pub id: String,
    pub company_name: String,
    pub company_category: String,
    pub website_domain: String,
    pub account_owner: String,
    pub deal_value: f64,
    pub pipeline_stage: String,
    pub primary_contact: String,
    pub last_contact: String,
    pub logo_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavItem {
    pub label: String,
    pub count: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavSection {
    pub label: String,
    pub items: Vec<NavItem>,
}

/// Column keys a view shows, in order.
pub fn view_columns(view: &str) -> Vec<&'static str> {
    match view {
        "Deals" => vec!["company", "deal_value", "stage", "owner", "last_contact"],
        "Contacts" => vec!["company", "contact", "domain", "owner", "last_contact"],
        "Renewals" => vec!["company", "stage", "last_contact", "deal_value"],
        _ => vec![
            "company",
            "category",
            "domain",
            "owner",
            "deal_value",
            "stage",
            "last_contact",
        ],
    }
}

/// Whether a record belongs to a view: deals have a value, renewals are at a renewal stage.
pub fn view_admits(view: &str, record: &CompanyRecord) -> bool {
    match view {
        "Deals" => record.deal_value > 0.0,
        "Renewals" => record.pipeline_stage.starts_with("Renewal"),
        _ => true,
    }
}

/// Heading of a view's empty state.
pub fn empty_title(view: &str) -> String {
    match view {
        "Deals" => "No deals yet".into(),
        "Contacts" => "No contacts yet".into(),
        "Renewals" => "No renewals yet".into(),
        _ => "No companies yet".into(),
    }
}

fn column_for(key: &str, stages: &[String]) -> Column {
    let label = match key {
        "company" => "Company",
        "category" => "Category",
        "domain" => "Domain",
        "owner" => "Owner",
        "deal_value" => "Value",
        "stage" => "Stage",
        "contact" => "Primary contact",
        _ => "Last contact",
    };
    let kind = match key {
        "company" => ColumnKind::Title,
        "domain" => ColumnKind::Link,
        "deal_value" => ColumnKind::Number,
        "stage" => ColumnKind::Tag,
        _ => ColumnKind::Text,
    };
    Column {
        tag_choices: if key == "stage" {
            stages.to_vec()
        } else {
            Vec::new()
        },
        ..Column::new(key, label, kind)
    }
}

/// Grid rows for a view. Each row carries the cell text its column reads.
pub fn view_rows(records: &[CompanyRecord], view: &str) -> Vec<GridRow> {
    records
        .iter()
        .filter(|r| view_admits(view, r))
        .map(|r| {
            let mut values = BTreeMap::new();
            values.insert("company".to_string(), r.company_name.clone());
            values.insert("category".to_string(), r.company_category.clone());
            values.insert("domain".to_string(), r.website_domain.clone());
            values.insert("owner".to_string(), r.account_owner.clone());
            values.insert("deal_value".to_string(), format!("{:.0}", r.deal_value));
            values.insert("stage".to_string(), r.pipeline_stage.clone());
            values.insert("contact".to_string(), r.primary_contact.clone());
            values.insert("last_contact".to_string(), r.last_contact.clone());
            GridRow {
                row_id: r.id.clone(),
                row_logo: r.logo_url.clone(),
                values,
            }
        })
        .collect()
}

/// The first cell that differs between two row sets, as `(row id, column key, new value)`.
/// Rows added or removed do not count here; the caller handles those.
pub fn changed_cell(old: &[GridRow], new: &[GridRow]) -> Option<(String, String, String)> {
    if old.len() != new.len() {
        return None;
    }
    for (before, after) in old.iter().zip(new.iter()) {
        if before.row_id != after.row_id {
            return None;
        }
        for (key, value) in &after.values {
            if before.values.get(key) != Some(value) {
                return Some((after.row_id.clone(), key.clone(), value.clone()));
            }
        }
    }
    None
}

/// A blank row for the Add row action; the host replaces its id on save.
pub fn blank_row(id: &str) -> GridRow {
    GridRow {
        row_id: id.to_string(),
        row_logo: None,
        values: BTreeMap::new(),
    }
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn default_nav() -> Vec<NavSection> {
    let item = |label: &str| NavItem {
        label: label.into(),
        count: None,
    };
    vec![
        NavSection {
            label: String::new(),
            items: vec![
                item("Home"),
                item("Companies"),
                item("Contacts"),
                item("Reports"),
            ],
        },
        NavSection {
            label: "Pipelines".into(),
            items: vec![item("New business"), item("Renewals")],
        },
    ]
}

/// Company register page: sidebar, header, view tabs, toolbar and the editable table.
#[component]
pub fn CompanyRegister(
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    account: Account,
    #[props(default)] sections: Vec<NavSection>,
    #[props(default)] views: Vec<String>,
    #[props(default)] stage_choices: Vec<String>,
    #[props(default)] company_records: Vec<CompanyRecord>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_add: EventHandler<String>,
    #[props(default)] on_new_view: EventHandler<()>,
    #[props(default)] on_navigate: EventHandler<String>,
    #[props(default)] on_cell_change: EventHandler<(String, String, String)>,
) -> Element {
    let base = use_hook(|| format!("kit-cr-{}", next_id()));
    let view_names: Vec<String> = if views.is_empty() {
        DEFAULT_VIEWS.iter().map(|v| v.to_string()).collect()
    } else {
        views
    };
    let stages: Vec<String> = if stage_choices.is_empty() {
        DEFAULT_STAGES.iter().map(|s| s.to_string()).collect()
    } else {
        stage_choices
    };
    let nav = if sections.is_empty() {
        default_nav()
    } else {
        sections
    };
    let mut active_view = use_signal(|| view_names.first().cloned().unwrap_or_default());
    let mut edits = use_signal(BTreeMap::<String, Vec<GridRow>>::new);
    let mut generation = use_signal(|| 0_usize);
    let mut announce = use_signal(String::new);
    let mut nav_open = use_signal(|| false);

    let view = active_view();
    let current_rows: Vec<GridRow> = edits()
        .get(&view)
        .cloned()
        .unwrap_or_else(|| view_rows(&company_records, &view));
    let columns: Vec<Column> = view_columns(&view)
        .into_iter()
        .map(|k| column_for(k, &stages))
        .collect();
    let headings: Vec<String> = columns.iter().map(|c| c.column_label.clone()).collect();
    let empty_view = is_empty || current_rows.is_empty();
    let mut focus_cell = use_signal(|| None::<Cell>);
    let nav_class = if nav_open() {
        "kit-cr__nav is-open"
    } else {
        "kit-cr__nav"
    };
    let tab_ids: Vec<String> = (0..view_names.len())
        .map(|i| format!("{base}-tab-{i}"))
        .collect();
    let row_total = current_rows.len();
    let initials_text = initials(&account.company_name);

    let records_for_rows = company_records.clone();
    let view_for_rows = view.clone();
    let on_rows = move |next: Vec<GridRow>| {
        let before = edits()
            .get(&view_for_rows)
            .cloned()
            .unwrap_or_else(|| view_rows(&records_for_rows, &view_for_rows));
        if let Some((id, column, value)) = changed_cell(&before, &next) {
            announce.set(format!("Saved {column} for {id}"));
            on_cell_change.call((id, column, value));
        }
        edits.with_mut(|m| {
            m.insert(view_for_rows.clone(), next);
        });
    };

    let tab_ids_for_keys = tab_ids.clone();
    let names_for_keys = view_names.clone();
    let on_tabs_keys = move |event: KeyboardEvent| {
        let current = names_for_keys
            .iter()
            .position(|v| *v == active_view())
            .unwrap_or(0);
        let last = tab_ids_for_keys.len().saturating_sub(1);
        match event.key() {
            Key::ArrowRight => {
                event.prevent_default();
                focus_after_frame(&tab_ids_for_keys[(current + 1).min(last)]);
            }
            Key::ArrowLeft => {
                event.prevent_default();
                focus_after_frame(&tab_ids_for_keys[current.saturating_sub(1)]);
            }
            _ => {}
        }
    };

    let records_for_add = company_records.clone();
    let view_for_add = view.clone();
    let add_row = move |_| {
        let id = format!("new-{}", generation() + 1);
        let mut next = edits()
            .get(&view_for_add)
            .cloned()
            .unwrap_or_else(|| view_rows(&records_for_add, &view_for_add));
        focus_cell.set(Some(Cell {
            row: next.len(),
            col: 0,
        }));
        next.push(blank_row(&id));
        edits.with_mut(|m| {
            m.insert(view_for_add.clone(), next);
        });
        generation.set(generation() + 1);
        announce.set("Row added".to_string());
        on_add.call(view_for_add.clone());
    };

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-cr", "aria-labelledby": "{base}-title",
            button { class: "kit-cr__menu-btn", r#type: "button", "aria-expanded": "{nav_open()}", "aria-controls": "{base}-nav",
                onclick: move |_| nav_open.toggle(),
                "Menu"
            }
            nav { id: "{base}-nav", class: "{nav_class}", "aria-label": "Sidebar",
                p { class: "kit-cr__ws", "{workspace_name}" }
                for (si, section) in nav.iter().enumerate() {
                    div { key: "{si}", class: "kit-cr__section",
                        if !section.label.is_empty() {
                            p { class: "kit-cr__section-label", "{section.label}" }
                        }
                        ul { class: "kit-cr__nav-list",
                            for (ii, item) in section.items.iter().enumerate() {
                                li { key: "{ii}",
                                    button { class: "kit-cr__nav-item", r#type: "button",
                                        onclick: {
                                            let label = item.label.clone();
                                            move |_| {
                                                nav_open.set(false);
                                                on_navigate.call(label.clone());
                                            }
                                        },
                                        span { "{item.label}" }
                                        if let Some(n) = item.count {
                                            span { class: "kit-cr__count", "{n}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "kit-cr__account",
                    span { class: "kit-cr__avatar", "aria-hidden": "true",
                        if let Some(src) = account.avatar_url.clone() {
                            img { src: "{src}", alt: "" }
                        } else {
                            "{initials_text}"
                        }
                    }
                    span { class: "kit-cr__account-name", "{account.company_name}" }
                }
            }

            div { class: "kit-cr__page",
                nav { class: "kit-cr__crumbs", "aria-label": "Breadcrumb",
                    ol {
                        li { "CRM" }
                        li { "aria-current": "page", "{view}" }
                    }
                }
                div { class: "kit-cr__tabs-row",
                    div { class: "kit-cr__tabs", role: "tablist", "aria-label": "Views", onkeydown: on_tabs_keys,
                        for (i, label) in view_names.iter().enumerate() {
                            button { key: "{i}", id: "{tab_ids[i]}", class: "kit-cr__tab", r#type: "button", role: "tab",
                                "aria-selected": if *label == view { "true" } else { "false" },
                                tabindex: if *label == view { "0" } else { "-1" },
                                onclick: {
                                    let label = label.clone();
                                    move |_| active_view.set(label.clone())
                                },
                                "{label}"
                            }
                        }
                    }
                    button { class: "kit-cr__tab kit-cr__tab--add", r#type: "button", "aria-label": "New view",
                        onclick: move |_| on_new_view.call(()),
                        "+"
                    }
                }

                div { class: "kit-cr__toolbar",
                    button { class: "kit-cr__btn kit-cr__btn--primary", r#type: "button", disabled: is_loading,
                        onclick: add_row,
                        "Add row"
                    }
                    p { class: "kit-cr__hint", "Double-click a cell to edit" }
                    span { class: "kit-cr__total", "{row_total} rows" }
                }

                if is_loading {
                    div { class: "kit-cr__table-wrap", "aria-busy": "true",
                        table { class: "kit-cr__ph-table",
                            thead {
                                tr {
                                    for (i, heading) in headings.iter().enumerate() {
                                        th { key: "{i}", scope: "col", "{heading}" }
                                    }
                                }
                            }
                            tbody {
                                for r in 0..4 {
                                    tr { key: "{r}",
                                        for c in 0..headings.len() {
                                            td { key: "{c}", span { class: "kit-cr__ph" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if empty_view {
                    div { class: "kit-cr__empty",
                        p { class: "kit-cr__empty-title", "{empty_title(&view)}" }
                        p { class: "kit-cr__empty-text", "Add the first record to start this view." }
                        button { class: "kit-cr__btn kit-cr__btn--primary", r#type: "button",
                            onclick: {
                                let view = view.clone();
                                move |_| on_add.call(view.clone())
                            },
                            "Add a record"
                        }
                    }
                } else {
                    div { class: "kit-cr__table-wrap",
                        SheetGrid {
                            grid_heading: "{view}",
                            grid_columns: columns,
                            grid_rows: current_rows,
                            on_rows_change: on_rows,
                            focus_request: Some(focus_cell),
                        }
                    }
                }
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
