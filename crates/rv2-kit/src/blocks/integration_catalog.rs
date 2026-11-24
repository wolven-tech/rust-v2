//! Integration catalog: find a third-party tool, read what it does, and connect it. Connected
//! tools show a status badge; disconnecting asks for confirmation inside the card first.
//!
//! Design choice: the connect flow belongs to the caller. The card reports intent and shows
//! the pending state until `connected` flips or `error` is set.
//! Design choice: a filter matches an integration's badges. A filter with no matching
//! integration in the category shows the empty message, not a hidden control.
//! Design choice: the logo is an `Element` supplied by the caller, so the catalog stays free of
//! brand artwork.
//! Not implemented: the pointer hover reveal of disconnect; disconnect is always visible as a
//! text control, which also covers touch.
//! Browser checks: search narrows and the count matches, connect shows pending on one card and
//! keeps its width, disconnect confirmation, Escape clears a search, empty state and clear
//! action, phone chip row for categories with no sideways scroll.

use dioxus::prelude::*;
use rv2_affect::badge::{Badge, BadgeTone};
use rv2_affect::button::{Button, ButtonSize, ButtonVariant};

use crate::internal::{KIT_CSS, class_list};

const CSS: Asset = asset!("/assets/integration-catalog.css");

/// Cards shown per category before the "Discover more" cell; a 3-column grid of 8 cards plus
/// the cell fills three rows.
pub const VISIBLE_LIMIT: usize = 8;

pub const FEATURED: &str = "featured";
pub const ALL: &str = "all";

#[derive(Clone, Debug)]
pub struct Integration {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub description: String,
    pub category_ids: Vec<String>,
    pub connected: bool,
    pub logo: Element,
    pub badges: Vec<String>,
    pub error: Option<String>,
}

impl PartialEq for Integration {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.connected == other.connected
            && self.error == other.error
            && self.name == other.name
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CategoryOption {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilterOption {
    pub id: String,
    pub label: String,
}

/// Number of connected integrations.
pub fn connected_count(items: &[Integration]) -> usize {
    items.iter().filter(|i| i.connected).count()
}

fn in_category(item: &Integration, category: &str, featured: &[String]) -> bool {
    match category {
        ALL => true,
        FEATURED => featured.contains(&item.id),
        other => item.category_ids.iter().any(|c| c == other),
    }
}

fn matches_search(item: &Integration, search: &str) -> bool {
    let needle = search.trim().to_lowercase();
    needle.is_empty()
        || item.name.to_lowercase().contains(&needle)
        || item.vendor.to_lowercase().contains(&needle)
        || item.description.to_lowercase().contains(&needle)
}

/// Integrations for a category that match the search and carry every active filter badge
/// in `filters` (any of them, when several are active).
pub fn filter_integrations<'a>(
    items: &'a [Integration],
    category: &str,
    featured: &[String],
    search: &str,
    filters: &[String],
) -> Vec<&'a Integration> {
    items
        .iter()
        .filter(|i| in_category(i, category, featured))
        .filter(|i| matches_search(i, search))
        .filter(|i| filters.is_empty() || i.badges.iter().any(|b| filters.contains(b)))
        .collect()
}

/// Items in a category before the search and filters, for the category count.
pub fn category_total(items: &[Integration], category: &str, featured: &[String]) -> usize {
    items
        .iter()
        .filter(|i| in_category(i, category, featured))
        .count()
}

