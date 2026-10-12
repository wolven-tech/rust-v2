//! Searchable, controlled selection shared across product workspaces.

use dioxus::prelude::*;

const CSS: Asset = asset!("/assets/command-select.css");

/// Matches every search word against the visible label, retaining caller order.
pub fn command_matches(label: &str, query: &str) -> bool {
    let label = label.to_lowercase();
    query
        .split_whitespace()
        .all(|word| label.contains(&word.to_lowercase()))
}

/// Arrow navigation wraps; an empty list never produces an option index.
pub fn command_step(current: usize, count: usize, previous: bool) -> Option<usize> {
    if count == 0 {
        return None;
    }
    let current = current.min(count - 1);
    Some(if previous {
        (current + count - 1) % count
    } else {
        (current + 1) % count
    })
}

fn return_focus(trigger: Signal<Option<MountedEvent>>) {
    if let Some(node) = trigger() {
        spawn(async move {
            let _ = node.set_focus(true).await;
        });
    }
}

/// Controlled single selection. `id` must be unique within the page.
///
/// Options use stable `(value, label)` pairs. Filtering never changes the
/// selected value; only an explicit option selection calls `onchange`.
#[component]
pub fn CommandSelect(
    id: String,
    label: String,
    value: String,
    options: Vec<(String, String)>,
    onchange: EventHandler<String>,
    #[props(default)] name: Option<String>,
    #[props(default)] class: String,
    #[props(default = "Choose…".to_string())] placeholder: String,
    #[props(default = "Search options…".to_string())] search_placeholder: String,
    #[props(default = "No matching options".to_string())] empty_message: String,
    #[props(default)] disabled: bool,
) -> Element {
    let mut open = use_signal(|| false);
    let mut query = use_signal(String::new);
    let mut active = use_signal(|| 0usize);
    let mut trigger = use_signal(|| None::<MountedEvent>);
    let visible: Vec<_> = options
        .iter()
        .filter(|(_, label)| command_matches(label, &query()))
        .cloned()
        .collect();
    let selected_label = options
        .iter()
        .find(|(key, _)| key == &value)
        .map(|(_, text)| text.clone())
        .unwrap_or(placeholder);
    let current = active().min(visible.len().saturating_sub(1));
    let expanded = open() && !disabled;
    let active_id = (!visible.is_empty()).then(|| format!("{id}-option-{current}"));
    let search_options = visible.clone();
    let selected_value = value.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "rv2-command-select {class}",
            span { id: "{id}-label", class: "rv2-command-select__label", "{label}" }
            if let Some(name) = name { input { r#type: "hidden", name, value: "{value}", disabled } }
            button {
                id: "{id}", r#type: "button", class: "rv2-command-select__trigger",
                disabled, aria_haspopup: "dialog", aria_expanded: "{expanded}",
                aria_controls: expanded.then(|| format!("{id}-popup")),
                aria_labelledby: "{id}-label {id}-value",
                onmounted: move |node| trigger.set(Some(node)),
                onpointerdown: move |event| { if open() { event.prevent_default(); } },
                onclick: { let options = options.clone(); let value = value.clone(); move |_| {
                    query.set(String::new());
                    active.set(options.iter().position(|(key, _)| key == &value).unwrap_or(0));
                    let was_open = open();
                    open.set(!was_open);
                    if was_open { return_focus(trigger); }
                } },
                onkeydown: move |event| {
                    if matches!(event.key(), Key::ArrowDown | Key::ArrowUp) {
                        event.prevent_default();
                        query.set(String::new());
                        active.set(if event.key() == Key::ArrowUp { options.len().saturating_sub(1) } else { 0 });
                        open.set(true);
                    }
                },
                span { id: "{id}-value", class: "rv2-command-select__value", title: "{selected_label}", "{selected_label}" }
                svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8", "aria-hidden": "true",
                    path { d: "m7 10 5 5 5-5", stroke_linecap: "round", stroke_linejoin: "round" }
                }
            }
            if expanded {
                div { id: "{id}-popup", class: "rv2-command-select__popup", role: "dialog", aria_labelledby: "{id}-label",
                    input {
                        class: "rv2-command-select__search", r#type: "search", role: "combobox",
                        aria_label: "Search {label}", aria_expanded: "true", aria_autocomplete: "list",
                        aria_controls: "{id}-list", aria_activedescendant: active_id,
                        autocomplete: "off", placeholder: search_placeholder, value: "{query}",
                        onmounted: move |node| async move { let _ = node.set_focus(true).await; },
                        oninput: move |event| { query.set(event.value()); active.set(0); },
                        onblur: move |_| open.set(false),
                        onkeydown: move |event| {
                            if event.is_composing() { return; }
                            let key = event.key();
                            match key {
                                Key::ArrowDown | Key::ArrowUp => {
                                    event.prevent_default();
                                    if let Some(next) = command_step(current, search_options.len(), key == Key::ArrowUp) { active.set(next); }
                                }
                                Key::Home | Key::End if !search_options.is_empty() => {
                                    event.prevent_default();
                                    active.set(if key == Key::Home { 0 } else { search_options.len() - 1 });
                                }
                                Key::Enter => {
                                    event.prevent_default();
                                    if let Some((key, _)) = search_options.get(current) {
                                        let key = key.clone();
                                        open.set(false);
                                        return_focus(trigger);
                                        if key != selected_value { onchange.call(key); }
                                    }
                                }
                                Key::Escape => { event.prevent_default(); event.stop_propagation(); open.set(false); return_focus(trigger); }
                                _ => {}
                            }
                        },
                    }
                    div { id: "{id}-list", role: "listbox", aria_labelledby: "{id}-label", class: "rv2-command-select__list",
                        for (index, (key, text)) in visible.iter().enumerate() {
                            CommandOption {
                                key: "{key}", id: format!("{id}-option-{index}"), text: text.clone(),
                                highlighted: index == current, selected: key == &value,
                                onhover: move |_| active.set(index),
                                onselect: { let key = key.clone(); let value = value.clone(); move |_| {
                                    open.set(false); return_focus(trigger);
                                    if key != value { onchange.call(key.clone()); }
                                } },
                            }
                        }
                    }
                    p { class: "rv2-command-select__status", role: "status",
                        if visible.is_empty() { "{empty_message}" } else if visible.len() == 1 { "1 option" } else { "{visible.len()} options" }
                    }
                }
            }
        }
    }
}

#[component]
fn CommandOption(
    id: String,
    text: String,
    highlighted: bool,
    selected: bool,
    onselect: EventHandler<()>,
    onhover: EventHandler<()>,
) -> Element {
    let mut mounted = use_signal(|| None::<MountedEvent>);
    use_effect(use_reactive((&highlighted,), move |(highlighted,)| {
        if highlighted && let Some(node) = mounted() {
            spawn(async move {
                let _ = node
                    .scroll_to_with_options(ScrollToOptions {
                        behavior: ScrollBehavior::Instant,
                        vertical: ScrollLogicalPosition::Nearest,
                        horizontal: ScrollLogicalPosition::Nearest,
                    })
                    .await;
            });
        }
    }));
    rsx! {
        div { id, role: "option", aria_selected: "{selected}",
            class: "rv2-command-select__option", "data-active": "{highlighted}",
            onmounted: move |node| mounted.set(Some(node)),
            onpointerdown: move |event| event.prevent_default(),
            onmouseenter: move |_| onhover.call(()), onclick: move |_| onselect.call(()),
            span { "{text}" }
            if selected { span { aria_hidden: "true", "✓" } }
        }
    }
}
