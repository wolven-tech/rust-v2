//! Connected apps catalogue: a directory of external services the assistant can connect to, filtered by tab and by
//! search, with a request form for services that are not listed.
//!
//! Design choice: connect and disconnect change the row in place at once, so the screen keeps its own connection
//! overrides on top of `connected`; the host still receives `on_connect` and `on_disconnect` and owns the truth.
//! Design choice: "Connecting" resolves after `CONNECT_MS` in the browser, standing in for the host's round trip.
//! Design choice: the sidebar is drawn inline so the screen stands alone, without the workspace frame.

use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::internal::*;

const CSS: Asset = asset!("/assets/connected-apps-catalogue.css");

/// How long the connecting state shows before it resolves, in milliseconds.
pub const CONNECT_MS: u32 = 650;

#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub name: String,
    pub avatar_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct App {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub category: String,
    pub connected: bool,
    pub popular: bool,
    pub logo_url: Option<String>,
    pub bundle: Vec<String>,
}

/// The top tabs. Request is an action that opens the request form rather than a filter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    All,
    Connected,
    Available,
    Request,
    Popular,
}

impl Tab {
    pub const ALL: [Self; 5] = [
        Self::All,
        Self::Connected,
        Self::Available,
        Self::Request,
        Self::Popular,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Connected => "Connected",
            Self::Available => "Available",
            Self::Request => "Request",
            Self::Popular => "Popular",
        }
    }

    /// Whether a row with this connection and popularity appears under the tab.
    pub fn admits(self, connected: bool, popular: bool) -> bool {
        match self {
            Self::All | Self::Request => true,
            Self::Connected => connected,
            Self::Available => !connected,
            Self::Popular => popular,
        }
    }
}

/// Connection state after the person's own changes on the screen override the host's flag.
pub fn effective_connected(app: &App, overrides: &BTreeMap<String, bool>) -> bool {
    overrides.get(&app.id).copied().unwrap_or(app.connected)
}

/// Apps a tab and a search admit, in their given order. Search matches anywhere in the name, ignoring case.
pub fn visible_apps<'a>(
    apps: &'a [App],
    tab: Tab,
    query: &str,
    overrides: &BTreeMap<String, bool>,
) -> Vec<&'a App> {
    let needle = query.trim().to_lowercase();
    apps.iter()
        .filter(|app| tab.admits(effective_connected(app, overrides), app.popular))
        .filter(|app| needle.is_empty() || app.name.to_lowercase().contains(&needle))
        .collect()
}

/// Count of apps a tab admits, for its label.
pub fn tab_count(apps: &[App], tab: Tab, overrides: &BTreeMap<String, bool>) -> usize {
    apps.iter()
        .filter(|app| tab.admits(effective_connected(app, overrides), app.popular))
        .count()
}

/// Whether the request form may submit: a name is required.
pub fn can_submit(name: &str) -> bool {
    !name.trim().is_empty()
}