#[component]
fn IntegrationCard(
    item: Integration,
    pending: bool,
    any_pending: bool,
    on_connect: Option<EventHandler<String>>,
    on_disconnect: Option<EventHandler<String>>,
    disabled: bool,
) -> Element {
    let mut confirming = use_signal(|| false);
    let id = item.id.clone();
    let connect_id = id.clone();
    let disconnect_id = id.clone();
    let name = item.name.clone();
    let inert = disabled || any_pending;
    let connecting_text = if pending {
        format!("Connecting {name}")
    } else {
        String::new()
    };
    rsx! {
        article { class: "kit-catalog__card", "aria-label": "{name}", "data-connected": item.connected.then_some("true"),
            header { class: "kit-catalog__card-head",
                div { class: "kit-catalog__logo", "aria-hidden": "true", {item.logo.clone()} }
                div { class: "kit-catalog__identity",
                    h4 { class: "kit-catalog__name",
                        "{item.name} "
                        span { class: "kit-catalog__verified", "aria-label": "Verified", "✓" }
                    }
                    p { class: "kit-catalog__vendor", "By {item.vendor}" }
                }
            }
            p { class: "kit-catalog__desc", "{item.description}" }
            if let Some(error) = item.error.clone() {
                p { class: "kit-catalog__error", role: "alert", "{error}" }
            }
            if !item.badges.is_empty() {
                div { class: "kit-catalog__tags",
                    for badge in item.badges.iter() {
                        span { key: "{badge}", class: "kit-catalog__tag", "{badge}" }
                    }
                }
            }
            footer { class: "kit-catalog__actions",
                if item.connected {
                    Badge { tone: BadgeTone::Success, dot: true, "Connected" }
                    if confirming() {
                        div { class: "kit-catalog__confirm", role: "group", "aria-label": "Confirm disconnect {name}",
                            span { "Disconnect {name}?" }
                            button {
                                r#type: "button",
                                class: "kit-catalog__btn kit-catalog__btn--danger",
                                disabled: inert,
                                onclick: move |_| {
                                    confirming.set(false);
                                    if let Some(handler) = on_disconnect { handler.call(disconnect_id.clone()); }
                                },
                                "Disconnect"
                            }
                            button {
                                r#type: "button",
                                class: "kit-catalog__btn",
                                onclick: move |_| confirming.set(false),
                                "Keep"
                            }
                        }
                    } else {
                        button {
                            r#type: "button",
                            class: "kit-catalog__btn kit-catalog__disconnect",
                            disabled: inert,
                            "aria-label": "Disconnect {name}",
                            onclick: move |_| confirming.set(true),
                            "Disconnect"
                        }
                    }
                } else {
                    Button {
                        variant: ButtonVariant::Secondary,
                        size: ButtonSize::Sm,
                        class: "kit-catalog__connect",
                        label: if pending { "Connecting…".to_string() } else { "Connect".to_string() },
                        icon: pending.then(|| rsx! { span { class: "kit-catalog__spinner", "aria-hidden": "true" } }),
                        disabled: inert,
                        onclick: move |_| {
                            if let Some(handler) = on_connect { handler.call(connect_id.clone()); }
                        },
                    }
                    span { class: "kit-sr", role: "status", "{connecting_text}" }
                }
            }
        }
    }
}

