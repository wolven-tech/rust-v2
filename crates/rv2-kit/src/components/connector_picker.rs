//! Connector picker: a searchable list of services, grouped Connected then Available, with a detail
//! pane for the picked one and a connect or explore action.
//!
//! Design choice: the list is uncontrolled; `on_connection_change` reports each switch. Up and Down
//! walk the connected group and then the available group as one sequence.
//!
//! Browser checks: the pulse ring is not shown under reduced motion; the detail stacks below the
//! list on a narrow container and the key hints drop; Cmd/Ctrl+/ focuses the search.

use dioxus::prelude::*;
use rv2_affect::switch::Switch;

use crate::internal::{KIT_CSS, class_list, next_id};

/// Next position in a list of `len`, clamped at the ends. A `None` current starts at the first (down) or last (up).
fn step_index(current: Option<usize>, len: usize, dir: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match current {
        None if dir > 0 => 0,
        None => len - 1,
        Some(index) => (index as isize + dir).clamp(0, len as isize - 1) as usize,
    })
}

const CSS: Asset = asset!("/assets/connector-picker.css");

#[derive(Clone, Debug, PartialEq)]
pub struct Connector {
    pub connector_id: String,
    pub connector_name: String,
    pub connector_domain: String,
    pub connector_category: String,
    pub connector_summary: String,
    pub connector_features: Vec<String>,
    pub connector_mark: Option<Element>,
    pub is_connected: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActionMode {
    #[default]
    Explore,
    Connect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConnectorLabels {
    pub title: Option<String>,
    pub search_placeholder: Option<String>,
    pub explore: Option<String>,
    pub connect: Option<String>,
    pub disconnect: Option<String>,
}

/// Ten generic services, three of them connected.
pub fn sample_connectors() -> Vec<Connector> {
    let names = [
        ("store", "Storefront", "Commerce"),
        ("ads", "Ad account", "Advertising"),
        ("feed", "Data feed", "Analytics"),
        ("mail", "Mailer", "Messaging"),
        ("crm", "Contacts", "CRM"),
        ("pay", "Payments", "Finance"),
        ("docs", "Documents", "Storage"),
        ("chat", "Team chat", "Messaging"),
        ("sheet", "Spreadsheet", "Data"),
        ("calendar", "Calendar", "Scheduling"),
    ];
    names
        .iter()
        .enumerate()
        .map(|(index, (id, name, category))| Connector {
            connector_id: id.to_string(),
            connector_name: name.to_string(),
            connector_domain: format!("{id}.example"),
            connector_category: category.to_string(),
            connector_summary: format!("Connect {name} to read and write its records."),
            connector_features: vec!["Two-way sync".to_string(), "Scheduled refresh".to_string()],
            connector_mark: None,
            is_connected: index < 3,
        })
        .collect()
}

pub fn connected_count(list: &[Connector]) -> usize {
    list.iter().filter(|c| c.is_connected).count()
}

/// `3 of 10 connected`.
pub fn count_text(list: &[Connector]) -> String {
    format!("{} of {} connected", connected_count(list), list.len())
}

/// Indices of the connected and available connectors that match `query`, each in list order.
pub fn grouped_matches(list: &[Connector], query: &str) -> (Vec<usize>, Vec<usize>) {
    let needle = query.trim().to_lowercase();
    let hit = |c: &Connector| {
        needle.is_empty()
            || c.connector_name.to_lowercase().contains(&needle)
            || c.connector_category.to_lowercase().contains(&needle)
    };
    let connected = (0..list.len())
        .filter(|&i| list[i].is_connected && hit(&list[i]))
        .collect();
    let available = (0..list.len())
        .filter(|&i| !list[i].is_connected && hit(&list[i]))
        .collect();
    (connected, available)
}

/// `"Product + Service"`, or the labelled title when one is given.
pub fn header_title(product: &str, picked: &str, labels: &ConnectorLabels) -> String {
    labels
        .title
        .clone()
        .unwrap_or_else(|| format!("{product} + {picked}"))
}

/// The action text for the detail pane.
pub fn action_text(mode: ActionMode, is_connected: bool, labels: &ConnectorLabels) -> String {
    match mode {
        ActionMode::Explore => labels
            .explore
            .clone()
            .unwrap_or_else(|| "Explore".to_string()),
        ActionMode::Connect if is_connected => labels
            .disconnect
            .clone()
            .unwrap_or_else(|| "Disconnect".to_string()),
        ActionMode::Connect => labels
            .connect
            .clone()
            .unwrap_or_else(|| "Connect".to_string()),
    }
}

/// Flips the connection of the connector with `id`; returns the new state.
pub fn toggle_connection(list: &mut [Connector], id: &str) -> Option<bool> {
    let connector = list.iter_mut().find(|c| c.connector_id == id)?;
    connector.is_connected = !connector.is_connected;
    Some(connector.is_connected)
}

#[component]
pub fn ConnectorPicker(
    #[props(default)] connector_list: Option<Vec<Connector>>,
    #[props(default = "Product".to_string(), into)] product_name: String,
    #[props(default)] product_mark: Option<Element>,
    #[props(default)] initial_selection_id: Option<String>,
    #[props(default)] on_connection_change: EventHandler<(String, bool)>,
    #[props(default)] on_explore: EventHandler<String>,
    #[props(default)] on_back: Option<EventHandler<()>>,
    #[props(default)] on_close: Option<EventHandler<()>>,
    #[props(default)] label_text: ConnectorLabels,
    #[props(default)] action_mode: ActionMode,
    #[props(default, into)] extra_class: String,
) -> Element {
    let id = use_hook(|| format!("kit-cp-{}", next_id()));
    let mut list = use_signal(|| connector_list.clone().unwrap_or_else(sample_connectors));
    let mut picked = use_signal(|| {
        initial_selection_id
            .clone()
            .or_else(|| list().first().map(|c| c.connector_id.clone()))
            .unwrap_or_default()
    });
    let mut query = use_signal(String::new);
    let mut detail_pane = use_signal(|| false);

    let all = list();
    let (connected, available) = grouped_matches(&all, &query());
    let order: Vec<usize> = connected.iter().chain(available.iter()).copied().collect();
    let picked_id = picked();
    let picked_index = all.iter().position(|c| c.connector_id == picked_id);
    let detail = picked_index.map(|i| all[i].clone());
    let title = header_title(
        &product_name,
        detail.as_ref().map_or("", |c| c.connector_name.as_str()),
        &label_text,
    );
    let count = count_text(&all);
    let action = detail
        .as_ref()
        .map(|c| action_text(action_mode, c.is_connected, &label_text))
        .unwrap_or_default();
    let search_placeholder = label_text
        .search_placeholder
        .clone()
        .unwrap_or_else(|| "Search services".to_string());
    let active_position = order.iter().position(|i| Some(*i) == picked_index);
    let search_id = format!("{id}-search");
    let mut sections: Vec<(&str, Vec<usize>)> = vec![
        ("Connected", connected.clone()),
        ("Available", available.clone()),
    ];
    sections.retain(|(_, members)| !members.is_empty());
    let ids_snapshot = all.clone();
    let show_switch = action_mode == ActionMode::Explore;

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section {
            class: class_list("kit-cp", &extra_class),
            "aria-label": "{title}",
            "data-pane": if detail_pane() { "detail" } else { "list" },
            onkeydown: {
                let search = search_id.clone();
                move |event: KeyboardEvent| {
                    let mods = event.modifiers();
                    if (mods.ctrl() || mods.meta()) && event.key() == Key::Character("/".into()) {
                        event.prevent_default();
                        document::eval(&format!("document.getElementById({search:?})?.focus();"));
                    }
                }
            },
            header { class: "kit-cp__head",
                if let Some(back) = on_back {
                    button {
                        r#type: "button",
                        class: "kit-cp__back",
                        "aria-label": "Back",
                        onclick: move |_| back.call(()),
                        "‹"
                    }
                }
                h3 { class: "kit-cp__title", "{title}" }
                input {
                    id: "{search_id}",
                    class: "kit-cp__search",
                    role: "combobox",
                    r#type: "search",
                    "aria-label": "{search_placeholder}",
                    "aria-expanded": "true",
                    "aria-controls": "{id}-list",
                    placeholder: "{search_placeholder}",
                    value: "{query}",
                    oninput: move |event| query.set(event.value()),
                    onkeydown: {
                        let order_keys = order.clone();
                        let ids_keys = ids_snapshot.clone();
                        move |event: KeyboardEvent| {
                            let key = event.key();
                            let current = active_position;
                            match key {
                                Key::ArrowDown | Key::ArrowUp => {
                                    event.prevent_default();
                                    let dir = if key == Key::ArrowDown { 1 } else { -1 };
                                    if let Some(next) = step_index(current, order_keys.len(), dir) {
                                        picked.set(ids_keys[order_keys[next]].connector_id.clone());
                                    }
                                }
                                Key::Enter => {
                                    event.prevent_default();
                                    if let Some(index) = picked_index {
                                        let target = ids_keys[index].connector_id.clone();
                                        let mut next = list();
                                        if let Some(state) = toggle_connection(&mut next, &target) {
                                            list.set(next);
                                            on_connection_change.call((target, state));
                                        }
                                    }
                                }
                                Key::ArrowLeft if query().is_empty() => {
                                    if let Some(back) = on_back {
                                        back.call(());
                                    }
                                }
                                Key::Escape => {
                                    if query().is_empty() {
                                        if let Some(close) = on_close {
                                            close.call(());
                                        }
                                    } else {
                                        query.set(String::new());
                                    }
                                }
                                _ => {}
                            }
                        }
                    },
                }
            }
            div { class: "kit-cp__body",
                div { id: "{id}-list", class: "kit-cp__list", role: "listbox", "aria-label": "Services",
                    for (name, members) in sections.iter() {
                        div { key: "{name}", class: "kit-cp__group", role: "group", "aria-label": "{name}",
                            span { class: "kit-cp__group-title", "{name}" }
                            for index in members.iter().copied() {
                                {
                                    let row = all[index].clone();
                                    let row_id = row.connector_id.clone();
                                    let is_picked = row.connector_id == picked_id;
                                    let mark_row = row.connector_mark.clone();
                                    let letter = row.connector_name.chars().next().map(|c| c.to_string()).unwrap_or_default();
                                    rsx! {
                                        div {
                                            key: "{row_id}",
                                            class: "kit-cp__row",
                                            role: "option",
                                            "aria-selected": if is_picked { "true" } else { "false" },
                                            "data-connected": row.is_connected.then_some("true"),
                                            onclick: {
                                                let search = search_id.clone();
                                                move |_| {
                                                    picked.set(row_id.clone());
                                                    detail_pane.set(true);
                                                    document::eval(&format!("document.getElementById({search:?})?.focus();"));
                                                }
                                            },
                                            span { class: "kit-cp__mark", "aria-hidden": "true",
                                                if let Some(mark) = mark_row { {mark} } else { "{letter}" }
                                            }
                                            span { class: "kit-cp__row-name", "{row.connector_name}" }
                                            span { class: "kit-cp__row-category", "{row.connector_category}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if sections.is_empty() {
                        p { class: "kit-cp__none", "No matches" }
                    }
                }
                div { class: "kit-sr", role: "status",
                    if sections.is_empty() {
                        "No matches"
                    } else if let Some(item) = detail.as_ref() {
                        "{item.connector_name}, "
                        if item.is_connected { "connected" } else { "not connected" }
                    }
                }
                if let Some(item) = detail.clone() {
                    div { class: "kit-cp__detail",
                        button {
                            r#type: "button",
                            class: "kit-cp__pane-back",
                            onclick: move |_| detail_pane.set(false),
                            "‹ Services"
                        }
                        div { class: "kit-cp__detail-head",
                            span { class: "kit-cp__detail-mark", "aria-hidden": "true",
                                if let Some(mark) = item.connector_mark.clone() { {mark} } else { {item.connector_name.chars().take(1).collect::<String>()} }
                            }
                            div { class: "kit-cp__detail-text",
                                h4 { class: "kit-cp__detail-name", "{item.connector_name}" }
                                span { class: "kit-cp__domain", "{item.connector_domain}" }
                            }
                            if show_switch {
                                Switch {
                                    label: rsx! { "Connected" },
                                    checked: item.is_connected,
                                    onchange: {
                                        let target = item.connector_id.clone();
                                        move |_| {
                                            let mut next = list();
                                            if let Some(state) = toggle_connection(&mut next, &target) {
                                                list.set(next);
                                                on_connection_change.call((target.clone(), state));
                                            }
                                        }
                                    },
                                }
                            }
                        }
                        p { class: "kit-cp__status", if item.is_connected { "Connected" } else { "Not connected" } }
                        p { class: "kit-cp__summary", "{item.connector_summary}" }
                        ul { class: "kit-cp__features",
                            for feature in item.connector_features.iter() {
                                li { "{feature}" }
                            }
                        }
                        div { class: "kit-cp__actions",
                            if show_switch {
                                button {
                                    r#type: "button",
                                    class: "kit-cp__action",
                                    onclick: {
                                        let target = item.connector_id.clone();
                                        move |_| on_explore.call(target.clone())
                                    },
                                    "{action}"
                                }
                            } else {
                                button {
                                    r#type: "button",
                                    class: "kit-cp__action",
                                    "data-variant": "primary",
                                    onclick: {
                                        let target = item.connector_id.clone();
                                        move |_| {
                                            let mut next = list();
                                            if let Some(state) = toggle_connection(&mut next, &target) {
                                                list.set(next);
                                                on_connection_change.call((target.clone(), state));
                                            }
                                        }
                                    },
                                    "{action}"
                                }
                            }
                        }
                    }
                }
            }
            footer { class: "kit-cp__foot",
                span { class: "kit-cp__count", "{count}" }
                span { class: "kit-cp__hints", "↑↓ move · Enter switch · Esc close" }
            }
        }
    }
}