fn initial_of(name: &str) -> String {
    name.chars()
        .next()
        .map(|c| c.to_uppercase().collect::<String>())
        .unwrap_or_default()
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[derive(Clone, PartialEq)]
struct RowView {
    id: String,
    name: String,
    summary: String,
    initial: String,
    logo_url: Option<String>,
    bundle_text: Option<String>,
    connected: bool,
}

/// Connected apps catalogue: sticky header with tabs and search, category sections of app rows, and a request form.
#[component]
pub fn ConnectedAppsCatalogue(
    #[props(default = "Juniper Works".to_string(), into)] workspace_name: String,
    account: Account,
    #[props(default)] apps: Vec<App>,
    #[props(default)] categories: Vec<String>,
    #[props(default)] is_loading: bool,
    #[props(default)] is_empty: bool,
    #[props(default)] on_connect: EventHandler<String>,
    #[props(default)] on_disconnect: EventHandler<String>,
    #[props(default)] on_request: EventHandler<(String, String)>,
    #[props(default)] on_open_app: EventHandler<String>,
) -> Element {
    let base = use_hook(|| format!("kit-cac-{}", next_id()));
    let mut tab = use_signal(Tab::default);
    let mut query = use_signal(String::new);
    let mut overrides = use_signal(BTreeMap::<String, bool>::new);
    let mut connecting = use_signal(Vec::<String>::new);
    let mut menu_for = use_signal(|| None::<String>);
    let mut confirm_for = use_signal(|| None::<String>);
    let mut request_open = use_signal(|| false);
    let mut request_name = use_signal(String::new);
    let mut request_note = use_signal(String::new);
    let mut announce = use_signal(String::new);

    let current_tab = tab();
    let overrides_now = overrides();
    let visible: Vec<App> = visible_apps(&apps, current_tab, &query(), &overrides_now)
        .into_iter()
        .cloned()
        .collect();
    let mut section_names: Vec<String> = if categories.is_empty() {
        let mut names: Vec<String> = Vec::new();
        for app in &apps {
            if !names.contains(&app.category) {
                names.push(app.category.clone());
            }
        }
        names
    } else {
        categories.clone()
    };
    section_names.retain(|c| visible.iter().any(|a| a.category == *c));
    let sections: Vec<(String, Vec<RowView>)> = section_names
        .iter()
        .map(|cat| {
            let rows = visible
                .iter()
                .filter(|a| a.category == *cat)
                .map(|a| RowView {
                    id: a.id.clone(),
                    name: a.name.clone(),
                    summary: a.summary.clone(),
                    initial: initial_of(&a.name),
                    logo_url: a.logo_url.clone(),
                    bundle_text: (!a.bundle.is_empty()).then(|| a.bundle.join(", ")),
                    connected: effective_connected(a, &overrides_now),
                })
                .collect();
            (cat.clone(), rows)
        })
        .collect();
    let searching = !query().trim().is_empty();
    let no_match = !is_loading && !is_empty && visible.is_empty();
    let account_initials = initials(&account.name);
    let tab_ids: Vec<String> = (0..Tab::ALL.len())
        .map(|i| format!("{base}-tab-{i}"))
        .collect();
    let tab_ids_for_keys = tab_ids.clone();
    let counts: Vec<(Tab, usize)> = Tab::ALL
        .iter()
        .map(|t| (*t, tab_count(&apps, *t, &overrides_now)))
        .collect();
    let submit_enabled = can_submit(&request_name());

    let on_tab_keys = move |event: KeyboardEvent| {
        let current = Tab::ALL.iter().position(|t| *t == tab()).unwrap_or(0);
        let last = Tab::ALL.len() - 1;
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

    rsx! {
        document::Link { rel: "stylesheet", href: KIT_CSS }
        document::Link { rel: "stylesheet", href: CSS }
        section { class: "kit-cac", "aria-labelledby": "{base}-title",
            nav { class: "kit-cac__nav", "aria-label": "Sidebar",
                p { class: "kit-cac__ws", "{workspace_name}" }
                ul { class: "kit-cac__nav-list",
                    for item in ["Home", "Inbox", "Integrations", "Assistant", "Settings"] {
                        li { key: "{item}",
                            span { class: "kit-cac__nav-item", "aria-current": if item == "Integrations" { "page" } else { "false" }, "{item}" }
                        }
                    }
                }
                div { class: "kit-cac__account",
                    span { class: "kit-cac__avatar", "aria-hidden": "true",
                        if let Some(src) = account.avatar_url.clone() {
                            img { src: "{src}", alt: "" }
                        } else {
                            "{account_initials}"
                        }
                    }
                    span { class: "kit-cac__account-name", "{account.name}" }
                }
            }

            div { class: "kit-cac__page",
                header { class: "kit-cac__head",
                    div { class: "kit-cac__headrow",
                        h1 { id: "{base}-title", class: "kit-cac__title", "Connected apps" }
                        button { class: "kit-cac__btn kit-cac__btn--primary", r#type: "button",
                            onclick: move |_| {
                                request_open.set(true);
                                tab.set(Tab::Request);
                            },
                            "Request integration"
                        }
                    }
                    div { class: "kit-cac__tabs", role: "tablist", "aria-label": "Filter apps", onkeydown: on_tab_keys,
                        for (i, (t, count)) in counts.iter().enumerate() {
                            button { key: "{i}", id: "{tab_ids[i]}", class: "kit-cac__tab", r#type: "button", role: "tab",
                                "aria-selected": if *t == current_tab { "true" } else { "false" },
                                tabindex: if *t == current_tab { "0" } else { "-1" },
                                onclick: {
                                    let t = *t;
                                    move |_| {
                                        tab.set(t);
                                        request_open.set(t == Tab::Request);
                                    }
                                },
                                "{t.label()}"
                                if *t != Tab::Request {
                                    span { class: "kit-cac__count", "{count}" }
                                }
                            }
                        }
                    }
                    label { class: "kit-cac__search",
                        span { class: "kit-sr", "Search apps" }
                        input { r#type: "search", class: "kit-cac__input", placeholder: "Search apps", value: "{query()}",
                            oninput: move |e| query.set(e.value()),
                            onkeydown: move |event: KeyboardEvent| {
                                if matches!(event.key(), Key::Escape) {
                                    query.set(String::new());
                                }
                            },
                        }
                    }
                }

                if request_open() {
                    form { class: "kit-cac__request", "aria-label": "Request an integration",
                        onsubmit: move |event: FormEvent| {
                            event.prevent_default();
                            if can_submit(&request_name()) {
                                on_request.call((request_name(), request_note()));
                                announce.set(format!("Request sent for {}", request_name()));
                                request_name.set(String::new());
                                request_note.set(String::new());
                                request_open.set(false);
                                tab.set(Tab::All);
                            }
                        },
                        onkeydown: move |event: KeyboardEvent| {
                            if matches!(event.key(), Key::Escape) {
                                event.prevent_default();
                                request_open.set(false);
                                tab.set(Tab::All);
                                request_name.set(String::new());
                                request_note.set(String::new());
                            }
                        },
                        label { class: "kit-cac__field",
                            span { "Integration name" }
                            input { class: "kit-cac__input", r#type: "text", name: "name", value: "{request_name()}",
                                oninput: move |e| request_name.set(e.value()),
                            }
                        }
                        label { class: "kit-cac__field",
                            span { "Note" }
                            textarea { class: "kit-cac__input", name: "note", rows: "2", value: "{request_note()}",
                                oninput: move |e| request_note.set(e.value()),
                            }
                        }
                        div { class: "kit-cac__form-actions",
                            button { class: "kit-cac__btn", r#type: "button",
                                onclick: move |_| {
                                    request_open.set(false);
                                    tab.set(Tab::All);
                                },
                                "Cancel"
                            }
                            button { class: "kit-cac__btn kit-cac__btn--primary", r#type: "submit", disabled: !submit_enabled, "Submit request" }
                        }
                    }
                }

                main { class: "kit-cac__list", "aria-busy": "{is_loading}",
                    if is_loading {
                        for s in 0..2 {
                            section { key: "{s}", class: "kit-cac__section",
                                span { class: "kit-cac__ph kit-cac__ph--head" }
                                for r in 0..3 {
                                    div { key: "{r}", class: "kit-cac__row kit-cac__row--ph",
                                        span { class: "kit-cac__ph kit-cac__ph--tile" }
                                        span { class: "kit-cac__ph kit-cac__ph--line" }
                                    }
                                }
                            }
                        }
                    } else if is_empty {
                        div { class: "kit-cac__empty",
                            p { class: "kit-cac__empty-title", "No integrations yet" }
                            p { class: "kit-cac__empty-text", "Services you connect appear here, grouped by what they do." }
                            button { class: "kit-cac__btn kit-cac__btn--primary", r#type: "button",
                                onclick: move |_| {
                                    request_open.set(true);
                                    tab.set(Tab::Request);
                                },
                                "Request integration"
                            }
                        }
                    } else if no_match {
                        div { class: "kit-cac__empty",
                            p { class: "kit-cac__empty-title", "No apps match" }
                            if searching {
                                button { class: "kit-cac__btn", r#type: "button", onclick: move |_| query.set(String::new()), "Clear search" }
                            }
                        }
                    } else {
                        for (si, (category, rows)) in sections.iter().enumerate() {
                            section { key: "{si}", class: "kit-cac__section", "aria-label": "{category}",
                                h2 { class: "kit-cac__section-title", "{category}" }
                                ul { class: "kit-cac__rows",
                                    for row in rows.iter() {
                                        li { key: "{row.id}", class: "kit-cac__row", role: "group", "aria-label": "{row.name}, {row.summary}",
                                            span { class: "kit-cac__tile", "aria-hidden": "true",
                                                if let Some(src) = row.logo_url.clone() {
                                                    img { src: "{src}", alt: "" }
                                                } else {
                                                    "{row.initial}"
                                                }
                                            }
                                            div { class: "kit-cac__body",
                                                button { class: "kit-cac__name", r#type: "button",
                                                    onclick: {
                                                        let id = row.id.clone();
                                                        move |_| on_open_app.call(id.clone())
                                                    },
                                                    "{row.name}"
                                                }
                                                span { class: "kit-cac__summary", "{row.summary}" }
                                                if let Some(bundle) = row.bundle_text.clone() {
                                                    span { class: "kit-cac__bundle", "Includes {bundle}" }
                                                }
                                            }
                                            div { class: "kit-cac__control",
                                                if connecting().contains(&row.id) {
                                                    span { class: "kit-cac__progress", role: "status", "Connecting…" }
                                                } else if row.connected {
                                                    span { class: "kit-cac__badge", "Connected" }
                                                    button { class: "kit-cac__btn", r#type: "button", "aria-haspopup": "menu",
                                                        "aria-expanded": if menu_for() == Some(row.id.clone()) { "true" } else { "false" },
                                                        "aria-label": "Connected: {row.name}, open menu",
                                                        onclick: {
                                                            let id = row.id.clone();
                                                            move |_| {
                                                                let next = if menu_for() == Some(id.clone()) { None } else { Some(id.clone()) };
                                                                menu_for.set(next);
                                                            }
                                                        },
                                                        "Menu"
                                                    }
                                                    if menu_for() == Some(row.id.clone()) {
                                                        div { class: "kit-cac__menu", role: "menu",
                                                            onkeydown: move |event: KeyboardEvent| {
                                                                if matches!(event.key(), Key::Escape) {
                                                                    menu_for.set(None);
                                                                    confirm_for.set(None);
                                                                }
                                                            },
                                                            if confirm_for() == Some(row.id.clone()) {
                                                                p { class: "kit-cac__confirm", "Disconnect {row.name}?" }
                                                                button { class: "kit-cac__btn kit-cac__btn--danger", r#type: "button", role: "menuitem",
                                                                    onclick: {
                                                                        let id = row.id.clone();
                                                                        move |_| {
                                                                            overrides.with_mut(|m| {
                                                                                m.insert(id.clone(), false);
                                                                            });
                                                                            on_disconnect.call(id.clone());
                                                                            announce.set(format!("Disconnected {id}"));
                                                                            confirm_for.set(None);
                                                                            menu_for.set(None);
                                                                        }
                                                                    },
                                                                    "Confirm disconnect"
                                                                }
                                                            } else {
                                                                button { class: "kit-cac__btn", r#type: "button", role: "menuitem",
                                                                    onclick: {
                                                                        let id = row.id.clone();
                                                                        move |_| confirm_for.set(Some(id.clone()))
                                                                    },
                                                                    "Disconnect"
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    button { class: "kit-cac__btn kit-cac__btn--primary", r#type: "button", "aria-label": "Connect {row.name}",
                                                        onclick: {
                                                            let id = row.id.clone();
                                                            move |_| {
                                                                connecting.with_mut(|c| c.push(id.clone()));
                                                                on_connect.call(id.clone());
                                                                announce.set(format!("Connecting {id}"));
                                                                let finish = id.clone();
                                                                spawn(async move {
                                                                    sleep(CONNECT_MS).await;
                                                                    connecting.with_mut(|c| c.retain(|x| *x != finish));
                                                                    overrides.with_mut(|m| {
                                                                        m.insert(finish.clone(), true);
                                                                    });
                                                                    announce.set(format!("Connected {finish}"));
                                                                });
                                                            }
                                                        },
                                                        "Connect"
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
            }
            p { class: "kit-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}