#[component]
pub fn IntegrationCatalog(
    #[props(default = "Integrations".to_string(), into)] title: String,
    #[props(default = "Connect the tools your work already lives in.".to_string(), into)]
    intro: String,
    integrations: Vec<Integration>,
    categories: Vec<CategoryOption>,
    #[props(default)] featured_ids: Vec<String>,
    #[props(default)] pending_id: Option<String>,
    #[props(default)] on_connect: Option<EventHandler<String>>,
    #[props(default)] on_disconnect: Option<EventHandler<String>>,
    #[props(default)] filter_options: Vec<FilterOption>,
    #[props(default)] active_filter_ids: Vec<String>,
    #[props(default)] on_filter_change: Option<EventHandler<Vec<String>>>,
    #[props(default)] search_text: String,
    #[props(default)] on_search_change: Option<EventHandler<String>>,
    #[props(default)] disabled: bool,
    #[props(default)] read_more_href: Option<String>,
    #[props(default, into)] extra_classes: String,
) -> Element {
    let mut category = use_signal(|| FEATURED.to_string());
    let mut filter_open = use_signal(|| false);
    let root_class = class_list("kit-catalog", &extra_classes);
    let active = category();
    let matches = filter_integrations(
        &integrations,
        &active,
        &featured_ids,
        &search_text,
        &active_filter_ids,
    );
    let total = category_total(&integrations, &active, &featured_ids);
    let connected = connected_count(&integrations);
    let shown: Vec<Integration> = matches
        .iter()
        .take(VISIBLE_LIMIT)
        .map(|i| (*i).clone())
        .collect();
    let outside = integrations.len().saturating_sub(shown.len());
    let heading = match active.as_str() {
        FEATURED => "Essentials".to_string(),
        ALL => "All apps".to_string(),
        other => categories
            .iter()
            .find(|c| c.id == other)
            .map(|c| c.label.clone())
            .unwrap_or_default(),
    };
    let result_text = if search_text.trim().is_empty() {
        matches.len().to_string()
    } else {
        format!("{} matching “{}”", matches.len(), search_text.trim())
    };
    let mut category_items = vec![
        CategoryOption {
            id: FEATURED.to_string(),
            label: "Essentials".to_string(),
        },
        CategoryOption {
            id: ALL.to_string(),
            label: "All apps".to_string(),
        },
    ];
    category_items.extend(categories.iter().cloned());
    let any_pending = pending_id.is_some();

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "{root_class}", "aria-labelledby": "kit-catalog-title",
            header { class: "kit-catalog__head",
                div {
                    h2 { id: "kit-catalog-title", class: "kit-catalog__title", "{title}" }
                    p { class: "kit-catalog__intro", "{intro}"
                        if let Some(href) = read_more_href.clone() {
                            a { class: "kit-catalog__more-link", href: "{href}", " Read more" }
                        }
                    }
                }
                p { class: "kit-catalog__connected", "aria-live": "polite", "{connected} connected" }
            }
            div { class: "kit-catalog__search",
                input {
                    class: "kit-catalog__input",
                    r#type: "search",
                    "aria-label": "Search integrations",
                    placeholder: "Search integrations",
                    value: "{search_text}",
                    oninput: move |e: FormEvent| {
                        if let Some(handler) = on_search_change { handler.call(e.value()); }
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Escape && !search_text.is_empty() {
                            e.prevent_default();
                            if let Some(handler) = on_search_change { handler.call(String::new()); }
                        }
                    },
                }
                if !filter_options.is_empty() {
                    div { class: "kit-catalog__filter",
                        button {
                            r#type: "button",
                            class: "kit-catalog__btn",
                            "aria-haspopup": "listbox",
                            "aria-expanded": if filter_open() { "true" } else { "false" },
                            onclick: move |_| filter_open.set(!filter_open()),
                            "Filter"
                            if !active_filter_ids.is_empty() {
                                span { class: "kit-catalog__count", " {active_filter_ids.len()}" }
                            }
                        }
                        if filter_open() {
                            ul { class: "kit-catalog__menu", role: "listbox", "aria-label": "Filters", "aria-multiselectable": "true",
                                for option in filter_options.iter() {
                                    {
                                        let checked = active_filter_ids.contains(&option.id);
                                        let oid = option.id.clone();
                                        let current = active_filter_ids.clone();
                                        rsx! {
                                            li { key: "{option.id}", role: "option", "aria-selected": if checked { "true" } else { "false" },
                                                onclick: move |_| {
                                                    let mut next = current.clone();
                                                    match next.iter().position(|f| *f == oid) {
                                                        Some(at) => { next.remove(at); }
                                                        None => next.push(oid.clone()),
                                                    }
                                                    if let Some(handler) = on_filter_change { handler.call(next); }
                                                },
                                                span { class: "kit-catalog__check", "aria-hidden": "true", if checked { "✓" } else { "" } }
                                                "{option.label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "kit-catalog__body",
                nav { class: "kit-catalog__categories", "aria-label": "Categories",
                    for option in category_items.iter() {
                        {
                            let cid = option.id.clone();
                            let selected = option.id == active;
                            let count = category_total(&integrations, &option.id, &featured_ids);
                            rsx! {
                                button {
                                    key: "{option.id}",
                                    r#type: "button",
                                    class: "kit-catalog__category",
                                    "aria-pressed": if selected { "true" } else { "false" },
                                    onclick: move |_| category.set(cid.clone()),
                                    span { "{option.label}" }
                                    span { class: "kit-catalog__category-count", "{count}" }
                                }
                            }
                        }
                    }
                }
                section { class: "kit-catalog__main", "aria-labelledby": "kit-catalog-heading",
                    h3 { id: "kit-catalog-heading", class: "kit-catalog__heading",
                        "{heading} "
                        span { class: "kit-catalog__count", "aria-live": "polite", "{result_text}" }
                    }
                    if matches.is_empty() {
                        div { class: "kit-catalog__empty",
                            p { if total == 0 { "Nothing in this category yet." } else { "No integrations match this search." } }
                            if !search_text.is_empty() {
                                button {
                                    r#type: "button",
                                    class: "kit-catalog__btn",
                                    onclick: move |_| {
                                        if let Some(handler) = on_search_change { handler.call(String::new()); }
                                    },
                                    "Clear search"
                                }
                            }
                        }
                    } else {
                        div { class: "kit-catalog__grid",
                            for item in shown.iter().cloned() {
                                {
                                    let pending = pending_id.as_deref() == Some(item.id.as_str());
                                    rsx! {
                                        IntegrationCard {
                                            key: "{item.id}",
                                            item: item,
                                            pending: pending,
                                            any_pending: any_pending,
                                            on_connect: on_connect,
                                            on_disconnect: on_disconnect,
                                            disabled: disabled,
                                        }
                                    }
                                }
                            }
                            if active.as_str() != ALL && outside > 0 {
                                div { class: "kit-catalog__more",
                                    p { class: "kit-catalog__more-title", "Discover more" }
                                    p { class: "kit-catalog__more-count", "+{outside} apps" }
                                    button {
                                        r#type: "button",
                                        class: "kit-catalog__btn",
                                        onclick: move |_| category.set(ALL.to_string()),
                                        "View all"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
